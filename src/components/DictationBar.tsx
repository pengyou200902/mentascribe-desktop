import { FC, useEffect, useState, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface DictationBarProps {
  isRecording: boolean;
  isProcessing: boolean;
  audioLevel?: number;
}

export const DictationBar: FC<DictationBarProps> = ({
  isRecording,
  isProcessing,
  audioLevel = 0,
}) => {
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(10).fill(0.2));
  const animationRef = useRef<number>();
  const isDragging = useRef(false);

  useEffect(() => {
    if (isRecording) {
      const animate = () => {
        setWaveformBars((prev) =>
          prev.map(() => {
            const base = audioLevel > 0 ? audioLevel : 0.5;
            return Math.random() * base * 0.7 + 0.3;
          })
        );
        animationRef.current = requestAnimationFrame(animate);
      };

      const interval = setInterval(() => {
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
        }
        animate();
      }, 80);

      return () => {
        clearInterval(interval);
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
        }
      };
    } else {
      setWaveformBars(Array(10).fill(0.2));
    }
  }, [isRecording, audioLevel]);

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
                  height: `${Math.max(25, height * 100)}%`,
                }}
              />
            ))}
          </div>
        )}

        {/* Status text */}
        <span className="status-text">
          {isRecording ? 'Listening...' : isProcessing ? 'Processing...' : 'Ready'}
        </span>
      </div>
    </div>
  );
};
