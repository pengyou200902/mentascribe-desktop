#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';

const [, , command, ...args] = process.argv;

if (!command) {
  console.error('usage: node scripts/with-windows-cuda-env.mjs <command> [...args]');
  process.exit(2);
}

const env = { ...process.env };

if (process.platform === 'win32') {
  configureWindowsCudaEnv(env);
}

const result = spawnSync(command, args, {
  env,
  stdio: 'inherit',
  shell: process.platform === 'win32',
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function configureWindowsCudaEnv(env) {
  const cl = env.CMAKE_CUDA_HOST_COMPILER || env.CUDAHOSTCXX || findVs2022Cl();
  const ninja = env.CMAKE_MAKE_PROGRAM || findNinja();

  if (cl) {
    env.CMAKE_CUDA_HOST_COMPILER = cl;
    env.CUDAHOSTCXX = cl;
  }

  if (ninja) {
    env.CMAKE_GENERATOR = env.CMAKE_GENERATOR || 'Ninja';
    env.CMAKE_MAKE_PROGRAM = ninja;
  }

  if (!cl || !ninja) {
    const missing = [!cl && 'VS 2022 cl.exe', !ninja && 'ninja.exe']
      .filter(Boolean)
      .join(' and ');
    console.warn(
      `[windows-cuda-env] ${missing} not found; continuing with the ambient build environment.`,
    );
  }
}

function findVs2022Cl() {
  const roots = [
    process.env['ProgramFiles(x86)'],
    process.env.ProgramFiles,
  ].filter(Boolean);
  const editions = ['BuildTools', 'Community', 'Professional', 'Enterprise'];
  const candidates = [];

  for (const root of roots) {
    for (const edition of editions) {
      const toolsRoot = join(
        root,
        'Microsoft Visual Studio',
        '2022',
        edition,
        'VC',
        'Tools',
        'MSVC',
      );
      for (const version of listDirs(toolsRoot)) {
        candidates.push(join(toolsRoot, version, 'bin', 'Hostx64', 'x64', 'cl.exe'));
      }
    }
  }

  return candidates.filter(existsSync).sort(comparePathsDesc)[0];
}

function findNinja() {
  const fromPath = findOnPath('ninja.exe');
  if (fromPath) return fromPath;

  const roots = [
    process.env.ProgramFiles,
    process.env['ProgramFiles(x86)'],
  ].filter(Boolean);
  const candidates = [];

  for (const root of roots) {
    for (const year of ['18', '2022']) {
      for (const edition of ['Community', 'BuildTools', 'Professional', 'Enterprise']) {
        candidates.push(
          join(
            root,
            'Microsoft Visual Studio',
            year,
            edition,
            'Common7',
            'IDE',
            'CommonExtensions',
            'Microsoft',
            'CMake',
            'Ninja',
            'ninja.exe',
          ),
        );
      }
    }
  }

  return candidates.find(existsSync);
}

function findOnPath(executable) {
  for (const entry of (process.env.PATH || '').split(';')) {
    const candidate = join(entry, executable);
    if (existsSync(candidate)) return candidate;
  }
}

function listDirs(path) {
  try {
    return readdirSync(path, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);
  } catch {
    return [];
  }
}

function comparePathsDesc(a, b) {
  return dirname(b).localeCompare(dirname(a), undefined, { numeric: true });
}
