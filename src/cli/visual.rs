//! Generative visual synthesis command parser and dispatcher.

use super::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn visual`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn visual",
    about = "Generative visual image synthesis"
)]
pub struct VisualArgs {
    #[command(subcommand)]
    pub command: Option<VisualSubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Supported subcommands for `syn visual`.
#[derive(Subcommand, Debug, Clone)]
pub enum VisualSubcommand {
    /// Generate an image from a prompt
    Generate {
        /// Text prompt describing the image
        prompt: String,

        /// Model identifier to invoke
        #[arg(short = 'm', long = "model")]
        model: Option<String>,

        /// LoRA adapter spec in name:weight format
        #[arg(short = 'l', long = "lora")]
        lora: Option<String>,

        /// Image dimensions in WxH format (e.g. 512x512)
        #[arg(short = 's', long = "size")]
        size: Option<String>,
    },
}

/// Dispatches `syn visual` commands directly to `syntropctl visual`.
pub fn handle_syn_visual(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn visual".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = VisualArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'visual' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["visual".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_args_parse_generate() {
        let args = VisualArgs::try_parse_from([
            "syn visual",
            "generate",
            "a futuristic city",
            "--size",
            "1024x1024",
        ])
        .unwrap();
        assert!(matches!(
            args.command,
            Some(VisualSubcommand::Generate {
                ref prompt,
                ref size,
                ..
            }) if prompt == "a futuristic city" && size.as_deref() == Some("1024x1024")
        ));
    }
}
