//! Generative video synthesis command parser and dispatcher.

use crate::cli::dispatch::find_in_path;
use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command line arguments for `syn video`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "syn video",
    about = "Generative short-form video synthesis"
)]
pub struct VideoArgs {
    #[command(subcommand)]
    pub command: Option<VideoSubcommand>,

    /// Format output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

/// Supported subcommands for `syn video`.
#[derive(Subcommand, Debug, Clone)]
pub enum VideoSubcommand {
    /// Generate a short animated video clip from a prompt
    Generate {
        /// Text prompt describing the animation
        prompt: String,

        /// Total count of frames to generate
        #[arg(short = 'f', long = "frames")]
        frames: Option<usize>,

        /// Frame playback rate in frames per second
        #[arg(long = "fps")]
        fps: Option<u32>,

        /// Generate keyframe storyboard strip instead of full temporal video
        #[arg(long = "storyboard")]
        storyboard: Option<usize>,

        /// Allow graceful degradation to CPU storyboard keyframes on zero-VRAM hardware
        #[arg(long = "allow-degrade")]
        allow_degrade: bool,
    },
}

/// Dispatches `syn video` commands directly to `syntropctl video`.
pub fn handle_syn_video(args: &[String]) -> anyhow::Result<()> {
    let mut parse_args = vec!["syn video".to_string()];
    parse_args.extend_from_slice(args);

    if let Err(e) = VideoArgs::try_parse_from(&parse_args) {
        if e.kind() == clap::error::ErrorKind::DisplayHelp
            || e.kind() == clap::error::ErrorKind::DisplayVersion
        {
            print!("{e}");
            return Ok(());
        }
    }

    if !find_in_path("syntropctl") {
        eprintln!(
            "command 'video' needs 'syntropctl', which is not installed. reinstall: curl -fsSL https://syntropd.github.io/install.sh | sudo bash"
        );
        std::process::exit(127);
    }

    let mut exec_args = vec!["video".to_string()];
    exec_args.extend_from_slice(args);
    let err = Command::new("syntropctl").args(&exec_args).exec();
    Err(anyhow::anyhow!("failed to exec 'syntropctl': {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_args_parse_generate() {
        let args = VideoArgs::try_parse_from([
            "syn video",
            "generate",
            "ocean waves crashing",
            "--frames",
            "16",
            "--fps",
            "8",
        ])
        .unwrap();
        assert!(matches!(
            args.command,
            Some(VideoSubcommand::Generate {
                ref prompt,
                frames: Some(16),
                fps: Some(8),
                ..
            }) if prompt == "ocean waves crashing"
        ));
    }
}
