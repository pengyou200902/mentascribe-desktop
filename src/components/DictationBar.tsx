import { FC, useEffect, useState, useRef, useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useStore } from '../lib/store';

interface DictationBarProps {
  isRecording: boolean;
  isProcessing: boolean;
  audioLevel?: number;
  error?: string | null;
  statusOverride?: string;
}

export const DictationBar: FC<DictationBarProps> = ({
  isRecording,
  isProcessing,
  audioLevel = 0,
  error = null,
}) => {
  const prevLevelsRef = useRef<number[]>(Array(12).fill(0.15));
  const audioLevelRef = useRef(audioLevel);
  const isProcessingRef = useRef(isProcessing);
  const targetHeightsRef = useRef<number[]>(Array(12).fill(0.15));
  const lastUpdateRef = useRef(0);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(12).fill(0.15));
  const [isHovered, setIsHovered] = useState(false);
  const widgetRef = useRef<HTMLDivElement>(null);
  const { settings } = useStore();

  // Get the configured hotkey for display
  const hotkeyDisplay = settings?.hotkey?.key || 'F6';
  const hotkeyMode = settings?.hotkey?.mode || 'hold';

  // Determine if we should show expanded state
  const isExpanded = isHovered || isRecording || isProcessing;
  const isActive = isRecording || isProcessing;

  // Keep refs in sync
  useEffect(() => {
    audioLevelRef.current = audioLevel;
  }, [audioLevel]);

  useEffect(() => {
    isProcessingRef.current = isProcessing;
  }, [isProcessing]);

  // Setup window-level mouse tracking for transparent windows
  // Use multiple approaches to ensure hover works
  useEffect(() => {
    const window = getCurrentWindow();

    // Method 1: Listen for window focus events (window must be focused for events)
    const unlistenFocus = window.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        // When window loses focus, collapse after a short delay
        setTimeout(() => {
          setIsHovered(false);
        }, 100);
      }
    });

    // Method 2: Document-level mouse tracking (works when window has any focus)
    const handleMouseMove = (e: MouseEvent) => {
      if (!widgetRef.current) return;

      const rect = widgetRef.current.getBoundingClientRect();
      const padding = 8;
      const isInside =
        e.clientX >= rect.left - padding &&
        e.clientX <= rect.right + padding &&
        e.clientY >= rect.top - padding &&
        e.clientY <= rect.bottom + padding;

      setIsHovered(isInside);
    };

    const handleMouseLeave = () => {
      setIsHovered(false);
    };

    // Method 3: Window-level mouse enter detection
    const handleWindowMouseEnter = () => {
      setIsHovered(true);
    };

    // Add all event listeners
    document.addEventListener('mousemove', handleMouseMove, { passive: true });
    document.addEventListener('mouseleave', handleMouseLeave);

    // Use mouseenter on window element as backup
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

  // Animate waveform only when recording (not during processing/transcription)
  useEffect(() => {
    // When processing (transcribing), show flat zero-level waveform
    if (isProcessing && !isRecording) {
      setWaveformBars(Array(12).fill(0.15));
      prevLevelsRef.current = Array(12).fill(0.15);
      targetHeightsRef.current = Array(12).fill(0.15);
      lastUpdateRef.current = 0;
      return;
    }

    // When not recording, reset to flat
    if (!isRecording) {
      setWaveformBars(Array(12).fill(0.15));
      prevLevelsRef.current = Array(12).fill(0.15);
      targetHeightsRef.current = Array(12).fill(0.15);
      lastUpdateRef.current = 0;
      return;
    }

    // Recording: animate based on audio level
    const initialHeights = Array(12).fill(0).map(() => 0.4 + Math.random() * 0.3);
    prevLevelsRef.current = initialHeights;
    targetHeightsRef.current = initialHeights;
    setWaveformBars(initialHeights);

    let animationFrameId: number;

    const animate = () => {
      const level = audioLevelRef.current;
      const now = Date.now();
      const updateInterval = 40;

      if (now - lastUpdateRef.current > updateInterval) {
        lastUpdateRef.current = now;

        targetHeightsRef.current = targetHeightsRef.current.map(() => {
          const baseHeight = 0.3 + Math.random() * 0.4;
          const audioBoost = level * (0.3 + Math.random() * 0.5);
          return Math.min(1.0, baseHeight + audioBoost);
        });
      }

      const newBars = prevLevelsRef.current.map((prevHeight, i) => {
        const target = targetHeightsRef.current[i];
        const smoothing = 0.4;
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
  }, [isRecording, isProcessing]);

  // Direct hover handlers as additional fallback
  const handlePointerEnter = useCallback(() => {
    setIsHovered(true);
  }, []);

  const handlePointerLeave = useCallback(() => {
    setIsHovered(false);
  }, []);

  // Get status text
  const getStatusText = () => {
    if (error) return error;
    if (isProcessing) return 'Transcribing...';
    if (isRecording) return 'Listening...';
    return null;
  };

  const statusText = getStatusText();

  return (
    <div
      ref={widgetRef}
      className={`dictation-widget ${isExpanded ? 'expanded' : 'collapsed'} ${isActive ? 'active' : ''} ${error ? 'has-error' : ''}`}
      onPointerEnter={handlePointerEnter}
      onPointerLeave={handlePointerLeave}
      onMouseEnter={handlePointerEnter}
      onMouseLeave={handlePointerLeave}
    >
      {/* Invisible hit area for better mouse detection */}
      <div className="widget-hitarea" />

      {/* Collapsed state: minimal pill handle */}
      <div className="widget-collapsed">
        <div className="pill-indicator">
          <div className={`pill-dot ${isRecording ? 'recording' : ''} ${isProcessing ? 'processing' : ''}`} />
        </div>
      </div>

      {/* Expanded state: instruction + waveform */}
      <div className="widget-expanded">
        {/* Instruction text or status */}
        <div className="instruction-row">
          {statusText ? (
            <span className={`status-text ${error ? 'error' : isRecording ? 'recording' : 'processing'}`}>
              {statusText}
            </span>
          ) : (
            <span className="instruction-text">
              {hotkeyMode === 'toggle' ? 'Press' : 'Click or hold'}{' '}
              <kbd className="hotkey-badge">{hotkeyDisplay}</kbd>
              {' '}to start dictating
            </span>
          )}
        </div>

        {/* Waveform visualization */}
        <div className="waveform-row">
          <div className="waveform-container">
            {waveformBars.map((height, i) => (
              <div
                key={i}
                className={`waveform-dot ${isActive ? 'active' : 'idle'}`}
                style={{
                  height: isActive
                    ? `${Math.round(Math.min(24, Math.max(4, height * 24)))}px`
                    : '4px',
                }}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
