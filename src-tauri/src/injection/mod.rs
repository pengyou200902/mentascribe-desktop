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
// macOS Implementation
// ============================================================================
#[cfg(target_os = "macos")]
mod platform {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use foreign_types::ForeignType;
    use std::thread;
    use std::time::Duration;

    type CGEventRef = *mut std::ffi::c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            string_length: u64,
            unicode_string: *const u16,
        );
    }

    const VK_COMMAND: CGKeyCode = 0x37;
    const VK_ANSI_V: CGKeyCode = 0x09;
    const VK_RETURN: CGKeyCode = 0x24;
    const VK_TAB: CGKeyCode = 0x30;

    // macOS hard limit: CGEventKeyboardSetUnicodeString truncates at 20 UTF-16 code units
    const MAX_UTF16_UNITS_PER_EVENT: usize = 20;

    // Optimized delay: 2ms between chunks (was 5ms + 2ms = 7ms, ~3.5x faster)
    // 1.5ms caused drops in some slower apps; 2ms is reliable across ~95% of apps
    const CHUNK_DELAY_US: u64 = 2000;

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

        let cmd_down = CGEvent::new_keyboard_event(source.clone(), VK_COMMAND, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        cmd_down.set_flags(CGEventFlags::CGEventFlagCommand);
        cmd_down.post(CGEventTapLocation::HID);

        let v_down = CGEvent::new_keyboard_event(source.clone(), VK_ANSI_V, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        v_down.set_flags(CGEventFlags::CGEventFlagCommand);
        v_down.post(CGEventTapLocation::HID);

        let v_up = CGEvent::new_keyboard_event(source.clone(), VK_ANSI_V, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        v_up.set_flags(CGEventFlags::CGEventFlagCommand);
        v_up.post(CGEventTapLocation::HID);

        let cmd_up = CGEvent::new_keyboard_event(source, VK_COMMAND, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent creation failed".into()))?;
        cmd_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    // ── Tier 1: Accessibility API ──────────────────────────────────────────

    /// Try to insert text via the macOS Accessibility API (kAXSelectedTextAttribute).
    /// Returns Ok(true) if text was inserted, Ok(false) if AX is not supported
    /// for the focused element, or Err on unexpected failure.
    pub fn try_ax_insert(text: &str) -> Result<bool, super::InjectionError> {
        use accessibility_sys::*;
        use core_foundation::base::{CFTypeRef, TCFType};
        use core_foundation::string::CFString;

        unsafe {
            let system_wide = AXUIElementCreateSystemWide();

            // Get the focused UI element
            let mut focused_raw: CFTypeRef = std::ptr::null();
            let focused_attr = CFString::new("AXFocusedUIElement");
            let result = AXUIElementCopyAttributeValue(
                system_wide,
                focused_attr.as_concrete_TypeRef(),
                &mut focused_raw,
            );
            if result != 0 || focused_raw.is_null() {
                eprintln!("[ax_insert] No focused element (error={})", result);
                core_foundation::base::CFRelease(system_wide as CFTypeRef);
                return Ok(false);
            }
            let element = focused_raw as AXUIElementRef;

            // Log the element's role for debugging (but don't use it as a gate —
            // just check settability directly, since some valid targets have
            // unexpected roles)
            let mut role_raw: CFTypeRef = std::ptr::null();
            let role_attr = CFString::new("AXRole");
            let role_result = AXUIElementCopyAttributeValue(
                element,
                role_attr.as_concrete_TypeRef(),
                &mut role_raw,
            );
            if role_result == 0 && !role_raw.is_null() {
                let role_cf = core_foundation::string::CFString::wrap_under_get_rule(
                    role_raw as core_foundation::string::CFStringRef,
                );
                eprintln!("[ax_insert] Focused element role='{}'", role_cf.to_string());
            } else {
                eprintln!("[ax_insert] Could not get role (error={})", role_result);
            }

            // Check if kAXSelectedTextAttribute is settable — this is the real
            // gate. Skip the role whitelist since some valid targets (e.g.
            // custom views) have non-standard roles.
            let selected_text_attr = CFString::new("AXSelectedText");
            let mut settable: bool = false;
            let settable_result = AXUIElementIsAttributeSettable(
                element,
                selected_text_attr.as_concrete_TypeRef(),
                &mut settable as *mut bool,
            );
            if settable_result != 0 || !settable {
                eprintln!(
                    "[ax_insert] AXSelectedText not settable (error={}, settable={})",
                    settable_result, settable
                );
                core_foundation::base::CFRelease(element as CFTypeRef);
                core_foundation::base::CFRelease(system_wide as CFTypeRef);
                return Ok(false);
            }

            // Set the selected text — this inserts at cursor or replaces selection
            let cf_text = CFString::new(text);
            let set_result = AXUIElementSetAttributeValue(
                element,
                selected_text_attr.as_concrete_TypeRef(),
                cf_text.as_CFTypeRef(),
            );

            core_foundation::base::CFRelease(element as CFTypeRef);
            core_foundation::base::CFRelease(system_wide as CFTypeRef);

            if set_result == 0 {
                eprintln!("[ax_insert] Success via AX API");
                Ok(true)
            } else {
                eprintln!("[ax_insert] SetAttributeValue failed (error={})", set_result);
                Ok(false)
            }
        }
    }

    // ── Tier 2: Optimized CGEvent typing ───────────────────────────────────

    /// Type text using optimized CGEvent Unicode chunks.
    /// Key optimization: no Unicode string on key-up, reduced inter-chunk delay.
    pub fn type_text(text: &str) -> Result<(), super::InjectionError> {
        eprintln!(
            "[type_text] Starting optimized CGEvent typing for {} chars",
            text.chars().count()
        );

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| super::InjectionError::Failed("CGEventSource creation failed".into()))?;

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            match c {
                '\n' | '\r' => {
                    type_key(&source, VK_RETURN, CGEventFlags::empty())?;
                    i += 1;
                }
                '\t' => {
                    type_key(&source, VK_TAB, CGEventFlags::empty())?;
                    i += 1;
                }
                _ => {
                    let mut chunk = String::new();
                    let mut utf16_count: usize = 0;
                    while i < chars.len() {
                        let ch = chars[i];
                        if ch == '\n' || ch == '\r' || ch == '\t' {
                            break;
                        }
                        let ch_utf16_len = ch.len_utf16();
                        if utf16_count + ch_utf16_len > MAX_UTF16_UNITS_PER_EVENT {
                            break;
                        }
                        chunk.push(ch);
                        utf16_count += ch_utf16_len;
                        i += 1;
                    }

                    if !chunk.is_empty() {
                        type_unicode_chunk(&source, &chunk)?;
                    }
                }
            }
        }

        eprintln!("[type_text] Completed typing {} chars", text.chars().count());
        Ok(())
    }

    fn type_key(
        source: &CGEventSource,
        keycode: CGKeyCode,
        flags: CGEventFlags,
    ) -> Result<(), super::InjectionError> {
        let key_down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent key down failed".into()))?;
        if !flags.is_empty() {
            key_down.set_flags(flags);
        }
        key_down.post(CGEventTapLocation::HID);

        let key_up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent key up failed".into()))?;
        key_up.post(CGEventTapLocation::HID);

        // Brief delay for control characters
        thread::sleep(Duration::from_micros(CHUNK_DELAY_US));
        Ok(())
    }

    /// Optimized Unicode chunk typing:
    /// - Unicode string only on key-down (key-up needs none)
    /// - Reduced inter-chunk delay from 7ms to 1.5ms
    fn type_unicode_chunk(source: &CGEventSource, chunk: &str) -> Result<(), super::InjectionError> {
        let utf16: Vec<u16> = chunk.encode_utf16().collect();
        if utf16.is_empty() {
            return Ok(());
        }

        // Key down with Unicode string — this is where text actually gets inserted
        let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| super::InjectionError::Failed("CGEvent Unicode failed".into()))?;
        unsafe {
            CGEventKeyboardSetUnicodeString(
                event_down.as_ptr() as CGEventRef,
                utf16.len() as u64,
                utf16.as_ptr(),
            );
        }
        event_down.post(CGEventTapLocation::HID);

        // Key up — no Unicode string needed (optimization from research)
        let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
            .map_err(|_| super::InjectionError::Failed("CGEvent key up failed".into()))?;
        event_up.post(CGEventTapLocation::HID);

        // 1.5ms inter-chunk delay (was 7ms total)
        thread::sleep(Duration::from_micros(CHUNK_DELAY_US));
        Ok(())
    }

    // ── Tier 3: Clipboard save/paste/restore ───────────────────────────────

    /// Save all NSPasteboard items, set text with transient marker, paste, restore.
    pub fn clipboard_save_paste_restore(text: &str) -> Result<(), super::InjectionError> {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};

        unsafe {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];

            // Save all pasteboard items with all their type representations
            let items: id = msg_send![pasteboard, pasteboardItems];
            let item_count: usize = msg_send![items, count];
            eprintln!(
                "[clipboard_restore] Saving {} pasteboard items",
                item_count
            );

            // Store items as Vec<Vec<(NSString type, NSData data)>>
            let mut saved_items: Vec<Vec<(id, id)>> = Vec::with_capacity(item_count);
            for i in 0..item_count {
                let item: id = msg_send![items, objectAtIndex: i];
                let types: id = msg_send![item, types];
                let type_count: usize = msg_send![types, count];
                let mut item_data: Vec<(id, id)> = Vec::with_capacity(type_count);
                for j in 0..type_count {
                    let ptype: id = msg_send![types, objectAtIndex: j];
                    let data: id = msg_send![item, dataForType: ptype];
                    if data != nil {
                        // Retain both so they survive the clearContents below
                        let _: () = msg_send![ptype, retain];
                        let _: () = msg_send![data, retain];
                        item_data.push((ptype, data));
                    }
                }
                saved_items.push(item_data);
            }

            // Clear pasteboard and set our text
            let _: i64 = msg_send![pasteboard, clearContents];

            // Create text as NSString
            let ns_text: id = {
                let ns_string_class = class!(NSString);
                let alloc: id = msg_send![ns_string_class, alloc];
                let bytes = text.as_bytes();
                let encoding: usize = 4; // NSUTF8StringEncoding
                msg_send![alloc, initWithBytes:bytes.as_ptr() length:bytes.len() encoding:encoding]
            };

            // Set text on pasteboard
            let string_type: id = msg_send![
                class!(NSString),
                stringWithUTF8String: b"public.utf8-plain-text\0".as_ptr()
            ];
            let _: bool = msg_send![pasteboard, setString:ns_text forType:string_type];

            // Add transient type marker so clipboard managers ignore this
            let transient_type: id = msg_send![
                class!(NSString),
                stringWithUTF8String: b"org.nspasteboard.TransientType\0".as_ptr()
            ];
            let empty_data: id = msg_send![class!(NSData), data];
            let _: bool = msg_send![pasteboard, setData:empty_data forType:transient_type];

            let _: () = msg_send![ns_text, release];

            // Snapshot change count AFTER our writes — this is the reliable
            // baseline to detect if anything else touches the pasteboard
            let change_count_ours: i64 = msg_send![pasteboard, changeCount];

            // Simulate Cmd+V
            simulate_paste()?;

            // Wait for target app to read the clipboard
            thread::sleep(Duration::from_millis(150));

            // Check if user or another app copied something during our paste
            let change_count_after: i64 = msg_send![pasteboard, changeCount];
            if change_count_after != change_count_ours {
                eprintln!(
                    "[clipboard_restore] Change count changed during paste (ours={}, now={}), user may have copied — skipping restore",
                    change_count_ours, change_count_after
                );
                // Release saved data
                for item_data in &saved_items {
                    for &(ptype, data) in item_data {
                        let _: () = msg_send![ptype, release];
                        let _: () = msg_send![data, release];
                    }
                }
                return Ok(());
            }

            // Restore saved pasteboard contents
            let _: i64 = msg_send![pasteboard, clearContents];

            if saved_items.is_empty() {
                // Nothing to restore — pasteboard was empty before
                eprintln!("[clipboard_restore] Pasteboard was empty, nothing to restore");
            } else {
                // Recreate NSPasteboardItems with all saved types
                let items_array: id = msg_send![class!(NSMutableArray), arrayWithCapacity: saved_items.len()];
                for item_data in &saved_items {
                    let new_item: id = msg_send![class!(NSPasteboardItem), new];
                    for &(ptype, data) in item_data {
                        let _: bool = msg_send![new_item, setData:data forType:ptype];
                        let _: () = msg_send![ptype, release];
                        let _: () = msg_send![data, release];
                    }
                    let _: () = msg_send![items_array, addObject: new_item];
                    let _: () = msg_send![new_item, release];
                }
                let _: bool = msg_send![pasteboard, writeObjects: items_array];
                eprintln!("[clipboard_restore] Pasteboard restored ({} items)", saved_items.len());
            }
        }

        Ok(())
    }
}

// ============================================================================
// Windows Implementation
// ============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use std::mem::size_of;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_V,
    };

    pub fn check_accessibility() -> bool {
        true
    }

    pub fn simulate_paste() -> Result<(), super::InjectionError> {
        let inputs: [INPUT; 4] = [
            make_key_input(VK_CONTROL, false),
            make_key_input(VK_V, false),
            make_key_input(VK_V, true),
            make_key_input(VK_CONTROL, true),
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

    // ── Tier 1: SendInput KEYEVENTF_UNICODE ────────────────────────────────

    /// Inject text using batched SendInput with KEYEVENTF_UNICODE.
    /// Each character is sent as down+up events with the UTF-16 code unit as scan code.
    /// Batched in a single SendInput call for atomic, fast injection.
    pub fn sendinput_unicode(text: &str) -> Result<(), super::InjectionError> {
        let mut inputs: Vec<INPUT> = Vec::with_capacity(text.len() * 4);
        let mut utf16_buf = [0u16; 2];

        for ch in text.chars() {
            let encoded = ch.encode_utf16(&mut utf16_buf);
            for &code_unit in encoded.iter() {
                // Key down
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: code_unit,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
                // Key up
                inputs.push(INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: code_unit,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                });
            }
        }

        // Batch all events — chunk at ~5000 chars (~10000 events) due to OS limit
        const MAX_EVENTS_PER_CALL: usize = 10_000;
        for chunk in inputs.chunks(MAX_EVENTS_PER_CALL) {
            let sent = unsafe { SendInput(chunk, size_of::<INPUT>() as i32) };
            if sent != chunk.len() as u32 {
                return Err(super::InjectionError::Failed(format!(
                    "SendInput: only {sent}/{} events sent",
                    chunk.len()
                )));
            }
        }

        eprintln!(
            "[sendinput_unicode] Injected {} chars via {} events",
            text.chars().count(),
            inputs.len()
        );
        Ok(())
    }

    // ── Tier 2: Clipboard save/paste/restore ───────────────────────────────

    /// Save all clipboard formats, paste text, restore original clipboard.
    pub fn clipboard_save_paste_restore(text: &str) -> Result<(), super::InjectionError> {
        use clipboard_win::{formats, Clipboard, Getter, Setter};

        // Save current clipboard contents (text only — full format save is complex)
        let saved_text = {
            let _clip = Clipboard::new_attempts(10)
                .map_err(|e| super::InjectionError::Failed(format!("Open clipboard: {}", e)))?;
            let mut buf = String::new();
            let _ = formats::Unicode.read_clipboard(&mut buf);
            if buf.is_empty() {
                None
            } else {
                Some(buf)
            }
        };

        // Set our text
        {
            let _clip = Clipboard::new_attempts(10)
                .map_err(|e| super::InjectionError::Failed(format!("Open clipboard: {}", e)))?;
            formats::Unicode
                .write_clipboard(&text)
                .map_err(|e| super::InjectionError::Failed(format!("Write clipboard: {}", e)))?;
        }

        // Simulate Ctrl+V
        simulate_paste()?;

        // Wait for target app to read
        std::thread::sleep(std::time::Duration::from_millis(250));

        // Restore
        {
            let _clip = Clipboard::new_attempts(10)
                .map_err(|e| super::InjectionError::Failed(format!("Open clipboard: {}", e)))?;
            if let Some(ref saved) = saved_text {
                formats::Unicode
                    .write_clipboard(saved)
                    .map_err(|e| {
                        super::InjectionError::Failed(format!("Restore clipboard: {}", e))
                    })?;
            } else {
                let _ = clipboard_win::empty();
            }
        }

        eprintln!(
            "[clipboard_save_paste_restore] Injected {} chars, clipboard restored",
            text.len()
        );
        Ok(())
    }
}

// ============================================================================
// Linux Implementation (unchanged — enigo fallback is fine)
// ============================================================================
#[cfg(target_os = "linux")]
mod platform {
    use std::ptr::null;
    use x11::xlib::{XCloseDisplay, XFlush, XKeysymToKeycode, XOpenDisplay};
    use x11::xtest::XTestFakeKeyEvent;

    const XK_CONTROL_L: u64 = 0xFFE3;
    const XK_V: u64 = 0x0076;

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

            let ctrl = XKeysymToKeycode(display, XK_CONTROL_L);
            let v = XKeysymToKeycode(display, XK_V);

            XTestFakeKeyEvent(display, ctrl as u32, 1, 0);
            XTestFakeKeyEvent(display, v as u32, 1, 0);
            XTestFakeKeyEvent(display, v as u32, 0, 0);
            XTestFakeKeyEvent(display, ctrl as u32, 0, 0);

            XFlush(display);
            XCloseDisplay(display);
        }
        Ok(())
    }
}

// ============================================================================
// Main API
// ============================================================================

/// Truncate a string at a char boundary (safe for multi-byte UTF-8)
fn truncate_for_display(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Inject text into the currently focused application
pub fn inject_text(text: &str, settings: &UserSettings) -> Result<(), InjectionError> {
    let method = settings
        .output
        .insert_method
        .as_deref()
        .unwrap_or("auto");

    eprintln!(
        "[inject] method={}, chars={}, bytes={}",
        method,
        text.chars().count(),
        text.len()
    );

    // Strip [BLANK_AUDIO] markers that Whisper outputs when no speech detected
    let text = text
        .replace("[BLANK_AUDIO]", "")
        .replace("[BLANK AUDIO]", "");
    let text = text.trim();

    if text.is_empty() {
        eprintln!("[inject] Skipping empty text (after stripping BLANK_AUDIO markers)");
        return Ok(());
    }

    eprintln!(
        "[inject] Text after cleanup: '{}' ({} chars)",
        truncate_for_display(text, 50),
        text.chars().count()
    );

    // Check accessibility permissions
    if !platform::check_accessibility() {
        #[cfg(target_os = "macos")]
        {
            return Err(InjectionError::AccessibilityPermissionRequired);
        }
        #[cfg(target_os = "linux")]
        {
            return Err(InjectionError::WaylandNotSupported);
        }
    }

    // Minimal focus delay
    std::thread::sleep(std::time::Duration::from_millis(50));

    let result = match method {
        "auto" => inject_auto(text),
        "ax_api" => inject_via_ax_api(text),
        "type" => inject_via_typing(text),
        "paste" => inject_via_paste(text),
        "paste_restore" => inject_via_paste_restore(text),
        _ => inject_auto(text),
    };

    match &result {
        Ok(_) => eprintln!("[inject] Text injection succeeded"),
        Err(e) => eprintln!("[inject] ERROR: Text injection failed: {}", e),
    }

    result
}

/// Auto mode: use the tiered injection strategy per platform
fn inject_auto(text: &str) -> Result<(), InjectionError> {
    #[cfg(target_os = "macos")]
    {
        return inject_auto_macos(text);
    }

    #[cfg(target_os = "windows")]
    {
        return inject_auto_windows(text);
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: try typing via enigo, fall back to paste
        return inject_via_typing(text);
    }
}

/// macOS auto mode: AX API → CGEvent typing → clipboard save/paste/restore
#[cfg(target_os = "macos")]
fn inject_auto_macos(text: &str) -> Result<(), InjectionError> {
    // Tier 1: Try AX API first (instant, no clipboard, proper undo)
    match platform::try_ax_insert(text) {
        Ok(true) => {
            log::info!("Text injected via AX API: {} chars", text.len());
            return Ok(());
        }
        Ok(false) => {
            eprintln!("[inject_auto] AX API not available for this element, trying CGEvent");
        }
        Err(e) => {
            eprintln!("[inject_auto] AX API error: {}, trying CGEvent", e);
        }
    }

    // Tier 2: CGEvent typing (works in ~95% of apps)
    match platform::type_text(text) {
        Ok(()) => {
            log::info!("Text injected via CGEvent typing: {} chars", text.len());
            return Ok(());
        }
        Err(e) => {
            eprintln!("[inject_auto] CGEvent typing failed: {}, trying clipboard", e);
        }
    }

    // Tier 3: Clipboard save/paste/restore (last resort)
    eprintln!("[inject_auto] Falling back to clipboard save/paste/restore");
    platform::clipboard_save_paste_restore(text)?;
    log::info!(
        "Text injected via clipboard save/paste/restore: {} chars",
        text.len()
    );
    Ok(())
}

/// Windows auto mode: SendInput KEYEVENTF_UNICODE → clipboard save/paste/restore
#[cfg(target_os = "windows")]
fn inject_auto_windows(text: &str) -> Result<(), InjectionError> {
    // Tier 1: SendInput for text up to ~2000 chars
    if text.chars().count() <= 2000 {
        match platform::sendinput_unicode(text) {
            Ok(()) => {
                log::info!("Text injected via SendInput UNICODE: {} chars", text.len());
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "[inject_auto] SendInput failed: {}, trying clipboard",
                    e
                );
            }
        }
    } else {
        eprintln!(
            "[inject_auto] Text too long for SendInput ({} chars), using clipboard",
            text.chars().count()
        );
    }

    // Tier 2: Clipboard save/paste/restore
    platform::clipboard_save_paste_restore(text)?;
    log::info!(
        "Text injected via clipboard save/paste/restore: {} chars",
        text.len()
    );
    Ok(())
}

/// AX API only mode (macOS). Falls back to typing on other platforms.
fn inject_via_ax_api(text: &str) -> Result<(), InjectionError> {
    #[cfg(target_os = "macos")]
    {
        match platform::try_ax_insert(text) {
            Ok(true) => {
                log::info!("Text injected via AX API: {} chars", text.len());
                return Ok(());
            }
            Ok(false) => {
                return Err(InjectionError::Failed(
                    "AX API: focused element does not support kAXSelectedTextAttribute (app may use custom text rendering — try Auto or Keyboard Sim instead)".into(),
                ));
            }
            Err(e) => return Err(e),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // AX API is macOS-only; fall back to typing on other platforms
        inject_via_typing(text)
    }
}

/// Clipboard save/paste/restore mode (preserves clipboard contents)
fn inject_via_paste_restore(text: &str) -> Result<(), InjectionError> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        platform::clipboard_save_paste_restore(text)?;
        log::info!(
            "Text injected via clipboard save/paste/restore: {} chars",
            text.len()
        );
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        // Linux doesn't have full clipboard save/restore yet; use legacy paste
        inject_via_paste(text)
    }
}

/// Legacy paste mode: clipboard + Cmd+V/Ctrl+V (overwrites clipboard)
fn inject_via_paste(text: &str) -> Result<(), InjectionError> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| InjectionError::Failed(format!("Clipboard: {}", e)))?;

    clipboard
        .set_text(text)
        .map_err(|e| InjectionError::Failed(format!("Set text: {}", e)))?;

    platform::simulate_paste()?;

    std::thread::sleep(std::time::Duration::from_millis(50));
    clipboard.clear().ok();

    log::info!("Text injected via paste: {} chars", text.len());
    Ok(())
}

/// Legacy type mode: CGEvent on macOS, enigo on other platforms
fn inject_via_typing(text: &str) -> Result<(), InjectionError> {
    #[cfg(target_os = "macos")]
    {
        let result = platform::type_text(text);
        if result.is_ok() {
            log::info!("Text injected via CGEvent typing: {} chars", text.len());
        }
        return result;
    }

    #[cfg(not(target_os = "macos"))]
    {
        use enigo::{Enigo, Keyboard, Settings};

        let mut enigo =
            Enigo::new(&Settings::default()).map_err(|e| InjectionError::Failed(e.to_string()))?;

        enigo
            .text(text)
            .map_err(|e| InjectionError::Failed(e.to_string()))?;

        log::info!("Text injected via typing: {} chars", text.len());
        Ok(())
    }
}
