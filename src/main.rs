//! syntropd — Native AI Subsystem for systemd CLI

use clap::{Parser, Subcommand};
use syntropd::sockets::SocketHealth;
use syntropd::{inspect_subsystem, suite, triage, units};

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { json } => handle_status(json),
        Commands::Triage { unit, json } => handle_triage(&unit, json),
        Commands::Explain { unit, json } => handle_explain(&unit, json),
        Commands::Units { r#type, json } => handle_units(r#type.as_deref(), json),
        Commands::Version { json } => handle_version(json),
    }
}

fn handle_status(json: bool) -> anyhow::Result<()> {
    let status = inspect_subsystem();

    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!("============================================================");
    println!(" syntropd Subsystem Status — Native AI for systemd (v{})", status.version);
    println!("============================================================");

    println!("\n[Kernel & OS Capabilities]");
    println!("  PID 1 Systemd:       {}", if status.capabilities.systemd_running { "Running (PASS)" } else { "Not Detected (FAIL)" });
    if let Some(ref ver) = status.capabilities.systemd_version {
        println!("  Systemd Version:     {}", ver);
    }
    println!("  cgroups v2:          {}", if status.capabilities.cgroups_v2 { "Active (PASS)" } else { "Disabled/Legacy (FAIL)" });
    if let Some(ref ctrl) = status.capabilities.cgroup_controllers {
        println!("  Controllers:         {}", ctrl);
    }
    println!("  Memory PSI:          {}", if status.capabilities.psi_available { "Available (PASS)" } else { "Unavailable (FAIL)" });
    println!("  DRM Render Nodes:    {}", if status.capabilities.dri_devices.is_empty() { "None detected".to_string() } else { status.capabilities.dri_devices.join(", ") });
    println!("  AI Accelerators:     {}", if status.capabilities.accel_devices.is_empty() { "None detected".to_string() } else { status.capabilities.accel_devices.join(", ") });
    println!("  Group 'syntrop':     {}", if status.capabilities.syntrop_group_exists { "Present" } else { "Missing" });
    println!("  User 'sentry':       {}", if status.capabilities.sentry_user_exists { "Present" } else { "Missing" });

    println!("\n[IPC & Varlink Sockets]");
    println!("  {:<12} {:<36} {:<10}", "DAEMON", "SOCKET ADDRESS", "STATE");
    println!("  {:-<12} {:-<36} {:-<10}", "", "", "");
    for s in &status.sockets {
        let state_str = match s.health {
            SocketHealth::ActiveListening => "LISTENING",
            SocketHealth::InactiveFileExists => "INACTIVE",
            SocketHealth::Missing => "MISSING",
            SocketHealth::NotASocket => "INVALID",
        };
        println!("  {:<12} {:<36} {:<10}", s.daemon, s.address, state_str);
    }

    println!("\n[Systemd Units]");
    println!("  {:<26} {:<8} {:<12} {:<14}", "UNIT", "TYPE", "INSTALLED", "ACTIVE STATE");
    println!("  {:-<26} {:-<8} {:-<12} {:-<14}", "", "", "", "");
    for u in &status.units {
        let inst_str = if u.installed { "yes" } else { "no" };
        println!("  {:<26} {:<8} {:<12} {:<14}", u.name, u.unit_type, inst_str, u.active_state);
    }

    println!();
    Ok(())
}

fn handle_triage(unit: &str, json: bool) -> anyhow::Result<()> {
    let report = triage::triage_unit(unit);

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("============================================================");
    println!(" syntropd Autonomous Triage: {}", report.unit);
    println!("============================================================");
    println!("  State:          {}", report.active_state);
    println!("  Fault Category: {:?}", report.fault_category);
    println!("  Summary:        {}", report.summary);
    println!("\n  Recommendation:\n    {}", report.recommendation);

    if !report.journal_excerpt.is_empty() {
        println!("\n  Recent Journal Output:");
        for line in &report.journal_excerpt {
            println!("    {}", line);
        }
    }
    println!();
    Ok(())
}

fn handle_explain(unit: &str, json: bool) -> anyhow::Result<()> {
    handle_triage(unit, json)
}

fn handle_units(r#type: Option<&str>, json: bool) -> anyhow::Result<()> {
    let mut all = units::inspect_all_units();
    if let Some(t) = r#type {
        all.retain(|u| u.unit_type.eq_ignore_ascii_case(t));
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
        return Ok(());
    }

    println!("============================================================");
    println!(" syntropd Subsystem Units Catalog");
    println!("============================================================");
    println!("  {:<26} {:<8} {:<10} {:<12} {:<24}", "UNIT", "TYPE", "UMBRELLA", "INSTALLED", "ACTIVE");
    println!("  {:-<26} {:-<8} {:-<10} {:-<12} {:-<24}", "", "", "", "", "");

    for u in &all {
        let umb_str = if u.is_umbrella { "yes" } else { "no" };
        let inst_str = if u.installed { "yes" } else { "no" };
        println!("  {:<26} {:<8} {:<10} {:<12} {:<24}", u.name, u.unit_type, umb_str, inst_str, u.active_state);
    }
    println!();
    Ok(())
}

fn handle_version(json: bool) -> anyhow::Result<()> {
    let components = suite::get_components();
    let version = env!("CARGO_PKG_VERSION");

    #[derive(serde::Serialize)]
    struct VersionInfo<'a> {
        syntropd_umbrella_version: &'static str,
        license: &'static str,
        components: &'a [suite::SuiteComponent],
    }

    let info = VersionInfo {
        syntropd_umbrella_version: version,
        license: "Apache-2.0",
        components,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    println!("syntropd {} (Apache-2.0)", version);
    println!("Native AI Subsystem for systemd — Umbrella Meta-Package");
    println!("\nIncluded Suite Components:");
    for c in components {
        println!("  {:<12} (crate: {:<20} v{:<6})", c.name, c.package, c.version);
        println!("    Role: {}", c.role);
    }
    println!();
    Ok(())
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
