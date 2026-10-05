//! Hugging Face API credential discovery and interactive onboarding.
//!
//! Resolves tokens from CLI flags, environment variables (`HF_TOKEN`),
//! local cache (`~/.cache/huggingface/token`), or prompts the operator
//! interactively when running in a terminal.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

/// Resolves Hugging Face token from environment variables or cache.
pub fn existing_token() -> Option<String> {
    if let Ok(tok) = std::env::var("HF_TOKEN") {
        let t = tok.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    if let Ok(tok) = std::env::var("HUGGING_FACE_HUB_TOKEN") {
        let t = tok.trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    cache_token_path().and_then(|p| fs::read_to_string(p).ok()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Computes canonical path to `~/.cache/huggingface/token`.
pub fn cache_token_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    Some(home.join(".cache/huggingface/token"))
}

/// Saves token to `~/.cache/huggingface/token` with secure `0600` permissions.
pub fn save_token_to_cache(token: &str) -> anyhow::Result<()> {
    if let Some(path) = cache_token_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, token.trim())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
    }
    Ok(())
}

/// Ensures a Hugging Face token is resolved or prompted interactively.
pub fn ensure_hf_token(cli_token: Option<&str>, dry_run: bool) -> anyhow::Result<Option<String>> {
    if let Some(tok) = cli_token {
        let t = tok.trim();
        if !t.is_empty() {
            if !dry_run {
                save_token_to_cache(t)?;
            }
            std::env::set_var("HF_TOKEN", t);
            println!("  [✓] Hugging Face token configured via CLI.");
            return Ok(Some(t.to_string()));
        }
    }

    if let Some(tok) = existing_token() {
        std::env::set_var("HF_TOKEN", &tok);
        println!("  [✓] Detected Hugging Face credentials (authenticated CDN enabled).");
        return Ok(Some(tok));
    }

    if dry_run || !io::stdin().is_terminal() {
        return Ok(None);
    }

    println!("\n[Hugging Face Authentication]");
    println!("Providing a Hugging Face token enables unthrottled downloads and access to gated models (Gemma, Llama).");
    print!("Enter Hugging Face Token [press Enter to skip]: ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
        println!("  [i] Continuing with unauthenticated access (anonymous rate limits apply).");
        Ok(None)
    } else {
        save_token_to_cache(trimmed)?;
        std::env::set_var("HF_TOKEN", trimmed);
        println!("  [✓] Token saved to ~/.cache/huggingface/token");
        Ok(Some(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_existing_token_resolution() {
        let _ = existing_token();
    }

    #[test]
    fn test_cache_token_path() {
        assert!(cache_token_path().is_some());
    }
}
