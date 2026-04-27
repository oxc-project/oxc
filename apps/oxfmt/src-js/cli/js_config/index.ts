import { pathToFileURL } from "node:url";
import { getUnsupportedTypeScriptModuleLoadHint } from "./node_version";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

/**
 * Load a JavaScript/TypeScript config file and import it.
 *
 * Uses native Node.js `import()` to evaluate the config file.
 * The config file should have a default export containing the oxfmt configuration object.
 */
async function importJsConfig(path: string): Promise<unknown> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", Date.now().toString());

  const { default: config } = await import(fileUrl.href).catch((err) => {
    const hint = getUnsupportedTypeScriptModuleLoadHint(err, path);
    if (hint && err instanceof Error) err.message += `\n\n${hint}`;
    throw err;
  });

  if (config === undefined) throw new Error("Configuration file has no default export.");

  return config;
}

/**
 * Load and validate a standard oxfmt JS/TS config file.
 * The default export must be a plain object containing oxfmt options.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export async function loadJsConfig(path: string): Promise<object> {
  const config = await importJsConfig(path);

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
  const config = await importJsConfig(path);

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
