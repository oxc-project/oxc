import { lint } from "./bindings.js";
import { debugAssertIsNonNull } from "./utils/asserts.ts";

// Lazy-loaded JS plugin-related functions.
// The type annotations below use `typeof import("./plugins/index.ts").<fn>` so that the lazy-loaded variables
// always have the exact same function signatures as the implementations exported from the plugins module.
let loadPlugin: typeof import("./plugins/index.ts").loadPlugin | null = null;
let setupRuleConfigs: typeof import("./plugins/index.ts").setupRuleConfigs | null = null;
let lintFile: typeof import("./plugins/index.ts").lintFile | null = null;
let createWorkspace: typeof import("./workspace/index.ts").createWorkspace | null = null;
let destroyWorkspace: typeof import("./workspace/index.ts").destroyWorkspace | null = null;
// Lazy-loaded JS/TS config loader (experimental)
let loadJsConfigs: typeof import("./js_config.ts").loadJsConfigs | null = null;

/**
 * Load a plugin.
 *
 * Lazy-loads plugins code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param path - Absolute path of plugin file
 * @param pluginName - Plugin name (either alias or package name)
 * @param pluginNameIsAlias - `true` if plugin name is an alias (takes priority over name that plugin defines itself)
 * @param workspaceUri - Workspace URI (`null` in CLI mode, `string` in LSP mode)
 * @returns Plugin details or error serialized to JSON string
 */
function loadPluginWrapper(
  path: string,
  pluginName: string | null,
  pluginNameIsAlias: boolean,
  workspaceUri: string | null,
): Promise<string> {
  if (loadPlugin === null) {
    // Use promises here instead of making `loadPluginWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `loadPluginWrapper`
    return import("./plugins/index.ts").then((mod) => {
      ({ loadPlugin, lintFile, setupRuleConfigs } = mod);
      return loadPlugin(path, pluginName, pluginNameIsAlias, workspaceUri);
    });
  }
  return loadPlugin(path, pluginName, pluginNameIsAlias, workspaceUri);
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
 * @param workspaceUri - Workspace URI (`null` in CLI mode, `string` in LSP mode)
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
  workspaceUri: string | null,
): string | null {
  // `lintFileWrapper` is never called without `loadPluginWrapper` being called first,
  // so `lintFile` must be defined here
  debugAssertIsNonNull(lintFile);
  return lintFile(
    filePath,
    bufferId,
    buffer,
    ruleIds,
    optionsIds,
    settingsJSON,
    globalsJSON,
    workspaceUri,
  );
}

/**
 * Create a new workspace.
 *
 * Lazy-loads workspace code on first call, so that overhead is skipped if user doesn't use JS plugins.
 *
 * @param workspace - Workspace URI
 * @returns Promise which resolves when workspace is created
 */
function createWorkspaceWrapper(workspace: string): Promise<undefined> {
  if (createWorkspace === null) {
    // Use promises here instead of making `createWorkspaceWrapper` an async function,
    // to avoid a micro-tick and extra wrapper `Promise` in all later calls to `createWorkspaceWrapper`
    return import("./workspace/index.ts").then((mod) => {
      ({ createWorkspace, destroyWorkspace } = mod);
      return createWorkspace(workspace);
    });
  }
  return Promise.resolve(createWorkspace(workspace));
}

/**
 * Destroy a workspace.
 *
 * Delegates to `destroyWorkspace`, which was lazy-loaded by `createWorkspaceWrapper`.
 *
 * @param workspace - Workspace URI
 * @returns `undefined`
 */
function destroyWorkspaceWrapper(workspace: string): undefined {
  // `destroyWorkspaceWrapper` is never called without `createWorkspaceWrapper` being called first,
  // so `destroyWorkspace` must be defined here
  debugAssertIsNonNull(destroyWorkspace);
  destroyWorkspace(workspace);
}
/**
 * Load JavaScript/TypeScript config files (experimental).
 *
 * Lazy-loads the js_config module on first call.
 * Uses native Node.js TypeScript support to import config files.
 *
 * @param paths - Array of absolute paths to JavaScript/TypeScript config files
 * @returns JSON-stringified result with all configs or error
 */
function loadJsConfigsWrapper(paths: string[]): Promise<string> {
  if (loadJsConfigs === null) {
    return import("./js_config.ts").then((mod) => {
      loadJsConfigs = mod.loadJsConfigs;
      return loadJsConfigs(paths);
    });
  }
  return loadJsConfigs(paths);
}

// Get command line arguments, skipping first 2 (node binary and script path)
const args = process.argv.slice(2);

// Node.js sets non-TTY `stdio` to non-blocking mode, which causes "Resource temporarily unavailable" errors
// in language server when passing a lot of data via stdin/stdout.
// https://github.com/oxc-project/oxc/issues/19265
// See also issue related to this workaround in `oxfmt` CLI:
// https://github.com/oxc-project/oxc/issues/17939
//
// As a workaround, if used with pipe, set blocking mode before calling NAPI bindings.
// See: https://github.com/napi-rs/napi-rs/issues/1630
if (!process.stdout.isTTY) {
  // @ts-expect-error: `_handle` is an internal API
  process.stdin._handle?.setBlocking?.(true);
  // @ts-expect-error: `_handle` is an internal API
  process.stdout._handle?.setBlocking?.(true);
}

// Call Rust, passing callbacks and CLI arguments
const success = await lint(
  args,
  loadPluginWrapper,
  setupRuleConfigsWrapper,
  lintFileWrapper,
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
  loadJsConfigsWrapper,
);

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
