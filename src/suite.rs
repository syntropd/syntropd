//! Suite component inventory and metadata for syntropd subsystem.

use serde::Serialize;

/// Suite component descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuiteComponent {
    /// Component identifier name.
    pub name: &'static str,
    /// Crates.io package name.
    pub package: &'static str,
    /// Crates.io package version.
    pub version: &'static str,
    /// Binary target name(s).
    pub binaries: &'static [&'static str],
    /// Functional role in the subsystem.
    pub role: &'static str,
    /// Primary Varlink or IPC socket path.
    pub primary_socket: Option<&'static str>,
    /// Systemd service unit name.
    pub service_unit: &'static str,
    /// Systemd socket unit name.
    pub socket_unit: Option<&'static str>,
}

/// Authoritative inventory of all suite components.
pub const SUITE_COMPONENTS: &[SuiteComponent] = &[
    SuiteComponent {
        name: "inferenced",
        package: "syntrop-inferenced",
        version: "0.1.0",
        binaries: &["inferenced"],
        role: "Hardware arbiter, demand paging, dynamic memory quotas & Ollama gateway",
        primary_socket: Some("/run/syntrop/io.syntrop.Inference1"),
        service_unit: "inferenced.service",
        socket_unit: Some("inferenced.socket"),
    },
    SuiteComponent {
        name: "modeld",
        package: "syntrop-modeld",
        version: "0.2.0",
        binaries: &["modeld"],
        role: "Content-addressable model store, zero-copy shared memory tensor sharing",
        primary_socket: Some("/run/syntrop/io.syntrop.Model1"),
        service_unit: "modeld.service",
        socket_unit: Some("modeld.socket"),
    },
    SuiteComponent {
        name: "contextd",
        package: "syntrop-contextd",
        version: "0.2.0",
        binaries: &["contextd"],
        role: "Causal event graph, configuration drift detection & chronologies",
        primary_socket: Some("/run/syntrop/io.syntrop.Context1"),
        service_unit: "contextd.service",
        socket_unit: Some("contextd.socket"),
    },
    SuiteComponent {
        name: "toold",
        package: "syntrop-toold",
        version: "0.2.0",
        binaries: &["toold"],
        role: "Sandboxed action & diagnostic execution with automated rollback",
        primary_socket: Some("/run/syntrop/io.syntrop.Tool1"),
        service_unit: "toold.service",
        socket_unit: Some("toold.socket"),
    },
    SuiteComponent {
        name: "runtimed",
        package: "syntrop-runtimed",
        version: "0.1.0",
        binaries: &["runtimed"],
        role: "Headless model execution, tensor generation & NPU acceleration",
        primary_socket: Some("/run/syntrop/io.syntrop.Runtime1"),
        service_unit: "runtimed.service",
        socket_unit: Some("runtimed.socket"),
    },
    SuiteComponent {
        name: "sentry",
        package: "syntrop-sentry",
        version: "0.1.0",
        binaries: &["sentry", "systemd-sentry"],
        role: "Autonomous zero-trust systemd supervisor daemon and crash watchdog",
        primary_socket: Some("/run/systemd-sentry/sentry.sock"),
        service_unit: "systemd-sentry.service",
        socket_unit: Some("systemd-sentry.socket"),
    },
    SuiteComponent {
        name: "syntropctl",
        package: "syntropctl",
        version: "0.1.0",
        binaries: &["syntropctl"],
        role: "Unified operator CLI for inspection, drift, models, and failure triage",
        primary_socket: None,
        service_unit: "syntrop-triage@.service",
        socket_unit: None,
    },
];

/// Retrieve all components in the syntrop suite.
pub fn get_components() -> &'static [SuiteComponent] {
    SUITE_COMPONENTS
}

/// Lookup a component by its name or package name.
pub fn find_component(name: &str) -> Option<&'static SuiteComponent> {
    SUITE_COMPONENTS.iter().find(|c| {
        c.name.eq_ignore_ascii_case(name)
            || c.package.eq_ignore_ascii_case(name)
            || c.binaries.iter().any(|b| b.eq_ignore_ascii_case(name))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suite_components_count() {
        assert_eq!(SUITE_COMPONENTS.len(), 7);
    }

    #[test]
    fn test_find_component() {
        assert!(find_component("inferenced").is_some());
        assert!(find_component("syntrop-toold").is_some());
        assert!(find_component("systemd-sentry").is_some());
        assert!(find_component("syntropctl").is_some());
        assert!(find_component("nonexistent").is_none());
    }

    #[test]
    fn test_packages_use_standard_prefix() {
        for component in SUITE_COMPONENTS {
            if component.name != "syntropctl" {
                assert!(
                    component.package.starts_with("syntrop-"),
                    "Component {} must have syntrop- prefix",
                    component.name
                );
            } else {
                assert_eq!(component.package, "syntropctl");
            }
        }
    }
}
