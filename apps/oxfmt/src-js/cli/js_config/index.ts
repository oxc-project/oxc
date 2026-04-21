import { pathToFileURL } from "node:url";
import { getUnsupportedTypeScriptModuleLoadHint } from "./node_version";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

/**
 * Load and validate a standard oxfmt JS/TS config file.
 * The default export must be a plain object containing oxfmt options.
 *
 * @param path - Absolute path to the JavaScript/TypeScript config file
 * @returns Config object
 */
export async function loadJsConfig(path: string): Promise<object> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = pathToFileURL(path);
  fileUrl.searchParams.set("cache", Date.now().toString());

  const { default: config } = await import(fileUrl.href).catch((err) => {
    const hint = getUnsupportedTypeScriptModuleLoadHint(err, path);
    if (hint && err instanceof Error) err.message += `\n\n${hint}`;
    throw err;
  });

  if (config === undefined) throw new Error("Configuration file has no default export.");

  if (!isObject(config)) {
    throw new Error("Configuration file must have a default export that is an object.");
  }

  return config as object;
}

// ---

const VP_OXFMT_CONFIG_FIELD = "fmt";
let vitePlusCache = null as typeof import("vite-plus") | null;

/**
 * Load a Vite+ config file (`vite.config.ts`) via `vite-plus`'s `resolveConfig` and extract the `.fmt` field.
 *
 * @param path - Absolute path to the Vite config file
 * @returns Config object from `.fmt` field, or `null` to signal "skip"
 */
export async function loadVitePlusConfig(path: string): Promise<object | null> {
  vitePlusCache ??= await import("vite-plus");
  const config = await vitePlusCache.resolveConfig({ configFile: path }, "build");

  // NOTE: return `null` if `.fmt` is missing (signals "skip" to Rust side)
  if (VP_OXFMT_CONFIG_FIELD in config === false) return null;

  const fmtConfig = config[VP_OXFMT_CONFIG_FIELD];

  if (!isObject(fmtConfig)) {
    throw new Error(
      `The \`${VP_OXFMT_CONFIG_FIELD}\` field in the default export must be an object.`,
    );
  }

  return fmtConfig as object;
}
