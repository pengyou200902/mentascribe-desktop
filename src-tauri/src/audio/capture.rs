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
    static ref CURRENT_AUDIO_LEVEL: Mutex<f32> = Mutex::new(0.0);
    /// Flag to prevent start_capture while stop_capture is in progress
    static ref IS_STOPPING: Mutex<bool> = Mutex::new(false);
}

/// Calculate RMS (root mean square) audio level from samples
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Get the current audio level (0.0 to 1.0)
pub fn get_current_level() -> f32 {
    *CURRENT_AUDIO_LEVEL.lock().unwrap()
}

/// Reset all capture state - used to recover from stuck states
pub fn reset_state() {
    eprintln!("[capture] Resetting all capture state...");
    *IS_STOPPING.lock().unwrap() = false;
    *AUDIO_THREAD.lock().unwrap() = None;
    *CURRENT_AUDIO_LEVEL.lock().unwrap() = 0.0;
    AUDIO_BUFFER.lock().unwrap().clear();
    eprintln!("[capture] State reset complete");
}

/// Check if capture is currently active
pub fn is_capturing() -> bool {
    AUDIO_THREAD.lock().unwrap().is_some()
}

pub fn start_capture() -> Result<(), AudioError> {
    eprintln!("[capture] start_capture called");

    // Check if stop is in progress (prevents race condition)
    if *IS_STOPPING.lock().unwrap() {
        eprintln!("[capture] ERROR: Stop in progress, cannot start new capture");
        return Err(AudioError::AlreadyRunning);
    }

    // Check if already running
    if AUDIO_THREAD.lock().unwrap().is_some() {
        eprintln!("[capture] ERROR: Already running");
        return Err(AudioError::AlreadyRunning);
    }

    // Clear buffer
    AUDIO_BUFFER.lock().unwrap().clear();
    eprintln!("[capture] Buffer cleared");

    // Create channel for stop signal
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    // Spawn audio thread that owns the stream
    let thread_handle = thread::spawn(move || {
        let result = (|| -> Result<(), AudioError> {
            let host = cpal::default_host();
            eprintln!("[capture] Using audio host: {:?}", host.id());

            let device = host
                .default_input_device()
                .ok_or(AudioError::NoInputDevice)?;

            let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            eprintln!("[capture] Using input device: {}", device_name);

            let config = device
                .default_input_config()
                .map_err(|e| AudioError::ConfigError(e.to_string()))?;

            *SAMPLE_RATE.lock().unwrap() = config.sample_rate().0;
            *CHANNELS.lock().unwrap() = config.channels();

            eprintln!(
                "[capture] Audio config: {} Hz, {} channels",
                config.sample_rate().0,
                config.channels()
            );

            use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
            static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
            static TOTAL_SAMPLES: AtomicUsize = AtomicUsize::new(0);
            CALLBACK_COUNT.store(0, AtomicOrdering::SeqCst);
            TOTAL_SAMPLES.store(0, AtomicOrdering::SeqCst);

            let stream = device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let count = CALLBACK_COUNT.fetch_add(1, AtomicOrdering::SeqCst);
                        TOTAL_SAMPLES.fetch_add(data.len(), AtomicOrdering::SeqCst);

                        // Log first few callbacks to confirm stream is working
                        if count < 3 {
                            eprintln!(
                                "[capture] Audio callback #{}: received {} samples",
                                count + 1,
                                data.len()
                            );
                        }

                        // Calculate audio level from this chunk
                        let rms = calculate_rms(data);
                        // Normalize to 0-1 range (typical speech RMS is around 0.01-0.1)
                        // Use higher multiplier for better sensitivity
                        let normalized = (rms * 15.0).min(1.0);

                        if let Ok(mut level) = CURRENT_AUDIO_LEVEL.lock() {
                            let old_level = *level;
                            // Less smoothing for more responsive visualization
                            *level = old_level * 0.15 + normalized * 0.85;
                        }

                        if let Ok(mut buf) = AUDIO_BUFFER.lock() {
                            buf.extend_from_slice(data);
                        }
                    },
                    |err| {
                        eprintln!("[capture] ERROR: Audio stream error: {}", err);
                    },
                    None,
                )
                .map_err(|e| AudioError::StreamError(e.to_string()))?;

            eprintln!("[capture] Stream built, starting playback...");

            stream
                .play()
                .map_err(|e| AudioError::PlayError(e.to_string()))?;

            eprintln!("[capture] Audio stream started, waiting for stop signal...");

            // Block until stop signal received
            let _ = stop_rx.recv();

            let total = TOTAL_SAMPLES.load(AtomicOrdering::SeqCst);
            let callbacks = CALLBACK_COUNT.load(AtomicOrdering::SeqCst);
            eprintln!(
                "[capture] Stopping: received {} callbacks, {} total samples",
                callbacks, total
            );

            // Stream is dropped here when thread ends
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
    eprintln!("[capture] stop_capture called");

    // Set stopping flag to prevent new captures from starting
    *IS_STOPPING.lock().unwrap() = true;
    eprintln!("[capture] IS_STOPPING flag set to true");

    // Take the thread handle
    let handle = AUDIO_THREAD
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| {
            // Clear stopping flag on error
            *IS_STOPPING.lock().unwrap() = false;
            AudioError::NotRunning
        })?;

    // Send stop signal
    eprintln!("[capture] Sending stop signal...");
    let _ = handle.stop_sender.send(());

    // Wait for thread to finish
    eprintln!("[capture] Waiting for audio thread to finish...");
    let _ = handle.thread_handle.join();
    eprintln!("[capture] Audio thread finished");

    // Reset audio level
    *CURRENT_AUDIO_LEVEL.lock().unwrap() = 0.0;

    let samples = AUDIO_BUFFER.lock().unwrap().clone();
    let sample_rate = *SAMPLE_RATE.lock().unwrap();
    let channels = *CHANNELS.lock().unwrap();

    eprintln!(
        "[capture] Audio buffer: {} samples at {}Hz, {} channels ({:.2}s of audio)",
        samples.len(),
        sample_rate,
        channels,
        if sample_rate > 0 {
            samples.len() as f32 / sample_rate as f32
        } else {
            0.0
        }
    );

    // Check if we got any audio
    if samples.is_empty() {
        eprintln!("[capture] WARNING: No audio samples captured! Check microphone permissions.");
    } else {
        // Calculate some stats
        let max_amplitude = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let rms = calculate_rms(&samples);
        eprintln!(
            "[capture] Audio stats: max_amplitude={:.6}, rms={:.6}",
            max_amplitude, rms
        );
    }

    // Clear stopping flag now that we're done
    *IS_STOPPING.lock().unwrap() = false;
    eprintln!("[capture] IS_STOPPING flag cleared");

    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}

/// Resample audio to 16kHz mono for Whisper, with silence trimming
pub fn prepare_for_whisper(audio: &AudioData) -> Vec<f32> {
    use super::vad::trim_silence;

    eprintln!(
        "[audio] prepare_for_whisper: input {} samples at {}Hz, {} channels",
        audio.samples.len(),
        audio.sample_rate,
        audio.channels
    );

    if audio.samples.is_empty() {
        eprintln!("[audio] WARNING: Input audio buffer is empty!");
        return Vec::new();
    }

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

    eprintln!("[audio] After mono conversion: {} samples", mono_samples.len());

    // Resample to 16kHz if needed
    if audio.sample_rate != 16000 {
        mono_samples = resample(&mono_samples, audio.sample_rate, 16000);
        eprintln!("[audio] After resampling to 16kHz: {} samples", mono_samples.len());
    }

    // Calculate max amplitude to check if there's actual audio
    let max_amplitude = mono_samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    eprintln!("[audio] Max amplitude in audio: {:.6}", max_amplitude);

    // Only trim silence if we have meaningful audio
    // Use a very low threshold (0.005) and only trim if result keeps at least 50% of audio
    let trimmed = trim_silence(&mono_samples, 0.005, 1600);

    // If trimming would remove more than 80% of audio, skip trimming
    let final_samples = if trimmed.len() < mono_samples.len() / 5 {
        eprintln!(
            "[audio] Silence trimming too aggressive ({} -> {} samples), skipping trim",
            mono_samples.len(),
            trimmed.len()
        );
        mono_samples
    } else {
        eprintln!(
            "[audio] Trimmed silence: {} -> {} samples (removed {} samples)",
            mono_samples.len(),
            trimmed.len(),
            mono_samples.len() - trimmed.len()
        );
        trimmed.to_vec()
    };

    eprintln!(
        "[audio] Final audio for Whisper: {} samples ({:.2}s at 16kHz)",
        final_samples.len(),
        final_samples.len() as f32 / 16000.0
    );

    final_samples
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
