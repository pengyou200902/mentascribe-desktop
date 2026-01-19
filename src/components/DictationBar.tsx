import { FC, useEffect, useState, useRef } from 'react';
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
  const isDragging = useRef(false);
  const prevLevelsRef = useRef<number[]>(Array(12).fill(0.15));
  const audioLevelRef = useRef(audioLevel);
  const isProcessingRef = useRef(isProcessing);
  const targetHeightsRef = useRef<number[]>(Array(12).fill(0.15));
  const lastUpdateRef = useRef(0);
  const [waveformBars, setWaveformBars] = useState<number[]>(Array(12).fill(0.15));
  const [isHovered, setIsHovered] = useState(false);
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

  // Continuously animate waveform when recording or processing
  useEffect(() => {
    if (!isRecording && !isProcessing) {
      // Reset to flat line when not active
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
      className={`dictation-widget ${isExpanded ? 'expanded' : 'collapsed'} ${isActive ? 'active' : ''} ${error ? 'has-error' : ''}`}
      onMouseDown={handleMouseDown}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
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
                    ? `${Math.min(100, Math.max(15, height * 100))}%`
                    : '15%',
                  animationDelay: `${i * 0.05}s`,
                }}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
