// napi-JS `oxfmt` API entry point

import {
  formatFile,
  formatEmbeddedCode,
  formatEmbeddedDoc,
  sortTailwindClasses,
} from "./libs/apis";
import { toFormatFileResult, toNullable } from "./libs/napi-callbacks";
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
export function defineConfig<T extends OxfmtConfig>(config: T & OxfmtConfig): T {
  return config;
}

// NOTE: Native bindings are loaded lazily on first `format()`/`jsTextToDoc()` call,
// instead of via a static `import "./bindings"`.
//
// A static import would run `requireNative()` which `dlopen`s the native `.node` addon.
// That is wasteful (and sometimes harmful),
// because the two paths that import this entry don't always need the binding:
// 1. Config loading: config files do `import { defineConfig } from "oxfmt"`,
//    where `defineConfig` is a plain identity function needing no native code.
//    Normally the binding is already cached there (the CLI's own copy),
//    so an eager load is just a harmless cache hit.
//    But when a nested config resolves "oxfmt" to a separate install (its own `node_modules`),
//    it triggers a fresh re-entrant `dlopen` on the main thread, which hangs (observed on WSL2).
//    See https://github.com/oxc-project/oxc/issues/23125
// 2. `jsTextToDoc` (via prettier-plugin-oxfmt, runs in the worker process where the binding is NOT preloaded):
//    deferring the load until it's actually called avoids paying the `dlopen` cost on runs that have no embedded code to format.
let BINDINGS_CACHE = null as typeof import("./bindings") | null;

/**
 * Format the given source text according to the specified options.
 */
export async function format(fileName: string, sourceText: string, options?: FormatConfig) {
  if (typeof fileName !== "string") throw new TypeError("`fileName` must be a string");
  if (typeof sourceText !== "string") throw new TypeError("`sourceText` must be a string");

  BINDINGS_CACHE ??= await import("./bindings");
  return BINDINGS_CACHE.format(
    fileName,
    sourceText,
    options ?? {},
    (options, code) => toFormatFileResult(formatFile({ options, code })),
    (options, code) => toNullable(formatEmbeddedCode({ options, code })),
    (options, texts) => toNullable(formatEmbeddedDoc({ options, texts })),
    (options, classes) => toNullable(sortTailwindClasses({ options, classes })),
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
  BINDINGS_CACHE ??= await import("./bindings");
  return BINDINGS_CACHE.jsTextToDoc(
    sourceExt,
    sourceText,
    oxfmtPluginOptionsJson,
    parentContext,
    () => toFormatFileResult(Promise.reject("formatFile is unavailable for jsTextToDoc")),
    (options, code) => toNullable(formatEmbeddedCode({ options, code })),
    (options, texts) => toNullable(formatEmbeddedDoc({ options, texts })),
    (options, classes) => toNullable(sortTailwindClasses({ options, classes })),
  );
}
