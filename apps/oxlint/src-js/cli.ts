import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.ts";

// Lazy-loaded JS plugin-related functions.
// Using `typeof wrapper` here makes TS check that the function signatures of `loadPlugin` and `loadPluginWrapper`
// are identical. Ditto `lintFile` and `lintFileWrapper`.
let loadPlugin: typeof loadPluginWrapper | null = null;
let setupConfigs: typeof setupConfigsWrapper | null = null;
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
    return import("./plugins/index.ts").then((mod) => {
      ({ loadPlugin, lintFile, setupConfigs } = mod);
      return loadPlugin(path, packageName);
    });
  }
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(path, packageName);
}

/**
 * Bootstrap configuration options.
 *
 * Delegates to `setupConfigs`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 */
function setupConfigsWrapper(optionsJSON: string): void {
  debugAssertIsNonNull(setupConfigs);
  setupConfigs(optionsJSON);
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
 * @param optionsIds - IDs of options to use for rules on this file, in same order as `ruleIds`
 * @param settingsJSON - Settings for file, as JSON
 * @param globalsJSON - Globals for file, as JSON
 * @returns Diagnostics or error serialized to JSON string
 */
function lintFileWrapper(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
): string | null {
  // `lintFileWrapper` is never called without `loadPluginWrapper` being called first,
  // so `lintFile` must be defined here
  debugAssertIsNonNull(lintFile);
  return lintFile(filePath, bufferId, buffer, ruleIds, optionsIds, settingsJSON, globalsJSON);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing `loadPlugin`, `setupConfigs`, and `lintFile` as callbacks, and CLI arguments
const success = await lint(args, loadPluginWrapper, setupConfigsWrapper, lintFileWrapper);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
