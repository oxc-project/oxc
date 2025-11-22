import { lint } from './bindings.js';
import { debugAssertIsNonNull } from './utils/asserts.js';

// Lazy-loaded JS plugin-related functions.
// Using `typeof wrapper` here makes TS check that the function signatures of `loadPlugin` and `loadPluginWrapper`
// are identical. Ditto `lintFile` and `lintFileWrapper`.
let loadPlugin: typeof loadPluginWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;

/**
 * Load a plugin.
 *
 * Lazy-loads plugins code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param path - Absolute path of plugin file
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns Plugin details or error serialized to JSON string
 */
function loadPluginWrapper(path: string, packageName: string | null): Promise<string> {
  if (loadPlugin === null) {
    // Use promises here instead of making `loadPluginWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `loadPluginWrapper`
    return import('./plugins/index.js').then((mod) => {
      ({ loadPlugin, lintFile } = mod);
      return loadPlugin(path, packageName);
    });
  }
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(path, packageName);
}

/**
 * Lint a file.
 *
 * Delegates to `lintFile`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param settingsJSON - Settings for file, as JSON
 * @returns Diagnostics or error serialized to JSON string
 */
function lintFileWrapper(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  settingsJSON: string,
): string {
  // `lintFileWrapper` is never called without `loadPluginWrapper` being called first,
  // so `lintFile` must be defined here
  debugAssertIsNonNull(lintFile);
  return lintFile(filePath, bufferId, buffer, ruleIds, settingsJSON);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing `loadPlugin` and `lintFile` as callbacks, and CLI arguments
const success = await lint(args, loadPluginWrapper, lintFileWrapper);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
