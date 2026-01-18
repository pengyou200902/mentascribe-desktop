//! Cloud STT fallback
//!
//! Used when local transcription is unavailable or user prefers cloud processing.

use crate::audio::AudioData;
use crate::settings::UserSettings;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CloudError {
    #[error("No cloud provider configured")]
    NoProvider,
    #[error("API request failed: {0}")]
    RequestError(String),
    #[error("API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTranscriptionRequest {
    pub audio: Vec<u8>,
    pub language: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTranscriptionResponse {
    pub text: String,
    pub language: Option<String>,
}

/// Transcribe audio using cloud STT service
pub async fn transcribe(
    audio: &AudioData,
    settings: &UserSettings,
) -> Result<String, CloudError> {
    let provider = settings
        .transcription
        .cloud_provider
        .as_ref()
        .ok_or(CloudError::NoProvider)?;

    match provider.as_str() {
        "openai" => transcribe_openai(audio, settings).await,
        "aws" => transcribe_aws(audio, settings).await,
        "assemblyai" => transcribe_assemblyai(audio, settings).await,
        _ => Err(CloudError::NoProvider),
    }
}

async fn transcribe_openai(
    audio: &AudioData,
    _settings: &UserSettings,
) -> Result<String, CloudError> {
    // Convert audio to WAV format for OpenAI API
    let _wav_data = audio_to_wav(audio)?;

    // TODO: Implement OpenAI Whisper API call
    // This requires multipart form upload with the audio file

    log::warn!("OpenAI cloud transcription not yet implemented");
    Err(CloudError::ApiError(
        "OpenAI cloud transcription not yet implemented".to_string(),
    ))
}

async fn transcribe_aws(
    _audio: &AudioData,
    _settings: &UserSettings,
) -> Result<String, CloudError> {
    // TODO: Implement AWS Transcribe
    log::warn!("AWS Transcribe not yet implemented");
    Err(CloudError::ApiError(
        "AWS Transcribe not yet implemented".to_string(),
    ))
}

async fn transcribe_assemblyai(
    _audio: &AudioData,
    _settings: &UserSettings,
) -> Result<String, CloudError> {
    // TODO: Implement AssemblyAI
    log::warn!("AssemblyAI not yet implemented");
    Err(CloudError::ApiError(
        "AssemblyAI not yet implemented".to_string(),
    ))
}

/// Convert audio samples to WAV format
fn audio_to_wav(audio: &AudioData) -> Result<Vec<u8>, CloudError> {
    use std::io::Cursor;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, spec)
        .map_err(|e| CloudError::RequestError(e.to_string()))?;

    // Convert f32 samples to i16
    for sample in &audio.samples {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer
            .write_sample(sample_i16)
            .map_err(|e| CloudError::RequestError(e.to_string()))?;
    }

    writer
        .finalize()
        .map_err(|e| CloudError::RequestError(e.to_string()))?;

    Ok(cursor.into_inner())
}
