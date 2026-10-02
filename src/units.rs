//! Systemd unit catalog and inspection utilities.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Systemd unit descriptor in the syntrop subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnitDescriptor {
    pub name: &'static str,
    pub unit_type: &'static str,
    pub description: &'static str,
    pub documentation: &'static str,
    pub is_umbrella: bool,
}

const fn desc(
    name: &'static str,
    unit_type: &'static str,
    description: &'static str,
    documentation: &'static str,
    is_umbrella: bool,
) -> SystemdUnitDescriptor {
    SystemdUnitDescriptor { name, unit_type, description, documentation, is_umbrella }
}

/// Authoritative catalog of all units in the subsystem.
pub const ALL_UNITS: &[SystemdUnitDescriptor] = &[
    desc("syntrop-sockets.target", "target", "syntropd Unified Socket Activation Umbrella", "https://syntropd.github.io/architecture.html#socket", true),
    desc("syntrop-triage@.service", "service", "syntropd Autonomous Triage for Failed Unit %I", "https://syntropd.github.io/manual.html", true),
    desc("syntrop-admin@.service", "service", "syntropd Autonomous OS Self-Healing & Remediation for %I", "https://syntropd.github.io/manual.html", true),
    desc("syntrop-companion.service", "service", "syntropd Linux Cognitive Desktop Companion", "https://syntropd.github.io/manual.html", true),
    desc("syntrop-tuning.service", "service", "syntropd Dynamic Kernel Telemetry & Closed-Loop Tuning Governor", "https://syntropd.github.io/manual.html", true),
    desc("toold.socket", "socket", "toold Varlink socket", "https://github.com/syntropd/toold", false),
    desc("toold.service", "service", "toold sandboxed execution daemon", "https://github.com/syntropd/toold", false),
    desc("runtimed.socket", "socket", "runtimed Varlink socket", "https://github.com/syntropd/runtimed", false),
    desc("runtimed.service", "service", "runtimed tensor execution daemon", "https://github.com/syntropd/runtimed", false),
    desc("inferenced.socket", "socket", "inferenced activation sockets", "https://github.com/syntropd/inferenced", false),
    desc("inferenced.service", "service", "inferenced hardware arbiter & paging daemon", "https://github.com/syntropd/inferenced", false),
    desc("contextd.socket", "socket", "contextd Varlink socket", "https://github.com/syntropd/contextd", false),
    desc("contextd.service", "service", "contextd causal chronology daemon", "https://github.com/syntropd/contextd", false),
    desc("modeld.socket", "socket", "modeld Varlink socket", "https://github.com/syntropd/modeld", false),
    desc("modeld.service", "service", "modeld content-addressable model cache daemon", "https://github.com/syntropd/modeld", false),
    desc("systemd-sentry.socket", "socket", "systemd-sentry IPC socket", "https://github.com/syntropd/sentry", false),
    desc("systemd-sentry.service", "service", "systemd-sentry crash triage and supervisor", "https://github.com/syntropd/sentry", false),
    desc("routerd.socket", "socket", "routerd Varlink socket", "https://github.com/syntropd/routerd", false),
    desc("routerd.service", "service", "routerd model router and gateway daemon", "https://github.com/syntropd/routerd", false),
];

/// Status of a systemd unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitStatus {
    pub name: String,
    pub unit_type: String,
    pub description: String,
    pub installed: bool,
    pub active_state: String,
    pub is_umbrella: bool,
}

/// Standard paths where systemd looks for units.
pub const UNIT_SEARCH_PATHS: &[&str] = &[
    "/etc/systemd/system",
    "/run/systemd/system",
    "/usr/lib/systemd/system",
    "/lib/systemd/system",
    "/etc/systemd/user",
    "/usr/lib/systemd/user",
];

/// Check if a unit file exists on disk in any systemd search path.
pub fn is_unit_installed(unit_name: &str) -> bool {
    let clean_name = if let Some(base) = unit_name.strip_suffix("@.service") {
        format!("{}@.service", base)
    } else {
        unit_name.to_string()
    };

    UNIT_SEARCH_PATHS
        .iter()
        .any(|dir| Path::new(dir).join(&clean_name).exists())
}

/// Query active state via systemctl is-active.
pub fn query_unit_active_state(unit_name: &str) -> String {
    let is_user = unit_name == "syntrop-companion.service";
    if is_user {
        if let Ok(out) = Command::new("systemctl")
            .arg("--user")
            .arg("is-active")
            .arg(unit_name)
            .output()
        {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !state.is_empty() && state != "unknown" {
                return state;
            }
        }
    }

    let output = Command::new("systemctl")
        .arg("is-active")
        .arg(unit_name)
        .output();

    match output {
        Ok(out) => {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if state.is_empty() {
                "inactive".to_string()
            } else {
                state
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Inspect all units in the catalog.
pub fn inspect_all_units() -> Vec<UnitStatus> {
    ALL_UNITS
        .iter()
        .map(|desc| {
            let installed = is_unit_installed(desc.name);
            let active_state = if installed {
                query_unit_active_state(desc.name)
            } else {
                "not-installed".to_string()
            };

            UnitStatus {
                name: desc.name.to_string(),
                unit_type: desc.unit_type.to_string(),
                description: desc.description.to_string(),
                installed,
                active_state,
                is_umbrella: desc.is_umbrella,
            }
        })
        .collect()
}

/// Comprehensive canonical unit supervision summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSupervisionReport {
    pub total_units: usize,
    pub installed_count: usize,
    pub active_count: usize,
    pub failed_units: Vec<String>,
    pub all_healthy: bool,
}

/// Perform canonical supervision across all subsystem units.
pub fn supervise_canonical_units() -> CanonicalSupervisionReport {
    let statuses = inspect_all_units();
    let total_units = statuses.len();
    let mut installed_count = 0;
    let mut active_count = 0;
    let mut failed_units = Vec::new();

    for s in &statuses {
        if s.installed {
            installed_count += 1;
        }
        if s.active_state == "active" {
            active_count += 1;
        } else if s.active_state == "failed" {
            failed_units.push(s.name.clone());
        }
    }

    let all_healthy = failed_units.is_empty();
    CanonicalSupervisionReport {
        total_units,
        installed_count,
        active_count,
        failed_units,
        all_healthy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_units_count() {
        assert_eq!(ALL_UNITS.len(), 19);
    }

    #[test]
    fn test_umbrella_units() {
        let umbrella_units: Vec<_> = ALL_UNITS.iter().filter(|u| u.is_umbrella).collect();
        assert_eq!(umbrella_units.len(), 5);
        assert_eq!(umbrella_units[0].name, "syntrop-sockets.target");
        assert_eq!(umbrella_units[1].name, "syntrop-triage@.service");
        assert_eq!(umbrella_units[2].name, "syntrop-admin@.service");
        assert_eq!(umbrella_units[3].name, "syntrop-companion.service");
        assert_eq!(umbrella_units[4].name, "syntrop-tuning.service");
    }

    #[test]
    fn test_unit_installed_nonexistent() {
        assert!(!is_unit_installed("nonexistent_phantom_unit_12345.service"));
    }

    #[test]
    fn test_supervise_canonical_units() {
        let rep = supervise_canonical_units();
        assert_eq!(rep.total_units, 19);
        assert!(rep.installed_count <= 19);
    }
}

