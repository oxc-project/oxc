import { basename as pathBasename } from "node:path";
import { pathToFileURL } from "node:url";

const VITE_CONFIG_NAME = "vite.config.ts";
const VITE_OXFMT_CONFIG_FIELD = "fmt";

/**
 * Load a JavaScript/TypeScript config file.
 *
 * Uses native Node.js `import()` to evaluate the config file.
 * For:
 *  - oxfmt.config.ts files, the entire default export is used as the config object.
 *  - vite.config.ts files, the config is read from the `fmt` field of the default export. If the `fmt` field is missing, an empty object is used as the config.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export async function loadJsConfig(path: string): Promise<object> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", Date.now().toString());
  const { default: rawConfig } = await import(fileUrl.href);

  let config = rawConfig;

  if (config === undefined) throw new Error(`Configuration file has no default export: ${path}`);
  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new Error(`Configuration file must have a default export that is an object: ${path}`);
  }

  if (pathBasename(path) === VITE_CONFIG_NAME) {
    config =
      VITE_OXFMT_CONFIG_FIELD in config
        ? (config as Record<string, unknown>)[VITE_OXFMT_CONFIG_FIELD]
        : {};
    if (typeof config !== "object" || config === null || Array.isArray(config)) {
      throw new Error(
        `The \`${VITE_OXFMT_CONFIG_FIELD}\` field in the default export must be an object: ${path}`,
      );
    }
  }

  return config;
}
