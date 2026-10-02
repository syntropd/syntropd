//! Kernel telemetry and dynamic closed-loop tuning command dispatcher.

use super::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn telemetry`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn telemetry",
    about = "Zero-allocation kernel telemetry and closed-loop tuning governor"
)]
pub struct TelemetryArgs {
    #[command(subcommand)]
    pub command: Option<TelemetrySubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Subcommands supported under `syn telemetry`.
#[derive(Subcommand, Debug, Clone)]
pub enum TelemetrySubcommand {
    /// Inspect non-blocking kernel PSI pressure and eBPF runqueue latency
    Status,

    /// Query or configure dynamic closed-loop PSI tuning policy
    Tune {
        /// Tuning policy preset (conservative, balanced, aggressive)
        #[arg(short = 'p', long = "policy")]
        policy: Option<String>,
    },
}

/// Command line arguments for `syn tune`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn tune",
    about = "Configure dynamic closed-loop PSI tuning policy and draft horizons"
)]
pub struct TuneArgs {
    /// Tuning policy preset (conservative, balanced, aggressive)
    #[arg(short = 'p', long = "policy")]
    pub policy: Option<String>,

    /// Format output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Dispatches `syn telemetry` commands directly to `syntropctl telemetry`.
pub fn handle_syn_telemetry(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn telemetry".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = TelemetryArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'telemetry' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["telemetry".to_string()];
    if args.is_empty() {
        exec_args.push("status".to_string());
    } else {
        exec_args.extend_from_slice(args);
    }
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

/// Dispatches `syn tune` shorthand directly to `syntropctl telemetry tune`.
pub fn handle_syn_tune(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn tune".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = TuneArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'tune' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["telemetry".to_string(), "tune".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_args_parse_status() {
        let args = TelemetryArgs::try_parse_from(["syn telemetry", "status"]).unwrap();
        assert!(matches!(args.command, Some(TelemetrySubcommand::Status)));
    }

    #[test]
    fn test_telemetry_args_parse_tune() {
        let args = TelemetryArgs::try_parse_from([
            "syn telemetry",
            "tune",
            "--policy",
            "aggressive",
            "--json",
        ])
        .unwrap();
        match args.command {
            Some(TelemetrySubcommand::Tune { policy }) => {
                assert_eq!(policy.as_deref(), Some("aggressive"));
                assert!(args.json);
            }
            _ => panic!("Expected Tune"),
        }
    }

    #[test]
    fn test_tune_args_parse() {
        let args = TuneArgs::try_parse_from(["syn tune", "-p", "balanced", "--json"]).unwrap();
        assert_eq!(args.policy.as_deref(), Some("balanced"));
        assert!(args.json);
    }
}
