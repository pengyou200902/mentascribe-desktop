import { FC, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore, UserSettings } from '../lib/store';

interface SettingsProps {
  onBack: () => void;
  embedded?: boolean;
}

interface ModelInfo {
  id: string;
  name: string;
  size_mb: number;
  downloaded: boolean;
}

export const Settings: FC<SettingsProps> = ({ onBack, embedded = false }) => {
  const { settings, updateSettings } = useStore();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);

  useEffect(() => {
    loadModels();
  }, []);

  async function loadModels() {
    try {
      const availableModels = await invoke<ModelInfo[]>('get_available_models');
      setModels(availableModels);
    } catch (error) {
      console.error('Failed to load models:', error);
    }
  }

  async function downloadModel(modelId: string) {
    setDownloading(modelId);
    try {
      await invoke('download_model', { size: modelId });
      await loadModels();
    } catch (error) {
      console.error('Failed to download model:', error);
    }
    setDownloading(null);
  }

  function handleChange<K extends keyof UserSettings>(
    section: K,
    key: keyof UserSettings[K],
    value: any
  ) {
    if (!settings) return;

    const newSettings = {
      ...settings,
      [section]: {
        ...settings[section],
        [key]: value,
      },
    };

    updateSettings(newSettings);
  }

  if (!settings) return <div>Loading...</div>;

  return (
    <div className="space-y-6">
      {!embedded && (
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Settings</h2>
          <button
            onClick={onBack}
            className="text-gray-400 hover:text-white px-3 py-1 rounded hover:bg-gray-700"
          >
            Close
          </button>
        </div>
      )}

      {/* Transcription Settings */}
      <section className="bg-gray-800 rounded-lg p-4">
        <h3 className="font-medium mb-4">Transcription</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Language</label>
            <select
              value={settings.transcription.language || 'auto'}
              onChange={(e) =>
                handleChange('transcription', 'language', e.target.value)
              }
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
            >
              <option value="auto">Auto-detect</option>
              <option value="en">English</option>
              <option value="es">Spanish</option>
              <option value="fr">French</option>
              <option value="de">German</option>
              <option value="zh">Chinese</option>
              <option value="ja">Japanese</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-2">Model</label>
            <div className="space-y-2">
              {models.map((model) => (
                <div
                  key={model.id}
                  className="flex items-center justify-between bg-gray-700 rounded px-3 py-2"
                >
                  <div className="flex items-center gap-2">
                    <input
                      type="radio"
                      name="model"
                      checked={settings.transcription.model_size === model.id}
                      onChange={() =>
                        handleChange('transcription', 'model_size', model.id)
                      }
                      disabled={!model.downloaded}
                      className="text-blue-500"
                    />
                    <span>{model.name}</span>
                  </div>

                  {model.downloaded ? (
                    <span className="text-green-500 text-sm">Downloaded</span>
                  ) : downloading === model.id ? (
                    <span className="text-yellow-500 text-sm">Downloading...</span>
                  ) : (
                    <button
                      onClick={() => downloadModel(model.id)}
                      className="text-blue-500 text-sm hover:underline"
                    >
                      Download
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Hotkey Settings */}
      <section className="bg-gray-800 rounded-lg p-4">
        <h3 className="font-medium mb-4">Hotkey</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Activation Key</label>
            <select
              value={settings.hotkey.key || 'F6'}
              onChange={(e) => handleChange('hotkey', 'key', e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
            >
              {['F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12'].map(
                (key) => (
                  <option key={key} value={key}>
                    {key}
                  </option>
                )
              )}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Mode</label>
            <select
              value={settings.hotkey.mode || 'hold'}
              onChange={(e) => handleChange('hotkey', 'mode', e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
            >
              <option value="hold">Hold to talk</option>
              <option value="toggle">Toggle on/off</option>
            </select>
          </div>
        </div>
      </section>

      {/* Output Settings */}
      <section className="bg-gray-800 rounded-lg p-4">
        <h3 className="font-medium mb-4">Output</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Insert Method</label>
            <select
              value={settings.output.insert_method || 'auto'}
              onChange={(e) =>
                handleChange('output', 'insert_method', e.target.value)
              }
              className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
            >
              <option value="auto">Auto (smart detection, recommended)</option>
              <option value="paste">Paste (use clipboard)</option>
              <option value="type">Type (simulate keystrokes)</option>
            </select>
          </div>

          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={settings.output.auto_capitalize ?? true}
              onChange={(e) =>
                handleChange('output', 'auto_capitalize', e.target.checked)
              }
              className="text-blue-500"
            />
            <span>Auto-capitalize sentences</span>
          </label>
        </div>
      </section>

      {/* Widget Settings */}
      <section className="bg-gray-800 rounded-lg p-4">
        <h3 className="font-medium mb-4">Widget</h3>

        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={settings.widget?.draggable ?? false}
            onChange={(e) =>
              handleChange('widget', 'draggable', e.target.checked)
            }
            className="text-blue-500"
          />
          <span>Draggable widget</span>
        </label>
        <p className="text-sm text-gray-400 mt-1 ml-6">
          Drag the floating bar to any position on screen
        </p>
      </section>

      {/* AI Cleanup Settings */}
      <section className="bg-gray-800 rounded-lg p-4">
        <h3 className="font-medium mb-4">AI Cleanup (Optional)</h3>

        <label className="flex items-center gap-2 mb-4">
          <input
            type="checkbox"
            checked={settings.cleanup.enabled}
            onChange={(e) =>
              handleChange('cleanup', 'enabled', e.target.checked)
            }
            className="text-blue-500"
          />
          <span>Enable AI text cleanup</span>
        </label>

        {settings.cleanup.enabled && (
          <div className="space-y-4 ml-6">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Provider</label>
              <select
                value={settings.cleanup.provider || 'openai'}
                onChange={(e) =>
                  handleChange('cleanup', 'provider', e.target.value)
                }
                className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
              >
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="openrouter">OpenRouter</option>
                <option value="ollama">Ollama (Local)</option>
              </select>
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-1">API Key</label>
              <input
                type="password"
                value={settings.cleanup.api_key || ''}
                onChange={(e) =>
                  handleChange('cleanup', 'api_key', e.target.value)
                }
                placeholder="Enter your API key"
                className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
              />
            </div>

            <div className="space-y-2">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.cleanup.remove_filler}
                  onChange={(e) =>
                    handleChange('cleanup', 'remove_filler', e.target.checked)
                  }
                  className="text-blue-500"
                />
                <span>Remove filler words</span>
              </label>

              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.cleanup.add_punctuation}
                  onChange={(e) =>
                    handleChange('cleanup', 'add_punctuation', e.target.checked)
                  }
                  className="text-blue-500"
                />
                <span>Add punctuation</span>
              </label>

              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={settings.cleanup.format_paragraphs}
                  onChange={(e) =>
                    handleChange('cleanup', 'format_paragraphs', e.target.checked)
                  }
                  className="text-blue-500"
                />
                <span>Format paragraphs</span>
              </label>
            </div>
          </div>
        )}
      </section>
    </div>
  );
};
