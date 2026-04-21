use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;

use anyhow::{Context, Result};
use evdev::{Device, InputEventKind, Key};
use log::{debug, warn};

#[derive(Clone, Debug)]
pub struct KeyboardDevice {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct KeyPress {
    pub device_name: String,
    pub key_code: Key,
}

pub struct InputManager {
    devices: Vec<KeyboardDevice>,
}

impl InputManager {
    pub fn discover(device_filters: &[String]) -> Result<Self> {
        let devices = list_keyboards(device_filters)?;
        Ok(Self { devices })
    }

    pub fn devices(&self) -> &[KeyboardDevice] {
        &self.devices
    }

    pub fn spawn(self, tx: Sender<KeyPress>) {
        for keyboard in self.devices {
            let tx = tx.clone();
            thread::spawn(move || {
                if let Err(error) = read_device_loop(keyboard, tx) {
                    warn!("{error:#}");
                }
            });
        }
    }
}

fn list_keyboards(device_filters: &[String]) -> Result<Vec<KeyboardDevice>> {
    let mut keyboards = Vec::new();

    for entry in fs::read_dir("/dev/input").context("failed to read /dev/input")? {
        let entry = entry.context("failed to read /dev/input entry")?;
        let path = entry.path();
        if !is_event_device(&path) {
            continue;
        }

        let device = match Device::open(&path) {
            Ok(device) => device,
            Err(error) => {
                debug!("skipping {}: {error}", path.display());
                continue;
            }
        };

        if !looks_like_keyboard(&device) {
            continue;
        }

        let name = device
            .name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "unknown keyboard".to_string());

        if !device_filters.is_empty()
            && !device_filters
                .iter()
                .any(|filter| name.to_lowercase().contains(&filter.to_lowercase()))
        {
            continue;
        }

        keyboards.push(KeyboardDevice { path, name });
    }

    keyboards.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(keyboards)
}

fn looks_like_keyboard(device: &Device) -> bool {
    let Some(keys) = device.supported_keys() else {
        return false;
    };

    keys.contains(Key::KEY_A)
        && keys.contains(Key::KEY_Z)
        && keys.contains(Key::KEY_SPACE)
        && keys.contains(Key::KEY_ENTER)
}

fn is_event_device(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("event"))
}

fn read_device_loop(keyboard: KeyboardDevice, tx: Sender<KeyPress>) -> Result<()> {
    let mut device = Device::open(&keyboard.path)
        .with_context(|| format!("failed to open {}", keyboard.path.display()))?;

    loop {
        for event in device
            .fetch_events()
            .with_context(|| format!("input device loop failed for {}", keyboard.path.display()))?
        {
            if let InputEventKind::Key(key_code) = event.kind() {
                if event.value() == 1 {
                    let _ = tx.send(KeyPress {
                        device_name: keyboard.name.clone(),
                        key_code,
                    });
                }
            }
        }
    }
}
