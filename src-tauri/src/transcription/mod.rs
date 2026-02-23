pub mod whisper;
pub mod cloud;
#[cfg(feature = "voxtral")]
pub mod voxtral_ffi;
#[cfg(feature = "voxtral")]
pub mod voxtral;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_mb: u32,
    pub downloaded: bool,
    pub coreml_downloaded: bool,
    pub coreml_size_mb: u32,
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
pub struct MetalStatus {
    /// Metal GPU feature is compiled into this build (macOS only)
    pub compiled: bool,
    /// Current platform supports Metal GPU acceleration
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
}

/// Status of the Voxtral engine. When the feature is disabled, returns a stub
/// with compiled=false.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxtralStatus {
    pub compiled: bool,
    pub metal: bool,
    pub model_downloaded: bool,
    pub model_loaded: bool,
}

impl Default for VoxtralStatus {
    fn default() -> Self {
        Self {
            compiled: false,
            metal: false,
            model_downloaded: false,
            model_loaded: false,
        }
    }
}
