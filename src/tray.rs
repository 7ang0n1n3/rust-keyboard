use std::time::Duration;

use anyhow::{Context, Result};
use ksni::TrayMethods;
use ksni::menu::{CheckmarkItem, MenuItem, StandardItem, SubMenu};
use log::warn;

use crate::config::{Config, SoundProfile};
use crate::engine::{self, EngineController, RunningEngine};

pub fn run(config: Config) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime for tray service")?;

    runtime.block_on(async move {
        run_with_retries(config).await
    })?;

    Ok(())
}

async fn run_with_retries(config: Config) -> Result<()> {
    const RETRY_DELAY: Duration = Duration::from_secs(2);

    loop {
        match start_services(&config).await {
            Ok((_engine, _tray_handle)) => {
                std::future::pending::<()>().await;
                #[allow(unreachable_code)]
                return Ok(());
            }
            Err(error) => {
                warn!("startup failed: {error:#}");
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }
}

async fn start_services(
    config: &Config,
) -> Result<(RunningEngine, ksni::Handle<KeyboardTray>)> {
    let engine = engine::start_background(config)?;
    let tray = KeyboardTray::new(config.clone(), engine.controller().clone());
    let handle = tray.spawn().await.context("failed to start ksni tray")?;
    Ok((engine, handle))
}

#[derive(Clone, Debug)]
struct KeyboardTray {
    config: Config,
    engine: EngineController,
}

impl KeyboardTray {
    fn new(config: Config, engine: EngineController) -> Self {
        Self { config, engine }
    }

    fn set_profile(&mut self, profile: SoundProfile) {
        self.config.audio.profile = profile;
        self.engine.set_profile(profile);
        let _ = self.config.save();
    }

    fn set_volume(&mut self, volume: f32) {
        self.config.audio.volume = volume;
        self.engine.set_volume(volume);
        let _ = self.config.save();
    }

    fn status_label(&self) -> String {
        format!(
            "{} {:.0}%",
            self.config.audio.profile.as_label(),
            self.config.audio.volume * 100.0
        )
    }
}

impl ksni::Tray for KeyboardTray {
    fn id(&self) -> String {
        "rust-keyboard".into()
    }

    fn title(&self) -> String {
        self.status_label()
    }

    fn icon_name(&self) -> String {
        "input-keyboard".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let current_profile = self.config.audio.profile;
        let current_volume = normalize_volume(self.config.audio.volume);

        vec![
            StandardItem {
                label: format!("Keeb sound: {}", self.status_label()),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            SubMenu {
                label: "Profile".into(),
                submenu: vec![
                    profile_item("Apple laptop", SoundProfile::Apple, current_profile),
                    profile_item("Android tap", SoundProfile::Android, current_profile),
                    profile_item("Blue switch", SoundProfile::Blue, current_profile),
                    profile_item("Brown switch", SoundProfile::Brown, current_profile),
                    profile_item("Red switch", SoundProfile::Red, current_profile),
                ],
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Volume".into(),
                submenu: vec![
                    volume_item("25%", 0.25, current_volume),
                    volume_item("45%", 0.45, current_volume),
                    volume_item("65%", 0.65, current_volume),
                    volume_item("85%", 0.85, current_volume),
                ],
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.engine.stop();
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn profile_item(
    label: &str,
    profile: SoundProfile,
    current_profile: SoundProfile,
) -> MenuItem<KeyboardTray> {
    CheckmarkItem {
        label: label.into(),
        checked: current_profile == profile,
        activate: Box::new(move |tray: &mut KeyboardTray| {
            tray.set_profile(profile);
        }),
        ..Default::default()
    }
    .into()
}

fn volume_item(label: &str, volume: f32, current_volume: f32) -> MenuItem<KeyboardTray> {
    CheckmarkItem {
        label: label.into(),
        checked: current_volume == volume,
        activate: Box::new(move |tray: &mut KeyboardTray| {
            tray.set_volume(volume);
        }),
        ..Default::default()
    }
    .into()
}

fn normalize_volume(volume: f32) -> f32 {
    const PRESETS: [f32; 4] = [0.25, 0.45, 0.65, 0.85];

    PRESETS
        .into_iter()
        .min_by(|left, right| {
            (volume - *left)
                .abs()
                .partial_cmp(&(volume - *right).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0.45)
}
