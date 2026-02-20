use crate::audio::{capture::prepare_for_whisper, AudioData};
use crate::settings::UserSettings;
use once_cell::sync::Lazy;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperState, WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

use super::{CoremlStatus, MetalStatus, ModelInfo};

// Cache for the Whisper model context to avoid reloading on every transcription.
// Arc-wrapped so we can clone the context out of the cache and release the mutex
// before running inference (which takes 1-30+ seconds).
struct ModelCache {
    context: Option<Arc<WhisperContext>>,
    model_size: String,
    model_path: PathBuf,
}

static MODEL_CACHE: Lazy<Mutex<ModelCache>> = Lazy::new(|| {
    Mutex::new(ModelCache {
        context: None,
        model_size: String::new(),
        model_path: PathBuf::new(),
    })
});

// Pre-created WhisperState cache. After each transcription, we spawn a background
// thread to create the next WhisperState so it's ready immediately when the user
// stops their next recording. This saves 50-200ms (state allocation is 200-400MB).
// WhisperState holds its own Arc<WhisperInnerContext> so the model stays alive.
struct CachedWhisperState {
    state: WhisperState,
    model_size: String,
}

static STATE_CACHE: Lazy<Mutex<Option<CachedWhisperState>>> = Lazy::new(|| Mutex::new(None));

#[derive(Error, Debug)]
pub enum WhisperError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Model download failed: {0}")]
    DownloadError(String),
    #[error("Transcription failed: {0}")]
    TranscriptionError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
const VAD_MODEL_FILENAME: &str = "ggml-silero-vad.bin";

fn get_models_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".mentascribe")
        .join("models")
}

fn get_model_filename(size: &str) -> String {
    if size == "large" {
        "ggml-large-v3.bin".to_string()
    } else {
        format!("ggml-{}.bin", size)
    }
}

fn get_model_path(size: &str) -> PathBuf {
    get_models_dir().join(get_model_filename(size))
}

/// Get the CoreML encoder model directory name for a given model size.
/// whisper.cpp expects: `ggml-{name}-encoder.mlmodelc/` next to the GGML model.
fn coreml_encoder_name(size: &str) -> String {
    if size == "large" {
        "ggml-large-v3-encoder.mlmodelc".to_string()
    } else {
        format!("ggml-{}-encoder.mlmodelc", size)
    }
}

/// Check if the CoreML encoder model is downloaded for a given size.
pub fn is_coreml_downloaded(size: &str) -> bool {
    get_models_dir().join(coreml_encoder_name(size)).is_dir()
}

/// Approximate GGML model download size in bytes for a given model size.
/// Used as a fallback when Content-Length is absent (chunked transfer encoding).
fn ggml_size_bytes(size: &str) -> u64 {
    match size {
        "tiny" => 75_000_000,
        "base" => 142_000_000,
        "small" => 466_000_000,
        "medium" => 1_500_000_000,
        "large" => 2_900_000_000,
        "large-v3-turbo" => 1_500_000_000,
        "large-v3-turbo-q5_0" => 547_000_000,
        "large-v3-q5_0" => 1_100_000_000,
        "distil-large-v3.5" => 756_000_000,
        _ => 0,
    }
}

/// Approximate CoreML encoder zip download size in bytes for a given model size.
/// Used as a fallback when Content-Length is absent (chunked transfer encoding).
/// Returns 0 for models without available CoreML encoders.
fn coreml_size_bytes(size: &str) -> u64 {
    match size {
        "tiny" => 42_000_000,
        "base" => 78_000_000,
        "small" => 244_000_000,
        "medium" => 776_000_000,
        "large" => 1_550_000_000,
        _ => 0,
    }
}

/// Approximate CoreML encoder zip download sizes (MB) from HuggingFace.
/// Returns 0 for models without available CoreML encoders (quantized, turbo).
fn coreml_size_mb(size: &str) -> u32 {
    match size {
        "tiny" => 42,
        "base" => 78,
        "small" => 244,
        "medium" => 776,
        "large" => 1550,
        // Quantized and turbo models don't have separate CoreML encoders
        _ => 0,
    }
}

/// Get CoreML support status for this platform.
pub fn get_coreml_status() -> CoremlStatus {
    CoremlStatus {
        compiled: cfg!(target_os = "macos"),
        supported: cfg!(target_os = "macos"),
        apple_silicon: cfg!(all(target_os = "macos", target_arch = "aarch64")),
    }
}

/// Get Metal GPU support status for this platform.
/// Metal is compiled via the "metal" feature on whisper-rs (macOS only)
/// and enabled at runtime via `ctx_params.use_gpu(true)`.
pub fn get_metal_status() -> MetalStatus {
    MetalStatus {
        compiled: cfg!(target_os = "macos"),
        supported: cfg!(target_os = "macos"),
    }
}

pub fn get_available_models() -> Vec<ModelInfo> {
    let models_dir = get_models_dir();

    vec![
        ModelInfo {
            id: "tiny".to_string(),
            name: "Tiny".to_string(),
            size_mb: 75,
            downloaded: models_dir.join("ggml-tiny.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("tiny"),
            coreml_size_mb: coreml_size_mb("tiny"),
        },
        ModelInfo {
            id: "base".to_string(),
            name: "Base".to_string(),
            size_mb: 142,
            downloaded: models_dir.join("ggml-base.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("base"),
            coreml_size_mb: coreml_size_mb("base"),
        },
        ModelInfo {
            id: "small".to_string(),
            name: "Small".to_string(),
            size_mb: 466,
            downloaded: models_dir.join("ggml-small.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("small"),
            coreml_size_mb: coreml_size_mb("small"),
        },
        ModelInfo {
            id: "medium".to_string(),
            name: "Medium".to_string(),
            size_mb: 1500,
            downloaded: models_dir.join("ggml-medium.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("medium"),
            coreml_size_mb: coreml_size_mb("medium"),
        },
        ModelInfo {
            id: "large".to_string(),
            name: "Large v3".to_string(),
            size_mb: 2900,
            downloaded: models_dir.join("ggml-large-v3.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("large"),
            coreml_size_mb: coreml_size_mb("large"),
        },
        ModelInfo {
            id: "large-v3-turbo".to_string(),
            name: "Large v3 Turbo".to_string(),
            size_mb: 1500,
            downloaded: models_dir.join("ggml-large-v3-turbo.bin").exists(),
            coreml_downloaded: false,
            coreml_size_mb: 0,
        },
        ModelInfo {
            id: "large-v3-turbo-q5_0".to_string(),
            name: "Large v3 Turbo Q5".to_string(),
            size_mb: 547,
            downloaded: models_dir.join("ggml-large-v3-turbo-q5_0.bin").exists(),
            coreml_downloaded: false,
            coreml_size_mb: 0,
        },
        ModelInfo {
            id: "large-v3-q5_0".to_string(),
            name: "Large v3 Q5".to_string(),
            size_mb: 1100,
            downloaded: models_dir.join("ggml-large-v3-q5_0.bin").exists(),
            coreml_downloaded: false,
            coreml_size_mb: 0,
        },
        ModelInfo {
            id: "distil-large-v3.5".to_string(),
            name: "Distil Large v3.5".to_string(),
            size_mb: 756,
            downloaded: models_dir.join("ggml-distil-large-v3.5.bin").exists(),
            coreml_downloaded: false,
            coreml_size_mb: 0,
        },
    ]
}

/// Get the download URL for a model. Most models are in the ggerganov/whisper.cpp repo,
/// but distil models are hosted in separate distil-whisper repos with different filenames.
fn get_model_download_url(size: &str) -> String {
    match size {
        "distil-large-v3.5" => {
            "https://huggingface.co/distil-whisper/distil-large-v3.5-ggml/resolve/main/ggml-model.bin".to_string()
        }
        _ => format!("{}/{}", MODEL_BASE_URL, get_model_filename(size)),
    }
}

pub async fn download_model(
    size: &str,
    on_progress: impl Fn(u8),
) -> Result<(), WhisperError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let model_name = get_model_filename(size);
    let url = get_model_download_url(size);
    let path = models_dir.join(&model_name);

    log::info!("Downloading model '{}' from {} to {:?}", size, url, path);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(WhisperError::DownloadError(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or_else(|| ggml_size_bytes(size));
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;
    let mut file =
        std::fs::File::create(&path).map_err(|e| WhisperError::DownloadError(e.to_string()))?;
    let mut response = response;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?
    {
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let percent = (downloaded * 100 / total_size).min(100) as u8;
            if percent != last_percent {
                last_percent = percent;
                on_progress(percent);
            }
        }
    }

    log::info!("Model downloaded successfully ({} bytes)", downloaded);
    Ok(())
}

/// Delete a downloaded GGML model.
pub fn delete_model(size: &str) -> Result<(), WhisperError> {
    let model_path = get_model_path(size);
    if model_path.exists() {
        std::fs::remove_file(&model_path)?;
        // Clear cache if this was the cached model
        if let Ok(mut cache) = MODEL_CACHE.lock() {
            if cache.model_size == size {
                cache.context = None;
                cache.model_size.clear();
                cache.model_path = PathBuf::new();
            }
        }
        log::info!("Deleted GGML model: {}", size);
    }
    Ok(())
}

/// Delete a downloaded CoreML encoder model.
pub fn delete_coreml_model(size: &str) -> Result<(), WhisperError> {
    let dir = get_models_dir().join(coreml_encoder_name(size));
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir)?;
        log::info!("Deleted CoreML model: {}", size);
    }
    Ok(())
}

/// Download the CoreML encoder model for a given size.
/// Downloads the zip from HuggingFace and extracts it into the models directory.
/// Calls `on_progress(percent)` during download (0-99), and 100 is reserved for extraction complete.
pub async fn download_coreml_model(
    size: &str,
    on_progress: impl Fn(u8),
) -> Result<(), WhisperError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let encoder_name = coreml_encoder_name(size);
    let zip_name = format!("{}.zip", encoder_name);
    let url = format!("{}/{}", MODEL_BASE_URL, zip_name);
    let zip_path = models_dir.join(&zip_name);
    let dest_dir = models_dir.join(&encoder_name);

    // Skip if already downloaded
    if dest_dir.is_dir() {
        log::info!("CoreML model already exists: {:?}", dest_dir);
        on_progress(100);
        return Ok(());
    }

    log::info!("Downloading CoreML model from {} to {:?}", url, zip_path);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(WhisperError::DownloadError(format!(
            "HTTP {} for CoreML model",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or_else(|| coreml_size_bytes(size));
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;
    let mut file = std::fs::File::create(&zip_path)
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;
    let mut response = response;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?
    {
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            // Cap download phase at 99% — 100% means extraction done
            let percent = (downloaded * 99 / total_size).min(99) as u8;
            if percent != last_percent {
                last_percent = percent;
                on_progress(percent);
            }
        }
    }
    drop(file);

    log::info!(
        "CoreML zip downloaded ({} bytes), extracting...",
        downloaded
    );

    // Extract using unzip (always available on macOS)
    let output = std::process::Command::new("unzip")
        .arg("-o") // overwrite without prompting
        .arg("-q") // quiet
        .arg(&zip_path)
        .arg("-d")
        .arg(&models_dir)
        .output()
        .map_err(|e| WhisperError::DownloadError(format!("Failed to run unzip: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Clean up zip on failure
        std::fs::remove_file(&zip_path).ok();
        return Err(WhisperError::DownloadError(format!(
            "unzip failed: {}",
            stderr
        )));
    }

    // Clean up zip file
    std::fs::remove_file(&zip_path).ok();

    if dest_dir.is_dir() {
        log::info!("CoreML model extracted successfully: {:?}", dest_dir);
        on_progress(100);
    } else {
        return Err(WhisperError::DownloadError(format!(
            "Extraction succeeded but {:?} not found",
            dest_dir
        )));
    }

    Ok(())
}

fn get_vad_model_path() -> PathBuf {
    get_models_dir().join(VAD_MODEL_FILENAME)
}

/// Download the Silero VAD model (~2MB) if not already present.
/// Called automatically during model preload.
pub async fn ensure_vad_model() -> Result<(), WhisperError> {
    let path = get_vad_model_path();
    if path.exists() {
        return Ok(());
    }

    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let url = format!("{}/{}", MODEL_BASE_URL, VAD_MODEL_FILENAME);
    log::info!("Downloading VAD model from {} to {:?}", url, path);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(WhisperError::DownloadError(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;

    std::fs::write(&path, &bytes)?;
    log::info!("VAD model downloaded successfully ({} bytes)", bytes.len());
    Ok(())
}

// Wrapper to allow WhisperVadContext in a static Mutex.
// Safety: whisper.cpp VAD context uses no thread-local storage and is safe
// to move between threads. It's only accessed behind the VAD_CACHE mutex.
struct SendableVadContext(WhisperVadContext);
unsafe impl Send for SendableVadContext {}

// Cached VAD context to avoid reloading the 2MB Silero model from disk
// on every transcription. Saves ~5-20ms per call (disk I/O + ONNX init).
static VAD_CACHE: Lazy<Mutex<Option<SendableVadContext>>> = Lazy::new(|| Mutex::new(None));

/// Pre-filter audio using Silero VAD to extract only speech segments.
/// This strips non-speech audio before whisper inference, dramatically reducing
/// computation for recordings with silence/noise.
///
/// Returns the filtered audio samples, or the original samples if VAD is unavailable.
/// Expects 16kHz mono f32 input.
fn vad_filter_speech(samples: &[f32]) -> Vec<f32> {
    let vad_path = get_vad_model_path();
    if !vad_path.exists() {
        log::debug!("VAD model not found, skipping pre-filtering");
        return samples.to_vec();
    }

    let vad_start = std::time::Instant::now();

    // Get or create cached VAD context
    let mut vad_guard = match VAD_CACHE.lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::warn!("VAD cache lock poisoned: {}, skipping pre-filtering", e);
            return samples.to_vec();
        }
    };

    if vad_guard.is_none() {
        let mut ctx_params = WhisperVadContextParams::new();
        ctx_params.set_n_threads(2);

        match WhisperVadContext::new(vad_path.to_str().unwrap(), ctx_params) {
            Ok(ctx) => {
                log::info!("VAD context created and cached");
                *vad_guard = Some(SendableVadContext(ctx));
            }
            Err(e) => {
                log::warn!("Failed to load VAD model: {}, skipping pre-filtering", e);
                return samples.to_vec();
            }
        }
    }

    let vad_ctx = &mut vad_guard.as_mut().unwrap().0;

    // Configure VAD params for dictation use
    let mut vad_params = WhisperVadParams::new();
    vad_params.set_threshold(0.5);
    vad_params.set_min_speech_duration(250); // 250ms minimum speech
    vad_params.set_min_silence_duration(100); // 100ms silence to split
    vad_params.set_speech_pad(30); // 30ms padding around speech

    // Run VAD to get speech timestamps
    let segments = match vad_ctx.segments_from_samples(vad_params, samples) {
        Ok(segs) => segs,
        Err(e) => {
            log::warn!("VAD inference failed: {}, skipping pre-filtering", e);
            return samples.to_vec();
        }
    };

    let n_segments = segments.num_segments();
    if n_segments == 0 {
        log::info!("VAD: no speech detected in audio, passing through unchanged");
        return samples.to_vec();
    }

    // Extract speech samples from detected segments
    let mut speech_samples = Vec::new();
    for seg in segments {
        // Timestamps are in centiseconds (0.01s), convert to sample indices at 16kHz
        let start_sample = (seg.start * 160.0) as usize; // 0.01s * 16000 = 160 samples/cs
        let end_sample = ((seg.end * 160.0) as usize).min(samples.len());
        if start_sample < end_sample {
            speech_samples.extend_from_slice(&samples[start_sample..end_sample]);
        }
    }

    if speech_samples.is_empty() {
        log::warn!("VAD: extracted 0 speech samples, passing through original");
        return samples.to_vec();
    }

    let original_duration = samples.len() as f32 / 16000.0;
    let filtered_duration = speech_samples.len() as f32 / 16000.0;

    // Guard: if VAD filtered audio is too short (<0.5s), whisper can produce
    // degenerate output (hallucinations, empty text). Fall back to original audio.
    if filtered_duration < 0.5 {
        log::warn!(
            "VAD: filtered audio too short ({:.2}s < 0.5s), passing through original ({:.2}s)",
            filtered_duration,
            original_duration
        );
        return samples.to_vec();
    }
    let vad_elapsed = vad_start.elapsed();

    log::info!(
        "VAD: {:.2}s -> {:.2}s ({} segments, {:.0}% reduction) in {:.1}ms",
        original_duration,
        filtered_duration,
        n_segments,
        (1.0 - filtered_duration / original_duration) * 100.0,
        vad_elapsed.as_secs_f64() * 1000.0
    );

    speech_samples
}

// ======================= VAD-Triggered Streaming =======================
//
// During recording, a background VAD monitor thread periodically reads the
// WHISPER_BUFFER, detects completed utterances (speech followed by silence),
// and transcribes them immediately. On stop, only the remaining "tail" audio
// needs transcription, reducing perceived latency from 1-3s to 200-500ms.

/// Accumulated transcription text from completed utterances during recording.
static STREAMING_RESULTS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// How many samples from the start of WHISPER_BUFFER have been fully
/// transcribed by the streaming monitor. Used to compute the "tail" on stop.
static STREAMING_CONSUMED: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

/// Handle for the VAD monitor thread.
struct VadMonitorHandle {
    stop_sender: std::sync::mpsc::Sender<()>,
    thread_handle: std::thread::JoinHandle<()>,
}

static VAD_MONITOR: Lazy<Mutex<Option<VadMonitorHandle>>> = Lazy::new(|| Mutex::new(None));

/// Configuration for streaming transcription during recording.
pub struct StreamingConfig {
    pub model_size: String,
    pub language: Option<String>,
}

/// Start the VAD-triggered streaming monitor.
/// Call this after `start_capture()` to enable background transcription during recording.
pub fn start_streaming(config: StreamingConfig) {
    // Clear previous streaming state
    *STREAMING_RESULTS.lock().unwrap() = Vec::new();
    *STREAMING_CONSUMED.lock().unwrap() = 0;

    // Check if VAD model is available
    let vad_path = get_vad_model_path();
    if !vad_path.exists() {
        log::info!("VAD model not found, streaming transcription disabled");
        return;
    }

    // Check if whisper model is available
    let model_path = get_model_path(&config.model_size);
    if !model_path.exists() {
        log::info!("Whisper model not found, streaming transcription disabled");
        return;
    }

    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let thread = std::thread::Builder::new()
        .name("vad-streaming-monitor".to_string())
        .spawn(move || {
            vad_monitor_loop(stop_rx, config);
        })
        .expect("Failed to spawn VAD monitor thread");

    *VAD_MONITOR.lock().unwrap() = Some(VadMonitorHandle {
        stop_sender: stop_tx,
        thread_handle: thread,
    });

    log::info!("VAD streaming monitor started");
}

/// Stop the VAD monitor and return (accumulated_results, consumed_sample_count).
/// After this returns, all streaming transcriptions are complete.
pub fn stop_streaming() -> (Vec<String>, usize) {
    let handle = VAD_MONITOR.lock().unwrap().take();
    if let Some(h) = handle {
        // Signal stop
        h.stop_sender.send(()).ok();
        // Join thread — ensures all in-progress transcriptions complete
        h.thread_handle.join().ok();
        log::info!("VAD streaming monitor stopped");
    }

    let results = std::mem::take(&mut *STREAMING_RESULTS.lock().unwrap());
    let consumed = *STREAMING_CONSUMED.lock().unwrap();
    *STREAMING_CONSUMED.lock().unwrap() = 0;

    log::info!(
        "Streaming results: {} segments, {} samples consumed",
        results.len(),
        consumed
    );

    (results, consumed)
}

/// Main loop for the VAD streaming monitor thread.
/// Periodically reads new audio from WHISPER_BUFFER, runs VAD to detect
/// completed utterances, and transcribes them in-thread.
fn vad_monitor_loop(stop_rx: std::sync::mpsc::Receiver<()>, config: StreamingConfig) {
    use crate::audio::capture::snapshot_whisper_buffer;

    let mut abs_position: usize = 0; // Next sample to read from WHISPER_BUFFER
    let mut pending_audio: Vec<f32> = Vec::with_capacity(16000 * 10); // ~10s capacity
    let mut pending_start: usize = 0; // Absolute position of pending_audio[0]

    // Minimum silence gap after speech to consider an utterance "complete" (in seconds)
    const MIN_SILENCE_GAP: f32 = 0.5;
    // Minimum speech duration to bother transcribing (in samples at 16kHz)
    const MIN_SPEECH_SAMPLES: usize = 8000; // 0.5s
    // How often to check for new audio (ms)
    const CHECK_INTERVAL_MS: u64 = 300;
    // Minimum audio to accumulate before running VAD (in samples at 16kHz)
    const MIN_VAD_SAMPLES: usize = 16000; // 1s

    log::info!(
        "VAD monitor loop started (model={}, gap={:.1}s, interval={}ms)",
        config.model_size,
        MIN_SILENCE_GAP,
        CHECK_INTERVAL_MS
    );

    loop {
        // Check for stop signal (non-blocking)
        if stop_rx.try_recv().is_ok() {
            log::info!("VAD monitor received stop signal");
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(CHECK_INTERVAL_MS));

        // Get new samples from WHISPER_BUFFER
        let (new_samples, new_len) = snapshot_whisper_buffer(abs_position);
        if !new_samples.is_empty() {
            abs_position = new_len;
            pending_audio.extend_from_slice(&new_samples);
        }

        // Need minimum audio to run meaningful VAD
        if pending_audio.len() < MIN_VAD_SAMPLES {
            continue;
        }

        // Run VAD on pending audio to find speech segments
        let vad_path = get_vad_model_path();
        if !vad_path.exists() {
            continue;
        }

        let vad_start = std::time::Instant::now();
        let mut vad_guard = match VAD_CACHE.lock() {
            Ok(guard) => guard,
            Err(_) => continue,
        };

        if vad_guard.is_none() {
            let mut ctx_params = WhisperVadContextParams::new();
            ctx_params.set_n_threads(2);
            match WhisperVadContext::new(vad_path.to_str().unwrap(), ctx_params) {
                Ok(ctx) => {
                    *vad_guard = Some(SendableVadContext(ctx));
                }
                Err(e) => {
                    log::warn!("Failed to load VAD model for streaming: {}", e);
                    continue;
                }
            }
        }

        let vad_ctx = &mut vad_guard.as_mut().unwrap().0;
        let mut vad_params = WhisperVadParams::new();
        vad_params.set_threshold(0.5);
        vad_params.set_min_speech_duration(250);
        vad_params.set_min_silence_duration(100);
        vad_params.set_speech_pad(30);

        let segments = match vad_ctx.segments_from_samples(vad_params, &pending_audio) {
            Ok(segs) => segs,
            Err(e) => {
                log::warn!("VAD inference failed in streaming: {}", e);
                continue;
            }
        };

        // Collect segment timestamps (start/end in centiseconds)
        let seg_list: Vec<(f32, f32)> = segments.into_iter().map(|s| (s.start, s.end)).collect();

        // Release VAD lock before potentially long transcription
        drop(vad_guard);

        let vad_elapsed = vad_start.elapsed();

        if seg_list.is_empty() {
            continue;
        }

        // Check if there's a completed utterance: last segment must end with enough
        // silence gap before the end of pending audio
        let pending_duration_sec = pending_audio.len() as f32 / 16000.0;
        let last_seg_end_sec = seg_list.last().unwrap().1 * 0.01; // centiseconds → seconds
        let gap = pending_duration_sec - last_seg_end_sec;

        if gap < MIN_SILENCE_GAP {
            // Speech is still ongoing or gap too short — wait for more audio
            continue;
        }

        // We have a completed utterance! Extract speech samples.
        let mut speech_samples: Vec<f32> = Vec::new();
        for &(start_cs, end_cs) in &seg_list {
            let start_sample = (start_cs * 160.0) as usize; // 0.01s * 16000 = 160
            let end_sample = ((end_cs * 160.0) as usize).min(pending_audio.len());
            if start_sample < end_sample {
                speech_samples.extend_from_slice(&pending_audio[start_sample..end_sample]);
            }
        }

        if speech_samples.len() < MIN_SPEECH_SAMPLES {
            // Too short to transcribe reliably — wait for more
            continue;
        }

        log::info!(
            "VAD streaming: detected completed utterance ({} segments, {:.2}s speech, {:.2}s gap, VAD took {:.1}ms)",
            seg_list.len(),
            speech_samples.len() as f32 / 16000.0,
            gap,
            vad_elapsed.as_secs_f64() * 1000.0
        );

        // Check stop signal BEFORE starting a potentially long transcription (1-30s).
        // Without this check, stop_streaming() would block until an in-progress
        // run_whisper() completes, adding seconds of latency to the stop path.
        if stop_rx.try_recv().is_ok() {
            log::info!("VAD monitor received stop signal before transcription, aborting to let tail handle it");
            break;
        }

        // Transcribe the completed utterance directly (we're already on a background thread).
        // No need to send to the dedicated transcription thread — that's for the final tail.
        let transcription_start = std::time::Instant::now();
        let model_path = get_model_path(&config.model_size);
        match run_whisper(
            &model_path,
            &config.model_size,
            &speech_samples,
            config.language.as_deref(),
        ) {
            Ok(text) => {
                if !text.is_empty() {
                    log::info!(
                        "VAD streaming: transcribed '{}' in {:.2}s",
                        if text.len() > 60 { format!("{}...", &text[..60]) } else { text.clone() },
                        transcription_start.elapsed().as_secs_f64()
                    );
                    STREAMING_RESULTS.lock().unwrap().push(text);
                } else {
                    log::info!(
                        "VAD streaming: empty transcription (hallucination suppressed) in {:.2}s",
                        transcription_start.elapsed().as_secs_f64()
                    );
                }
            }
            Err(e) => {
                log::warn!("VAD streaming: transcription failed: {}", e);
            }
        }

        // Advance past the consumed audio. Clear everything up to the end of the
        // last speech segment + some padding to avoid re-processing.
        let clear_to_sample = ((seg_list.last().unwrap().1 * 160.0) as usize)
            .min(pending_audio.len());
        pending_start += clear_to_sample;
        pending_audio.drain(..clear_to_sample);

        // Update consumed count so stop_capture knows the tail boundary
        *STREAMING_CONSUMED.lock().unwrap() = pending_start;
    }

    log::info!("VAD monitor loop exiting");
}

/// Preload the Whisper model into MODEL_CACHE so the first transcription is fast.
///
/// This loads the GGML model file from disk, initializes the Metal/CoreML GPU backend,
/// and calls `create_state()` once to trigger any first-run CoreML model compilation.
/// The state is then discarded — only the context is kept cached.
///
/// Safe to call from a background thread via `std::thread::spawn` or `spawn_blocking`.
/// If the model is already cached with the same size, this is a no-op.
pub fn preload_model(model_size: &str) -> Result<(), WhisperError> {
    let model_path = get_model_path(model_size);

    if !model_path.exists() {
        return Err(WhisperError::ModelNotFound(format!(
            "Model '{}' not found at {:?}",
            model_size, model_path
        )));
    }

    let total_start = std::time::Instant::now();

    // Lock the cache and check if we already have this model loaded
    let mut cache = MODEL_CACHE
        .lock()
        .map_err(|e| WhisperError::TranscriptionError(format!("Cache lock error: {}", e)))?;

    if cache.context.is_some()
        && cache.model_size == model_size
        && cache.model_path == model_path
    {
        log::info!(
            "preload_model: model '{}' already cached, skipping",
            model_size
        );
        return Ok(());
    }

    // Load the model
    log::info!(
        "preload_model: loading '{}' from {:?}",
        model_size,
        model_path
    );
    let load_start = std::time::Instant::now();

    let mut ctx_params = WhisperContextParameters::default();
    ctx_params.flash_attn(true);
    ctx_params.use_gpu(true); // Enable Metal GPU acceleration for decoder

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        ctx_params,
    )
    .map_err(|e| WhisperError::TranscriptionError(format!("Failed to load model: {}", e)))?;

    let load_elapsed = load_start.elapsed();
    log::info!(
        "preload_model: model loaded in {:.2}s",
        load_elapsed.as_secs_f64()
    );

    // Create state to trigger CoreML first-run compilation, then cache it
    // for the first transcription (saves 50-200ms on first use).
    let state_start = std::time::Instant::now();
    let preload_state = ctx
        .create_state()
        .map_err(|e| {
            WhisperError::TranscriptionError(format!(
                "Failed to create state during preload: {}",
                e
            ))
        })?;
    let state_elapsed = state_start.elapsed();
    log::info!(
        "preload_model: state created (CoreML warmup) in {:.2}s, caching for first transcription",
        state_elapsed.as_secs_f64()
    );
    // Cache the state for the first transcription instead of discarding it
    if let Ok(mut state_cache) = STATE_CACHE.lock() {
        *state_cache = Some(CachedWhisperState {
            state: preload_state,
            model_size: model_size.to_string(),
        });
    }

    // Store in cache (Arc-wrapped for lock-free inference)
    cache.context = Some(Arc::new(ctx));
    cache.model_size = model_size.to_string();
    cache.model_path = model_path;

    let total_elapsed = total_start.elapsed();
    log::info!(
        "preload_model: complete in {:.2}s (load={:.2}s, state={:.2}s)",
        total_elapsed.as_secs_f64(),
        load_elapsed.as_secs_f64(),
        state_elapsed.as_secs_f64()
    );

    Ok(())
}

/// A job for the dedicated transcription thread.
struct TranscriptionJob {
    samples: Vec<f32>,
    model_size: String,
    language: Option<String>,
    run_vad: bool,
    result_tx: tokio::sync::oneshot::Sender<Result<String, WhisperError>>,
}

/// Lazy-initialized sender for the dedicated transcription thread.
/// The thread is spawned on first use and persists for the app lifetime.
static TRANSCRIPTION_TX: Lazy<std::sync::mpsc::Sender<TranscriptionJob>> = Lazy::new(|| {
    let (tx, rx) = std::sync::mpsc::channel::<TranscriptionJob>();
    std::thread::Builder::new()
        .name("whisper-transcription".to_string())
        .spawn(move || {
            log::info!("Dedicated transcription thread started");
            for job in rx {
                let samples = if job.run_vad {
                    vad_filter_speech(&job.samples)
                } else {
                    job.samples
                };

                let result = if samples.is_empty() {
                    Ok(String::new())
                } else {
                    let path = get_model_path(&job.model_size);
                    run_whisper(&path, &job.model_size, &samples, job.language.as_deref())
                };

                // Send result back (ignore error if receiver was dropped)
                job.result_tx.send(result).ok();
            }
            log::info!("Dedicated transcription thread exiting");
        })
        .expect("Failed to spawn transcription thread");
    tx
});

/// Transcribe audio, optionally combining with pre-transcribed streaming results.
///
/// When `streaming_prefix` is provided (from VAD-triggered streaming during recording),
/// only the tail audio is transcribed and appended to the streaming results.
pub async fn transcribe(
    audio: AudioData,
    settings: &UserSettings,
    streaming_prefix: Option<String>,
) -> Result<String, WhisperError> {
    let model_size = settings
        .transcription
        .model_size
        .as_deref()
        .unwrap_or("small")
        .to_string();
    let model_path = get_model_path(&model_size);

    if !model_path.exists() {
        return Err(WhisperError::ModelNotFound(model_size.clone()));
    }

    // Prepare audio for Whisper (16kHz mono)
    let samples = prepare_for_whisper(audio);

    // If no tail audio, return just the streaming prefix
    if samples.is_empty() {
        return Ok(streaming_prefix.unwrap_or_default());
    }

    // Send to dedicated transcription thread (replaces tokio::spawn_blocking).
    // The persistent thread avoids thread-pool scheduling overhead (~1-5ms)
    // and keeps a warm execution context.
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();
    let language = settings.transcription.language.clone();

    TRANSCRIPTION_TX
        .send(TranscriptionJob {
            samples,
            model_size,
            language,
            run_vad: true,
            result_tx,
        })
        .map_err(|_| WhisperError::TranscriptionError("Transcription thread closed".into()))?;

    let tail_text = result_rx
        .await
        .map_err(|_| WhisperError::TranscriptionError("Transcription thread dropped result".into()))??;

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

/// Check if a model is a "turbo" variant (pruned to 4 decoder layers).
/// Turbo models are more resilient to aggressive optimizations like reduced
/// audio_ctx and single_segment mode. Full models (32 decoder layers) need
/// more conservative settings to avoid hallucinations.
fn is_turbo_model(model_size: &str) -> bool {
    model_size.contains("turbo")
}

/// Check if a model is a "distil" variant (distilled to 2 decoder layers).
/// Distil models are even more aggressive than turbo (2 vs 4 decoder layers)
/// and can safely use all turbo-level optimizations.
fn is_distil_model(model_size: &str) -> bool {
    model_size.contains("distil")
}

/// Check if a model has a lightweight decoder (turbo=4 layers, distil=2 layers).
/// These models are resilient to aggressive optimizations like reduced audio_ctx
/// and single_segment mode. Full models (32 decoder layers) need conservative settings.
fn is_lightweight_decoder(model_size: &str) -> bool {
    is_turbo_model(model_size) || is_distil_model(model_size)
}

/// Known whisper hallucination phrases that appear when the model generates
/// text from silence or near-silence. These are artifacts from the training
/// data (YouTube subtitles) that the model memorized.
const HALLUCINATION_PHRASES: &[&str] = &[
    "thank you",
    "thanks for watching",
    "thanks for listening",
    "thank you for watching",
    "thank you for listening",
    "please subscribe",
    "like and subscribe",
    "subtitles by",
    "transcribed by",
    "copyright",
    "the end",
    "you",
];

/// Check if text is likely a hallucination (common phrases whisper generates
/// from silence/noise rather than actual speech).
fn is_likely_hallucination(text: &str) -> bool {
    let normalized = text.trim().to_lowercase();
    // Empty or very short results from non-trivial audio are suspicious
    if normalized.is_empty() {
        return false; // Empty is handled elsewhere, not a hallucination
    }
    // Check exact matches and prefix matches against known hallucination phrases
    for phrase in HALLUCINATION_PHRASES {
        if normalized == *phrase
            || normalized == format!("{}.", phrase)
            || normalized == format!("{}!", phrase)
        {
            return true;
        }
    }
    false
}

fn run_whisper(
    model_path: &PathBuf,
    model_size: &str,
    samples: &[f32],
    language: Option<&str>,
) -> Result<String, WhisperError> {
    use whisper_rs::{FullParams, SamplingStrategy};

    let run_start = std::time::Instant::now();

    // Get or create the cached context, then clone the Arc and release the lock.
    // This ensures inference (which takes 1-30s) doesn't block preload or other callers.
    let ctx = {
        let mut cache = MODEL_CACHE
            .lock()
            .map_err(|e| WhisperError::TranscriptionError(format!("Cache lock error: {}", e)))?;

        // Check if we need to reload the model
        if cache.context.is_none()
            || cache.model_size != model_size
            || cache.model_path != *model_path
        {
            log::info!(
                "Loading Whisper model: {} from {:?}",
                model_size,
                model_path
            );

            let load_start = std::time::Instant::now();

            let mut ctx_params = WhisperContextParameters::default();
            ctx_params.flash_attn(true);
            ctx_params.use_gpu(true); // Enable Metal GPU acceleration for decoder

            let new_ctx = WhisperContext::new_with_params(
                model_path.to_str().unwrap(),
                ctx_params,
            )
            .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

            cache.context = Some(Arc::new(new_ctx));
            cache.model_size = model_size.to_string();
            cache.model_path = model_path.clone();

            log::info!(
                "Whisper model loaded and cached in {:.2}s",
                load_start.elapsed().as_secs_f64()
            );
        } else {
            log::info!("Using cached Whisper model: {}", model_size);
        }

        // Clone the Arc (cheap pointer copy) and drop the MutexGuard
        Arc::clone(cache.context.as_ref().unwrap())
    }; // <-- lock released here

    // Try to use a pre-created state from the cache (saves 50-200ms).
    // Only use it if the model matches — model changes invalidate the cache.
    let state_start = std::time::Instant::now();
    let mut state = {
        let cached = STATE_CACHE.lock().ok().and_then(|mut guard| {
            guard.as_ref().map(|c| c.model_size == model_size).unwrap_or(false)
                .then(|| guard.take().unwrap().state)
        });
        if let Some(s) = cached {
            log::info!(
                "Using pre-created WhisperState from cache in {:.4}s",
                state_start.elapsed().as_secs_f64()
            );
            s
        } else {
            let s = ctx
                .create_state()
                .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
            log::info!(
                "Whisper state created (no cache hit) in {:.2}s",
                state_start.elapsed().as_secs_f64()
            );
            s
        }
    };

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Use available performance cores for parallel inference
    // With CoreML handling the encoder on ANE, CPU threads mainly affect the decoder.
    // 4-6 threads often outperform 8 on Apple Silicon due to reduced contention.
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4)
        .min(6);
    params.set_n_threads(n_threads);

    // Determine model characteristics for parameter tuning
    let is_turbo = is_turbo_model(model_size);
    let is_distil = is_distil_model(model_size);
    let is_lightweight = is_lightweight_decoder(model_size);
    let has_coreml = is_coreml_downloaded(model_size);
    let audio_seconds = samples.len() as f32 / 16000.0;

    // === Dynamic audio_ctx: limit encoder window to actual audio length ===
    // Whisper always processes a 30s window (1500 mel frames). For short dictation,
    // most of that is zero-padded silence. Setting audio_ctx proportionally skips it.
    //
    // IMPORTANT: CoreML encoder models are compiled with a FIXED input shape (1500 frames).
    // Setting audio_ctx < 1500 causes shape mismatch -> garbage encoder output -> hallucinations.
    //
    // IMPORTANT: Full large-v3 models (32 decoder layers, including quantized variants like
    // large-v3-q5_0) are very sensitive to reduced audio_ctx. The 32-layer decoder amplifies
    // small encoder artifacts from truncated context into confident hallucinations like
    // "Thank you". Only lightweight models (turbo=4 layers, distil=2 layers) are resilient
    // enough for this optimization.
    let audio_ctx = if has_coreml || !is_lightweight {
        0 // 0 = use default (full 1500 window), safe for CoreML and full-size decoders
    } else {
        // Lightweight decoder + CPU-only: shrink encoder window proportionally for speed
        let ctx = ((audio_seconds / 30.0) * 1500.0).ceil() as i32 + 128;
        let ctx = ((ctx + 255) / 256) * 256; // round up to multiple of 256
        ctx.clamp(768, 1500) // min 768 (quality), max 1500 (whisper limit)
    };
    if audio_ctx > 0 {
        params.set_audio_ctx(audio_ctx);
    }

    // === Temperature settings ===
    // Start with greedy decoding (temperature 0) for speed, but allow fallback
    // with temperature_inc(0.2) so whisper retries with increasing randomness on
    // failure. Disabling fallback entirely (0.0) caused "thank you" hallucinations
    // when VAD produced short/unusual fragments with no recovery path.
    params.set_temperature(0.0);
    params.set_temperature_inc(0.2);

    // === Skip timestamp token generation ===
    // We only extract text, not timestamps -- saves 5-10%.
    params.set_no_timestamps(true);

    // === Single-segment mode (lightweight decoder models only) ===
    // single_segment skips multi-segment seeking logic for speed.
    // Safe for turbo (4 layers) and distil (2 layers) models on short dictation.
    //
    // For full large-v3 models (32 decoder layers), the seeking logic acts as a
    // critical hallucination safeguard: it detects when a segment is likely wrong
    // (via entropy/logprob thresholds) and retries with different parameters.
    // Disabling it removes the only recovery path, causing persistent "Thank you"
    // hallucinations on the full large-v3 and large-v3-q5_0 models.
    params.set_single_segment(is_lightweight);

    // === Anti-hallucination parameters ===
    // suppress_blank: Suppress blank/empty segments at the start of output.
    // Prevents the decoder from emitting whitespace-only tokens.
    params.set_suppress_blank(true);

    // no_speech_thold: Probability threshold for classifying a segment as silence.
    // When the model's no-speech probability exceeds this, the segment text is suppressed.
    // Default is 0.6. For full large-v3 models, use a slightly lower threshold to be
    // more aggressive at filtering hallucinations from silence.
    params.set_no_speech_thold(if is_lightweight { 0.6 } else { 0.5 });

    // entropy_thold: Segments with average token entropy above this are considered
    // low-confidence and trigger temperature fallback or are discarded.
    // Default is 2.4. For full models, use a tighter threshold to catch hallucinations
    // earlier (hallucinated text often has higher entropy than real transcription).
    params.set_entropy_thold(if is_lightweight { 2.4 } else { 2.2 });

    // logprob_thold: Segments with average token log probability below this are
    // considered low-confidence. Default is -1.0. For full models, use a slightly
    // higher (less negative) threshold to reject more uncertain outputs.
    params.set_logprob_thold(if is_lightweight { -1.0 } else { -0.8 });

    // === Cap decoder output tokens ===
    // Prevents hallucination loops that can add seconds of latency.
    params.set_max_tokens(128);

    log::info!(
        "Whisper params: model={} ({}), n_threads={}, audio_ctx={}{} ({:.1}s audio), greedy(best_of=1), \
         temp_inc=0.2, no_timestamps, single_segment={}, suppress_blank=true, \
         no_speech_thold={}, entropy_thold={}, logprob_thold={}, max_tokens=128",
        model_size,
        if is_distil { "distil/2-layer" } else if is_turbo { "turbo/4-layer" } else { "full/32-layer" },
        n_threads,
        audio_ctx,
        if has_coreml { " (CoreML, full window)" } else if !is_lightweight { " (full window, full-decoder)" } else { "" },
        audio_seconds,
        is_lightweight,
        if is_lightweight { 0.6 } else { 0.5 },
        if is_lightweight { 2.4 } else { 2.2 },
        if is_lightweight { -1.0 } else { -0.8 },
    );

    // Set language if specified
    if let Some(lang) = language {
        if lang != "auto" {
            params.set_language(Some(lang));
        }
    }

    // Disable printing to stdout
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(false);

    let inference_start = std::time::Instant::now();
    state
        .full(params, samples)
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
    let inference_elapsed = inference_start.elapsed();

    let num_segments = state.full_n_segments();

    log::info!(
        "Whisper inference complete in {:.2}s, found {} segments",
        inference_elapsed.as_secs_f64(),
        num_segments
    );

    let mut text = String::new();
    for i in 0..num_segments {
        let segment = state
            .get_segment(i)
            .ok_or_else(|| WhisperError::TranscriptionError(format!("Segment {} not found", i)))?;
        let segment_text = segment
            .to_str()
            .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
        log::debug!("Segment {}: '{}'", i, segment_text);
        text.push_str(segment_text);
    }

    let result = text.trim().to_string();

    // === Post-inference hallucination guard ===
    // Even with proper parameters, the full large-v3 model can occasionally produce
    // known hallucination phrases (especially on very short audio). If the result
    // matches a known hallucination pattern AND the audio was short, return empty
    // rather than injecting garbage text into the user's document.
    if is_likely_hallucination(&result) {
        log::warn!(
            "Whisper output '{}' matches known hallucination pattern (model={}, {:.1}s audio), suppressing",
            result,
            model_size,
            audio_seconds
        );
        let total_elapsed = run_start.elapsed();
        log::info!(
            "Whisper transcription complete in {:.2}s -- hallucination suppressed (0 chars)",
            total_elapsed.as_secs_f64()
        );
        // Still pre-create state for next transcription before returning
        drop(state);
        let bg_ctx = Arc::clone(&ctx);
        let bg_model_size = model_size.to_string();
        std::thread::spawn(move || {
            if let Ok(new_state) = bg_ctx.create_state() {
                if let Ok(mut cache) = STATE_CACHE.lock() {
                    *cache = Some(CachedWhisperState {
                        state: new_state,
                        model_size: bg_model_size,
                    });
                }
            }
        });
        return Ok(String::new());
    }

    let total_elapsed = run_start.elapsed();
    log::info!(
        "Whisper transcription complete in {:.2}s -- result: '{}' ({} chars)",
        total_elapsed.as_secs_f64(),
        if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.clone()
        },
        result.len()
    );

    // Drop the used state (contains stale inference data) and pre-create the next
    // one in a background thread so it's ready for the next transcription.
    // This moves the 50-200ms state allocation off the critical path.
    drop(state);
    let bg_ctx = Arc::clone(&ctx);
    let bg_model_size = model_size.to_string();
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        match bg_ctx.create_state() {
            Ok(new_state) => {
                if let Ok(mut cache) = STATE_CACHE.lock() {
                    *cache = Some(CachedWhisperState {
                        state: new_state,
                        model_size: bg_model_size.clone(),
                    });
                    log::info!(
                        "Pre-created WhisperState in background in {:.2}s (model={})",
                        start.elapsed().as_secs_f64(),
                        bg_model_size
                    );
                }
            }
            Err(e) => {
                log::warn!("Background WhisperState pre-creation failed: {}", e);
            }
        }
    });

    Ok(result)
}
