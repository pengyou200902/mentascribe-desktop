import { FC, useEffect, useState, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

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
  statusOverride,
}) => {
  const isDragging = useRef(false);
  const prevLevelsRef = useRef<number[]>(Array(12).fill(0.15));
  const audioLevelRef = useRef(audioLevel);
  const isProcessingRef = useRef(isProcessing);
  const targetHeightsRef = useRef<number[]>(Array(12).fill(0.15));
  const lastUpdateRef = useRef(0);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(12).fill(0.15));

  // Keep refs in sync
  useEffect(() => {
    audioLevelRef.current = audioLevel;
  }, [audioLevel]);

  useEffect(() => {
    isProcessingRef.current = isProcessing;
  }, [isProcessing]);

  // Continuously animate waveform when recording or processing
  useEffect(() => {
    if (!isRecording && !isProcessing) {
      setWaveformBars(Array(12).fill(0.15));
      prevLevelsRef.current = Array(12).fill(0.15);
      targetHeightsRef.current = Array(12).fill(0.15);
      lastUpdateRef.current = 0;
      return;
    }

    // Immediately set bars to visible random heights when starting
    const initialHeights = Array(12).fill(0).map(() => 0.4 + Math.random() * 0.3);
    prevLevelsRef.current = initialHeights;
    targetHeightsRef.current = initialHeights;
    setWaveformBars(initialHeights);

    let animationFrameId: number;

    const animate = () => {
      const level = audioLevelRef.current;
      const now = Date.now();

      // Update target heights frequently for responsive animation
      const updateInterval = 40; // ~25fps for target updates

      if (now - lastUpdateRef.current > updateInterval) {
        lastUpdateRef.current = now;

        // Generate new random target heights based on audio level
        targetHeightsRef.current = targetHeightsRef.current.map(() => {
          if (isProcessingRef.current) {
            // During processing, show gentle pulsing animation
            return 0.35 + Math.random() * 0.4; // 0.35-0.75 range
          }

          // During recording: respond to audio level
          // Base height ensures visible animation even when quiet
          // Use wider range for more visible movement
          const baseHeight = 0.3 + Math.random() * 0.4; // 0.3-0.7 range

          // Audio boost: when speaking, bars grow taller
          // level is 0-1, multiply for dramatic effect
          const audioBoost = level * (0.3 + Math.random() * 0.5);

          // Combine: base + audio response, capped at 1.0
          return Math.min(1.0, baseHeight + audioBoost);
        });
      }

      // Smoothly interpolate towards target heights
      const newBars = prevLevelsRef.current.map((prevHeight, i) => {
        const target = targetHeightsRef.current[i];
        // Fast interpolation for snappy, visible response
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

  const handleMouseDown = async (e: React.MouseEvent) => {
    // Only drag on left click
    if (e.button !== 0) return;

    isDragging.current = true;
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error('Failed to start dragging:', err);
    }
    isDragging.current = false;
  };

  return (
    <div
      className="dictation-bar"
      onMouseDown={handleMouseDown}
    >
      {/* Microphone icon / Recording indicator */}
      <div className="drag-content">
        {isRecording ? (
          <div className="indicator-dot recording" />
        ) : isProcessing ? (
          <div className="indicator-dot processing" />
        ) : (
          <svg
            className="mic-icon"
            viewBox="0 0 24 24"
            fill="currentColor"
            width="20"
            height="20"
          >
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
            <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
          </svg>
        )}

        {/* Waveform visualization - only show when recording or processing */}
        {(isRecording || isProcessing) && (
          <div className="waveform">
            {waveformBars.map((height, i) => (
              <div
                key={i}
                className={`waveform-bar ${isRecording ? 'active' : 'processing'}`}
                style={{
                  height: `${Math.min(100, Math.max(20, height * 100))}%`,
                }}
              />
            ))}
          </div>
        )}

        {/* Status text */}
        <span className={`status-text ${error ? 'error' : ''}`}>
          {error ? 'Error!' : statusOverride ? statusOverride : isRecording ? 'Listening...' : isProcessing ? 'Processing...' : 'Ready'}
        </span>
      </div>
    </div>
  );
};
