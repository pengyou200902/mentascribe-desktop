import { FC } from 'react';

interface MenuBarProps {
  currentView: 'main' | 'settings' | 'history';
  onViewChange: (view: 'main' | 'settings' | 'history') => void;
  isRecording: boolean;
}

export const MenuBar: FC<MenuBarProps> = ({ currentView, onViewChange, isRecording }) => {
  return (
    <header className="flex items-center justify-between p-4 border-b border-gray-700">
      <div className="flex items-center gap-2">
        <div
          className={`w-3 h-3 rounded-full ${
            isRecording ? 'bg-red-500 animate-pulse' : 'bg-gray-500'
          }`}
        />
        <span className="font-semibold">MentaScribe</span>
      </div>

      <nav className="flex gap-2">
        <button
          onClick={() => onViewChange('main')}
          className={`px-3 py-1 rounded text-sm ${
            currentView === 'main'
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          Home
        </button>
        <button
          onClick={() => onViewChange('history')}
          className={`px-3 py-1 rounded text-sm ${
            currentView === 'history'
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          History
        </button>
        <button
          onClick={() => onViewChange('settings')}
          className={`px-3 py-1 rounded text-sm ${
            currentView === 'settings'
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:text-white'
          }`}
        >
          Settings
        </button>
      </nav>
    </header>
  );
};
