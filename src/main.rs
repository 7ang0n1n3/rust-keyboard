mod app;
mod audio;
mod cli;
mod config;
mod engine;
mod input;
mod tray;

use anyhow::Result;
use clap::Parser;

use crate::app::App;
use crate::cli::{Cli, Command};
use crate::config::Config;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config = Config::load_or_default()?;

    match cli.command {
        Some(Command::Doctor) => App::new(config)?.doctor(),
        Some(Command::DumpConfig) => App::new(config)?.dump_config(),
        None => tray::run(config),
    }
}
