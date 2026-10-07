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
    println!("syntrop {} — front door to the suite\n", env!("CARGO_PKG_VERSION"));
    for (ns, bin, desc) in NAMESPACES {
        println!("  {:<8} {} ({})", ns, desc, bin);
    }
    const EXTRA: &[(&str, &str)] = &[
        ("pull", "Pull and register a model directly (modelctl)"),
        ("setup", "Bootstrap model family and speculative router (modelctl/routerd)"),
        ("admin", "Autonomous OS self-healing & administration (syntropctl admin)"),
        ("companion", "Linux Cognitive Desktop Companion (syntropctl companion)"),
        ("telemetry", "Dynamic kernel telemetry & closed-loop PSI tuning (syntropctl)"),
        ("visual", "Generative visual image synthesis (syntropctl visual)"),
        ("audio", "Generative music & acoustic atmosphere (syntropctl audio)"),
        ("video", "Generative short-form video synthesis (syntropctl video)"),
        ("completions", "Generate shell completion scripts (syn completions)"),
        ("uninstall", "Safely remove or purge syntropd and daemons (syntrop-uninstall)"),
    ];
    for (cmd, desc) in EXTRA {
        println!("  {:<8} {}", cmd, desc);
    }
    println!("\nusage: syn <question> | syn <namespace> <cmd> [args...] | syn setup --family [qwen|granite|phi|gemma|bitnet] | syn pull <model> | syn admin <cmd> | syn companion <cmd> | syn telemetry [status|tune] | syn visual|audio|video generate | syn completions [bash|zsh|fish] | syn uninstall [--purge]");
    println!("effort: -e, --effort <tier>  (none, low, med, high, max; defaults to 0 tokens on CPU / tight memory, 1,024 on GPU with healthy VRAM)");
    println!("examples:");
    println!("  syn say hello in one sentence");
    println!("  syn -e low explain quantum computing");
    println!("  syn visual generate \"a sunset over mountains\"");
    println!("  syn audio generate \"cyberpunk synthwave beats\" --bpm 120");
    println!("  syn video generate \"ocean waves crashing\" --frames 16");
    println!("  syn setup --family qwen");
    println!("  syn pull qwen2.5:0.5b");
    println!("  syn completions bash");
    println!("  syn uninstall --dry-run");
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

    // Intercept `syn completions [shell]`
    if ns == "completions" || ns == "completion" {
        return syntropd::cli::completions::handle_syn_completions(&cli.args);
    }

    // Intercept `syn setup <args...>` and dispatch to model autoloader setup.
    if ns == "setup" {
        return syntropd::cli::setup::handle_syn_setup(&cli.args);
    }

    // Intercept `syn admin <args...>` and dispatch to autonomous self-healing admin.
    if ns == "admin" {
        return syntropd::cli::admin::handle_syn_admin(&cli.args);
    }

    // Intercept `syn companion <args...>` and dispatch to desktop companion.
    if ns == "companion" {
        return syntropd::cli::companion::handle_syn_companion(&cli.args);
    }

    // Intercept `syn telemetry <args...>` and dispatch to kernel telemetry.
    if ns == "telemetry" {
        return syntropd::cli::telemetry::handle_syn_telemetry(&cli.args);
    }

    // Intercept `syn tune <args...>` and dispatch to dynamic tuning governor.
    if ns == "tune" {
        return syntropd::cli::telemetry::handle_syn_tune(&cli.args);
    }

    // Intercept `syn visual <args...>` and dispatch to `syntropctl visual <args...>`.
    if ns == "visual" {
        return syntropd::cli::multimedia::handle_syn_visual(&cli.args);
    }

    // Intercept `syn audio <args...>` and dispatch to `syntropctl audio <args...>`.
    if ns == "audio" {
        return syntropd::cli::multimedia::handle_syn_audio(&cli.args);
    }

    // Intercept `syn video <args...>` and dispatch to `syntropctl video <args...>`.
    if ns == "video" {
        return syntropd::cli::multimedia::handle_syn_video(&cli.args);
    }

    // Intercept `syn uninstall <args...>` and dispatch to syntrop-uninstall.
    if ns == "uninstall" {
        let uninstaller = if find_in_path("syntrop-uninstall") {
            "syntrop-uninstall".to_string()
        } else if std::path::Path::new("/usr/local/bin/syntrop-uninstall").exists() {
            "/usr/local/bin/syntrop-uninstall".to_string()
        } else if std::path::Path::new("/usr/bin/syntrop-uninstall").exists() {
            "/usr/bin/syntrop-uninstall".to_string()
        } else {
            eprintln!("syntrop uninstaller not found. Run: curl -fsSL https://syntropd.github.io/uninstall.sh | sudo bash");
            std::process::exit(1);
        };
        let is_root = rustix::process::geteuid().is_root();
        let mut cmd = if is_root {
            Command::new(&uninstaller)
        } else {
            let mut c = Command::new("sudo");
            c.arg(&uninstaller);
            c
        };
        cmd.args(&cli.args);
        let err = cmd.exec();
        return Err(anyhow::anyhow!("failed to exec '{}': {}", uninstaller, err));
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

    // Intercept `syn talk <args...>` and dispatch to `syn companion talk <args...>`.
    if ns == "talk" {
        let mut talk_args = vec!["talk".to_string()];
        talk_args.extend_from_slice(&cli.args);
        return syntropd::cli::companion::handle_syn_companion(&talk_args);
    }

    // Intercept `syn prompt <args...>` and dispatch to incident decorator.
    if ns == "prompt" {
        return syntropd::cli::prompt::handle_syn_prompt(&cli.args);
    }

    // Intercept `syn decide <args...>` and dispatch to triage approval.
    if ns == "decide" {
        return syntropd::cli::decide::handle_syn_decide(&cli.args);
    }

    // Intercept `syn audit <args...>` and dispatch to `syntropctl audit <args...>`.
    if ns == "audit" {
        if !find_in_path("syntropctl") {
            eprintln!(
                "command 'audit' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
            );
            std::process::exit(127);
        }
        let mut cmd_args = vec!["audit".to_string()];
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
