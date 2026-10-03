//! Preflight check verifying configuration file write permissions.

use std::fs::OpenOptions;
use std::path::Path;

/// Check whether the target configuration file can be written by the current process.
/// If access is restricted or denied, prints a helpful notice advising running with `sudo syn setup`.
pub fn check_config_writable(path: &Path) {
    if is_config_write_restricted(path) {
        eprintln!(
            "  [!] Notice: write permissions to {} appear restricted. \
            Please re-run with: sudo syn setup",
            path.display()
        );
    }
}

/// Evaluates if writing to the configuration path is restricted.
pub fn is_config_write_restricted(path: &Path) -> bool {
    if path.exists() {
        OpenOptions::new().write(true).open(path).is_err()
    } else if let Some(parent) = path.parent() {
        if parent.exists() {
            let probe = parent.join(format!(".tmp_write_probe_{}", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&probe) {
                Ok(_) => {
                    let _ = std::fs::remove_file(&probe);
                    false
                }
                Err(_) => true,
            }
        } else {
            true
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_writable_temp_dir() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("routerd.toml");
        assert!(!is_config_write_restricted(&target));
    }

    #[test]
    fn test_nonexistent_unwritable_parent() {
        let target = Path::new("/proc/sys/unwritable_probe/routerd.toml");
        assert!(is_config_write_restricted(target));
    }
}
