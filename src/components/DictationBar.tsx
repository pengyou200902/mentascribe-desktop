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
}) => {
  const isDragging = useRef(false);
  const prevLevelsRef = useRef<number[]>(Array(8).fill(0.15));
  const audioLevelRef = useRef(audioLevel);
  const isProcessingRef = useRef(isProcessing);
  const targetHeightsRef = useRef<number[]>(Array(8).fill(0.15));
  const lastUpdateRef = useRef(0);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(8).fill(0.15));

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
      setWaveformBars(Array(8).fill(0.15));
      prevLevelsRef.current = Array(8).fill(0.15);
      targetHeightsRef.current = Array(8).fill(0.15);
      lastUpdateRef.current = 0;
      return;
    }

    // Immediately set bars to visible random heights when starting
    const initialHeights = Array(8).fill(0).map(() => 0.4 + Math.random() * 0.3);
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
          if (isProcessingRef.current) {
            return 0.35 + Math.random() * 0.4;
          }
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

  const handleMouseDown = async (e: React.MouseEvent) => {
    if (e.button !== 0) return;

    isDragging.current = true;
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error('Failed to start dragging:', err);
    }
    isDragging.current = false;
  };

  // Determine mic state for styling
  const micState = error ? 'error' : isRecording ? 'recording' : isProcessing ? 'processing' : 'idle';

  return (
    <div
      className="dictation-bar"
      onMouseDown={handleMouseDown}
    >
      <div className="drag-content">
        {/* Microphone icon - always visible, color changes based on state */}
        <svg
          className={`mic-icon ${micState}`}
          viewBox="0 0 24 24"
          fill="currentColor"
          width="16"
          height="16"
        >
          <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
          <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
        </svg>

        {/* Waveform visualization - show when recording or processing */}
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
      </div>
    </div>
  );
};
