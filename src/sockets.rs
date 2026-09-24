//! Socket definitions, status monitoring, and connectivity probing.

use serde::{Deserialize, Serialize};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// Socket descriptor in the syntrop subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketDefinition {
    /// Associated daemon name.
    pub daemon: &'static str,
    /// Path or address of socket.
    pub address: &'static str,
    /// Whether this is a Varlink IPC interface.
    pub is_varlink: bool,
    /// Recommended file permission mode.
    pub mode: u32,
    /// Owning group.
    pub group: &'static str,
}

/// Known socket definitions across all daemons.
pub const ALL_SOCKETS: &[SocketDefinition] = &[
    SocketDefinition {
        daemon: "toold",
        address: "/run/syntrop/io.syntrop.Tool1",
        is_varlink: true,
        mode: 0o660,
        group: "syntrop",
    },
    SocketDefinition {
        daemon: "runtimed",
        address: "/run/syntrop/io.syntrop.Runtime1",
        is_varlink: true,
        mode: 0o660,
        group: "syntrop",
    },
    SocketDefinition {
        daemon: "inferenced",
        address: "/run/syntrop/io.syntrop.Inference1",
        is_varlink: true,
        mode: 0o666,
        group: "syntrop",
    },
    SocketDefinition {
        daemon: "inferenced",
        address: "/run/syntrop/sentry.sock",
        is_varlink: false,
        mode: 0o660,
        group: "sentry",
    },
    SocketDefinition {
        daemon: "inferenced",
        address: "/run/syntrop/fd.sock",
        is_varlink: false,
        mode: 0o660,
        group: "inferenced",
    },
    SocketDefinition {
        daemon: "contextd",
        address: "/run/syntrop/io.syntrop.Context1",
        is_varlink: true,
        mode: 0o660,
        group: "syntrop",
    },
    SocketDefinition {
        daemon: "modeld",
        address: "/run/syntrop/io.syntrop.Model1",
        is_varlink: true,
        mode: 0o660,
        group: "modeld",
    },
    SocketDefinition {
        daemon: "modeld",
        address: "/run/syntrop/modeld-fd.sock",
        is_varlink: false,
        mode: 0o660,
        group: "modeld",
    },
    SocketDefinition {
        daemon: "sentry",
        address: "/run/systemd-sentry/sentry.sock",
        is_varlink: false,
        mode: 0o660,
        group: "sentry",
    },
];

/// Observed state of an IPC socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketHealth {
    /// Socket is present and accepting connections.
    ActiveListening,
    /// Socket file exists on filesystem but connection refused or timed out.
    InactiveFileExists,
    /// Socket file does not exist on filesystem.
    Missing,
    /// Path exists but is not a UNIX domain socket.
    NotASocket,
}

/// Detailed socket report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketReport {
    pub daemon: String,
    pub address: String,
    pub is_varlink: bool,
    pub health: SocketHealth,
}

/// Inspect all subsystem sockets and test liveness.
pub fn inspect_sockets() -> Vec<SocketReport> {
    ALL_SOCKETS
        .iter()
        .map(|def| {
            let health = probe_socket_health(def.address);
            SocketReport {
                daemon: def.daemon.to_string(),
                address: def.address.to_string(),
                is_varlink: def.is_varlink,
                health,
            }
        })
        .collect()
}

/// Probe a specific UNIX socket address.
pub fn probe_socket_health(addr: &str) -> SocketHealth {
    let path = Path::new(addr);
    if !path.exists() {
        return SocketHealth::Missing;
    }

    if let Ok(metadata) = path.metadata() {
        if !metadata.file_type().is_socket() {
            return SocketHealth::NotASocket;
        }
    } else {
        return SocketHealth::Missing;
    }

    // Try connecting to verify if systemd socket activation or daemon is actively listening
    match UnixStream::connect(path) {
        Ok(stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
            SocketHealth::ActiveListening
        }
        Err(_) => SocketHealth::InactiveFileExists,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use tempfile::tempdir;

    #[test]
    fn test_all_sockets_definition() {
        assert_eq!(ALL_SOCKETS.len(), 9);
        for s in ALL_SOCKETS {
            assert!(s.address.starts_with("/run/"));
        }
    }

    #[test]
    fn test_probe_missing_socket() {
        let health = probe_socket_health("/nonexistent/syntrop_test.sock");
        assert_eq!(health, SocketHealth::Missing);
    }

    #[test]
    fn test_probe_not_a_socket() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let health = probe_socket_health(temp.path().to_str().unwrap());
        assert_eq!(health, SocketHealth::NotASocket);
    }

    #[test]
    fn test_probe_active_listening_socket() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");
        let _listener = UnixListener::bind(&sock_path).unwrap();

        let health = probe_socket_health(sock_path.to_str().unwrap());
        assert_eq!(health, SocketHealth::ActiveListening);
    }
}
