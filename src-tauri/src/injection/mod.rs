use crate::settings::UserSettings;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InjectionError {
    #[error("Text injection failed: {0}")]
    Failed(String),
    #[error("Platform not supported")]
    UnsupportedPlatform,
    #[error("Accessibility permission required. Go to System Settings > Privacy & Security > Accessibility and enable MentaScribe")]
    AccessibilityPermissionRequired,
}

/// Check if we have Accessibility permissions on macOS
#[cfg(target_os = "macos")]
fn check_accessibility_permissions() -> bool {
    use std::process::Command;

    // Use AppleScript to check if we're trusted
    // AXIsProcessTrusted() would be better but requires linking to ApplicationServices
    let output = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to return (exists process \"Finder\")"])
        .output();

    match output {
        Ok(out) => {
            // If we can query System Events, we likely have accessibility permissions
            let success = out.status.success();
            eprintln!("[inject] Accessibility check: {}", if success { "granted" } else { "denied" });
            success
        }
        Err(e) => {
            eprintln!("[inject] Accessibility check failed: {}", e);
            false
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility_permissions() -> bool {
    true // No special permissions needed on other platforms
}

/// Inject text into the currently focused application
pub fn inject_text(text: &str, settings: &UserSettings) -> Result<(), InjectionError> {
    let method = settings
        .output
        .insert_method
        .as_deref()
        .unwrap_or("paste");

    eprintln!(
        "[inject] inject_text called: method={}, text_len={}, text='{}'",
        method,
        text.len(),
        if text.len() > 80 { &text[..80] } else { text }
    );

    // Skip empty or whitespace-only text
    if text.trim().is_empty() {
        eprintln!("[inject] Skipping empty text");
        return Ok(());
    }

    // Skip [BLANK_AUDIO] which Whisper outputs when no speech detected
    if text.contains("[BLANK_AUDIO]") || text.contains("[BLANK AUDIO]") {
        eprintln!("[inject] Skipping BLANK_AUDIO marker");
        return Ok(());
    }

    // Check accessibility permissions before attempting injection
    #[cfg(target_os = "macos")]
    if !check_accessibility_permissions() {
        eprintln!("[inject] ERROR: Accessibility permissions not granted");
        return Err(InjectionError::AccessibilityPermissionRequired);
    }

    // Delay to allow focus to return to the target application
    // after the dictation window processes the recording
    eprintln!("[inject] Waiting for focus to return to target application...");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let result = match method {
        "paste" => inject_via_paste(text),
        _ => inject_via_typing(text),
    };

    match &result {
        Ok(_) => eprintln!("[inject] Text injection succeeded"),
        Err(e) => eprintln!("[inject] ERROR: Text injection failed: {}", e),
    }

    result
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

/// Inject text via clipboard paste (cross-platform using arboard)
fn inject_via_paste(text: &str) -> Result<(), InjectionError> {
    use arboard::Clipboard;

    // Copy text to clipboard using arboard (cross-platform)
    let mut clipboard = Clipboard::new()
        .map_err(|e| InjectionError::Failed(format!("Failed to access clipboard: {}", e)))?;

    eprintln!("[inject] Setting clipboard text...");
    clipboard
        .set_text(text)
        .map_err(|e| InjectionError::Failed(format!("Failed to set clipboard text: {}", e)))?;
    eprintln!("[inject] Clipboard text set successfully");

    // Delay to ensure clipboard is ready
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Simulate Cmd+V / Ctrl+V
    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to send Cmd+V - works better with Apple apps like Notes
        use std::process::Command;
        eprintln!("[inject] Sending Cmd+V via AppleScript...");
        let output = Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
            .output()
            .map_err(|e| InjectionError::Failed(format!("Failed to run AppleScript: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("[inject] AppleScript error: {}", stderr);
            return Err(InjectionError::Failed(format!("AppleScript failed: {}", stderr)));
        }
        eprintln!("[inject] Cmd+V sent via AppleScript");
    }

    #[cfg(not(target_os = "macos"))]
    {
        use enigo::{Direction, Enigo, Key, Keyboard, Settings};
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| InjectionError::Failed(e.to_string()))?;

        eprintln!("[inject] Sending Ctrl+V...");
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;
        eprintln!("[inject] Ctrl+V sent");
    }

    // Longer delay to ensure paste completes before clearing clipboard
    // The target application needs time to process the paste command
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Clear clipboard after paste to avoid leaving transcribed text in clipboard
    if let Err(e) = clipboard.clear() {
        eprintln!("[inject] Warning: Failed to clear clipboard: {}", e);
        // Don't fail the operation if clipboard clear fails
    } else {
        eprintln!("[inject] Clipboard cleared after paste");
    }

    log::info!("Text injected via paste: {} chars", text.len());
    Ok(())
}
