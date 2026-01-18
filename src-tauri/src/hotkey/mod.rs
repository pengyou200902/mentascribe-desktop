use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Failed to create hotkey manager: {0}")]
    ManagerError(String),
    #[error("Failed to register hotkey: {0}")]
    RegisterError(String),
}

static HOTKEY_MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

pub fn setup_hotkey(app: AppHandle) -> Result<(), HotkeyError> {
    let manager = GlobalHotKeyManager::new()
        .map_err(|e| HotkeyError::ManagerError(e.to_string()))?;

    // Default hotkey: F6
    let hotkey = HotKey::new(None, Code::F6);
    let hotkey_id = hotkey.id();

    manager
        .register(hotkey)
        .map_err(|e| HotkeyError::RegisterError(e.to_string()))?;

    HOTKEY_MANAGER.set(manager).ok();

    // Listen for hotkey events
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();

        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == hotkey_id {
                    match event.state {
                        global_hotkey::HotKeyState::Pressed => {
                            log::info!("Hotkey pressed");
                            app.emit("hotkey-pressed", ()).ok();
                        }
                        global_hotkey::HotKeyState::Released => {
                            log::info!("Hotkey released");
                            app.emit("hotkey-released", ()).ok();
                        }
                    }
                }
            }
        }
    });

    log::info!("Global hotkey registered: F6");
    Ok(())
}

/// Update the registered hotkey
#[allow(dead_code)]
pub fn update_hotkey(key: &str) -> Result<(), HotkeyError> {
    let manager = HOTKEY_MANAGER
        .get()
        .ok_or_else(|| HotkeyError::ManagerError("Manager not initialized".to_string()))?;

    // Parse key string to Code
    let code = parse_key_code(key)
        .ok_or_else(|| HotkeyError::RegisterError(format!("Unknown key: {}", key)))?;

    let hotkey = HotKey::new(None, code);

    manager
        .register(hotkey)
        .map_err(|e| HotkeyError::RegisterError(e.to_string()))?;

    log::info!("Hotkey updated to: {}", key);
    Ok(())
}

fn parse_key_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}
