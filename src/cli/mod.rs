//! Command-line interface: argument parsing and report rendering.

pub mod admin;
pub mod companion;
pub mod dispatch;
pub mod multimedia;
pub mod parse_cli;
pub mod render_report;
pub mod setup;
pub mod telemetry;

pub use multimedia::{audio, video, visual};
pub use setup::router_config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_exports() {
        assert!(!dispatch::NAMESPACES.is_empty());
    }
}
