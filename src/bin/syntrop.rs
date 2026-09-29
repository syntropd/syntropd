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
    find_in_path, is_bare_prompt, prompt_argv, resolve, suggest, NAMESPACES,
};


#[derive(Parser, Debug)]
#[command(
    name = "syntrop",
    about = "Front door to the syntropd suite",
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// Namespace: router, runtime, store, hardware, context, tools, fleet, system, pull
    #[arg(allow_hyphen_values = true)]
    namespace: Option<String>,

    /// Arguments passed through untouched to the repo CLI
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Replace this process with `routerctl ask <words...>`; the reply,
/// pipes, and exit code behave exactly like invoking it directly.
fn exec_prompt(first: &str, rest: &[String]) -> anyhow::Result<()> {
    if !find_in_path("routerctl") {
        eprintln!(
            "asking needs 'routerctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }
    let err = Command::new("routerctl").args(prompt_argv(first, rest)).exec();
    Err(anyhow::anyhow!("failed to exec 'routerctl': {}", err))
}

fn print_overview() {
    println!("syntrop {} — front door to the suite", env!("CARGO_PKG_VERSION"));
    println!();
    for (ns, bin, desc) in NAMESPACES {
        println!("  {:<8} {} ({})", ns, desc, bin);
    }
    println!("  {:<8} {} ({})", "pull", "Pull and register a model directly", "modelctl");
    println!();
    println!("usage: syn <question> | syn <namespace> <command> [args...] | syn pull <model>");
    println!("examples:");
    println!("  syn say hello in one sentence");
    println!("  syn pull qwen2.5:0.5b");
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
    use syntropd::cli::dispatch::edit_distance;


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
    fn bare_words_are_prompts_namespaces_are_not() {
        assert!(is_bare_prompt("say"));
        assert!(is_bare_prompt("Say hello in one sentence."));
        assert!(is_bare_prompt("explain"));
        assert!(!is_bare_prompt("pull"));
        assert!(!is_bare_prompt("router"));
        assert!(!is_bare_prompt("fleet"));
        assert!(!is_bare_prompt("--help"));
        assert!(!is_bare_prompt("--json"));
        assert!(!is_bare_prompt("-V"));
    }

    #[test]
    fn prompt_argv_keeps_every_word() {
        assert_eq!(prompt_argv("say", &[]), vec!["ask", "say"]);
        assert_eq!(
            prompt_argv("say", &["hello".to_string(), "there".to_string()]),
            vec!["ask", "say", "hello", "there"]
        );
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("router", "router"), 0);
        assert_eq!(edit_distance("rounter", "router"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn cli_parses_pull_command() {
        let cli = Cli::try_parse_from(["syn", "pull", "qwen2.5:0.5b"]).unwrap();
        assert_eq!(cli.namespace.as_deref(), Some("pull"));
        assert_eq!(cli.args, vec!["qwen2.5:0.5b".to_string()]);
    }
}
