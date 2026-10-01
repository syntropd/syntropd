//! Linux Cognitive Desktop Companion command parser and dispatcher.

use super::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn companion`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn companion",
    about = "Linux Cognitive Desktop Companion multimodal visual reasoning & actuation"
)]
pub struct CompanionArgs {
    #[command(subcommand)]
    pub command: Option<CompanionSubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Supported subcommands for `syn companion`.
#[derive(Subcommand, Debug, Clone)]
pub enum CompanionSubcommand {
    /// Captures screen frame, queries routerd with visual context, and renders grounded answer
    Ask {
        /// Grounded question or prompt regarding current screen content
        prompt: String,

        /// Optional display identifier (e.g. :0, wayland-0)
        #[arg(long)]
        display: Option<String>,
    },

    /// Plans UI actions from screen, executes mouse/keyboard commands via Actuator1, and validates state
    Execute {
        /// High-level natural language instruction or UI action plan
        instruction: String,

        /// Validate and preview action plan without emitting hardware actuator events
        #[arg(long)]
        dry_run: bool,
    },

    /// Daemonized session listening for voice triggers or hotkey chords
    Listen {
        /// Listen for voice commands via ambient audio and VAD
        #[arg(long)]
        voice: bool,

        /// Hotkey chord identifier (e.g. Super+Space, F12)
        #[arg(long)]
        hotkey: Option<String>,

        /// Run a single listen cycle and exit
        #[arg(long)]
        once: bool,
    },
}

/// Dispatches `syn companion` commands directly to `syntropctl companion`.
pub fn handle_syn_companion(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn companion".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = CompanionArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'companion' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["companion".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_args_parse_ask() {
        let args = CompanionArgs::try_parse_from(["syn companion", "ask", "what window is active?"]).unwrap();
        match args.command {
            Some(CompanionSubcommand::Ask { prompt, display }) => {
                assert_eq!(prompt, "what window is active?");
                assert!(display.is_none());
            }
            _ => panic!("Expected Ask subcommand"),
        }
    }

    #[test]
    fn test_companion_args_parse_execute() {
        let args = CompanionArgs::try_parse_from([
            "syn companion",
            "execute",
            "click 0.5 0.5",
            "--dry-run",
        ])
        .unwrap();
        match args.command {
            Some(CompanionSubcommand::Execute {
                instruction,
                dry_run,
            }) => {
                assert_eq!(instruction, "click 0.5 0.5");
                assert!(dry_run);
            }
            _ => panic!("Expected Execute subcommand"),
        }
    }

    #[test]
    fn test_companion_args_parse_listen() {
        let args = CompanionArgs::try_parse_from([
            "syn companion",
            "listen",
            "--voice",
            "--hotkey",
            "Super+Space",
            "--once",
        ])
        .unwrap();
        match args.command {
            Some(CompanionSubcommand::Listen {
                voice,
                hotkey,
                once,
            }) => {
                assert!(voice);
                assert_eq!(hotkey.as_deref(), Some("Super+Space"));
                assert!(once);
            }
            _ => panic!("Expected Listen subcommand"),
        }
    }
}
