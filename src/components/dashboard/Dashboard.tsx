import { useState } from 'react';
import { Sidebar } from './Sidebar';
import { HomePage } from './HomePage';
import { HistoryPage } from './HistoryPage';
import { DictionaryPage } from './DictionaryPage';
import { SettingsPage } from './SettingsPage';
import type { DashboardPage } from '../../types';

export function Dashboard() {
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
    <div className="dashboard-layout flex h-screen bg-white">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-hidden">
        {renderPage()}
      </main>
    </div>
  );
}
