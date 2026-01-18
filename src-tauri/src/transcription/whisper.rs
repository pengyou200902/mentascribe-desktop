use crate::audio::{capture::prepare_for_whisper, AudioData};
use crate::settings::UserSettings;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;
use whisper_rs::{WhisperContext, WhisperContextParameters};

use super::ModelInfo;

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
    get_models_dir().join(format!("ggml-{}.bin", size))
}

pub fn get_available_models() -> Vec<ModelInfo> {
    let models_dir = get_models_dir();

    vec![
        ModelInfo {
            id: "tiny".to_string(),
            name: "Tiny (75MB)".to_string(),
            size_mb: 75,
            downloaded: models_dir.join("ggml-tiny.bin").exists(),
        },
        ModelInfo {
            id: "base".to_string(),
            name: "Base (142MB)".to_string(),
            size_mb: 142,
            downloaded: models_dir.join("ggml-base.bin").exists(),
        },
        ModelInfo {
            id: "small".to_string(),
            name: "Small (466MB)".to_string(),
            size_mb: 466,
            downloaded: models_dir.join("ggml-small.bin").exists(),
        },
        ModelInfo {
            id: "medium".to_string(),
            name: "Medium (1.5GB)".to_string(),
            size_mb: 1500,
            downloaded: models_dir.join("ggml-medium.bin").exists(),
        },
        ModelInfo {
            id: "large".to_string(),
            name: "Large (2.9GB)".to_string(),
            size_mb: 2900,
            downloaded: models_dir.join("ggml-large-v3.bin").exists(),
        },
    ]
}

pub async fn download_model(size: &str) -> Result<(), WhisperError> {
    let models_dir = get_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let model_name = if size == "large" {
        "ggml-large-v3.bin"
    } else {
        &format!("ggml-{}.bin", size)
    };

    let url = format!("{}/{}", MODEL_BASE_URL, model_name);
    let path = models_dir.join(model_name);

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

    let bytes = response
        .bytes()
        .await
        .map_err(|e| WhisperError::DownloadError(e.to_string()))?;

    std::fs::write(&path, bytes)?;

    log::info!("Model downloaded successfully");
    Ok(())
}

pub async fn transcribe(audio: &AudioData, settings: &UserSettings) -> Result<String, WhisperError> {
    let model_size = settings
        .transcription
        .model_size
        .as_deref()
        .unwrap_or("base")
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
