import { useEffect, useState } from 'react';
import { useDictionaryStore } from '../../lib/dictionaryStore';
import type { DictionaryEntry } from '../../types';

interface EditModalProps {
  entry?: DictionaryEntry;
  onSave: (phrase: string, replacement: string) => void;
  onCancel: () => void;
}

function EditModal({ entry, onSave, onCancel }: EditModalProps) {
  const [phrase, setPhrase] = useState(entry?.phrase ?? '');
  const [replacement, setReplacement] = useState(entry?.replacement ?? '');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (phrase.trim() && replacement.trim()) {
      onSave(phrase.trim(), replacement.trim());
    }
  };

  return (
    <div className="fixed inset-0 bg-black/20 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-white rounded-2xl p-6 w-full max-w-md shadow-xl border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-6">
          {entry ? 'Edit entry' : 'Add entry'}
        </h3>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              Phrase
            </label>
            <input
              type="text"
              value={phrase}
              onChange={(e) => setPhrase(e.target.value)}
              placeholder="What you say (e.g., mentaflux)"
              className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-lg text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-900/10 focus:border-gray-300 transition-colors"
              autoFocus
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1.5">
              Replacement
            </label>
            <input
              type="text"
              value={replacement}
              onChange={(e) => setReplacement(e.target.value)}
              placeholder="Corrected spelling (e.g., MentaFlux)"
              className="w-full px-3 py-2 bg-gray-50 border border-gray-200 rounded-lg text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-900/10 focus:border-gray-300 transition-colors"
            />
          </div>
          <div className="flex gap-3 justify-end pt-4">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!phrase.trim() || !replacement.trim()}
              className="px-4 py-2 text-sm font-medium bg-gray-900 hover:bg-gray-800 disabled:bg-gray-300 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
            >
              {entry ? 'Save' : 'Add'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export function DictionaryPage() {
  const { entries, isLoading, loadDictionary, addEntry, updateEntry, removeEntry, toggleEntry } = useDictionaryStore();
  const [showModal, setShowModal] = useState(false);
  const [editingEntry, setEditingEntry] = useState<DictionaryEntry | undefined>();
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  useEffect(() => {
    loadDictionary();
  }, [loadDictionary]);

  const handleAdd = () => {
    setEditingEntry(undefined);
    setShowModal(true);
  };

  const handleEdit = (entry: DictionaryEntry) => {
    setEditingEntry(entry);
    setShowModal(true);
  };

  const handleSave = async (phrase: string, replacement: string) => {
    try {
      if (editingEntry) {
        await updateEntry(editingEntry.id, phrase, replacement, editingEntry.enabled);
      } else {
        await addEntry(phrase, replacement);
      }
      setShowModal(false);
      setEditingEntry(undefined);
    } catch (err) {
      console.error('Failed to save entry:', err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await removeEntry(id);
    } catch (err) {
      console.error('Failed to delete entry:', err);
    }
  };

  const handleToggle = async (id: string) => {
    try {
      await toggleEntry(id);
    } catch (err) {
      console.error('Failed to toggle entry:', err);
    }
  };

  return (
    <div className="h-full overflow-y-auto bg-white">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-semibold text-gray-900">Dictionary</h1>
            <p className="text-sm text-gray-500 mt-1">
              Custom vocabulary for better accuracy
            </p>
          </div>
          <button
            onClick={handleAdd}
            className="flex items-center gap-2 px-4 py-2 bg-gray-900 hover:bg-gray-800 text-white text-sm font-medium rounded-lg transition-colors"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            Add entry
          </button>
        </div>

        {/* Modal */}
        {showModal && (
          <EditModal
            entry={editingEntry}
            onSave={handleSave}
            onCancel={() => {
              setShowModal(false);
              setEditingEntry(undefined);
            }}
          />
        )}

        {/* Info box */}
        <div className="bg-gray-50 rounded-xl px-6 py-4 mb-6">
          <p className="text-sm text-gray-500">
            Add words that are frequently misrecognized. When transcribed text contains a matching phrase,
            it will be automatically replaced with your correction.
          </p>
        </div>

        {/* Dictionary list */}
        {isLoading ? (
          <div className="text-center py-12 text-gray-400">Loading...</div>
        ) : entries.length === 0 ? (
          <div className="text-center py-16">
            <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-1">No dictionary entries</h3>
            <p className="text-gray-500 text-sm mb-4">
              Add custom phrases to improve transcription accuracy
            </p>
            <button
              onClick={handleAdd}
              className="px-4 py-2 bg-gray-900 hover:bg-gray-800 text-white text-sm font-medium rounded-lg transition-colors"
            >
              Add your first entry
            </button>
          </div>
        ) : (
          <div className="bg-gray-50 rounded-xl overflow-hidden">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-100">
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">
                    Phrase
                  </th>
                  <th className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3">
                    Replacement
                  </th>
                  <th className="text-center text-xs font-medium text-gray-500 uppercase tracking-wider px-4 py-3 w-20">
                    Active
                  </th>
                  <th className="w-20"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-100">
                {entries.map((entry) => (
                  <tr
                    key={entry.id}
                    className="hover:bg-gray-100/50 transition-colors"
                    onMouseEnter={() => setHoveredId(entry.id)}
                    onMouseLeave={() => setHoveredId(null)}
                  >
                    <td className="px-4 py-3 text-sm text-gray-700">{entry.phrase}</td>
                    <td className="px-4 py-3 text-sm text-gray-700">{entry.replacement}</td>
                    <td className="px-4 py-3 text-center">
                      <button
                        onClick={() => handleToggle(entry.id)}
                        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                          entry.enabled ? 'bg-gray-900' : 'bg-gray-300'
                        }`}
                      >
                        <span
                          className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow transition-transform ${
                            entry.enabled ? 'translate-x-4' : 'translate-x-1'
                          }`}
                        />
                      </button>
                    </td>
                    <td className="px-4 py-3">
                      <div
                        className={`flex items-center gap-1 justify-end transition-opacity ${
                          hoveredId === entry.id ? 'opacity-100' : 'opacity-0'
                        }`}
                      >
                        <button
                          onClick={() => handleEdit(entry)}
                          className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-white rounded-lg transition-colors"
                          title="Edit"
                        >
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
                          </svg>
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
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
