//! Operator shell prompt incident decorator.
//!
//! Sub-millisecond non-blocking probe against `/run/syntrop/pending` and
//! `/run/systemd-sentry/sentry.sock` for pending incident notifications.

use clap::Parser;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const SENTRY_SOCKET: &str = "/run/systemd-sentry/sentry.sock";
const SOCKET_TIMEOUT_MS: u64 = 5;

/// Command line arguments for `syn prompt`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn prompt",
    about = "Sub-millisecond shell prompt hook for pending incident indication"
)]
pub struct PromptArgs {
    /// Emit only the raw integer incident count
    #[arg(long)]
    pub raw: bool,

    /// Emit incident state as structured JSON
    #[arg(long)]
    pub json: bool,
}

/// Pending incident summary retrieved from local staging or supervisor IPC.
#[derive(Debug, Clone, Default)]
pub struct IncidentSummary {
    pub count: usize,
    pub unit: Option<String>,
}

fn get_base_dir() -> PathBuf {
    std::env::var("RUNTIME_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/syntrop"))
}

/// Checks `/run/syntrop/pending` for staged incidents.
fn check_pending_disk(base: &Path) -> Option<IncidentSummary> {
    let pending_dir = base.join("pending");
    let entries = fs::read_dir(pending_dir).ok()?;

    let mut count = 0;
    let mut first_unit = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            count += 1;
            if first_unit.is_none() {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(v) = serde_json::from_str::<Value>(&data) {
                        if let Some(u) = v.get("unit").and_then(|s| s.as_str()) {
                            first_unit = Some(u.to_string());
                        }
                    }
                }
            }
        }
    }

    if count > 0 {
        Some(IncidentSummary {
            count,
            unit: first_unit,
        })
    } else {
        None
    }
}

/// Queries `/run/systemd-sentry/sentry.sock` with a strict <= 5ms timeout.
fn check_sentry_socket() -> Option<IncidentSummary> {
    let sock_path = Path::new(SENTRY_SOCKET);
    if !sock_path.exists() {
        return None;
    }

    let mut stream = UnixStream::connect(sock_path).ok()?;
    let timeout = Some(Duration::from_millis(SOCKET_TIMEOUT_MS));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);

    let req = serde_json::json!({
        "type": "ListIncidents",
        "payload": { "limit": 10 }
    });
    let mut payload = serde_json::to_vec(&req).ok()?;
    payload.push(b'\n');

    stream.write_all(&payload).ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;

    let resp: Value = serde_json::from_str(&line).ok()?;
    if resp.get("status").and_then(|s| s.as_str()) == Some("ok") {
        if let Some(data) = resp.get("data").and_then(|d| d.as_array()) {
            if !data.is_empty() {
                let count = data.len();
                let unit = data[0]
                    .get("unit")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());
                return Some(IncidentSummary { count, unit });
            }
        }
    }

    None
}

/// Dispatches `syn prompt` and emits decoration badge or raw count.
pub fn handle_syn_prompt(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn prompt".to_string()];
    parse_args.extend_from_slice(args);

    let cli = match PromptArgs::try_parse_from(&parse_args) {
        Ok(a) => a,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                print!("{e}");
                return Ok(());
            }
            PromptArgs {
                raw: false,
                json: false,
            }
        }
    };

    let base = get_base_dir();
    let summary = check_pending_disk(&base)
        .or_else(check_sentry_socket)
        .unwrap_or_default();

    if cli.raw {
        println!("{}", summary.count);
        return Ok(());
    }

    if cli.json {
        let out = serde_json::json!({
            "pending": summary.count,
            "unit": summary.unit,
        });
        println!("{out}");
        return Ok(());
    }

    // Zero incidents pending: produce 0 bytes output immediately (<1ms latency).
    if summary.count == 0 {
        return Ok(());
    }

    let unit_desc = summary.unit.as_deref().unwrap_or("unknown");
    if summary.count == 1 {
        println!("⚡ [Syntrop Sentry] 1 incident pending ({unit_desc} failed). Run 'syn decide'.");
    } else {
        println!(
            "⚡ [Syntrop Sentry] {} incidents pending ({unit_desc} failed). Run 'syn decide'.",
            summary.count
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_args_parsing() {
        let args = PromptArgs::try_parse_from(["syn prompt", "--raw"]).unwrap();
        assert!(args.raw);
        assert!(!args.json);

        let args_json = PromptArgs::try_parse_from(["syn prompt", "--json"]).unwrap();
        assert!(args_json.json);
        assert!(!args_json.raw);
    }

    #[test]
    fn test_prompt_clean_no_output() {
        let dir = std::env::temp_dir().join(format!("test_prompt_clean_{}", std::process::id()));
        let _ = fs::create_dir_all(dir.join("pending"));
        std::env::set_var("RUNTIME_DIRECTORY", &dir);

        let res = handle_syn_prompt(&[]);
        assert!(res.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_prompt_pending_incident_detected() {
        let dir = std::env::temp_dir().join(format!("test_prompt_inc_{}", std::process::id()));
        let pending = dir.join("pending");
        let _ = fs::create_dir_all(&pending);
        let inc_file = pending.join("inc-1.json");
        fs::write(
            &inc_file,
            serde_json::json!({
                "incident_id": "inc-001",
                "unit": "test-service.service",
            })
            .to_string(),
        )
        .unwrap();

        let summary = check_pending_disk(&dir).expect("should find incident");
        assert_eq!(summary.count, 1);
        assert_eq!(summary.unit.as_deref(), Some("test-service.service"));

        let _ = fs::remove_dir_all(&dir);
    }
}
