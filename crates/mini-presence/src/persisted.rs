//! Durable replay guard with a process lock over each read/check/append or
//! compaction transaction. Cooperating processes may keep separate handles;
//! every mutation reloads committed disk state before deciding freshness.
//!
//! Complete malformed records fail closed, including a newline-terminated
//! final record. Only an unterminated tail can be an interrupted append; it
//! is removed under the same lock before another write. Files and containing
//! directories are flushed before reporting acceptance. Runtime deletion is
//! an error, never an empty guard. Whole-state rollback still requires an
//! external retained checkpoint; filesystem flushes cannot detect it.

use crate::verify::ReplayGuard;
use did_mini::Did;
use mini_crypto::HashAlgorithm;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
type Key = (String, [u8; 32]);
const MAX_REPLAY_GUARD_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug)]
pub struct FileReplayGuard {
    path: PathBuf,
    retention_ms: u64,
    seen: HashMap<Key, u64>,
    write_failures: u64,
    legacy: bool,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn digest_hex(bytes: &[u8]) -> String {
    HashAlgorithm::Blake3
        .digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn encode_line(device: &str, sequence: &[u8; 32], recorded_at_ms: u64) -> String {
    let hex: String = sequence.iter().map(|b| format!("{b:02x}")).collect();
    let content = format!("mini-presence/replay/v2\t{device}\t{hex}\t{recorded_at_ms}");
    format!("{content}\t{}\n", digest_hex(content.as_bytes()))
}

/// Legacy three-field records are accepted only for a locked conversion to
/// v2. New records bind version, identity, nonce and timestamp to a checksum.
/// Checksums detect accidental corruption, not malicious administrator edits.
fn decode_line(line: &str) -> Option<(String, [u8; 32], u64)> {
    let fields: Vec<_> = line.split('\t').collect();
    let fields = if fields.first() == Some(&"mini-presence/replay/v2") {
        if fields.len() != 5 {
            return None;
        }
        let (content, checksum) = line.rsplit_once('\t')?;
        if digest_hex(content.as_bytes()) != checksum {
            return None;
        }
        &fields[1..4]
    } else {
        if fields.len() != 3 {
            return None;
        }
        fields.as_slice()
    };
    let device = fields[0].to_owned();
    Did::parse(&device).ok()?;
    let recorded_at_ms = fields[2].parse().ok()?;
    let hex = fields[1].as_bytes();
    if hex.len() != 64 || !hex.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut sequence = [0u8; 32];
    for (i, byte) in sequence.iter_mut().enumerate() {
        let high = (hex[2 * i] as char).to_digit(16)?;
        let low = (hex[2 * i + 1] as char).to_digit(16)?;
        *byte = ((high << 4) | low) as u8;
    }
    Some((device, sequence, recorded_at_ms))
}

impl FileReplayGuard {
    pub fn open(path: impl AsRef<Path>, retention_ms: u64) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let _lock = mini_durable::lock_exclusive(&lock_path(&path))?;
        match fs::metadata(&path) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                mini_durable::atomic_replace(&path, b"")?
            }
            Err(error) => return Err(error),
        }
        let mut guard = Self {
            path,
            retention_ms,
            seen: HashMap::new(),
            write_failures: 0,
            legacy: false,
        };
        guard.reload()?;
        guard.prune_locked()?;
        Ok(guard)
    }
    pub fn write_failures(&self) -> u64 {
        self.write_failures
    }

    /// Reload under the process lock. Missing state after open is corruption,
    /// not first use. Flush before trusting a prior uncertain append outcome.
    fn reload(&mut self) -> io::Result<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        if file.metadata()?.len() > MAX_REPLAY_GUARD_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "replay guard exceeds size cap",
            ));
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_REPLAY_GUARD_FILE_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_REPLAY_GUARD_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "replay guard exceeds size cap",
            ));
        }
        let complete = bytes.iter().rposition(|b| *b == b'\n').map_or(0, |i| i + 1);
        let mut seen = HashMap::new();
        let mut legacy = false;
        for (index, line) in bytes[..complete]
            .split_inclusive(|b| *b == b'\n')
            .enumerate()
        {
            legacy |= !line.starts_with(b"mini-presence/replay/v2\t");
            let decoded = std::str::from_utf8(&line[..line.len() - 1])
                .ok()
                .and_then(decode_line)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("corrupt replay record at line {}", index + 1),
                    )
                })?;
            let (device, sequence, epoch) = decoded;
            seen.entry((device, sequence))
                .and_modify(|old: &mut u64| *old = (*old).max(epoch))
                .or_insert(epoch);
        }
        if complete != bytes.len() {
            file.set_len(complete as u64)?;
        }
        file.sync_all()?;
        mini_durable::sync_parent(&self.path)?;
        self.seen = seen;
        self.legacy = legacy;
        Ok(())
    }

    pub fn prune(&mut self) -> io::Result<usize> {
        let _lock = mini_durable::lock_exclusive(&lock_path(&self.path))?;
        self.reload()?;
        self.prune_locked()
    }
    fn prune_locked(&mut self) -> io::Result<usize> {
        let now = self.commit_clock()?;
        let mut staged = self.seen.clone();
        staged.retain(|_, epoch| epoch.saturating_add(self.retention_ms) > now);
        let removed = self.seen.len() - staged.len();
        if removed > 0 || self.legacy {
            let mut entries: Vec<_> = staged.iter().collect();
            entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
            let mut bytes = Vec::new();
            for ((device, sequence), epoch) in entries {
                bytes.extend_from_slice(encode_line(device, sequence, *epoch).as_bytes());
            }
            mini_durable::atomic_replace(&self.path, &bytes)?;
            self.seen = staged;
            self.legacy = false;
        }
        Ok(removed)
    }

    // Persist a time floor before pruning. A wall-clock rollback after a
    // previous sweep cannot make an expired attestation fresh again locally.
    fn commit_clock(&self) -> io::Result<u64> {
        let mut name = self.path.as_os_str().to_os_string();
        name.push(".clock");
        let path = PathBuf::from(name);
        let floor = match fs::read_to_string(&path) {
            Ok(value) => {
                let fields: Vec<_> = value.lines().collect();
                if fields.len() != 3 || fields[0] != "mini-presence/clock/v1" {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "corrupt replay clock",
                    ));
                }
                let epoch = fields[1].parse::<u64>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "corrupt replay clock")
                })?;
                let content = format!("mini-presence/clock/v1\n{epoch}\n");
                if fields[2] != digest_hex(content.as_bytes()) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "corrupt replay clock checksum",
                    ));
                }
                epoch
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error),
        }
        .max(self.seen.values().copied().max().unwrap_or(0));
        let now = now_ms();
        if now < floor {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "wall clock moved behind retained replay time; refusing expiry and acceptance",
            ));
        }
        let content = format!("mini-presence/clock/v1\n{now}\n");
        mini_durable::atomic_replace(
            &path,
            format!("{content}{}\n", digest_hex(content.as_bytes())).as_bytes(),
        )?;
        Ok(now)
    }

    /// Fallible form for callers that need the actual storage failure.
    /// `Ok(false)` denotes an already-recorded nonce; `Err` never accepts it.
    pub fn try_check_and_record(&mut self, device: &Did, sequence: &[u8; 32]) -> io::Result<bool> {
        let _lock = mini_durable::lock_exclusive(&lock_path(&self.path))?;
        self.reload()?;
        let key = (device.as_str().to_owned(), *sequence);
        if self.seen.contains_key(&key) {
            return Ok(false);
        }
        let epoch = self.commit_clock()?;
        let bytes = encode_line(&key.0, sequence, epoch);
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        if file.metadata()?.len().saturating_add(bytes.len() as u64) > MAX_REPLAY_GUARD_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "replay guard capacity exceeded",
            ));
        }
        file.write_all(bytes.as_bytes())?;
        file.sync_all()?;
        mini_durable::sync_parent(&self.path)?;
        self.seen.insert(key, epoch);
        Ok(true)
    }
}
fn lock_path(path: &Path) -> PathBuf {
    let mut lock = path.as_os_str().to_os_string();
    lock.push(".lock");
    PathBuf::from(lock)
}
impl ReplayGuard for FileReplayGuard {
    fn is_seen(&self, device: &Did, sequence: &[u8; 32]) -> bool {
        self.seen
            .contains_key(&(device.as_str().to_owned(), *sequence))
    }
    fn check_and_record(&mut self, device: &Did, sequence: &[u8; 32]) -> bool {
        match self.try_check_and_record(device, sequence) {
            Ok(result) => result,
            Err(_) => {
                self.write_failures = self.write_failures.saturating_add(1);
                false
            }
        }
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
        assert!(
            !guard.check_and_record(&d, &sequence),
            "deletion requires reopening, not silent reinitialization"
        );
        let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
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

    #[test]
    fn a_newline_terminated_malformed_final_line_is_not_a_torn_tail() {
        for bad in [b"broken\n".as_slice(), b"\n", b"\xff\n"] {
            let path = tmp_path("complete-bad-tail");
            fs::write(&path, bad).unwrap();
            assert_eq!(
                FileReplayGuard::open(&path, 60_000).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(fs::read(&path).unwrap(), bad);
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn stale_handles_reload_before_append_and_compaction() {
        let path = tmp_path("shared-handles");
        let d = did();
        let mut a = FileReplayGuard::open(&path, 60_000).unwrap();
        let mut b = FileReplayGuard::open(&path, 60_000).unwrap();
        let sequence = [42; 32];
        assert!(a.check_and_record(&d, &sequence));
        assert!(!b.check_and_record(&d, &sequence));
        assert_eq!(b.prune().unwrap(), 0);
        let reopened = FileReplayGuard::open(&path, 60_000).unwrap();
        assert!(reopened.is_seen(&d, &sequence));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn two_processes_accept_a_nonce_only_once() {
        const ENV: &str = "MINI_REPLAY_CHILD";
        if let Some(path) = std::env::var_os(ENV) {
            let path = PathBuf::from(path);
            let d = Did::parse(&std::env::var("MINI_REPLAY_DID").unwrap()).unwrap();
            let mut guard = FileReplayGuard::open(&path, 60_000).unwrap();
            if guard.check_and_record(&d, &[43; 32]) {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path.with_extension("accepted"))
                    .unwrap();
            }
            return;
        }
        let path = tmp_path("processes");
        let d = did();
        let mut children: Vec<_> = (0..4)
            .map(|_| {
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "persisted::tests::two_processes_accept_a_nonce_only_once",
                    ])
                    .env(ENV, &path)
                    .env("MINI_REPLAY_DID", d.as_str())
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        assert!(path.with_extension("accepted").exists());
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 1);
        fs::remove_file(path).unwrap();
    }
}
