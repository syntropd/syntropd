//! syntropd — Native AI Subsystem for systemd CLI

use clap::Parser;
use syntropd::cli::parse_cli::{Cli, Commands};
use syntropd::cli::render_report as render;

fn main() -> anyhow::Result<()> {
    dispatch(Cli::parse().command)
}

/// Dispatch a parsed command to its report handler.
fn dispatch(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Status { json } => render::handle_status(json),
        Commands::Triage { unit, json } => render::handle_triage(&unit, json),
        Commands::Explain { unit, json } => render::handle_explain(&unit, json),
        Commands::Units { r#type, json } => render::handle_units(r#type.as_deref(), json),
        Commands::Version { json } => render::handle_version(json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_version_json() {
        dispatch(Commands::Version { json: true }).unwrap();
    }

    #[test]
    fn test_dispatch_units_filtered_json() {
        dispatch(Commands::Units {
            r#type: Some("service".to_string()),
            json: true,
        })
        .unwrap();
    }
}
