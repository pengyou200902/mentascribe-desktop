use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to get default input config: {0}")]
    ConfigError(String),
    #[error("Failed to build stream: {0}")]
    StreamError(String),
    #[error("Stream error: {0}")]
    PlayError(String),
    #[error("Capture already running")]
    AlreadyRunning,
    #[error("No capture running")]
    NotRunning,
}

pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

struct AudioThreadHandle {
    stop_sender: Sender<()>,
    thread_handle: JoinHandle<()>,
}

lazy_static::lazy_static! {
    static ref AUDIO_BUFFER: Mutex<Vec<f32>> = Mutex::new(Vec::new());
    static ref AUDIO_THREAD: Mutex<Option<AudioThreadHandle>> = Mutex::new(None);
    static ref SAMPLE_RATE: Mutex<u32> = Mutex::new(16000);
    static ref CHANNELS: Mutex<u16> = Mutex::new(1);
}

pub fn start_capture() -> Result<(), AudioError> {
    // Check if already running
    if AUDIO_THREAD.lock().unwrap().is_some() {
        return Err(AudioError::AlreadyRunning);
    }

    // Clear buffer
    AUDIO_BUFFER.lock().unwrap().clear();

    // Create channel for stop signal
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    // Spawn audio thread that owns the stream
    let thread_handle = thread::spawn(move || {
        let result = (|| -> Result<(), AudioError> {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or(AudioError::NoInputDevice)?;

            let config = device
                .default_input_config()
                .map_err(|e| AudioError::ConfigError(e.to_string()))?;

            *SAMPLE_RATE.lock().unwrap() = config.sample_rate().0;
            *CHANNELS.lock().unwrap() = config.channels();

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buf) = AUDIO_BUFFER.lock() {
                            buf.extend_from_slice(data);
                        }
                    },
                    |err| {
                        log::error!("Audio stream error: {}", err);
                    },
                    None,
                )
                .map_err(|e| AudioError::StreamError(e.to_string()))?;

            stream
                .play()
                .map_err(|e| AudioError::PlayError(e.to_string()))?;

            log::info!("Audio capture started");

            // Block until stop signal received
            let _ = stop_rx.recv();

            // Stream is dropped here when thread ends
            log::info!("Audio thread stopping");
            Ok(())
        })();

        if let Err(e) = result {
            log::error!("Audio thread error: {}", e);
        }
    });

    // Give the thread a moment to initialize
    thread::sleep(std::time::Duration::from_millis(100));

    *AUDIO_THREAD.lock().unwrap() = Some(AudioThreadHandle {
        stop_sender: stop_tx,
        thread_handle,
    });

    Ok(())
}

pub fn stop_capture() -> Result<AudioData, AudioError> {
    // Take the thread handle
    let handle = AUDIO_THREAD
        .lock()
        .unwrap()
        .take()
        .ok_or(AudioError::NotRunning)?;

    // Send stop signal
    let _ = handle.stop_sender.send(());

    // Wait for thread to finish
    let _ = handle.thread_handle.join();

    let samples = AUDIO_BUFFER.lock().unwrap().clone();
    let sample_rate = *SAMPLE_RATE.lock().unwrap();
    let channels = *CHANNELS.lock().unwrap();

    log::info!(
        "Audio capture stopped: {} samples at {}Hz",
        samples.len(),
        sample_rate
    );

    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}

/// Resample audio to 16kHz mono for Whisper
pub fn prepare_for_whisper(audio: &AudioData) -> Vec<f32> {
    let mut mono_samples = if audio.channels > 1 {
        // Convert to mono by averaging channels
        audio
            .samples
            .chunks(audio.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
            .collect()
    } else {
        audio.samples.clone()
    };

    // Resample to 16kHz if needed
    if audio.sample_rate != 16000 {
        mono_samples = resample(&mono_samples, audio.sample_rate, 16000);
    }

    mono_samples
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = (i as f64 * ratio) as usize;
        if src_idx < samples.len() {
            resampled.push(samples[src_idx]);
        }
    }

    resampled
}
