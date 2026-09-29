//! Helper functions for CLI command, namespace, and question dispatch.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// (namespace, repo binary, one-line description)
pub const NAMESPACES: &[(&str, &str, &str)] = &[
    ("router", "routerctl", "Talk to LLMs: ask, setup, models, default, test"),
    ("runtime", "runtimectl", "Run local models directly"),
    ("store", "modelctl", "Model file storage: list, import, prune"),
    ("hardware", "inferenctl", "GPU/accelerator planes and leases"),
    ("context", "contextctl", "System history and config drift"),
    ("tools", "toolctl", "Sandboxed repair tools and rollback"),
    ("fleet", "syntropctl", "Fleet health and failure forensics"),
    ("system", "syntropd", "Units, triage, and suite version"),
];

/// Resolves a namespace string to its owning binary name.
pub fn resolve(namespace: &str) -> Option<&'static str> {
    NAMESPACES
        .iter()
        .find(|(ns, _, _)| *ns == namespace)
        .map(|(_, bin, _)| *bin)
}

/// Closest namespace within a small edit distance, for "did you mean?".
pub fn suggest(unknown: &str) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
    for (ns, _, _) in NAMESPACES {
        let d = edit_distance(unknown, ns);
        if d <= 2 && best.map(|(_, b)| d < b).unwrap_or(true) {
            best = Some((ns, d));
        }
    }
    best.map(|(ns, _)| ns)
}

/// Computes Levenshtein edit distance between two strings.
pub fn edit_distance(a: &str, b: &str) -> usize {
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

/// True when the first word is a question, not a namespace dispatch or intercepted command.
pub fn is_bare_prompt(first: &str) -> bool {
    !first.starts_with('-') && first != "pull" && resolve(first).is_none()
}

/// `routerctl` argv for a bare question: `ask` plus every word untouched.
pub fn prompt_argv(first: &str, rest: &[String]) -> Vec<String> {
    let mut argv = Vec::with_capacity(rest.len() + 2);
    argv.push("ask".to_string());
    argv.push(first.to_string());
    argv.extend(rest.iter().cloned());
    argv
}

/// Checks if a binary name exists and is executable in `$PATH` or as a path.
pub fn find_in_path(bin: &str) -> bool {
    if bin.contains('/') {
        return is_executable(Path::new(bin));
    }
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(bin)))
        })
        .unwrap_or(false)
}

/// Checks whether a given path is an executable regular file.
pub fn is_executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
