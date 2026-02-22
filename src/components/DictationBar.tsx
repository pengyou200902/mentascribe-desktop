import { FC, useEffect, useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  WAVEFORM_BAR_COUNT, WAVEFORM_INITIAL_HEIGHT, WAVEFORM_UPDATE_INTERVAL_MS,
  WAVEFORM_SMOOTHING, WAVEFORM_BASE_MIN, WAVEFORM_CENTER_AMPLITUDE,
  WAVEFORM_NOISE_RANGE, WAVEFORM_RANDOM_RANGE, AUDIO_BOOST_BASE,
  AUDIO_BOOST_RANGE, WAVEFORM_MAX_HEIGHT, BAR_MIN_HEIGHT_PX, BAR_HEIGHT_SCALE,
  PROCESSING_DOT_COUNT, PROCESSING_DOT_DELAY_STEP,
  CURSOR_POLL_INTERVAL_MS, PRELOAD_FLASH_DURATION_MS,
  DEFAULT_HOTKEY_LABEL, DEFAULT_HOTKEY_MODE,
} from '../config/widget';

interface DictationBarProps {
  isRecording: boolean;
  isProcessing: boolean;
  isPreloading?: boolean;
  audioLevel?: number;
  error?: string | null;
  statusOverride?: string;
  draggable?: boolean;
  opacity?: number;
  hotkeyLabel?: string;
  hotkeyMode?: string;
}

export const DictationBar: FC<DictationBarProps> = ({
  isRecording,
  isProcessing,
  isPreloading = false,
  audioLevel = 0,
  error = null,
  draggable = false,
  opacity = 1.0,
  hotkeyLabel = DEFAULT_HOTKEY_LABEL,
  hotkeyMode = DEFAULT_HOTKEY_MODE,
}) => {
  const audioLevelRef = useRef(audioLevel);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT));
  const [isHovered, setIsHovered] = useState(false);
  const [initComplete, setInitComplete] = useState(false);
  const prevPreloadingRef = useRef(isPreloading);
  const widgetRef = useRef<HTMLDivElement>(null);
  const prevLevelsRef = useRef<number[]>(Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT));
  const targetHeightsRef = useRef<number[]>(Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT));
  const lastUpdateRef = useRef(0);
  const prevDraggableRef = useRef(draggable);

  // Detect preloading completion for smooth transition animation
  useEffect(() => {
    if (prevPreloadingRef.current && !isPreloading) {
      // Preload just finished — show brief completion flash
      setInitComplete(true);
      const timer = setTimeout(() => setInitComplete(false), PRELOAD_FLASH_DURATION_MS);
      return () => clearTimeout(timer);
    }
    prevPreloadingRef.current = isPreloading;
  }, [isPreloading]);

  // Log draggable prop changes — forward to Rust terminal
  useEffect(() => {
    invoke('frontend_log', { msg: `[DictationBar] draggable prop = ${draggable} (prev: ${prevDraggableRef.current})` }).catch(() => {});
    prevDraggableRef.current = draggable;
  }, [draggable]);

  // Determine state
  const isActive = isRecording || isProcessing;
  const isExpanded = isHovered || isActive || isPreloading || !!error || initComplete;
  const isIdle = isExpanded && !isActive && !isPreloading && !error && !initComplete;

  // Keep refs in sync
  useEffect(() => {
    audioLevelRef.current = audioLevel;
  }, [audioLevel]);

  // Cursor proximity detection via Rust — works regardless of window focus.
  // Uses [NSEvent mouseLocation] which is always available, unlike JS mouse
  // events which only fire when the NSPanel has focus.
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const over = await invoke<boolean>('is_cursor_over_pill');
        setIsHovered(over);
      } catch {}
    }, CURSOR_POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, []);

  // Animate waveform only when recording
  useEffect(() => {
    if (!isRecording) {
      setWaveformBars(Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT));
      prevLevelsRef.current = Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT);
      targetHeightsRef.current = Array(WAVEFORM_BAR_COUNT).fill(WAVEFORM_INITIAL_HEIGHT);
      lastUpdateRef.current = 0;
      return;
    }

    // Recording: animate based on audio level
    const initialHeights = Array(WAVEFORM_BAR_COUNT).fill(0).map(() => WAVEFORM_INITIAL_HEIGHT + Math.random() * WAVEFORM_RANDOM_RANGE);
    prevLevelsRef.current = initialHeights;
    targetHeightsRef.current = initialHeights;
    setWaveformBars(initialHeights);

    let animationFrameId: number;

    const animate = () => {
      const level = audioLevelRef.current;
      const now = Date.now();
      if (now - lastUpdateRef.current > WAVEFORM_UPDATE_INTERVAL_MS) {
        lastUpdateRef.current = now;
        targetHeightsRef.current = targetHeightsRef.current.map((_, i) => {
          // Create a wave-like pattern with center bars taller
          const centerIdx = (WAVEFORM_BAR_COUNT - 1) / 2;
          const centerFactor = 1 - Math.abs(i - centerIdx) / (centerIdx + 1);
          const baseHeight = WAVEFORM_BASE_MIN + centerFactor * WAVEFORM_CENTER_AMPLITUDE + Math.random() * WAVEFORM_NOISE_RANGE;
          const audioBoost = level * (AUDIO_BOOST_BASE + Math.random() * AUDIO_BOOST_RANGE);
          return Math.min(WAVEFORM_MAX_HEIGHT, baseHeight + audioBoost);
        });
      }

      const newBars = prevLevelsRef.current.map((prevHeight, i) => {
        const target = targetHeightsRef.current[i];
        const smoothing = WAVEFORM_SMOOTHING;
        return prevHeight + (target - prevHeight) * smoothing;
      });

      prevLevelsRef.current = newBars;
      setWaveformBars(newBars);

      animationFrameId = requestAnimationFrame(animate);
    };

    animationFrameId = requestAnimationFrame(animate);

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [isRecording]);

  // Dynamically resize the Tauri window to match the pill dimensions.
  // Uses resize_pill Rust command which atomically sets size + position via
  // setFrame:display:, keeping the bottom edge fixed so the pill grows upward.
  useEffect(() => {
    if (!widgetRef.current) return;
    const observer = new ResizeObserver(() => {
      if (!widgetRef.current) return;
      const w = widgetRef.current.offsetWidth;
      const h = widgetRef.current.offsetHeight;
      if (w > 0 && h > 0) {
        invoke('resize_pill', { width: w, height: h }).catch(() => {});
      }
    });
    observer.observe(widgetRef.current);
    return () => observer.disconnect();
  }, []);

  // Helper to forward logs to Rust terminal (fire-and-forget)
  const flog = useCallback((msg: string) => {
    invoke('frontend_log', { msg }).catch(() => {});
  }, []);

  // Native drag — all mouse tracking happens in Rust via NSEvent monitors.
  // JS just signals drag start; [NSEvent mouseLocation] handles coordinates
  // reliably across mixed-DPI monitors (bypasses WKWebView screenX/Y bug).
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!draggable || e.button !== 0) return;
    e.preventDefault();
    flog('[drag] Starting native drag via NSEvent monitors');
    invoke('start_native_drag').catch((err) => {
      flog(`[drag] ERROR: start_native_drag failed: ${err}`);
    });
  }, [draggable, flog]);

  // Render expanded idle state - subtle dots inside pill
  const renderExpandedIdle = () => (
    <div className="wispr-idle-dots">
      {Array(WAVEFORM_BAR_COUNT).fill(0).map((_, i) => (
        <div key={i} className="wispr-idle-dot" />
      ))}
    </div>
  );

  // Render initializing state - warm-up glow bars
  const renderInitializing = () => (
    <div className="wispr-initializing">
      <div className="wispr-init-bars">
        <div className="wispr-init-bar" />
        <div className="wispr-init-bar" />
        <div className="wispr-init-bar" />
        <div className="wispr-init-bar" />
        <div className="wispr-init-bar" />
      </div>
      <span className="wispr-init-label">Warming up</span>
    </div>
  );

  // Render recording state - vertical waveform bars
  const renderRecording = () => (
    <div className="wispr-waveform">
      {waveformBars.map((height, i) => (
        <div
          key={i}
          className="wispr-bar"
          style={{
            height: `${Math.round(Math.max(BAR_MIN_HEIGHT_PX, height * BAR_HEIGHT_SCALE))}px`,
          }}
        />
      ))}
    </div>
  );

  // Render processing state - dots with spinner
  const renderProcessing = () => (
    <div className="wispr-processing">
      <div className="wispr-dots">
        {Array(PROCESSING_DOT_COUNT).fill(0).map((_, i) => (
          <div key={i} className="wispr-dot" style={{ animationDelay: `${i * PROCESSING_DOT_DELAY_STEP}s` }} />
        ))}
      </div>
      <div className="wispr-spinner" />
    </div>
  );

  // Render error state
  const renderError = () => (
    <div className="wispr-error">
      <span className="wispr-error-text">{error}</span>
    </div>
  );

  return (
    <div
      ref={widgetRef}
      className="wispr-widget"
      style={{ opacity, ...(draggable ? { cursor: 'grab' } : {}) }}
      onMouseDown={handleMouseDown}
    >
      {isIdle && (
        <div className="wispr-tooltip">
          {hotkeyMode === 'hold'
            ? <>Hold <span className="wispr-hotkey">{hotkeyLabel}</span> to start dictating</>
            : <>Press <span className="wispr-hotkey">{hotkeyLabel}</span> to start dictating</>}
        </div>
      )}
      <div
        className={`wispr-pill ${isExpanded ? 'expanded' : 'collapsed'} ${isActive ? 'active' : ''} ${error ? 'has-error' : ''} ${isPreloading ? 'initializing' : ''} ${initComplete ? 'init-complete' : ''}`}
      >
        {isExpanded && (
          <div className="wispr-content">
            {error ? renderError() :
             isProcessing ? renderProcessing() :
             isRecording ? renderRecording() :
             isPreloading ? renderInitializing() :
             renderExpandedIdle()}
          </div>
        )}
      </div>
    </div>
  );
};
