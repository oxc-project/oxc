import type { Program } from "@oxc-project/types";

export interface CommentOptions {
  /**
   * Print normal comments that do not have special meanings.
   *
   * At present only statement level comments are printed.
   *
   * @default true
   */
  normal?: boolean;
  /**
   * Print jsdoc comments (`/** jsdoc *\/`).
   *
   * @default true
   */
  jsdoc?: boolean;
  /**
   * Print annotation comments, e.g. `/* #__PURE__ *\/`, `/* webpackChunkName *\/`,
   * `/* @vite-ignore *\/` and coverage ignore comments.
   *
   * @default true
   */
  annotation?: boolean;
  /**
   * How to handle legal comments (comments containing `@license`, `@preserve`,
   * or starting with `//!` / `/*!`).
   *
   * - `"none"` - Do not preserve any legal comments.
   * - `"inline"` - Preserve all legal comments inline.
   * - `"eof"` - Move all legal comments to the end of the file.
   *
   * @default "inline"
   */
  legal?: "none" | "inline" | "eof";
}

export interface PrintOptions {
  /**
   * Original source text the AST was parsed from.
   *
   * Required for printing comments (oxc comments reference source spans) and for
   * accurate source maps. When omitted, comments are not printed.
   */
  sourceText?: string;
  /**
   * Source filename, used as the `source` field of the source map.
   *
   * @default "unknown"
   */
  filename?: string;
  /**
   * Produce a source map, returned as `map` on the result.
   *
   * @default false
   */
  sourcemap?: boolean;
  /**
   * Use single quotes instead of double quotes.
   *
   * @default false
   */
  singleQuote?: boolean;
  /**
   * Remove whitespace (minified output). Comments are removed unless `comments`
   * is set explicitly.
   *
   * @default false
   */
  minify?: boolean;
  /**
   * Print comments. Requires `sourceText`.
   *
   * `false` disables all comments, `true` or an object enables them selectively.
   *
   * @default true (when `sourceText` is provided and not minifying)
   */
  comments?: boolean | CommentOptions;
  /**
   * Indentation character.
   *
   * @default "tab"
   */
  indentChar?: "space" | "tab";
  /**
   * Number of characters per indentation level.
   *
   * @default 1
   */
  indentWidth?: number;
  /**
   * Initial indentation level for the generated code.
   *
   * @default 0
   */
  initialIndent?: number;
}

export interface SourceMap {
  file?: string;
  mappings: string;
  names: Array<string>;
  sourceRoot?: string;
  sources: Array<string>;
  sourcesContent?: Array<string>;
  version: number;
  x_google_ignoreList?: Array<number>;
}

export interface OxcError {
  severity: "Error" | "Warning" | "Advice";
  message: string;
  labels: Array<{ message?: string; start: number; end: number }>;
  helpMessage?: string;
  codeframe?: string;
}

export interface PrintResult {
  code: string;
  map?: SourceMap;
  errors: Array<OxcError>;
}

/**
 * Print an ESTree AST to JavaScript / TypeScript source code.
 *
 * The AST must be in the shape `oxc-parser` produces (ESTree / TS-ESTree compatible).
 *
 * @example
 * ```js
 * import { parseSync } from "oxc-parser";
 * import { print } from "oxc-codegen";
 *
 * const { program } = parseSync("test.js", "const x = 1 + 2;");
 * print(program).code; // "const x = 1 + 2;\n"
 * ```
 */
export function print(program: Program, options?: PrintOptions | undefined | null): PrintResult;

/**
 * Returns `true` if raw transfer back (and therefore `print`) is supported.
 *
 * Raw transfer back is only supported on 64-bit little-endian systems,
 * and NodeJS >= v22.0.0 or Deno >= v2.0.0.
 */
export function rawTransferBackSupported(): boolean;
