//! `syntrop` — front door to the suite.
//!
//! Thin router over the per-repo CLIs. It resolves the first argument to a
//! namespace and `exec`s the owning binary with the rest untouched, so help
//! text, flags, pipes, and exit codes behave exactly like the repo tool.
//! No flags are duplicated here; the repo CLIs stay the source of truth.

use clap::Parser;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    name = "syntrop",
    about = "Front door to the syntropd suite",
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// Namespace: router, runtime, store, hardware, context, tools, fleet, system
    #[arg(allow_hyphen_values = true)]
    namespace: Option<String>,

    /// Arguments passed through untouched to the repo CLI
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// (namespace, repo binary, one-line description)
const NAMESPACES: &[(&str, &str, &str)] = &[
    ("router", "routerctl", "Talk to LLMs: setup, models, default, test"),
    ("runtime", "runtimectl", "Run local models directly"),
    ("store", "modelctl", "Model file storage: list, import, prune"),
    ("hardware", "inferenctl", "GPU/accelerator planes and leases"),
    ("context", "contextctl", "System history and config drift"),
    ("tools", "toolctl", "Sandboxed repair tools and rollback"),
    ("fleet", "syntropctl", "Fleet health and failure forensics"),
    ("system", "syntropd", "Units, triage, and suite version"),
];

fn resolve(namespace: &str) -> Option<&'static str> {
    NAMESPACES
        .iter()
        .find(|(ns, _, _)| *ns == namespace)
        .map(|(_, bin, _)| *bin)
}

/// Closest namespace within a small edit distance, for "did you mean?".
fn suggest(unknown: &str) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
    for (ns, _, _) in NAMESPACES {
        let d = edit_distance(unknown, ns);
        if d <= 2 && best.map(|(_, b)| d < b).unwrap_or(true) {
            best = Some((ns, d));
        }
    }
    best.map(|(ns, _)| ns)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

fn find_in_path(bin: &str) -> bool {
    if bin.contains('/') {
        return is_executable(std::path::Path::new(bin));
    }
    std::env::var_os("PATH").map(|paths| {
        std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(bin)))
    }).unwrap_or(false)
}

fn is_executable(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn print_overview() {
    println!("syntrop {} — front door to the suite", env!("CARGO_PKG_VERSION"));
    println!();
    for (ns, bin, desc) in NAMESPACES {
        println!("  {:<8} {} ({})", ns, desc, bin);
    }
    println!();
    println!("usage: syn <namespace> <command> [args...]");
    println!("examples:");
    println!("  syn router models");
    println!("  syn fleet status");
    println!("  syn system units");
    println!("help:  syn <namespace> --help");
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Help/version are handled manually (never intercepted) so that
    // `syn <ns> --help` always reaches the repo CLI.
    let Some(ns) = cli.namespace.as_deref() else {
        print_overview();
        return Ok(());
    };
    if ns == "--help" || ns == "-h" {
        print_overview();
        return Ok(());
    }
    if ns == "--version" || ns == "-V" {
        println!("syntrop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let Some(bin) = resolve(ns) else {
        match suggest(ns) {
            Some(s) => eprintln!("unknown namespace '{}'. did you mean '{}'?", ns, s),
            None => eprintln!(
                "unknown namespace '{}'. namespaces: {}",
                ns,
                NAMESPACES.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().join(", ")
            ),
        }
        std::process::exit(2);
    };

    if !find_in_path(bin) {
        eprintln!(
            "namespace '{}' needs '{}', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash",
            ns, bin
        );
        std::process::exit(127);
    }

    // Replace this process: signals, pipes, and exit codes behave exactly
    // like invoking the repo CLI directly.
    let err = Command::new(bin).args(&cli.args).exec();
    Err(anyhow::anyhow!("failed to exec '{}': {}", bin, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_namespace_resolves() {
        for (ns, bin, _) in NAMESPACES {
            assert_eq!(resolve(ns), Some(*bin));
        }
        assert_eq!(resolve("nope"), None);
    }

    #[test]
    fn suggestions_catch_typos() {
        assert_eq!(suggest("rounter"), Some("router"));
        assert_eq!(suggest("runetime"), Some("runtime"));
        assert_eq!(suggest("xyz"), None);
    }

    #[test]
    fn hyphen_flags_pass_through() {
        let cli = Cli::try_parse_from(["syn", "router", "--help"]).unwrap();
        assert_eq!(cli.namespace.as_deref(), Some("router"));
        assert_eq!(cli.args, vec!["--help".to_string()]);
        let cli = Cli::try_parse_from(["syn", "--help"]).unwrap();
        assert_eq!(cli.namespace.as_deref(), Some("--help"));
        let cli = Cli::try_parse_from(["syn", "runtime", "generate", "--prompt", "hi"]).unwrap();
        assert_eq!(cli.namespace.as_deref(), Some("runtime"));
        assert_eq!(cli.args.len(), 3);
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("router", "router"), 0);
        assert_eq!(edit_distance("rounter", "router"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }
}
