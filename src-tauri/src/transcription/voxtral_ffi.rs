//! FFI bindings for voxtral.c — safe Rust wrappers around the C API.
//!
//! Only compiled when `voxtral` feature is enabled.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};
use std::ptr;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Opaque C types
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct VoxCtx {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct VoxStream {
    _opaque: [u8; 0],
}

// ---------------------------------------------------------------------------
// Raw extern "C" bindings
// ---------------------------------------------------------------------------

extern "C" {
    // Metal GPU initialization (must be called before vox_load on Apple Silicon)
    #[cfg(target_os = "macos")]
    pub fn vox_metal_init() -> c_int;
    #[cfg(target_os = "macos")]
    pub fn vox_metal_available() -> c_int;

    // Lifecycle
    pub fn vox_load(model_dir: *const c_char) -> *mut VoxCtx;
    pub fn vox_free(ctx: *mut VoxCtx);
    pub fn vox_set_delay(ctx: *mut VoxCtx, delay_ms: c_int);

    // Streaming
    pub fn vox_stream_init(ctx: *mut VoxCtx) -> *mut VoxStream;
    pub fn vox_stream_feed(s: *mut VoxStream, samples: *const c_float, n_samples: c_int) -> c_int;
    pub fn vox_stream_finish(s: *mut VoxStream) -> c_int;
    pub fn vox_stream_get(s: *mut VoxStream, out_tokens: *mut *const c_char, max: c_int) -> c_int;
    pub fn vox_stream_flush(s: *mut VoxStream) -> c_int;
    pub fn vox_stream_force_encode(s: *mut VoxStream) -> c_int;
    pub fn vox_stream_set_continuous(s: *mut VoxStream, enable: c_int);
    pub fn vox_set_processing_interval(s: *mut VoxStream, seconds: c_float);
    pub fn vox_stream_free(s: *mut VoxStream);

    // One-shot convenience
    pub fn vox_transcribe_audio(
        ctx: *mut VoxCtx,
        samples: *const c_float,
        n_samples: c_int,
    ) -> *mut c_char;
}

// ---------------------------------------------------------------------------
// Safe wrapper: VoxtralContext
// ---------------------------------------------------------------------------

/// Safe wrapper around a loaded voxtral model context.
/// Owns the C-allocated `vox_ctx_t` and frees it on drop.
pub struct VoxtralContext {
    ptr: *mut VoxCtx,
}

// SAFETY: vox_ctx_t has no thread-local state; all access in our code
// is behind a Mutex. The C library itself does not use thread-local storage.
unsafe impl Send for VoxtralContext {}
unsafe impl Sync for VoxtralContext {}

impl VoxtralContext {
    /// Load a voxtral model from the given directory.
    /// The directory must contain `consolidated.safetensors`, `tekken.json`, and `params.json`.
    pub fn load(model_dir: &str) -> Result<Self, String> {
        // Initialize Metal GPU before loading the model.
        // This must be called once before vox_load() so the library uses GPU
        // acceleration instead of falling back to CPU-only BLAS.
        #[cfg(target_os = "macos")]
        {
            let metal_ok = unsafe { vox_metal_init() };
            let metal_avail = unsafe { vox_metal_available() };
            eprintln!(
                "[voxtral] Metal init: {} (available: {})",
                if metal_ok != 0 { "OK" } else { "FAILED" },
                metal_avail != 0
            );
        }

        let c_path = CString::new(model_dir)
            .map_err(|e| format!("Invalid model path: {}", e))?;
        let ptr = unsafe { vox_load(c_path.as_ptr()) };
        if ptr.is_null() {
            return Err(format!("vox_load failed for directory: {}", model_dir));
        }
        Ok(Self { ptr })
    }

    /// Set the transcription delay in milliseconds (80–2400, default 480).
    pub fn set_delay(&self, delay_ms: i32) {
        unsafe { vox_set_delay(self.ptr, delay_ms as c_int) }
    }

    /// One-shot transcribe raw audio samples (mono, 16kHz, f32 [-1,1]).
    /// Returns the transcribed text.
    pub fn transcribe_audio(&self, samples: &[f32]) -> Result<String, String> {
        let result = unsafe {
            vox_transcribe_audio(self.ptr, samples.as_ptr(), samples.len() as c_int)
        };
        if result.is_null() {
            return Err("vox_transcribe_audio returned null".to_string());
        }
        let text = unsafe { CStr::from_ptr(result) }
            .to_string_lossy()
            .into_owned();
        // The C function returns a malloc'd string — we must free it
        unsafe { libc::free(result as *mut libc::c_void) };
        Ok(text)
    }

    /// Create a new streaming context. The returned stream borrows
    /// the model context (the C stream holds an internal pointer to it).
    pub fn stream_init(&self) -> Result<VoxtralStream, String> {
        let ptr = unsafe { vox_stream_init(self.ptr) };
        if ptr.is_null() {
            return Err("vox_stream_init failed".to_string());
        }
        Ok(VoxtralStream { ptr })
    }

    /// Get the raw pointer (for advanced use only).
    pub fn as_ptr(&self) -> *mut VoxCtx {
        self.ptr
    }
}

impl Drop for VoxtralContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { vox_free(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

// ---------------------------------------------------------------------------
// Safe wrapper: VoxtralStream
// ---------------------------------------------------------------------------

/// Safe wrapper around a voxtral streaming transcription context.
/// Must not outlive the `VoxtralContext` that created it.
pub struct VoxtralStream {
    ptr: *mut VoxStream,
}

// SAFETY: Same reasoning as VoxtralContext — no thread-local state,
// and we ensure single-threaded access via Mutex in the caller.
unsafe impl Send for VoxtralStream {}

impl VoxtralStream {
    /// Feed audio samples to the stream (mono, 16kHz, f32 [-1,1]).
    /// Returns Ok(()) on success.
    pub fn feed(&self, samples: &[f32]) -> Result<(), String> {
        let ret = unsafe {
            vox_stream_feed(self.ptr, samples.as_ptr(), samples.len() as c_int)
        };
        if ret < 0 {
            return Err("vox_stream_feed failed".to_string());
        }
        Ok(())
    }

    /// Retrieve pending decoded tokens. Returns a Vec of token strings.
    pub fn get_tokens(&self, max: usize) -> Vec<String> {
        let mut ptrs: Vec<*const c_char> = vec![ptr::null(); max];
        let count = unsafe {
            vox_stream_get(self.ptr, ptrs.as_mut_ptr(), max as c_int)
        };
        let mut tokens = Vec::with_capacity(count as usize);
        for i in 0..(count as usize) {
            if !ptrs[i].is_null() {
                let s = unsafe { CStr::from_ptr(ptrs[i]) }
                    .to_string_lossy()
                    .into_owned();
                tokens.push(s);
            }
        }
        tokens
    }

    /// Force the encoder to process whatever audio is buffered (with right-padding).
    pub fn flush(&self) -> Result<(), String> {
        let ret = unsafe { vox_stream_flush(self.ptr) };
        if ret < 0 {
            return Err("vox_stream_flush failed".to_string());
        }
        Ok(())
    }

    /// Force the encoder to process accumulated mel frames WITHOUT adding
    /// right-padding silence. Cheaper than flush() — used periodically during
    /// recording to keep the encoder/decoder current.
    pub fn force_encode(&self) -> Result<(), String> {
        let ret = unsafe { vox_stream_force_encode(self.ptr) };
        if ret < 0 {
            return Err("vox_stream_force_encode failed".to_string());
        }
        Ok(())
    }

    /// Signal end of audio and process remaining data.
    pub fn finish(&self) -> Result<(), String> {
        let ret = unsafe { vox_stream_finish(self.ptr) };
        if ret < 0 {
            return Err("vox_stream_finish failed".to_string());
        }
        Ok(())
    }

    /// Set the processing interval (seconds between encoder runs).
    pub fn set_processing_interval(&self, seconds: f32) {
        unsafe { vox_set_processing_interval(self.ptr, seconds) }
    }

    /// Enable continuous mode for live/infinite streams.
    pub fn set_continuous(&self, enable: bool) {
        unsafe { vox_stream_set_continuous(self.ptr, if enable { 1 } else { 0 }) }
    }
}

impl Drop for VoxtralStream {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { vox_stream_free(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}
