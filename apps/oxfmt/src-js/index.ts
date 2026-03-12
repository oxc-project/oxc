// napi-JS `oxfmt` API entry point
// See also `format()` function in `./src/main_napi.rs`

import { format as napiFormat, jsTextToDoc as napiJsTextToDoc } from "./bindings";
import {
  resolvePlugins,
  formatFile,
  formatEmbeddedCode,
  formatEmbeddedDoc,
  sortTailwindClasses,
} from "./libs/apis";
// Types are auto-generated from the JSON Schema.
// See `config.generated.ts` for the full list of generated types.
import type {
  Oxfmtrc,
  FormatConfig,
  SortImportsConfig,
  SortPackageJsonConfig,
  SortTailwindcssConfig,
} from "./config.generated";

/**
 * Define an oxfmt configuration with type inference.
 */
export function defineConfig<T extends Oxfmtrc>(config: T): T {
  return config;
}

/**
 * Configuration options for the `format()` API.
 *
 * Based on `FormatConfig` generated from the JSON Schema,
 * with additional deprecated aliases for backward compatibility.
 */
export type FormatOptions = FormatConfig & {
  /** @deprecated Use `sortImports` instead. */
  experimentalSortImports?: SortImportsConfig;
  /** @deprecated Use `sortPackageJson` instead. */
  experimentalSortPackageJson?: boolean | SortPackageJsonConfig;
  /** @deprecated Use `sortTailwindcss` instead. */
  experimentalTailwindcss?: SortTailwindcssConfig;
};

// Backward-compatible type aliases using `Options` suffix.
export type SortImportsOptions = SortImportsConfig;
export type SortPackageJsonOptions = SortPackageJsonConfig;
export type SortTailwindcssOptions = SortTailwindcssConfig;
/** @deprecated Use `SortTailwindcssOptions` instead. */
export type TailwindcssOptions = SortTailwindcssConfig;

/**
 * Format the given source text according to the specified options.
 */
export async function format(fileName: string, sourceText: string, options?: FormatOptions) {
  if (typeof fileName !== "string") throw new TypeError("`fileName` must be a string");
  if (typeof sourceText !== "string") throw new TypeError("`sourceText` must be a string");

  // Capture plugins from options so `resolvePlugins` can use them without needing
  // a config file (the NAPI path receives options directly, not via .oxfmtrc).
  const plugins = (options?.plugins as string[] | undefined) ?? [];

  return napiFormat(
    fileName,
    sourceText,
    options ?? {},
    (numThreads: number) => resolvePlugins(numThreads, plugins),
    (options, code) => formatFile({ options, code }),
    (options, code) => formatEmbeddedCode({ options, code }),
    (options, texts) => formatEmbeddedDoc({ options, texts }),
    (options, classes) => sortTailwindClasses({ options, classes }),
  );
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
    (numThreads: number) => resolvePlugins(numThreads, []),
    (_options, _code) => Promise.reject(/* Unreachable */),
    (options, code) => formatEmbeddedCode({ options, code }),
    (options, texts) => formatEmbeddedDoc({ options, texts }),
    (options, classes) => sortTailwindClasses({ options, classes }),
  );
}
