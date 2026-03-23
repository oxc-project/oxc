// napi-JS `oxfmt` API entry point

import {
  format as napiFormat,
  jsTextToDoc as napiJsTextToDoc,
  resolveConfig as napiResolveConfig,
} from "./bindings";
import {
  resolvePlugins,
  formatFile,
  formatEmbeddedCode,
  formatEmbeddedDoc,
  sortTailwindClasses,
} from "./libs/apis";
import { loadJsConfig } from "./cli/js_config";
// Types are auto-generated from the JSON Schema.
import type {
  Oxfmtrc,
  FormatConfig,
  SortImportsConfig,
  SortPackageJsonConfig,
  SortTailwindcssConfig,
} from "./config.generated";

// --- Type exports ---

// Re-export all generated config types.
// So that downstream libraries can reference them in declaration emit without TS4058/TS4082 errors.
export type * from "./config.generated";

// The same naming convention as `oxlint` for consistency.
// Using `interface extends` so that TypeScript displays `OxfmtConfig` in errors
// and hovers instead of resolving to the generated `Oxfmtrc` name.
export interface OxfmtConfig extends Oxfmtrc {}

// Backward-compatible type aliases using `Options` suffix.

/**
 * Configuration options for the `format()` API.
 *
 * Based on `FormatConfig` generated from the JSON Schema,
 * with additional deprecated aliases for backward compatibility.
 * @deprecated Use `FormatConfig` instead.
 */
export type FormatOptions = FormatConfig & {
  /** @deprecated Use `sortImports` instead. */
  experimentalSortImports?: SortImportsConfig;
  /** @deprecated Use `sortPackageJson` instead. */
  experimentalSortPackageJson?: boolean | SortPackageJsonConfig;
  /** @deprecated Use `sortTailwindcss` instead. */
  experimentalTailwindcss?: SortTailwindcssConfig;
};
/** @deprecated Use `FormatConfig["sortImports"]` instead. */
export type SortImportsOptions = SortImportsConfig;
/** @deprecated Use `FormatConfig["sortPackageJson"]` instead. */
export type SortPackageJsonOptions = SortPackageJsonConfig;
/** @deprecated Use `FormatConfig["sortTailwindcss"]` instead. */
export type SortTailwindcssOptions = SortTailwindcssConfig;
/** @deprecated Use `FormatConfig["sortTailwindcss"]` instead. */
export type TailwindcssOptions = SortTailwindcssConfig;

// --- Function exports ---

/**
 * Define an oxfmt configuration with type inference.
 */
export function defineConfig<T extends OxfmtConfig>(config: T): T {
  return config;
}

/**
 * Format the given source text according to the specified options.
 */
export async function format(
  fileName: string,
  sourceText: string,
  options?: FormatConfig,
) {
  if (typeof fileName !== "string")
    throw new TypeError("`fileName` must be a string");
  if (typeof sourceText !== "string")
    throw new TypeError("`sourceText` must be a string");

  return napiFormat(
    fileName,
    sourceText,
    options ?? {},
    resolvePlugins,
    (options, code) => formatFile({ options, code }),
    (options, code) => formatEmbeddedCode({ options, code }),
    (options, texts) => formatEmbeddedDoc({ options, texts }),
    (options, classes) => sortTailwindClasses({ options, classes }),
  );
}

/**
 * Resolve the effective configuration for the given file path.
 *
 * Returns `null` when neither an Oxfmt config file nor `.editorconfig` is found.
 */
export async function resolveConfig(
  fileName: string,
): Promise<FormatConfig | null> {
  if (typeof fileName !== "string")
    throw new TypeError("`fileName` must be a string");
  return napiResolveConfig(fileName, loadJsConfig);
}

/**
 * Format a JS/TS snippet for Prettier `textToDoc()` plugin flow.
 */
export async function jsTextToDoc(
  sourceExt: string,
  sourceText: string,
  oxfmtPluginOptionsJson: string,
  parentContext: string,
) {
  return napiJsTextToDoc(
    sourceExt,
    sourceText,
    oxfmtPluginOptionsJson,
    parentContext,
    resolvePlugins,
    (_options, _code) => Promise.reject(/* Unreachable */),
    (options, code) => formatEmbeddedCode({ options, code }),
    (options, texts) => formatEmbeddedDoc({ options, texts }),
    (options, classes) => sortTailwindClasses({ options, classes }),
  );
}
