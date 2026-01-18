mod audio;
mod transcription;
mod hotkey;
mod injection;
mod settings;
mod api;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use std::sync::Mutex;

pub struct AppState {
    pub is_recording: Mutex<bool>,
    pub settings: Mutex<settings::UserSettings>,
}

#[tauri::command]
fn start_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
    if *is_recording {
        return Err("Already recording".to_string());
    }
    *is_recording = true;

    // Start audio capture
    audio::capture::start_capture().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Get recording state and settings before any await
    let was_recording = {
        let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
        if !*is_recording {
            return Err("Not recording".to_string());
        }
        *is_recording = false;
        true
    };

    if !was_recording {
        return Err("Not recording".to_string());
    }

    // Stop audio capture and get audio data
    let audio_data = audio::capture::stop_capture().map_err(|e| e.to_string())?;

    // Emit processing event
    app.emit("transcription-processing", ()).ok();

    // Clone settings for use in async block
    let settings = {
        let s = state.settings.lock().map_err(|e| e.to_string())?;
        s.clone()
    };

    // Transcribe audio
    let text = transcription::whisper::transcribe(&audio_data, &settings)
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion event
    app.emit("transcription-complete", &text).ok();

    Ok(text)
}

#[tauri::command]
fn inject_text(text: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    injection::inject_text(&text, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<settings::UserSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
fn update_settings(
    new_settings: settings::UserSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings.clone();

    // Persist settings
    settings::save_settings(&new_settings).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn login(email: String, password: String) -> Result<api::AuthToken, String> {
    api::client::login(&email, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(size: String) -> Result<(), String> {
    transcription::whisper::download_model(&size)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_available_models() -> Vec<transcription::ModelInfo> {
    transcription::whisper::get_available_models()
}

fn open_settings_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html#settings".into()))
            .title("MentaScribe Settings")
            .inner_size(480.0, 640.0)
            .resizable(true)
            .build()
            .ok();
    }
}

fn open_history_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("history") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        WebviewWindowBuilder::new(app, "history", WebviewUrl::App("index.html#history".into()))
            .title("MentaScribe History")
            .inner_size(480.0, 500.0)
            .resizable(true)
            .build()
            .ok();
    }
}

fn toggle_dictation_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("dictation") {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            window.show().ok();
        }
    }
}

pub fn run() {
    env_logger::init();

    // Load or create default settings
    let settings = settings::load_settings().unwrap_or_default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Initialize global hotkey
            let app_handle = app.handle().clone();
            hotkey::setup_hotkey(app_handle.clone())?;

            // Position dictation window at bottom center, above the dock
            if let Some(window) = app.get_webview_window("dictation") {
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen_size = monitor.size();
                    let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(200, 48));

                    // Center horizontally, position 100px from bottom (above dock)
                    let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                    let y = screen_size.height as i32 - window_size.height as i32 - 100;

                    window.set_position(tauri::PhysicalPosition::new(x, y)).ok();
                    window.show().ok();
                }
            }

            // Build tray menu
            let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let history_item = MenuItem::with_id(app, "history", "History...", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&settings_item, &history_item, &toggle_item, &quit_item],
            )?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "settings" => {
                        open_settings_window(app);
                    }
                    "history" => {
                        open_history_window(app);
                    }
                    "toggle" => {
                        toggle_dictation_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_dictation_window(app);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .manage(AppState {
            is_recording: Mutex::new(false),
            settings: Mutex::new(settings),
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            inject_text,
            get_settings,
            update_settings,
            login,
            download_model,
            get_available_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
