use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Failed to register hotkey: {0}")]
    RegisterError(String),
}

pub fn setup_hotkey(app: AppHandle) -> Result<(), HotkeyError> {
    let shortcut = Shortcut::new(Some(Modifiers::empty()), Code::F6);

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            match event.state {
                ShortcutState::Pressed => {
                    log::info!("F6 pressed");
                    _app.emit("hotkey-pressed", ()).ok();
                }
                ShortcutState::Released => {
                    log::info!("F6 released");
                    _app.emit("hotkey-released", ()).ok();
                }
            }
        })
        .map_err(|e| HotkeyError::RegisterError(e.to_string()))?;

    log::info!("Global hotkey registered: F6");
    Ok(())
}
