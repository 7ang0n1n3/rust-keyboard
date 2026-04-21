use anyhow::{Context, Result};

use crate::config::{Config, config_path};
use crate::input::InputManager;

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn doctor(&self) -> Result<()> {
        println!("config: {}", config_path()?.display());
        println!("backend: evdev (Wayland-friendly, compositor-independent)");
        println!();

        let manager = InputManager::discover(&self.config.runtime.device_filters)?;
        if manager.devices().is_empty() {
            println!("no keyboard devices found");
            println!(
                "hint: on Arch Linux, add your user to the `input` group or configure udev access"
            );
            return Ok(());
        }

        println!("detected keyboards:");
        for keyboard in manager.devices() {
            println!("  - {} ({})", keyboard.name, keyboard.path.display());
        }

        Ok(())
    }

    pub fn dump_config(&self) -> Result<()> {
        let rendered = toml::to_string_pretty(&self.config).context("failed to render config")?;
        println!("{rendered}");
        Ok(())
    }
}
