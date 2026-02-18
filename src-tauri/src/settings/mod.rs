use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranscriptionSettings {
    pub provider: Option<String>,      // "whisper-local", "vosk", "cloud"
    pub language: Option<String>,      // "auto", "en", "es", etc.
    pub model_size: Option<String>,    // "tiny", "base", "small", "medium", "large"
    pub cloud_provider: Option<String>, // "aws", "openai", "assemblyai"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanupSettings {
    pub enabled: bool,
    pub provider: Option<String>,      // "openai", "anthropic", "ollama", "custom"
    pub model: Option<String>,
    pub custom_endpoint: Option<String>,
    pub api_key: Option<String>,
    pub remove_filler: bool,
    pub add_punctuation: bool,
    pub format_paragraphs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotkeySettings {
    pub key: Option<String>,           // "F6", "F5", etc.
    pub mode: Option<String>,          // "hold", "toggle"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputSettings {
    pub insert_method: Option<String>, // "type", "paste"
    pub auto_capitalize: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WidgetSettings {
    pub draggable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettings {
    pub transcription: TranscriptionSettings,
    pub cleanup: CleanupSettings,
    pub hotkey: HotkeySettings,
    pub output: OutputSettings,
    #[serde(default)]
    pub widget: WidgetSettings,
}

fn get_settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("settings.json")
}

pub fn load_settings() -> Result<UserSettings, SettingsError> {
    let path = get_settings_path();

    if !path.exists() {
        return Ok(UserSettings::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let settings = serde_json::from_str(&contents)?;
    Ok(settings)
}

pub fn save_settings(settings: &UserSettings) -> Result<(), SettingsError> {
    let path = get_settings_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, contents)?;

    log::info!("Settings saved to {:?}", path);
    Ok(())
}
