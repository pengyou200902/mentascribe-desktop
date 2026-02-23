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

        // Apply opacity from settings
        let opacity = app.state::<AppState>().settings.lock()
            .map(|s| s.widget.opacity)
            .unwrap_or(1.0);
        apply_panel_opacity(app, opacity);
    }
}

/// Apply opacity to the NSPanel via [NSWindow setAlphaValue:]
#[cfg(target_os = "macos")]
fn apply_panel_opacity(app: &tauri::AppHandle, opacity: f64) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    use tauri_nspanel::ManagerExt;

    let opacity = opacity.clamp(MIN_PANEL_OPACITY, MAX_PANEL_OPACITY);
    if let Ok(panel) = app.get_webview_panel("dictation") {
        unsafe {
            let ns_panel: id = msg_send![&*panel, self];
            let _: () = msg_send![ns_panel, setAlphaValue: opacity as f64];
        }
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

    // Start streaming transcription in background.
    // Dispatches to Voxtral (native streaming) or Whisper (VAD-triggered) based on engine setting.
    {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;

        if is_voxtral_engine(&settings) {
            #[cfg(feature = "voxtral")]
            {
                let delay_ms = settings.transcription.voxtral_delay_ms.unwrap_or(480);
                transcription::voxtral::start_streaming(transcription::voxtral::StreamingConfig {
                    delay_ms,
                }).map_err(|e| {
                    eprintln!("[recording] ERROR: Voxtral streaming start failed: {}", e);
                    // Reset recording state since we failed
                    *is_recording = false;
                    e.to_string()
                })?;
            }
            #[cfg(not(feature = "voxtral"))]
            {
                *is_recording = false;
                return Err("Voxtral engine not available (not compiled)".to_string());
            }
        } else {
            let model_size = settings
                .transcription
                .model_size
                .clone()
                .unwrap_or_else(|| "small".to_string());
            let language = settings.transcription.language.clone();
            transcription::whisper::start_streaming(transcription::whisper::StreamingConfig {
                model_size,
                language,
            });
        }
    }

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

            std::thread::sleep(std::time::Duration::from_millis(AUDIO_LEVEL_SLEEP_MS));
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

    // Stop streaming monitor first (ensures all in-progress transcriptions complete
    // before we stop capture). Returns accumulated results and consumed sample count.
    let use_voxtral = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        is_voxtral_engine(&settings)
    };

    eprintln!("[recording] Stopping streaming monitor (engine={})...", if use_voxtral { "voxtral" } else { "whisper" });

    let (streaming_results, consumed_samples) = if use_voxtral {
        #[cfg(feature = "voxtral")]
        { transcription::voxtral::stop_streaming() }
        #[cfg(not(feature = "voxtral"))]
        { (Vec::new(), 0usize) }
    } else {
        transcription::whisper::stop_streaming()
    };

    let streaming_prefix = if streaming_results.is_empty() {
        eprintln!("[recording] No streaming results (no completed utterances detected)");
        None
    } else {
        // Voxtral tokens include their own spacing (e.g. " Hello," " world.").
        // Whisper segments are separate sentences that need a space between them.
        let prefix = if use_voxtral {
            streaming_results.join("")
        } else {
            streaming_results.join(" ")
        };
        eprintln!(
            "[recording] Streaming results: {} segments, {} consumed samples, prefix='{}...'",
            streaming_results.len(),
            consumed_samples,
            if prefix.len() > 60 { &prefix[..60] } else { &prefix }
        );
        Some(prefix)
    };

    // Stop audio capture and get audio data
    eprintln!("[recording] Stopping audio capture...");
    let mut audio_data = audio::capture::stop_capture().map_err(|e| {
        eprintln!("[recording] ERROR: Failed to stop audio capture: {}", e);
        e.to_string()
    })?;
    eprintln!(
        "[recording] Audio captured: {} samples at {}Hz ({:.2}s)",
        audio_data.samples.len(),
        audio_data.sample_rate,
        audio_data.samples.len() as f32 / audio_data.sample_rate as f32
    );

    // Trim whisper_samples to only the tail (audio not yet transcribed by streaming).
    // This dramatically reduces inference time on stop — only the final partial utterance
    // needs processing instead of the entire recording.
    if consumed_samples > 0 {
        if let Some(ref mut ws) = audio_data.whisper_samples {
            if consumed_samples < ws.len() {
                let tail_len = ws.len() - consumed_samples;
                eprintln!(
                    "[recording] Trimming whisper buffer: {} total -> {} tail ({:.2}s)",
                    ws.len(),
                    tail_len,
                    tail_len as f32 / 16000.0
                );
                *ws = ws[consumed_samples..].to_vec();
            } else {
                eprintln!(
                    "[recording] All audio consumed by streaming ({} >= {}), no tail",
                    consumed_samples,
                    ws.len()
                );
                *ws = Vec::new();
            }
        }
    }

    // Emit processing event
    app.emit("transcription-processing", ()).ok();

    // Clone settings for use in async block
    let settings = {
        let s = state.settings.lock().map_err(|e| e.to_string())?;
        s.clone()
    };

    // Calculate duration before moving audio_data into transcribe
    let duration_ms = (audio_data.samples.len() as f32 / audio_data.sample_rate as f32 * 1000.0) as u32;

    // Transcribe remaining tail audio and combine with streaming prefix.
    // Voxtral streaming processes ALL audio incrementally (including finish()),
    // so when consumed_samples == usize::MAX we skip tail transcription entirely —
    // the streaming results ARE the final transcription.
    let raw_text = if use_voxtral && consumed_samples == usize::MAX {
        // Voxtral streaming already processed everything. No tail needed.
        let text = streaming_prefix.unwrap_or_default();
        eprintln!(
            "[recording] Voxtral streaming handled all audio, skipping tail transcription (text='{}')",
            if text.len() > 60 { &text[..60] } else { &text }
        );
        text
    } else if use_voxtral {
        // Voxtral streaming wasn't active (model not loaded), try one-shot
        eprintln!("[recording] Starting voxtral one-shot transcription...");
        #[cfg(feature = "voxtral")]
        {
            transcription::voxtral::transcribe(audio_data, &settings, streaming_prefix)
                .await
                .map_err(|e| {
                    eprintln!("[recording] ERROR: Voxtral transcription failed: {}", e);
                    e.to_string()
                })?
        }
        #[cfg(not(feature = "voxtral"))]
        {
            streaming_prefix.unwrap_or_default()
        }
    } else {
        transcription::whisper::transcribe(audio_data, &settings, streaming_prefix)
            .await
            .map_err(|e| {
                eprintln!("[recording] ERROR: Transcription failed: {}", e);
                e.to_string()
            })?
    };
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
    let (old_hotkey, old_draggable, old_opacity, old_model_size, old_engine) = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        (
            settings.hotkey.key.clone(),
            settings.widget.draggable,
            settings.widget.opacity,
            settings.transcription.model_size.clone(),
            settings.transcription.engine.clone(),
        )
    };

    let new_draggable = new_settings.widget.draggable;
    if old_draggable != new_draggable {
        eprintln!("[settings] DRAGGABLE CHANGED: {} -> {}", old_draggable, new_draggable);

        // When draggable is turned OFF, snap widget back to bottom-center of current screen
        #[cfg(target_os = "macos")]
        if !new_draggable {
            eprintln!("[settings] Snapping widget to bottom-center (draggable OFF)");
            native_position_on_cursor_monitor(&app, false).ok();
        }
    }

    let new_opacity = new_settings.widget.opacity;

    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings.clone();

    // Persist settings
    settings::save_settings(&new_settings).map_err(|e| e.to_string())?;

    // Re-register hotkey if it changed
    if old_hotkey != new_settings.hotkey.key {
        drop(settings); // Release lock before hotkey operations
        hotkey::unregister_all(&app).map_err(|e| e.to_string())?;
        hotkey::setup_hotkey(app.clone(), new_settings.hotkey.key.as_deref())
            .map_err(|e| e.to_string())?;
    }

    // Apply opacity change to NSPanel
    #[cfg(target_os = "macos")]
    if (old_opacity - new_opacity).abs() > f64::EPSILON {
        apply_panel_opacity(&app, new_opacity);
    }

    // Notify all windows (especially dictation) that settings changed
    app.emit("settings-changed", &new_settings).ok();

    // Handle engine switching — unload old engine to free GPU memory
    let new_engine = new_settings.transcription.engine.clone();
    if old_engine != new_engine {
        let switching_to_voxtral = new_engine.as_deref() == Some("voxtral");
        log::info!("Engine changed: {:?} -> {:?}", old_engine, new_engine);

        if switching_to_voxtral {
            // Unload Whisper to free GPU memory, preload Voxtral
            #[cfg(feature = "voxtral")]
            {
                // Note: We don't have a whisper::unload_model() — the cache is replaced on next preload
                if transcription::voxtral::is_model_downloaded() {
                    let preload_app = app.clone();
                    std::thread::spawn(move || {
                        log::info!("Switching to Voxtral, preloading...");
                        preload_app.emit("model-preload-start", "voxtral-mini-4b").ok();
                        let start = std::time::Instant::now();
                        match transcription::voxtral::preload_model() {
                            Ok(()) => {
                                let elapsed = start.elapsed().as_secs_f64();
                                log::info!("Voxtral preloaded in {:.2}s", elapsed);
                                preload_app.emit("model-preload-complete", serde_json::json!({
                                    "model": "voxtral-mini-4b",
                                    "elapsed_secs": elapsed,
                                })).ok();
                            }
                            Err(e) => {
                                log::error!("Failed to preload Voxtral: {}", e);
                                preload_app.emit("model-preload-error", serde_json::json!({
                                    "model": "voxtral-mini-4b",
                                    "error": e.to_string(),
                                })).ok();
                            }
                        }
                    });
                }
            }
        } else {
            // Switching away from Voxtral — unload it, preload Whisper
            #[cfg(feature = "voxtral")]
            {
                transcription::voxtral::unload_model();
            }
        }
    }

    // Preload new Whisper model in background if model_size changed (and using Whisper engine)
    let new_model_size = new_settings.transcription.model_size.clone();
    if old_model_size != new_model_size && !is_voxtral_engine(&new_settings) {
        if let Some(model_size) = new_model_size {
            let preload_app = app.clone();
            std::thread::spawn(move || {
                log::info!("Model changed to '{}', preloading in background...", model_size);
                preload_app.emit("model-preload-start", &model_size).ok();
                let start = std::time::Instant::now();
                match transcription::whisper::preload_model(&model_size) {
                    Ok(()) => {
                        let elapsed = start.elapsed().as_secs_f64();
                        log::info!("Model '{}' preloaded in {:.2}s", model_size, elapsed);
                        preload_app
                            .emit(
                                "model-preload-complete",
                                serde_json::json!({
                                    "model": &model_size,
                                    "elapsed_secs": elapsed,
                                }),
                            )
                            .ok();
                    }
                    Err(e) => {
                        log::error!("Failed to preload model '{}': {}", model_size, e);
                        preload_app
                            .emit(
                                "model-preload-error",
                                serde_json::json!({
                                    "model": &model_size,
                                    "error": e.to_string(),
                                }),
                            )
                            .ok();
                    }
                }
            });
        }
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
async fn download_model(app: tauri::AppHandle, size: String) -> Result<(), String> {
    let app_clone = app.clone();
    let size_clone = size.clone();
    transcription::whisper::download_model(&size, move |percent| {
        app_clone
            .emit(
                "download-progress",
                serde_json::json!({
                    "model_type": "ggml",
                    "model_id": &size_clone,
                    "percent": percent,
                }),
            )
            .ok();
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_available_models() -> Vec<transcription::ModelInfo> {
    transcription::whisper::get_available_models()
}

#[tauri::command]
fn get_coreml_status() -> transcription::CoremlStatus {
    transcription::whisper::get_coreml_status()
}

#[tauri::command]
fn get_metal_status() -> transcription::MetalStatus {
    transcription::whisper::get_metal_status()
}

#[tauri::command]
async fn download_coreml_model(app: tauri::AppHandle, size: String) -> Result<(), String> {
    let app_clone = app.clone();
    let size_clone = size.clone();
    transcription::whisper::download_coreml_model(&size, move |percent| {
        app_clone
            .emit(
                "download-progress",
                serde_json::json!({
                    "model_type": "coreml",
                    "model_id": &size_clone,
                    "percent": percent,
                }),
            )
            .ok();
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_model(size: String) -> Result<(), String> {
    transcription::whisper::delete_model(&size).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_coreml_model(size: String) -> Result<(), String> {
    transcription::whisper::delete_coreml_model(&size).map_err(|e| e.to_string())
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

// ---------------------------------------------------------------------------
// Voxtral IPC commands (feature-gated)
// ---------------------------------------------------------------------------

/// Check if the current engine setting is "voxtral" AND the feature is compiled in.
fn is_voxtral_engine(settings: &settings::UserSettings) -> bool {
    #[cfg(feature = "voxtral")]
    {
        settings.transcription.engine.as_deref() == Some("voxtral")
    }
    #[cfg(not(feature = "voxtral"))]
    {
        let _ = settings;
        false
    }
}

#[tauri::command]
fn get_voxtral_status() -> transcription::VoxtralStatus {
    #[cfg(feature = "voxtral")]
    {
        let s = transcription::voxtral::get_status();
        transcription::VoxtralStatus {
            compiled: s.compiled,
            metal: s.metal,
            model_downloaded: s.model_downloaded,
            model_loaded: s.model_loaded,
        }
    }
    #[cfg(not(feature = "voxtral"))]
    {
        transcription::VoxtralStatus::default()
    }
}

#[tauri::command]
fn get_voxtral_models() -> Vec<transcription::ModelInfo> {
    #[cfg(feature = "voxtral")]
    {
        transcription::voxtral::get_available_models()
    }
    #[cfg(not(feature = "voxtral"))]
    {
        Vec::new()
    }
}

#[tauri::command]
async fn download_voxtral_model(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(feature = "voxtral")]
    {
        let app_clone = app.clone();
        transcription::voxtral::download_model(move |percent| {
            app_clone
                .emit(
                    "download-progress",
                    serde_json::json!({
                        "model_type": "voxtral",
                        "model_id": "voxtral-mini-4b",
                        "percent": percent,
                    }),
                )
                .ok();
        })
        .await
        .map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "voxtral"))]
    {
        let _ = app;
        Err("Voxtral feature not compiled".to_string())
    }
}

#[tauri::command]
fn delete_voxtral_model() -> Result<(), String> {
    #[cfg(feature = "voxtral")]
    {
        transcription::voxtral::delete_model().map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "voxtral"))]
    {
        Err("Voxtral feature not compiled".to_string())
    }
}

/// Frontend debug log forwarding — prints to terminal so we can see drag events
#[tauri::command]
fn frontend_log(msg: String) {
    eprintln!("[frontend] {}", msg);
}

/// Native drag state — stored in a static so NSEvent monitor blocks can access it.
/// All coordinates are in AppKit space (bottom-left origin, Y increases upward).
#[cfg(target_os = "macos")]
struct NativeDragState {
    initial_mouse_x: f64,
    initial_mouse_y: f64,
    initial_origin_x: f64,
    initial_origin_y: f64,
    panel_ptr: usize,        // NSPanel id stored as usize (for Send)
    monitors: [usize; 4],    // [local_drag, global_drag, local_mouseup, global_mouseup]
    active: bool,            // false = drag ended, handlers become no-ops
}

// SAFETY: Fields are only accessed from the main thread (monitor handlers + tauri commands)
#[cfg(target_os = "macos")]
unsafe impl Send for NativeDragState {}

#[cfg(target_os = "macos")]
static NATIVE_DRAG_STATE: std::sync::Mutex<Option<NativeDragState>> = std::sync::Mutex::new(None);

/// GCD FFI for deferring work to next run loop iteration.
/// Note: &_dispatch_main_q as *const _ is a C macro expanding to &_dispatch_main_q,
/// so we link the actual symbol directly.
#[cfg(target_os = "macos")]
extern "C" {
    static _dispatch_main_q: std::os::raw::c_void;
    fn dispatch_async_f(
        queue: *const std::os::raw::c_void,
        context: *mut std::os::raw::c_void,
        work: extern "C" fn(*mut std::os::raw::c_void),
    );
}

/// Callback for dispatch_async_f — removes monitors on the NEXT run loop iteration.
/// Apple docs: "It is NOT safe to remove a monitor from within the handler block."
#[cfg(target_os = "macos")]
extern "C" fn deferred_remove_monitors(_ctx: *mut std::os::raw::c_void) {
    use cocoa::base::id;
    use objc::{class, msg_send, sel, sel_impl};

    if let Ok(mut guard) = NATIVE_DRAG_STATE.lock() {
        if let Some(state) = guard.take() {
            for &mon in &state.monitors {
                if mon != 0 {
                    unsafe {
                        let _: () = msg_send![class!(NSEvent), removeMonitor: mon as id];
                    }
                }
            }
            eprintln!("[native_drag] Monitors removed (deferred)");
        }
    }
}

/// Remove native drag event monitors and clear state.
/// Safe to call from outside handlers (e.g. start of a new drag).
#[cfg(target_os = "macos")]
fn stop_native_drag_inner() {
    use cocoa::base::id;
    use objc::{class, msg_send, sel, sel_impl};

    if let Ok(mut guard) = NATIVE_DRAG_STATE.lock() {
        if let Some(state) = guard.take() {
            for &mon in &state.monitors {
                if mon != 0 {
                    unsafe {
                        let _: () = msg_send![class!(NSEvent), removeMonitor: mon as id];
                    }
                }
            }
            eprintln!("[native_drag] Drag stopped, monitors removed");
        }
    }
}

/// Handle a drag/mouseUp event from an NSEvent monitor.
/// Called from both local and global monitor blocks.
/// Returns true if drag ended (mouseUp).
#[cfg(target_os = "macos")]
fn handle_native_drag_event(event_type: u64) {
    use cocoa::base::id;
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};

    if event_type == 6 {
        // NSLeftMouseDragged — move panel to follow mouse
        if let Ok(guard) = NATIVE_DRAG_STATE.lock() {
            if let Some(state) = guard.as_ref() {
                if !state.active { return; }
                unsafe {
                    let mouse: NSPoint = msg_send![class!(NSEvent), mouseLocation];
                    let dx = mouse.x - state.initial_mouse_x;
                    let dy = mouse.y - state.initial_mouse_y;
                    let new_origin = NSPoint::new(
                        state.initial_origin_x + dx,
                        state.initial_origin_y + dy,
                    );
                    let panel = state.panel_ptr as *mut objc::runtime::Object;
                    let _: () = msg_send![panel, setFrameOrigin: new_origin];
                }
            }
        }
    } else if event_type == 2 {
        // NSLeftMouseUp — mark drag ended, defer monitor removal
        if let Ok(mut guard) = NATIVE_DRAG_STATE.lock() {
            if let Some(state) = guard.as_mut() {
                state.active = false;
            }
        }
        // IMPORTANT: Cannot removeMonitor from inside its handler block!
        // Defer to the next run loop iteration via GCD.
        unsafe {
            dispatch_async_f(
                &_dispatch_main_q as *const _,
                std::ptr::null_mut(),
                deferred_remove_monitors,
            );
        }
        eprintln!("[native_drag] MouseUp — deferred cleanup scheduled");
    }
}

/// Start native drag — installs NSEvent monitors that track mouse movement entirely
/// in AppKit coordinate space, bypassing JavaScript's broken screenX/screenY on mixed-DPI.
///
/// Called from JS mousedown. The monitors handle all movement and auto-cleanup on mouseup.
#[cfg(target_os = "macos")]
#[tauri::command]
fn start_native_drag(app: tauri::AppHandle) -> Result<(), String> {
    use cocoa::base::id;
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};
    use tauri_nspanel::ManagerExt;
    use tauri_nspanel::block::ConcreteBlock;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSRect { origin: NSPoint, size: NSPoint }

    // Clean up any existing drag first (safe — called outside handler)
    stop_native_drag_inner();

    let panel = app.get_webview_panel("dictation")
        .map_err(|e| format!("{:?}", e))?;

    unsafe {
        let mouse: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        let frame: NSRect = msg_send![&*panel, frame];
        // Get the actual NSPanel id pointer via [panel self].
        // IMPORTANT: &*panel may be a reference into the Rust WebviewPanel struct,
        // which gets freed when `panel` drops. [self] returns the underlying ObjC
        // object pointer that AppKit retains independently.
        let ns_panel: id = msg_send![&*panel, self];
        let panel_ptr = ns_panel as usize;

        eprintln!("[native_drag] Starting: mouse=({:.1},{:.1}), origin=({:.1},{:.1}), panel_ptr=0x{:x}",
            mouse.x, mouse.y, frame.origin.x, frame.origin.y, panel_ptr);

        // Store initial state (monitors will be updated after installation)
        *NATIVE_DRAG_STATE.lock().map_err(|e| e.to_string())? = Some(NativeDragState {
            initial_mouse_x: mouse.x,
            initial_mouse_y: mouse.y,
            initial_origin_x: frame.origin.x,
            initial_origin_y: frame.origin.y,
            panel_ptr,
            monitors: [0; 4],
            active: true,
        });

        // Use separate monitors for drag vs mouseUp to avoid calling msg_send!
        // inside the block (the objc crate's debug verification panics inside
        // extern "C" block invoke functions, which can't unwind).
        let drag_mask: u64 = 1 << 6;   // NSEventMaskLeftMouseDragged
        let mouseup_mask: u64 = 1 << 2; // NSEventMaskLeftMouseUp

        // Drag handler — moves panel (local monitor returns event, global returns void)
        let local_drag = ConcreteBlock::new(|event: id| -> id {
            handle_native_drag_event(6); // NSLeftMouseDragged
            event
        });
        let local_drag = local_drag.copy();

        let local_drag_monitor: id = msg_send![
            class!(NSEvent),
            addLocalMonitorForEventsMatchingMask: drag_mask
            handler: &*local_drag
        ];

        let global_drag = ConcreteBlock::new(|_event: id| {
            handle_native_drag_event(6);
        });
        let global_drag = global_drag.copy();

        let global_drag_monitor: id = msg_send![
            class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: drag_mask
            handler: &*global_drag
        ];

        // MouseUp handler — ends drag
        let local_mouseup = ConcreteBlock::new(|event: id| -> id {
            handle_native_drag_event(2); // NSLeftMouseUp
            event
        });
        let local_mouseup = local_mouseup.copy();

        let local_mouseup_monitor: id = msg_send![
            class!(NSEvent),
            addLocalMonitorForEventsMatchingMask: mouseup_mask
            handler: &*local_mouseup
        ];

        let global_mouseup = ConcreteBlock::new(|_event: id| {
            handle_native_drag_event(2);
        });
        let global_mouseup = global_mouseup.copy();

        let global_mouseup_monitor: id = msg_send![
            class!(NSEvent),
            addGlobalMonitorForEventsMatchingMask: mouseup_mask
            handler: &*global_mouseup
        ];

        // Update state with monitor IDs
        if let Ok(mut guard) = NATIVE_DRAG_STATE.lock() {
            if let Some(state) = guard.as_mut() {
                state.monitors = [
                    local_drag_monitor as usize,
                    global_drag_monitor as usize,
                    local_mouseup_monitor as usize,
                    global_mouseup_monitor as usize,
                ];
            }
        }

        eprintln!("[native_drag] Monitors installed: local_drag={:?}, global_drag={:?}, local_mouseup={:?}, global_mouseup={:?}",
            local_drag_monitor, global_drag_monitor, local_mouseup_monitor, global_mouseup_monitor);
    }

    Ok(())
}

/// Constants for dictation window dimensions (logical points, as defined in tauri.conf.json).
/// These are initial/fallback values; the frontend dynamically resizes the window to match
/// the pill widget, so native positioning uses the actual window frame size instead.
const DICTATION_WINDOW_WIDTH: f64 = 52.0;
const DICTATION_WINDOW_HEIGHT: f64 = 10.0;
/// Offset from the bottom of the screen to position just above the macOS dock
const DOCK_OFFSET: f64 = 20.0;
/// Extra padding around pill frame for cursor proximity detection
const CURSOR_PROXIMITY_PADDING: f64 = 20.0;
/// Opacity clamp range for the dictation panel
const MIN_PANEL_OPACITY: f64 = 0.2;
const MAX_PANEL_OPACITY: f64 = 1.0;
/// Audio level emitter sleep interval
const AUDIO_LEVEL_SLEEP_MS: u64 = 25;

/// Position the dictation panel at bottom-center of the monitor containing the cursor.
///
/// Uses native macOS AppKit APIs directly, staying entirely in AppKit coordinate space
/// (bottom-left origin, y increases upward). This bypasses tao's coordinate conversion
/// layer which has multiple bugs in mixed-DPI multi-monitor setups:
/// - Tauri issue #7890: monitor.position() mixes logical/physical coordinate spaces
/// - tao's set_position(PhysicalPosition) divides by the window's CURRENT scale factor
/// - tao's window_position() uses NSScreen.mainScreen (changes with focus) for y-flip
///
/// By using NSEvent.mouseLocation + NSScreen.screens + setFrameOrigin directly, all
/// coordinates stay in one consistent AppKit space with no conversions.
///
/// If `only_if_different_monitor` is true, skips repositioning when the window center
/// is already on the cursor's screen (used by the 150ms poll to avoid unnecessary moves).
#[cfg(target_os = "macos")]
fn native_position_on_cursor_monitor(app: &tauri::AppHandle, only_if_different_monitor: bool) -> Result<bool, String> {
    use cocoa::base::id;
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};
    use tauri_nspanel::ManagerExt;

    // NSRect is a CGRect — we define a local copy to avoid import issues across cocoa versions
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSRect {
        origin: NSPoint,
        size: NSPoint, // NSSize has the same layout as NSPoint (two f64s)
    }

    let panel = app.get_webview_panel("dictation")
        .map_err(|e| {
            eprintln!("[native_pos] ERROR: Failed to get panel: {:?}", e);
            format!("{:?}", e)
        })?;

    unsafe {
        // Get cursor position in AppKit coordinates (bottom-left origin, y increases upward)
        let mouse_loc: NSPoint = msg_send![class!(NSEvent), mouseLocation];

        // Iterate NSScreen.screens to find the one containing the cursor
        let screens: id = msg_send![class!(NSScreen), screens];
        let count: usize = msg_send![screens, count];

        let mut target_screen_frame: Option<NSRect> = None;
        let mut target_visible_frame: Option<NSRect> = None;
        let mut target_screen_idx: usize = 0;

        for i in 0..count {
            let screen: id = msg_send![screens, objectAtIndex: i];
            let frame: NSRect = msg_send![screen, frame];
            if mouse_loc.x >= frame.origin.x && mouse_loc.x < frame.origin.x + frame.size.x &&
               mouse_loc.y >= frame.origin.y && mouse_loc.y < frame.origin.y + frame.size.y {
                let visible: NSRect = msg_send![screen, visibleFrame];
                target_screen_frame = Some(frame);
                target_visible_frame = Some(visible);
                target_screen_idx = i;
                break;
            }
        }

        let screen_frame = target_screen_frame
            .ok_or_else(|| {
                eprintln!("[native_pos] ERROR: No screen found for cursor at ({:.1}, {:.1}), {} screens available", mouse_loc.x, mouse_loc.y, count);
                "No screen found for cursor".to_string()
            })?;
        let visible_frame = target_visible_frame.unwrap();

        // Get actual window frame (frontend dynamically resizes to match pill)
        let win_frame: NSRect = msg_send![&*panel, frame];

        if only_if_different_monitor {
            // Check if window center is already on the target screen
            let cx = win_frame.origin.x + win_frame.size.x / 2.0;
            let cy = win_frame.origin.y + win_frame.size.y / 2.0;

            let on_same_screen =
                cx >= screen_frame.origin.x &&
                cx < screen_frame.origin.x + screen_frame.size.x &&
                cy >= screen_frame.origin.y &&
                cy < screen_frame.origin.y + screen_frame.size.y;

            if on_same_screen {
                return Ok(false);
            }
            eprintln!("[native_pos] MOVING: window center ({:.1}, {:.1}) NOT on screen {} (origin: {:.1},{:.1} size: {:.1}x{:.1})",
                cx, cy, target_screen_idx,
                screen_frame.origin.x, screen_frame.origin.y,
                screen_frame.size.x, screen_frame.size.y);
        }

        // Calculate bottom-center in AppKit coordinates using actual window width.
        // visibleFrame already excludes dock and menu bar areas.
        // In AppKit, origin.y is the bottom edge, so we add DOCK_OFFSET above it.
        let actual_width = if win_frame.size.x > 0.0 { win_frame.size.x } else { DICTATION_WINDOW_WIDTH };
        let x = visible_frame.origin.x + (visible_frame.size.x - actual_width) / 2.0;
        let y = visible_frame.origin.y + DOCK_OFFSET;

        eprintln!("[native_pos] Positioning on screen {} — mouse: ({:.1}, {:.1}), target: ({:.1}, {:.1}), visible: origin({:.1},{:.1}) size({:.1}x{:.1})",
            target_screen_idx, mouse_loc.x, mouse_loc.y, x, y,
            visible_frame.origin.x, visible_frame.origin.y,
            visible_frame.size.x, visible_frame.size.y);

        let new_origin = NSPoint::new(x, y);
        let _: () = msg_send![&*panel, setFrameOrigin: new_origin];

        Ok(true)
    }
}

/// Resize the dictation pill window while keeping its bottom edge and horizontal center fixed.
///
/// Called by the frontend's ResizeObserver when the pill CSS-transitions between collapsed
/// and expanded states. Uses `setFrame:display:` to atomically set both size and position,
/// preventing the visual jump that `setContentSize:` would cause (it preserves the top-left
/// corner, but we need the bottom edge anchored so the pill grows upward).
#[cfg(target_os = "macos")]
#[tauri::command]
fn resize_pill(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), String> {
    use cocoa::foundation::NSPoint;
    use objc::{msg_send, sel, sel_impl};
    use tauri_nspanel::ManagerExt;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSRect { origin: NSPoint, size: NSPoint }

    let panel = app.get_webview_panel("dictation")
        .map_err(|e| format!("{:?}", e))?;

    unsafe {
        let old_frame: NSRect = msg_send![&*panel, frame];

        // Keep bottom edge (origin.y) fixed, re-center horizontally
        let old_center_x = old_frame.origin.x + old_frame.size.x / 2.0;
        let new_frame = NSRect {
            origin: NSPoint::new(old_center_x - width / 2.0, old_frame.origin.y),
            size: NSPoint::new(width, height),
        };

        let display: i8 = 1; // BOOL YES
        let _: () = msg_send![&*panel, setFrame:new_frame display:display];
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn resize_pill(app: tauri::AppHandle, width: f64, height: f64) -> Result<(), String> {
    use tauri::Manager;
    if let Some(win) = app.get_webview_window("dictation") {
        win.set_size(tauri::LogicalSize::new(width, height))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Check if the cursor is within proximity of the pill window.
///
/// Uses [NSEvent mouseLocation] which reports the global cursor position regardless
/// of which app has focus. This solves the hover-detection problem where the NSPanel
/// (with NSNonactivatingPanelMask) doesn't forward mouse events to the webview when
/// another application is focused.
#[cfg(target_os = "macos")]
#[tauri::command]
fn is_cursor_over_pill(app: tauri::AppHandle) -> Result<bool, String> {
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};
    use tauri_nspanel::ManagerExt;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct NSRect { origin: NSPoint, size: NSPoint }

    let panel = app.get_webview_panel("dictation")
        .map_err(|e| format!("{:?}", e))?;

    unsafe {
        let mouse: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        let frame: NSRect = msg_send![&*panel, frame];

        let padding = CURSOR_PROXIMITY_PADDING;
        let inside =
            mouse.x >= frame.origin.x - padding &&
            mouse.x <= frame.origin.x + frame.size.x + padding &&
            mouse.y >= frame.origin.y - padding &&
            mouse.y <= frame.origin.y + frame.size.y + padding;

        Ok(inside)
    }
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn is_cursor_over_pill(_app: tauri::AppHandle) -> Result<bool, String> {
    // Non-macOS: fall back to always false (JS events handle hover)
    Ok(false)
}

/// Open the dashboard window, optionally navigating to a specific page.
fn open_dashboard_window(app: &tauri::AppHandle, page: Option<&str>) {
    if let Some(window) = app.get_webview_window("dashboard") {
        window.show().ok();
        window.set_focus().ok();
        // Navigate to the requested page within the existing dashboard
        if let Some(p) = page {
            app.emit("navigate-to-page", p).ok();
        }
    } else {
        // Create dashboard with optional page in URL hash
        let url = if let Some(p) = page {
            format!("index.html#dashboard/{}", p)
        } else {
            "index.html#dashboard".to_string()
        };
        WebviewWindowBuilder::new(app, "dashboard", WebviewUrl::App(url.into()))
            .title("MentaScribe")
            .inner_size(800.0, 600.0)
            .min_inner_size(640.0, 480.0)
            .resizable(true)
            .build()
            .ok();
    }
}

/// Reposition dictation window to the monitor where the mouse currently is.
/// Returns true if window was moved to a different monitor.
#[tauri::command]
fn reposition_to_mouse_monitor(app: tauri::AppHandle) -> Result<bool, String> {
    // Skip repositioning when widget is draggable (user controls position)
    let is_draggable = app.state::<AppState>().settings.lock()
        .map(|s| s.widget.draggable)
        .unwrap_or(false);
    if is_draggable {
        // Log once per second (this is called every 150ms, so ~7 calls/sec)
        // Use a simple static counter to throttle
        use std::sync::atomic::AtomicU64;
        static SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
        let count = SKIP_COUNT.fetch_add(1, Ordering::Relaxed);
        if count % 40 == 0 {
            eprintln!("[reposition] SKIPPED (draggable=true), skip count: {}", count);
        }
        return Ok(false);
    }

    let window = app.get_webview_window("dictation")
        .ok_or_else(|| "Dictation window not found".to_string())?;

    // Skip if window is not visible
    if !window.is_visible().unwrap_or(false) {
        return Ok(false);
    }

    // Use native AppKit positioning on macOS (bypasses tao's coordinate bugs)
    #[cfg(target_os = "macos")]
    {
        return native_position_on_cursor_monitor(&app, true);
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Non-macOS fallback using tao APIs
        let cursor_pos = window.cursor_position()
            .map_err(|e| format!("Failed to get cursor position: {}", e))?;
        let monitor = window.current_monitor().ok().flatten()
            .or_else(|| window.primary_monitor().ok().flatten())
            .ok_or_else(|| "No monitor found".to_string())?;
        let screen_pos = monitor.position();
        let screen_size = monitor.size();
        let current_pos = window.outer_position().unwrap_or(tauri::PhysicalPosition::new(0, 0));
        let actual_window_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(140, 48));
        let window_center_x = current_pos.x + actual_window_size.width as i32 / 2;
        let window_center_y = current_pos.y + actual_window_size.height as i32 / 2;
        let window_on_same_monitor =
            window_center_x >= screen_pos.x &&
            window_center_x < screen_pos.x + screen_size.width as i32 &&
            window_center_y >= screen_pos.y &&
            window_center_y < screen_pos.y + screen_size.height as i32;
        if !window_on_same_monitor {
            let scale = monitor.scale_factor();
            let pos = monitor.position();
            let size = monitor.size();
            // Use actual window size (frontend dynamically resizes to match pill)
            let ww = actual_window_size.width as i32;
            let wh = actual_window_size.height as i32;
            let doff = (DOCK_OFFSET * scale) as i32;
            let x = pos.x + (size.width as i32 - ww) / 2;
            let y = pos.y + size.height as i32 - wh - doff;
            window.set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|e| format!("Failed to set position: {}", e))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn toggle_dictation_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("dictation") {
        let is_visible = window.is_visible().unwrap_or(false);
        eprintln!("[toggle] toggle_dictation_window called, currently visible: {}", is_visible);

        if is_visible {
            eprintln!("[toggle] Hiding dictation window");
            window.hide().ok();
        } else {
            // Check if widget is draggable - if so, skip repositioning to preserve user's position
            let is_draggable = app.state::<AppState>().settings.lock()
                .map(|s| s.widget.draggable)
                .unwrap_or(false);
            eprintln!("[toggle] Showing dictation window, draggable={}", is_draggable);

            window.show().ok();
            // Re-apply panel settings after show (macOS may reset them)
            refresh_panel_settings(app);

            if !is_draggable {
                // Position on cursor's monitor after show (panel must exist)
                #[cfg(target_os = "macos")]
                {
                    eprintln!("[toggle] Repositioning to cursor monitor (draggable=false)");
                    match native_position_on_cursor_monitor(app, false) {
                        Ok(moved) => eprintln!("[toggle] Position result: moved={}", moved),
                        Err(e) => eprintln!("[toggle] Position ERROR: {}", e),
                    }
                }
            } else {
                eprintln!("[toggle] Skipping reposition (draggable=true, preserving user position)");
            }
        }
    } else {
        eprintln!("[toggle] ERROR: dictation window not found!");
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

            // Auto-detect CoreML: if use_coreml is None and platform supports it, enable
            let coreml_status = transcription::whisper::get_coreml_status();
            if loaded_settings.transcription.use_coreml.is_none() && coreml_status.supported {
                log::info!("CoreML supported on this platform (apple_silicon={}), auto-enabling", coreml_status.apple_silicon);
                let mut auto_settings = loaded_settings.clone();
                auto_settings.transcription.use_coreml = Some(true);
                settings::save_settings(&auto_settings).ok();
                // Update the managed state too
                if let Ok(mut s) = app_handle.state::<AppState>().settings.lock() {
                    s.transcription.use_coreml = Some(true);
                }
            }

            // Check which engine is configured and preload accordingly
            let use_voxtral_engine = is_voxtral_engine(&loaded_settings);

            if use_voxtral_engine {
                // Preload Voxtral model
                #[cfg(feature = "voxtral")]
                {
                    if transcription::voxtral::is_model_downloaded() {
                        let preload_app_handle = app_handle.clone();
                        std::thread::spawn(move || {
                            log::info!("Background preload: starting for Voxtral model");
                            preload_app_handle.emit("model-preload-start", "voxtral-mini-4b").ok();
                            let start = std::time::Instant::now();
                            match transcription::voxtral::preload_model() {
                                Ok(()) => {
                                    let elapsed = start.elapsed().as_secs_f64();
                                    log::info!("Background preload: Voxtral ready in {:.2}s", elapsed);
                                    preload_app_handle.emit("model-preload-complete", serde_json::json!({
                                        "model": "voxtral-mini-4b",
                                        "elapsed_secs": elapsed,
                                    })).ok();
                                }
                                Err(e) => {
                                    log::error!("Background preload: Voxtral failed: {}", e);
                                    preload_app_handle.emit("model-preload-error", serde_json::json!({
                                        "model": "voxtral-mini-4b",
                                        "error": e.to_string(),
                                    })).ok();
                                }
                            }
                        });
                    } else {
                        log::info!("Voxtral model not downloaded, emitting model-needs-download");
                        app_handle.emit("model-needs-download", "voxtral-mini-4b").ok();
                    }
                }
            } else {
                // Preload Whisper model (existing behavior)
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

                if model_downloaded {
                    let preload_model_size = configured_model.to_string();
                    let preload_app_handle = app_handle.clone();
                    std::thread::spawn(move || {
                        log::info!(
                            "Background preload: starting for model '{}'",
                            preload_model_size
                        );

                        // Download VAD model if not present (~2MB, fast)
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build();
                        if let Ok(rt) = rt {
                            if let Err(e) = rt.block_on(transcription::whisper::ensure_vad_model()) {
                                log::warn!("Failed to download VAD model: {} (VAD pre-filtering will be skipped)", e);
                            }
                        }

                        preload_app_handle.emit("model-preload-start", &preload_model_size).ok();

                        let start = std::time::Instant::now();
                        match transcription::whisper::preload_model(&preload_model_size) {
                            Ok(()) => {
                                let elapsed = start.elapsed().as_secs_f64();
                                log::info!(
                                    "Background preload: model '{}' ready in {:.2}s",
                                    preload_model_size,
                                    elapsed
                                );
                                preload_app_handle.emit("model-preload-complete", serde_json::json!({
                                    "model": &preload_model_size,
                                    "elapsed_secs": elapsed,
                                })).ok();
                            }
                            Err(e) => {
                                let elapsed = start.elapsed().as_secs_f64();
                                log::error!(
                                    "Background preload: failed for model '{}' after {:.2}s: {}",
                                    preload_model_size,
                                    elapsed,
                                    e
                                );
                                preload_app_handle.emit("model-preload-error", serde_json::json!({
                                    "model": &preload_model_size,
                                    "error": e.to_string(),
                                })).ok();
                            }
                        }
                    });
                }
            }

            // Show dictation window and convert to NSPanel
            if let Some(window) = app.get_webview_window("dictation") {
                window.show().ok();
            } else {
                println!("[window] setup: WARNING - dictation window not found");
            }

            // Convert dictation window to NSPanel on macOS for fullscreen overlay support
            // This MUST be done after the window is shown and rendered
            setup_dictation_panel(&app_handle);

            // Position at bottom-center of cursor's monitor (after panel exists)
            #[cfg(target_os = "macos")]
            {
                native_position_on_cursor_monitor(&app_handle, false).ok();
            }

            // Build tray menu (shown on right-click)
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let history_item = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide Widget", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&settings_item, &history_item, &toggle_item, &quit_item],
            )?;

            // Build tray icon — single click opens dashboard, right-click shows menu
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "settings" => {
                        open_dashboard_window(app, Some("settings"));
                    }
                    "history" => {
                        open_dashboard_window(app, Some("history"));
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
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        open_dashboard_window(tray.app_handle(), None);
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
            get_coreml_status,
            get_metal_status,
            download_coreml_model,
            delete_model,
            delete_coreml_model,
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
            start_native_drag,
            resize_pill,
            is_cursor_over_pill,
            // Voxtral
            get_voxtral_status,
            get_voxtral_models,
            download_voxtral_model,
            delete_voxtral_model,
            // Debug
            frontend_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
