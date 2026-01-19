use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DictionaryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Entry not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub id: String,
    pub phrase: String,
    pub replacement: String,
    pub enabled: bool,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DictionaryData {
    entries: Vec<DictionaryEntry>,
}

fn get_dictionary_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("dictionary.json")
}

fn load_dictionary_data() -> Result<DictionaryData, DictionaryError> {
    let path = get_dictionary_path();

    if !path.exists() {
        return Ok(DictionaryData::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

fn save_dictionary_data(data: &DictionaryData) -> Result<(), DictionaryError> {
    let path = get_dictionary_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, contents)?;

    log::info!("Dictionary saved to {:?}", path);
    Ok(())
}

pub fn get_dictionary() -> Result<Vec<DictionaryEntry>, DictionaryError> {
    let data = load_dictionary_data()?;
    Ok(data.entries)
}

pub fn add_entry(phrase: String, replacement: String) -> Result<DictionaryEntry, DictionaryError> {
    let mut data = load_dictionary_data()?;

    let entry = DictionaryEntry {
        id: Uuid::new_v4().to_string(),
        phrase,
        replacement,
        enabled: true,
        synced: false,
    };

    data.entries.push(entry.clone());
    save_dictionary_data(&data)?;

    Ok(entry)
}

pub fn update_entry(
    id: String,
    phrase: String,
    replacement: String,
    enabled: bool,
) -> Result<DictionaryEntry, DictionaryError> {
    let mut data = load_dictionary_data()?;

    let entry = data
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| DictionaryError::NotFound(id.clone()))?;

    entry.phrase = phrase;
    entry.replacement = replacement;
    entry.enabled = enabled;
    entry.synced = false;

    let updated = entry.clone();
    save_dictionary_data(&data)?;

    Ok(updated)
}

pub fn remove_entry(id: String) -> Result<bool, DictionaryError> {
    let mut data = load_dictionary_data()?;
    let original_len = data.entries.len();
    data.entries.retain(|e| e.id != id);

    if data.entries.len() < original_len {
        save_dictionary_data(&data)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_enabled_entries() -> Result<Vec<DictionaryEntry>, DictionaryError> {
    let data = load_dictionary_data()?;
    Ok(data.entries.into_iter().filter(|e| e.enabled).collect())
}

/// Apply dictionary replacements to text (case-insensitive word boundary matching)
pub fn apply_replacements(text: &str) -> Result<String, DictionaryError> {
    let entries = get_enabled_entries()?;

    if entries.is_empty() {
        return Ok(text.to_string());
    }

    let mut result = text.to_string();

    for entry in entries {
        // Case-insensitive replacement with word boundaries
        let pattern = format!(r"(?i)\b{}\b", regex::escape(&entry.phrase));
        if let Ok(re) = regex::Regex::new(&pattern) {
            result = re.replace_all(&result, entry.replacement.as_str()).to_string();
        }
    }

    Ok(result)
}

pub fn mark_synced(ids: &[String]) -> Result<(), DictionaryError> {
    let mut data = load_dictionary_data()?;

    for entry in data.entries.iter_mut() {
        if ids.contains(&entry.id) {
            entry.synced = true;
        }
    }

    save_dictionary_data(&data)?;
    Ok(())
}
