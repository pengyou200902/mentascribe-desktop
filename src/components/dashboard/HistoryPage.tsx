import { useEffect, useState } from 'react';
import { useHistoryStore } from '../../lib/historyStore';

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

  // Check if an entry represents silent audio (empty or very short)
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
    <div className="h-full overflow-y-auto bg-white">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-2xl font-semibold text-gray-900">History</h1>
            <p className="text-sm text-gray-500 mt-1">{totalCount} transcription{totalCount !== 1 ? 's' : ''}</p>
          </div>
          {entries.length > 0 && (
            <button
              onClick={() => setShowClearConfirm(true)}
              className="px-3 py-1.5 text-sm text-gray-500 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            >
              Clear all
            </button>
          )}
        </div>

        {/* Clear confirmation modal */}
        {showClearConfirm && (
          <div className="fixed inset-0 bg-black/20 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-white rounded-xl p-6 max-w-sm shadow-lg border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-2">Clear all history?</h3>
              <p className="text-sm text-gray-500 mb-6">
                This will permanently delete all {totalCount} transcriptions. This action cannot be undone.
              </p>
              <div className="flex gap-3 justify-end">
                <button
                  onClick={() => setShowClearConfirm(false)}
                  className="px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleClearAll}
                  className="px-4 py-2 text-sm font-medium bg-red-500 hover:bg-red-600 text-white rounded-lg transition-colors"
                >
                  Clear all
                </button>
              </div>
            </div>
          </div>
        )}

        {/* History timeline */}
        {isLoading && entries.length === 0 ? (
          <div className="text-center py-12 text-gray-400">Loading...</div>
        ) : entries.length === 0 ? (
          <div className="text-center py-16">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-1">No history yet</h3>
            <p className="text-gray-500 text-sm">
              Your transcription history will appear here
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
                      className={`flex gap-4 px-4 py-3 hover:bg-gray-100/50 transition-colors relative group ${
                        index !== dateEntries.length - 1 ? 'border-b border-gray-100' : ''
                      }`}
                      onMouseEnter={() => setHoveredId(entry.id)}
                      onMouseLeave={() => setHoveredId(null)}
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
                          <p className="text-sm text-gray-700 whitespace-pre-wrap pr-16 leading-relaxed">
                            {entry.text}
                          </p>
                        )}
                      </div>

                      {/* Action buttons */}
                      {!isSilentAudio(entry.text) && (
                        <div
                          className={`absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-1 transition-opacity ${
                            hoveredId === entry.id ? 'opacity-100' : 'opacity-0'
                          }`}
                        >
                          <button
                            onClick={() => handleCopy(entry.text, entry.id)}
                            className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-white rounded-lg transition-colors"
                            title="Copy"
                          >
                            {copiedId === entry.id ? (
                              <svg className="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                              </svg>
                            ) : (
                              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
                              </svg>
                            )}
                          </button>
                          <button
                            onClick={() => handleDelete(entry.id)}
                            className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-white rounded-lg transition-colors"
                            title="Delete"
                          >
                            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                              <path strokeLinecap="round" strokeLinejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                            </svg>
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
                  className="px-6 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors disabled:opacity-50"
                >
                  {isLoading ? 'Loading...' : 'Load more'}
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
