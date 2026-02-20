use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{FastFixedIn, PolynomialDegree, Resampler};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
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
    /// Pre-processed 16kHz mono samples ready for Whisper, produced incrementally
    /// during recording by the CPAL callback. `None` if real-time resampling failed
    /// or was unavailable (fallback to post-stop processing in `prepare_for_whisper`).
    pub whisper_samples: Option<Vec<f32>>,
}

struct AudioThreadHandle {
    stop_sender: Sender<()>,
    thread_handle: JoinHandle<()>,
}

/// Holds the rubato resampler and a mono sample accumulator buffer.
/// Created once per recording session; shared between the audio thread and callback
/// via `Arc<Mutex<>>`. The callback uses `try_lock()` to avoid blocking.
struct ResamplerState {
    resampler: FastFixedIn<f32>,
    /// Mono samples waiting to fill a complete resampler chunk (1024 samples).
    mono_accumulator: Vec<f32>,
    /// The resampler's fixed input chunk size.
    chunk_size: usize,
    /// Whether real-time resampling has been marked as failed (skip further attempts).
    failed: bool,
}

lazy_static::lazy_static! {
    static ref AUDIO_BUFFER: Mutex<Vec<f32>> = Mutex::new(Vec::new());
    /// Pre-processed 16kHz mono buffer, populated incrementally by the CPAL callback.
    static ref WHISPER_BUFFER: Mutex<Vec<f32>> = Mutex::new(Vec::new());
    static ref AUDIO_THREAD: Mutex<Option<AudioThreadHandle>> = Mutex::new(None);
    static ref SAMPLE_RATE: Mutex<u32> = Mutex::new(16000);
    static ref CHANNELS: Mutex<u16> = Mutex::new(1);
    static ref CURRENT_AUDIO_LEVEL: Mutex<f32> = Mutex::new(0.0);
    /// Flag to prevent start_capture while stop_capture is in progress
    static ref IS_STOPPING: Mutex<bool> = Mutex::new(false);
    /// Shared resampler state for the current recording session.
    /// `None` when not recording or if resampler creation failed.
    static ref RESAMPLER_STATE: Mutex<Option<Arc<Mutex<ResamplerState>>>> = Mutex::new(None);
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
    WHISPER_BUFFER.lock().unwrap().clear();
    *RESAMPLER_STATE.lock().unwrap() = None;
    eprintln!("[capture] State reset complete");
}

/// Check if capture is currently active
pub fn is_capturing() -> bool {
    AUDIO_THREAD.lock().unwrap().is_some()
}

/// Convert a multi-channel interleaved chunk to mono by averaging channels.
/// Returns the input unchanged if already mono.
fn to_mono(data: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    let ch = channels as usize;
    data.chunks(ch)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Process mono samples through the resampler, draining full chunks from the
/// accumulator. Appends resampled output to `whisper_buf`. Returns `true` on
/// success, `false` if the resampler encountered an error (caller should mark
/// the state as failed).
fn drain_resampler(state: &mut ResamplerState, whisper_buf: &mut Vec<f32>) -> bool {
    while state.mono_accumulator.len() >= state.chunk_size {
        let chunk: Vec<f32> = state.mono_accumulator.drain(..state.chunk_size).collect();
        match state.resampler.process(&[&chunk], None) {
            Ok(result) => {
                if let Some(channel) = result.first() {
                    whisper_buf.extend_from_slice(channel);
                }
            }
            Err(e) => {
                eprintln!(
                    "[capture] Resampler process error in callback: {}, disabling real-time resampling",
                    e
                );
                return false;
            }
        }
    }
    true
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

    // Clear buffers and pre-allocate
    {
        // Raw buffer: up to 30s at 48kHz stereo
        let mut buf = AUDIO_BUFFER.lock().unwrap();
        buf.clear();
        buf.reserve(48000 * 2 * 30);
    }
    {
        // Whisper buffer: up to 30s at 16kHz mono
        let mut wbuf = WHISPER_BUFFER.lock().unwrap();
        wbuf.clear();
        wbuf.reserve(16000 * 30);
    }
    // Clear any previous resampler state (will be created after we know the device config)
    *RESAMPLER_STATE.lock().unwrap() = None;

    eprintln!("[capture] Buffers cleared and pre-allocated");

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

            let sr = config.sample_rate().0;
            let ch = config.channels();
            *SAMPLE_RATE.lock().unwrap() = sr;
            *CHANNELS.lock().unwrap() = ch;

            eprintln!(
                "[capture] Audio config: {} Hz, {} channels",
                sr, ch
            );

            // Create resampler if sample rate differs from 16kHz.
            // If already 16kHz, we only need mono conversion (no resampler needed).
            let resampler_arc: Option<Arc<Mutex<ResamplerState>>> = if sr != 16000 {
                let ratio = 16000_f64 / sr as f64;
                let chunk_size = 1024_usize;
                match FastFixedIn::<f32>::new(ratio, 2.0, PolynomialDegree::Cubic, chunk_size, 1) {
                    Ok(r) => {
                        eprintln!(
                            "[capture] Real-time resampler created: {}Hz -> 16kHz (ratio={:.4}, chunk={})",
                            sr, ratio, chunk_size
                        );
                        let state = ResamplerState {
                            resampler: r,
                            mono_accumulator: Vec::with_capacity(chunk_size * 2),
                            chunk_size,
                            failed: false,
                        };
                        let arc = Arc::new(Mutex::new(state));
                        // Store in global so stop_capture can flush
                        *RESAMPLER_STATE.lock().unwrap() = Some(Arc::clone(&arc));
                        Some(arc)
                    }
                    Err(e) => {
                        eprintln!(
                            "[capture] WARNING: Failed to create real-time resampler: {}. \
                             Will fall back to post-stop resampling.",
                            e
                        );
                        None
                    }
                }
            } else {
                // Already 16kHz -- just need mono conversion, no resampler
                // We still create a "passthrough" ResamplerState with no resampler,
                // but it's simpler to handle this case inline in the callback.
                eprintln!("[capture] Input is already 16kHz, only mono conversion needed in callback");
                None
            };

            use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
            static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
            static TOTAL_SAMPLES: AtomicUsize = AtomicUsize::new(0);
            CALLBACK_COUNT.store(0, AtomicOrdering::SeqCst);
            TOTAL_SAMPLES.store(0, AtomicOrdering::SeqCst);

            // Capture values for the callback closure
            let cb_channels = ch;
            let cb_sample_rate = sr;

            // Request smaller buffer for lower tail latency (256 frames instead of
            // default 512). CPAL will use the nearest supported size if 256 isn't exact.
            let mut stream_config: cpal::StreamConfig = config.into();
            stream_config.buffer_size = cpal::BufferSize::Fixed(256);

            let stream = device
                .build_input_stream(
                    &stream_config,
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

                        if let Ok(mut level) = CURRENT_AUDIO_LEVEL.try_lock() {
                            let old_level = *level;
                            // Less smoothing for more responsive visualization
                            *level = old_level * 0.15 + normalized * 0.85;
                        }

                        // Append raw samples to AUDIO_BUFFER (for audio level display etc.)
                        if let Ok(mut buf) = AUDIO_BUFFER.try_lock() {
                            buf.extend_from_slice(data);
                        }

                        // --- Real-time mono conversion + resampling for Whisper ---
                        if let Some(ref rs_arc) = resampler_arc {
                            // try_lock: if the mutex is contended (e.g., stop_capture flushing),
                            // skip this chunk rather than blocking the audio thread.
                            if let Ok(mut rs) = rs_arc.try_lock() {
                                if !rs.failed {
                                    // Convert to mono
                                    let mono = to_mono(data, cb_channels);
                                    // Append to accumulator
                                    rs.mono_accumulator.extend_from_slice(&mono);
                                    // Drain full chunks through resampler
                                    if let Ok(mut wbuf) = WHISPER_BUFFER.try_lock() {
                                        if !drain_resampler(&mut rs, &mut wbuf) {
                                            rs.failed = true;
                                        }
                                    }
                                    // If WHISPER_BUFFER lock failed, samples stay in accumulator
                                    // and will be processed on the next callback.
                                }
                            }
                        } else if cb_sample_rate == 16000 {
                            // Already 16kHz: just convert to mono and append directly
                            if let Ok(mut wbuf) = WHISPER_BUFFER.try_lock() {
                                let mono = to_mono(data, cb_channels);
                                wbuf.extend_from_slice(&mono);
                            }
                        }
                        // If resampler_arc is None and sample_rate != 16kHz, real-time
                        // resampling is unavailable; prepare_for_whisper will handle it.
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

    // Wait for thread to finish (stream is dropped, no more callbacks)
    eprintln!("[capture] Waiting for audio thread to finish...");
    let _ = handle.thread_handle.join();
    eprintln!("[capture] Audio thread finished");

    // Flush remaining samples in the resampler accumulator.
    // The audio thread has ended so there are no more callbacks contending the lock.
    let whisper_samples = {
        let rs_opt = RESAMPLER_STATE.lock().unwrap().take();
        match rs_opt {
            Some(rs_arc) => {
                let mut rs = rs_arc.lock().unwrap();
                if rs.failed {
                    eprintln!("[capture] Resampler was marked failed, no pre-processed whisper samples");
                    None
                } else {
                    let mut wbuf = std::mem::take(&mut *WHISPER_BUFFER.lock().unwrap());
                    // Flush any remaining samples in the accumulator via process_partial
                    if !rs.mono_accumulator.is_empty() {
                        let remainder: Vec<f32> = rs.mono_accumulator.drain(..).collect();
                        eprintln!(
                            "[capture] Flushing {} remaining mono samples through resampler",
                            remainder.len()
                        );
                        match rs.resampler.process_partial(Some(&[&remainder]), None) {
                            Ok(result) => {
                                if let Some(channel) = result.first() {
                                    wbuf.extend_from_slice(channel);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "[capture] Resampler flush error: {}, discarding pre-processed buffer",
                                    e
                                );
                                // Fall back to post-stop processing
                                wbuf.clear();
                            }
                        }
                    }
                    if wbuf.is_empty() {
                        None
                    } else {
                        eprintln!(
                            "[capture] Pre-processed whisper buffer: {} samples ({:.2}s at 16kHz)",
                            wbuf.len(),
                            wbuf.len() as f32 / 16000.0
                        );
                        Some(wbuf)
                    }
                }
            }
            None => {
                // No resampler was created. Check if we have direct 16kHz mono samples.
                let wbuf = std::mem::take(&mut *WHISPER_BUFFER.lock().unwrap());
                if wbuf.is_empty() {
                    None
                } else {
                    eprintln!(
                        "[capture] Pre-processed whisper buffer (passthrough): {} samples ({:.2}s at 16kHz)",
                        wbuf.len(),
                        wbuf.len() as f32 / 16000.0
                    );
                    Some(wbuf)
                }
            }
        }
    };

    // Reset audio level
    *CURRENT_AUDIO_LEVEL.lock().unwrap() = 0.0;

    let samples = std::mem::take(&mut *AUDIO_BUFFER.lock().unwrap());
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
        whisper_samples,
    })
}

/// Read a snapshot of WHISPER_BUFFER from position `from` onwards.
/// Returns (new_samples, current_buffer_length).
/// Used by the VAD streaming monitor to read new audio without blocking the CPAL callback.
pub fn snapshot_whisper_buffer(from: usize) -> (Vec<f32>, usize) {
    if let Ok(wbuf) = WHISPER_BUFFER.lock() {
        let len = wbuf.len();
        if len > from {
            (wbuf[from..].to_vec(), len)
        } else {
            (Vec::new(), len)
        }
    } else {
        (Vec::new(), from)
    }
}

/// Resample audio to 16kHz mono for Whisper.
/// If pre-processed whisper samples are available (from real-time resampling during
/// recording), returns them directly — eliminating post-stop latency entirely.
/// Otherwise falls back to the original mono conversion + resampling pipeline.
///
/// Takes ownership of AudioData to avoid cloning samples when already mono.
/// Note: silence trimming removed — Silero VAD pre-filtering in whisper.rs
/// handles speech/silence segmentation with much higher accuracy.
pub fn prepare_for_whisper(audio: AudioData) -> Vec<f32> {
    eprintln!(
        "[audio] prepare_for_whisper: input {} samples at {}Hz, {} channels, whisper_samples={}",
        audio.samples.len(),
        audio.sample_rate,
        audio.channels,
        if audio.whisper_samples.is_some() { "yes" } else { "no" }
    );

    // Fast path: use pre-processed 16kHz mono samples from real-time resampling
    if let Some(whisper_samples) = audio.whisper_samples {
        if !whisper_samples.is_empty() {
            eprintln!(
                "[audio] Using pre-processed whisper samples: {} samples ({:.2}s at 16kHz) -- zero post-stop latency",
                whisper_samples.len(),
                whisper_samples.len() as f32 / 16000.0
            );
            return whisper_samples;
        }
        eprintln!("[audio] Pre-processed whisper samples were empty, falling back to post-stop processing");
    }

    // Fallback path: original mono conversion + resampling
    eprintln!("[audio] Falling back to post-stop mono conversion + resampling");

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
        // No clone needed: we own the AudioData and can move the samples directly
        audio.samples
    };

    eprintln!("[audio] After mono conversion: {} samples", mono_samples.len());

    // Resample to 16kHz if needed
    if audio.sample_rate != 16000 {
        mono_samples = resample(&mono_samples, audio.sample_rate, 16000);
        eprintln!("[audio] After resampling to 16kHz: {} samples", mono_samples.len());
    }

    eprintln!(
        "[audio] Final audio for Whisper: {} samples ({:.2}s at 16kHz)",
        mono_samples.len(),
        mono_samples.len() as f32 / 16000.0
    );

    mono_samples
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;

    // Use FastFixedIn with cubic interpolation — much faster than sinc for speech-to-text.
    // Cubic is more than sufficient quality for ASR (we don't need music-production fidelity).
    let chunk_size = 1024;
    let mut resampler = match FastFixedIn::<f32>::new(ratio, 2.0, PolynomialDegree::Cubic, chunk_size, 1) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[audio] rubato resampler creation failed: {}, falling back to linear", e);
            return resample_linear(samples, from_rate, to_rate);
        }
    };

    let mut output: Vec<f32> = Vec::with_capacity((samples.len() as f64 * ratio) as usize + chunk_size);

    // Process full chunks
    let mut pos = 0;
    while pos + chunk_size <= samples.len() {
        let chunk = &samples[pos..pos + chunk_size];
        match resampler.process(&[chunk], None) {
            Ok(result) => {
                if let Some(channel) = result.first() {
                    output.extend_from_slice(channel);
                }
            }
            Err(e) => {
                eprintln!("[audio] rubato process error: {}, falling back to linear", e);
                return resample_linear(samples, from_rate, to_rate);
            }
        }
        pos += chunk_size;
    }

    // Process remaining samples (partial chunk)
    if pos < samples.len() {
        let remainder = &samples[pos..];
        match resampler.process_partial(Some(&[remainder]), None) {
            Ok(result) => {
                if let Some(channel) = result.first() {
                    output.extend_from_slice(channel);
                }
            }
            Err(e) => {
                eprintln!("[audio] rubato process_partial error: {}, falling back to linear", e);
                return resample_linear(samples, from_rate, to_rate);
            }
        }
    }

    output
}

/// Fallback linear interpolation resampler (used if rubato fails)
fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;

        if src_idx + 1 < samples.len() {
            let sample =
                samples[src_idx] as f64 * (1.0 - frac) + samples[src_idx + 1] as f64 * frac;
            resampled.push(sample as f32);
        } else if src_idx < samples.len() {
            resampled.push(samples[src_idx]);
        }
    }

    resampled
}
