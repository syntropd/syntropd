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

/// Authoritative catalog of all units in the subsystem.
pub const ALL_UNITS: &[SystemdUnitDescriptor] = &[
    SystemdUnitDescriptor {
        name: "syntrop-sockets.target",
        unit_type: "target",
        description: "syntropd Unified Socket Activation Umbrella",
        documentation: "https://syntropd.github.io/architecture.html#socket",
        is_umbrella: true,
    },
    SystemdUnitDescriptor {
        name: "syntrop-triage@.service",
        unit_type: "service",
        description: "syntropd Autonomous Triage for Failed Unit %I",
        documentation: "https://syntropd.github.io/manual.html",
        is_umbrella: true,
    },
    SystemdUnitDescriptor {
        name: "toold.socket",
        unit_type: "socket",
        description: "toold Varlink socket",
        documentation: "https://github.com/syntropd/toold",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "toold.service",
        unit_type: "service",
        description: "toold sandboxed execution daemon",
        documentation: "https://github.com/syntropd/toold",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "runtimed.socket",
        unit_type: "socket",
        description: "runtimed Varlink socket",
        documentation: "https://github.com/syntropd/runtimed",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "runtimed.service",
        unit_type: "service",
        description: "runtimed tensor execution daemon",
        documentation: "https://github.com/syntropd/runtimed",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "inferenced.socket",
        unit_type: "socket",
        description: "inferenced activation sockets",
        documentation: "https://github.com/syntropd/inferenced",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "inferenced.service",
        unit_type: "service",
        description: "inferenced hardware arbiter & paging daemon",
        documentation: "https://github.com/syntropd/inferenced",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "contextd.socket",
        unit_type: "socket",
        description: "contextd Varlink socket",
        documentation: "https://github.com/syntropd/contextd",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "contextd.service",
        unit_type: "service",
        description: "contextd causal chronology daemon",
        documentation: "https://github.com/syntropd/contextd",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "modeld.socket",
        unit_type: "socket",
        description: "modeld Varlink socket",
        documentation: "https://github.com/syntropd/modeld",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "modeld.service",
        unit_type: "service",
        description: "modeld content-addressable model cache daemon",
        documentation: "https://github.com/syntropd/modeld",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "systemd-sentry.socket",
        unit_type: "socket",
        description: "systemd-sentry IPC socket",
        documentation: "https://github.com/syntropd/sentry",
        is_umbrella: false,
    },
    SystemdUnitDescriptor {
        name: "systemd-sentry.service",
        unit_type: "service",
        description: "systemd-sentry crash triage and supervisor",
        documentation: "https://github.com/syntropd/sentry",
        is_umbrella: false,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_units_count() {
        assert_eq!(ALL_UNITS.len(), 14);
    }

    #[test]
    fn test_umbrella_units() {
        let umbrella_units: Vec<_> = ALL_UNITS.iter().filter(|u| u.is_umbrella).collect();
        assert_eq!(umbrella_units.len(), 2);
        assert_eq!(umbrella_units[0].name, "syntrop-sockets.target");
        assert_eq!(umbrella_units[1].name, "syntrop-triage@.service");
    }

    #[test]
    fn test_unit_installed_nonexistent() {
        assert!(!is_unit_installed("nonexistent_phantom_unit_12345.service"));
    }
}
