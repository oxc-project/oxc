import { basename as pathBasename } from "node:path";

import { registerLanguageOptions } from "./js_language_options_registry.ts";
import { getErrorMessage } from "./utils/utils.ts";
import { getUnsupportedTypeScriptModuleLoadHintForError } from "./utils/node_version.ts";

interface JsConfigResult {
  path: string;
  config: unknown; // Will be validated as Oxlintrc on Rust side, `null` means "skip this config"
}

const LANGUAGE_OPTIONS_ID_FIELD = "_languageOptionsId";
const LANGUAGE_OPTIONS_HAS_PARSER_FIELD = "_languageOptionsHasParser";

const isObject = (v: unknown) => typeof v === "object" && v !== null && !Array.isArray(v);

const VITE_CONFIG_NAME = "vite.config.ts";
const VITE_OXLINT_CONFIG_FIELD = "lint";
const SUPPORTED_REDUNDANT_PROCESSORS = new Set(["svelte/svelte", "svelte/.svelte"]);

type LoadJsConfigsResult =
  | { Success: JsConfigResult[] }
  | { Failures: { path: string; error: string }[] }
  | { Error: string };

type NormalizedConfig = Record<string, unknown>;
type ExternalPluginConfigEntry = string | { name: string; specifier: string };

type NormalizationTarget = "root" | "override";

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

  const enter = (entry: object, path: string): void => {
    if (visited.has(entry)) return;
    if (inStack.has(entry)) {
      const idx = stackObjects.indexOf(entry);
      const cycleStart = idx === -1 ? "<unknown>" : stackPaths[idx];
      throw new Error(formatCycleError(path, cycleStart, idx));
    }

    inStack.add(entry);
    stackObjects.push(entry);
    stackPaths.push(path);

    if (Array.isArray(entry)) {
      for (let i = 0; i < entry.length; i++) {
        const item = entry[i];
        if (typeof item === "string") continue;
        if (Array.isArray(item)) {
          enter(item, `${path}[${i}]`);
          continue;
        }
        if (!isObject(item)) {
          throw new Error(`\`extends[${i}]\` must be a config object or string.`);
        }
        enter(item, `${path}[${i}]`);
      }
    } else {
      const maybeExtends = (entry as Record<string, unknown>).extends;
      if (maybeExtends !== undefined) {
        if (!Array.isArray(maybeExtends)) {
          throw new Error("`extends` must be an array of config objects or strings.");
        }
        for (let i = 0; i < maybeExtends.length; i++) {
          const item = maybeExtends[i];
          if (typeof item === "string") continue;
          if (Array.isArray(item)) {
            enter(item, `${path}.extends[${i}]`);
            continue;
          }
          if (!isObject(item)) {
            throw new Error(`\`extends[${i}]\` must be a config object or string.`);
          }
          enter(item, `${path}.extends[${i}]`);
        }
      }
    }

    inStack.delete(entry);
    stackObjects.pop();
    stackPaths.pop();
    visited.add(entry);
  };

  enter(root, "<root>");
}

function isSupportedRedundantProcessor(processor: unknown): processor is string {
  return typeof processor === "string" && SUPPORTED_REDUNDANT_PROCESSORS.has(processor);
}

function normalizeFlatPluginMap(
  value: Record<string, unknown>,
  path: string,
): ExternalPluginConfigEntry[] {
  return Object.entries(value).map(([pluginName, pluginValue]) => {
    if (!isObject(pluginValue)) {
      throw new Error(`${path}.plugins.${pluginName} must be a plugin object.`);
    }

    const meta = (pluginValue as Record<string, unknown>).meta;
    if (!isObject(meta) || typeof meta.name !== "string" || meta.name.length === 0) {
      throw new Error(
        `${path}.plugins.${pluginName} must define \`meta.name\` as a package name string so Oxlint can resolve it.`,
      );
    }

    return {
      name: pluginName,
      specifier: meta.name,
    };
  });
}

function mergeExternalPluginEntries(
  existing: unknown,
  additions: ExternalPluginConfigEntry[],
  path: string,
): ExternalPluginConfigEntry[] {
  if (existing === undefined) return additions;
  if (!Array.isArray(existing)) throw new Error(`${path}.jsPlugins must be an array.`);
  return [...existing, ...additions] as ExternalPluginConfigEntry[];
}

function normalizeConfigForRust(root: object): Record<string, unknown> {
  const normalizedConfigs = new WeakMap<
    object,
    { root?: NormalizedConfig; override?: NormalizedConfig }
  >();

  const normalizeExtends = (value: unknown, path: string): Array<string | NormalizedConfig> => {
    if (!Array.isArray(value)) {
      throw new Error("`extends` must be an array of config objects or strings.");
    }

    const normalized: Array<string | NormalizedConfig> = [];

    const append = (item: unknown, itemPath: string): void => {
      if (typeof item === "string") {
        normalized.push(item);
        return;
      }
      if (Array.isArray(item)) {
        for (let i = 0; i < item.length; i++) {
          append(item[i], `${itemPath}[${i}]`);
        }
        return;
      }
      if (!isObject(item)) {
        throw new Error(`\`extends[${normalized.length}]\` must be a config object or string.`);
      }
      normalized.push(normalizeRoot(item, itemPath));
    };

    for (let i = 0; i < value.length; i++) {
      append(value[i], `${path}.extends[${i}]`);
    }

    return normalized;
  };

  const populateNormalizedConfig = (
    normalized: NormalizedConfig,
    config: object,
    path: string,
    target: NormalizationTarget,
  ): void => {
    let inferredJsPlugins: ExternalPluginConfigEntry[] = [];

    for (const [key, value] of Object.entries(config as Record<string, unknown>)) {
      if (key === "languageOptions") {
        if (!isObject(value)) throw new Error(`${path}.languageOptions must be an object.`);
        normalized[LANGUAGE_OPTIONS_ID_FIELD] = registerLanguageOptions(value);
        if (Object.hasOwn(value, "parser")) {
          normalized[LANGUAGE_OPTIONS_HAS_PARSER_FIELD] =
            (value as Record<string, unknown>).parser != null;
        }
        continue;
      }

      if (key === "extends" && value !== undefined) {
        if (target === "override") {
          throw new Error(`${path}.extends is not supported inside override-like flat config fragments.`);
        }
        normalized.extends = normalizeExtends(value, path);
        continue;
      }

      if (key === "overrides" && value !== undefined) {
        if (target === "override") {
          throw new Error(
            `${path}.overrides is not supported inside override-like flat config fragments.`,
          );
        }
        if (!Array.isArray(value)) throw new Error("`overrides` must be an array.");
        normalized.overrides = value.map((item, index) => {
          if (!isObject(item)) throw new Error(`\`overrides[${index}]\` must be an object.`);
          return normalizeOverride(item, `${path}.overrides[${index}]`);
        });
        continue;
      }

      if (key === "plugins" && value !== undefined) {
        if (Array.isArray(value)) {
          normalized.plugins = value;
          continue;
        }
        if (!isObject(value)) throw new Error(`${path}.plugins must be an array or object.`);
        inferredJsPlugins = inferredJsPlugins.concat(normalizeFlatPluginMap(value, path));
        continue;
      }

      if (key === "jsPlugins" && value !== undefined) {
        if (!Array.isArray(value)) throw new Error(`${path}.jsPlugins must be an array.`);
        normalized.jsPlugins = value;
        continue;
      }

      if (key === "files") {
        if (target !== "override") {
          throw new Error(`${path}.files is only supported in flat config fragments inside \`extends\`.`);
        }
        normalized.files = value;
        continue;
      }

      if (key === "processor") {
        if (!isSupportedRedundantProcessor(value)) {
          throw new Error(
            `${path}.processor=${JSON.stringify(value)} is not supported by Oxlint's flat-config compatibility layer.`,
          );
        }
        continue;
      }

      if (key === "name") {
        continue;
      }

      if (key === "ignorePatterns") {
        if (!Array.isArray(value) || value.some((pattern) => typeof pattern !== "string")) {
          throw new Error(`${path}.ignorePatterns must be an array of glob strings.`);
        }

        const ignorePatterns = normalized.ignorePatterns;
        if (ignorePatterns === undefined) {
          normalized.ignorePatterns = value;
        } else if (Array.isArray(ignorePatterns)) {
          normalized.ignorePatterns = [...ignorePatterns, ...value];
        } else {
          throw new Error(`${path}.ignorePatterns must be an array.`);
        }
        continue;
      }

      if (key === "ignores") {
        if (!Array.isArray(value) || value.some((pattern) => typeof pattern !== "string")) {
          throw new Error(`${path}.ignores must be an array of glob strings.`);
        }
        if (target === "override") {
          throw new Error(
            `${path}.ignores is only supported on flat config fragments without \`files\` because Oxlint overrides do not support per-override ignore globs.`,
          );
        }

        const ignorePatterns = normalized.ignorePatterns;
        if (ignorePatterns === undefined) {
          normalized.ignorePatterns = value;
        } else if (Array.isArray(ignorePatterns)) {
          normalized.ignorePatterns = [...ignorePatterns, ...value];
        } else {
          throw new Error(`${path}.ignorePatterns must be an array when combined with flat-config \`ignores\`.`);
        }
        continue;
      }

      normalized[key] = value;
    }

    if (inferredJsPlugins.length > 0) {
      normalized.jsPlugins = mergeExternalPluginEntries(normalized.jsPlugins, inferredJsPlugins, path);
    }
  };

  const normalizeRoot = (config: object, path: string): NormalizedConfig => {
    const cached = normalizedConfigs.get(config)?.root;
    if (cached !== undefined) return cached;

    const entry = normalizedConfigs.get(config) ?? {};
    const normalized: NormalizedConfig = {};
    entry.root = normalized;
    normalizedConfigs.set(config, entry);

    if (Object.hasOwn(config as Record<string, unknown>, "files")) {
      normalized.overrides = [normalizeOverride(config, path)];
      return normalized;
    }

    populateNormalizedConfig(normalized, config, path, "root");
    return normalized;
  };

  const normalizeOverride = (config: object, path: string): NormalizedConfig => {
    const cached = normalizedConfigs.get(config)?.override;
    if (cached !== undefined) return cached;

    const entry = normalizedConfigs.get(config) ?? {};
    const normalized: NormalizedConfig = {};
    entry.override = normalized;
    normalizedConfigs.set(config, entry);

    populateNormalizedConfig(normalized, config, path, "override");
    return normalized;
  };

  return normalizeRoot(root, "<root>");
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
    const cacheKey = Date.now();
    const results = await Promise.allSettled(
      paths.map(async (path): Promise<JsConfigResult> => {
        // Bypass Node.js module cache to allow reloading changed config files (used for LSP, where we reload configs after important changes)
        const fileUrl = new URL(`file://${path}?cache=${cacheKey}`);
        const module = await import(fileUrl.href);
        const config = module.default;

        if (config === undefined) {
          throw new Error(`Configuration file has no default export.`);
        }

        // Vite config: extract `.lint` field, skip `defineConfig()` validation
        if (pathBasename(path) === VITE_CONFIG_NAME) {
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
          return { path, config: normalizeConfigForRust(lintConfig as object) };
        }

        if (!isObject(config)) {
          throw new Error(`Configuration file must have a default export that is an object.`);
        }
        validateConfigExtends(config as object);
        return { path, config: normalizeConfigForRust(config as object) };
      }),
    );

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

    // If any config failed to load, report all errors
    if (errors.length > 0) {
      return JSON.stringify({ Failures: errors } satisfies LoadJsConfigsResult);
    }

    return JSON.stringify({ Success: successes } satisfies LoadJsConfigsResult);
  } catch (err) {
    return JSON.stringify({
      Error: getErrorMessage(err),
    } satisfies LoadJsConfigsResult);
  }
}
