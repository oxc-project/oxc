import { createRequire } from 'node:module';
import { lint } from './bindings.js';

// Lazy-load `loadPlugin`, `lintFile`, and parser functions, on first call to `loadPlugin`.
// This avoids loading this code if user doesn't utilize JS plugins.
let loadPlugin: typeof loadPluginWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;
let loadCustomParser: typeof loadCustomParserWrapper | null = null;
let parseWithCustomParser: typeof parseWithCustomParserWrapper | null = null;

function loadPluginWrapper(path: string, packageName?: string): Promise<string> {
  if (loadPlugin === null) {
    const require = createRequire(import.meta.url);
    // `plugins.js` is in root of `dist`. See `tsdown.config.ts`.
    ({ loadPlugin, lintFile, loadCustomParser, parseWithCustomParser } = require('./plugins.js'));
  }
  return loadPlugin(path, packageName);
}

function lintFileWrapper(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  stringifiedSettings: string,
): string {
  // `lintFile` is never called without `loadPlugin` being called first, so `lintFile` must be defined here
  return lintFile(filePath, bufferId, buffer, ruleIds, stringifiedSettings);
}

function loadCustomParserWrapper(path: string, packageName?: string): Promise<string> {
  if (loadCustomParser === null) {
    const require = createRequire(import.meta.url);
    ({ loadPlugin, lintFile, loadCustomParser, parseWithCustomParser } = require('./plugins.js'));
  }
  return loadCustomParser(path, packageName);
}

async function parseWithCustomParserWrapper(
  parserPath: string,
  code: string,
  options?: string,
): Promise<Uint8Array> {
  if (parseWithCustomParser === null) {
    const require = createRequire(import.meta.url);
    ({ loadPlugin, lintFile, loadCustomParser, parseWithCustomParser, getCustomParser } = require('./plugins.js'));
  }
  // Get the parser instance
  const parser = getCustomParser(parserPath);
  if (!parser) {
    throw new Error(`Parser not loaded: ${parserPath}`);
  }
  // Parse options from JSON string if provided
  const parserOptions = options ? JSON.parse(options) : undefined;
  const result = parseWithCustomParser(parser, code, parserOptions);
  return result.buffer;
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing `loadPlugin`, `lintFile`, `loadCustomParser`, and `parseWithCustomParser` as callbacks, and CLI arguments
const success = await lint(args, loadPluginWrapper, lintFileWrapper, loadCustomParserWrapper, parseWithCustomParserWrapper);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
