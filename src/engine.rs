use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Result, bail};
use log::warn;

use crate::audio::AudioEngine;
use crate::config::{Config, SoundProfile};
use crate::input::InputManager;

#[derive(Clone, Copy, Debug)]
pub struct RuntimeSettings {
    pub profile: SoundProfile,
    pub volume: f32,
}

#[derive(Clone, Debug)]
pub struct EngineController {
    settings: Arc<Mutex<RuntimeSettings>>,
    should_stop: Arc<AtomicBool>,
}

pub struct RunningEngine {
    controller: EngineController,
    join_handle: Option<JoinHandle<()>>,
}

impl RunningEngine {
    pub fn controller(&self) -> &EngineController {
        &self.controller
    }
}

impl Drop for RunningEngine {
    fn drop(&mut self) {
        self.controller.stop();
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl EngineController {
    pub fn set_profile(&self, profile: SoundProfile) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.profile = profile;
        }
    }

    pub fn set_volume(&self, volume: f32) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.volume = volume;
        }
    }

    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

pub fn start_background(config: &Config) -> Result<RunningEngine> {
    let runtime = RuntimeSettings {
        profile: config.audio.profile,
        volume: config.audio.volume,
    };
    start(config, runtime, false)
}

fn start(
    config: &Config,
    initial_settings: RuntimeSettings,
    dry_run: bool,
) -> Result<RunningEngine> {
    let manager = InputManager::discover(&config.runtime.device_filters)?;
    if manager.devices().is_empty() {
        bail!("no readable keyboard devices found; run `rust-keyboard doctor` for guidance");
    }

    let (tx, rx) = mpsc::channel();
    manager.spawn(tx);

    let should_stop = Arc::new(AtomicBool::new(false));
    let settings = Arc::new(Mutex::new(initial_settings));
    let audio = if dry_run {
        None
    } else {
        Some(AudioEngine::new()?)
    };

    let thread_stop = should_stop.clone();
    let thread_settings = settings.clone();
    let join_handle = thread::spawn(move || {
        loop {
            if thread_stop.load(Ordering::SeqCst) {
                break;
            }

            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    let runtime = *thread_settings
                        .lock()
                        .expect("runtime settings mutex poisoned");
                    if let Some(audio) = &audio {
                        audio.play_click(
                            runtime.profile,
                            runtime.volume,
                            key_velocity_hint(event.key_code),
                        );
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    warn!("input threads disconnected");
                    break;
                }
            }
        }
    });

    Ok(RunningEngine {
        controller: EngineController {
            settings,
            should_stop,
        },
        join_handle: Some(join_handle),
    })
}
fn key_velocity_hint(code: evdev::Key) -> f32 {
    if code == evdev::Key::KEY_SPACE || code == evdev::Key::KEY_ENTER {
        1.0
    } else if code == evdev::Key::KEY_LEFTSHIFT || code == evdev::Key::KEY_RIGHTSHIFT {
        0.9
    } else {
        0.72
    }
}
