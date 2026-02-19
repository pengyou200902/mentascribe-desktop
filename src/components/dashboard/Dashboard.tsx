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
      <main className="flex-1 overflow-hidden bg-white dark:bg-stone-900 transition-colors duration-200">
        {renderPage()}
      </main>
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
