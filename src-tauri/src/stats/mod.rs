use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub date: String,
    pub transcriptions: u64,
    pub words: u64,
    pub audio_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStats {
    pub total_transcriptions: u64,
    pub total_words: u64,
    pub total_audio_seconds: f64,
    pub streak_days: u32,
    pub last_used_date: Option<String>,
    pub daily_history: Vec<DailyStats>,
}

impl Default for LocalStats {
    fn default() -> Self {
        Self {
            total_transcriptions: 0,
            total_words: 0,
            total_audio_seconds: 0.0,
            streak_days: 0,
            last_used_date: None,
            daily_history: Vec::new(),
        }
    }
}

fn get_stats_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("mentascribe").join("stats.json")
}

pub fn load_stats() -> Result<LocalStats, StatsError> {
    let path = get_stats_path();

    if !path.exists() {
        return Ok(LocalStats::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let stats = serde_json::from_str(&contents)?;
    Ok(stats)
}

pub fn save_stats(stats: &LocalStats) -> Result<(), StatsError> {
    let path = get_stats_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(stats)?;
    std::fs::write(&path, contents)?;

    log::info!("Stats saved to {:?}", path);
    Ok(())
}

pub fn record_transcription(word_count: u32, duration_ms: u32) -> Result<LocalStats, StatsError> {
    let mut stats = load_stats()?;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let audio_seconds = duration_ms as f64 / 1000.0;

    // Update totals
    stats.total_transcriptions += 1;
    stats.total_words += word_count as u64;
    stats.total_audio_seconds += audio_seconds;

    // Update streak
    if let Some(ref last_date) = stats.last_used_date {
        if last_date == &today {
            // Same day, streak unchanged
        } else if is_yesterday(last_date, &today) {
            // Consecutive day, increment streak
            stats.streak_days += 1;
        } else {
            // Streak broken, reset to 1
            stats.streak_days = 1;
        }
    } else {
        // First usage, start streak
        stats.streak_days = 1;
    }
    stats.last_used_date = Some(today.clone());

    // Update daily history
    if let Some(daily) = stats.daily_history.iter_mut().find(|d| d.date == today) {
        daily.transcriptions += 1;
        daily.words += word_count as u64;
        daily.audio_seconds += audio_seconds;
    } else {
        stats.daily_history.push(DailyStats {
            date: today,
            transcriptions: 1,
            words: word_count as u64,
            audio_seconds,
        });
    }

    // Keep only last 30 days
    if stats.daily_history.len() > 30 {
        stats.daily_history.sort_by(|a, b| b.date.cmp(&a.date));
        stats.daily_history.truncate(30);
    }

    save_stats(&stats)?;
    Ok(stats)
}

fn is_yesterday(last_date: &str, today: &str) -> bool {
    if let (Ok(last), Ok(current)) = (
        NaiveDate::parse_from_str(last_date, "%Y-%m-%d"),
        NaiveDate::parse_from_str(today, "%Y-%m-%d"),
    ) {
        let diff = current.signed_duration_since(last).num_days();
        diff == 1
    } else {
        false
    }
}

pub fn get_stats() -> Result<LocalStats, StatsError> {
    let mut stats = load_stats()?;

    // Update streak if needed (check for broken streak)
    if let Some(ref last_date) = stats.last_used_date {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if last_date != &today && !is_yesterday(last_date, &today) {
            // Streak is broken but not yet reset
            stats.streak_days = 0;
        }
    }

    Ok(stats)
}
