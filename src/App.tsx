import { useEffect, useState, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { Dashboard } from './components/dashboard/Dashboard';
import { useStore } from './lib/store';
import {
  MAX_HISTORY_ENTRIES, MIC_ERROR_TIMEOUT_MS, ERROR_TIMEOUT_MS,
  MODEL_PRELOAD_ERROR_TIMEOUT_MS, MODEL_DOWNLOAD_ERROR_TIMEOUT_MS,
  MONITOR_POLL_INTERVAL_MS, MONITOR_LOG_FREQUENCY,
  DEFAULT_HOTKEY_LABEL, DEFAULT_HOTKEY_MODE, DEFAULT_WIDGET_OPACITY,
} from './config/widget';

type WindowType = 'dictation' | 'dashboard';

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [isDownloadingModel, setIsDownloadingModel] = useState(false);
  const [isPreloading, setIsPreloading] = useState(false);
  const { settings, loadSettings } = useStore();

  // Use refs to avoid stale closures in event listeners
  const isRecordingRef = useRef(isRecording);
  const isProcessingRef = useRef(isProcessing);
  const settingsRef = useRef(settings);

  // Helper to save transcription to history
  const saveToHistory = useCallback((text: string) => {
    try {
      const stored = localStorage.getItem('transcription-history');
      const history = stored ? JSON.parse(stored) : [];
      history.unshift({
        id: crypto.randomUUID(),
        text,
        timestamp: new Date().toISOString(),
      });
      // Keep only last 100 entries
      const trimmed = history.slice(0, MAX_HISTORY_ENTRIES);
      localStorage.setItem('transcription-history', JSON.stringify(trimmed));
    } catch (e) {
      console.error('Failed to save to history:', e);
    }
  }, []);

  // Keep refs in sync with state
  useEffect(() => {
    isRecordingRef.current = isRecording;
  }, [isRecording]);

  useEffect(() => {
    isProcessingRef.current = isProcessing;
  }, [isProcessing]);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  // Determine which window type we're in based on URL hash
  const getWindowType = (): WindowType => {
    const hash = window.location.hash.slice(1);
    if (hash === 'dashboard' || hash.startsWith('dashboard')) return 'dashboard';
    return 'dictation';
  };

  const [windowType] = useState<WindowType>(getWindowType);

  const startRecording = useCallback(async () => {
    if (isRecordingRef.current || isProcessingRef.current) {
      console.log('Already recording or processing, skipping start');
      return;
    }
    // Set ref immediately to prevent duplicate calls during await
    isRecordingRef.current = true;
    try {
      console.log('Starting recording...');
      await invoke('start_recording');
      setIsRecording(true);
      console.log('Recording started');
    } catch (error) {
      // Reset ref on error
      isRecordingRef.current = false;
      console.error('Failed to start recording:', error);
      // Show brief error so user gets feedback (clears quickly)
      setError('Mic busy — try again');
      setTimeout(() => setError(null), MIC_ERROR_TIMEOUT_MS);
    }
  }, []);

  const stopRecording = useCallback(async () => {
    if (!isRecordingRef.current) {
      console.log('Not recording, skipping stop');
      return;
    }
    // Set refs immediately to prevent duplicate calls during await
    isRecordingRef.current = false;
    isProcessingRef.current = true;
    try {
      console.log('Stopping recording...');
      setIsRecording(false);
      setIsProcessing(true); // Show processing immediately
      setError(null); // Clear any previous error

      const text = await invoke<string>('stop_recording');
      console.log('Recording stopped, transcribed text:', text);

      if (text && text.trim()) {
        try {
          await invoke('inject_text', { text });
          saveToHistory(text); // Save to history on success
        } catch (injectionError) {
          console.error('Failed to inject text:', injectionError);
          setError(`Failed to paste: ${injectionError}`);
          // Clear error after 5 seconds
          setTimeout(() => setError(null), ERROR_TIMEOUT_MS);
        }
      } else {
        console.log('No text transcribed (empty result)');
      }
    } catch (err: unknown) {
      console.error('Failed to stop recording:', err);
      const errorMessage = err instanceof Error ? err.message : String(err);

      // Check if it's a model not found error
      if (errorMessage.includes('Model not found')) {
        const modelSize = settingsRef.current?.transcription?.model_size || 'small';
        setError(`No speech model. Downloading ${modelSize}...`);
        // Trigger model download
        try {
          await invoke('download_model', { size: modelSize });
          setError(null);
        } catch (downloadErr) {
          setError('Please download a model in Settings');
        }
      } else {
        setError(`Failed: ${errorMessage}`);
      }
      setTimeout(() => setError(null), ERROR_TIMEOUT_MS);
    } finally {
      isProcessingRef.current = false; // Reset ref immediately
      setIsProcessing(false); // Always reset processing state
    }
  }, [saveToHistory]);

  // Load settings on mount and when changed from another window
  useEffect(() => {
    loadSettings();
    const unlisten = listen('settings-changed', () => {
      loadSettings();
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [loadSettings]);

  // Multi-monitor tracking: periodically check if mouse moved to different monitor
  // Only for dictation window
  useEffect(() => {
    if (windowType !== 'dictation') return;

    let pollCount = 0;
    const checkMouseMonitor = async () => {
      try {
        const moved = await invoke<boolean>('reposition_to_mouse_monitor');
        pollCount++;
        if (moved) {
          console.log(`[poll] reposition_to_mouse_monitor returned TRUE (moved) at poll #${pollCount}`);
        }
        // Log draggable state every ~3 seconds
        if (pollCount % MONITOR_LOG_FREQUENCY === 0) {
          console.log(`[poll] poll #${pollCount}, draggable=${settingsRef.current?.widget?.draggable}`);
        }
      } catch (err) {
        console.error(`[poll] reposition_to_mouse_monitor ERROR:`, err);
      }
    };

    // Check every 150ms for monitor changes (fast enough to feel responsive)
    const intervalId = setInterval(checkMouseMonitor, MONITOR_POLL_INTERVAL_MS);
    console.log(`[poll] Started 150ms monitor tracking, draggable=${settings?.widget?.draggable}`);

    return () => {
      console.log(`[poll] Stopped 150ms monitor tracking`);
      clearInterval(intervalId);
    };
  }, [windowType]);

  // Set up event listeners (only once)
  useEffect(() => {
    const unlistenPressed = listen('hotkey-pressed', async () => {
      // Only the dictation window should handle recording — dashboard must ignore
      // to prevent race conditions where both windows invoke start/stop simultaneously
      if (windowType !== 'dictation') return;

      const mode = settingsRef.current?.hotkey?.mode ?? DEFAULT_HOTKEY_MODE; // Default to toggle
      console.log('Hotkey pressed, mode:', mode, 'isRecording:', isRecordingRef.current);

      if (mode === 'toggle') {
        if (isRecordingRef.current) {
          await stopRecording();
        } else {
          await startRecording();
        }
      } else {
        // Hold mode - start on press
        await startRecording();
      }
    });

    const unlistenReleased = listen('hotkey-released', async () => {
      if (windowType !== 'dictation') return;

      const mode = settingsRef.current?.hotkey?.mode ?? DEFAULT_HOTKEY_MODE;
      console.log('Hotkey released, mode:', mode, 'isRecording:', isRecordingRef.current);

      if (mode !== 'toggle' && isRecordingRef.current) {
        // Hold mode - stop on release
        await stopRecording();
      }
    });

    const unlistenProcessing = listen('transcription-processing', () => {
      console.log('Transcription processing started');
      setIsProcessing(true);
    });

    const unlistenComplete = listen<string>('transcription-complete', (event) => {
      console.log('Transcription complete:', event.payload);
      setIsProcessing(false);
    });

    const unlistenAudioLevel = listen<number>('audio-level', (event) => {
      setAudioLevel(event.payload);
    });

    // Handle model preloading status events
    const unlistenPreloadStart = listen<string>('model-preload-start', (event) => {
      console.log(`Model preload started: ${event.payload}`);
      setIsPreloading(true);
    });

    const unlistenPreloadComplete = listen<{ model: string; elapsed_secs: number }>('model-preload-complete', (event) => {
      console.log(`Model preload complete: ${event.payload.model} in ${event.payload.elapsed_secs.toFixed(1)}s`);
      setIsPreloading(false);
    });

    const unlistenPreloadError = listen<{ model: string; error: string }>('model-preload-error', (event) => {
      console.error(`Model preload failed: ${event.payload.error}`);
      setIsPreloading(false);
      // Show a brief error then clear it — preload failure is non-fatal
      setError(`Model warmup failed`);
      setTimeout(() => setError(null), MODEL_PRELOAD_ERROR_TIMEOUT_MS);
    });

    // Handle model needs download - auto-download on startup
    const unlistenModelDownload = listen<string>('model-needs-download', async (event) => {
      const modelSize = event.payload;
      console.log(`Model '${modelSize}' not found, downloading...`);
      setIsDownloadingModel(true);
      setError(`Downloading ${modelSize} speech model...`);
      try {
        await invoke('download_model', { size: modelSize });
        console.log(`Model '${modelSize}' downloaded successfully`);
        setError(null);
      } catch (err) {
        console.error(`Failed to download ${modelSize} model:`, err);
        setError('Failed to download speech model. Please download manually in Settings.');
        setTimeout(() => setError(null), MODEL_DOWNLOAD_ERROR_TIMEOUT_MS);
      } finally {
        setIsDownloadingModel(false);
      }
    });

    return () => {
      unlistenPressed.then((f) => f());
      unlistenReleased.then((f) => f());
      unlistenProcessing.then((f) => f());
      unlistenComplete.then((f) => f());
      unlistenAudioLevel.then((f) => f());
      unlistenPreloadStart.then((f) => f());
      unlistenPreloadComplete.then((f) => f());
      unlistenPreloadError.then((f) => f());
      unlistenModelDownload.then((f) => f());
    };
  }, [startRecording, stopRecording]);

  // Render based on window type
  if (windowType === 'dashboard') {
    return <Dashboard />;
  }

  // Log when widget settings change
  const draggableValue = settings?.widget?.draggable ?? false;
  const opacityValue = settings?.widget?.opacity ?? DEFAULT_WIDGET_OPACITY;
  const hotkeyLabel = settings?.hotkey?.key || DEFAULT_HOTKEY_LABEL;
  const hotkeyMode = settings?.hotkey?.mode || DEFAULT_HOTKEY_MODE;
  useEffect(() => {
    console.log(`[app] draggable prop changed to: ${draggableValue}`);
  }, [draggableValue]);

  // Main dictation bar overlay
  return (
    <div className="dictation-container">
      <DictationBar
        isRecording={isRecording}
        isProcessing={isProcessing || isDownloadingModel}
        isPreloading={isPreloading}
        audioLevel={audioLevel}
        error={error}
        statusOverride={isDownloadingModel ? 'Downloading model...' : undefined}
        draggable={draggableValue}
        opacity={opacityValue}
        hotkeyLabel={hotkeyLabel}
        hotkeyMode={hotkeyMode}
      />
    </div>
  );
}

export default App;
