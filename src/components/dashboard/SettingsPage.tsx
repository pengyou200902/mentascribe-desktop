import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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

const WidgetIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15" />
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

const ChevronDownIcon = ({ className = "w-4 h-4" }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
  </svg>
);

// Language flag/icon components
const GlobeIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418" />
  </svg>
);

const HoldIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.042 21.672L13.684 16.6m0 0l-2.51 2.225.569-9.47 5.227 7.917-3.286-.672zM12 2.25V4.5m5.834.166l-1.591 1.591M20.25 10.5H18M7.757 14.743l-1.59 1.59M6 10.5H3.75m4.007-4.243l-1.59-1.59" />
  </svg>
);

const ToggleIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5.636 5.636a9 9 0 1012.728 0M12 3v9" />
  </svg>
);

const ClipboardIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
  </svg>
);

const TypewriterIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
  </svg>
);

const RecordIcon = () => (
  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="8" />
  </svg>
);

const StopIcon = () => (
  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
    <rect x="6" y="6" width="12" height="12" rx="2" />
  </svg>
);

const ClearIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
  </svg>
);

const TrashIcon = () => (
  <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
  </svg>
);

interface ModelInfo {
  id: string;
  name: string;
  size_mb: number;
  downloaded: boolean;
  coreml_downloaded: boolean;
  coreml_size_mb: number;
}

interface CoremlStatus {
  compiled: boolean;
  supported: boolean;
  apple_silicon: boolean;
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

// Custom Dropdown Component
interface DropdownOption {
  value: string;
  label: string;
  icon?: React.ReactNode;
  description?: string;
}

interface DropdownProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: DropdownOption[];
  placeholder?: string;
}

function Dropdown({ label, value, onChange, options, placeholder }: DropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const selectedOption = options.find(opt => opt.value === value);

  // Close on outside click
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Keyboard navigation
  useEffect(() => {
    if (!isOpen) return;

    function handleKeyDown(event: KeyboardEvent) {
      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setHighlightedIndex(prev => Math.min(prev + 1, options.length - 1));
          break;
        case 'ArrowUp':
          event.preventDefault();
          setHighlightedIndex(prev => Math.max(prev - 1, 0));
          break;
        case 'Enter':
          event.preventDefault();
          if (highlightedIndex >= 0) {
            onChange(options[highlightedIndex].value);
            setIsOpen(false);
          }
          break;
        case 'Escape':
          setIsOpen(false);
          break;
      }
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, highlightedIndex, options, onChange]);

  // Scroll highlighted item into view
  useEffect(() => {
    if (isOpen && highlightedIndex >= 0 && listRef.current) {
      const items = listRef.current.children;
      if (items[highlightedIndex]) {
        items[highlightedIndex].scrollIntoView({ block: 'nearest' });
      }
    }
  }, [highlightedIndex, isOpen]);

  return (
    <div ref={dropdownRef} className="relative">
      <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
        {label}
      </label>

      {/* Trigger Button */}
      <button
        type="button"
        onClick={() => {
          setIsOpen(!isOpen);
          setHighlightedIndex(options.findIndex(opt => opt.value === value));
        }}
        className={`
          w-full flex items-center justify-between gap-3 px-4 py-3
          bg-white dark:bg-stone-900
          border-2 rounded-xl
          transition-all duration-200 ease-out
          ${isOpen
            ? 'border-amber-500 dark:border-amber-400 ring-4 ring-amber-500/10 dark:ring-amber-400/10'
            : 'border-stone-200 dark:border-stone-700 hover:border-stone-300 dark:hover:border-stone-600'
          }
        `}
      >
        <div className="flex items-center gap-3 min-w-0">
          {selectedOption?.icon && (
            <span className={`flex-shrink-0 ${isOpen ? 'text-amber-600 dark:text-amber-400' : 'text-stone-400 dark:text-stone-500'} transition-colors`}>
              {selectedOption.icon}
            </span>
          )}
          <div className="text-left min-w-0">
            <span className={`block text-sm font-medium truncate ${selectedOption ? 'text-stone-900 dark:text-stone-100' : 'text-stone-400 dark:text-stone-500'}`}>
              {selectedOption?.label || placeholder || 'Select...'}
            </span>
            {selectedOption?.description && (
              <span className="block text-xs text-stone-500 dark:text-stone-400 truncate mt-0.5">
                {selectedOption.description}
              </span>
            )}
          </div>
        </div>
        <ChevronDownIcon
          className={`w-5 h-5 flex-shrink-0 text-stone-400 dark:text-stone-500 transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
        />
      </button>

      {/* Dropdown Panel */}
      {isOpen && (
        <div
          className="absolute z-50 w-full mt-2 py-2 bg-white dark:bg-stone-900 border-2 border-stone-200 dark:border-stone-700 rounded-xl shadow-xl shadow-stone-900/10 dark:shadow-black/30 overflow-hidden animate-dropdown-in"
          style={{ maxHeight: '280px' }}
        >
          <div ref={listRef} className="overflow-y-auto max-h-64 scrollbar-thin">
            {options.map((option, index) => {
              const isSelected = option.value === value;
              const isHighlighted = index === highlightedIndex;

              return (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => {
                    onChange(option.value);
                    setIsOpen(false);
                  }}
                  onMouseEnter={() => setHighlightedIndex(index)}
                  className={`
                    w-full flex items-center gap-3 px-4 py-3 text-left
                    transition-all duration-100
                    ${isHighlighted ? 'bg-stone-100 dark:bg-stone-800' : ''}
                    ${isSelected ? 'bg-amber-50 dark:bg-amber-900/20' : ''}
                  `}
                >
                  {/* Icon or Selection Indicator */}
                  <span className={`
                    flex-shrink-0 w-5 h-5 flex items-center justify-center
                    ${isSelected ? 'text-amber-600 dark:text-amber-400' : 'text-stone-400 dark:text-stone-500'}
                  `}>
                    {option.icon || (isSelected && <CheckIcon />)}
                  </span>

                  {/* Label & Description */}
                  <div className="flex-1 min-w-0">
                    <span className={`
                      block text-sm font-medium truncate
                      ${isSelected ? 'text-amber-700 dark:text-amber-400' : 'text-stone-900 dark:text-stone-100'}
                    `}>
                      {option.label}
                    </span>
                    {option.description && (
                      <span className="block text-xs text-stone-500 dark:text-stone-400 truncate mt-0.5">
                        {option.description}
                      </span>
                    )}
                  </div>

                  {/* Selected Checkmark */}
                  {isSelected && !option.icon && (
                    <span className="flex-shrink-0 text-amber-500 dark:text-amber-400">
                      <CheckIcon />
                    </span>
                  )}
                  {isSelected && option.icon && (
                    <span className="flex-shrink-0 w-2 h-2 rounded-full bg-amber-500 dark:bg-amber-400" />
                  )}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

// Card Select Component for Mode Selection
interface CardSelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string; icon: React.ReactNode; description: string }[];
}

function CardSelect({ label, value, onChange, options }: CardSelectProps) {
  return (
    <div>
      <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-3">
        {label}
      </label>
      <div className="grid grid-cols-2 gap-3">
        {options.map((option) => {
          const isSelected = option.value === value;
          return (
            <button
              key={option.value}
              type="button"
              onClick={() => onChange(option.value)}
              className={`
                relative flex flex-col items-start gap-2 p-4 rounded-xl text-left
                border-2 transition-all duration-200 ease-out
                ${isSelected
                  ? 'border-amber-500 dark:border-amber-400 bg-amber-50 dark:bg-amber-900/20 shadow-lg shadow-amber-500/10'
                  : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50 hover:border-stone-300 dark:hover:border-stone-600'
                }
              `}
            >
              {isSelected && (
                <span className="absolute top-3 right-3 w-5 h-5 bg-amber-500 dark:bg-amber-400 rounded-full flex items-center justify-center text-white">
                  <CheckIcon />
                </span>
              )}
              <span className={`
                p-2 rounded-lg
                ${isSelected
                  ? 'bg-amber-100 dark:bg-amber-900/40 text-amber-600 dark:text-amber-400'
                  : 'bg-stone-100 dark:bg-stone-700/50 text-stone-500 dark:text-stone-400'
                }
              `}>
                {option.icon}
              </span>
              <div>
                <span className={`
                  block text-sm font-semibold
                  ${isSelected ? 'text-amber-700 dark:text-amber-400' : 'text-stone-900 dark:text-stone-100'}
                `}>
                  {option.label}
                </span>
                <span className="block text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                  {option.description}
                </span>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

// Hotkey Recorder Component
interface HotkeyRecorderProps {
  value: string;
  onChange: (value: string) => void;
}

interface ParsedHotkey {
  modifiers: string[];
  key: string | null;
}

function parseHotkey(hotkey: string): ParsedHotkey {
  if (!hotkey) return { modifiers: [], key: null };

  const parts = hotkey.split('+').map(p => p.trim());
  const modifierNames = ['Ctrl', 'Alt', 'Shift', 'Meta', 'Cmd', 'Control', 'Option'];
  const modifiers: string[] = [];
  let key: string | null = null;

  for (const part of parts) {
    if (modifierNames.some(m => m.toLowerCase() === part.toLowerCase())) {
      // Normalize modifier names
      let normalizedMod = part;
      if (part.toLowerCase() === 'control') normalizedMod = 'Ctrl';
      if (part.toLowerCase() === 'option') normalizedMod = 'Alt';
      if (part.toLowerCase() === 'cmd') normalizedMod = 'Meta';
      modifiers.push(normalizedMod.charAt(0).toUpperCase() + normalizedMod.slice(1).toLowerCase());
    } else {
      key = part;
    }
  }

  return { modifiers, key };
}

function HotkeyRecorder({ value, onChange }: HotkeyRecorderProps) {
  const [isRecording, setIsRecording] = useState(false);
  const [currentModifiers, setCurrentModifiers] = useState<Set<string>>(new Set());
  const [currentKey, setCurrentKey] = useState<string | null>(null);
  const [showSuccess, setShowSuccess] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const parsed = parseHotkey(value);

  // Get display name for a key
  const getKeyDisplayName = (key: string): string => {
    const keyMap: Record<string, string> = {
      ' ': 'Space',
      'ArrowUp': '↑',
      'ArrowDown': '↓',
      'ArrowLeft': '←',
      'ArrowRight': '→',
      'Escape': 'Esc',
      'Delete': 'Del',
      'Backspace': '⌫',
      'Enter': '↵',
      'Tab': '⇥',
      'CapsLock': 'Caps',
      'Control': 'Ctrl',
      'Meta': '⌘',
    };
    return keyMap[key] || key;
  };

  // Get platform-specific modifier symbol
  const getModifierSymbol = (modifier: string): { symbol: string; label: string } => {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const modifierMap: Record<string, { symbol: string; label: string }> = {
      'Ctrl': { symbol: isMac ? '⌃' : 'Ctrl', label: 'Control' },
      'Alt': { symbol: isMac ? '⌥' : 'Alt', label: isMac ? 'Option' : 'Alt' },
      'Shift': { symbol: isMac ? '⇧' : 'Shift', label: 'Shift' },
      'Meta': { symbol: isMac ? '⌘' : '⊞', label: isMac ? 'Command' : 'Windows' },
    };
    return modifierMap[modifier] || { symbol: modifier, label: modifier };
  };

  useEffect(() => {
    if (!isRecording) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const newModifiers = new Set<string>();
      if (e.ctrlKey) newModifiers.add('Ctrl');
      if (e.altKey) newModifiers.add('Alt');
      if (e.shiftKey) newModifiers.add('Shift');
      if (e.metaKey) newModifiers.add('Meta');

      setCurrentModifiers(newModifiers);

      // Check if this is a non-modifier key
      const modifierKeys = ['Control', 'Alt', 'Shift', 'Meta'];
      if (!modifierKeys.includes(e.key)) {
        const keyName = e.key.length === 1 ? e.key.toUpperCase() : e.key;
        setCurrentKey(keyName);

        // Build the final hotkey string
        const modArray = Array.from(newModifiers);
        const hotkeyString = [...modArray, keyName].join('+');

        onChange(hotkeyString);
        setIsRecording(false);
        setShowSuccess(true);
        setTimeout(() => setShowSuccess(false), 1500);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault();

      const newModifiers = new Set<string>();
      if (e.ctrlKey) newModifiers.add('Ctrl');
      if (e.altKey) newModifiers.add('Alt');
      if (e.shiftKey) newModifiers.add('Shift');
      if (e.metaKey) newModifiers.add('Meta');

      setCurrentModifiers(newModifiers);
    };

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
  }, [isRecording, onChange]);

  // Click outside to cancel recording
  useEffect(() => {
    if (!isRecording) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsRecording(false);
        setCurrentModifiers(new Set());
        setCurrentKey(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isRecording]);

  const startRecording = () => {
    setIsRecording(true);
    setCurrentModifiers(new Set());
    setCurrentKey(null);
  };

  const stopRecording = () => {
    setIsRecording(false);
    setCurrentModifiers(new Set());
    setCurrentKey(null);
  };

  const clearHotkey = () => {
    onChange('');
    setCurrentModifiers(new Set());
    setCurrentKey(null);
  };

  // Render a single key cap
  const KeyCap = ({ children, isModifier = false, isActive = false }: { children: React.ReactNode; isModifier?: boolean; isActive?: boolean }) => (
    <span
      className={`
        inline-flex items-center justify-center
        px-3 py-2 min-w-[2.5rem]
        rounded-lg font-mono text-sm font-semibold
        transition-all duration-150
        ${isActive
          ? 'bg-amber-500 text-white shadow-lg shadow-amber-500/30 scale-105'
          : isModifier
            ? 'bg-stone-200 dark:bg-stone-700 text-stone-600 dark:text-stone-300'
            : 'bg-stone-100 dark:bg-stone-800 text-stone-900 dark:text-stone-100 border-2 border-stone-300 dark:border-stone-600'
        }
        ${isModifier ? 'text-xs' : 'text-sm'}
      `}
      style={{
        boxShadow: isActive ? undefined : '0 2px 0 0 rgba(0,0,0,0.1)',
      }}
    >
      {children}
    </span>
  );

  // Display keys for current state or saved value
  const displayModifiers = isRecording ? Array.from(currentModifiers) : parsed.modifiers;
  const displayKey = isRecording ? currentKey : parsed.key;
  const hasHotkey = displayModifiers.length > 0 || displayKey;

  return (
    <div ref={containerRef} className="space-y-4">
      <label className="block text-sm font-medium text-stone-700 dark:text-stone-300">
        Activation Shortcut
      </label>

      {/* Main Hotkey Display */}
      <div
        className={`
          relative overflow-hidden
          rounded-2xl border-2 p-6
          transition-all duration-300 ease-out
          ${isRecording
            ? 'border-amber-500 dark:border-amber-400 bg-gradient-to-br from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20'
            : showSuccess
              ? 'border-green-500 dark:border-green-400 bg-green-50 dark:bg-green-900/20'
              : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50'
          }
        `}
      >
        {/* Animated background when recording */}
        {isRecording && (
          <div className="absolute inset-0 overflow-hidden pointer-events-none">
            <div className="absolute inset-0 bg-gradient-to-r from-amber-500/5 via-orange-500/10 to-amber-500/5 animate-shimmer" />
            <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-amber-500 via-orange-500 to-amber-500 animate-pulse-bar" />
          </div>
        )}

        {/* Success checkmark animation */}
        {showSuccess && (
          <div className="absolute top-3 right-3 w-6 h-6 bg-green-500 rounded-full flex items-center justify-center animate-scale-in">
            <CheckIcon />
          </div>
        )}

        <div className="relative flex flex-col items-center gap-4">
          {/* Status text */}
          <div className="text-center">
            {isRecording ? (
              <div className="space-y-1">
                <p className="text-sm font-medium text-amber-700 dark:text-amber-400 animate-pulse">
                  Press your desired key combination...
                </p>
                <p className="text-xs text-stone-500 dark:text-stone-400">
                  Use modifiers like Ctrl, Alt, Shift, or ⌘ with any key
                </p>
              </div>
            ) : hasHotkey ? (
              <p className="text-xs text-stone-500 dark:text-stone-400 mb-2">
                Current shortcut
              </p>
            ) : (
              <p className="text-sm text-stone-500 dark:text-stone-400">
                No shortcut configured
              </p>
            )}
          </div>

          {/* Key display */}
          {hasHotkey && (
            <div className="flex items-center gap-2 flex-wrap justify-center">
              {displayModifiers.map((mod) => {
                const { symbol } = getModifierSymbol(mod);
                return (
                  <KeyCap key={mod} isModifier isActive={isRecording}>
                    {symbol}
                  </KeyCap>
                );
              })}
              {displayModifiers.length > 0 && displayKey && (
                <span className="text-stone-400 dark:text-stone-500 font-bold">+</span>
              )}
              {displayKey && (
                <KeyCap isActive={isRecording}>
                  {getKeyDisplayName(displayKey)}
                </KeyCap>
              )}
            </div>
          )}

          {/* Placeholder keys when empty and not recording */}
          {!hasHotkey && !isRecording && (
            <div className="flex items-center gap-2 opacity-40">
              <KeyCap isModifier>Ctrl</KeyCap>
              <span className="text-stone-400 dark:text-stone-500 font-bold">+</span>
              <KeyCap>?</KeyCap>
            </div>
          )}
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2">
        {isRecording ? (
          <button
            type="button"
            onClick={stopRecording}
            className="
              flex-1 flex items-center justify-center gap-2
              px-4 py-3 rounded-xl
              bg-stone-100 dark:bg-stone-800
              text-stone-700 dark:text-stone-300
              font-medium text-sm
              hover:bg-stone-200 dark:hover:bg-stone-700
              transition-colors duration-200
            "
          >
            <StopIcon />
            Cancel
          </button>
        ) : (
          <>
            <button
              type="button"
              onClick={startRecording}
              className="
                flex-1 flex items-center justify-center gap-2
                px-4 py-3 rounded-xl
                bg-amber-500 hover:bg-amber-600
                text-white font-medium text-sm
                shadow-lg shadow-amber-500/25
                hover:shadow-amber-500/40
                transition-all duration-200
                hover:scale-[1.02] active:scale-[0.98]
              "
            >
              <RecordIcon />
              {hasHotkey ? 'Change Shortcut' : 'Record Shortcut'}
            </button>
            {hasHotkey && (
              <button
                type="button"
                onClick={clearHotkey}
                className="
                  flex items-center justify-center
                  px-4 py-3 rounded-xl
                  bg-stone-100 dark:bg-stone-800
                  text-stone-500 dark:text-stone-400
                  hover:text-red-500 dark:hover:text-red-400
                  hover:bg-red-50 dark:hover:bg-red-900/20
                  transition-all duration-200
                "
                title="Clear shortcut"
              >
                <ClearIcon />
              </button>
            )}
          </>
        )}
      </div>

      {/* Quick presets */}
      <div className="pt-2">
        <p className="text-xs text-stone-500 dark:text-stone-400 mb-2">Quick presets</p>
        <div className="flex flex-wrap gap-2">
          {[
            { label: 'F6', value: 'F6' },
            { label: 'F8', value: 'F8' },
            { label: 'Ctrl+Space', value: 'Ctrl+Space' },
            { label: 'Alt+S', value: 'Alt+S' },
            { label: '⌘+Shift+V', value: 'Meta+Shift+V' },
          ].map((preset) => (
            <button
              key={preset.value}
              type="button"
              onClick={() => {
                onChange(preset.value);
                setShowSuccess(true);
                setTimeout(() => setShowSuccess(false), 1500);
              }}
              className={`
                px-3 py-1.5 rounded-lg text-xs font-medium
                transition-all duration-200
                ${value === preset.value
                  ? 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 ring-1 ring-amber-500/50'
                  : 'bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 hover:bg-stone-200 dark:hover:bg-stone-700'
                }
              `}
            >
              {preset.label}
            </button>
          ))}
        </div>
      </div>
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
  const [coremlStatus, setCoremlStatus] = useState<CoremlStatus | null>(null);
  const [downloadingCoreml, setDownloadingCoreml] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<Record<string, number>>({});
  const [deleting, setDeleting] = useState<string | null>(null);

  useEffect(() => {
    loadModels();
    loadCoremlStatus();
  }, []);

  // Listen for download progress events from the backend
  useEffect(() => {
    const unlisten = listen<{ model_type: string; model_id: string; percent: number }>(
      'download-progress',
      (event) => {
        const key = `${event.payload.model_type}:${event.payload.model_id}`;
        setDownloadProgress((prev) => ({ ...prev, [key]: event.payload.percent }));
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function loadCoremlStatus() {
    try {
      const status = await invoke<CoremlStatus>('get_coreml_status');
      setCoremlStatus(status);
    } catch (error) {
      console.error('Failed to load CoreML status:', error);
    }
  }

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
    setDownloadProgress((prev) => ({ ...prev, [`ggml:${modelId}`]: 0 }));
    try {
      await invoke('download_model', { size: modelId });
      await loadModels();
    } catch (error) {
      console.error('Failed to download model:', error);
    }
    setDownloading(null);
    setDownloadProgress((prev) => {
      const next = { ...prev };
      delete next[`ggml:${modelId}`];
      return next;
    });
  }

  async function downloadCoremlModel(modelId: string) {
    setDownloadingCoreml(modelId);
    setDownloadProgress((prev) => ({ ...prev, [`coreml:${modelId}`]: 0 }));
    try {
      await invoke('download_coreml_model', { size: modelId });
      await loadModels();
    } catch (error) {
      console.error('Failed to download CoreML model:', error);
    }
    setDownloadingCoreml(null);
    setDownloadProgress((prev) => {
      const next = { ...prev };
      delete next[`coreml:${modelId}`];
      return next;
    });
  }

  async function handleDeleteModel(modelId: string) {
    if (deleting) return;
    setDeleting(`ggml:${modelId}`);
    try {
      await invoke('delete_model', { size: modelId });
      // If deleted model was selected, clear selection
      if (settings?.transcription.model_size === modelId) {
        handleChange('transcription', 'model_size', '');
      }
      await loadModels();
    } catch (error) {
      console.error('Failed to delete model:', error);
    }
    setDeleting(null);
  }

  async function handleDeleteCoremlModel(modelId: string) {
    if (deleting) return;
    setDeleting(`coreml:${modelId}`);
    try {
      await invoke('delete_coreml_model', { size: modelId });
      await loadModels();
    } catch (error) {
      console.error('Failed to delete CoreML model:', error);
    }
    setDeleting(null);
  }

  function formatSize(mb: number): string {
    if (mb >= 1000) return `${(mb / 1000).toFixed(1)}GB`;
    return `${mb}MB`;
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

          {/* Widget */}
          <SettingsSection
            icon={<WidgetIcon />}
            title="Widget"
            description="Floating dictation bar behavior"
          >
            <Toggle
              label="Draggable widget"
              description="Drag the floating bar to any position on screen"
              checked={settings.widget?.draggable ?? false}
              onChange={(checked) => handleChange('widget', 'draggable', checked)}
            />
            <div className="pt-2">
              <div className="flex items-center justify-between mb-2">
                <div>
                  <span className="text-sm font-medium text-stone-700 dark:text-stone-300">Opacity</span>
                  <p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">Adjust the widget transparency</p>
                </div>
                <span className="text-sm font-medium text-stone-500 dark:text-stone-400 tabular-nums">
                  {Math.round((settings.widget?.opacity ?? 1.0) * 100)}%
                </span>
              </div>
              <input
                type="range"
                min="20"
                max="100"
                step="5"
                value={Math.round((settings.widget?.opacity ?? 1.0) * 100)}
                onChange={(e) => handleChange('widget', 'opacity', parseInt(e.target.value) / 100)}
                className="w-full h-1.5 rounded-full appearance-none cursor-pointer bg-stone-200 dark:bg-stone-700 accent-amber-500 dark:accent-amber-400"
              />
            </div>
          </SettingsSection>

          {/* Transcription */}
          <SettingsSection
            icon={<MicrophoneIcon />}
            title="Transcription"
            description="Speech recognition settings"
          >
            <Dropdown
              label="Language"
              value={settings.transcription.language || 'auto'}
              onChange={(value) => handleChange('transcription', 'language', value)}
              options={[
                { value: 'auto', label: 'Auto-detect', icon: <GlobeIcon />, description: 'Automatically detect language' },
                { value: 'en', label: 'English', description: 'United States, UK, Australia' },
                { value: 'es', label: 'Spanish', description: 'Spain, Latin America' },
                { value: 'fr', label: 'French', description: 'France, Canada, Belgium' },
                { value: 'de', label: 'German', description: 'Germany, Austria, Switzerland' },
                { value: 'zh', label: 'Chinese', description: 'Simplified & Traditional' },
                { value: 'ja', label: 'Japanese', description: 'Japan' },
                { value: 'ko', label: 'Korean', description: 'South Korea' },
                { value: 'pt', label: 'Portuguese', description: 'Portugal, Brazil' },
                { value: 'it', label: 'Italian', description: 'Italy' },
                { value: 'ru', label: 'Russian', description: 'Russia' },
              ]}
            />

            <div>
              <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-3">
                Speech Model
              </label>
              <div className="space-y-2">
                {models.map((model) => {
                  const ggmlProgress = downloadProgress[`ggml:${model.id}`];
                  const isSelected = settings.transcription.model_size === model.id;
                  return (
                    <div
                      key={model.id}
                      className={`
                        flex items-center justify-between p-3 rounded-xl border transition-all duration-200
                        ${isSelected
                          ? 'border-amber-500 dark:border-amber-400 bg-amber-50 dark:bg-amber-900/20'
                          : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50'
                        }
                      `}
                    >
                      <label className="flex items-center gap-3 cursor-pointer flex-1">
                        <input
                          type="radio"
                          name="model"
                          checked={isSelected}
                          onChange={() => handleChange('transcription', 'model_size', model.id)}
                          disabled={!model.downloaded}
                          className="w-4 h-4 text-amber-500 focus:ring-amber-500/20 border-stone-300 dark:border-stone-600"
                        />
                        <div>
                          <span className={`text-sm font-medium ${model.downloaded ? 'text-stone-900 dark:text-stone-100' : 'text-stone-400 dark:text-stone-500'}`}>
                            {model.name}
                          </span>
                          <span className="text-xs text-stone-500 dark:text-stone-400 ml-2">
                            ({formatSize(model.size_mb)})
                          </span>
                          {model.id.includes('turbo') && (
                            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 ml-2">
                              Fast
                            </span>
                          )}
                          {model.id.includes('q5') && (
                            <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-400 ml-2">
                              Quantized
                            </span>
                          )}
                        </div>
                      </label>

                      <div className="flex items-center gap-2">
                        {model.downloaded ? (
                          <>
                            <span className="flex items-center gap-1 text-xs font-medium text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30 px-2 py-1 rounded-lg">
                              <CheckIcon />
                              Ready
                            </span>
                            <button
                              onClick={() => handleDeleteModel(model.id)}
                              disabled={deleting === `ggml:${model.id}`}
                              className="p-1.5 rounded-lg text-stone-400 dark:text-stone-500 hover:text-red-500 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                              title={`Delete ${model.name}`}
                            >
                              <TrashIcon />
                            </button>
                          </>
                        ) : downloading === model.id ? (
                          <span className="flex items-center gap-2 text-xs font-medium text-amber-600 dark:text-amber-400">
                            <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                            </svg>
                            {ggmlProgress != null ? `${ggmlProgress}%` : 'Downloading...'}
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
                    </div>
                  );
                })}
              </div>
            </div>

            {/* CoreML Acceleration */}
            {coremlStatus?.supported && (
              <div className="pt-4 border-t border-stone-100 dark:border-stone-700">
                <div className="flex items-center justify-between mb-2">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-stone-700 dark:text-stone-300">
                        CoreML Acceleration
                      </span>
                      {coremlStatus.apple_silicon && (
                        <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400">
                          Apple Silicon
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                      Use Apple Neural Engine for faster transcription
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => handleChange('transcription', 'use_coreml', !(settings.transcription.use_coreml ?? true))}
                    className={`
                      relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200
                      ${(settings.transcription.use_coreml ?? true) ? 'bg-amber-500 dark:bg-amber-400' : 'bg-stone-300 dark:bg-stone-600'}
                    `}
                  >
                    <span
                      className={`
                        inline-block h-4 w-4 transform rounded-full bg-white shadow-sm transition-transform duration-200
                        ${(settings.transcription.use_coreml ?? true) ? 'translate-x-6' : 'translate-x-1'}
                      `}
                    />
                  </button>
                </div>

                {(settings.transcription.use_coreml ?? true) && (
                  <div className="space-y-2">
                    <p className="text-xs text-stone-400 dark:text-stone-500 mb-2">
                      CoreML encoders accelerate your selected speech model via Apple Neural Engine. Each encoder requires its corresponding base model above.
                    </p>
                    {models.filter(m => m.downloaded && m.coreml_size_mb > 0).map(model => {
                      const isActiveModel = settings.transcription.model_size === model.id;
                      const coremlProgress = downloadProgress[`coreml:${model.id}`];
                      return (
                        <div
                          key={model.id}
                          className={`
                            flex items-center justify-between p-3 rounded-xl border transition-all duration-200
                            ${model.coreml_downloaded && isActiveModel
                              ? 'border-green-400 dark:border-green-600 bg-green-50 dark:bg-green-900/20'
                              : model.coreml_downloaded
                                ? 'border-stone-200 dark:border-stone-700 bg-stone-50 dark:bg-stone-800/30'
                                : 'border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-800/50'
                            }
                          `}
                        >
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium text-stone-900 dark:text-stone-100">
                                {model.id}
                              </span>
                              <span className="text-xs text-stone-400 dark:text-stone-500">
                                {formatSize(model.coreml_size_mb)}
                              </span>
                              {isActiveModel && (
                                <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400">
                                  Selected
                                </span>
                              )}
                            </div>
                            <p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                              {model.coreml_downloaded
                                ? isActiveModel
                                  ? 'Neural Engine acceleration active'
                                  : 'Downloaded, will be used when this model is selected'
                                : 'Download to enable hardware acceleration'
                              }
                            </p>
                          </div>
                          <div className="flex items-center gap-2 ml-3">
                            {model.coreml_downloaded ? (
                              <>
                                <span className={`flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-lg ${
                                  isActiveModel
                                    ? 'text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900/30'
                                    : 'text-stone-500 dark:text-stone-400 bg-stone-100 dark:bg-stone-700/50'
                                }`}>
                                  <CheckIcon />
                                  {isActiveModel ? 'In Use' : 'Ready'}
                                </span>
                                <button
                                  onClick={() => handleDeleteCoremlModel(model.id)}
                                  disabled={deleting === `coreml:${model.id}`}
                                  className="p-1.5 rounded-lg text-stone-400 dark:text-stone-500 hover:text-red-500 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                                  title={`Delete CoreML encoder (${model.id})`}
                                >
                                  <TrashIcon />
                                </button>
                              </>
                            ) : downloadingCoreml === model.id ? (
                              <span className="flex items-center gap-2 text-xs font-medium text-amber-600 dark:text-amber-400">
                                <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                                </svg>
                                {coremlProgress != null
                                  ? coremlProgress >= 99
                                    ? 'Extracting...'
                                    : `${coremlProgress}%`
                                  : 'Downloading...'
                                }
                              </span>
                            ) : (
                              <button
                                onClick={() => downloadCoremlModel(model.id)}
                                className="flex items-center gap-1.5 text-xs font-medium text-amber-600 dark:text-amber-400 hover:text-amber-700 dark:hover:text-amber-300 bg-amber-100 dark:bg-amber-900/30 hover:bg-amber-200 dark:hover:bg-amber-900/50 px-3 py-1.5 rounded-lg transition-colors"
                              >
                                <DownloadIcon />
                                Download
                              </button>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            )}
          </SettingsSection>

          {/* Hotkey */}
          <SettingsSection
            icon={<KeyboardIcon />}
            title="Hotkey"
            description="Configure your activation shortcut"
          >
            <HotkeyRecorder
              value={settings.hotkey.key || 'F6'}
              onChange={(value) => handleChange('hotkey', 'key', value)}
            />

            <div className="pt-4 border-t border-stone-100 dark:border-stone-800">
              <CardSelect
                label="Activation Mode"
                value={settings.hotkey.mode || 'hold'}
                onChange={(value) => handleChange('hotkey', 'mode', value)}
                options={[
                  { value: 'hold', label: 'Hold to Talk', icon: <HoldIcon />, description: 'Press and hold key while speaking' },
                  { value: 'toggle', label: 'Toggle On/Off', icon: <ToggleIcon />, description: 'Press once to start, again to stop' },
                ]}
              />
            </div>
          </SettingsSection>

          {/* Output */}
          <SettingsSection
            icon={<OutputIcon />}
            title="Output"
            description="How text is inserted"
          >
            <CardSelect
              label="Insert Method"
              value={settings.output.insert_method || 'paste'}
              onChange={(value) => handleChange('output', 'insert_method', value)}
              options={[
                { value: 'paste', label: 'Paste', icon: <ClipboardIcon />, description: 'Copy to clipboard and paste (recommended)' },
                { value: 'type', label: 'Type', icon: <TypewriterIcon />, description: 'Simulate individual keystrokes' },
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
                <Dropdown
                  label="Provider"
                  value={settings.cleanup.provider || 'openai'}
                  onChange={(value) => handleChange('cleanup', 'provider', value)}
                  options={[
                    { value: 'openai', label: 'OpenAI', description: 'GPT-4o, GPT-4o-mini' },
                    { value: 'anthropic', label: 'Anthropic', description: 'Claude Sonnet, Haiku' },
                    { value: 'openrouter', label: 'OpenRouter', description: 'Multiple providers' },
                    { value: 'ollama', label: 'Ollama', description: 'Local models (free)' },
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
