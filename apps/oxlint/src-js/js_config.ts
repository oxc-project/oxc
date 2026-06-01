import { importJsConfig, loadViteConfigField } from "@oxapps/shared";
import { getErrorMessage } from "./utils/utils.ts";
import { DateNow, JSONStringify } from "./utils/globals.ts";

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
 * Resolve a single config path to a `JsConfigResult`.
 * Standard mode: default export must be a plain object.
 */
async function resolveJsConfig(path: string, cacheKey: number): Promise<JsConfigResult> {
  const config = await importJsConfig(path, cacheKey);
  validateConfigExtends(config);
  return { path, config };
}

/**
 * Resolve a single Vite+ config path to a `JsConfigResult`.
 * Extracts the `.lint` field via `vite-plus`. Returns `null` config when missing (signals "skip").
 */
async function resolveVitePlusConfig(path: string): Promise<JsConfigResult> {
  const lintConfig = await loadViteConfigField(path, "lint");
  if (lintConfig === null) return { path, config: null };
  validateConfigExtends(lintConfig);
  return { path, config: lintConfig };
}

/**
 * Load config files in parallel using the given resolver, and return JSON-stringified result.
 */
async function loadConfigs(
  paths: string[],
  resolver: (path: string) => Promise<JsConfigResult>,
): Promise<string> {
  try {
    const results = await Promise.allSettled(paths.map(resolver));

    const successes: JsConfigResult[] = [];
    const errors: { path: string; error: string }[] = [];

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      if (result.status === "fulfilled") {
        successes.push(result.value);
      } else {
        errors.push({
          path: paths[i],
          error: getErrorMessage(result.reason),
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
export const loadJsConfigs: ConfigLoader = (paths) => {
  // Share one cache-busting key across the batch so that `?cache=<key>` is identical
  // for every path resolved in this call (consistent reload semantics for LSP).
  const cacheKey = DateNow();
  return loadConfigs(paths, (path) => resolveJsConfig(path, cacheKey));
};

/**
 * Load Vite+ config files in parallel, extracting the `.lint` field from each.
 */
export const loadVitePlusConfigs: ConfigLoader = (paths) =>
  loadConfigs(paths, resolveVitePlusConfig);
