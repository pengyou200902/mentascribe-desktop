import { useEffect } from 'react';
import { useStatsStore } from '../../lib/statsStore';
import { useHistoryStore } from '../../lib/historyStore';

// Flame icon for streak
const FlameIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.21 0 003 2.48z" />
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 18a3.75 3.75 0 00.495-7.467 5.99 5.99 0 00-1.925 3.546 5.974 5.974 0 01-2.133-1A3.75 3.75 0 0012 18z" />
  </svg>
);

// Pencil icon for words
const PencilIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L6.832 19.82a4.5 4.5 0 01-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 011.13-1.897L16.863 4.487zm0 0L19.5 7.125" />
  </svg>
);

// Clock icon for WPM
const ClockIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

export function HomePage() {
  const { stats, loadStats } = useStatsStore();
  const { entries, loadHistory, isLoading } = useHistoryStore();

  useEffect(() => {
    loadStats();
    loadHistory();
  }, [loadStats, loadHistory]);

  const formatNumber = (n: number): string => {
    if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  };

  const calculateWPM = (): number => {
    if (!stats || stats.total_audio_seconds === 0) return 0;
    const minutes = stats.total_audio_seconds / 60;
    return Math.round(stats.total_words / minutes);
  };

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: true,
    });
  };

  // Check if an entry represents silent audio (empty or very short)
  const isSilentAudio = (text: string): boolean => {
    const trimmed = text.trim();
    return trimmed === '' || trimmed.length < 2;
  };

  const groupEntriesByDate = () => {
    const groups: { [key: string]: typeof entries } = {};

    entries.slice(0, 20).forEach((entry) => {
      const date = new Date(entry.timestamp);
      const today = new Date();
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);

      let dateKey: string;
      if (date.toDateString() === today.toDateString()) {
        dateKey = 'TODAY';
      } else if (date.toDateString() === yesterday.toDateString()) {
        dateKey = 'YESTERDAY';
      } else {
        dateKey = date.toLocaleDateString('en-US', {
          weekday: 'long',
          month: 'short',
          day: 'numeric',
        }).toUpperCase();
      }

      if (!groups[dateKey]) {
        groups[dateKey] = [];
      }
      groups[dateKey].push(entry);
    });

    return groups;
  };

  const groupedEntries = groupEntriesByDate();

  return (
    <div className="h-full overflow-y-auto bg-white">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header with welcome and stats */}
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-2xl font-semibold text-gray-900">
            Home
          </h1>
          <div className="flex items-center gap-4 text-sm">
            <div className="flex items-center gap-1.5 text-gray-600">
              <FlameIcon />
              <span className="font-medium">{stats?.streak_days ?? 0} day{(stats?.streak_days ?? 0) !== 1 ? 's' : ''}</span>
            </div>
            <div className="w-px h-4 bg-gray-200" />
            <div className="flex items-center gap-1.5 text-gray-600">
              <PencilIcon />
              <span className="font-medium">{formatNumber(stats?.total_words ?? 0)} words</span>
            </div>
            <div className="w-px h-4 bg-gray-200" />
            <div className="flex items-center gap-1.5 text-gray-600">
              <ClockIcon />
              <span className="font-medium">{calculateWPM()} WPM</span>
            </div>
          </div>
        </div>

        {/* Quick tip banner */}
        <div className="bg-gray-50 rounded-lg px-5 py-4 mb-8 border border-gray-100">
          <p className="text-gray-500 text-sm">
            Press your hotkey to start dictating. Your transcriptions will appear here.
          </p>
        </div>

        {/* Timeline History */}
        {isLoading && entries.length === 0 ? (
          <div className="text-center py-12 text-gray-400">Loading...</div>
        ) : entries.length === 0 ? (
          <div className="text-center py-16">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-1">No transcriptions yet</h3>
            <p className="text-gray-500 text-sm">
              Start dictating to see your history here
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            {Object.entries(groupedEntries).map(([date, dateEntries]) => (
              <div key={date}>
                <div className="text-xs font-medium text-gray-400 uppercase tracking-wider mb-3">
                  {date}
                </div>
                <div className="bg-gray-50 rounded-lg overflow-hidden border border-gray-100">
                  {dateEntries.map((entry, index) => (
                    <div
                      key={entry.id}
                      className={`flex gap-4 px-4 py-3 hover:bg-gray-100/50 transition-colors ${
                        index !== dateEntries.length - 1 ? 'border-b border-gray-100' : ''
                      }`}
                    >
                      <div className="w-20 flex-shrink-0">
                        <span className="text-sm text-gray-400 font-medium">
                          {formatTime(entry.timestamp)}
                        </span>
                      </div>
                      <div className="flex-1 min-w-0">
                        {isSilentAudio(entry.text) ? (
                          <span className="text-sm text-gray-400 italic">No speech detected</span>
                        ) : (
                          <p className="text-sm text-gray-700 whitespace-pre-wrap leading-relaxed">
                            {entry.text}
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
