import { FC, useState, useEffect } from 'react';

interface HistoryProps {
  onBack: () => void;
}

interface TranscriptionEntry {
  id: string;
  text: string;
  timestamp: Date;
}

export const History: FC<HistoryProps> = ({ onBack }) => {
  const [history, setHistory] = useState<TranscriptionEntry[]>([]);

  useEffect(() => {
    // Load history from local storage for now
    // In production, this would sync with the API
    const stored = localStorage.getItem('transcription-history');
    if (stored) {
      const parsed = JSON.parse(stored);
      setHistory(
        parsed.map((entry: any) => ({
          ...entry,
          timestamp: new Date(entry.timestamp),
        }))
      );
    }
  }, []);

  function formatDate(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) {
      return 'Just now';
    } else if (diff < 3600000) {
      const minutes = Math.floor(diff / 60000);
      return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    } else if (diff < 86400000) {
      const hours = Math.floor(diff / 3600000);
      return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  function clearHistory() {
    localStorage.removeItem('transcription-history');
    setHistory([]);
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={onBack}
            className="text-gray-400 hover:text-white"
          >
            &larr; Back
          </button>
          <h2 className="text-xl font-semibold">History</h2>
        </div>

        {history.length > 0 && (
          <button
            onClick={clearHistory}
            className="text-red-400 hover:text-red-300 text-sm"
          >
            Clear all
          </button>
        )}
      </div>

      {history.length === 0 ? (
        <div className="text-center py-12 text-gray-400">
          <p>No transcriptions yet.</p>
          <p className="text-sm mt-2">
            Press F6 to start dictating.
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {history.map((entry) => (
            <div
              key={entry.id}
              className="bg-gray-800 rounded-lg p-4"
            >
              <p className="text-white mb-2">{entry.text}</p>
              <p className="text-sm text-gray-400">
                {formatDate(entry.timestamp)}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
