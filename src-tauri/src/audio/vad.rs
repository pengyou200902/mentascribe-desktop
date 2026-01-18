//! Voice Activity Detection (VAD)
//!
//! Simple energy-based VAD for detecting speech in audio.
//! For production, consider using Silero VAD or WebRTC VAD.

/// Configuration for VAD
pub struct VadConfig {
    /// Energy threshold for speech detection (0.0 - 1.0)
    pub energy_threshold: f32,
    /// Minimum speech duration in samples
    pub min_speech_samples: usize,
    /// Silence duration to end speech segment
    pub silence_samples: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.01,
            min_speech_samples: 1600, // 100ms at 16kHz
            silence_samples: 4800,    // 300ms at 16kHz
        }
    }
}

/// Simple energy-based voice activity detection
pub struct VoiceActivityDetector {
    config: VadConfig,
    is_speaking: bool,
    silence_count: usize,
    speech_count: usize,
}

impl VoiceActivityDetector {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            is_speaking: false,
            silence_count: 0,
            speech_count: 0,
        }
    }

    /// Process a chunk of audio samples and return whether speech is detected
    pub fn process(&mut self, samples: &[f32]) -> bool {
        let energy = calculate_energy(samples);
        let is_speech = energy > self.config.energy_threshold;

        if is_speech {
            self.speech_count += samples.len();
            self.silence_count = 0;

            if self.speech_count >= self.config.min_speech_samples {
                self.is_speaking = true;
            }
        } else {
            self.silence_count += samples.len();

            if self.silence_count >= self.config.silence_samples {
                self.is_speaking = false;
                self.speech_count = 0;
            }
        }

        self.is_speaking
    }

    /// Reset the detector state
    pub fn reset(&mut self) {
        self.is_speaking = false;
        self.silence_count = 0;
        self.speech_count = 0;
    }

    /// Check if currently detecting speech
    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }
}

/// Calculate RMS energy of audio samples
fn calculate_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Trim silence from the beginning and end of audio
pub fn trim_silence(samples: &[f32], threshold: f32, frame_size: usize) -> &[f32] {
    let mut start = 0;
    let mut end = samples.len();

    // Find start
    for (i, chunk) in samples.chunks(frame_size).enumerate() {
        if calculate_energy(chunk) > threshold {
            start = i * frame_size;
            break;
        }
    }

    // Find end
    for (i, chunk) in samples.rchunks(frame_size).enumerate() {
        if calculate_energy(chunk) > threshold {
            end = samples.len() - (i * frame_size);
            break;
        }
    }

    if start < end {
        &samples[start..end]
    } else {
        samples
    }
}
