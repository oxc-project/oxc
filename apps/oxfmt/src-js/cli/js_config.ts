import { basename as pathBasename } from "node:path";
import { pathToFileURL } from "node:url";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

const VITE_CONFIG_NAME = "vite.config.ts";
const VITE_OXFMT_CONFIG_FIELD = "fmt";

/**
 * Load a JavaScript/TypeScript config file.
 *
 * Uses native Node.js `import()` to evaluate the config file.
 * The config file should have a default export containing the oxfmt configuration object.
 *
 * For `vite.config.ts`, extracts the `.fmt` field from the default export.
 * Returns `null` if the field is missing, signaling "skip this config" to the Rust side.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object, or `null` to signal "skip"
 */
export async function loadJsConfig(path: string): Promise<object | null> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", Date.now().toString());

  // NOTE: Suppress stdout during `import()`.
  // This is necessary because some config files may contain `console.log()` statements for debugging,
  // and we don't want those logs to interfere with the LSP protocol stream (which uses stdout for communication).
  // Without this, LSP message frames can get corrupted, leading to parsing errors and broken LSP functionality.
  // As a side effect, this will also suppress any logs for CLI usage,
  // but it's a trade-off and keep our CLI output clean and focused on errors and results.
  const origStdoutWrite = process.stdout.write.bind(process.stdout);
  process.stdout.write = () => true;
  let config: unknown;
  try {
    ({ default: config } = await import(fileUrl.href));
  } finally {
    process.stdout.write = origStdoutWrite;
  }

  if (config === undefined) throw new Error(`Configuration file has no default export: ${path}`);

  // Vite config: extract `.fmt` field
  if (pathBasename(path) === VITE_CONFIG_NAME) {
    // NOTE: Vite configs may export a function via `defineConfig(() => ({ ... }))`,
    // but we don't know the arguments to call the function.
    // Treat non-object exports as "no config" and skip.
    if (!isObject(config)) return null;

    const fmtConfig = (config as Record<string, unknown>)[VITE_OXFMT_CONFIG_FIELD];
    // NOTE: return `null` if missing (signals "skip" to Rust side)
    if (fmtConfig === undefined) return null;

    if (!isObject(fmtConfig)) {
      throw new Error(
        `The \`${VITE_OXFMT_CONFIG_FIELD}\` field in the default export must be an object: ${path}`,
      );
    }
    return fmtConfig;
  }

  if (!isObject(config)) {
    throw new Error(`Configuration file must have a default export that is an object: ${path}`);
  }

  return config;
}
