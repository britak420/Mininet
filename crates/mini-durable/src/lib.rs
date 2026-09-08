//! Filesystem commit primitives. Success requires both file and directory
//! barriers; errors are never downgraded to best-effort durability.
//!
//! Callers must serialize read/modify/write with a stable lock file. Lock
//! files must not be unlinked while in use, including by cleanup tools.
//! These APIs assume an owner-controlled local directory and a filesystem
//! and device that honor OS locking, atomic rename, and flush requests.
//! They do not prevent an administrator restoring/deleting the entire state.

#![forbid(unsafe_code)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Commit directory entries, including on Windows. Unsupported filesystem
/// semantics fail closed; Windows requires a writable directory handle.
pub fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    let directory = File::open(path)?;
    #[cfg(windows)]
    let directory = {
        use std::os::windows::fs::OpenOptionsExt;
        OpenOptions::new()
            .write(true)
            .custom_flags(0x0200_0000) // FILE_FLAG_BACKUP_SEMANTICS
            .open(path)?
    };
    #[cfg(any(unix, windows))]
    return directory.sync_all();
    #[cfg(not(any(unix, windows)))]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "directory durability is unavailable",
    ))
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

/// Commit the parent directory after creating, removing, or renaming a file.
pub fn sync_parent(path: &Path) -> io::Result<()> {
    sync_directory(parent(path))
}

/// Create missing ancestors and commit each new directory entry before use.
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(meta) if meta.is_dir() => return Ok(()),
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "state parent is not a directory",
            ))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    create_dir_all(parent(path))?;
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => {}
        Err(e) => return Err(e),
    }
    sync_parent(path)
}

fn open_lock(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
}

/// Acquire a stable OS lock. The returned file owns the lock until dropped.
pub fn lock_exclusive(path: &Path) -> io::Result<File> {
    let file = open_lock(path)?;
    file.lock()?;
    Ok(file)
}

/// Refuse a second live owner instead of blocking service startup.
pub fn try_lock_exclusive(path: &Path) -> io::Result<File> {
    let file = open_lock(path)?;
    file.try_lock().map_err(io::Error::from)?;
    Ok(file)
}

/// Atomically replace a file, flushing contents before rename and its parent
/// after. On an error after rename the operation may have committed: callers
/// must fail closed and reload, never issue a conflicting retry from memory.
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let filename = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing state filename"))?;
    let mut temp_name = filename.to_os_string();
    temp_name.push(format!(
        ".{}-{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let temp = parent(path).join(temp_name);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)?;
        sync_parent(path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "mini-durable-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn replaces_and_flushes_real_directory() {
        let root = temp("replace");
        create_dir_all(&root.join("nested")).unwrap();
        let path = root.join("nested/state");
        atomic_replace(&path, b"first").unwrap();
        atomic_replace(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn locks_exclude_another_handle_and_release_on_drop() {
        let root = temp("lock");
        create_dir_all(&root).unwrap();
        let lock = try_lock_exclusive(&root.join("lock")).unwrap();
        assert!(try_lock_exclusive(&root.join("lock")).is_err());
        drop(lock);
        drop(try_lock_exclusive(&root.join("lock")).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_replace_preserves_old_file() {
        let root = temp("error");
        create_dir_all(&root).unwrap();
        let path = root.join("state");
        fs::create_dir(&path).unwrap();
        assert!(atomic_replace(&path, b"invalid destination").is_err());
        assert!(path.is_dir());
        fs::remove_dir_all(root).unwrap();
    }
}
