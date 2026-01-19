import { useEffect, useState } from 'react';
import { useHistoryStore } from '../../lib/historyStore';

// Icons
const ClockIcon = () => (
  <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24">
    <circle
      className="fill-stone-100 dark:fill-stone-800"
      cx="12"
      cy="12"
      r="9"
    />
    <path
      className="stroke-stone-400 dark:stroke-stone-500"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"
    />
  </svg>
);

const CopyIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
  </svg>
);

const CheckIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
  </svg>
);

const TrashIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
  </svg>
);

export function HistoryPage() {
  const { entries, totalCount, isLoading, hasMore, loadHistory, loadMore, deleteEntry, clearAll } = useHistoryStore();
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

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

    entries.forEach((entry) => {
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

  const handleCopy = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteEntry(id);
    } catch (err) {
      console.error('Failed to delete:', err);
    }
  };

  const handleClearAll = async () => {
    try {
      await clearAll();
      setShowClearConfirm(false);
    } catch (err) {
      console.error('Failed to clear:', err);
    }
  };

  const groupedEntries = groupEntriesByDate();

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-2xl font-semibold text-stone-900 dark:text-stone-100 tracking-tight">
              History
            </h1>
            <p className="text-sm text-stone-500 dark:text-stone-400 mt-0.5">
              {totalCount} transcription{totalCount !== 1 ? 's' : ''}
            </p>
          </div>
          {entries.length > 0 && (
            <button
              onClick={() => setShowClearConfirm(true)}
              className="px-4 py-2 text-sm font-medium text-stone-500 dark:text-stone-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-xl transition-all duration-200"
            >
              Clear all
            </button>
          )}
        </div>

        {/* Clear confirmation modal */}
        {showClearConfirm && (
          <div className="fixed inset-0 bg-black/30 dark:bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
            <div className="bg-white dark:bg-stone-800 rounded-2xl p-6 max-w-sm shadow-xl border border-stone-200 dark:border-stone-700 animate-scale-in">
              <div className="w-12 h-12 bg-red-100 dark:bg-red-900/30 rounded-xl flex items-center justify-center mx-auto mb-4">
                <svg className="w-6 h-6 text-red-500 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-stone-900 dark:text-stone-100 text-center mb-2">
                Clear all history?
              </h3>
              <p className="text-sm text-stone-500 dark:text-stone-400 text-center mb-6">
                This will permanently delete all {totalCount} transcriptions. This action cannot be undone.
              </p>
              <div className="flex gap-3">
                <button
                  onClick={() => setShowClearConfirm(false)}
                  className="flex-1 px-4 py-2.5 text-sm font-medium text-stone-700 dark:text-stone-300 bg-stone-100 dark:bg-stone-700 hover:bg-stone-200 dark:hover:bg-stone-600 rounded-xl transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleClearAll}
                  className="flex-1 px-4 py-2.5 text-sm font-medium bg-red-500 hover:bg-red-600 text-white rounded-xl transition-colors"
                >
                  Clear all
                </button>
              </div>
            </div>
          </div>
        )}

        {/* History timeline */}
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
              <ClockIcon />
            </div>
            <h3 className="text-lg font-medium text-stone-900 dark:text-stone-100 mb-2">
              No history yet
            </h3>
            <p className="text-sm text-stone-500 dark:text-stone-400 max-w-sm mx-auto">
              Your transcription history will appear here
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            {Object.entries(groupedEntries).map(([date, dateEntries], groupIndex) => (
              <div
                key={date}
                className="animate-fade-in"
                style={{ animationDelay: `${groupIndex * 0.05}s` }}
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
                        flex gap-4 px-4 py-3.5 transition-colors duration-150 relative group
                        hover:bg-stone-100/50 dark:hover:bg-stone-700/30
                        ${index !== dateEntries.length - 1 ? 'border-b border-stone-100 dark:border-stone-800' : ''}
                      `}
                      onMouseEnter={() => setHoveredId(entry.id)}
                      onMouseLeave={() => setHoveredId(null)}
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
                          <p className="text-sm text-stone-700 dark:text-stone-300 whitespace-pre-wrap pr-20 leading-relaxed">
                            {entry.text}
                          </p>
                        )}
                      </div>

                      {/* Action buttons */}
                      {!isSilentAudio(entry.text) && (
                        <div
                          className={`
                            absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-1
                            transition-all duration-200
                            ${hoveredId === entry.id ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-2'}
                          `}
                        >
                          <button
                            onClick={() => handleCopy(entry.text, entry.id)}
                            className={`
                              p-2 rounded-lg transition-all duration-200
                              ${copiedId === entry.id
                                ? 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400'
                                : 'bg-white dark:bg-stone-700 text-stone-400 dark:text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 shadow-sm'
                              }
                            `}
                            title="Copy"
                          >
                            {copiedId === entry.id ? <CheckIcon /> : <CopyIcon />}
                          </button>
                          <button
                            onClick={() => handleDelete(entry.id)}
                            className="p-2 bg-white dark:bg-stone-700 rounded-lg text-stone-400 dark:text-stone-400 hover:text-red-500 dark:hover:text-red-400 shadow-sm transition-all duration-200"
                            title="Delete"
                          >
                            <TrashIcon />
                          </button>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}

            {/* Load more button */}
            {hasMore && (
              <div className="text-center pt-4">
                <button
                  onClick={loadMore}
                  disabled={isLoading}
                  className="px-6 py-2.5 text-sm font-medium text-stone-600 dark:text-stone-300 bg-stone-100 dark:bg-stone-800 hover:bg-stone-200 dark:hover:bg-stone-700 rounded-xl transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isLoading ? (
                    <span className="flex items-center gap-2">
                      <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                      </svg>
                      Loading...
                    </span>
                  ) : 'Load more'}
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
