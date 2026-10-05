//! Command-line interface: argument parsing and report rendering.

pub mod admin;
pub mod companion;
pub mod completions;
pub mod decide;
pub mod dispatch;
pub mod multimedia;
pub mod parse_cli;
pub mod prompt;
pub mod reporting;
pub mod setup;

pub use multimedia::{audio, video, visual};
pub use reporting::{render_report, telemetry};
pub use setup::router_config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_exports() {
        assert!(!dispatch::NAMESPACES.is_empty());
    }
}
