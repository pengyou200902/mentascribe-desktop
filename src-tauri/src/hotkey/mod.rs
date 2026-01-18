use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Failed to register hotkey: {0}")]
    RegisterError(String),
    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

/// Parse a key name string to a Code enum
fn parse_key_code(key: &str) -> Result<Code, HotkeyError> {
    match key.to_uppercase().as_str() {
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),
        _ => Err(HotkeyError::UnknownKey(key.to_string())),
    }
}

/// Setup global hotkey with a configurable key
pub fn setup_hotkey(app: AppHandle, key_name: Option<&str>) -> Result<(), HotkeyError> {
    let key = key_name.unwrap_or("F6").to_string(); // Convert to owned String
    let code = parse_key_code(&key)?;
    let shortcut = Shortcut::new(Some(Modifiers::empty()), code);

    let key_for_closure = key.clone(); // Clone for use in closure
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            match event.state {
                ShortcutState::Pressed => {
                    log::info!("{} pressed", key_for_closure);
                    _app.emit("hotkey-pressed", ()).ok();
                }
                ShortcutState::Released => {
                    log::info!("{} released", key_for_closure);
                    _app.emit("hotkey-released", ()).ok();
                }
            }
        })
        .map_err(|e| HotkeyError::RegisterError(e.to_string()))?;

    log::info!("Global hotkey registered: {}", key);
    Ok(())
}

/// Unregister all hotkeys (for re-registration when settings change)
pub fn unregister_all(app: &AppHandle) -> Result<(), HotkeyError> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| HotkeyError::RegisterError(e.to_string()))?;
    log::info!("All hotkeys unregistered");
    Ok(())
}
