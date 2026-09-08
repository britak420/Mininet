//! Persistent (file-backed) [`ReplayGuard`] (D-0366; beta blocker item 3 in
//! `docs/BETA_STATUS.md`, "persistent replay store").
//!
//! [`FileReplayGuard`] is the durable backend [`ReplayGuard`]'s own doc
//! comment has always called for: replay resistance that survives process
//! restarts, not just [`crate::InMemoryReplayGuard`]'s per-process memory.
//! Entries older than `retention_ms` are dropped on open (and can be swept
//! again later in a long-lived process via [`FileReplayGuard::prune`]) --
//! this bounds the file's growth the same way
//! [`crate::RangePolicy::max_age_ms`] already bounds how long any guard
//! needs to remember a sequence value (a durable guard need not outlive the
//! freshness window an attestation is itself checked against).
//!
//! ## Honest limits
//!
//! This is a flat append-only file with an `fsync` after each write, not a
//! write-ahead log or database. [`ReplayGuard::check_and_record`] (F-12,
//! D-0487) durably writes *before* accepting a nonce into memory, and
//! accepts it in memory only if that write actually succeeded --
//! [`FileReplayGuard::write_failures`] still exposes a running count of
//! failed durable writes for a caller that wants to notice degraded
//! durability, but a failed write is no longer silently forgotten: it now
//! also reports `false` ("not durably accepted") through the trait's own
//! infallible return value, so a crash or restart immediately after a
//! `check_and_record` failure can never resurrect a nonce this process
//! itself never durably remembered. The one gap this ordering cannot
//! close is a crash strictly *between* the `fsync` completing and this
//! function returning to its caller -- vanishingly narrow, and the
//! durable record itself is already correct by that point; a restart in
//! that exact window replays as "already recorded," never as "forgotten."
//! No cross-process file locking: two processes opening the same path
//! concurrently can race on the append -- unchanged by this fix, and a
//! real gap this type still does not close (see D-0487's own Failure
//! point). `open` tolerates exactly one kind of corruption -- a truncated
//! final line from a crash mid-write -- and is discarded silently; any
//! other malformed line is treated as real corruption and returned as an
//! error rather than silently dropped or, as a corrupted non-ASCII hex
//! field previously risked, panicking the process (F-12). A file over
//! [`MAX_REPLAY_GUARD_FILE_BYTES`] is refused before the eager read that
//! would otherwise allocate for it. [`FileReplayGuard::prune`] is never
//! called automatically -- no scheduler or background thread lives in
//! this crate; a caller that wants periodic garbage collection must
//! invoke it itself.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use did_mini::Did;

use crate::verify::ReplayGuard;

type Key = (String, [u8; 32]);

/// Hard cap on the durable file's size before [`FileReplayGuard::open`]
/// will read it (F-12): each record is well under 200 bytes, so this bounds
/// the eager `BufReader::lines().collect()` read to a sane allocation
/// rather than trusting an arbitrarily large file (a runaway process, disk
/// corruption merging files, or a hostile actor with write access to the
/// path) to size that allocation for us. `retention_ms`-driven pruning
/// keeps a healthy file far below this in ordinary operation; hitting the
/// cap is itself a signal something is wrong, not a routine occurrence.
const MAX_REPLAY_GUARD_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// A durable, file-backed [`ReplayGuard`]. Each entry remembers the time it
/// was recorded so [`FileReplayGuard::prune`] can sweep expired entries
/// without needing to reopen the file.
#[derive(Debug)]
pub struct FileReplayGuard {
    path: PathBuf,
    retention_ms: u64,
    seen: HashMap<Key, u64>,
    write_failures: u64,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn encode_line(device: &str, sequence: &[u8; 32], recorded_at_ms: u64) -> String {
    let mut hex = String::with_capacity(64);
    for byte in sequence {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("{device}\t{hex}\t{recorded_at_ms}\n")
}

/// Decode one `device\thex\trecorded_at_ms\n` line. Operates entirely on
/// `hex`'s raw bytes, never re-slicing the original `&str` by byte index
/// (F-12): a corrupted or hostile file can carry a non-ASCII, multi-byte
/// UTF-8 sequence in the hex field, and `str` indexing panics if a slice
/// boundary lands inside such a character instead of on one -- exactly the
/// kind of input a malformed/truncated/corrupted line can produce. Checking
/// every byte is an ASCII hex digit before ever converting anything closes
/// that panic path; malformed input becomes `None` (real corruption,
/// reported by the caller), never a crash.
fn decode_line(line: &str) -> Option<(String, [u8; 32], u64)> {
    let mut fields = line.split('\t');
    let device = fields.next()?.to_string();
    let hex = fields.next()?;
    let recorded_at_ms: u64 = fields.next()?.parse().ok()?;
    if fields.next().is_some() {
        return None;
    }
    let hex_bytes = hex.as_bytes();
    if hex_bytes.len() != 64 || !hex_bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut sequence = [0u8; 32];
    for (i, chunk) in sequence.iter_mut().enumerate() {
        let hi = (hex_bytes[i * 2] as char).to_digit(16)?;
        let lo = (hex_bytes[i * 2 + 1] as char).to_digit(16)?;
        *chunk = ((hi << 4) | lo) as u8;
    }
    Some((device, sequence, recorded_at_ms))
}

impl FileReplayGuard {
    /// Open (creating if absent) a durable replay guard backed by the file
    /// at `path`. Entries older than `retention_ms` (relative to the
    /// system clock at open time) are dropped and the file is compacted
    /// to reflect exactly the surviving entries.
    pub fn open(path: impl AsRef<Path>, retention_ms: u64) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut seen: HashMap<Key, u64> = HashMap::new();
        let mut dropped_any = false;

        if path.exists() {
            let file = File::open(&path)?;
            let size = file.metadata()?.len();
            if size > MAX_REPLAY_GUARD_FILE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "replay-guard file at {} is {size} bytes, over the {MAX_REPLAY_GUARD_FILE_BYTES}-byte cap -- refusing to read it eagerly",
                        path.display()
                    ),
                ));
            }
            let lines: Vec<String> = BufReader::new(file).lines().collect::<io::Result<_>>()?;
            let now = now_ms();
            let last_index = lines.len().checked_sub(1);
            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    continue;
                }
                match decode_line(line) {
                    Some((device, sequence, recorded_at_ms)) => {
                        if recorded_at_ms.saturating_add(retention_ms) <= now {
                            dropped_any = true;
                            continue;
                        }
                        seen.insert((device, sequence), recorded_at_ms);
                    }
                    None if Some(i) == last_index => {
                        // Tolerated: a crash mid-write can leave exactly the
                        // final line truncated. Discard it silently.
                        dropped_any = true;
                    }
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("corrupt replay-guard record at line {}", i + 1),
                        ));
                    }
                }
            }
        }

        let mut guard = FileReplayGuard {
            path,
            retention_ms,
            seen,
            write_failures: 0,
        };
        if dropped_any {
            guard.compact()?;
        }
        Ok(guard)
    }

    /// How many durable writes have failed since this guard was opened.
    /// Since F-12/D-0487, a failed durable write is *also* refused in
    /// memory (`check_and_record` returns `false` and does not accept the
    /// nonce) — this counter is for observability/alerting on degraded
    /// durability, not the only signal a failure occurred; see this
    /// module's own doc comment.
    pub fn write_failures(&self) -> u64 {
        self.write_failures
    }

    /// Sweep entries older than this guard's `retention_ms` (relative to
    /// the current system clock) out of memory and, if any were removed,
    /// out of the durable file too. Returns how many entries were removed.
    /// Never called automatically -- see this module's own doc comment.
    pub fn prune(&mut self) -> io::Result<usize> {
        let now = now_ms();
        let retention_ms = self.retention_ms;
        let before = self.seen.len();
        self.seen
            .retain(|_, recorded_at_ms| recorded_at_ms.saturating_add(retention_ms) > now);
        let removed = before - self.seen.len();
        if removed > 0 {
            self.compact()?;
        }
        Ok(removed)
    }

    /// Rewrite the file to contain exactly this guard's current in-memory
    /// entries, atomically (write to a temp file, `fsync`, then rename).
    fn compact(&mut self) -> io::Result<()> {
        let tmp_path = self.path.with_extension("tmp");
        let mut tmp = File::create(&tmp_path)?;
        for ((device, sequence), recorded_at_ms) in &self.seen {
            tmp.write_all(encode_line(device, sequence, *recorded_at_ms).as_bytes())?;
        }
        tmp.sync_all()?;
        fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }

    fn append_record(
        &mut self,
        device: &str,
        sequence: &[u8; 32],
        recorded_at_ms: u64,
    ) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(encode_line(device, sequence, recorded_at_ms).as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

impl ReplayGuard for FileReplayGuard {
    fn is_seen(&self, device: &Did, sequence: &[u8; 32]) -> bool {
        self.seen
            .contains_key(&(device.as_str().to_string(), *sequence))
    }

    fn check_and_record(&mut self, device: &Did, sequence: &[u8; 32]) -> bool {
        let key = (device.as_str().to_string(), *sequence);
        if self.seen.contains_key(&key) {
            return false;
        }
        let recorded_at_ms = now_ms();
        // F-12: commit durably *before* the in-memory acceptance, and only
        // accept in memory if the durable write actually succeeded. The
        // previous order accepted in memory unconditionally and only
        // recorded a write failure in a counter -- a crash/restart between
        // that acceptance and a successful durable write would forget the
        // acceptance entirely, letting the same nonce be replayed and
        // accepted again. Returning `false` here is not a lie about
        // whether this nonce was "seen before" in the trait's literal
        // sense; it reports the only fact this infallible interface can
        // carry that a caller can safely act on: this attempt did not
        // result in a durably remembered acceptance, so it must not be
        // treated as one.
        if self
            .append_record(&key.0, sequence, recorded_at_ms)
            .is_err()
        {
            self.write_failures += 1;
            return false;
        }
        self.seen.insert(key, recorded_at_ms);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::Controller;

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mini-presence-replay-guard-test-{name}-{}-{}",
            std::process::id(),
            now_ms()
        ));
        p
    }

    fn did() -> Did {
        Controller::incept_single().unwrap().did()
    }

    #[test]
    fn a_fresh_sequence_value_is_recorded_and_then_reported_seen() {
        let path = tmp_path("fresh");
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
        let d = did();
        let sequence = [7u8; 32];
        assert!(!guard.is_seen(&d, &sequence));
        assert!(guard.check_and_record(&d, &sequence));
        assert!(guard.is_seen(&d, &sequence));
        assert!(!guard.check_and_record(&d, &sequence));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_record_survives_reopening_the_same_path() {
        let path = tmp_path("survives-reopen");
        let d = did();
        let sequence = [9u8; 32];
        {
            let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
            assert!(guard.check_and_record(&d, &sequence));
        }
        // Simulate a process restart: a brand new guard over the same file.
        let guard = FileReplayGuard::open(&path, 60_000).unwrap();
        assert!(guard.is_seen(&d, &sequence));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn an_entry_older_than_retention_is_dropped_on_open() {
        let path = tmp_path("expired");
        let d = did();
        let sequence = [3u8; 32];
        let ancient_ms = 1u64;
        fs::write(&path, encode_line(d.as_str(), &sequence, ancient_ms)).unwrap();

        let guard = FileReplayGuard::open(&path, 1_000).unwrap();
        assert!(!guard.is_seen(&d, &sequence));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn opening_compacts_away_expired_entries_from_disk() {
        let path = tmp_path("compacts");
        let d = did();
        let stale_sequence = [4u8; 32];
        let fresh_sequence = [5u8; 32];
        let now = now_ms();
        let mut contents = String::new();
        contents.push_str(&encode_line(d.as_str(), &stale_sequence, 1));
        contents.push_str(&encode_line(d.as_str(), &fresh_sequence, now));
        fs::write(&path, contents).unwrap();

        let _guard = FileReplayGuard::open(&path, 60_000).unwrap();
        let on_disk = fs::read_to_string(&path).unwrap();
        assert!(!on_disk.contains(&{
            let mut h = String::new();
            for b in &stale_sequence {
                h.push_str(&format!("{b:02x}"));
            }
            h
        }));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_truncated_final_line_is_tolerated() {
        let path = tmp_path("truncated-tail");
        let d = did();
        let sequence = [6u8; 32];
        let mut contents = encode_line(d.as_str(), &sequence, now_ms());
        contents.push_str("not-a-complete-record-line");
        fs::write(&path, contents).unwrap();

        let guard = FileReplayGuard::open(&path, 60_000).unwrap();
        assert!(guard.is_seen(&d, &sequence));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_malformed_non_final_line_is_reported_as_corruption() {
        let path = tmp_path("corrupt-mid");
        let d = did();
        let sequence = [8u8; 32];
        let mut contents = String::from("garbage-not-a-record\n");
        contents.push_str(&encode_line(d.as_str(), &sequence, now_ms()));
        fs::write(&path, contents).unwrap();

        let err = FileReplayGuard::open(&path, 60_000).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn write_failures_starts_at_zero_and_stays_zero_on_success() {
        let path = tmp_path("write-failures");
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
        assert_eq!(guard.write_failures(), 0);
        guard.check_and_record(&did(), &[1u8; 32]);
        assert_eq!(guard.write_failures(), 0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn opening_a_nonexistent_path_starts_empty() {
        let path = tmp_path("nonexistent");
        let guard = FileReplayGuard::open(&path, 60_000).unwrap();
        assert!(!guard.is_seen(&did(), &[0u8; 32]));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn distinct_devices_with_the_same_sequence_value_are_tracked_independently() {
        let path = tmp_path("distinct-devices");
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
        let a = did();
        let b = did();
        let sequence = [2u8; 32];
        assert!(guard.check_and_record(&a, &sequence));
        assert!(guard.check_and_record(&b, &sequence));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn prune_removes_expired_entries_and_compacts_the_file() {
        let path = tmp_path("prune");
        let d = did();
        let sequence = [1u8; 32];
        // retention_ms = 0 means anything already recorded is immediately
        // prunable (its recorded_at_ms + 0 <= now for any now >= that
        // instant).
        let mut guard = FileReplayGuard::open(&path, 0).unwrap();
        guard.check_and_record(&d, &sequence);
        assert!(guard.is_seen(&d, &sequence));
        let removed = guard.prune().unwrap();
        assert_eq!(removed, 1);
        assert!(!guard.is_seen(&d, &sequence));
        let on_disk = fs::read_to_string(&path).unwrap();
        assert!(on_disk.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn prune_is_a_no_op_when_nothing_has_expired() {
        let path = tmp_path("prune-noop");
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
        guard.check_and_record(&did(), &[2u8; 32]);
        let removed = guard.prune().unwrap();
        assert_eq!(removed, 0);
        let _ = fs::remove_file(&path);
    }

    // F-12: a durable-write failure must never leave the in-memory verdict
    // ahead of disk. Root runs in this sandbox ignore POSIX permission
    // bits, so the failure is forced by removing the parent directory
    // `append_record`'s `OpenOptions::open` needs, then restoring it --
    // a real, privilege-independent I/O error (the same technique used for
    // `mini-witness-service`'s equivalent D-0481 test), not a simulated one.

    #[test]
    fn a_durable_write_failure_is_refused_not_silently_accepted_in_memory() {
        let dir = tmp_path("write-failure-dir");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("guard.log");
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
        let d = did();
        let sequence = [11u8; 32];

        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(guard.write_failures(), 0);
        let accepted = guard.check_and_record(&d, &sequence);
        assert!(!accepted, "a failed durable write must not report success");
        assert_eq!(guard.write_failures(), 1);
        assert!(
            !guard.is_seen(&d, &sequence),
            "a nonce whose durable write failed must not be remembered in memory either"
        );

        // Restore the directory and prove the exact same nonce is still
        // genuinely fresh -- nothing about the failed attempt poisoned it.
        fs::create_dir_all(&dir).unwrap();
        assert!(guard.check_and_record(&d, &sequence));
        assert!(guard.is_seen(&d, &sequence));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_write_failure_is_not_resurrected_by_a_simulated_restart() {
        let dir = tmp_path("write-failure-restart");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("guard.log");
        let d = did();
        let sequence = [12u8; 32];
        {
            let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
            fs::remove_dir_all(&dir).unwrap();
            assert!(!guard.check_and_record(&d, &sequence));
            fs::create_dir_all(&dir).unwrap();
        }
        // A brand new guard over the same path (a process restart) must
        // not find this nonce recorded -- it never durably committed.
        let guard = FileReplayGuard::open(&path, 60_000).unwrap();
        assert!(!guard.is_seen(&d, &sequence));
        let _ = fs::remove_dir_all(&dir);
    }

    // F-12: a non-ASCII byte in the hex field must never panic the decoder
    // via a `&str` slice landing inside a multi-byte UTF-8 character --
    // only ever a clean rejection.

    #[test]
    fn a_non_ascii_multibyte_hex_field_is_rejected_not_a_panic() {
        let d = did();
        // 62 ASCII bytes plus one 2-byte UTF-8 character ("é") makes 64
        // *bytes* total (passing the old byte-length check) while landing
        // a slice boundary inside the multi-byte character for several
        // values of `i` -- exactly what could previously panic.
        let hex = format!("{}\u{e9}", "a".repeat(62));
        assert_eq!(hex.len(), 64);
        let line = format!("{}\t{hex}\t1000\n", d.as_str());
        // Must not panic; must cleanly report "not a record."
        assert_eq!(decode_line(line.trim_end()), None);
    }

    #[test]
    fn a_file_with_a_non_ascii_hex_line_is_reported_as_corruption_not_a_crash() {
        let path = tmp_path("non-ascii-hex");
        let d = did();
        let hex = format!("{}\u{e9}", "b".repeat(62));
        // Not the final line -- the malformed-tail tolerance is a separate,
        // deliberate case (`a_truncated_final_line_is_tolerated`); this
        // proves a non-ASCII hex field elsewhere in the file is real
        // corruption, reported as an error, never a panic.
        let mut contents = format!("{}\t{hex}\t1000\n", d.as_str());
        contents.push_str(&encode_line(d.as_str(), &[13u8; 32], now_ms()));
        fs::write(&path, contents).unwrap();

        let err = FileReplayGuard::open(&path, 60_000).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn non_hex_ascii_characters_are_also_rejected() {
        let d = did();
        let hex = "z".repeat(64);
        let line = format!("{}\t{hex}\t1000\n", d.as_str());
        assert_eq!(decode_line(line.trim_end()), None);
    }

    #[test]
    fn a_file_over_the_size_cap_is_refused_before_reading() {
        let path = tmp_path("oversized");
        // Cheaper than writing real records: an over-cap run of newline
        // bytes, which `open` must reject on size alone, before ever
        // attempting to parse a single line.
        let contents = vec![b'\n'; (MAX_REPLAY_GUARD_FILE_BYTES + 1) as usize];
        fs::write(&path, contents).unwrap();

        let err = FileReplayGuard::open(&path, 60_000).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(&path);
    }
}
