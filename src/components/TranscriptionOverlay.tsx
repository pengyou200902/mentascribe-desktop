import { FC } from 'react';

interface TranscriptionOverlayProps {
  isRecording: boolean;
  isProcessing: boolean;
  lastTranscription: string | null;
}

export const TranscriptionOverlay: FC<TranscriptionOverlayProps> = ({
  isRecording,
  isProcessing,
  lastTranscription,
}) => {
  return (
    <div className="space-y-4">
      {/* Status indicator */}
      <div className="flex flex-col items-center justify-center py-8">
        <div
          className={`w-24 h-24 rounded-full flex items-center justify-center transition-all ${
            isRecording
              ? 'bg-red-500/20 border-2 border-red-500 animate-pulse'
              : isProcessing
              ? 'bg-yellow-500/20 border-2 border-yellow-500'
              : 'bg-gray-700 border-2 border-gray-600'
          }`}
        >
          {isRecording ? (
            <svg
              className="w-12 h-12 text-red-500"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
              <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
            </svg>
          ) : isProcessing ? (
            <svg
              className="w-12 h-12 text-yellow-500 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          ) : (
            <svg
              className="w-12 h-12 text-gray-500"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
              <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
            </svg>
          )}
        </div>

        <p className="mt-4 text-lg">
          {isRecording
            ? 'Listening...'
            : isProcessing
            ? 'Processing...'
            : 'Ready'}
        </p>
      </div>

      {/* Last transcription */}
      {lastTranscription && !isRecording && !isProcessing && (
        <div className="bg-gray-800 rounded-lg p-4">
          <h3 className="text-sm text-gray-400 mb-2">Last transcription:</h3>
          <p className="text-white">{lastTranscription}</p>
        </div>
      )}
    </div>
  );
};
