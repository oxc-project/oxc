/*
 * Language plugin API for embedded frameworks (Vue, Svelte, Angular, …).
 *
 * Based on RFC: https://github.com/oxc-project/oxc/discussions/21936
 * Implementation plan: https://github.com/oxc-project/oxc/issues/23207
 */

/**
 * Offset mapping between original framework source and virtual JS/TS source.
 *
 * Follows Volar-style mappings: each entry connects a range in the virtual
 * source to a range in the original file.
 */
export interface Mapping {
  virtualStart: number;
  virtualEnd: number;
  originalStart: number;
  originalEnd: number;
}

export type Mappings = Mapping[];

/**
 * Virtual JS/TS source produced by a language plugin for Rust rules and typed tooling.
 *
 * Note: `scriptKind` currently accepts `"ts" | "tsx"` in the RFC; `"js" | "jsx"`
 * support is expected to land with follow-up work.
 */
export interface TransformResult {
  sourceText: string;
  scriptKind: "js" | "jsx" | "ts" | "tsx";
  mappings: Mappings;
}

/**
 * Opaque language AST node. Framework plugins typically extend ESTree.
 */
export interface LanguageNode {
  type: string;
  [key: string]: unknown;
}

export interface LanguageToken {
  type: string;
  value: string;
  range?: [number, number];
  loc?: unknown;
  [key: string]: unknown;
}

/**
 * Result of `LanguagePlugin.parse`.
 */
export interface LanguageParseResult {
  ast: LanguageNode;
  tokens?: LanguageToken[];
  transform?: TransformResult | null;
}

/**
 * File loaded by a language plugin, consumed by JS rules and (optionally) Rust/typed tooling.
 */
export interface LoadedLanguageFile {
  languageId: string;
  ast: LanguageNode;
  tokens?: LanguageToken[];
  transform: TransformResult | null;
  isESTree: boolean;
  parserServices?: Record<string, unknown>;
}

/**
 * Static visitor-key schema for a language AST.
 *
 * Declares which fields are children and what node / union types they contain,
 * so Oxlint can generate specialized walkers.
 */
export interface VisitorKeySchema {
  nodes: Record<string, Record<string, string | string[]>>;
  unions?: Record<string, string[]>;
}

export interface LanguagePluginMeta {
  name: string;
}

/**
 * Options passed to `parse` / `load`.
 */
export type LanguagePluginOptions = Record<string, unknown>;

/**
 * Language plugin contract (RFC #21936).
 *
 * - `defaultFiles`: extension / filename defaults (e.g. `[".vue"]`), not unrestricted globs.
 *   Project config owns final file selection via `languagePlugins[].pattern`.
 * - `parse`: produce native AST (+ optional transform). Intended to be cacheable.
 * - `load`: attach parser services / scoping that may be harder to cache.
 */
export interface LanguagePlugin {
  meta: LanguagePluginMeta;
  /**
   * Convenience defaults for which files this language owns.
   * Prefer extensions / filenames such as `".vue"` or `".prettierrc"`.
   */
  defaultFiles?: string[];
  visitorKeys: VisitorKeySchema;
  parse: (
    filePath: string,
    sourceText: string,
    options: LanguagePluginOptions,
  ) => LanguageParseResult | Promise<LanguageParseResult>;
  load: (
    filePath: string,
    parseResult: LanguageParseResult,
    sourceText: string,
    options: LanguagePluginOptions,
  ) => LoadedLanguageFile | Promise<LoadedLanguageFile>;
}

/**
 * Define a language plugin.
 *
 * No-op at runtime beyond light validation; provides type safety for authors.
 *
 * @param plugin - Language plugin
 * @returns Same plugin
 */
export function defineLanguagePlugin<T extends LanguagePlugin>(plugin: T): T {
  if (plugin == null || typeof plugin !== "object") {
    throw new TypeError("`defineLanguagePlugin` expects a plugin object");
  }
  if (plugin.meta == null || typeof plugin.meta.name !== "string" || plugin.meta.name.length === 0) {
    throw new TypeError("Language plugin must define `meta.name` as a non-empty string");
  }
  if (plugin.visitorKeys == null || typeof plugin.visitorKeys !== "object") {
    throw new TypeError("Language plugin must define `visitorKeys`");
  }
  if (typeof plugin.parse !== "function") {
    throw new TypeError("Language plugin must define a `parse` function");
  }
  if (typeof plugin.load !== "function") {
    throw new TypeError("Language plugin must define a `load` function");
  }
  if (plugin.defaultFiles != null) {
    if (!Array.isArray(plugin.defaultFiles) || !plugin.defaultFiles.every((f) => typeof f === "string")) {
      throw new TypeError("`defaultFiles` must be an array of strings (extensions or filenames)");
    }
    for (const file of plugin.defaultFiles) {
      if (file.includes("/") || file.includes("*") || file.includes("?")) {
        throw new TypeError(
          `Language plugin \`defaultFiles\` should be extensions or filenames (e.g. ".vue"), not globs. Received: ${JSON.stringify(file)}`,
        );
      }
    }
  }
  return plugin;
}
