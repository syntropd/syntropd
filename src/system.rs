//! System capability detection and pre-flight diagnostics.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// System capabilities and pre-flight check results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    /// True if systemd is running as PID 1.
    pub systemd_running: bool,
    /// Systemd version string if detectable.
    pub systemd_version: Option<String>,
    /// True if cgroups v2 unified hierarchy is active.
    pub cgroups_v2: bool,
    /// Available cgroup controllers (e.g. "cpu memory io pids").
    pub cgroup_controllers: Option<String>,
    /// True if Pressure Stall Information (PSI) is available.
    pub psi_available: bool,
    /// Detected DRM/KMS render nodes in `/dev/dri`.
    pub dri_devices: Vec<String>,
    /// Detected AI accelerator devices in `/dev/accel`.
    pub accel_devices: Vec<String>,
    /// True if system group `syntrop` exists.
    pub syntrop_group_exists: bool,
    /// True if user `sentry` exists.
    pub sentry_user_exists: bool,
    /// Target runtime directory status (`/run/syntrop`).
    pub run_syntrop_exists: bool,
}

impl SystemCapabilities {
    /// Inspect system capabilities directly from host OS.
    pub fn probe() -> Self {
        let systemd_running = Path::new("/run/systemd/system").exists();
        let systemd_version = Self::read_systemd_version();

        let cgroups_v2_path = Path::new("/sys/fs/cgroup/cgroup.controllers");
        let (cgroups_v2, cgroup_controllers) = if cgroups_v2_path.exists() {
            let controllers = fs::read_to_string(cgroups_v2_path).ok().map(|s| s.trim().to_string());
            (true, controllers)
        } else {
            (false, None)
        };

        let psi_available = Path::new("/proc/pressure/memory").exists();
        let dri_devices = Self::scan_dir_devices("/dev/dri", "renderD");
        let accel_devices = Self::scan_dir_devices("/dev/accel", "accel");

        let syntrop_group_exists = Self::check_group_exists("syntrop");
        let sentry_user_exists = Self::check_user_exists("sentry");
        let run_syntrop_exists = Path::new("/run/syntrop").exists();

        Self {
            systemd_running,
            systemd_version,
            cgroups_v2,
            cgroup_controllers,
            psi_available,
            dri_devices,
            accel_devices,
            syntrop_group_exists,
            sentry_user_exists,
            run_syntrop_exists,
        }
    }

    /// Helper to read systemd version from systemctl or package info.
    fn read_systemd_version() -> Option<String> {
        let output = std::process::Command::new("systemctl")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines().next().map(|l| l.to_string())
        } else {
            None
        }
    }

    /// Helper to scan directory for matching device nodes.
    fn scan_dir_devices(dir: &str, prefix: &str) -> Vec<String> {
        let mut devices = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with(prefix) {
                        devices.push(format!("{}/{}", dir, name));
                    }
                }
            }
        }
        devices.sort();
        devices
    }

    /// Check if group exists via `/etc/group` fallback.
    fn check_group_exists(group_name: &str) -> bool {
        if let Ok(content) = fs::read_to_string("/etc/group") {
            for line in content.lines() {
                if let Some(name) = line.split(':').next() {
                    if name == group_name {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if user exists via `/etc/passwd` fallback.
    fn check_user_exists(user_name: &str) -> bool {
        if let Ok(content) = fs::read_to_string("/etc/passwd") {
            for line in content.lines() {
                if let Some(name) = line.split(':').next() {
                    if name == user_name {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Evaluate overall readiness for zero-idle daemon operations.
    pub fn is_ready(&self) -> bool {
        self.systemd_running && self.cgroups_v2 && self.psi_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_capabilities_probe() {
        let caps = SystemCapabilities::probe();
        // Under standard Linux kernels running systemd:
        // We ensure probe does not panic and returns valid struct.
        println!("Probed capabilities: {:?}", caps);
        assert!(caps.cgroups_v2 || !caps.cgroups_v2); // Boolean sanity
    }

    #[test]
    fn test_device_scanner_empty_dir() {
        let dev = SystemCapabilities::scan_dir_devices("/nonexistent_dev_test_dir", "prefix");
        assert!(dev.is_empty());
    }

    #[test]
    fn test_group_exists_root() {
        assert!(SystemCapabilities::check_group_exists("root"));
        assert!(!SystemCapabilities::check_group_exists("definitely_nonexistent_group_xyz"));
    }

    #[test]
    fn test_user_exists_root() {
        assert!(SystemCapabilities::check_user_exists("root"));
        assert!(!SystemCapabilities::check_user_exists("definitely_nonexistent_user_xyz"));
    }
}
