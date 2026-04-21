use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "rust-keyboard",
    version,
    about = "Wayland-first mechanical keyboard sound daemon for Arch Linux"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Inspect config and available keyboard devices.
    Doctor,
    /// Print the effective config.
    DumpConfig,
}
