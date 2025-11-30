import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.js";

// Lazy-loaded JS plugin-related functions.
// Using `typeof wrapper` here makes TS check that the function signatures of `loadPlugin` and `loadPluginWrapper`
// are identical. Ditto `lintFile` and `lintFileWrapper`.
let loadPlugin: typeof loadPluginWrapper | null = null;
let lintFile: typeof lintFileWrapper | null = null;
let createWorkspace: typeof createWorkspaceWrapper | null = null;
let destroyWorkspace: typeof destroyWorkspaceWrapper | null = null;

/**
 * Load a plugin.
 *
 * Delegates to `loadPlugin`, which was lazy-loaded by `createWorkspaceWrapper`.
 *
 * @param workspaceDir - Workspace root directory
 * @param path - Absolute path of plugin file
 * @param packageName - Optional package name from `package.json` (fallback if `plugin.meta.name` is not defined)
 * @returns Plugin details or error serialized to JSON string
 */
function loadPluginWrapper(
  workspaceDir: string,
  path: string,
  packageName: string | null,
): Promise<string> {
  debugAssertIsNonNull(loadPlugin);
  return loadPlugin(workspaceDir, path, packageName);
}

/**
 * Lint a file.
 *
 * Delegates to `lintFile`, which was lazy-loaded by `createWorkspaceWrapper`.
 *
 * @param workspaceDir - Directory of the workspace
 * @param filePath - Absolute path of file being linted
 * @param bufferId - ID of buffer containing file data
 * @param buffer - Buffer containing file data, or `null` if buffer with this ID was previously sent to JS
 * @param ruleIds - IDs of rules to run on this file
 * @param settingsJSON - Settings for file, as JSON
 * @returns Diagnostics or error serialized to JSON string
 */
function lintFileWrapper(
  rootDir: string,
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  settingsJSON: string,
): string {
  // `lintFileWrapper` is never called without `createWorkspaceWrapper` being called first,
  // so `lintFile` must be defined here
  debugAssertIsNonNull(lintFile);
  return lintFile(rootDir, filePath, bufferId, buffer, ruleIds, settingsJSON);
}

/**
 * Create a new workspace.
 *
 * Lazy-loads workspace code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param rootDir - Root directory of the workspace
 * @returns Promise that resolves when workspace is created
 */
function createWorkspaceWrapper(rootDir: string): Promise<undefined> {
  if (createWorkspace === null) {
    // Use promises here instead of making `createWorkspaceWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `createWorkspaceWrapper`
    return import("./plugins/index.js").then((mod) => {
      ({ loadPlugin, lintFile, createWorkspace, destroyWorkspace } = mod);
      return createWorkspace(rootDir);
    });
  }

  debugAssertIsNonNull(createWorkspace);
  return Promise.resolve(createWorkspace(rootDir));
}

/**
 * Destroy a workspace.
 *
 * @param rootDir - Root directory of the workspace
 */
function destroyWorkspaceWrapper(rootDir: string): void {
  // `destroyWorkspaceWrapper` is never called without `createWorkspaceWrapper` being called first,
  // so `destroyWorkspace` must be defined here
  debugAssertIsNonNull(destroyWorkspace);
  destroyWorkspace(rootDir);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Call Rust, passing `loadPlugin`, `lintFile`, `createWorkspace` and `destroyWorkspace` as callbacks, and CLI arguments
const success = await lint(
  args,
  loadPluginWrapper,
  lintFileWrapper,
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
