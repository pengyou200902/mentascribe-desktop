import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { ThemeProvider } from '../../lib/theme';
import { Sidebar } from './Sidebar';
import { HomePage } from './HomePage';
import { HistoryPage } from './HistoryPage';
import { DictionaryPage } from './DictionaryPage';
import { SettingsPage } from './SettingsPage';
import type { DashboardPage } from '../../types';

function getInitialPage(): DashboardPage {
  // Support URLs like index.html#dashboard/settings
  const hash = window.location.hash.slice(1); // remove #
  const parts = hash.split('/');
  if (parts.length > 1) {
    const page = parts[1] as DashboardPage;
    if (['home', 'history', 'dictionary', 'settings'].includes(page)) {
      return page;
    }
  }
  return 'home';
}

function DashboardContent() {
  const [currentPage, setCurrentPage] = useState<DashboardPage>(getInitialPage);
  const [isPreloading, setIsPreloading] = useState(false);
  const [preloadModel, setPreloadModel] = useState<string | null>(null);

  // Listen for navigation events from tray menu
  useEffect(() => {
    const unlisten = listen<string>('navigate-to-page', (event) => {
      const page = event.payload as DashboardPage;
      if (['home', 'history', 'dictionary', 'settings'].includes(page)) {
        setCurrentPage(page);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Listen for model preloading status
  useEffect(() => {
    const unlistenStart = listen<string>('model-preload-start', (event) => {
      setIsPreloading(true);
      setPreloadModel(event.payload);
    });

    const unlistenComplete = listen('model-preload-complete', () => {
      setIsPreloading(false);
      setPreloadModel(null);
    });

    const unlistenError = listen('model-preload-error', () => {
      setIsPreloading(false);
      setPreloadModel(null);
    });

    return () => {
      unlistenStart.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  const renderPage = () => {
    switch (currentPage) {
      case 'home':
        return <HomePage />;
      case 'history':
        return <HistoryPage />;
      case 'dictionary':
        return <DictionaryPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <HomePage />;
    }
  };

  return (
    <div className="dashboard-layout flex h-screen bg-stone-50 dark:bg-stone-950 transition-colors duration-200">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-stone-900 transition-colors duration-200">
        {/* Model preloading banner */}
        {isPreloading && (
          <div className="flex items-center gap-2 px-4 py-2 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200/60 dark:border-amber-800/40 text-amber-700 dark:text-amber-400 text-xs font-medium transition-all duration-300">
            <svg className="w-3.5 h-3.5 animate-spin flex-shrink-0" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            <span>
              Warming up speech model{preloadModel ? ` (${preloadModel})` : ''}...
            </span>
          </div>
        )}
        <main className="flex-1 overflow-hidden">
          {renderPage()}
        </main>
      </div>
    </div>
  );
}

export function Dashboard() {
  return (
    <ThemeProvider>
      <DashboardContent />
    </ThemeProvider>
  );
}
