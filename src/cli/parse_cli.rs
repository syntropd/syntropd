//! Parse the `syntropd` command line into typed commands.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "syntropd",
    about = "Native AI Subsystem for systemd — Umbrella Meta-Package & Supervisor",
    version,
    long_about = "syntropd integrates local AI acceleration, zero-idle socket activation, \
and autonomous failure triage directly into systemd as native OS primitives."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Display subsystem health, kernel capabilities, socket states, and daemon status
    Status {
        /// Output status in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Perform autonomous root-cause triage on a failed systemd unit
    Triage {
        /// Name of the failed systemd unit (e.g. nginx.service)
        unit: String,
        /// Output report in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Detailed root-cause explanation and remediation steps for a unit
    Explain {
        /// Name of the unit to explain
        unit: String,
        /// Output report in JSON format
        #[arg(long)]
        json: bool,
    },
    /// List all systemd units and sockets managed by syntropd
    Units {
        /// Filter by unit type (e.g. "service", "socket", "target")
        #[arg(long)]
        r#type: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Display version information for syntropd and all suite components
    Version {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_status() {
        let cli = Cli::try_parse_from(["syntropd", "status"]).unwrap();
        assert_eq!(cli.command, Commands::Status { json: false });

        let cli_json = Cli::try_parse_from(["syntropd", "status", "--json"]).unwrap();
        assert_eq!(cli_json.command, Commands::Status { json: true });
    }

    #[test]
    fn test_cli_parsing_triage() {
        let cli = Cli::try_parse_from(["syntropd", "triage", "nginx.service"]).unwrap();
        assert_eq!(
            cli.command,
            Commands::Triage {
                unit: "nginx.service".to_string(),
                json: false
            }
        );
    }

    #[test]
    fn test_cli_parsing_explain() {
        let cli = Cli::try_parse_from(["syntropd", "explain", "db.service", "--json"]).unwrap();
        assert_eq!(
            cli.command,
            Commands::Explain {
                unit: "db.service".to_string(),
                json: true
            }
        );
    }

    #[test]
    fn test_cli_parsing_units() {
        let cli = Cli::try_parse_from(["syntropd", "units", "--type", "socket"]).unwrap();
        assert_eq!(
            cli.command,
            Commands::Units {
                r#type: Some("socket".to_string()),
                json: false
            }
        );
    }

    #[test]
    fn test_cli_parsing_version() {
        let cli = Cli::try_parse_from(["syntropd", "version"]).unwrap();
        assert_eq!(cli.command, Commands::Version { json: false });
    }
}
