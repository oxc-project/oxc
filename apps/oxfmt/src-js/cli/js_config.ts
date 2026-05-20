import { importJsConfig, loadViteConfigField } from "@oxapps/shared";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

/**
 * Load and validate a standard oxfmt JS/TS config file.
 * The default export must be a plain object containing oxfmt options.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export async function loadJsConfig(path: string): Promise<object> {
  const config = await importJsConfig(path, Date.now());

  if (!isObject(config)) {
    throw new Error("Configuration file must have a default export that is an object.");
  }

  return config as object;
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
