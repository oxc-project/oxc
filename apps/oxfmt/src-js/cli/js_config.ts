import { basename as pathBasename } from "node:path";
import { createJiti } from "jiti";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

const VITE_CONFIG_NAME = "vite.config.ts";
const VITE_OXFMT_CONFIG_FIELD = "fmt";

/**
 * Load a JavaScript/TypeScript config file.
 *
 * Uses `jiti` to evaluate the config file, which supports TypeScript out-of-the-box.
 * The config file should have a default export containing the oxfmt configuration object.
 *
 * For `vite.config.ts`, extracts the `.fmt` field from the default export.
 * Returns `null` if the field is missing, signaling "skip this config" to the Rust side.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object, or `null` to signal "skip"
 */
export async function loadJsConfig(path: string): Promise<object | null> {
  // Bypass module cache by creating a fresh jiti instance
  const jiti = createJiti(import.meta.url, { moduleCache: false, fsCache: false });
  const module = (await jiti.import(path)) as { default: unknown };
  const config = module.default;

  if (config === undefined) throw new Error("Configuration file has no default export.");

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
        `The \`${VITE_OXFMT_CONFIG_FIELD}\` field in the default export must be an object.`,
      );
    }
    return fmtConfig;
  }

  if (!isObject(config)) {
    throw new Error("Configuration file must have a default export that is an object.");
  }

  return config;
}
