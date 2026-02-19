use crate::audio::{capture::prepare_for_whisper, AudioData};
use crate::settings::UserSettings;
use once_cell::sync::Lazy;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;
use whisper_rs::{WhisperContext, WhisperContextParameters};

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

fn get_models_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".mentascribe")
        .join("models")
}

fn get_model_path(size: &str) -> PathBuf {
    let filename = if size == "large" {
        "ggml-large-v3.bin".to_string()
    } else {
        format!("ggml-{}.bin", size)
    };
    get_models_dir().join(filename)
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
        _ => 0,
    }
}

/// Approximate CoreML encoder zip download size in bytes for a given model size.
/// Used as a fallback when Content-Length is absent (chunked transfer encoding).
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
fn coreml_size_mb(size: &str) -> u32 {
    match size {
        "tiny" => 42,
        "base" => 78,
        "small" => 244,
        "medium" => 776,
        "large" => 1550,
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
            name: "Tiny (75MB)".to_string(),
            size_mb: 75,
            downloaded: models_dir.join("ggml-tiny.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("tiny"),
            coreml_size_mb: coreml_size_mb("tiny"),
        },
        ModelInfo {
            id: "base".to_string(),
            name: "Base (142MB)".to_string(),
            size_mb: 142,
            downloaded: models_dir.join("ggml-base.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("base"),
            coreml_size_mb: coreml_size_mb("base"),
        },
        ModelInfo {
            id: "small".to_string(),
            name: "Small (466MB)".to_string(),
            size_mb: 466,
            downloaded: models_dir.join("ggml-small.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("small"),
            coreml_size_mb: coreml_size_mb("small"),
        },
        ModelInfo {
            id: "medium".to_string(),
            name: "Medium (1.5GB)".to_string(),
            size_mb: 1500,
            downloaded: models_dir.join("ggml-medium.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("medium"),
            coreml_size_mb: coreml_size_mb("medium"),
        },
        ModelInfo {
            id: "large".to_string(),
            name: "Large (2.9GB)".to_string(),
            size_mb: 2900,
            downloaded: models_dir.join("ggml-large-v3.bin").exists(),
            coreml_downloaded: is_coreml_downloaded("large"),
            coreml_size_mb: coreml_size_mb("large"),
        },
    ]
}

pub async fn download_model(
    size: &str,
    on_progress: impl Fn(u8),
) -> Result<(), WhisperError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let model_name = if size == "large" {
        "ggml-large-v3.bin".to_string()
    } else {
        format!("ggml-{}.bin", size)
    };

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

pub async fn transcribe(audio: &AudioData, settings: &UserSettings) -> Result<String, WhisperError> {
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

    // Run transcription in blocking task to not block async runtime
    let path = model_path.clone();
    let language = settings.transcription.language.clone();
    let size = model_size.clone();

    let result = tokio::task::spawn_blocking(move || {
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

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

        cache.context = Some(ctx);
        cache.model_size = model_size.to_string();
        cache.model_path = model_path.clone();

        log::info!("Whisper model loaded and cached");
    } else {
        log::info!("Using cached Whisper model: {}", model_size);
    }

    // Use the cached context
    let ctx = cache.context.as_ref().unwrap();

    let mut state = ctx
        .create_state()
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    // Use available performance cores for parallel inference
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4)
        .min(8); // Cap at 8 — diminishing returns beyond that
    params.set_n_threads(n_threads);

    log::info!("Whisper params: n_threads={}, greedy(best_of=1)", n_threads);

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

    state
        .full(params, samples)
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

    let num_segments = state
        .full_n_segments()
        .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

    log::info!("Whisper found {} segments", num_segments);

    let mut text = String::new();
    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
        log::debug!("Segment {}: '{}'", i, segment);
        text.push_str(&segment);
    }

    let result = text.trim().to_string();
    log::info!(
        "Whisper transcription result: '{}' ({} chars)",
        if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.clone()
        },
        result.len()
    );

    Ok(result)
}
