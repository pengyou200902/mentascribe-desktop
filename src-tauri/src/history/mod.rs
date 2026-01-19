use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: String,
    pub text: String,
    pub word_count: u32,
    pub duration_ms: u32,
    pub timestamp: String,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HistoryData {
    entries: Vec<TranscriptionEntry>,
}

fn get_history_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("history.json")
}

fn load_history_data() -> Result<HistoryData, HistoryError> {
    let path = get_history_path();

    if !path.exists() {
        return Ok(HistoryData::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let data = serde_json::from_str(&contents)?;
    Ok(data)
}

fn save_history_data(data: &HistoryData) -> Result<(), HistoryError> {
    let path = get_history_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(data)?;
    std::fs::write(&path, contents)?;

    log::info!("History saved to {:?}", path);
    Ok(())
}

pub fn add_entry(text: &str, word_count: u32, duration_ms: u32) -> Result<TranscriptionEntry, HistoryError> {
    let mut data = load_history_data()?;

    let entry = TranscriptionEntry {
        id: Uuid::new_v4().to_string(),
        text: text.to_string(),
        word_count,
        duration_ms,
        timestamp: Local::now().to_rfc3339(),
        synced: false,
    };

    data.entries.insert(0, entry.clone());

    // Keep only last 500 entries
    if data.entries.len() > 500 {
        data.entries.truncate(500);
    }

    save_history_data(&data)?;
    Ok(entry)
}

pub fn get_history(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<TranscriptionEntry>, HistoryError> {
    let data = load_history_data()?;

    let offset = offset.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(50) as usize;

    let entries: Vec<TranscriptionEntry> = data.entries
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(entries)
}

pub fn get_entry(id: &str) -> Result<Option<TranscriptionEntry>, HistoryError> {
    let data = load_history_data()?;
    Ok(data.entries.into_iter().find(|e| e.id == id))
}

pub fn delete_entry(id: &str) -> Result<bool, HistoryError> {
    let mut data = load_history_data()?;
    let original_len = data.entries.len();
    data.entries.retain(|e| e.id != id);

    if data.entries.len() < original_len {
        save_history_data(&data)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn clear_history() -> Result<(), HistoryError> {
    let data = HistoryData::default();
    save_history_data(&data)?;
    Ok(())
}

pub fn get_total_count() -> Result<usize, HistoryError> {
    let data = load_history_data()?;
    Ok(data.entries.len())
}

pub fn mark_synced(ids: &[String]) -> Result<(), HistoryError> {
    let mut data = load_history_data()?;

    for entry in data.entries.iter_mut() {
        if ids.contains(&entry.id) {
            entry.synced = true;
        }
    }

    save_history_data(&data)?;
    Ok(())
}
