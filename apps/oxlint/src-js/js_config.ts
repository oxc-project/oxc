import { getErrorMessage } from "./utils/utils.ts";
import { DateNow, JSONStringify } from "./utils/globals.ts";
import { getUnsupportedTypeScriptModuleLoadHintForError } from "./utils/node_version.ts";

interface JsConfigResult {
  path: string;
  config: unknown; // Will be validated as Oxlintrc on Rust side, `null` means "skip this config"
}

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

type LoadJsConfigsResult =
  | { Success: JsConfigResult[] }
  | { Failures: { path: string; error: string }[] }
  | { Error: string };

function validateConfigExtends(root: object): void {
  const visited = new WeakSet<object>();
  const inStack = new WeakSet<object>();
  const stackObjects: object[] = [];
  const stackPaths: string[] = [];

  const formatCycleError = (refPath: string, cycleStart: string, idx: number): string => {
    const cycle =
      idx === -1
        ? `${cycleStart} -> ${cycleStart}`
        : [...stackPaths.slice(idx), cycleStart].join(" -> ");

    return (
      "`extends` contains a circular reference.\n\n" +
      `${refPath} points back to ${cycleStart}\n` +
      `Cycle: ${cycle}`
    );
  };

  const visit = (config: object, path: string): void => {
    if (visited.has(config)) return;
    if (inStack.has(config)) {
      const idx = stackObjects.indexOf(config);
      const cycleStart = idx === -1 ? "<unknown>" : stackPaths[idx];
      throw new Error(formatCycleError(path, cycleStart, idx));
    }

    inStack.add(config);
    stackObjects.push(config);
    stackPaths.push(path);

    const maybeExtends = (config as Record<string, unknown>).extends;
    if (maybeExtends !== undefined) {
      if (!Array.isArray(maybeExtends)) {
        throw new Error(
          "`extends` must be an array of config objects (strings/paths are not supported).",
        );
      }
      for (let i = 0; i < maybeExtends.length; i++) {
        const item = maybeExtends[i];
        if (!isObject(item)) {
          throw new Error(
            `\`extends[${i}]\` must be a config object (strings/paths are not supported).`,
          );
        }

        const itemPath = `${path}.extends[${i}]`;
        if (inStack.has(item)) {
          const idx = stackObjects.indexOf(item);
          const cycleStart = idx === -1 ? "<unknown>" : stackPaths[idx];
          throw new Error(formatCycleError(itemPath, cycleStart, idx));
        }

        visit(item, itemPath);
      }
    }

    inStack.delete(config);
    stackObjects.pop();
    stackPaths.pop();
    visited.add(config);
  };

  visit(root, "<root>");
}

/**
 * Import a JS/TS config file and return its default export.
 */
async function importConfig(path: string, cacheKey: number): Promise<unknown> {
  // Bypass Node.js module cache to allow reloading changed config files (used for LSP)
  const fileUrl = new URL(`file://${path}?cache=${cacheKey}`);
  const module = await import(fileUrl.href);
  const config = module.default;

  if (config === undefined) {
    throw new Error(`Configuration file has no default export.`);
  }

  return config;
}

/**
 * Resolve a single config path to a `JsConfigResult`.
 * Standard mode: default export must be a plain object.
 */
async function resolveJsConfig(path: string, cacheKey: number): Promise<JsConfigResult> {
  const config = await importConfig(path, cacheKey);

  if (!isObject(config)) {
    throw new Error(`Configuration file must have a default export that is an object.`);
  }
  validateConfigExtends(config as object);
  return { path, config };
}

const VITE_OXLINT_CONFIG_FIELD = "lint";

/**
 * Resolve a single Vite+ config path to a `JsConfigResult`.
 * Extracts the `.lint` field. Returns `null` config when missing (signals "skip").
 */
async function resolveVitePlusConfig(path: string, cacheKey: number): Promise<JsConfigResult> {
  const config = await importConfig(path, cacheKey);

  // NOTE: Vite configs may export a function via `defineConfig(() => ({ ... }))`,
  // but we don't know the arguments to call the function.
  // Treat non-object exports as "no config" and skip.
  if (!isObject(config)) {
    return { path, config: null };
  }

  const lintConfig = (config as Record<string, unknown>)[VITE_OXLINT_CONFIG_FIELD];
  // NOTE: return `null` if `.lint` is missing which signals "skip" this
  if (lintConfig === undefined) {
    return { path, config: null };
  }

  if (!isObject(lintConfig)) {
    throw new Error(
      `The \`${VITE_OXLINT_CONFIG_FIELD}\` field in the default export must be an object.`,
    );
  }
  validateConfigExtends(lintConfig as object);
  return { path, config: lintConfig };
}

/**
 * Load config files in parallel using the given resolver, and return JSON-stringified result.
 */
async function loadConfigs(
  paths: string[],
  resolver: (path: string, cacheKey: number) => Promise<JsConfigResult>,
): Promise<string> {
  try {
    const cacheKey = DateNow();
    const results = await Promise.allSettled(paths.map((path) => resolver(path, cacheKey)));

    const successes: JsConfigResult[] = [];
    const errors: { path: string; error: string }[] = [];

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      if (result.status === "fulfilled") {
        successes.push(result.value);
      } else {
        const path = paths[i];
        const unsupportedNodeHint = getUnsupportedTypeScriptModuleLoadHintForError(
          result.reason,
          path,
        );
        errors.push({
          path,
          error: unsupportedNodeHint ?? getErrorMessage(result.reason),
        });
      }
    }

    if (errors.length > 0) {
      return JSONStringify({ Failures: errors } satisfies LoadJsConfigsResult);
    }

    return JSONStringify({ Success: successes } satisfies LoadJsConfigsResult);
  } catch (err) {
    return JSONStringify({
      Error: getErrorMessage(err),
    } satisfies LoadJsConfigsResult);
  }
}

export type ConfigLoader = (paths: string[]) => Promise<string>;

/**
 * Load standard oxlint JS/TS config files in parallel.
 */
export const loadJsConfigs: ConfigLoader = (paths) => loadConfigs(paths, resolveJsConfig);

/**
 * Load Vite+ config files in parallel, extracting the `.lint` field from each.
 */
export const loadVitePlusConfigs: ConfigLoader = (paths) =>
  loadConfigs(paths, resolveVitePlusConfig);
