//! Per-home author sequence allocation. The OS lock covers trusted-head
//! reconciliation and both durable files. The checkpoint is bound to the
//! local author and detects isolated counter deletion/rollback; signed store
//! history detects rollback of both files when that history survives.
//! Restoring the home and every external copy together remains outside local
//! filesystem guarantees and requires a separately retained author checkpoint.

use crate::error::{CliError, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

fn counter_path(home: &Path) -> std::path::PathBuf {
    home.join("sequence")
}
fn lock_path(home: &Path) -> std::path::PathBuf {
    home.join("sequence.lock")
}
fn checkpoint_path(home: &Path) -> std::path::PathBuf {
    home.join("sequence.head")
}
fn io(error: std::io::Error) -> CliError {
    CliError::Io(error.to_string())
}

/// Allocate only after comparing local counters with actual signed objects
/// from this author in the caller's store. A claimed author/index row is never
/// enough: verify root/device provenance before using the sequence as a floor.
/// The durable checkpoint also preserves reservations not yet inserted.
pub fn next(home: &Path, store_path: &Path) -> Result<u64> {
    let identity = crate::identity::load(home)?;
    allocate(home, identity.human_did().as_str(), || {
        let store = crate::store::open_store(store_path)?;
        let mut floor = 0;
        for id in store
            .by_author(&identity.human_did())
            .map_err(|e| CliError::Store(e.to_string()))?
        {
            let object = store.get(&id).map_err(|e| CliError::Store(e.to_string()))?;
            // Unauthenticated inserted content cannot raise the counter.
            if mini_objects::verify_provenance(
                &object,
                &identity.human.kel(),
                &identity.device.kel(),
            )
            .is_ok()
            {
                floor = floor.max(object.sequence);
            }
        }
        Ok(floor)
    })
}

fn allocate(home: &Path, author: &str, trusted_head: impl FnOnce() -> Result<u64>) -> Result<u64> {
    mini_durable::create_dir_all(home).map_err(io)?;
    let _lock = mini_durable::lock_exclusive(&lock_path(home)).map_err(io)?;
    let path = counter_path(home);
    let current = match fs::read_to_string(&path) {
        Ok(value) => Some(
            value
                .trim()
                .parse::<u64>()
                .map_err(|_| CliError::CorruptSequenceFile)?,
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(io(error)),
    };
    let checkpoint = match fs::read_to_string(checkpoint_path(home)) {
        Ok(value) => Some(decode_checkpoint(&value, author)?),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(io(error)),
    };
    if let Some(checkpoint) = checkpoint {
        if current != Some(checkpoint) {
            return Err(CliError::CorruptSequenceFile);
        }
    }
    let floor = trusted_head()?;
    let current = current.unwrap_or(0);
    if current < floor {
        return Err(CliError::CorruptSequenceFile);
    }
    let next = current
        .checked_add(1)
        .ok_or_else(|| CliError::Io("sequence counter exhausted".to_owned()))?;
    // Reserve first. A crash between these two replacements leaves a mismatch
    // that refuses further signing; no acknowledged allocation is reusable.
    mini_durable::atomic_replace(
        &checkpoint_path(home),
        encode_checkpoint(author, next).as_bytes(),
    )
    .map_err(io)?;
    mini_durable::atomic_replace(&path, next.to_string().as_bytes()).map_err(io)?;
    Ok(next)
}
fn encode_checkpoint(author: &str, sequence: u64) -> String {
    let content = format!("mini-author-sequence/v1\n{author}\n{sequence}\n");
    format!("{content}{}\n", blake3::hash(content.as_bytes()).to_hex())
}
fn decode_checkpoint(value: &str, author: &str) -> Result<u64> {
    let lines: Vec<_> = value.lines().collect();
    if lines.len() != 4 || lines[0] != "mini-author-sequence/v1" || lines[1] != author {
        return Err(CliError::CorruptSequenceFile);
    }
    let sequence = lines[2]
        .parse::<u64>()
        .map_err(|_| CliError::CorruptSequenceFile)?;
    if value != encode_checkpoint(author, sequence) {
        return Err(CliError::CorruptSequenceFile);
    }
    Ok(sequence)
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
    fn test_next(home: &Path) -> Result<u64> {
        allocate(home, "test-home", || Ok(0))
    }

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
        assert_eq!(test_next(&p).unwrap(), 1);
        assert_eq!(test_next(&p).unwrap(), 2);
        assert_eq!(test_next(&p).unwrap(), 3);
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

        assert_eq!(test_next(&p).unwrap(), 1);
        assert!(lock_path(&p).is_file());
        assert_eq!(test_next(&p).unwrap(), 2);
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
            let handles: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| test_next(&p).unwrap()))
                .collect();
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
            let err = test_next(&home).unwrap_err();
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
        assert!(test_next(&home).is_err());
        assert_eq!(fs::read(counter_path(&home)).unwrap(), [0xff, 0xfe]);
    }

    #[test]
    fn a_directory_in_place_of_the_counter_file_is_a_read_error_not_first_use() {
        let home = new_test_home("dir-in-place");
        fs::remove_file(counter_path(&home)).ok();
        fs::create_dir(counter_path(&home)).unwrap();
        // Reading a directory as a string is an I/O error distinct from
        // NotFound -- must not be folded into "no counter yet" either.
        assert!(test_next(&home).is_err());
        assert!(counter_path(&home).is_dir());
    }

    #[test]
    fn an_exhausted_counter_at_u64_max_is_preserved_not_silently_reset() {
        let home = new_test_home("exhausted");
        fs::write(counter_path(&home), u64::MAX.to_string()).unwrap();
        assert!(test_next(&home).is_err());
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
        assert_eq!(test_next(&home).unwrap(), 1);
    }

    #[test]
    fn deleted_or_rolled_back_counter_is_refused_against_retained_checkpoint() {
        for delete in [false, true] {
            let home = new_test_home("rollback");
            assert_eq!(test_next(&home).unwrap(), 1);
            assert_eq!(test_next(&home).unwrap(), 2);
            if delete {
                fs::remove_file(counter_path(&home)).unwrap();
            } else {
                fs::write(counter_path(&home), b"1").unwrap();
            }
            assert!(test_next(&home).is_err());
            fs::remove_dir_all(home).unwrap();
        }
    }

    #[test]
    fn checkpoint_checksum_author_and_counter_pair_must_match() {
        let home = new_test_home("checkpoint-binding");
        test_next(&home).unwrap();
        assert!(allocate(&home, "different-author", || Ok(0)).is_err());
        let bytes = fs::read_to_string(checkpoint_path(&home)).unwrap();
        fs::write(
            checkpoint_path(&home),
            bytes.replace("test-home", "other-home"),
        )
        .unwrap();
        assert!(test_next(&home).is_err());
    }

    #[test]
    fn verified_signed_author_history_refuses_full_local_counter_rollback() {
        let home = tempfile::tempdir().unwrap();
        let store_dir = tempfile::tempdir().unwrap();
        let identity = crate::identity::init(home.path()).unwrap();
        let store_path = store_dir.path();
        let first = next(home.path(), store_path).unwrap();
        let saved_counter = fs::read(counter_path(home.path())).unwrap();
        let saved_checkpoint = fs::read(checkpoint_path(home.path())).unwrap();
        let second = next(home.path(), store_path).unwrap();
        let object = mini_objects::ObjectBuilder::new(mini_objects::ObjectType::POST)
            .sequence(second)
            .timestamp_ms(1)
            .payload(mini_objects::Payload::Public(
                b"signed author head".to_vec(),
            ))
            .sign(&identity.human_did(), &identity.device)
            .unwrap();
        crate::store::open_store(store_path)
            .unwrap()
            .insert(&object)
            .unwrap();
        assert!(second > first);
        fs::write(counter_path(home.path()), saved_counter).unwrap();
        fs::write(checkpoint_path(home.path()), saved_checkpoint).unwrap();
        assert!(next(home.path(), store_path).is_err());
        fs::remove_file(counter_path(home.path())).unwrap();
        fs::remove_file(checkpoint_path(home.path())).unwrap();
        assert!(next(home.path(), store_path).is_err());
    }

    #[test]
    fn separate_processes_reserve_distinct_sequences() {
        const ENV: &str = "MINI_SEQUENCE_CHILD";
        if let Some(home) = std::env::var_os(ENV) {
            let home = std::path::PathBuf::from(home);
            let value = test_next(&home).unwrap();
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(home.join(format!("reserved-{value}")))
                .unwrap();
            return;
        }
        let home = new_test_home("processes");
        let mut children: Vec<_> = (0..4)
            .map(|_| {
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "sequence::tests::separate_processes_reserve_distinct_sequences",
                    ])
                    .env(ENV, &home)
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        assert_eq!(fs::read_to_string(counter_path(&home)).unwrap(), "4");
        for value in 1..=4 {
            assert!(home.join(format!("reserved-{value}")).exists());
        }
        fs::remove_dir_all(home).unwrap();
    }
}
