use std::f32::consts::PI;

use std::num::NonZero;

use anyhow::{Context, Result};
use rodio::{DeviceSinkBuilder, buffer::SamplesBuffer, mixer::Mixer};

use crate::config::SoundProfile;

pub struct AudioEngine {
    _sink: rodio::MixerDeviceSink,
    mixer: Mixer,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let sink = DeviceSinkBuilder::open_default_sink()
            .context("failed to open default audio output")?;
        let mixer = sink.mixer().clone();

        Ok(Self { _sink: sink, mixer })
    }

    pub fn play_click(&self, profile: SoundProfile, volume: f32, velocity: f32) {
        let sample_rate = 48_000;
        let frames = click_waveform(profile, volume, velocity, sample_rate);
        let source = SamplesBuffer::new(
            NonZero::new(1).expect("non-zero literal"),
            NonZero::new(sample_rate).expect("non-zero literal"),
            frames,
        );
        self.mixer.add(source);
    }
}

fn click_waveform(profile: SoundProfile, volume: f32, velocity: f32, sample_rate: u32) -> Vec<f32> {
    let (base_freq, overtone, decay_ms, noise_mix, body_mix, pitch_sweep) = match profile {
        SoundProfile::Apple => (1_850.0, 3_450.0, 13.0, 0.05, 0.88, 0.18),
        SoundProfile::Android => (2_050.0, 4_150.0, 11.0, 0.08, 0.66, 0.10),
        SoundProfile::Blue => (2_400.0, 5_200.0, 28.0, 0.48, 0.52, 0.06),
        SoundProfile::Brown => (1_750.0, 3_100.0, 22.0, 0.24, 0.76, 0.08),
        SoundProfile::Red => (1_250.0, 2_200.0, 18.0, 0.10, 0.82, 0.04),
    };

    let total_samples = ((decay_ms / 1_000.0) * sample_rate as f32).round() as usize;
    let total_samples = total_samples.max(64);
    let velocity = velocity.clamp(0.2, 1.0);
    let amplitude = volume.clamp(0.0, 1.0) * velocity;

    let mut frames = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = i as f32 / total_samples as f32;
        let env = match profile {
            SoundProfile::Apple => (-13.5 * progress).exp(),
            SoundProfile::Android => (-16.0 * progress).exp(),
            _ => (-8.5 * progress).exp(),
        };
        let sweep = 1.0 - (progress * pitch_sweep);
        let tone = (2.0 * PI * (base_freq * sweep) * t).sin() * body_mix
            + (2.0 * PI * (overtone * (1.0 + pitch_sweep * 0.25)) * t).sin() * (1.0 - body_mix);
        let attack = match profile {
            SoundProfile::Apple => (1.0 - (-32.0 * progress).exp()).clamp(0.0, 1.0),
            SoundProfile::Android => (1.0 - (-48.0 * progress).exp()).clamp(0.0, 1.0),
            _ => 1.0,
        };
        let noise = hash_noise(i as u32) * noise_mix;
        let click = match profile {
            SoundProfile::Apple => {
                hash_noise(i as u32 ^ 0x00ab_cdef) * 0.04 * (1.0 - progress).powf(2.4)
            }
            SoundProfile::Android => {
                let transient = (2.0 * PI * 6_100.0 * t).sin() * 0.035;
                transient + hash_noise(i as u32 ^ 0x0001_0bad) * 0.02 * (1.0 - progress).powf(3.0)
            }
            _ => 0.0,
        };
        frames.push(((tone * (1.0 - noise_mix) + noise + click) * env * attack) * amplitude);
    }

    frames
}

fn hash_noise(seed: u32) -> f32 {
    let mixed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    let normalized = ((mixed / 65_536) % 32_768) as f32 / 16_384.0;
    normalized - 1.0
}
