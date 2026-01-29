import { format as napiFormat } from "./bindings";
import {
  resolvePlugins,
  formatEmbeddedCode,
  formatFile,
  sortTailwindClasses,
} from "./libs/prettier";
import type { Options } from "prettier";

// napi-JS `oxfmt` API entry point
// See also `format()` function in `./src/main_napi.rs`

/**
 * Format the given source text according to the specified options.
 */
export async function format(fileName: string, sourceText: string, options?: FormatOptions) {
  if (typeof fileName !== "string") throw new TypeError("`fileName` must be a string");
  if (typeof sourceText !== "string") throw new TypeError("`sourceText` must be a string");

  return napiFormat(
    fileName,
    sourceText,
    options ?? {},
    resolvePlugins,
    (options, parserName, code) => formatEmbeddedCode({ options, parserName, code }),
    (options, parserName, fileName, code) => formatFile({ options, parserName, fileName, code }),
    (filepath, options, classes) => sortTailwindClasses({ filepath, classes, options }),
  );
}

// NOTE: Regarding the handwritten TypeScript types.
//
// Initially, I tried to use the `FormatConfig` struct to automatically generate types with `napi(object)`,
// but since `Oxfmtrc` has many fields defined as `enum`, the API usage would look like this:
//
// ```ts
// oxfmt.format("file.ts", "const a=1;", {
//   endOfLine: oxfmt.EndOfLine.Lf,
//   // ...
// });
// ```
//
// Since it cannot be specified with string literals, the API usability is not good.
//
// Therefore, I decided to just handwrite the TypeScript types.
// There is already a mechanism to generate JSON Schema,
// so it might be possible to generate type definitions from that.
// TODO: in the future.

/**
 * Configuration options for the `format()` API.
 */
export type FormatOptions = Pick<
  Options,
  | "useTabs"
  | "tabWidth"
  | "singleQuote"
  | "jsxSingleQuote"
  | "quoteProps"
  | "trailingComma"
  | "semi"
  | "arrowParens"
  | "bracketSpacing"
  | "bracketSameLine"
  | "objectWrap"
  | "singleAttributePerLine"
  | "embeddedLanguageFormatting"
  | "proseWrap"
  | "htmlWhitespaceSensitivity"
  | "vueIndentScriptAndStyle"
> & {
  /** Which end of line characters to apply. (Default: `"lf"`) */
  endOfLine?: "lf" | "crlf" | "cr";
  /** The line length that the printer will wrap on. (Default: `100`) */
  printWidth?: number;
  /** Whether to insert a final newline at the end of the file. (Default: `true`) */
  insertFinalNewline?: boolean;
  /** Experimental: Sort import statements. Disabled by default. */
  experimentalSortImports?: SortImportsOptions;
  /** Experimental: Sort `package.json` keys. (Default: `true`) */
  experimentalSortPackageJson?: boolean;
  /**
   * Experimental: Enable Tailwind CSS class sorting in JSX class/className attributes.
   * (Default: disabled)
   */
  experimentalTailwindcss?: TailwindcssOptions;
} & Record<string, unknown>; // Also allow additional options for we don't have typed yet.

/**
 * Configuration options for sort imports.
 */
export type SortImportsOptions = {
  /** Partition imports by newlines. (Default: `false`) */
  partitionByNewline?: boolean;
  /** Partition imports by comments. (Default: `false`) */
  partitionByComment?: boolean;
  /** Sort side-effect imports. (Default: `false`) */
  sortSideEffects?: boolean;
  /** Sort order. (Default: `"asc"`) */
  order?: "asc" | "desc";
  /** Ignore case when sorting. (Default: `true`) */
  ignoreCase?: boolean;
  /** Add newlines between import groups. (Default: `true`) */
  newlinesBetween?: boolean;
  /** Prefixes to identify internal imports. (Default: `["~/", "@/"]`) */
  internalPattern?: string[];
  /**
   * Groups configuration for organizing imports.
   * Each array element represents a group, and multiple group names in the same array are treated as one.
   * Accepts both `string` and `string[]` as group elements.
   */
  groups?: (string | string[])[];
  /** Define custom groups for matching specific imports. */
  customGroups?: { groupName: string; elementNamePattern: string[] }[];
};

/**
 * Configuration options for Tailwind CSS class sorting.
 * See https://github.com/tailwindlabs/prettier-plugin-tailwindcss#options
 */
export type TailwindcssOptions = {
  /** Path to Tailwind config file (v3). e.g., `"./tailwind.config.js"` */
  config?: string;
  /** Path to Tailwind stylesheet (v4). e.g., `"./src/app.css"` */
  stylesheet?: string;
  /**
   * List of custom function names whose arguments should be sorted.
   * e.g., `["clsx", "cva", "tw"]` (Default: `[]`)
   */
  functions?: string[];
  /**
   * List of additional HTML/JSX attributes to sort (beyond `class` and `className`).
   * e.g., `["myClassProp", ":class"]` (Default: `[]`)
   */
  attributes?: string[];
  /** Preserve whitespace around classes. (Default: `false`) */
  preserveWhitespace?: boolean;
  /** Preserve duplicate classes. (Default: `false`) */
  preserveDuplicates?: boolean;
};
