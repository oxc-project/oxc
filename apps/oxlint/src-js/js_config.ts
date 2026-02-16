import { getErrorMessage } from "./utils/utils.ts";
import { isDefineConfig } from "./package/config.ts";
import { JSONStringify } from "./utils/globals.ts";

interface JsConfigResult {
  path: string;
  config: unknown; // Will be validated as Oxlintrc on Rust side
}

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
        if (typeof item !== "object" || item === null || Array.isArray(item)) {
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
 * Load JavaScript config files in parallel.
 *
 * Uses native Node.js TypeScript support to import the config files.
 * Each config file should have a default export containing the oxlint configuration.
 *
 * @param paths - Array of absolute paths to JavaScript/TypeScript config files
 * @returns JSON-stringified result with all configs or error
 */
export async function loadJsConfigs(paths: string[]): Promise<string> {
  try {
    const results = await Promise.allSettled(
      paths.map(async (path): Promise<JsConfigResult> => {
        const fileUrl = new URL(`file://${path}`);
        const module = await import(fileUrl.href);
        const config = module.default;

        if (config === undefined) {
          throw new Error(`Configuration file has no default export.`);
        }

        if (typeof config !== "object" || config === null || Array.isArray(config)) {
          throw new Error(`Configuration file must have a default export that is an object.`);
        }

        if (!isDefineConfig(config)) {
          throw new Error(
            `Configuration file must wrap its default export with defineConfig() from "oxlint".`,
          );
        }

        validateConfigExtends(config as object);

        return { path, config };
      }),
    );

    const successes: JsConfigResult[] = [];
    const errors: { path: string; error: string }[] = [];

    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      if (result.status === "fulfilled") {
        successes.push(result.value);
      } else {
        errors.push({ path: paths[i], error: getErrorMessage(result.reason) });
      }
    }

    // If any config failed to load, report all errors
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
