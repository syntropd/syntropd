//! Interactive triage decision command for pending Sentry incidents.
//!
//! Inspects pending incidents from `/run/syntrop/pending`, displays remediation
//! recipes with risk ratings, and dispatches approval to `syntropctl admin remediate`.

use clap::Parser;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;

const SENTRY_SOCKET: &str = "/run/systemd-sentry/sentry.sock";

/// Command line arguments for `syn decide`.
#[derive(Parser, Debug, Clone, Default)]
#[command(name = "syn decide", about = "Interactive triage and operator approval for autonomous remediations")]
pub struct DecideArgs {
    /// Target specific incident by ID to approve
    #[arg(long)]
    pub approve: Option<String>,
    /// Target specific incident by ID to reject
    #[arg(long)]
    pub reject: Option<String>,
    /// Automatically approve remediation without interactive prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Preview remediation without executing mutations
    #[arg(long)]
    pub dry_run: bool,
    /// Format output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct IncidentItem {
    pub file_path: Option<PathBuf>,
    pub id: String,
    pub unit: String,
    pub fault_class: String,
    pub proposed_recipe: String,
    pub risk_rating: String,
    pub explanation: String,
    pub journal: Vec<String>,
}

fn get_base_dir() -> PathBuf {
    std::env::var("RUNTIME_DIRECTORY").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/run/syntrop"))
}

/// Loads staged incidents from `/run/syntrop/pending`.
pub fn load_disk_incidents(base: &Path) -> Vec<IncidentItem> {
    let mut items = Vec::new();
    let Ok(entries) = fs::read_dir(base.join("pending")) else { return items; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(v) = serde_json::from_str::<Value>(&data) {
                    items.push(IncidentItem {
                        file_path: Some(path),
                        id: v.get("incident_id").and_then(|s| s.as_str()).unwrap_or("unknown").into(),
                        unit: v.get("unit").and_then(|s| s.as_str()).unwrap_or("unknown").into(),
                        fault_class: v.get("fault_class").and_then(|s| s.as_str()).unwrap_or("TransientCrash").into(),
                        proposed_recipe: v.get("proposed_action").and_then(|s| s.as_str()).unwrap_or("restart").into(),
                        risk_rating: v.get("tier").and_then(|s| s.as_str()).unwrap_or("LOW").into(),
                        explanation: v.get("explanation").and_then(|s| s.as_str()).unwrap_or("Systemd unit failure detected").into(),
                        journal: v.get("journal_excerpt").and_then(|j| j.as_array()).map(|arr| {
                            arr.iter().filter_map(|x| x.as_str().map(String::from)).collect()
                        }).unwrap_or_default(),
                    });
                }
            }
        }
    }
    items
}

/// Fallback: loads incidents from supervisor socket `/run/systemd-sentry/sentry.sock`.
pub fn load_socket_incidents() -> Vec<IncidentItem> {
    let sock = Path::new(SENTRY_SOCKET);
    if !sock.exists() { return Vec::new(); }
    let Ok(mut stream) = UnixStream::connect(sock) else { return Vec::new(); };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(50)));
    let req = serde_json::json!({ "type": "ListIncidents", "payload": { "limit": 10 } });
    if let Ok(mut payload) = serde_json::to_vec(&req) {
        payload.push(b'\n');
        let _ = stream.write_all(&payload);
    }
    let mut line = String::new();
    let Ok(_) = BufReader::new(stream).read_line(&mut line) else { return Vec::new(); };
    let Ok(resp) = serde_json::from_str::<Value>(&line) else { return Vec::new(); };
    let Some(data) = resp.get("data").and_then(|d| d.as_array()) else { return Vec::new(); };
    data.iter().map(|item| IncidentItem {
        file_path: None,
        id: item.get("incident_id").or_else(|| item.get("id")).and_then(|s| s.as_str()).unwrap_or("sentry-inc").into(),
        unit: item.get("unit").and_then(|s| s.as_str()).unwrap_or("unknown").into(),
        fault_class: item.get("fault_class").and_then(|s| s.as_str()).unwrap_or("ServiceFailure").into(),
        proposed_recipe: item.get("proposed_recipe").or_else(|| item.get("action")).and_then(|s| s.as_str()).unwrap_or("restart").into(),
        risk_rating: item.get("risk").or_else(|| item.get("tier")).and_then(|s| s.as_str()).unwrap_or("LOW").into(),
        explanation: item.get("explanation").and_then(|s| s.as_str()).unwrap_or("Service reported failed state").into(),
        journal: Vec::new(),
    }).collect()
}

/// Dispatches remediation execution to `syntropctl admin remediate`.
fn execute_remediation(unit: &str, recipe: &str, dry_run: bool) -> anyhow::Result<()> {
    let mut cmd = Command::new("syntropctl");
    cmd.args(["admin", "remediate", unit, "--recipe", recipe]);
    if dry_run { cmd.arg("--dry-run"); }
    let status = cmd.status()?;
    if !status.success() { anyhow::bail!("remediation command exited with {status}"); }
    Ok(())
}

/// Dispatches rollback execution to `syntropctl admin rollback`.
fn execute_rollback(unit: &str) -> anyhow::Result<()> {
    let status = Command::new("syntropctl").args(["admin", "rollback", unit]).status()?;
    if !status.success() { anyhow::bail!("rollback command exited with {status}"); }
    Ok(())
}

/// Interactively renders triage context and prompts the operator.
pub fn handle_syn_decide(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn decide".to_string()];
    parse_args.extend_from_slice(args);
    let cli = match DecideArgs::try_parse_from(&parse_args) {
        Ok(a) => a,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp || e.kind() == clap::error::ErrorKind::DisplayVersion {
                print!("{e}");
                return Ok(());
            }
            DecideArgs::default()
        }
    };
    let base = get_base_dir();
    let mut incidents = load_disk_incidents(&base);
    if incidents.is_empty() { incidents = load_socket_incidents(); }
    if incidents.is_empty() {
        if cli.json { println!("{}", serde_json::json!({ "pending": 0 })); } else { println!("No pending incidents for decision."); }
        return Ok(());
    }
    for inc in &incidents {
        if cli.json {
            println!("{}", serde_json::json!({ "incident_id": inc.id, "unit": inc.unit, "recipe": inc.proposed_recipe, "risk": inc.risk_rating }));
            continue;
        }
        println!("\n⚡ [Syntrop Sentry Incident Triage]\nIncident ID:  {}\nFailed Unit:  {}\nFault Class:  {}\nRecipe:       {}\nRisk Rating:  {}\nExplanation:  {}",
            inc.id, inc.unit, inc.fault_class, inc.proposed_recipe, inc.risk_rating, inc.explanation);
        if cli.yes {
            println!("Auto-approving remediation (--yes specified)...");
            execute_remediation(&inc.unit, &inc.proposed_recipe, cli.dry_run)?;
            if let Some(ref p) = inc.file_path { let _ = fs::remove_file(p); }
            continue;
        }
        loop {
            print!("\nApprove remediation? [y/N/details/rollback]: ");
            std::io::stdout().flush()?;
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice)?;
            let trimmed = choice.trim().to_ascii_lowercase();
            match trimmed.as_str() {
                "y" | "yes" => {
                    execute_remediation(&inc.unit, &inc.proposed_recipe, cli.dry_run)?;
                    if let Some(ref p) = inc.file_path { let _ = fs::remove_file(p); }
                    println!("Remediation executed for {}", inc.unit);
                    break;
                }
                "details" | "d" => {
                    println!("\n--- Incident Journal Details ---");
                    for line in &inc.journal { println!("  {line}"); }
                    if inc.journal.is_empty() {
                        let _ = Command::new("journalctl").args(["-u", &inc.unit, "-n", "15", "--no-pager"]).status();
                    }
                    println!("--------------------------------");
                }
                "rollback" | "r" => {
                    execute_rollback(&inc.unit)?;
                    println!("Rollback requested for {}", inc.unit);
                    break;
                }
                _ => {
                    println!("Remediation skipped for {}.", inc.unit);
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_args_parsing() {
        let args = DecideArgs::try_parse_from(["syn decide", "--yes", "--dry-run"]).unwrap();
        assert!(args.yes && args.dry_run && !args.json);
    }

    #[test]
    fn test_decide_no_incidents() {
        let dir = std::env::temp_dir().join(format!("test_decide_empty_{}", std::process::id()));
        let _ = fs::create_dir_all(dir.join("pending"));
        std::env::set_var("RUNTIME_DIRECTORY", &dir);
        let items = load_disk_incidents(&dir);
        assert!(items.is_empty());
        let res = handle_syn_decide(&["--json".to_string()]);
        assert!(res.is_ok());
        let _ = fs::remove_dir_all(&dir);
    }
}
