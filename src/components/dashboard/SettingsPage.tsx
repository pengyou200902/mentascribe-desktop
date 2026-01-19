import { FC, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore, UserSettings } from '../../lib/store';
import { useTheme } from '../../lib/theme';

// Icons
const SunIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" />
  </svg>
);

const MoonIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z" />
  </svg>
);

const SystemIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M9 17.25v1.007a3 3 0 01-.879 2.122L7.5 21h9l-.621-.621A3 3 0 0115 18.257V17.25m6-12V15a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 15V5.25m18 0A2.25 2.25 0 0018.75 3H5.25A2.25 2.25 0 003 5.25m18 0V12a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 12V5.25" />
  </svg>
);

const MicrophoneIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />
  </svg>
);

const KeyboardIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z" />
  </svg>
);

const OutputIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
  </svg>
);

const SparklesIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
  </svg>
);

const PaletteIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M4.098 19.902a3.75 3.75 0 005.304 0l6.401-6.402M6.75 21A3.75 3.75 0 013 17.25V4.125C3 3.504 3.504 3 4.125 3h5.25c.621 0 1.125.504 1.125 1.125v4.072M6.75 21a3.75 3.75 0 003.75-3.75V8.197M6.75 21h13.125c.621 0 1.125-.504 1.125-1.125v-5.25c0-.621-.504-1.125-1.125-1.125h-4.072M10.5 8.197l2.88-2.88c.438-.439 1.15-.439 1.59 0l3.712 3.713c.44.44.44 1.152 0 1.59l-2.879 2.88M6.75 17.25h.008v.008H6.75v-.008z" />
  </svg>
);

const CheckIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
  </svg>
);

const DownloadIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
  </svg>
);

interface ModelInfo {
  id: string;
  name: string;
  size_mb: number;
  downloaded: boolean;
}

// Section Component
interface SettingsSectionProps {
  icon: React.ReactNode;
  title: string;
  description?: string;
  children: React.ReactNode;
}

function SettingsSection({ icon, title, description, children }: SettingsSectionProps) {
  return (
    <section className="rounded-2xl border border-stone-100 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-800/30 overflow-hidden animate-fade-in">
      <div className="px-5 py-4 border-b border-stone-100 dark:border-stone-800 bg-white dark:bg-stone-800/50">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-xl bg-stone-100 dark:bg-stone-700/50 text-stone-500 dark:text-stone-400">
            {icon}
          </div>
          <div>
            <h3 className="font-semibold text-stone-900 dark:text-stone-100">{title}</h3>
            {description && (
              <p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">{description}</p>
            )}
          </div>
        </div>
      </div>
      <div className="p-5 space-y-4">
        {children}
      </div>
    </section>
  );
}

// Select Component
interface SelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}

function Select({ label, value, onChange, options }: SelectProps) {
  return (
    <div>
      <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
        {label}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-4 py-2.5 bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-700 rounded-xl text-stone-900 dark:text-stone-100 focus:outline-none focus:ring-2 focus:ring-amber-500/20 focus:border-amber-500 dark:focus:border-amber-400 transition-all duration-200 cursor-pointer"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
}

// Toggle Component
interface ToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function Toggle({ label, description, checked, onChange }: ToggleProps) {
  return (
    <label className="flex items-center justify-between cursor-pointer group">
      <div>
        <span className="text-sm font-medium text-stone-700 dark:text-stone-300 group-hover:text-stone-900 dark:group-hover:text-stone-100 transition-colors">
          {label}
        </span>
        {description && (
          <p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">{description}</p>
        )}
      </div>
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`
          relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200
          ${checked ? 'bg-amber-500 dark:bg-amber-400' : 'bg-stone-300 dark:bg-stone-600'}
        `}
      >
        <span
          className={`
            inline-block h-4 w-4 transform rounded-full bg-white shadow-sm transition-transform duration-200
            ${checked ? 'translate-x-6' : 'translate-x-1'}
          `}
        />
      </button>
    </label>
  );
}

// Input Component
interface InputProps {
  label: string;
  type?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

function Input({ label, type = 'text', value, onChange, placeholder }: InputProps) {
  return (
    <div>
      <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
        {label}
      </label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full px-4 py-2.5 bg-white dark:bg-stone-900 border border-stone-200 dark:border-stone-700 rounded-xl text-stone-900 dark:text-stone-100 placeholder-stone-400 dark:placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-amber-500/20 focus:border-amber-500 dark:focus:border-amber-400 transition-all duration-200"
      />
    </div>
  );
}

// Theme Selector Component
function ThemeSelector() {
  const { theme, setTheme } = useTheme();

  const themes = [
    { id: 'light' as const, icon: <SunIcon />, label: 'Light', description: 'Always use light theme' },
    { id: 'dark' as const, icon: <MoonIcon />, label: 'Dark', description: 'Always use dark theme' },
    { id: 'system' as const, icon: <SystemIcon />, label: 'System', description: 'Match system settings' },
  ];

  return (
    <div className="grid grid-cols-3 gap-3">
      {themes.map(({ id, icon, label, description }) => (
        <button
          key={id}
          onClick={() => setTheme(id)}
          className={`
            relative flex flex-col items-center gap-2 p-4 rounded-xl border-2 transition-all duration-200
            ${theme === id
              ? 'border-amber-500 dark:border-amber-400 bg-amber-50 dark:bg-amber-900/20'
              : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50 hover:border-stone-300 dark:hover:border-stone-600'
            }
          `}
        >
          {theme === id && (
            <div className="absolute top-2 right-2 w-5 h-5 bg-amber-500 dark:bg-amber-400 rounded-full flex items-center justify-center">
              <CheckIcon />
            </div>
          )}
          <div className={`p-2 rounded-lg ${theme === id ? 'text-amber-600 dark:text-amber-400' : 'text-stone-500 dark:text-stone-400'}`}>
            {icon}
          </div>
          <div className="text-center">
            <div className={`text-sm font-medium ${theme === id ? 'text-amber-700 dark:text-amber-400' : 'text-stone-700 dark:text-stone-300'}`}>
              {label}
            </div>
            <div className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
              {description}
            </div>
          </div>
        </button>
      ))}
    </div>
  );
}

export function SettingsPage() {
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

  if (!settings) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="flex items-center gap-3 text-stone-400 dark:text-stone-500">
          <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          <span className="text-sm">Loading settings...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-2xl mx-auto px-8 py-8">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-2xl font-semibold text-stone-900 dark:text-stone-100 tracking-tight">
            Settings
          </h1>
          <p className="text-sm text-stone-500 dark:text-stone-400 mt-0.5">
            Configure your preferences and transcription options
          </p>
        </div>

        <div className="space-y-6">
          {/* Appearance */}
          <SettingsSection
            icon={<PaletteIcon />}
            title="Appearance"
            description="Customize how MentaScribe looks"
          >
            <ThemeSelector />
          </SettingsSection>

          {/* Transcription */}
          <SettingsSection
            icon={<MicrophoneIcon />}
            title="Transcription"
            description="Speech recognition settings"
          >
            <Select
              label="Language"
              value={settings.transcription.language || 'auto'}
              onChange={(value) => handleChange('transcription', 'language', value)}
              options={[
                { value: 'auto', label: 'Auto-detect' },
                { value: 'en', label: 'English' },
                { value: 'es', label: 'Spanish' },
                { value: 'fr', label: 'French' },
                { value: 'de', label: 'German' },
                { value: 'zh', label: 'Chinese' },
                { value: 'ja', label: 'Japanese' },
              ]}
            />

            <div>
              <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-3">
                Speech Model
              </label>
              <div className="space-y-2">
                {models.map((model) => (
                  <div
                    key={model.id}
                    className={`
                      flex items-center justify-between p-3 rounded-xl border transition-all duration-200
                      ${settings.transcription.model_size === model.id
                        ? 'border-amber-500 dark:border-amber-400 bg-amber-50 dark:bg-amber-900/20'
                        : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50'
                      }
                    `}
                  >
                    <label className="flex items-center gap-3 cursor-pointer flex-1">
                      <input
                        type="radio"
                        name="model"
                        checked={settings.transcription.model_size === model.id}
                        onChange={() => handleChange('transcription', 'model_size', model.id)}
                        disabled={!model.downloaded}
                        className="w-4 h-4 text-amber-500 focus:ring-amber-500/20 border-stone-300 dark:border-stone-600"
                      />
                      <div>
                        <span className={`text-sm font-medium ${model.downloaded ? 'text-stone-900 dark:text-stone-100' : 'text-stone-400 dark:text-stone-500'}`}>
                          {model.name}
                        </span>
                        <span className="text-xs text-stone-500 dark:text-stone-400 ml-2">
                          ({model.size_mb}MB)
                        </span>
                      </div>
                    </label>

                    {model.downloaded ? (
                      <span className="flex items-center gap-1 text-xs font-medium text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-2 py-1 rounded-lg">
                        <CheckIcon />
                        Ready
                      </span>
                    ) : downloading === model.id ? (
                      <span className="flex items-center gap-2 text-xs font-medium text-amber-600 dark:text-amber-400">
                        <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                        Downloading...
                      </span>
                    ) : (
                      <button
                        onClick={() => downloadModel(model.id)}
                        className="flex items-center gap-1.5 text-xs font-medium text-amber-600 dark:text-amber-400 hover:text-amber-700 dark:hover:text-amber-300 bg-amber-100 dark:bg-amber-900/30 hover:bg-amber-200 dark:hover:bg-amber-900/50 px-3 py-1.5 rounded-lg transition-colors"
                      >
                        <DownloadIcon />
                        Download
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </SettingsSection>

          {/* Hotkey */}
          <SettingsSection
            icon={<KeyboardIcon />}
            title="Hotkey"
            description="Configure your activation shortcut"
          >
            <Select
              label="Activation Key"
              value={settings.hotkey.key || 'F6'}
              onChange={(value) => handleChange('hotkey', 'key', value)}
              options={['F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12'].map((key) => ({
                value: key,
                label: key,
              }))}
            />

            <Select
              label="Mode"
              value={settings.hotkey.mode || 'hold'}
              onChange={(value) => handleChange('hotkey', 'mode', value)}
              options={[
                { value: 'hold', label: 'Hold to talk' },
                { value: 'toggle', label: 'Toggle on/off' },
              ]}
            />
          </SettingsSection>

          {/* Output */}
          <SettingsSection
            icon={<OutputIcon />}
            title="Output"
            description="How text is inserted"
          >
            <Select
              label="Insert Method"
              value={settings.output.insert_method || 'paste'}
              onChange={(value) => handleChange('output', 'insert_method', value)}
              options={[
                { value: 'paste', label: 'Paste (use clipboard)' },
                { value: 'type', label: 'Type (simulate keystrokes)' },
              ]}
            />

            <Toggle
              label="Auto-capitalize sentences"
              description="Automatically capitalize the first letter of sentences"
              checked={settings.output.auto_capitalize ?? true}
              onChange={(checked) => handleChange('output', 'auto_capitalize', checked)}
            />
          </SettingsSection>

          {/* AI Cleanup */}
          <SettingsSection
            icon={<SparklesIcon />}
            title="AI Cleanup"
            description="Optional text enhancement"
          >
            <Toggle
              label="Enable AI text cleanup"
              description="Use AI to improve transcription quality"
              checked={settings.cleanup.enabled}
              onChange={(checked) => handleChange('cleanup', 'enabled', checked)}
            />

            {settings.cleanup.enabled && (
              <div className="space-y-4 pt-2 border-t border-stone-100 dark:border-stone-700 mt-4">
                <Select
                  label="Provider"
                  value={settings.cleanup.provider || 'openai'}
                  onChange={(value) => handleChange('cleanup', 'provider', value)}
                  options={[
                    { value: 'openai', label: 'OpenAI' },
                    { value: 'anthropic', label: 'Anthropic' },
                    { value: 'openrouter', label: 'OpenRouter' },
                    { value: 'ollama', label: 'Ollama (Local)' },
                  ]}
                />

                <Input
                  label="API Key"
                  type="password"
                  value={settings.cleanup.api_key || ''}
                  onChange={(value) => handleChange('cleanup', 'api_key', value)}
                  placeholder="Enter your API key"
                />

                <div className="space-y-3 pt-2">
                  <Toggle
                    label="Remove filler words"
                    description="Remove um, uh, like, etc."
                    checked={settings.cleanup.remove_filler}
                    onChange={(checked) => handleChange('cleanup', 'remove_filler', checked)}
                  />

                  <Toggle
                    label="Add punctuation"
                    description="Automatically add periods, commas, etc."
                    checked={settings.cleanup.add_punctuation}
                    onChange={(checked) => handleChange('cleanup', 'add_punctuation', checked)}
                  />

                  <Toggle
                    label="Format paragraphs"
                    description="Break text into logical paragraphs"
                    checked={settings.cleanup.format_paragraphs}
                    onChange={(checked) => handleChange('cleanup', 'format_paragraphs', checked)}
                  />
                </div>
              </div>
            )}
          </SettingsSection>
        </div>

        {/* Footer */}
        <div className="mt-8 pt-6 border-t border-stone-100 dark:border-stone-800">
          <p className="text-xs text-stone-400 dark:text-stone-500 text-center">
            Settings are saved automatically and persist across sessions
          </p>
        </div>
      </div>
    </div>
  );
}
