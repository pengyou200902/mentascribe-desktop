use crate::settings::UserSettings;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InjectionError {
    #[error("Text injection failed: {0}")]
    Failed(String),
    #[error("Accessibility permission required. Go to System Settings > Privacy & Security > Accessibility")]
    AccessibilityPermissionRequired,
    #[error("X11 display not available. Wayland is not yet supported.")]
    WaylandNotSupported,
}

// ============================================================================
// macOS Implementation (CGEventPost via CoreGraphics)
// ============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use foreign_types::ForeignType;
    use std::thread;
    use std::time::Duration;

    // CoreGraphics type for CGEventRef
    type CGEventRef = *mut std::ffi::c_void;

    // Link to CoreGraphics framework for additional functions
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            string_length: u64,
            unicode_string: *const u16,
        );
    }

    const VK_COMMAND: CGKeyCode = 0x37;
    #[allow(dead_code)]
    const VK_SHIFT: CGKeyCode = 0x38;
    const VK_ANSI_V: CGKeyCode = 0x09;
    const VK_RETURN: CGKeyCode = 0x24;
    const VK_TAB: CGKeyCode = 0x30;
    #[allow(dead_code)]
    const VK_SPACE: CGKeyCode = 0x31;

    // Maximum characters per CGEvent Unicode string (macOS limitation)
    // CGEventKeyboardSetUnicodeString truncates at 20 characters
    const MAX_UNICODE_CHARS_PER_EVENT: usize = 20;

    // Delay between key events in microseconds
    // 2ms provides good balance between speed and reliability across different apps
    const KEY_DELAY_US: u64 = 2000;

    // Delay between chunks of text (slightly longer for app processing)
    const CHUNK_DELAY_US: u64 = 5000; // 5ms between chunks

    pub fn check_accessibility() -> bool {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }

    pub fn simulate_paste() -> Result<(), super::InjectionError> {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| super::InjectionError::Failed("CGEventSource creation failed".into()))?;

        // Cmd down
        let cmd_down = CGEvent::new_keyboard_event(source.clone(), VK_COMMAND, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        cmd_down.set_flags(CGEventFlags::CGEventFlagCommand);
        cmd_down.post(CGEventTapLocation::HID);

        // V down with Cmd modifier
        let v_down = CGEvent::new_keyboard_event(source.clone(), VK_ANSI_V, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        v_down.set_flags(CGEventFlags::CGEventFlagCommand);
        v_down.post(CGEventTapLocation::HID);

        // V up
        let v_up = CGEvent::new_keyboard_event(source.clone(), VK_ANSI_V, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        v_up.set_flags(CGEventFlags::CGEventFlagCommand);
        v_up.post(CGEventTapLocation::HID);

        // Cmd up
        let cmd_up = CGEvent::new_keyboard_event(source, VK_COMMAND, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        cmd_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    /// Type text using native macOS CGEvent with proper Unicode support
    pub fn type_text(text: &str) -> Result<(), super::InjectionError> {
        eprintln!("[type_text] Starting native macOS typing for {} chars", text.len());

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| super::InjectionError::Failed("CGEventSource creation failed".into()))?;

        // Process text character by character, handling special characters
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // Handle special control characters with dedicated key events
            match c {
                '\n' | '\r' => {
                    eprintln!("[type_text] Typing newline");
                    type_key(&source, VK_RETURN, CGEventFlags::empty())?;
                    i += 1;
                }
                '\t' => {
                    eprintln!("[type_text] Typing tab");
                    type_key(&source, VK_TAB, CGEventFlags::empty())?;
                    i += 1;
                }
                _ => {
                    // Collect a chunk of regular characters (up to MAX_UNICODE_CHARS_PER_EVENT)
                    // Stop at special characters that need individual handling
                    let mut chunk = String::new();
                    while i < chars.len() && chunk.chars().count() < MAX_UNICODE_CHARS_PER_EVENT {
                        let ch = chars[i];
                        if ch == '\n' || ch == '\r' || ch == '\t' {
                            break;
                        }
                        chunk.push(ch);
                        i += 1;
                    }

                    if !chunk.is_empty() {
                        eprintln!("[type_text] Typing chunk: '{}' ({} chars)",
                            if chunk.len() > 30 { &chunk[..30] } else { &chunk },
                            chunk.chars().count());
                        type_unicode_chunk(&source, &chunk)?;
                    }
                }
            }
            // Note: delays are handled within type_key and type_unicode_chunk
        }

        eprintln!("[type_text] Completed typing {} chars", text.len());
        Ok(())
    }

    /// Type a single key with optional modifiers
    fn type_key(source: &CGEventSource, keycode: CGKeyCode, flags: CGEventFlags) -> Result<(), super::InjectionError> {
        // Key down
        let key_down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .map_err(|_| super::InjectionError::Failed(format!("CGEvent key down failed for keycode {}", keycode)))?;
        if !flags.is_empty() {
            key_down.set_flags(flags);
        }
        key_down.post(CGEventTapLocation::HID);

        thread::sleep(Duration::from_micros(KEY_DELAY_US));

        // Key up
        let key_up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .map_err(|_| super::InjectionError::Failed(format!("CGEvent key up failed for keycode {}", keycode)))?;
        key_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    /// Type a chunk of Unicode text using CGEventKeyboardSetUnicodeString
    fn type_unicode_chunk(source: &CGEventSource, chunk: &str) -> Result<(), super::InjectionError> {
        // Convert to UTF-16 for macOS
        let utf16: Vec<u16> = chunk.encode_utf16().collect();

        if utf16.is_empty() {
            return Ok(());
        }

        eprintln!("[type_unicode_chunk] Sending {} UTF-16 units for '{}...'",
            utf16.len(), if chunk.len() > 10 { &chunk[..10] } else { chunk });

        // Create a keyboard event with keycode 0 (we'll set the Unicode string)
        // Using keycode 0 with Unicode string is the standard approach for text input
        let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation for Unicode failed".into()))?;

        // Set the Unicode string on the event
        // This is the key to making Unicode text input work on macOS
        unsafe {
            CGEventKeyboardSetUnicodeString(
                event_down.as_ptr() as CGEventRef,
                utf16.len() as u64,
                utf16.as_ptr(),
            );
        }

        // Post to HID for broader compatibility (works with most applications)
        event_down.post(CGEventTapLocation::HID);

        thread::sleep(Duration::from_micros(KEY_DELAY_US));

        // Key up event (also with Unicode string for consistency)
        let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent key up for Unicode failed".into()))?;

        // Set same Unicode string on key-up for completeness
        unsafe {
            CGEventKeyboardSetUnicodeString(
                event_up.as_ptr() as CGEventRef,
                utf16.len() as u64,
                utf16.as_ptr(),
            );
        }

        event_up.post(CGEventTapLocation::HID);

        // Allow time for the application to process the chunk
        thread::sleep(Duration::from_micros(CHUNK_DELAY_US));

        Ok(())
    }
}

// ============================================================================
// Windows Implementation (SendInput via Win32 API)
// ============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use std::mem::size_of;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        VIRTUAL_KEY, VK_CONTROL, VK_V,
    };

    pub fn check_accessibility() -> bool {
        true // No special permissions on Windows
    }

    pub fn simulate_paste() -> Result<(), super::InjectionError> {
        let inputs: [INPUT; 4] = [
            make_key_input(VK_CONTROL, false), // Ctrl down
            make_key_input(VK_V, false),       // V down
            make_key_input(VK_V, true),        // V up
            make_key_input(VK_CONTROL, true),  // Ctrl up
        ];

        let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
        if sent != 4 {
            return Err(super::InjectionError::Failed(format!(
                "SendInput: {} of 4 events sent",
                sent
            )));
        }
        Ok(())
    }

    fn make_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if key_up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS(0)
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

// ============================================================================
// Linux Implementation (XTest via X11)
// ============================================================================
#[cfg(target_os = "linux")]
mod platform {
    use std::ptr::null;
    use x11::xlib::{XCloseDisplay, XFlush, XKeysymToKeycode, XOpenDisplay};
    use x11::xtest::XTestFakeKeyEvent;

    const XK_Control_L: u64 = 0xFFE3;
    const XK_v: u64 = 0x0076;

    pub fn check_accessibility() -> bool {
        !is_wayland()
    }

    fn is_wayland() -> bool {
        std::env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false)
            || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    pub fn simulate_paste() -> Result<(), super::InjectionError> {
        if is_wayland() {
            return Err(super::InjectionError::WaylandNotSupported);
        }

        unsafe {
            let display = XOpenDisplay(null());
            if display.is_null() {
                return Err(super::InjectionError::Failed(
                    "Failed to open X display".into(),
                ));
            }

            let ctrl = XKeysymToKeycode(display, XK_Control_L);
            let v = XKeysymToKeycode(display, XK_v);

            XTestFakeKeyEvent(display, ctrl as u32, 1, 0); // Ctrl down
            XTestFakeKeyEvent(display, v as u32, 1, 0); // V down
            XTestFakeKeyEvent(display, v as u32, 0, 0); // V up
            XTestFakeKeyEvent(display, ctrl as u32, 0, 0); // Ctrl up

            XFlush(display);
            XCloseDisplay(display);
        }
        Ok(())
    }
}

// ============================================================================
// Main API
// ============================================================================

/// Inject text into the currently focused application
pub fn inject_text(text: &str, settings: &UserSettings) -> Result<(), InjectionError> {
    let method = settings
        .output
        .insert_method
        .as_deref()
        .unwrap_or("paste");

    eprintln!("[inject] method={}, len={}", method, text.len());

    // Strip [BLANK_AUDIO] markers that Whisper outputs when no speech detected
    let text = text
        .replace("[BLANK_AUDIO]", "")
        .replace("[BLANK AUDIO]", "");
    let text = text.trim();

    // Skip empty or whitespace-only text
    if text.is_empty() {
        eprintln!("[inject] Skipping empty text (after stripping BLANK_AUDIO markers)");
        return Ok(());
    }

    eprintln!("[inject] Text after cleanup: '{}' ({} chars)",
        if text.len() > 50 { &text[..50] } else { text }, text.len());

    // Check accessibility permissions
    if !platform::check_accessibility() {
        #[cfg(target_os = "macos")]
        {
            eprintln!("[inject] ERROR: Accessibility permissions not granted");
            return Err(InjectionError::AccessibilityPermissionRequired);
        }
        #[cfg(target_os = "linux")]
        {
            eprintln!("[inject] ERROR: Wayland not supported");
            return Err(InjectionError::WaylandNotSupported);
        }
    }

    // Minimal focus delay (reduced from 300ms to 50ms)
    std::thread::sleep(std::time::Duration::from_millis(50));

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

/// Inject text via clipboard paste using native platform APIs
fn inject_via_paste(text: &str) -> Result<(), InjectionError> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| InjectionError::Failed(format!("Clipboard: {}", e)))?;

    clipboard
        .set_text(text)
        .map_err(|e| InjectionError::Failed(format!("Set text: {}", e)))?;

    // Simulate paste using native platform API (no delay needed - clipboard is synchronous)
    platform::simulate_paste()?;

    // Brief delay before clearing (reduced from 500ms to 50ms)
    std::thread::sleep(std::time::Duration::from_millis(50));

    clipboard.clear().ok();
    eprintln!("[inject] Clipboard cleared");

    log::info!("Text injected via paste: {} chars", text.len());
    Ok(())
}

/// Inject text by simulating keyboard input
fn inject_via_typing(text: &str) -> Result<(), InjectionError> {
    eprintln!("[inject_via_typing] Starting type injection for {} chars", text.len());

    // On macOS, use our native CGEvent implementation for better Unicode support
    #[cfg(target_os = "macos")]
    {
        eprintln!("[inject_via_typing] Using native macOS CGEvent implementation");
        let result = platform::type_text(text);
        if result.is_ok() {
            log::info!("Text injected via native macOS typing: {} chars", text.len());
        }
        return result;
    }

    // On other platforms, fall back to enigo
    #[cfg(not(target_os = "macos"))]
    {
        use enigo::{Enigo, Keyboard, Settings};

        eprintln!("[inject_via_typing] Using enigo for text injection");
        let mut enigo =
            Enigo::new(&Settings::default()).map_err(|e| InjectionError::Failed(e.to_string()))?;

        enigo
            .text(text)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;

        log::info!("Text injected via typing: {} chars", text.len());
        Ok(())
    }
}
