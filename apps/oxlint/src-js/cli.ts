import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.ts";

// Lazy-loaded JS plugin-related functions.
// The type annotations below use `typeof import("./plugins/index.ts").<fn>` so that the lazy-loaded variables
// always have the exact same function signatures as the implementations exported from the plugins module.
let loadPlugin: typeof import("./plugins/index.ts").loadPlugin | null = null;
let setupRuleConfigs: typeof import("./plugins/index.ts").setupRuleConfigs | null = null;
let lintFile: typeof import("./plugins/index.ts").lintFile | null = null;

/**
 * Load a plugin.
 *
 * Lazy-loads plugins code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param path - Absolute path of plugin file
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @returns Plugin details or error serialized to JSON string
 */
function loadPluginWrapper(
  path: string,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
): Promise<string> {
  if (loadPlugin === null) {
    // Use promises here instead of making `loadPluginWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `loadPluginWrapper`
    return import("./plugins/index.ts").then((mod) => {
      ({ loadPlugin, lintFile, setupRuleConfigs } = mod);
      return loadPlugin(path, pluginName, pluginNameIsAlias);
    });
  }
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(path, pluginName, pluginNameIsAlias);
}

/**
 * Bootstrap configuration options.
 *
 * Delegates to `setupRuleConfigs`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 * @returns `null` if success, or error message string
 */
function setupRuleConfigsWrapper(optionsJSON: string): string | null {
  debugAssertIsNonNull(setupRuleConfigs);
  return setupRuleConfigs(optionsJSON);
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

// Call Rust, passing `loadPlugin`, `setupRuleConfigs`, and `lintFile` as callbacks, and CLI arguments
const success = await lint(args, loadPluginWrapper, setupRuleConfigsWrapper, lintFileWrapper);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
