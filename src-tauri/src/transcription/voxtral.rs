//! Voxtral transcription engine — alternative to Whisper using Mistral's
//! Voxtral Mini 4B Realtime model via the voxtral.c C library.
//!
//! Only compiled when the `voxtral` Cargo feature is enabled.

use crate::audio::capture::{prepare_for_whisper, snapshot_whisper_buffer};
use crate::audio::AudioData;
use crate::settings::UserSettings;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use super::voxtral_ffi::VoxtralContext;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum VoxtralError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Model download failed: {0}")]
    DownloadError(String),
    #[error("Transcription failed: {0}")]
    TranscriptionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MODEL_ID: &str = "voxtral-mini-4b";
const MODEL_NAME: &str = "Voxtral Mini 4B Realtime";
const MODEL_SIZE_MB: u32 = 8900; // ~8.9 GB safetensors

const HF_BASE_URL: &str =
    "https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602/resolve/main";

/// Files required for the model.
const MODEL_FILES: &[(&str, u64)] = &[
    ("consolidated.safetensors", 8_900_000_000),
    ("tekken.json", 15_000_000),
    ("params.json", 500),
];

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

fn get_model_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".mentascribe")
        .join("models")
        .join(MODEL_ID)
}

// ---------------------------------------------------------------------------
// Model status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxtralStatus {
    /// voxtral feature was compiled into this build
    pub compiled: bool,
    /// Metal GPU acceleration is available (Apple Silicon)
    pub metal: bool,
    /// Model files are downloaded
    pub model_downloaded: bool,
    /// Model is currently loaded in memory
    pub model_loaded: bool,
}

pub fn get_status() -> VoxtralStatus {
    let model_dir = get_model_dir();
    let downloaded = MODEL_FILES.iter().all(|(name, _)| model_dir.join(name).exists());
    let loaded = VOXTRAL_CACHE.lock().map(|c| c.context.is_some()).unwrap_or(false);

    VoxtralStatus {
        compiled: true,
        metal: cfg!(all(target_os = "macos", target_arch = "aarch64")),
        model_downloaded: downloaded,
        model_loaded: loaded,
    }
}

pub fn is_model_downloaded() -> bool {
    let model_dir = get_model_dir();
    MODEL_FILES.iter().all(|(name, _)| model_dir.join(name).exists())
}

pub fn get_available_models() -> Vec<super::ModelInfo> {
    let downloaded = is_model_downloaded();
    vec![super::ModelInfo {
        id: MODEL_ID.to_string(),
        name: MODEL_NAME.to_string(),
        size_mb: MODEL_SIZE_MB,
        downloaded,
        coreml_downloaded: false, // Voxtral uses Metal directly, not CoreML
        coreml_size_mb: 0,
    }]
}

// ---------------------------------------------------------------------------
// Model download
// ---------------------------------------------------------------------------

pub async fn download_model<F: Fn(f64) + Send + 'static>(
    on_progress: F,
) -> Result<(), VoxtralError> {
    let model_dir = get_model_dir();
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| VoxtralError::DownloadError(format!("Failed to create model dir: {}", e)))?;

    let total_bytes: u64 = MODEL_FILES.iter().map(|(_, size)| size).sum();
    let mut downloaded_bytes: u64 = 0;

    let client = reqwest::Client::new();

    for (filename, expected_size) in MODEL_FILES {
        let file_path = model_dir.join(filename);

        // Skip if already downloaded and roughly the right size
        if file_path.exists() {
            if let Ok(meta) = std::fs::metadata(&file_path) {
                // Allow 10% tolerance for size check
                if meta.len() > expected_size / 2 {
                    log::info!("Voxtral model file '{}' already exists, skipping", filename);
                    downloaded_bytes += expected_size;
                    on_progress(downloaded_bytes as f64 / total_bytes as f64 * 100.0);
                    continue;
                }
            }
        }

        let url = format!("{}/{}", HF_BASE_URL, filename);
        log::info!("Downloading voxtral model file: {}", url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| VoxtralError::DownloadError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(VoxtralError::DownloadError(format!(
                "HTTP {} for {}",
                response.status(),
                url
            )));
        }

        let content_length = response.content_length().unwrap_or(*expected_size);

        // Download with progress tracking (using response.chunk() like whisper.rs)
        let tmp_path = file_path.with_extension("part");
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| VoxtralError::DownloadError(format!("Failed to create file: {}", e)))?;

        use std::io::Write;
        let mut response = response;
        let mut file_downloaded: u64 = 0;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| VoxtralError::DownloadError(format!("Download error: {}", e)))?
        {
            file.write_all(&chunk)
                .map_err(|e| VoxtralError::DownloadError(format!("Write error: {}", e)))?;
            file_downloaded += chunk.len() as u64;

            let total_progress =
                (downloaded_bytes + file_downloaded) as f64 / total_bytes as f64 * 100.0;
            on_progress(total_progress);
        }

        // Atomic rename
        std::fs::rename(&tmp_path, &file_path)
            .map_err(|e| VoxtralError::DownloadError(format!("Rename failed: {}", e)))?;

        downloaded_bytes += content_length;
        log::info!("Downloaded voxtral model file: {}", filename);
    }

    on_progress(100.0);
    Ok(())
}

pub fn delete_model() -> Result<(), VoxtralError> {
    // Unload from cache first
    if let Ok(mut cache) = VOXTRAL_CACHE.lock() {
        cache.context = None;
    }

    let model_dir = get_model_dir();
    if model_dir.exists() {
        std::fs::remove_dir_all(&model_dir)?;
        log::info!("Deleted voxtral model directory: {:?}", model_dir);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Model cache (same pattern as whisper.rs MODEL_CACHE)
// ---------------------------------------------------------------------------

struct VoxtralCache {
    context: Option<Arc<VoxtralContext>>,
}

static VOXTRAL_CACHE: Lazy<Mutex<VoxtralCache>> = Lazy::new(|| {
    Mutex::new(VoxtralCache { context: None })
});

/// Load the voxtral model into cache. No-op if already loaded.
pub fn preload_model() -> Result<(), VoxtralError> {
    let model_dir = get_model_dir();
    if !is_model_downloaded() {
        return Err(VoxtralError::ModelNotFound(format!(
            "Voxtral model not downloaded at {:?}",
            model_dir
        )));
    }

    let mut cache = VOXTRAL_CACHE
        .lock()
        .map_err(|e| VoxtralError::TranscriptionError(format!("Cache lock error: {}", e)))?;

    if cache.context.is_some() {
        log::info!("Voxtral model already cached, skipping preload");
        return Ok(());
    }

    let model_dir_str = model_dir
        .to_str()
        .ok_or_else(|| VoxtralError::ModelNotFound("Invalid model path".to_string()))?;

    log::info!("Loading voxtral model from {:?}...", model_dir);
    let start = std::time::Instant::now();

    let ctx = VoxtralContext::load(model_dir_str)
        .map_err(|e| VoxtralError::TranscriptionError(e))?;

    let elapsed = start.elapsed().as_secs_f64();
    log::info!("Voxtral model loaded in {:.2}s", elapsed);

    cache.context = Some(Arc::new(ctx));
    Ok(())
}

/// Unload the voxtral model from cache (frees GPU memory).
pub fn unload_model() {
    if let Ok(mut cache) = VOXTRAL_CACHE.lock() {
        if cache.context.is_some() {
            cache.context = None;
            log::info!("Voxtral model unloaded from cache");
        }
    }
}

fn get_cached_context() -> Result<Arc<VoxtralContext>, VoxtralError> {
    let cache = VOXTRAL_CACHE
        .lock()
        .map_err(|e| VoxtralError::TranscriptionError(format!("Cache lock error: {}", e)))?;

    cache.context.clone().ok_or_else(|| {
        VoxtralError::TranscriptionError("Voxtral model not loaded".to_string())
    })
}

// ---------------------------------------------------------------------------
// One-shot transcription
// ---------------------------------------------------------------------------

/// Transcribe audio using voxtral. Combines streaming prefix with tail transcription,
/// matching the same pattern as whisper::transcribe().
pub async fn transcribe(
    audio: AudioData,
    settings: &UserSettings,
    streaming_prefix: Option<String>,
) -> Result<String, VoxtralError> {
    // Prepare audio (16kHz mono f32)
    let samples = prepare_for_whisper(audio);

    // If no tail audio, return just the streaming prefix
    if samples.is_empty() {
        return Ok(streaming_prefix.unwrap_or_default());
    }

    // Get cached model context
    let ctx = get_cached_context()?;

    // Set delay from settings
    let delay_ms = settings
        .transcription
        .voxtral_delay_ms
        .unwrap_or(480);
    ctx.set_delay(delay_ms);

    // Run transcription in a blocking thread
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    std::thread::Builder::new()
        .name("voxtral-transcribe".to_string())
        .spawn(move || {
            let result = ctx.transcribe_audio(&samples);
            result_tx.send(result).ok();
        })
        .map_err(|e| VoxtralError::TranscriptionError(format!("Thread spawn failed: {}", e)))?;

    let tail_text = result_rx
        .await
        .map_err(|_| VoxtralError::TranscriptionError("Transcription thread dropped".into()))?
        .map_err(|e| VoxtralError::TranscriptionError(e))?;

    // Combine streaming prefix with tail transcription
    match streaming_prefix {
        Some(prefix) if !prefix.is_empty() => {
            if tail_text.is_empty() {
                Ok(prefix)
            } else {
                Ok(format!("{} {}", prefix, tail_text))
            }
        }
        _ => Ok(tail_text),
    }
}

// ---------------------------------------------------------------------------
// Native streaming (Voxtral processes audio incrementally — no VAD needed)
// ---------------------------------------------------------------------------

/// Accumulated token text from voxtral streaming during recording.
static VOXTRAL_STREAMING_RESULTS: Lazy<Mutex<Vec<String>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// Signal to stop the streaming loop.
static VOXTRAL_STREAMING_STOP: AtomicBool = AtomicBool::new(false);

/// Handle for the streaming thread.
static VOXTRAL_STREAM_HANDLE: Lazy<Mutex<Option<std::thread::JoinHandle<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Configuration for voxtral streaming.
pub struct StreamingConfig {
    pub delay_ms: i32,
}

/// Start voxtral native streaming transcription.
/// Spawns a background thread that polls the WHISPER_BUFFER and feeds audio
/// to vox_stream_feed(), collecting tokens in real-time.
/// Returns an error if the model is not downloaded or not loaded.
pub fn start_streaming(config: StreamingConfig) -> Result<(), VoxtralError> {
    // Clear previous state
    *VOXTRAL_STREAMING_RESULTS.lock().unwrap() = Vec::new();
    VOXTRAL_STREAMING_STOP.store(false, Ordering::SeqCst);

    if !is_model_downloaded() {
        return Err(VoxtralError::ModelNotFound(
            "Voxtral model not downloaded. Please download it in Settings.".to_string(),
        ));
    }

    let ctx = get_cached_context()?;

    // Apply delay setting
    ctx.set_delay(config.delay_ms);

    let thread = std::thread::Builder::new()
        .name("voxtral-streaming".to_string())
        .spawn(move || {
            voxtral_stream_loop(ctx);
        })
        .map_err(|e| VoxtralError::TranscriptionError(format!("Thread spawn failed: {}", e)))?;

    *VOXTRAL_STREAM_HANDLE.lock().unwrap() = Some(thread);
    log::info!("Voxtral streaming started");
    Ok(())
}

/// Stop voxtral streaming. Returns (accumulated_text_segments, consumed_samples).
/// When streaming was active: consumed_samples is usize::MAX (all audio consumed,
/// skip tail transcription). When no thread was running: returns ([], 0).
pub fn stop_streaming() -> (Vec<String>, usize) {
    // Signal stop
    VOXTRAL_STREAMING_STOP.store(true, Ordering::SeqCst);

    // Join thread with timeout to prevent hanging if the C code blocks
    let handle = VOXTRAL_STREAM_HANDLE.lock().unwrap().take();
    let thread_was_running = handle.is_some();
    if let Some(h) = handle {
        // Wait up to 10 seconds for the streaming thread to finish.
        // vox_stream_finish() processes remaining audio and can be slow.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if h.is_finished() {
                h.join().ok();
                log::info!("Voxtral streaming thread joined");
                break;
            }
            if std::time::Instant::now() >= deadline {
                log::warn!("Voxtral streaming thread did not finish within 10s, abandoning");
                // Thread is leaked but won't block the UI. It will eventually
                // complete and be cleaned up when the process exits.
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    let results = std::mem::take(&mut *VOXTRAL_STREAMING_RESULTS.lock().unwrap());
    log::info!("Voxtral streaming results: {} segments", results.len());

    if thread_was_running {
        // Streaming was active — all audio was consumed incrementally,
        // so skip tail transcription.
        (results, usize::MAX)
    } else {
        // No streaming thread was running (model wasn't loaded).
        // Return 0 consumed so the caller knows no audio was processed.
        log::warn!("Voxtral stop_streaming: no thread was running");
        (results, 0)
    }
}

/// Main streaming loop. Polls WHISPER_BUFFER every 50ms, feeds new audio
/// to the voxtral stream, and collects decoded tokens.
fn voxtral_stream_loop(ctx: Arc<VoxtralContext>) {
    let stream = match ctx.stream_init() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Voxtral stream init failed: {}", e);
            return;
        }
    };

    // Enable continuous mode for live recording
    stream.set_continuous(true);

    let mut abs_position: usize = 0;
    let poll_interval = std::time::Duration::from_millis(50);

    while !VOXTRAL_STREAMING_STOP.load(Ordering::SeqCst) {
        // Get new audio since last position
        let (new_samples, new_len) = snapshot_whisper_buffer(abs_position);

        if !new_samples.is_empty() {
            // Feed new audio to voxtral
            if let Err(e) = stream.feed(&new_samples) {
                log::error!("Voxtral stream feed error: {}", e);
                break;
            }
            abs_position = new_len;
        }

        // Poll for decoded tokens
        let tokens = stream.get_tokens(64);
        if !tokens.is_empty() {
            let text: String = tokens.join("");
            if !text.trim().is_empty() {
                VOXTRAL_STREAMING_RESULTS.lock().unwrap().push(text);
            }
        }

        std::thread::sleep(poll_interval);
    }

    // Finish the stream — process remaining buffered audio
    if let Err(e) = stream.finish() {
        log::error!("Voxtral stream finish error: {}", e);
    }

    // Drain remaining tokens after finish
    loop {
        let tokens = stream.get_tokens(64);
        if tokens.is_empty() {
            break;
        }
        let text: String = tokens.join("");
        if !text.trim().is_empty() {
            VOXTRAL_STREAMING_RESULTS.lock().unwrap().push(text);
        }
    }

    // stream is dropped here (calls vox_stream_free)
    log::info!("Voxtral streaming loop finished");
}
