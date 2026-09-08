//! A per-home monotonic counter for the author-scoped `sequence` field
//! every signed object needs. The counter is protected by an OS-backed
//! exclusive lock so separate `mini` processes using the same home cannot
//! allocate the same sequence number.

use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;

use crate::error::{CliError, Result};

fn counter_path(home: &Path) -> std::path::PathBuf {
    home.join("sequence")
}

fn lock_path(home: &Path) -> std::path::PathBuf {
    home.join("sequence.lock")
}

/// The next unused sequence number for this home, persisting the
/// increment before returning it. The lock is held across the complete
/// read-modify-write operation and is released automatically if the process
/// exits or an I/O error returns early.
pub fn next(home: &Path) -> Result<u64> {
    fs::create_dir_all(home).map_err(|e| CliError::Io(e.to_string()))?;

    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path(home))
        .map_err(|e| CliError::Io(e.to_string()))?;
    #[allow(clippy::incompatible_msrv)]
    lock.lock().map_err(|e| CliError::Io(e.to_string()))?;

    let path = counter_path(home);
    // Only a genuinely absent file means "no counter yet" (F-04). Every
    // other outcome -- a read error other than NotFound, or content that
    // does not parse as a trustworthy u64 -- must refuse rather than
    // silently restart from zero. Restarting on a truncated, unreadable,
    // or malformed existing counter would let a home reuse sequence
    // numbers it already issued, and this counter feeds directly into the
    // author-scoped `sequence` field every signed object carries.
    let current: u64 = match fs::read_to_string(&path) {
        Ok(s) => s
            .trim()
            .parse()
            .map_err(|_| CliError::CorruptSequenceFile)?,
        Err(e) if e.kind() == ErrorKind::NotFound => 0,
        Err(e) => return Err(CliError::Io(e.to_string())),
    };
    let next = current
        .checked_add(1)
        .ok_or_else(|| CliError::Io("sequence counter exhausted".to_string()))?;
    fs::write(&path, next.to_string()).map_err(|e| CliError::Io(e.to_string()))?;
    Ok(next)
}

/// The current wall-clock time in milliseconds, the `timestamp_ms` every
/// signed object needs.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_numbers_increase_and_persist_across_calls() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mini-cli-seq-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert_eq!(next(&p).unwrap(), 1);
        assert_eq!(next(&p).unwrap(), 2);
        assert_eq!(next(&p).unwrap(), 3);
    }

    #[test]
    fn sequence_lock_is_created_and_reused() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mini-cli-seq-lock-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        assert_eq!(next(&p).unwrap(), 1);
        assert!(lock_path(&p).is_file());
        assert_eq!(next(&p).unwrap(), 2);
    }

    #[test]
    fn concurrent_allocations_are_unique_and_monotonic() {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "mini-cli-seq-concurrent-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let values = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8).map(|_| scope.spawn(|| next(&p).unwrap())).collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });

        let mut sorted = values;
        sorted.sort_unstable();
        assert_eq!(sorted, (1..=8).collect::<Vec<_>>());
    }

    fn new_test_home(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "mini-cli-seq-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    // -----------------------------------------------------------------
    // F-04: only NotFound is first-use; everything else refuses rather
    // than silently restarting the counter
    // -----------------------------------------------------------------

    #[test]
    fn empty_nonnumeric_and_overflowing_files_are_refused_not_reset() {
        for value in ["", "not-a-number", "18446744073709551616"] {
            let home = new_test_home("malformed");
            fs::write(counter_path(&home), value).unwrap();
            let err = next(&home).unwrap_err();
            assert_eq!(err.to_string(), CliError::CorruptSequenceFile.to_string());
            // The rejected bytes are never rewritten.
            assert_eq!(fs::read_to_string(counter_path(&home)).unwrap(), value);
            fs::remove_dir_all(home).unwrap();
        }
    }

    #[test]
    fn invalid_utf8_content_is_refused_not_treated_as_first_use() {
        // Invalid UTF-8 fails at fs::read_to_string itself (a read error
        // distinct from NotFound), so this takes the Io path rather than
        // CorruptSequenceFile -- still correctly refused either way, never
        // silently treated as "no counter yet".
        let home = new_test_home("invalid-utf8");
        fs::write(counter_path(&home), [0xff, 0xfe]).unwrap();
        assert!(next(&home).is_err());
        assert_eq!(fs::read(counter_path(&home)).unwrap(), [0xff, 0xfe]);
    }

    #[test]
    fn a_directory_in_place_of_the_counter_file_is_a_read_error_not_first_use() {
        let home = new_test_home("dir-in-place");
        fs::remove_file(counter_path(&home)).ok();
        fs::create_dir(counter_path(&home)).unwrap();
        // Reading a directory as a string is an I/O error distinct from
        // NotFound -- must not be folded into "no counter yet" either.
        assert!(next(&home).is_err());
        assert!(counter_path(&home).is_dir());
    }

    #[test]
    fn an_exhausted_counter_at_u64_max_is_preserved_not_silently_reset() {
        let home = new_test_home("exhausted");
        fs::write(counter_path(&home), u64::MAX.to_string()).unwrap();
        assert!(next(&home).is_err());
        assert_eq!(
            fs::read_to_string(counter_path(&home)).unwrap(),
            u64::MAX.to_string()
        );
    }

    #[test]
    fn first_creation_still_starts_at_one() {
        // NotFound is the one legitimate first-use path -- confirms the
        // fix did not also break the ordinary new-home case.
        let home = new_test_home("first-creation");
        fs::remove_file(counter_path(&home)).ok();
        assert_eq!(next(&home).unwrap(), 1);
    }
}
