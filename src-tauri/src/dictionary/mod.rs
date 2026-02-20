use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;
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

/// In-memory cache of dictionary entries. Loaded from disk once on first access,
/// then only refreshed when mutations (add/update/remove) save back to disk.
/// Uses RwLock so transcription threads can read concurrently without blocking.
static DICTIONARY_CACHE: Lazy<RwLock<Option<Vec<DictionaryEntry>>>> =
    Lazy::new(|| RwLock::new(None));

fn get_dictionary_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("dictionary.json")
}

fn load_dictionary_from_disk() -> Result<DictionaryData, DictionaryError> {
    let path = get_dictionary_path();

    if !path.exists() {
        return Ok(DictionaryData::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

/// Save to disk and update the in-memory cache in one step.
fn save_and_cache(data: &DictionaryData) -> Result<(), DictionaryError> {
    let path = get_dictionary_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, contents)?;

    // Update the in-memory cache
    if let Ok(mut cache) = DICTIONARY_CACHE.write() {
        *cache = Some(data.entries.clone());
    }

    log::info!("Dictionary saved to {:?}", path);
    Ok(())
}

/// Get cached entries, loading from disk on first access.
fn get_cached_entries() -> Result<Vec<DictionaryEntry>, DictionaryError> {
    // Fast path: read lock, check if cached
    if let Ok(cache) = DICTIONARY_CACHE.read() {
        if let Some(ref entries) = *cache {
            return Ok(entries.clone());
        }
    }

    // Cold start: load from disk and populate cache
    let data = load_dictionary_from_disk()?;
    let entries = data.entries;
    if let Ok(mut cache) = DICTIONARY_CACHE.write() {
        *cache = Some(entries.clone());
    }
    Ok(entries)
}

pub fn get_dictionary() -> Result<Vec<DictionaryEntry>, DictionaryError> {
    get_cached_entries()
}

pub fn add_entry(phrase: String, replacement: String) -> Result<DictionaryEntry, DictionaryError> {
    let mut data = load_dictionary_from_disk()?;

    let entry = DictionaryEntry {
        id: Uuid::new_v4().to_string(),
        phrase,
        replacement,
        enabled: true,
        synced: false,
    };

    data.entries.push(entry.clone());
    save_and_cache(&data)?;

    Ok(entry)
}

pub fn update_entry(
    id: String,
    phrase: String,
    replacement: String,
    enabled: bool,
) -> Result<DictionaryEntry, DictionaryError> {
    let mut data = load_dictionary_from_disk()?;

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
    save_and_cache(&data)?;

    Ok(updated)
}

pub fn remove_entry(id: String) -> Result<bool, DictionaryError> {
    let mut data = load_dictionary_from_disk()?;
    let original_len = data.entries.len();
    data.entries.retain(|e| e.id != id);

    if data.entries.len() < original_len {
        save_and_cache(&data)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_enabled_entries() -> Result<Vec<DictionaryEntry>, DictionaryError> {
    let entries = get_cached_entries()?;
    Ok(entries.into_iter().filter(|e| e.enabled).collect())
}

/// Get vocabulary words (custom word entries where phrase == replacement) for Whisper's
/// initial_prompt. These bias the decoder toward recognizing specific names/terms.
pub fn get_vocabulary_prompt() -> Option<String> {
    let entries = get_cached_entries().ok()?;
    let words: Vec<&str> = entries
        .iter()
        .filter(|e| e.enabled && e.phrase == e.replacement)
        .map(|e| e.phrase.as_str())
        .collect();

    if words.is_empty() {
        None
    } else {
        Some(words.join(", "))
    }
}

/// Apply dictionary replacements to text (case-insensitive word boundary matching).
/// Only applies auto-correct entries (phrase != replacement). Vocabulary entries
/// are handled upstream via Whisper's initial_prompt.
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
    let mut data = load_dictionary_from_disk()?;

    for entry in data.entries.iter_mut() {
        if ids.contains(&entry.id) {
            entry.synced = true;
        }
    }

    save_and_cache(&data)?;
    Ok(())
}
