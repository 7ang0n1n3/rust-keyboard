use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub runtime: RuntimeConfig,
    pub audio: AudioConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub backend: InputBackend,
    pub device_filters: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AudioConfig {
    pub profile: SoundProfile,
    pub volume: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputBackend {
    Evdev,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundProfile {
    Apple,
    Android,
    Blue,
    BlueAlps,
    Brown,
    Red,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig {
                backend: InputBackend::Evdev,
                device_filters: Vec::new(),
            },
            audio: AudioConfig {
                profile: SoundProfile::Brown,
                volume: 0.45,
            },
        }
    }
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let path = config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.write_default(&path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("failed to parse config from {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        self.write_default(&path)
    }

    fn write_default(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }

        let rendered =
            toml::to_string_pretty(self).context("failed to serialize default config")?;
        fs::write(path, rendered)
            .with_context(|| format!("failed to write default config to {}", path.display()))
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("could not determine config directory")?;
    Ok(base.join("rust-keyboard").join("config.toml"))
}

impl SoundProfile {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Apple => "Apple",
            Self::Android => "Android",
            Self::Blue => "Blue",
            Self::BlueAlps => "Blue Alps",
            Self::Brown => "Brown",
            Self::Red => "Red",
        }
    }
}
