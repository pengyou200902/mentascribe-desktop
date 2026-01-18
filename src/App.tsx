import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { DictationBar } from './components/DictationBar';
import { Settings } from './components/Settings';
import { History } from './components/History';
import { useStore } from './lib/store';

type WindowType = 'dictation' | 'settings' | 'history';

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const { settings, loadSettings } = useStore();

  // Determine which window type we're in based on URL hash
  const getWindowType = (): WindowType => {
    const hash = window.location.hash.slice(1);
    if (hash === 'settings') return 'settings';
    if (hash === 'history') return 'history';
    return 'dictation';
  };

  const [windowType] = useState<WindowType>(getWindowType);

  useEffect(() => {
    loadSettings();

    const unlistenPressed = listen('hotkey-pressed', async () => {
      if (settings?.hotkey?.mode === 'toggle') {
        if (isRecording) {
          await stopRecording();
        } else {
          await startRecording();
        }
      } else {
        await startRecording();
      }
    });

    const unlistenReleased = listen('hotkey-released', async () => {
      if (settings?.hotkey?.mode !== 'toggle' && isRecording) {
        await stopRecording();
      }
    });

    const unlistenProcessing = listen('transcription-processing', () => {
      setIsProcessing(true);
    });

    const unlistenComplete = listen<string>('transcription-complete', () => {
      setIsProcessing(false);
    });

    const unlistenAudioLevel = listen<number>('audio-level', (event) => {
      setAudioLevel(event.payload);
    });

    return () => {
      unlistenPressed.then((f) => f());
      unlistenReleased.then((f) => f());
      unlistenProcessing.then((f) => f());
      unlistenComplete.then((f) => f());
      unlistenAudioLevel.then((f) => f());
    };
  }, [isRecording, settings]);

  async function startRecording() {
    try {
      await invoke('start_recording');
      setIsRecording(true);
    } catch (error) {
      console.error('Failed to start recording:', error);
    }
  }

  async function stopRecording() {
    try {
      const text = await invoke<string>('stop_recording');
      setIsRecording(false);

      if (text) {
        await invoke('inject_text', { text });
      }
    } catch (error) {
      console.error('Failed to stop recording:', error);
      setIsRecording(false);
    }
  }

  // Render based on window type
  if (windowType === 'settings') {
    return (
      <div className="min-h-screen bg-gray-900 text-white p-4">
        <Settings onBack={() => window.close()} />
      </div>
    );
  }

  if (windowType === 'history') {
    return (
      <div className="min-h-screen bg-gray-900 text-white p-4">
        <History onBack={() => window.close()} />
      </div>
    );
  }

  // Main dictation bar overlay
  return (
    <div className="dictation-container">
      <DictationBar
        isRecording={isRecording}
        isProcessing={isProcessing}
        audioLevel={audioLevel}
      />
    </div>
  );
}

export default App;
