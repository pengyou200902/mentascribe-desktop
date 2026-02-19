pub mod whisper;
pub mod cloud;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub downloaded: bool,
    pub coreml_downloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoremlStatus {
    /// CoreML is compiled into this build (macOS only)
    pub compiled: bool,
    /// Current machine supports CoreML
    pub supported: bool,
    /// Running on Apple Silicon (best CoreML performance)
    pub apple_silicon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
}
