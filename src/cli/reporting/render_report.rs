//! Render subsystem reports for each CLI command.

use crate::sockets::SocketHealth;
use crate::{inspect_subsystem, suite, triage, units};

pub fn handle_status(json: bool) -> anyhow::Result<()> {
    let status = inspect_subsystem();

    if json {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!("============================================================");
    println!(
        " syntropd Subsystem Status — Native AI for systemd (v{})",
        status.version
    );
    println!("============================================================");

    println!("\n[Kernel & OS Capabilities]");
    println!(
        "  PID 1 Systemd:       {}",
        if status.capabilities.systemd_running {
            "Running (PASS)"
        } else {
            "Not Detected (FAIL)"
        }
    );
    if let Some(ref ver) = status.capabilities.systemd_version {
        println!("  Systemd Version:     {}", ver);
    }
    println!(
        "  cgroups v2:          {}",
        if status.capabilities.cgroups_v2 {
            "Active (PASS)"
        } else {
            "Disabled/Legacy (FAIL)"
        }
    );
    if let Some(ref ctrl) = status.capabilities.cgroup_controllers {
        println!("  Controllers:         {}", ctrl);
    }
    println!(
        "  Memory PSI:          {}",
        if status.capabilities.psi_available {
            "Available (PASS)"
        } else {
            "Unavailable (FAIL)"
        }
    );
    println!(
        "  DRM Render Nodes:    {}",
        if status.capabilities.dri_devices.is_empty() {
            "None detected".to_string()
        } else {
            status.capabilities.dri_devices.join(", ")
        }
    );
    println!(
        "  AI Accelerators:     {}",
        if status.capabilities.accel_devices.is_empty() {
            "None detected".to_string()
        } else {
            status.capabilities.accel_devices.join(", ")
        }
    );
    println!(
        "  Group 'syntrop':     {}",
        if status.capabilities.syntrop_group_exists {
            "Present"
        } else {
            "Missing"
        }
    );
    println!(
        "  User 'sentry':       {}",
        if status.capabilities.sentry_user_exists {
            "Present"
        } else {
            "Missing"
        }
    );

    println!("\n[IPC & Varlink Sockets]");
    println!(
        "  {:<12} {:<36} {:<10}",
        "DAEMON", "SOCKET ADDRESS", "STATE"
    );
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
    println!(
        "  {:<26} {:<8} {:<12} {:<14}",
        "UNIT", "TYPE", "INSTALLED", "ACTIVE STATE"
    );
    println!("  {:-<26} {:-<8} {:-<12} {:-<14}", "", "", "", "");
    for u in &status.units {
        let inst_str = if u.installed { "yes" } else { "no" };
        println!(
            "  {:<26} {:<8} {:<12} {:<14}",
            u.name, u.unit_type, inst_str, u.active_state
        );
    }

    println!();
    Ok(())
}

pub fn handle_triage(unit: &str, json: bool) -> anyhow::Result<()> {
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

pub fn handle_explain(unit: &str, json: bool) -> anyhow::Result<()> {
    handle_triage(unit, json)
}

pub fn handle_units(r#type: Option<&str>, json: bool) -> anyhow::Result<()> {
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
    println!(
        "  {:<26} {:<8} {:<10} {:<12} {:<24}",
        "UNIT", "TYPE", "UMBRELLA", "INSTALLED", "ACTIVE"
    );
    println!(
        "  {:-<26} {:-<8} {:-<10} {:-<12} {:-<24}",
        "", "", "", "", ""
    );

    for u in &all {
        let umb_str = if u.is_umbrella { "yes" } else { "no" };
        let inst_str = if u.installed { "yes" } else { "no" };
        println!(
            "  {:<26} {:<8} {:<10} {:<12} {:<24}",
            u.name, u.unit_type, umb_str, inst_str, u.active_state
        );
    }
    println!();
    Ok(())
}

pub fn handle_version(json: bool) -> anyhow::Result<()> {
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
        println!(
            "  {:<12} (crate: {:<20} v{:<6})",
            c.name, c.package, c.version
        );
        println!("    Role: {}", c.role);
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_version_json() {
        handle_version(true).unwrap();
    }

    #[test]
    fn test_handle_version_text() {
        handle_version(false).unwrap();
    }

    #[test]
    fn test_handle_units_json_filtered() {
        handle_units(Some("service"), true).unwrap();
        handle_units(None, true).unwrap();
    }

    #[test]
    fn test_handle_status_json() {
        handle_status(true).unwrap();
    }

    #[test]
    fn test_handle_triage_and_explain_json() {
        handle_triage("definitely_nonexistent_unit_xyz.service", true).unwrap();
        handle_explain("definitely_nonexistent_unit_xyz.service", true).unwrap();
    }
}
