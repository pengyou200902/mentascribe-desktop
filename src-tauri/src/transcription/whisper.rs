use crate::audio::{capture::prepare_for_whisper, AudioData};
use crate::settings::UserSettings;
use std::path::PathBuf;
use thiserror::Error;

use super::ModelInfo;

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
    let model_size = settings.transcription.model_size.as_deref().unwrap_or("base");
    let model_path = get_model_path(model_size);

    if !model_path.exists() {
        return Err(WhisperError::ModelNotFound(model_size.to_string()));
    }

    // Prepare audio for Whisper (16kHz mono)
    let samples = prepare_for_whisper(audio);

    // Run transcription in blocking task to not block async runtime
    let path = model_path.clone();
    let language = settings.transcription.language.clone();

    let result = tokio::task::spawn_blocking(move || {
        run_whisper(&path, &samples, language.as_deref())
    })
    .await
    .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

    result
}

fn run_whisper(
    model_path: &PathBuf,
    samples: &[f32],
    language: Option<&str>,
) -> Result<String, WhisperError> {
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default(),
    )
    .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;

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

    let mut text = String::new();
    for i in 0..num_segments {
        let segment = state
            .full_get_segment_text(i)
            .map_err(|e| WhisperError::TranscriptionError(e.to_string()))?;
        text.push_str(&segment);
    }

    Ok(text.trim().to_string())
}
