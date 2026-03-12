import { basename as pathBasename } from "node:path";
import { pathToFileURL } from "node:url";

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
  const { default: config } = await import(fileUrl.href);

  if (config === undefined) throw new Error(`Configuration file has no default export: ${path}`);
  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new Error(`Configuration file must have a default export that is an object: ${path}`);
  }

  // Vite config: extract `.fmt` field
  if (pathBasename(path) === VITE_CONFIG_NAME) {
    const fmtConfig = (config as Record<string, unknown>)[VITE_OXFMT_CONFIG_FIELD];
    // NOTE: return `null` if missing (signals "skip" to Rust side)
    if (fmtConfig === undefined) return null;

    if (typeof fmtConfig !== "object" || fmtConfig === null || Array.isArray(fmtConfig)) {
      throw new Error(
        `The \`${VITE_OXFMT_CONFIG_FIELD}\` field in the default export must be an object: ${path}`,
      );
    }
    return fmtConfig;
  }

  return config;
}
