import { createRequire } from 'node:module';
import { lint } from './bindings.js';

// Lazy-load `loadPlugin` and `lintFile` functions, on first call to `loadPlugin`.
// This avoids loading this code if user doesn't utilize JS plugins.
let loadPlugin: typeof loadPluginWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;

function loadPluginWrapper(path: string, packageName?: string): Promise<string> {
  if (loadPlugin === null) {
    const require = createRequire(import.meta.url);
    // `plugins.js` is in root of `dist`. See `tsdown.config.ts`.
    ({ loadPlugin, lintFile } = require('./plugins.js'));
  }
  return loadPlugin(path, packageName);
}

function lintFileWrapper(filePath: string, bufferId: number, buffer: Uint8Array | null, ruleIds: number[]): string {
  // `lintFile` is never called without `loadPlugin` being called first, so `lintFile` must be defined here
  return lintFile(filePath, bufferId, buffer, ruleIds);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing `loadPlugin` and `lintFile` as callbacks, and CLI arguments
const success = await lint(args, loadPluginWrapper, lintFileWrapper);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
