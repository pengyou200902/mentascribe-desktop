use crate::settings::UserSettings;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InjectionError {
    #[error("Text injection failed: {0}")]
    Failed(String),
    #[error("Platform not supported")]
    UnsupportedPlatform,
}

/// Inject text into the currently focused application
pub fn inject_text(text: &str, settings: &UserSettings) -> Result<(), InjectionError> {
    let method = settings
        .output
        .insert_method
        .as_deref()
        .unwrap_or("type");

    match method {
        "paste" => inject_via_paste(text),
        _ => inject_via_typing(text),
    }
}

/// Inject text by simulating keyboard input
fn inject_via_typing(text: &str) -> Result<(), InjectionError> {
    use enigo::{Enigo, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InjectionError::Failed(e.to_string()))?;

    enigo
        .text(text)
        .map_err(|e| InjectionError::Failed(e.to_string()))?;

    log::info!("Text injected via typing: {} chars", text.len());
    Ok(())
}

/// Inject text via clipboard paste
fn inject_via_paste(text: &str) -> Result<(), InjectionError> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    // Copy text to clipboard using platform-specific method
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| InjectionError::Failed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| InjectionError::Failed(e.to_string()))?;
        }
        child
            .wait()
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use the typing method as fallback
        return inject_via_typing(text);
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| InjectionError::Failed(e.to_string()))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| InjectionError::Failed(e.to_string()))?;
        }
        child
            .wait()
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
    }

    // Simulate Cmd+V / Ctrl+V
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InjectionError::Failed(e.to_string()))?;

    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
    }

    log::info!("Text injected via paste: {} chars", text.len());
    Ok(())
}
