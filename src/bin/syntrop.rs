//! `syntrop` — front door to the suite.
//!
//! Thin router over the per-repo CLIs. It resolves the first argument to a
//! namespace and `exec`s the owning binary with the rest untouched, so help
//! text, flags, pipes, and exit codes behave exactly like the repo tool.
//! Bare words that are not a namespace are a question: they `exec`
//! `routerctl ask` with the words untouched.
//! Intercepts `syn pull <model>` and dispatches directly to `modelctl pull`.
//! No flags are duplicated here; the repo CLIs stay the source of truth.

use clap::Parser;
use std::os::unix::process::CommandExt;
use std::process::Command;
use syntropd::cli::dispatch::{
    find_in_path, is_bare_prompt, normalize_prompt_argv, resolve, suggest, Cli, NAMESPACES,
};

/// Replace this process with `routerctl ask <words...>`; the reply,
/// pipes, and exit code behave exactly like invoking it directly.
fn exec_prompt(first: &str, rest: &[String]) -> anyhow::Result<()> {
    if !find_in_path("routerctl") {
        eprintln!(
            "asking needs 'routerctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }
    let err = Command::new("routerctl")
        .args(normalize_prompt_argv(first, rest))
        .exec();
    Err(anyhow::anyhow!("failed to exec 'routerctl': {}", err))
}

fn print_overview() {
    println!(
        "syntrop {} — front door to the suite",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    for (ns, bin, desc) in NAMESPACES {
        println!("  {:<8} {} ({})", ns, desc, bin);
    }
    println!(
        "  {:<8} Pull and register a model directly (modelctl)",
        "pull"
    );
    println!();
    println!("usage: syn <question> | syn <namespace> <command> [args...] | syn pull <model> | syn decide | syn audit");
    println!("effort: -e, --effort <tier>  (none, low, med, high, max; defaults to 0 tokens on CPU / tight memory, 1,024 on GPU with healthy VRAM)");
    println!("examples:");
    println!("  syn say hello in one sentence");
    println!("  syn -e low explain quantum computing");
    println!("  syn pull qwen2.5:0.5b");
    println!("  syn decide");
    println!("  syn audit");
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

    // Intercept `syn pull <args...>` and dispatch to `modelctl pull <args...>`.
    if ns == "pull" {
        if !find_in_path("modelctl") {
            eprintln!(
                "pulling models needs 'modelctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
            );
            std::process::exit(127);
        }
        let mut pull_args = vec!["pull".to_string()];
        pull_args.extend_from_slice(&cli.args);
        let err = Command::new("modelctl").args(&pull_args).exec();
        return Err(anyhow::anyhow!("failed to exec 'modelctl': {}", err));
    }

    // Intercept `syn decide`, `syn prompt`, `syn audit` and dispatch to `syntropctl <cmd> <args...>`.
    if ns == "decide" || ns == "prompt" || ns == "audit" {
        if !find_in_path("syntropctl") {
            eprintln!(
                "command '{ns}' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
            );
            std::process::exit(127);
        }
        let mut cmd_args = vec![ns.to_string()];
        cmd_args.extend_from_slice(&cli.args);
        let err = Command::new("syntropctl").args(&cmd_args).exec();
        return Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err));
    }

    // Bare words are a question for the router (`syn say hello` asks
    // the fleet). A leading dash still errors: that is a mistyped flag,
    // not something anyone would ask.
    if is_bare_prompt(ns) {
        return exec_prompt(ns, &cli.args);
    }

    let Some(bin) = resolve(ns) else {
        match suggest(ns) {
            Some(s) => eprintln!("unknown namespace '{}'. did you mean '{}'?", ns, s),
            None => eprintln!(
                "unknown namespace '{}'. namespaces: {}",
                ns,
                NAMESPACES
                    .iter()
                    .map(|(n, _, _)| *n)
                    .collect::<Vec<_>>()
                    .join(", ")
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
