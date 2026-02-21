import { FC, useEffect, useState, useRef, useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { invoke } from '@tauri-apps/api/core';

interface DictationBarProps {
  isRecording: boolean;
  isProcessing: boolean;
  isPreloading?: boolean;
  audioLevel?: number;
  error?: string | null;
  statusOverride?: string;
  draggable?: boolean;
  opacity?: number;
  hotkeyKey?: string;
}

export const DictationBar: FC<DictationBarProps> = ({
  isRecording,
  isProcessing,
  isPreloading = false,
  audioLevel = 0,
  error = null,
  draggable = false,
  opacity = 1.0,
  hotkeyKey = 'Fn',
}) => {
  const audioLevelRef = useRef(audioLevel);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(9).fill(0.3));
  const [isHovered, setIsHovered] = useState(false);
  const [initComplete, setInitComplete] = useState(false);
  const prevPreloadingRef = useRef(isPreloading);
  const widgetRef = useRef<HTMLDivElement>(null);
  const prevLevelsRef = useRef<number[]>(Array(9).fill(0.3));
  const targetHeightsRef = useRef<number[]>(Array(9).fill(0.3));
  const lastUpdateRef = useRef(0);
  const prevDraggableRef = useRef(draggable);

  // Detect preloading completion for smooth transition animation
  useEffect(() => {
    if (prevPreloadingRef.current && !isPreloading) {
      // Preload just finished — show brief completion flash
      setInitComplete(true);
      const timer = setTimeout(() => setInitComplete(false), 600);
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
  const showExpandedHint = isHovered && !isActive && !error && !isPreloading && !initComplete;

  // Keep refs in sync
  useEffect(() => {
    audioLevelRef.current = audioLevel;
  }, [audioLevel]);

  // Setup window-level mouse tracking for transparent windows
  useEffect(() => {
    const window = getCurrentWindow();

    const unlistenFocus = window.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        setTimeout(() => setIsHovered(false), 100);
      }
    });

    const handleMouseMove = (e: MouseEvent) => {
      if (!widgetRef.current) return;
      const rect = widgetRef.current.getBoundingClientRect();
      const padding = 4;
      const isInside =
        e.clientX >= rect.left - padding &&
        e.clientX <= rect.right + padding &&
        e.clientY >= rect.top - padding &&
        e.clientY <= rect.bottom + padding;
      setIsHovered(isInside);
    };

    const handleMouseLeave = () => setIsHovered(false);
    const handleWindowMouseEnter = () => setIsHovered(true);

    document.addEventListener('mousemove', handleMouseMove, { passive: true });
    document.addEventListener('mouseleave', handleMouseLeave);

    const rootEl = document.getElementById('root');
    if (rootEl) {
      rootEl.addEventListener('mouseenter', handleWindowMouseEnter);
    }

    return () => {
      unlistenFocus.then(fn => fn());
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseleave', handleMouseLeave);
      if (rootEl) {
        rootEl.removeEventListener('mouseenter', handleWindowMouseEnter);
      }
    };
  }, []);

  // Animate waveform only when recording
  useEffect(() => {
    if (!isRecording) {
      setWaveformBars(Array(9).fill(0.3));
      prevLevelsRef.current = Array(9).fill(0.3);
      targetHeightsRef.current = Array(9).fill(0.3);
      lastUpdateRef.current = 0;
      return;
    }

    // Recording: animate based on audio level
    const initialHeights = Array(9).fill(0).map(() => 0.3 + Math.random() * 0.4);
    prevLevelsRef.current = initialHeights;
    targetHeightsRef.current = initialHeights;
    setWaveformBars(initialHeights);

    let animationFrameId: number;

    const animate = () => {
      const level = audioLevelRef.current;
      const now = Date.now();
      const updateInterval = 50;

      if (now - lastUpdateRef.current > updateInterval) {
        lastUpdateRef.current = now;
        targetHeightsRef.current = targetHeightsRef.current.map((_, i) => {
          // Create a wave-like pattern with center bars taller
          const centerFactor = 1 - Math.abs(i - 4) / 5;
          const baseHeight = 0.25 + centerFactor * 0.3 + Math.random() * 0.2;
          const audioBoost = level * (0.4 + Math.random() * 0.4);
          return Math.min(1.0, baseHeight + audioBoost);
        });
      }

      const newBars = prevLevelsRef.current.map((prevHeight, i) => {
        const target = targetHeightsRef.current[i];
        const smoothing = 0.35;
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

  const handlePointerEnter = useCallback(() => setIsHovered(true), []);
  const handlePointerLeave = useCallback(() => setIsHovered(false), []);

  // Dynamically resize the Tauri window to match the hitbox dimensions
  // The hitbox includes transparent padding around the pill for proximity hover detection
  useEffect(() => {
    if (!widgetRef.current) return;
    const win = getCurrentWindow();
    const observer = new ResizeObserver(() => {
      if (!widgetRef.current) return;
      const w = widgetRef.current.offsetWidth;
      const h = widgetRef.current.offsetHeight;
      if (w > 0 && h > 0) {
        win.setSize(new LogicalSize(w, h)).catch(() => {});
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

  // Render idle state - simple horizontal dash (collapsed pill indicator)
  const renderIdle = () => (
    <div className="wispr-idle">
      <div className="wispr-dash" />
    </div>
  );

  // Render expanded hint - WisperFlow-style tooltip with hotkey instruction
  const renderExpandedHint = () => (
    <div className="wispr-expanded-content">
      <span className="wispr-hint-text">
        Click or hold <kbd className="wispr-hint-key">{hotkeyKey}</kbd> to start dictating
      </span>
      <div className="wispr-hint-indicator">
        {Array(10).fill(0).map((_, i) => (
          <div key={i} className="wispr-hint-dot" />
        ))}
      </div>
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
            height: `${Math.round(Math.max(4, height * 20))}px`,
          }}
        />
      ))}
    </div>
  );

  // Render processing state - dots with spinner
  const renderProcessing = () => (
    <div className="wispr-processing">
      <div className="wispr-dots">
        {Array(8).fill(0).map((_, i) => (
          <div key={i} className="wispr-dot" style={{ animationDelay: `${i * 0.1}s` }} />
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

  // Build pill CSS classes
  const pillClasses = [
    'wispr-pill',
    showExpandedHint ? 'expanded' : '',
    isActive ? 'active' : '',
    error ? 'has-error' : '',
    isPreloading ? 'initializing' : '',
    initComplete ? 'init-complete' : '',
  ].filter(Boolean).join(' ');

  return (
    <div
      ref={widgetRef}
      className="wispr-hitbox"
      onPointerEnter={handlePointerEnter}
      onPointerLeave={handlePointerLeave}
      onMouseEnter={handlePointerEnter}
      onMouseLeave={handlePointerLeave}
    >
      <div
        className={pillClasses}
        style={{ opacity, ...(draggable ? { cursor: 'grab' } : {}) }}
        onMouseDown={handleMouseDown}
      >
        {showExpandedHint ? (
          renderExpandedHint()
        ) : (
          <div className="wispr-content">
            {error ? renderError() :
             isProcessing ? renderProcessing() :
             isRecording ? renderRecording() :
             isPreloading ? renderInitializing() :
             renderIdle()}
          </div>
        )}
      </div>
    </div>
  );
};
