import { Settings } from '../Settings';

export function SettingsPage() {
  return (
    <div className="h-full overflow-y-auto bg-white">
      <div className="max-w-2xl mx-auto px-8 py-8">
        <div className="mb-6">
          <h1 className="text-2xl font-semibold text-gray-900">Settings</h1>
          <p className="text-sm text-gray-500 mt-1">
            Configure transcription, hotkeys, and output preferences
          </p>
        </div>
        <div className="settings-light-mode">
          <Settings onBack={() => {}} embedded />
        </div>
      </div>
    </div>
  );
}
