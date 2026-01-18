import { FC, useEffect, useState, useRef } from 'react';

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
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(12).fill(0.2));
  const animationRef = useRef<number>();

  useEffect(() => {
    if (isRecording) {
      const animate = () => {
        setWaveformBars((prev) =>
          prev.map(() => {
            const base = audioLevel > 0 ? audioLevel : 0.3;
            return Math.random() * base * 0.8 + 0.2;
          })
        );
        animationRef.current = requestAnimationFrame(animate);
      };

      const interval = setInterval(() => {
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
        }
        animate();
      }, 100);

      return () => {
        clearInterval(interval);
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
        }
      };
    } else {
      setWaveformBars(Array(12).fill(0.2));
    }
  }, [isRecording, audioLevel]);

  return (
    <div
      className="dictation-bar"
      data-tauri-drag-region
    >
      {/* Recording indicator dot */}
      <div
        className={`indicator-dot ${
          isRecording ? 'recording' : isProcessing ? 'processing' : 'idle'
        }`}
      />

      {/* Waveform visualization */}
      <div className="waveform">
        {waveformBars.map((height, i) => (
          <div
            key={i}
            className={`waveform-bar ${isRecording ? 'active' : ''}`}
            style={{
              height: `${Math.max(20, height * 100)}%`,
              animationDelay: `${i * 30}ms`,
            }}
          />
        ))}
      </div>

      {/* Status text */}
      <span className="status-text">
        {isRecording ? 'Listening...' : isProcessing ? 'Processing...' : ''}
      </span>
    </div>
  );
};
