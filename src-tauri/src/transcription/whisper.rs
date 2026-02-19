use crate::audio::{capture::prepare_for_whisper, AudioData};
use crate::settings::UserSettings;
use once_cell::sync::Lazy;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;
use whisper_rs::{WhisperContext, WhisperContextParameters, WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

use super::{CoremlStatus, ModelInfo};

// Cache for the Whisper model context to avoid reloading on every transcription
struct ModelCache {
    context: Option<WhisperContext>,
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
    ]
}

pub async fn download_model(
    size: &str,
    on_progress: impl Fn(u8),
) -> Result<(), WhisperError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let model_name = get_model_filename(size);
    let url = format!("{}/{}", MODEL_BASE_URL, model_name);
    let path = models_dir.join(&model_name);

    log::info!("Downloading model from {} to {:?}", url, path);

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

    // Create VAD context (small model, fast to load)
    let mut ctx_params = WhisperVadContextParams::new();
    ctx_params.set_n_threads(2);

    let mut vad_ctx = match WhisperVadContext::new(vad_path.to_str().unwrap(), ctx_params) {
        Ok(ctx) => ctx,
        Err(e) => {
            log::warn!("Failed to load VAD model: {}, skipping pre-filtering", e);
            return samples.to_vec();
        }
    };

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

    // Call create_state() once to trigger CoreML first-run compilation.
    // This can take 5-30s on first run for a device, but is fast on subsequent runs.
    // The state is discarded — we only want the side effect of CoreML compilation.
    let state_start = std::time::Instant::now();
    let _state = ctx
        .create_state()
        .map_err(|e| {
            WhisperError::TranscriptionError(format!(
                "Failed to create state during preload: {}",
                e
            ))
        })?;
    let state_elapsed = state_start.elapsed();
    log::info!(
        "preload_model: state created (CoreML warmup) in {:.2}s",
        state_elapsed.as_secs_f64()
    );
    drop(_state);

    // Store in cache
    cache.context = Some(ctx);
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

pub async fn transcribe(audio: AudioData, settings: &UserSettings) -> Result<String, WhisperError> {
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

    // Run VAD + transcription in blocking task to not block async runtime.
    // Both VAD (model load + neural inference) and Whisper are heavy FFI work.
    let path = model_path.clone();
    let language = settings.transcription.language.clone();
    let size = model_size.clone();

    let result = tokio::task::spawn_blocking(move || {
        let samples = vad_filter_speech(&samples);
        run_whisper(&path, &size, &samples, language.as_deref())
    })
    .await
    .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

    result
}

fn run_whisper(
    model_path: &PathBuf,
    model_size: &str,
    samples: &[f32],
    language: Option<&str>,
) -> Result<String, WhisperError> {
    use whisper_rs::{FullParams, SamplingStrategy};

    let run_start = std::time::Instant::now();

    // Get or create the cached context
    let mut cache = MODEL_CACHE
        .lock()
        .map_err(|e| WhisperError::TranscriptionError(format!("Cache lock error: {}", e)))?;

    // Check if we need to reload the model
    if cache.context.is_none() || cache.model_size != model_size || cache.model_path != *model_path
    {
        log::info!(
            "Loading Whisper model: {} from {:?}",
            model_size,
            model_path
        );

        let load_start = std::time::Instant::now();

        let mut ctx_params = WhisperContextParameters::default();
        ctx_params.flash_attn(true);

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            ctx_params,
        )
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

        cache.context = Some(ctx);
        cache.model_size = model_size.to_string();
        cache.model_path = model_path.clone();

        log::info!(
            "Whisper model loaded and cached in {:.2}s",
            load_start.elapsed().as_secs_f64()
        );
    } else {
        log::info!("Using cached Whisper model: {}", model_size);
    }

    // Use the cached context
    let ctx = cache.context.as_ref().unwrap();

    let state_start = std::time::Instant::now();
    let mut state = ctx
        .create_state()
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
    log::info!(
        "Whisper state created in {:.2}s",
        state_start.elapsed().as_secs_f64()
    );

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Use available performance cores for parallel inference
    // With CoreML handling the encoder on ANE, CPU threads mainly affect the decoder.
    // 4-6 threads often outperform 8 on Apple Silicon due to reduced contention.
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4)
        .min(6);
    params.set_n_threads(n_threads);

    // === Dynamic audio_ctx: limit encoder window to actual audio length ===
    // Whisper always processes a 30s window (1500 mel frames). For short dictation,
    // most of that is zero-padded silence. Setting audio_ctx proportionally skips it.
    // whisper.cpp rejects audio_ctx > 1500 (returns -5), so clamp to that max.
    // whisper.cpp internally pads to multiples of 256 via GGML_PAD.
    let audio_seconds = samples.len() as f32 / 16000.0;
    let audio_ctx = ((audio_seconds / 30.0) * 1500.0).ceil() as i32 + 128;
    let audio_ctx = ((audio_ctx + 255) / 256) * 256; // round up to multiple of 256
    let audio_ctx = audio_ctx.clamp(768, 1500); // min 768 (quality), max 1500 (whisper limit)
    params.set_audio_ctx(audio_ctx);

    // === Disable temperature fallback ===
    // Prevents retry mechanism with increasing temperature on failure.
    // Default retries up to 4-5 times, causing tail-latency spikes.
    // No quality impact for clean microphone input.
    params.set_temperature(0.0);
    params.set_temperature_inc(0.0);

    // === Skip timestamp token generation ===
    // We only extract text, not timestamps — saves 5-10%.
    params.set_no_timestamps(true);

    // === Force single-segment output ===
    // Skips multi-segment seeking logic. Ideal for short dictation utterances.
    params.set_single_segment(true);

    // === Cap decoder output tokens ===
    // Prevents hallucination loops that can add seconds of latency.
    params.set_max_tokens(128);

    log::info!(
        "Whisper params: n_threads={}, audio_ctx={} ({:.1}s audio), greedy(best_of=1), no_timestamps, single_segment, max_tokens=128",
        n_threads, audio_ctx, audio_seconds
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
    let total_elapsed = run_start.elapsed();
    log::info!(
        "Whisper transcription complete in {:.2}s — result: '{}' ({} chars)",
        total_elapsed.as_secs_f64(),
        if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.clone()
        },
        result.len()
    );

    Ok(result)
}
