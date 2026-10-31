import { lint } from "./bindings.js";
import {
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
  lintFileWrapper,
  loadPluginWrapper,
  setupRuleConfigsWrapper,
} from "./js_plugin_callbacks.ts";

// Lazy-loaded JS/TS config loader (experimental)
let resolvedConfigLoader: import("./js_config.ts").ConfigLoader | null = null;

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
  if (resolvedConfigLoader === null) {
    return import("./js_config.ts").then(({ loadJsConfigs, loadVitePlusConfigs }) => {
      resolvedConfigLoader = process.env.VP_VERSION ? loadVitePlusConfigs : loadJsConfigs;
      return resolvedConfigLoader(paths);
    });
  }
  return resolvedConfigLoader(paths);
}

/**
 * Get number of JS threads requested with `--js-threads` option.
 *
 * Supports `--js-threads N` and `--js-threads=N`.
 * Invalid values are ignored here. They're reported by the Rust-side argument parser.
 *
 * @param args - Command line arguments
 * @returns Number of JS threads (1 if option not specified)
 */
function getJsThreadCount(args: string[]): number {
  let value: string | undefined;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--") break;
    if (arg === "--js-threads") {
      value = args[i + 1];
    } else if (arg.startsWith("--js-threads=")) {
      value = arg.slice("--js-threads=".length);
    }
  }
  if (value === undefined || !/^\d+$/.test(value)) return 1;
  return Math.max(Number(value), 1);
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
// Same for stderr, aligned with the fix in `oxfmt` CLI.
// https://github.com/oxc-project/oxc/issues/27361
// Currently Oxlint barely writes to stderr (diagnostics go to stdout),
// but output beyond the pipe buffer would be lost otherwise.
// @ts-expect-error: `_handle` is an internal API
if (!process.stderr.isTTY) process.stderr._handle?.setBlocking?.(true);

// LSP uses stdout for communication, so write logs to stderr to avoid breaking the protocol.
// Since LSP is handled on the Rust side, we have to check the flag here. (`lint()` starts the server and waits)
// Also, for Oxlint, this actually affects:
// - loading JS/TS config files
// - Execution of JS plugins
if (args.includes("--lsp")) process.stdout.write = process.stderr.write.bind(process.stderr);

// If `--js-threads N` option is specified, spawn `N - 1` JS plugin workers to run JS plugins in parallel.
// Each worker registers its callbacks with Rust before `lint` is called.
// Not supported in LSP mode.
const jsThreadCount = args.includes("--lsp") ? 1 : getJsThreadCount(args);
let workers: import("node:worker_threads").Worker[] = [];
if (jsThreadCount > 1) {
  const { spawnJsPluginWorkers } = await import("./js_plugin_workers.ts");
  workers = await spawnJsPluginWorkers(jsThreadCount - 1);
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

// Shut down JS plugin workers
if (workers.length > 0) await Promise.all(workers.map((worker) => worker.terminate()));

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
