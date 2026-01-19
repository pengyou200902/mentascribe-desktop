mod audio;
mod transcription;
mod hotkey;
mod injection;
mod settings;
mod api;
mod text;
mod stats;
mod history;
mod dictionary;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, LogicalPosition, Manager, WebviewUrl, WebviewWindowBuilder,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Apply macOS-specific window settings to keep the dictation window always on top,
/// including over fullscreen applications.
#[cfg(target_os = "macos")]
fn enforce_always_on_top(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;

    // Use CGWindowLevel constants for maximum visibility
    // kCGScreenSaverWindowLevel = 1000 - highest level that appears above fullscreen apps
    const SCREEN_SAVER_WINDOW_LEVEL: i64 = 1000;

    window.with_webview(|webview| {
        unsafe {
            let ns_window: id = webview.ns_window() as id;
            // Set window level to screen saver level to appear above fullscreen apps
            ns_window.setLevel_(SCREEN_SAVER_WINDOW_LEVEL);

            // Set collection behavior to allow the window to:
            // - Join all spaces (visible on all desktops)
            // - Stay stationary when switching spaces
            // - Appear above fullscreen applications
            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;
            ns_window.setCollectionBehavior_(behavior);
        }
    }).ok();
}

#[cfg(not(target_os = "macos"))]
fn enforce_always_on_top(_window: &tauri::WebviewWindow) {
    // On non-macOS platforms, the alwaysOnTop config setting is sufficient
}

pub struct AppState {
    pub is_recording: Mutex<bool>,
    pub settings: Mutex<settings::UserSettings>,
    pub audio_level_emitter_running: Arc<AtomicBool>,
}

#[tauri::command]
fn start_recording(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    eprintln!("[recording] start_recording called");

    let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
    if *is_recording {
        eprintln!("[recording] WARNING: already recording");
        return Err("Already recording".to_string());
    }
    *is_recording = true;

    // Start audio capture
    eprintln!("[recording] Starting audio capture...");
    if let Err(e) = audio::capture::start_capture() {
        eprintln!("[recording] ERROR: Failed to start audio capture: {}", e);
        // Reset state on failure
        *is_recording = false;
        return Err(e.to_string());
    }
    eprintln!("[recording] Audio capture started successfully");

    // Start audio level emitter
    let running = state.audio_level_emitter_running.clone();
    running.store(true, Ordering::SeqCst);

    let app_clone = app.clone();
    std::thread::spawn(move || {
        let mut frame_count = 0u32;
        while running.load(Ordering::SeqCst) {
            let level = audio::capture::get_current_level();
            app_clone.emit("audio-level", level).ok();

            // Log every 40 frames (~1 second) to avoid spam
            frame_count += 1;
            if frame_count % 40 == 0 {
                log::info!("Emitting audio level: {:.4}", level);
            }

            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        log::info!("Audio level emitter stopped");
    });

    Ok(())
}

#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    eprintln!("[recording] stop_recording called");

    // Stop audio level emitter first
    state.audio_level_emitter_running.store(false, Ordering::SeqCst);

    // Get recording state and settings before any await
    let was_recording = {
        let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
        if !*is_recording {
            eprintln!("[recording] WARNING: not currently recording");
            return Err("Not recording".to_string());
        }
        *is_recording = false;
        true
    };

    if !was_recording {
        return Err("Not recording".to_string());
    }

    // Stop audio capture and get audio data
    eprintln!("[recording] Stopping audio capture...");
    let audio_data = audio::capture::stop_capture().map_err(|e| {
        eprintln!("[recording] ERROR: Failed to stop audio capture: {}", e);
        e.to_string()
    })?;
    eprintln!(
        "[recording] Audio captured: {} samples at {}Hz ({:.2}s)",
        audio_data.samples.len(),
        audio_data.sample_rate,
        audio_data.samples.len() as f32 / audio_data.sample_rate as f32
    );

    // Emit processing event
    app.emit("transcription-processing", ()).ok();

    // Clone settings for use in async block
    let settings = {
        let s = state.settings.lock().map_err(|e| e.to_string())?;
        s.clone()
    };

    // Transcribe audio
    eprintln!("[recording] Starting transcription...");
    let raw_text = transcription::whisper::transcribe(&audio_data, &settings)
        .await
        .map_err(|e| {
            eprintln!("[recording] ERROR: Transcription failed: {}", e);
            e.to_string()
        })?;
    eprintln!(
        "[recording] Transcription complete: '{}' ({} chars)",
        if raw_text.len() > 100 {
            format!("{}...", &raw_text[..100])
        } else {
            raw_text.clone()
        },
        raw_text.len()
    );

    // Apply auto-capitalize if enabled
    let auto_capitalize = settings.output.auto_capitalize.unwrap_or(true);
    let mut text = text::process_text(&raw_text, auto_capitalize);

    // Apply dictionary replacements
    if let Ok(replaced) = dictionary::apply_replacements(&text) {
        text = replaced;
    }

    // Calculate stats for recording
    let word_count = text.split_whitespace().count() as u32;
    let duration_ms = (audio_data.samples.len() as f32 / audio_data.sample_rate as f32 * 1000.0) as u32;

    // Record to local history and stats (fire and forget, don't fail transcription)
    if let Err(e) = history::add_entry(&text, word_count, duration_ms) {
        eprintln!("[recording] WARNING: Failed to save to history: {}", e);
    }
    if let Err(e) = stats::record_transcription(word_count, duration_ms) {
        eprintln!("[recording] WARNING: Failed to record stats: {}", e);
    }

    // Emit completion event
    app.emit("transcription-complete", &text).ok();

    Ok(text)
}

#[tauri::command]
fn inject_text(text: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    injection::inject_text(&text, &settings).map_err(|e| e.to_string())
}

/// Reset recording state - used to recover from stuck states
#[tauri::command]
fn reset_recording_state(state: tauri::State<'_, AppState>) -> Result<(), String> {
    eprintln!("[recording] reset_recording_state called");

    // Reset backend recording flag
    let mut is_recording = state.is_recording.lock().map_err(|e| e.to_string())?;
    *is_recording = false;

    // Stop audio level emitter
    state.audio_level_emitter_running.store(false, Ordering::SeqCst);

    // Reset audio capture state
    audio::capture::reset_state();

    eprintln!("[recording] Recording state reset complete");
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<settings::UserSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
fn update_settings(
    app: tauri::AppHandle,
    new_settings: settings::UserSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let old_hotkey = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.hotkey.key.clone()
    };

    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings.clone();

    // Persist settings
    settings::save_settings(&new_settings).map_err(|e| e.to_string())?;

    // Re-register hotkey if it changed
    if old_hotkey != new_settings.hotkey.key {
        drop(settings); // Release lock before hotkey operations
        hotkey::unregister_all(&app).map_err(|e| e.to_string())?;
        hotkey::setup_hotkey(app, new_settings.hotkey.key.as_deref())
            .map_err(|e| e.to_string())?;
    }

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

// Stats commands
#[tauri::command]
fn get_stats() -> Result<stats::LocalStats, String> {
    stats::get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn record_transcription_stats(word_count: u32, duration_ms: u32) -> Result<stats::LocalStats, String> {
    stats::record_transcription(word_count, duration_ms).map_err(|e| e.to_string())
}

// History commands
#[tauri::command]
fn get_history(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<history::TranscriptionEntry>, String> {
    history::get_history(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history_entry(id: String) -> Result<Option<history::TranscriptionEntry>, String> {
    history::get_entry(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history_entry(id: String) -> Result<bool, String> {
    history::delete_entry(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    history::clear_history().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history_count() -> Result<usize, String> {
    history::get_total_count().map_err(|e| e.to_string())
}

// Dictionary commands
#[tauri::command]
fn get_dictionary() -> Result<Vec<dictionary::DictionaryEntry>, String> {
    dictionary::get_dictionary().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_dictionary_entry(phrase: String, replacement: String) -> Result<dictionary::DictionaryEntry, String> {
    dictionary::add_entry(phrase, replacement).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_dictionary_entry(
    id: String,
    phrase: String,
    replacement: String,
    enabled: bool,
) -> Result<dictionary::DictionaryEntry, String> {
    dictionary::update_entry(id, phrase, replacement, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_dictionary_entry(id: String) -> Result<bool, String> {
    dictionary::remove_entry(id).map_err(|e| e.to_string())
}

/// Constants for dictation window dimensions (logical pixels, as defined in tauri.conf.json)
const DICTATION_WINDOW_WIDTH: f64 = 340.0;
const DICTATION_WINDOW_HEIGHT: f64 = 120.0;
/// Offset from the bottom of the screen to position just above the macOS dock
/// Wispr Flow uses approximately 20px offset for a snug fit above the dock
const DOCK_OFFSET: f64 = 20.0;

/// Calculate the centered position for the dictation window on a given monitor.
/// Uses logical coordinates - Tauri's set_position(LogicalPosition) handles scale factor conversion.
///
/// The key insight is that:
/// - Monitor.position() and Monitor.size() return PHYSICAL coordinates
/// - We convert them to LOGICAL by dividing by scale_factor
/// - Then we calculate centering in logical space
/// - Finally we use LogicalPosition with set_position() and Tauri handles the rest
fn calculate_dictation_position(monitor: &tauri::window::Monitor) -> LogicalPosition<f64> {
    let scale_factor = monitor.scale_factor();
    let screen_pos = monitor.position();
    let screen_size = monitor.size();

    // Convert monitor's physical coordinates to logical coordinates
    let logical_screen_x = screen_pos.x as f64 / scale_factor;
    let logical_screen_y = screen_pos.y as f64 / scale_factor;
    let logical_screen_width = screen_size.width as f64 / scale_factor;
    let logical_screen_height = screen_size.height as f64 / scale_factor;

    // Calculate centered position in logical coordinates
    let x = logical_screen_x + (logical_screen_width - DICTATION_WINDOW_WIDTH) / 2.0;
    let y = logical_screen_y + logical_screen_height - DICTATION_WINDOW_HEIGHT - DOCK_OFFSET;

    LogicalPosition::new(x, y)
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

fn open_dashboard_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("dashboard") {
        window.show().ok();
        window.set_focus().ok();
    } else {
        WebviewWindowBuilder::new(app, "dashboard", WebviewUrl::App("index.html#dashboard".into()))
            .title("MentaScribe")
            .inner_size(800.0, 600.0)
            .min_inner_size(640.0, 480.0)
            .resizable(true)
            .build()
            .ok();
    }
}

/// Reposition dictation window to the monitor where the mouse currently is
/// Returns true if window was moved to a different monitor
#[tauri::command]
fn reposition_to_mouse_monitor(app: tauri::AppHandle) -> Result<bool, String> {
    let window = app.get_webview_window("dictation")
        .ok_or_else(|| "Dictation window not found".to_string())?;

    // Skip if window is not visible
    if !window.is_visible().unwrap_or(false) {
        return Ok(false);
    }

    // Get current cursor position (returns physical coordinates)
    let cursor_pos = window.cursor_position()
        .map_err(|e| format!("Failed to get cursor position: {}", e))?;

    // Find the monitor containing the cursor
    let monitors = window.available_monitors()
        .map_err(|e| format!("Failed to get monitors: {}", e))?;

    let target_monitor = monitors.into_iter().find(|m| {
        let pos = m.position();
        let size = m.size();
        let cursor_x = cursor_pos.x as i32;
        let cursor_y = cursor_pos.y as i32;
        cursor_x >= pos.x && cursor_x < pos.x + size.width as i32 &&
        cursor_y >= pos.y && cursor_y < pos.y + size.height as i32
    });

    let monitor = target_monitor
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "No monitor found".to_string())?;

    let screen_pos = monitor.position();
    let screen_size = monitor.size();

    // Check if window center is on the same monitor as cursor
    // Use physical coordinates for this check since monitor position/size are physical
    let current_pos = window.outer_position().unwrap_or(tauri::PhysicalPosition::new(0, 0));
    let actual_window_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(340, 120));
    let window_center_x = current_pos.x + actual_window_size.width as i32 / 2;
    let window_center_y = current_pos.y + actual_window_size.height as i32 / 2;

    let window_on_same_monitor =
        window_center_x >= screen_pos.x &&
        window_center_x < screen_pos.x + screen_size.width as i32 &&
        window_center_y >= screen_pos.y &&
        window_center_y < screen_pos.y + screen_size.height as i32;

    if !window_on_same_monitor {
        // Calculate target position using LOGICAL coordinates
        // This is the key fix: instead of manually calculating physical positions,
        // we use LogicalPosition and let Tauri handle scale factor conversions
        let target_pos = calculate_dictation_position(&monitor);

        // Hide window, reposition, then show - this prevents the flicker
        window.hide().ok();
        window.set_position(target_pos)
            .map_err(|e| format!("Failed to set position: {}", e))?;
        window.show().ok();
        // Re-apply always-on-top settings after show (macOS may reset them)
        enforce_always_on_top(&window);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn toggle_dictation_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("dictation") {
        if window.is_visible().unwrap_or(false) {
            window.hide().ok();
        } else {
            // Reposition to the monitor where the mouse is before showing
            let target_monitor = if let Ok(cursor_pos) = window.cursor_position() {
                window.available_monitors().ok()
                    .and_then(|monitors| {
                        monitors.into_iter().find(|m| {
                            let pos = m.position();
                            let size = m.size();
                            let cursor_x = cursor_pos.x as i32;
                            let cursor_y = cursor_pos.y as i32;
                            cursor_x >= pos.x && cursor_x < pos.x + size.width as i32 &&
                            cursor_y >= pos.y && cursor_y < pos.y + size.height as i32
                        })
                    })
            } else {
                None
            };

            let monitor = target_monitor
                .or_else(|| window.current_monitor().ok().flatten())
                .or_else(|| window.primary_monitor().ok().flatten());

            if let Some(monitor) = monitor {
                // Use LogicalPosition for proper centering on any monitor
                // This handles scale factor differences automatically
                let target_pos = calculate_dictation_position(&monitor);
                window.set_position(target_pos).ok();
            }

            window.show().ok();
            // Re-apply always-on-top settings after show (macOS may reset them)
            enforce_always_on_top(&window);
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
            // Initialize global hotkey from settings
            let app_handle = app.handle().clone();
            let loaded_settings = settings::load_settings().unwrap_or_default();
            let hotkey_key = loaded_settings.hotkey.key.as_deref();
            hotkey::setup_hotkey(app_handle.clone(), hotkey_key)?;

            // Check if the configured model is downloaded
            let models = transcription::whisper::get_available_models();
            let configured_model = loaded_settings
                .transcription
                .model_size
                .as_deref()
                .unwrap_or("small");

            let model_downloaded = models
                .iter()
                .find(|m| m.id == configured_model)
                .map(|m| m.downloaded)
                .unwrap_or(false);

            if !model_downloaded {
                log::info!("Configured model '{}' not found, emitting model-needs-download event", configured_model);
                app_handle.emit("model-needs-download", configured_model).ok();
            }

            // Position dictation window at bottom center of the monitor where mouse is
            if let Some(window) = app.get_webview_window("dictation") {
                // Try to get cursor position and find the monitor containing it
                let target_monitor = if let Ok(cursor_pos) = window.cursor_position() {
                    // Find the monitor containing the cursor
                    window.available_monitors().ok()
                        .and_then(|monitors| {
                            monitors.into_iter().find(|m| {
                                let pos = m.position();
                                let size = m.size();
                                let cursor_x = cursor_pos.x as i32;
                                let cursor_y = cursor_pos.y as i32;
                                cursor_x >= pos.x && cursor_x < pos.x + size.width as i32 &&
                                cursor_y >= pos.y && cursor_y < pos.y + size.height as i32
                            })
                        })
                } else {
                    None
                };

                // Fall back to current/primary monitor if cursor detection fails
                let monitor = target_monitor
                    .or_else(|| window.current_monitor().ok().flatten())
                    .or_else(|| window.primary_monitor().ok().flatten());

                if let Some(monitor) = monitor {
                    // Use LogicalPosition for proper centering on any monitor
                    // This handles scale factor differences automatically
                    let target_pos = calculate_dictation_position(&monitor);
                    window.set_position(target_pos).ok();
                    window.show().ok();
                    // Apply macOS-specific always-on-top settings (window level + collection behavior)
                    enforce_always_on_top(&window);
                }
            }

            // Build tray menu
            let dashboard_item = MenuItem::with_id(app, "dashboard", "Dashboard...", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let history_item = MenuItem::with_id(app, "history", "History...", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&dashboard_item, &settings_item, &history_item, &toggle_item, &quit_item],
            )?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "dashboard" => {
                        open_dashboard_window(app);
                    }
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
            audio_level_emitter_running: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            inject_text,
            reset_recording_state,
            get_settings,
            update_settings,
            login,
            download_model,
            get_available_models,
            // Stats
            get_stats,
            record_transcription_stats,
            // History
            get_history,
            get_history_entry,
            delete_history_entry,
            clear_history,
            get_history_count,
            // Dictionary
            get_dictionary,
            add_dictionary_entry,
            update_dictionary_entry,
            remove_dictionary_entry,
            // Window positioning
            reposition_to_mouse_monitor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
