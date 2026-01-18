import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { MenuBar } from './components/MenuBar';
import { Settings } from './components/Settings';
import { TranscriptionOverlay } from './components/TranscriptionOverlay';
import { History } from './components/History';
import { useStore } from './lib/store';

type View = 'main' | 'settings' | 'history';

function App() {
  const [view, setView] = useState<View>('main');
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [lastTranscription, setLastTranscription] = useState<string | null>(null);
  const { settings, loadSettings } = useStore();

  useEffect(() => {
    // Load settings on mount
    loadSettings();

    // Listen for hotkey events
    const unlistenPressed = listen('hotkey-pressed', async () => {
      if (settings?.hotkey?.mode === 'toggle') {
        if (isRecording) {
          await stopRecording();
        } else {
          await startRecording();
        }
      } else {
        // Hold mode
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

    const unlistenComplete = listen<string>('transcription-complete', (event) => {
      setIsProcessing(false);
      setLastTranscription(event.payload);
    });

    return () => {
      unlistenPressed.then((f) => f());
      unlistenReleased.then((f) => f());
      unlistenProcessing.then((f) => f());
      unlistenComplete.then((f) => f());
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
        // Inject text into active application
        await invoke('inject_text', { text });
      }
    } catch (error) {
      console.error('Failed to stop recording:', error);
      setIsRecording(false);
    }
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <MenuBar
        currentView={view}
        onViewChange={setView}
        isRecording={isRecording}
      />

      <main className="p-4">
        {view === 'main' && (
          <div className="space-y-4">
            <TranscriptionOverlay
              isRecording={isRecording}
              isProcessing={isProcessing}
              lastTranscription={lastTranscription}
            />

            <div className="text-center text-gray-400 text-sm">
              <p>Press <kbd className="px-2 py-1 bg-gray-700 rounded">F6</kbd> to start dictating</p>
            </div>
          </div>
        )}

        {view === 'settings' && (
          <Settings onBack={() => setView('main')} />
        )}

        {view === 'history' && (
          <History onBack={() => setView('main')} />
        )}
      </main>
    </div>
  );
}

export default App;
