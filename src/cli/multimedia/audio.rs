//! Generative music and audio synthesis command parser and dispatcher.

use crate::cli::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn audio`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn audio",
    about = "Generative music and acoustic atmosphere synthesis"
)]
pub struct AudioArgs {
    #[command(subcommand)]
    pub command: Option<AudioSubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Supported subcommands for `syn audio`.
#[derive(Subcommand, Debug, Clone)]
pub enum AudioSubcommand {
    /// Generate an audio track from a descriptive prompt
    Generate {
        /// Text prompt describing the music
        prompt: String,

        /// Duration of the audio clip in seconds
        #[arg(short = 'd', long = "duration")]
        duration: Option<u32>,

        /// Beats per minute tempo
        #[arg(short = 'b', long = "bpm")]
        bpm: Option<u32>,
    },
}

/// Dispatches `syn audio` commands directly to `syntropctl audio`.
pub fn handle_syn_audio(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn audio".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = AudioArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'audio' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["audio".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_args_parse_generate() {
        let args = AudioArgs::try_parse_from([
            "syn audio",
            "generate",
            "cyberpunk synthwave",
            "--duration",
            "10",
            "--bpm",
            "120",
        ])
        .unwrap();
        assert!(matches!(
            args.command,
            Some(AudioSubcommand::Generate {
                ref prompt,
                duration: Some(10),
                bpm: Some(120),
            }) if prompt == "cyberpunk synthwave"
        ));
    }
}
