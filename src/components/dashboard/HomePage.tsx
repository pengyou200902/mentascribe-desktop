import { useEffect } from 'react';
import { useStatsStore } from '../../lib/statsStore';
import { useHistoryStore } from '../../lib/historyStore';

// Stat icons with modern design
const FlameIcon = () => (
  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24">
    <path
      className="fill-amber-500/20 dark:fill-amber-400/20"
      d="M12 23c-4.97 0-9-4.03-9-9 0-2.39.94-4.68 2.64-6.36l.71-.71.71.71c1.95 1.95 3.03 4.55 3.03 7.32 0 .55.45 1 1 1s1-.45 1-1c0-3.44-1.47-6.74-4.02-9.03l-.95-.86 1.36-.25c.72-.13 1.45-.2 2.16-.2 6.08 0 11.02 4.94 11.02 11.02 0 4.06-2.24 7.78-5.82 9.69l.18-.15c.95-.78 1.64-1.81 2.01-2.96.52-1.61.34-3.35-.52-4.8l-.41-.69-.68.42c-.86.53-1.86.81-2.88.81-2.89 0-5.24-2.35-5.24-5.24 0-1.27.46-2.5 1.29-3.45l.41-.47.47.41c2.18 1.91 3.43 4.67 3.43 7.56 0 1.37-.27 2.7-.8 3.93l-.24.55.55-.24c1.51-.66 2.79-1.71 3.71-3.04l.36-.52.52.36c.55.38 1.01.87 1.36 1.44.85 1.38.98 3.05.37 4.56-.39.96-.98 1.82-1.74 2.53l-.58.54.79-.14c.35-.06.7-.14 1.04-.23C20.23 19.82 21 17.5 21 15c0-5.52-4.48-10-10-10S1 9.48 1 15c0 5.52 4.48 10 10 10h1v-2h-1z"
    />
    <path
      className="stroke-amber-500 dark:stroke-amber-400"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.21 0 003 2.48z"
    />
    <path
      className="stroke-amber-500 dark:stroke-amber-400"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M12 18a3.75 3.75 0 00.495-7.467 5.99 5.99 0 00-1.925 3.546 5.974 5.974 0 01-2.133-1A3.75 3.75 0 0012 18z"
    />
  </svg>
);

const PencilIcon = () => (
  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24">
    <path
      className="fill-stone-400/20 dark:fill-stone-500/20"
      d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25z"
    />
    <path
      className="stroke-stone-500 dark:stroke-stone-400"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L6.832 19.82a4.5 4.5 0 01-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 011.13-1.897L16.863 4.487zm0 0L19.5 7.125"
    />
  </svg>
);

const ClockIcon = () => (
  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24">
    <circle
      className="fill-stone-400/10 dark:fill-stone-500/10"
      cx="12"
      cy="12"
      r="9"
    />
    <path
      className="stroke-stone-500 dark:stroke-stone-400"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"
    />
  </svg>
);

const MicIcon = () => (
  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24">
    <path
      className="fill-stone-200 dark:fill-stone-700"
      d="M12 15.75a3.75 3.75 0 003.75-3.75V6a3.75 3.75 0 00-7.5 0v6a3.75 3.75 0 003.75 3.75z"
    />
    <path
      className="stroke-stone-400 dark:stroke-stone-500"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M12 15.75a3.75 3.75 0 003.75-3.75V6a3.75 3.75 0 00-7.5 0v6a3.75 3.75 0 003.75 3.75zM18.75 10.5v1.5a6.75 6.75 0 01-13.5 0v-1.5M12 18.75v3M9 21.75h6"
    />
  </svg>
);

// Decorative waveform component
const WaveformDecoration = () => (
  <div className="flex items-center gap-1 opacity-30">
    {[8, 16, 12, 20, 10, 14, 8, 18, 12].map((h, i) => (
      <div
        key={i}
        className="w-1 rounded-full bg-amber-500 dark:bg-amber-400"
        style={{
          height: `${h}px`,
          animation: `waveform-deco 1.2s ease-in-out infinite`,
          animationDelay: `${i * 0.08}s`
        }}
      />
    ))}
  </div>
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
        dateKey = 'Today';
      } else if (date.toDateString() === yesterday.toDateString()) {
        dateKey = 'Yesterday';
      } else {
        dateKey = date.toLocaleDateString('en-US', {
          weekday: 'long',
          month: 'short',
          day: 'numeric',
        });
      }

      if (!groups[dateKey]) {
        groups[dateKey] = [];
      }
      groups[dateKey].push(entry);
    });

    return groups;
  };

  const groupedEntries = groupEntriesByDate();
  const statItems = [
    {
      icon: <FlameIcon />,
      value: stats?.streak_days ?? 0,
      label: 'Day Streak',
      accent: true,
    },
    {
      icon: <PencilIcon />,
      value: formatNumber(stats?.total_words ?? 0),
      label: 'Total Words',
    },
    {
      icon: <ClockIcon />,
      value: calculateWPM(),
      label: 'Avg WPM',
    },
  ];

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header with decorative waveform */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-2xl font-semibold text-stone-900 dark:text-stone-100 tracking-tight">
              Welcome back
            </h1>
            <p className="text-sm text-stone-500 dark:text-stone-400 mt-0.5">
              Your voice, perfectly captured
            </p>
          </div>
          <WaveformDecoration />
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-3 gap-4 mb-8">
          {statItems.map((stat, index) => (
            <div
              key={stat.label}
              className={`
                relative overflow-hidden rounded-2xl p-5 transition-all duration-300
                ${stat.accent
                  ? 'bg-gradient-to-br from-amber-50 to-amber-100/50 dark:from-amber-900/20 dark:to-amber-800/10 border border-amber-200/50 dark:border-amber-700/30'
                  : 'bg-stone-50 dark:bg-stone-800/50 border border-stone-100 dark:border-stone-700/50'
                }
                hover:shadow-card dark:hover:shadow-card-dark
                animate-fade-in
              `}
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <div className={`
                    p-2 rounded-xl
                    ${stat.accent
                      ? 'bg-amber-500/10 dark:bg-amber-400/10'
                      : 'bg-stone-100 dark:bg-stone-700/50'
                    }
                  `}>
                    {stat.icon}
                  </div>
                  <div>
                    <div className={`
                      text-2xl font-semibold tracking-tight
                      ${stat.accent
                        ? 'text-amber-700 dark:text-amber-400'
                        : 'text-stone-900 dark:text-stone-100'
                      }
                    `}>
                      {stat.value}
                    </div>
                    <div className="text-xs font-medium text-stone-500 dark:text-stone-400 mt-0.5">
                      {stat.label}
                    </div>
                  </div>
                </div>
              </div>
              {/* Decorative gradient overlay */}
              {stat.accent && (
                <div className="absolute top-0 right-0 w-20 h-20 bg-gradient-radial from-amber-400/10 to-transparent rounded-full -translate-y-1/2 translate-x-1/2" />
              )}
            </div>
          ))}
        </div>

        {/* Quick tip banner */}
        <div className="relative overflow-hidden rounded-2xl px-5 py-4 mb-8 bg-stone-50 dark:bg-stone-800/30 border border-stone-100 dark:border-stone-700/50">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-stone-100 dark:bg-stone-700/50">
              <svg className="w-5 h-5 text-stone-400 dark:text-stone-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 18v-5.25m0 0a6.01 6.01 0 001.5-.189m-1.5.189a6.01 6.01 0 01-1.5-.189m3.75 7.478a12.06 12.06 0 01-4.5 0m3.75 2.383a14.406 14.406 0 01-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 10-7.517 0c.85.493 1.509 1.333 1.509 2.316V18" />
              </svg>
            </div>
            <p className="text-sm text-stone-600 dark:text-stone-300">
              Press your hotkey to start dictating. Your transcriptions will appear here.
            </p>
          </div>
        </div>

        {/* Recent Activity Section */}
        <div className="mb-6">
          <h2 className="text-sm font-semibold text-stone-900 dark:text-stone-100 mb-4 flex items-center gap-2">
            <span>Recent Activity</span>
            <div className="flex-1 h-px bg-stone-100 dark:bg-stone-800" />
          </h2>
        </div>

        {/* Timeline History */}
        {isLoading && entries.length === 0 ? (
          <div className="flex items-center justify-center py-16">
            <div className="flex items-center gap-3 text-stone-400 dark:text-stone-500">
              <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              <span className="text-sm">Loading...</span>
            </div>
          </div>
        ) : entries.length === 0 ? (
          <div className="text-center py-16">
            <div className="w-20 h-20 bg-stone-100 dark:bg-stone-800 rounded-2xl flex items-center justify-center mx-auto mb-5">
              <MicIcon />
            </div>
            <h3 className="text-lg font-medium text-stone-900 dark:text-stone-100 mb-2">
              No transcriptions yet
            </h3>
            <p className="text-sm text-stone-500 dark:text-stone-400 max-w-sm mx-auto">
              Start dictating to see your transcription history here
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            {Object.entries(groupedEntries).map(([date, dateEntries], groupIndex) => (
              <div
                key={date}
                className="animate-fade-in"
                style={{ animationDelay: `${groupIndex * 0.1}s` }}
              >
                <div className="flex items-center gap-3 mb-3">
                  <span className="text-xs font-semibold text-stone-400 dark:text-stone-500 uppercase tracking-wider">
                    {date}
                  </span>
                  <div className="flex-1 h-px bg-stone-100 dark:bg-stone-800" />
                </div>
                <div className="rounded-2xl overflow-hidden border border-stone-100 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-800/30">
                  {dateEntries.map((entry, index) => (
                    <div
                      key={entry.id}
                      className={`
                        flex gap-4 px-4 py-3.5 transition-colors duration-150
                        hover:bg-stone-100/50 dark:hover:bg-stone-700/30
                        ${index !== dateEntries.length - 1 ? 'border-b border-stone-100 dark:border-stone-800' : ''}
                      `}
                    >
                      <div className="w-20 flex-shrink-0 pt-0.5">
                        <span className="text-sm font-medium text-stone-400 dark:text-stone-500 tabular-nums">
                          {formatTime(entry.timestamp)}
                        </span>
                      </div>
                      <div className="flex-1 min-w-0">
                        {isSilentAudio(entry.text) ? (
                          <span className="text-sm text-stone-400 dark:text-stone-500 italic">
                            No speech detected
                          </span>
                        ) : (
                          <p className="text-sm text-stone-700 dark:text-stone-300 whitespace-pre-wrap leading-relaxed">
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
