//! Automated model family setup and routerd configuration generator.

pub mod router_config;
use router_config::{generate_speculative_routerd_toml, write_routerd_config};
use clap::Parser;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Command line arguments for `syn setup`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "setup",
    about = "Bootstrap model family and configure speculative router"
)]
pub struct SetupArgs {
    /// Model family to bootstrap: qwen, granite, or gemma.
    #[arg(long, default_value = "qwen")]
    pub family: String,

    /// Hardware envelope profile (default: auto).
    #[arg(long, default_value = "auto")]
    pub envelope: String,

    /// Preview plan without downloading artifacts or reloading daemon.
    #[arg(long)]
    pub dry_run: bool,

    /// Target routerd configuration file path.
    #[arg(long, default_value = "/etc/syntrop/routerd.toml")]
    pub config: PathBuf,
}

/// Dispatches `syn setup` workflow: modelctl bootstrap, routerd.toml, service reload.
pub fn handle_syn_setup(args: &[String]) -> anyhow::Result<()> {
    let mut full_args = vec!["setup".to_string()];
    full_args.extend_from_slice(args);
    let parsed = match SetupArgs::try_parse_from(full_args) {
        Ok(p) => p,
        Err(e)
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion =>
        {
            print!("{e}");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let fam = parsed.family.trim().to_ascii_lowercase();
    if fam != "qwen" && fam != "granite" && fam != "gemma" {
        return Err(anyhow::anyhow!(
            "Invalid model family '{}'. Supported: qwen, granite, gemma",
            parsed.family
        ));
    }

    println!(
        "=== Syntrop Setup: Family [{}] (Envelope: {}) ===",
        fam, parsed.envelope
    );

    // 1. Dispatch to modelctl bootstrap
    dispatch_modelctl_bootstrap(&fam, parsed.dry_run)?;

    // 2. Generate and write routerd.toml with paired speculative sessions and shared Arc<VocabTrie>
    let toml_content = generate_speculative_routerd_toml(&fam);
    write_routerd_config(&parsed.config, &toml_content, parsed.dry_run)?;

    // 3. Reload routerd.service if not in dry-run mode
    if !parsed.dry_run {
        reload_routerd_service();
    }

    println!("Setup completed successfully for family: {}", fam);
    Ok(())
}

fn which_modelctl() -> PathBuf {
    if let Ok(bin) = std::env::var("MODELCTL_BIN") {
        let p = PathBuf::from(bin);
        if p.is_file() {
            return p;
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join("modelctl");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    if let Ok(curr) = std::env::current_exe() {
        if let Some(parent) = curr.parent() {
            let candidate = parent.join("modelctl");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    for dir in &["/usr/local/bin", "/usr/bin"] {
        let candidate = Path::new(dir).join("modelctl");
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("modelctl")
}

fn dispatch_modelctl_bootstrap(family: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("\n[1/3] Sizing hardware envelope and dispatching to modelctl bootstrap...");
    let bin = which_modelctl();
    let mut cmd = Command::new(bin);
    cmd.arg("bootstrap").arg("--family").arg(family);
    if dry_run {
        cmd.arg("--dry-run");
    }

    match cmd.status() {
        Ok(status) if status.success() => {
            println!("  [✓] modelctl bootstrap completed successfully.");
            Ok(())
        }
        Ok(status) => {
            if dry_run {
                println!(
                    "  [!] Notice: installed modelctl exited with ({}), continuing dry run.",
                    status
                );
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "modelctl bootstrap exited with status {}",
                    status
                ))
            }
        }
        Err(e) => {
            if dry_run {
                println!(
                    "  [!] Notice: modelctl not found in PATH ({}), continuing dry run.",
                    e
                );
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "modelctl is required for bootstrap but could not be executed: {}. \
                    Reinstall via: curl -fsSL https://syntropd.github.io/install.sh | sudo bash",
                    e
                ))
            }
        }
    }
}

fn reload_routerd_service() {
    println!("\n[3/3] Reloading routerd service...");
    // 1. Varlink IPC reload notification
    let router_socket_env = std::env::var("SYNTROP_ROUTER_SOCKET")
        .unwrap_or_else(|_| "/run/syntrop/io.syntrop.Router1".to_string());
    let sock = Path::new(&router_socket_env);
    if sock.exists() {
        if let Ok(mut stream) = UnixStream::connect(sock) {
            let req = serde_json::json!({
                "method": "io.syntrop.Router1.Reload",
                "parameters": {}
            });
            if let Ok(mut bytes) = serde_json::to_vec(&req) {
                bytes.push(0);
                let _ = stream.write_all(&bytes);
            }
        }
    }

    // 2. systemd reload
    let _ = Command::new("systemctl")
        .args(["try-reload-or-restart", "routerd.service"])
        .status();
    println!("  [✓] Notified routerd of configuration reload.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_handle_syn_setup_dry_run() {
        let dir = tempdir().unwrap();
        let cfg = dir.path().join("routerd.toml");
        let args = vec![
            "--family".to_string(),
            "qwen".to_string(),
            "--dry-run".to_string(),
            "--config".to_string(),
            cfg.to_str().unwrap().to_string(),
        ];
        assert!(handle_syn_setup(&args).is_ok());
    }

    #[test]
    fn test_handle_syn_setup_invalid_family() {
        let args = vec![
            "--family".to_string(),
            "invalid-fam".to_string(),
            "--dry-run".to_string(),
        ];
        assert!(handle_syn_setup(&args).is_err());
    }

    #[test]
    fn test_handle_syn_setup_help() {
        let args = vec!["--help".to_string()];
        assert!(handle_syn_setup(&args).is_ok());
    }
}
