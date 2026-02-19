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
    tray::TrayIconBuilder,
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Convert the dictation window to an NSPanel for fullscreen overlay support on macOS.
///
/// IMPORTANT: Only NSPanel can appear above fullscreen applications on macOS.
/// Regular NSWindow cannot do this regardless of window level settings.
/// This is an Apple-enforced limitation since macOS Big Sur.
#[cfg(target_os = "macos")]
fn setup_dictation_panel(app: &tauri::AppHandle) {
    // Use the cocoa types re-exported from tauri_nspanel to avoid version mismatch
    use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior;
    use tauri_nspanel::WebviewWindowExt;

    // Window level constants from NSWindow.h
    // NSMainMenuWindowLevel = 24, we use NSMainMenuWindowLevel + 1 = 25
    const OVERLAY_WINDOW_LEVEL: i32 = 25;
    // NSNonactivatingPanelMask = 1 << 7 = 128 - makes panel not steal focus
    const NS_NONACTIVATING_PANEL_MASK: i32 = 128;

    println!("[nspanel] setup_dictation_panel called");

    if let Some(window) = app.get_webview_window("dictation") {
        println!("[nspanel] Found dictation window, converting to NSPanel...");

        match window.to_panel() {
            Ok(panel) => {
                // Set panel level to above main menu for overlay visibility
                panel.set_level(OVERLAY_WINDOW_LEVEL);
                println!("[nspanel] Panel level set to: {}", OVERLAY_WINDOW_LEVEL);

                // Set collection behavior for fullscreen overlay support:
                // - CanJoinAllSpaces: visible on all desktops/spaces
                // - Stationary: stays in place when switching spaces
                // - FullScreenAuxiliary: can appear above fullscreen apps
                // - IgnoresCycle: excluded from Cmd+Tab app switcher
                let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
                panel.set_collection_behaviour(behavior);
                println!("[nspanel] Collection behavior set for fullscreen overlay");

                // Make the panel non-activating so it doesn't steal focus
                panel.set_style_mask(NS_NONACTIVATING_PANEL_MASK);
                println!("[nspanel] Panel style mask set to non-activating");

                // Additional panel settings for overlay behavior
                panel.set_floating_panel(true);
                panel.set_hides_on_deactivate(false);

                log::info!("Dictation window successfully converted to NSPanel for fullscreen overlay support");
            }
            Err(e) => {
                log::error!("Failed to convert dictation window to NSPanel: {:?}", e);
                println!("[nspanel] ERROR: Failed to convert to panel: {:?}", e);
            }
        }
    } else {
        println!("[nspanel] WARNING: dictation window not found");
    }
}

/// Refresh the panel settings after showing the window.
/// This ensures the panel maintains its fullscreen overlay capabilities
/// and applies the current draggable setting.
#[cfg(target_os = "macos")]
fn refresh_panel_settings(app: &tauri::AppHandle) {
    // Use the cocoa types re-exported from tauri_nspanel to avoid version mismatch
    use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior;
    use tauri_nspanel::ManagerExt;

    // Window level constants
    const OVERLAY_WINDOW_LEVEL: i32 = 25;

    if let Ok(panel) = app.get_webview_panel("dictation") {
        // Re-apply the window level and collection behavior
        panel.set_level(OVERLAY_WINDOW_LEVEL);

        let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
        panel.set_collection_behaviour(behavior);

    }
}

#[cfg(not(target_os = "macos"))]
fn setup_dictation_panel(_app: &tauri::AppHandle) {
    // On non-macOS platforms, the alwaysOnTop config setting is sufficient
}

#[cfg(not(target_os = "macos"))]
fn refresh_panel_settings(_app: &tauri::AppHandle) {
    // On non-macOS platforms, no panel refresh needed
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

/// Convert monitor.position() back to CG display coordinate points.
///
/// On macOS, tao's monitor.position() applies `from_logical(cgdisplaybounds_origin, scale_factor)`
/// which corrupts coordinates in mixed-DPI setups (Tauri issue #7890). The offsets end up in the
/// primary monitor's point space multiplied by each monitor's OWN scale factor, creating
/// overlapping ranges when monitors have different DPI. Dividing back by scale_factor recovers
/// the original CGDisplayBounds point-space values.
fn monitor_origin_points(monitor: &tauri::window::Monitor) -> (f64, f64) {
    let pos = monitor.position();
    let sf = monitor.scale_factor();
    (pos.x as f64 / sf, pos.y as f64 / sf)
}

/// Get monitor size in display coordinate points (logical pixels).
fn monitor_size_points(monitor: &tauri::window::Monitor) -> (f64, f64) {
    let size = monitor.size();
    let sf = monitor.scale_factor();
    (size.width as f64 / sf, size.height as f64 / sf)
}

/// Find the monitor containing the cursor, with a nearest-monitor fallback.
///
/// All comparisons are done in CG display coordinate **points** to avoid the mixed
/// physical/logical coordinate bug (Tauri issue #7890) that causes wrong monitor
/// detection in mixed-DPI multi-monitor setups.
///
/// cursor_position() on macOS returns CGEvent.location() values (CG display points)
/// labeled as PhysicalPosition — they are NOT true physical pixels.
fn find_monitor_for_cursor(
    cursor: &tauri::PhysicalPosition<f64>,
    monitors: &[tauri::window::Monitor],
) -> Option<usize> {
    // cursor is in CG display points (despite being labeled PhysicalPosition)
    let cx = cursor.x;
    let cy = cursor.y;

    // Try strict bounds first — all in point space
    if let Some(idx) = monitors.iter().position(|m| {
        let (mx, my) = monitor_origin_points(m);
        let (mw, mh) = monitor_size_points(m);
        cx >= mx && cx < mx + mw && cy >= my && cy < my + mh
    }) {
        return Some(idx);
    }

    // Fallback: find the monitor whose center is nearest to the cursor in point space
    monitors.iter().enumerate().min_by_key(|(_, m)| {
        let (mx, my) = monitor_origin_points(m);
        let (mw, mh) = monitor_size_points(m);
        let center_x = mx + mw / 2.0;
        let center_y = my + mh / 2.0;
        let dx = cx - center_x;
        let dy = cy - center_y;
        // Multiply by 1000 for precision when casting to integer for min_by_key
        ((dx * dx + dy * dy) * 1000.0) as i64
    }).map(|(idx, _)| idx)
}

/// Constants for dictation window dimensions (logical points, as defined in tauri.conf.json)
const DICTATION_WINDOW_WIDTH: f64 = 340.0;
const DICTATION_WINDOW_HEIGHT: f64 = 120.0;
/// Offset from the bottom of the screen to position just above the macOS dock
/// Wispr Flow uses approximately 20px offset for a snug fit above the dock
const DOCK_OFFSET: f64 = 20.0;

/// Calculate the centered position for the dictation window on a given monitor.
///
/// Returns LogicalPosition in CG display coordinate points. Using LogicalPosition
/// avoids the mixed-DPI coordinate bug (Tauri issue #7890): when set_position receives
/// a PhysicalPosition, tao divides by the window's CURRENT monitor scale factor (not the
/// target's), placing the window on the wrong monitor. LogicalPosition passes through
/// to AppKit's setFrameTopLeftPoint without scale conversion.
fn calculate_dictation_position(monitor: &tauri::window::Monitor) -> tauri::LogicalPosition<f64> {
    let (screen_x, screen_y) = monitor_origin_points(monitor);
    let (screen_w, screen_h) = monitor_size_points(monitor);

    // Calculate centered bottom position in display points
    let x = screen_x + (screen_w - DICTATION_WINDOW_WIDTH) / 2.0;
    let y = screen_y + screen_h - DICTATION_WINDOW_HEIGHT - DOCK_OFFSET;

    tauri::LogicalPosition::new(x, y)
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

/// Reposition dictation window to the monitor where the mouse currently is.
/// All coordinate math is done in CG display coordinate **points** to work around
/// the mixed physical/logical coordinate bug in tao on macOS (Tauri issue #7890).
/// Returns true if window was moved to a different monitor.
#[tauri::command]
fn reposition_to_mouse_monitor(app: tauri::AppHandle) -> Result<bool, String> {
    // Skip repositioning when widget is draggable (user controls position)
    let is_draggable = app.state::<AppState>().settings.lock()
        .map(|s| s.widget.draggable)
        .unwrap_or(false);
    if is_draggable {
        return Ok(false);
    }

    let window = app.get_webview_window("dictation")
        .ok_or_else(|| "Dictation window not found".to_string())?;

    // Skip if window is not visible
    if !window.is_visible().unwrap_or(false) {
        return Ok(false);
    }

    // cursor_position() returns CG display points (labeled as PhysicalPosition on macOS)
    let cursor_pos = window.cursor_position()
        .map_err(|e| format!("Failed to get cursor position: {}", e))?;

    // Find the monitor containing the cursor (comparison in point space)
    let monitors: Vec<_> = window.available_monitors()
        .map_err(|e| format!("Failed to get monitors: {}", e))?
        .into_iter().collect();

    let monitor = find_monitor_for_cursor(&cursor_pos, &monitors)
        .map(|i| monitors[i].clone())
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "No monitor found".to_string())?;

    // Check if window center is already on the target monitor — all in point space.
    // outer_position() returns PhysicalPosition scaled by the window's current monitor
    // scale factor. Dividing by that factor recovers CG display points.
    let win_scale = window.scale_factor().unwrap_or(1.0);
    let current_pos = window.outer_position().unwrap_or(tauri::PhysicalPosition::new(0, 0));
    let actual_window_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(
        (DICTATION_WINDOW_WIDTH * win_scale) as u32,
        (DICTATION_WINDOW_HEIGHT * win_scale) as u32,
    ));

    // Convert to display points
    let win_x = current_pos.x as f64 / win_scale;
    let win_y = current_pos.y as f64 / win_scale;
    let win_w = actual_window_size.width as f64 / win_scale;
    let win_h = actual_window_size.height as f64 / win_scale;
    let win_center_x = win_x + win_w / 2.0;
    let win_center_y = win_y + win_h / 2.0;

    let (screen_x, screen_y) = monitor_origin_points(&monitor);
    let (screen_w, screen_h) = monitor_size_points(&monitor);

    let window_on_same_monitor =
        win_center_x >= screen_x &&
        win_center_x < screen_x + screen_w &&
        win_center_y >= screen_y &&
        win_center_y < screen_y + screen_h;

    if !window_on_same_monitor {
        let target_pos = calculate_dictation_position(&monitor);

        // Move directly without hide/show — the hide→show cycle causes a visible
        // flash/blink. set_position alone is instantaneous on macOS.
        window.set_position(target_pos)
            .map_err(|e| format!("Failed to set position: {}", e))?;
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
            // Check if widget is draggable - if so, skip repositioning to preserve user's position
            let is_draggable = app.state::<AppState>().settings.lock()
                .map(|s| s.widget.draggable)
                .unwrap_or(false);

            if !is_draggable {
                // Reposition to the monitor where the mouse is before showing
                let monitor = window.cursor_position().ok()
                    .and_then(|cursor_pos| {
                        let monitors: Vec<_> = window.available_monitors().ok()?.into_iter().collect();
                        find_monitor_for_cursor(&cursor_pos, &monitors).map(|i| monitors[i].clone())
                    })
                    .or_else(|| window.current_monitor().ok().flatten())
                    .or_else(|| window.primary_monitor().ok().flatten());

                if let Some(monitor) = monitor {
                    let target_pos = calculate_dictation_position(&monitor);
                    window.set_position(target_pos).ok();
                }
            }

            window.show().ok();
            // Re-apply panel settings after show (macOS may reset them)
            refresh_panel_settings(app);
        }
    }
}

pub fn run() {
    env_logger::init();

    // Load or create default settings
    let settings = settings::load_settings().unwrap_or_default();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());

    // Add NSPanel plugin on macOS for fullscreen overlay support
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder.setup(|app| {
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
                let monitor = window.cursor_position().ok()
                    .and_then(|cursor_pos| {
                        let monitors: Vec<_> = window.available_monitors().ok()?.into_iter().collect();
                        find_monitor_for_cursor(&cursor_pos, &monitors).map(|i| monitors[i].clone())
                    })
                    .or_else(|| window.current_monitor().ok().flatten())
                    .or_else(|| window.primary_monitor().ok().flatten());

                if let Some(monitor) = monitor {
                    let target_pos = calculate_dictation_position(&monitor);
                    window.set_position(target_pos).ok();
                    window.show().ok();
                } else {
                    println!("[window] setup: WARNING - no monitor found");
                }
            } else {
                println!("[window] setup: WARNING - dictation window not found");
            }

            // Convert dictation window to NSPanel on macOS for fullscreen overlay support
            // This MUST be done after the window is shown and rendered
            setup_dictation_panel(&app_handle);

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
                .show_menu_on_left_click(true)
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
