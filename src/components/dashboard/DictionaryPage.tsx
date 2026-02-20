import { useEffect, useState } from 'react';
import { useDictionaryStore } from '../../lib/dictionaryStore';
import type { DictionaryEntry } from '../../types';

// Icons
const BookIcon = () => (
  <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24">
    <path
      className="fill-stone-100 dark:fill-stone-800"
      d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25"
    />
    <path
      className="stroke-stone-400 dark:stroke-stone-500"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25"
    />
  </svg>
);

const PlusIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
  </svg>
);

const EditIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
  </svg>
);

const TrashIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
  </svg>
);

const ArrowRightIcon = () => (
  <svg className="w-3.5 h-3.5 text-stone-400 dark:text-stone-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
  </svg>
);

type EntryMode = 'word' | 'correct';

interface EditModalProps {
  entry?: DictionaryEntry;
  onSave: (phrase: string, replacement: string) => void;
  onCancel: () => void;
}

function EditModal({ entry, onSave, onCancel }: EditModalProps) {
  const isVocabularyEntry = entry ? entry.phrase === entry.replacement : true;
  const [mode, setMode] = useState<EntryMode>(isVocabularyEntry ? 'word' : 'correct');
  const [word, setWord] = useState(entry && isVocabularyEntry ? entry.phrase : '');
  const [phrase, setPhrase] = useState(entry && !isVocabularyEntry ? entry.phrase : '');
  const [replacement, setReplacement] = useState(entry && !isVocabularyEntry ? entry.replacement : '');

  const isValid = mode === 'word'
    ? word.trim().length > 0
    : phrase.trim().length > 0 && replacement.trim().length > 0;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;
    if (mode === 'word') {
      onSave(word.trim(), word.trim());
    } else {
      onSave(phrase.trim(), replacement.trim());
    }
  };

  const inputClass = "w-full px-4 py-2.5 bg-stone-50 dark:bg-stone-900 border border-stone-200 dark:border-stone-700 rounded-xl text-stone-900 dark:text-stone-100 placeholder-stone-400 dark:placeholder-stone-500 focus:outline-none focus:ring-2 focus:ring-amber-500/20 focus:border-amber-500 dark:focus:border-amber-400 transition-all duration-200";

  return (
    <div className="fixed inset-0 bg-black/30 dark:bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
      <div className="bg-white dark:bg-stone-800 rounded-2xl p-6 w-full max-w-md shadow-xl border border-stone-200 dark:border-stone-700 animate-scale-in">
        <h3 className="text-lg font-semibold text-stone-900 dark:text-stone-100 mb-5">
          {entry ? 'Edit entry' : 'Add entry'}
        </h3>

        {/* Segmented control */}
        <div className="flex p-1 bg-stone-100 dark:bg-stone-700/50 rounded-xl mb-5">
          <button
            type="button"
            onClick={() => setMode('word')}
            className={`flex-1 py-2 text-sm font-medium rounded-lg transition-all duration-200 ${
              mode === 'word'
                ? 'bg-white dark:bg-stone-600 text-stone-900 dark:text-stone-100 shadow-sm'
                : 'text-stone-500 dark:text-stone-400 hover:text-stone-700 dark:hover:text-stone-300'
            }`}
          >
            Custom word
          </button>
          <button
            type="button"
            onClick={() => setMode('correct')}
            className={`flex-1 py-2 text-sm font-medium rounded-lg transition-all duration-200 ${
              mode === 'correct'
                ? 'bg-white dark:bg-stone-600 text-stone-900 dark:text-stone-100 shadow-sm'
                : 'text-stone-500 dark:text-stone-400 hover:text-stone-700 dark:hover:text-stone-300'
            }`}
          >
            Auto-correct
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {mode === 'word' ? (
            <div>
              <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
                Word or phrase
              </label>
              <input
                type="text"
                value={word}
                onChange={(e) => setWord(e.target.value)}
                placeholder="e.g., MentaFlux, Dr. Müller"
                className={inputClass}
                autoFocus
              />
              <p className="mt-2 text-xs text-stone-400 dark:text-stone-500">
                Ensures this word is transcribed with the exact spelling you provide.
              </p>
            </div>
          ) : (
            <>
              <div>
                <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
                  When transcribed as
                </label>
                <input
                  type="text"
                  value={phrase}
                  onChange={(e) => setPhrase(e.target.value)}
                  placeholder="e.g., mental flux"
                  className={inputClass}
                  autoFocus
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-stone-700 dark:text-stone-300 mb-2">
                  Replace with
                </label>
                <input
                  type="text"
                  value={replacement}
                  onChange={(e) => setReplacement(e.target.value)}
                  placeholder="e.g., MentaFlux"
                  className={inputClass}
                />
              </div>
            </>
          )}

          <div className="flex gap-3 justify-end pt-4">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2.5 text-sm font-medium text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!isValid}
              className="px-5 py-2.5 text-sm font-medium bg-amber-500 hover:bg-amber-600 disabled:bg-stone-300 dark:disabled:bg-stone-700 disabled:cursor-not-allowed text-white rounded-xl transition-colors shadow-sm"
            >
              {entry ? 'Save changes' : 'Add entry'}
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
    <div className="h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto px-8 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-semibold text-stone-900 dark:text-stone-100 tracking-tight">
              Dictionary
            </h1>
            <p className="text-sm text-stone-500 dark:text-stone-400 mt-0.5">
              Custom vocabulary for better accuracy
            </p>
          </div>
          <button
            onClick={handleAdd}
            className="flex items-center gap-2 px-4 py-2.5 bg-amber-500 hover:bg-amber-600 text-white text-sm font-medium rounded-xl transition-colors shadow-sm"
          >
            <PlusIcon />
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
        <div className="relative overflow-hidden rounded-2xl px-5 py-4 mb-6 bg-stone-50 dark:bg-stone-800/30 border border-stone-100 dark:border-stone-700/50">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-stone-100 dark:bg-stone-700/50">
              <svg className="w-5 h-5 text-stone-400 dark:text-stone-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
              </svg>
            </div>
            <p className="text-sm text-stone-600 dark:text-stone-300">
              Add custom words and names for accurate transcription, or set up auto-corrections
              for commonly misrecognized phrases.
            </p>
          </div>
        </div>

        {/* Dictionary list */}
        {isLoading ? (
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
              <BookIcon />
            </div>
            <h3 className="text-lg font-medium text-stone-900 dark:text-stone-100 mb-2">
              No dictionary entries
            </h3>
            <p className="text-sm text-stone-500 dark:text-stone-400 max-w-sm mx-auto mb-5">
              Add custom words for better recognition or auto-corrections for misrecognized phrases
            </p>
            <button
              onClick={handleAdd}
              className="px-5 py-2.5 bg-amber-500 hover:bg-amber-600 text-white text-sm font-medium rounded-xl transition-colors shadow-sm"
            >
              Add your first entry
            </button>
          </div>
        ) : (
          <div className="rounded-2xl overflow-hidden border border-stone-100 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-800/30">
            <table className="w-full">
              <thead>
                <tr className="border-b border-stone-100 dark:border-stone-800">
                  <th className="text-left text-xs font-semibold text-stone-400 dark:text-stone-500 uppercase tracking-wider px-4 py-3">
                    Entry
                  </th>
                  <th className="text-center text-xs font-semibold text-stone-400 dark:text-stone-500 uppercase tracking-wider px-4 py-3 w-20">
                    Active
                  </th>
                  <th className="w-24"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-stone-100 dark:divide-stone-800">
                {entries.map((entry, index) => {
                  const isVocabulary = entry.phrase === entry.replacement;
                  return (
                    <tr
                      key={entry.id}
                      className="transition-colors duration-150 hover:bg-stone-100/50 dark:hover:bg-stone-700/30 animate-fade-in"
                      style={{ animationDelay: `${index * 0.03}s` }}
                      onMouseEnter={() => setHoveredId(entry.id)}
                      onMouseLeave={() => setHoveredId(null)}
                    >
                      <td className="px-4 py-3.5">
                        <div className="flex items-center gap-2.5">
                          {isVocabulary ? (
                            <span className="text-sm font-medium text-stone-700 dark:text-stone-300">
                              {entry.phrase}
                            </span>
                          ) : (
                            <span className="flex items-center gap-2 text-sm">
                              <span className="font-medium text-stone-700 dark:text-stone-300">{entry.phrase}</span>
                              <ArrowRightIcon />
                              <span className="text-stone-600 dark:text-stone-400">{entry.replacement}</span>
                            </span>
                          )}
                          <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium uppercase tracking-wider ${
                            isVocabulary
                              ? 'bg-stone-100 dark:bg-stone-700/60 text-stone-400 dark:text-stone-500'
                              : 'bg-stone-100 dark:bg-stone-700/60 text-stone-400 dark:text-stone-500'
                          }`}>
                            {isVocabulary ? 'word' : 'replace'}
                          </span>
                        </div>
                      </td>
                      <td className="px-4 py-3.5 text-center">
                        <button
                          onClick={() => handleToggle(entry.id)}
                          className={`
                            relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200
                            ${entry.enabled
                              ? 'bg-amber-500 dark:bg-amber-400'
                              : 'bg-stone-300 dark:bg-stone-600'
                            }
                          `}
                        >
                          <span
                            className={`
                              inline-block h-4 w-4 transform rounded-full bg-white shadow-sm transition-transform duration-200
                              ${entry.enabled ? 'translate-x-6' : 'translate-x-1'}
                            `}
                          />
                        </button>
                      </td>
                      <td className="px-4 py-3.5">
                        <div
                          className={`
                            flex items-center gap-1 justify-end transition-all duration-200
                            ${hoveredId === entry.id ? 'opacity-100' : 'opacity-0'}
                          `}
                        >
                          <button
                            onClick={() => handleEdit(entry)}
                            className="p-2 bg-white dark:bg-stone-700 rounded-lg text-stone-400 dark:text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 shadow-sm transition-all duration-200"
                            title="Edit"
                          >
                            <EditIcon />
                          </button>
                          <button
                            onClick={() => handleDelete(entry.id)}
                            className="p-2 bg-white dark:bg-stone-700 rounded-lg text-stone-400 dark:text-stone-400 hover:text-red-500 dark:hover:text-red-400 shadow-sm transition-all duration-200"
                            title="Delete"
                          >
                            <TrashIcon />
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
