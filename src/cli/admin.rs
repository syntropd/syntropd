//! Autonomous self-healing administration command parser and dispatcher.

use super::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn admin`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn admin",
    about = "Autonomous OS self-healing and administration"
)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: Option<AdminSubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Supported subcommands for `syn admin`.
#[derive(Subcommand, Debug, Clone)]
pub enum AdminSubcommand {
    /// Display overall autonomous healing status, active circuit-breaker states, and locked-out units
    Status,

    /// Trigger a guided or automated remediation recipe for a failed service
    Remediate {
        /// Target systemd unit
        unit: String,

        /// Remediation recipe name
        #[arg(long)]
        recipe: Option<String>,

        /// Preview remediation steps without applying mutations
        #[arg(long)]
        dry_run: bool,
    },

    /// Roll back state or configuration modifications executed during remediation
    Rollback {
        /// Target systemd unit
        unit: String,

        /// Optional snapshot identifier
        #[arg(long)]
        snapshot: Option<String>,
    },

    /// Inspect immutable forensic records of all self-healing actions
    Audit {
        /// Filter audit records by service unit name
        #[arg(short = 'u', long = "unit")]
        unit: Option<String>,

        /// Maximum number of audit records to return
        #[arg(short = 'n', long = "limit", default_value = "50")]
        limit: usize,
    },

    /// Inspect or reset circuit-breaker lockout states
    Lockout {
        #[command(subcommand)]
        command: AdminLockoutSubcommand,
    },
}

/// Subcommands under `syn admin lockout`.
#[derive(Subcommand, Debug, Clone)]
pub enum AdminLockoutSubcommand {
    /// Manually reset circuit-breaker lockout after operator inspection
    Reset {
        /// Target systemd unit
        unit: String,
    },
}

/// Dispatches `syn admin` commands directly to `syntropctl admin`.
pub fn handle_syn_admin(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn admin".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = AdminArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'admin' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["admin".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_args_parse_status() {
        let args = AdminArgs::try_parse_from(["syn admin", "status"]).unwrap();
        assert!(matches!(args.command, Some(AdminSubcommand::Status)));
    }

    #[test]
    fn test_admin_args_parse_remediate() {
        let args = AdminArgs::try_parse_from([
            "syn admin",
            "remediate",
            "nginx.service",
            "--recipe",
            "restart",
            "--dry-run",
        ])
        .unwrap();
        match args.command {
            Some(AdminSubcommand::Remediate {
                unit,
                recipe,
                dry_run,
            }) => {
                assert_eq!(unit, "nginx.service");
                assert_eq!(recipe.as_deref(), Some("restart"));
                assert!(dry_run);
            }
            _ => panic!("Expected Remediate"),
        }
    }

    #[test]
    fn test_admin_args_parse_lockout_reset() {
        let args = AdminArgs::try_parse_from([
            "syn admin",
            "lockout",
            "reset",
            "caddy.service",
        ])
        .unwrap();
        match args.command {
            Some(AdminSubcommand::Lockout {
                command: AdminLockoutSubcommand::Reset { unit },
            }) => {
                assert_eq!(unit, "caddy.service");
            }
            _ => panic!("Expected Lockout Reset"),
        }
    }
}
