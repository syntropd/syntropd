//! Command-line interface: argument parsing and report rendering.

pub mod admin;
pub mod companion;
pub mod dispatch;
pub mod parse_cli;
pub mod render_report;
pub mod setup;
pub mod telemetry;

pub use setup::router_config;

