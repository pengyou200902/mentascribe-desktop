import { useState } from 'react';
import { ThemeProvider } from '../../lib/theme';
import { Sidebar } from './Sidebar';
import { HomePage } from './HomePage';
import { HistoryPage } from './HistoryPage';
import { DictionaryPage } from './DictionaryPage';
import { SettingsPage } from './SettingsPage';
import type { DashboardPage } from '../../types';

function DashboardContent() {
  const [currentPage, setCurrentPage] = useState<DashboardPage>('home');

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
