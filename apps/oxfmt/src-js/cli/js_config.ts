import { pathToFileURL } from "node:url";

/**
 * Load a JavaScript/TypeScript config file.
 *
 * Uses native Node.js `import()` to evaluate the config file.
 * The config file should have a default export containing the oxfmt configuration object.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export async function loadJsConfig(path: string): Promise<object> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", Date.now().toString());
  const { default: config } = await import(fileUrl.href);

  if (config === undefined) throw new Error(`Configuration file has no default export: ${path}`);
  if (typeof config !== "object" || config === null || Array.isArray(config)) {
    throw new Error(`Configuration file must have a default export that is an object: ${path}`);
  }

  return config;
}
