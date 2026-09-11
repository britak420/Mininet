//! Shared process lease excludes appliance backup/restore. The lock lives
//! beside the home so replacing the home cannot replace the locked inode.
use crate::{CliError, Result};
use std::fs::{File, OpenOptions};
use std::path::Path;

#[allow(clippy::incompatible_msrv)] // pinned toolchain 1.94 provides OS file locks
pub(crate) fn lease(home: &Path) -> Result<File> {
    let parent = home
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| CliError::Io(e.to_string()))?;
    if home.is_symlink() {
        return Err(CliError::Io("node home must not be a symlink".into()));
    }
    let parent = parent
        .canonicalize()
        .map_err(|e| CliError::Io(e.to_string()))?;
    let name = home
        .file_name()
        .ok_or_else(|| CliError::Io("node home requires a directory name".into()))?;
    let mut lock_name = std::ffi::OsString::from(".mininet-maintenance-");
    lock_name.push(name);
    lock_name.push(".lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(parent.join(lock_name))
        .map_err(|e| CliError::Io(e.to_string()))?;
    file.try_lock_shared()
        .map_err(|e| CliError::Io(format!("node home is under maintenance: {e}")))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintenance_excludes_cli_and_shared_writers_can_coexist() {
        let parent = std::env::temp_dir().join(format!(
            "mini-lease-{}-{}",
            std::process::id(),
            crate::sequence::now_ms()
        ));
        std::fs::create_dir_all(&parent).unwrap();
        let home = parent.join("state");
        let first = lease(&home).unwrap();
        let second = lease(&home).unwrap();
        assert!(
            mini_durable::try_lock_exclusive(&parent.join(".mininet-maintenance-state.lock"))
                .is_err()
        );
        drop(first);
        drop(second);
        let exclusive =
            mini_durable::try_lock_exclusive(&parent.join(".mininet-maintenance-state.lock"))
                .unwrap();
        assert!(lease(&home).is_err());
        drop(exclusive);
        assert!(lease(&home).is_ok());
        std::fs::remove_dir_all(parent).unwrap();
    }
}
