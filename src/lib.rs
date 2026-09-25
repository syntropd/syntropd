//! `syntropd` — Native AI Subsystem for systemd Umbrella Meta-Package & Supervisor.
//!
//! Provides unified system status, systemd socket activation inspection,
//! autonomous fault triage, and component coordination across the syntrop suite:
//! - `syntrop-inferenced` (HW arbiter, demand paging, memory quotas)
//! - `syntrop-modeld` (Content-addressable model cache, zero-copy shm distribution)
//! - `syntrop-contextd` (Causal event graph, configuration drift detection)
//! - `syntrop-toold` (Sandboxed diagnostic execution, automated rollback)
//! - `syntrop-runtimed` (Headless model execution, tensor generation)
//! - `syntrop-sentry` (Autonomous zero-trust systemd supervisor)
//! - `syntrop-routerd` (Multi-provider LLM reverse proxy & telemetry router)
//! - `syntropctl` (Unified operator CLI)

pub mod sockets;
pub mod suite;
pub mod system;
pub mod triage;
pub mod units;

use serde::Serialize;

/// Comprehensive subsystem status overview.
#[derive(Debug, Clone, Serialize)]
pub struct SubsystemStatus {
    /// Host kernel & system capabilities.
    pub capabilities: system::SystemCapabilities,
    /// Suite component inventory.
    pub components: Vec<suite::SuiteComponent>,
    /// Observed IPC sockets status.
    pub sockets: Vec<sockets::SocketReport>,
    /// Systemd units status.
    pub units: Vec<units::UnitStatus>,
    /// Subsystem version string.
    pub version: &'static str,
}

/// Inspect the complete status of the syntropd subsystem.
pub fn inspect_subsystem() -> SubsystemStatus {
    SubsystemStatus {
        capabilities: system::SystemCapabilities::probe(),
        components: suite::get_components().to_vec(),
        sockets: sockets::inspect_sockets(),
        units: units::inspect_all_units(),
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_subsystem() {
        let status = inspect_subsystem();
        assert_eq!(status.version, "0.3.5");
        assert_eq!(status.components.len(), 8);
        assert!(!status.sockets.is_empty());
        assert!(!status.units.is_empty());
    }

    #[test]
    fn test_subsystem_serialization() {
        let status = inspect_subsystem();
        let json = serde_json::to_string_pretty(&status).expect("Serialization must succeed");
        assert!(json.contains("syntrop-inferenced"));
        assert!(json.contains("syntrop-sockets.target"));
    }
}
