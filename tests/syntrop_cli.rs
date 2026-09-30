//! Integration tests for `syntrop` CLI routing and bare prompt normalization.

use clap::Parser;
use syntropd::cli::dispatch::{
    edit_distance, is_bare_prompt, normalize_prompt_argv, prompt_argv, resolve, suggest, Cli,
    NAMESPACES,
};

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
    assert!(!is_bare_prompt("decide"));
    assert!(!is_bare_prompt("prompt"));
    assert!(!is_bare_prompt("audit"));
    assert!(!is_bare_prompt("router"));
    assert!(!is_bare_prompt("fleet"));
    assert!(!is_bare_prompt("--help"));
    assert!(!is_bare_prompt("--json"));
    assert!(!is_bare_prompt("-V"));
    assert!(is_bare_prompt("-e"));
    assert!(is_bare_prompt("--effort"));
    assert!(is_bare_prompt("--effort=low"));
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
fn normalize_prompt_argv_partitions_flags() {
    let rest = vec![
        "hello".to_string(),
        "-e".to_string(),
        "low".to_string(),
        "world".to_string(),
    ];
    let argv = normalize_prompt_argv("say", &rest);
    assert_eq!(argv, vec!["ask", "-e", "low", "say", "hello", "world"]);
}

#[test]
fn normalize_prompt_argv_inline_and_leading_flags() {
    let rest = vec!["explain".to_string(), "qubits".to_string()];
    let argv = normalize_prompt_argv("--effort=high", &rest);
    assert_eq!(argv, vec!["ask", "--effort=high", "explain", "qubits"]);

    let rest3 = vec![
        "medium".to_string(),
        "what".to_string(),
        "is".to_string(),
        "rust".to_string(),
    ];
    let argv3 = normalize_prompt_argv("-e", &rest3);
    assert_eq!(argv3, vec!["ask", "-e", "medium", "what", "is", "rust"]);

    let rest4 = vec!["say".to_string(), "hi".to_string()];
    let argv4 = normalize_prompt_argv("-ehigh", &rest4);
    assert_eq!(argv4, vec!["ask", "-ehigh", "say", "hi"]);
}

#[test]
fn normalize_prompt_argv_preserves_english_and_dangling_flags() {
    let rest = vec!["does".to_string(), "-e".to_string(), "mean".to_string()];
    let argv = normalize_prompt_argv("what", &rest);
    assert_eq!(argv, vec!["ask", "what", "does", "-e", "mean"]);

    let rest2 = vec!["me".to_string(), "a".to_string(), "joke".to_string(), "-e".to_string()];
    let argv2 = normalize_prompt_argv("tell", &rest2);
    assert_eq!(argv2, vec!["ask", "tell", "me", "a", "joke", "-e"]);

    let rest3 = vec!["is".to_string(), "-eval".to_string()];
    let argv3 = normalize_prompt_argv("what", &rest3);
    assert_eq!(argv3, vec!["ask", "what", "is", "-eval"]);
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

#[test]
fn cli_parses_pull_command_with_flags() {
    let cli =
        Cli::try_parse_from(["syn", "pull", "org/repo", "--quant", "Q4_K_M", "--force"]).unwrap();
    assert_eq!(cli.namespace.as_deref(), Some("pull"));
    assert_eq!(
        cli.args,
        vec![
            "org/repo".to_string(),
            "--quant".to_string(),
            "Q4_K_M".to_string(),
            "--force".to_string()
        ]
    );
}
