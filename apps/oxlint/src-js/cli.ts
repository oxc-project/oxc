import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.ts";

// Lazy-loaded JS plugin-related functions.
// Using `typeof wrapper` here makes TS check that the function signatures of `loadPlugin` and `loadPluginWrapper`
// are identical. Ditto `lintFile` and `lintFileWrapper`.
let loadPlugin: typeof loadPluginWrapper | null = null;
let setupConfigs: typeof setupConfigsWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;
let createWorkspace: typeof import("./workspace/index.ts").createWorkspace | null = null;
let destroyWorkspace: typeof destroyWorkspaceWrapper | null = null;

/**
 * Creates a new workspace.
 *
 * Lazy-loads workspace code on first call, so that overhead is skipped.
 *
 * @param workspace - Workspace URI
 * @returns Promise which resolves when workspace is created
 */
function createWorkspaceWrapper(workspace: string): Promise<undefined> {
  if (createWorkspace === null) {
    return import("./workspace/index.ts").then((mod) => {
      ({ createWorkspace, destroyWorkspace } = mod);
      return createWorkspace(workspace);
    });
  }
  debugAssertIsNonNull(createWorkspace);
  return Promise.resolve(createWorkspace(workspace));
}

/**
 *
 * Destroys a workspace.
 *
 * @param workspace - Workspace URI
 * @returns `undefined`
 */
function destroyWorkspaceWrapper(workspace: string): undefined {
  debugAssertIsNonNull(destroyWorkspace);
  destroyWorkspace(workspace);
}

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
      ({ loadPlugin, lintFile, setupConfigs } = mod);
      return loadPlugin(path, pluginName, pluginNameIsAlias);
    });
  }
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(path, pluginName, pluginNameIsAlias);
}

/**
 * Bootstrap configuration options.
 *
 * Delegates to `setupConfigs`, which was lazy-loaded by `loadPluginWrapper`.
 *
 * @param optionsJSON - Array of all rule options across all configurations, serialized as JSON
 * @returns `null` if success, or error message string
 */
function setupConfigsWrapper(optionsJSON: string): string | null {
  debugAssertIsNonNull(setupConfigs);
  return setupConfigs(optionsJSON);
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

// Call Rust, passing `loadPlugin`, `setupConfigs`, `lintFile`, `createWorkspace`, and `destroyWorkspace` as callbacks, and CLI arguments
const success = await lint(
  args,
  loadPluginWrapper,
  setupConfigsWrapper,
  lintFileWrapper,
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
