import { importJsConfig, loadViteConfigField } from "@oxapps/shared";

/**
 * Load and validate a standard oxfmt JS/TS config file.
 * The default export must be a plain object containing oxfmt options.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export function loadJsConfig(path: string): Promise<object> {
  return importJsConfig(path, Date.now());
}

/**
 * Load a Vite+ config file (`vite.config.ts`) via `vite-plus`'s `resolveConfig` and extract the `.fmt` field.
 *
 * @param path - Absolute path to the Vite config file
 * @returns Config object from `.fmt` field, or `null` to signal "skip"
 */
export async function loadVitePlusConfig(path: string): Promise<object | null> {
  return loadViteConfigField(path, "fmt");
}
