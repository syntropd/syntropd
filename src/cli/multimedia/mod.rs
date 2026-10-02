//! Multimedia subcommands for audio, video, and visual synthesis.

pub mod audio;
pub mod video;
pub mod visual;

pub use audio::{handle_syn_audio, AudioArgs, AudioSubcommand};
pub use video::{handle_syn_video, VideoArgs, VideoSubcommand};
pub use visual::{handle_syn_visual, VisualArgs, VisualSubcommand};

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_multimedia_reexports() {
        let _audio = AudioArgs::try_parse_from(["syn audio", "--help"]);
        let _video = VideoArgs::try_parse_from(["syn video", "--help"]);
        let _visual = VisualArgs::try_parse_from(["syn visual", "--help"]);
    }
}
