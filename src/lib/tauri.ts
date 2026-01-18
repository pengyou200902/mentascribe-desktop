import { invoke } from '@tauri-apps/api/core';

export interface AuthToken {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: string;
    email: string;
    name?: string;
    avatar_url?: string;
  };
}

export interface ModelInfo {
  id: string;
  name: string;
  size_mb: number;
  downloaded: boolean;
}

/**
 * Start audio recording
 */
export async function startRecording(): Promise<void> {
  return invoke('start_recording');
}

/**
 * Stop recording and get transcribed text
 */
export async function stopRecording(): Promise<string> {
  return invoke('stop_recording');
}

/**
 * Inject text into the active application
 */
export async function injectText(text: string): Promise<void> {
  return invoke('inject_text', { text });
}

/**
 * Login with email and password
 */
export async function login(email: string, password: string): Promise<AuthToken> {
  return invoke('login', { email, password });
}

/**
 * Download a Whisper model
 */
export async function downloadModel(size: string): Promise<void> {
  return invoke('download_model', { size });
}

/**
 * Get list of available models
 */
export async function getAvailableModels(): Promise<ModelInfo[]> {
  return invoke('get_available_models');
}
