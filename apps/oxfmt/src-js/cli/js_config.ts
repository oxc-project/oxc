import { importJsConfig } from "@oxapps/shared";

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

const VITE_OXFMT_CONFIG_FIELD = "fmt";
/**
 * Load a Vite+ config file (`vite.config.ts`) and extract the `.fmt` field.
 *
 * @param path - Absolute path to the Vite config file
 * @returns Config object from `.fmt` field, or `null` to signal "skip"
 */
export async function loadVitePlusConfig(path: string): Promise<object | null> {
  const config = await importJsConfig(path, Date.now());

  // NOTE: Vite configs may export a function via `defineConfig(() => ({ ... }))`,
  // but we don't know the arguments to call the function.
  // Treat non-object exports as "no config" and skip for now.
  if (!isObject(config)) return null;

  const fmtConfig = (config as Record<string, unknown>)[VITE_OXFMT_CONFIG_FIELD];
  // NOTE: return `null` if missing (signals "skip" to Rust side)
  if (fmtConfig === undefined) return null;

  if (!isObject(fmtConfig)) {
    throw new Error(
      `The \`${VITE_OXFMT_CONFIG_FIELD}\` field in the default export must be an object.`,
    );
  }

  return fmtConfig;
}
