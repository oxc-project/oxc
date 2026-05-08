/*
 * This file is generated from npm/oxfmt/configuration_schema.json.
 * Run `just formatter-config-ts` to regenerate.
 */

export type ArrowParensConfig = "always" | "avoid";
export type EmbeddedLanguageFormattingConfig = "auto" | "off";
export type EndOfLineConfig = "lf" | "crlf" | "cr";
export type HtmlWhitespaceSensitivityConfig = "css" | "strict" | "ignore";
export type JsdocUserConfig = boolean | JsdocConfig;
export type ObjectWrapConfig = "preserve" | "collapse";
/**
 * A set of glob patterns.
 * Patterns are matched against paths relative to the configuration file's directory.
 */
export type GlobSet = string[];
export type ProseWrapConfig = "always" | "never" | "preserve";
export type QuotePropsConfig = "as-needed" | "consistent" | "preserve";
export type SortImportsUserConfig = boolean | SortImportsConfig;
export type SortGroupItemConfig = NewlinesBetweenMarker | string | string[];
export type SortOrderConfig = "asc" | "desc";
export type SortPackageJsonUserConfig = boolean | SortPackageJsonConfig;
export type SortTailwindcssUserConfig = boolean | SortTailwindcssConfig;
export type TrailingCommaConfig = "all" | "es5" | "none";

/**
 * Configuration options for the Oxfmt.
 *
 * Most options are the same as Prettier's options, but not all of them.
 * In addition, some options are our own extensions.
 */
export interface Oxfmtrc {
  /**
   * Include parentheses around a sole arrow function parameter.
   *
   * - Default: `"always"`
   */
  arrowParens?: ArrowParensConfig;
  /**
   * Put the `>` of a multi-line HTML (HTML, JSX, Vue, Angular) element at the end of the last line,
   * instead of being alone on the next line (does not apply to self closing elements).
   *
   * - Default: `false`
   */
  bracketSameLine?: boolean;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Default: `true`
   */
  bracketSpacing?: boolean;
  /**
   * Control whether to format embedded parts (For example, CSS-in-JS, or JS-in-Vue, etc.) in the file.
   *
   * NOTE: XXX-in-JS support is incomplete.
   *
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig;
  /**
   * Specify the global whitespace sensitivity for HTML, Vue, Angular, and Handlebars.
   *
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig;
  /**
   * Ignore files matching these glob patterns.
   * Patterns are based on the location of the Oxfmt configuration file.
   *
   * - Default: `[]`
   */
  ignorePatterns?: string[];
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean;
  /**
   * Enable JSDoc comment formatting.
   *
   * When enabled, JSDoc comments are normalized and reformatted:
   * tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  jsdoc?: JsdocUserConfig;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Default: `false`
   */
  jsxSingleQuote?: boolean;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Default: `"preserve"`
   */
  objectWrap?: ObjectWrapConfig;
  /**
   * File-specific overrides.
   * When a file matches multiple overrides, the later override takes precedence (array order matters).
   *
   * - Default: `[]`
   */
  overrides?: OxfmtOverrideConfig[];
  /**
   * Specify the line length that the printer will wrap on.
   *
   * If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
   *
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number;
  /**
   * How to wrap prose.
   *
   * By default, formatter will not change wrapping in markdown text since some services use a linebreak-sensitive renderer, e.g. GitHub comments and BitBucket.
   * To wrap prose to the print width, change this option to "always".
   * If you want to force all prose blocks to be on a single line and rely on editor/viewer soft wrapping instead, you can use "never".
   *
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig;
  /**
   * Change when properties in objects are quoted.
   *
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Default: `true`
   */
  semi?: boolean;
  /**
   * Enforce single attribute per line in HTML, Vue, and JSX.
   *
   * - Default: `false`
   */
  singleAttributePerLine?: boolean;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.quote_type`
   */
  singleQuote?: boolean;
  /**
   * Sort import statements.
   *
   * Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   * For details, see each field's documentation.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  sortImports?: SortImportsUserConfig;
  /**
   * Sort `package.json` keys.
   *
   * The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
   * But we believe it is clearer and easier to navigate.
   * For details, see each field's documentation.
   *
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig;
  /**
   * Sort Tailwind CSS classes.
   *
   * Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
   * For details, see each field's documentation.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssUserConfig;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size` (falls back to `.editorconfig.tab_width`)
   */
  tabWidth?: number;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean;
  [k: string]: unknown;
}
export interface JsdocConfig {
  /**
   * Append default values to `@param` descriptions (e.g. "Default is `value`").
   *
   * - Default: `true`
   */
  addDefaultToDescription?: boolean;
  /**
   * Add spaces inside JSDoc type braces: `{string}` → `{ string }`.
   *
   * - Default: `false`
   */
  bracketSpacing?: boolean;
  /**
   * Capitalize the first letter of tag descriptions.
   *
   * - Default: `true`
   */
  capitalizeDescriptions?: boolean;
  /**
   * How to format comment blocks.
   *
   * - `"singleLine"` — Convert to single-line `/** content * /` when possible.
   * - `"multiline"` — Always use multi-line format.
   * - `"keep"` — Preserve original formatting.
   *
   * - Default: `"singleLine"`
   */
  commentLineStrategy?: string;
  /**
   * Emit `@description` tag instead of inline description.
   *
   * - Default: `false`
   */
  descriptionTag?: boolean;
  /**
   * Add a trailing dot to the end of descriptions.
   *
   * - Default: `false`
   */
  descriptionWithDot?: boolean;
  /**
   * Preserve indentation in unparsable `@example` code.
   *
   * - Default: `false`
   */
  keepUnparsableExampleIndent?: boolean;
  /**
   * Strategy for wrapping description lines at print width.
   *
   * - `"greedy"` — Always re-wrap text to fit within print width.
   * - `"balance"` — Preserve original line breaks if all lines fit within print width.
   *
   * - Default: `"greedy"`
   */
  lineWrappingStyle?: string;
  /**
   * Use fenced code blocks (```` ``` ````) instead of 4-space indentation for code without a language tag.
   *
   * - Default: `false`
   */
  preferCodeFences?: boolean;
  /**
   * Add a blank line between the last `@param` and `@returns`.
   *
   * - Default: `false`
   */
  separateReturnsFromParam?: boolean;
  /**
   * Add blank lines between different tag groups (e.g. between `@param` and `@returns`).
   *
   * - Default: `false`
   */
  separateTagGroups?: boolean;
  [k: string]: unknown;
}
export interface OxfmtOverrideConfig {
  /**
   * Glob patterns to exclude from this override.
   */
  excludeFiles?: GlobSet;
  /**
   * Glob patterns to match files for this override.
   */
  files: GlobSet;
  /**
   * Format options to apply for matched files.
   */
  options?: FormatConfig;
  [k: string]: unknown;
}
export interface FormatConfig {
  /**
   * Include parentheses around a sole arrow function parameter.
   *
   * - Default: `"always"`
   */
  arrowParens?: ArrowParensConfig;
  /**
   * Put the `>` of a multi-line HTML (HTML, JSX, Vue, Angular) element at the end of the last line,
   * instead of being alone on the next line (does not apply to self closing elements).
   *
   * - Default: `false`
   */
  bracketSameLine?: boolean;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Default: `true`
   */
  bracketSpacing?: boolean;
  /**
   * Control whether to format embedded parts (For example, CSS-in-JS, or JS-in-Vue, etc.) in the file.
   *
   * NOTE: XXX-in-JS support is incomplete.
   *
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig;
  /**
   * Specify the global whitespace sensitivity for HTML, Vue, Angular, and Handlebars.
   *
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig;
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean;
  /**
   * Enable JSDoc comment formatting.
   *
   * When enabled, JSDoc comments are normalized and reformatted:
   * tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  jsdoc?: JsdocUserConfig;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Default: `false`
   */
  jsxSingleQuote?: boolean;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Default: `"preserve"`
   */
  objectWrap?: ObjectWrapConfig;
  /**
   * Specify the line length that the printer will wrap on.
   *
   * If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
   *
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number;
  /**
   * How to wrap prose.
   *
   * By default, formatter will not change wrapping in markdown text since some services use a linebreak-sensitive renderer, e.g. GitHub comments and BitBucket.
   * To wrap prose to the print width, change this option to "always".
   * If you want to force all prose blocks to be on a single line and rely on editor/viewer soft wrapping instead, you can use "never".
   *
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig;
  /**
   * Change when properties in objects are quoted.
   *
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Default: `true`
   */
  semi?: boolean;
  /**
   * Enforce single attribute per line in HTML, Vue, and JSX.
   *
   * - Default: `false`
   */
  singleAttributePerLine?: boolean;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.quote_type`
   */
  singleQuote?: boolean;
  /**
   * Sort import statements.
   *
   * Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   * For details, see each field's documentation.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  sortImports?: SortImportsUserConfig;
  /**
   * Sort `package.json` keys.
   *
   * The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
   * But we believe it is clearer and easier to navigate.
   * For details, see each field's documentation.
   *
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig;
  /**
   * Sort Tailwind CSS classes.
   *
   * Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
   * For details, see each field's documentation.
   *
   * Pass `true` or an object to enable with defaults, or omit/set `false` to disable.
   *
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssUserConfig;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size` (falls back to `.editorconfig.tab_width`)
   */
  tabWidth?: number;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean;
  [k: string]: unknown;
}
export interface SortImportsConfig {
  /**
   * Define your own groups for matching very specific imports.
   *
   * The `customGroups` list is ordered: The first definition that matches an element will be used.
   * Custom groups have a higher priority than any predefined group.
   *
   * If you want a predefined group to take precedence over a custom group,
   * you must write a custom group definition that does the same as what the predefined group does, and put it first in the list.
   *
   * If you specify multiple conditions like `elementNamePattern`, `selector`, and `modifiers`,
   * all conditions must be met for an import to match the custom group (AND logic).
   *
   * - Default: `[]`
   */
  customGroups?: CustomGroupItemConfig[];
  /**
   * Specifies a list of predefined import groups for sorting.
   *
   * Each import will be assigned a single group specified in the groups option (or the `unknown` group if no match is found).
   * The order of items in the `groups` option determines how groups are ordered.
   *
   * Within a given group, members will be sorted according to the type, order, ignoreCase, etc. options.
   *
   * Individual groups can be combined together by placing them in an array.
   * The order of groups in that array does not matter.
   * All members of the groups in the array will be sorted together as if they were part of a single group.
   *
   * Predefined groups are characterized by a single selector and potentially multiple modifiers.
   * You may enter modifiers in any order, but the selector must always come at the end.
   *
   * The list of selectors is sorted from most to least important:
   * - `type` — TypeScript type imports.
   * - `side_effect_style` — Side effect style imports.
   * - `side_effect` — Side effect imports.
   * - `style` — Style imports.
   * - `index` — Main file from the current directory.
   * - `sibling` — Modules from the same directory.
   * - `parent` — Modules from the parent directory.
   * - `subpath` — Node.js subpath imports.
   * - `internal` — Your internal modules.
   * - `builtin` — Node.js Built-in Modules.
   * - `external` — External modules installed in the project.
   * - `import` — Any import.
   *
   * The list of modifiers is sorted from most to least important:
   * - `side_effect` — Side effect imports.
   * - `type` — TypeScript type imports.
   * - `value` — Value imports.
   * - `default` — Imports containing the default specifier.
   * - `wildcard` — Imports containing the wildcard (`* as`) specifier.
   * - `named` — Imports containing at least one named specifier.
   *
   * - Default: See below
   * ```json
   * [
   * "builtin",
   * "external",
   * ["internal", "subpath"],
   * ["parent", "sibling", "index"],
   * "style",
   * "unknown"
   * ]
   * ```
   *
   * Also, you can override the global `newlinesBetween` setting for specific group boundaries
   * by including a `{ "newlinesBetween": boolean }` marker object in the `groups` list at the desired position.
   */
  groups?: SortGroupItemConfig[];
  /**
   * Specifies whether sorting should be case-sensitive.
   *
   * - Default: `true`
   */
  ignoreCase?: boolean;
  /**
   * Specifies a prefix for identifying internal imports.
   *
   * This is useful for distinguishing your own modules from external dependencies.
   *
   * - Default: `["~/", "@/"]`
   */
  internalPattern?: string[];
  /**
   * Specifies whether to add newlines between groups.
   *
   * When `false`, no newlines are added between groups.
   *
   * - Default: `true`
   */
  newlinesBetween?: boolean;
  /**
   * Specifies whether to sort items in ascending or descending order.
   *
   * - Default: `"asc"`
   */
  order?: SortOrderConfig;
  /**
   * Enables the use of comments to separate imports into logical groups.
   *
   * When `true`, all comments will be treated as delimiters, creating partitions.
   *
   * ```js
   * import { b1, b2 } from 'b'
   * // PARTITION
   * import { a } from 'a'
   * import { c } from 'c'
   * ```
   *
   * - Default: `false`
   */
  partitionByComment?: boolean;
  /**
   * Enables the empty line to separate imports into logical groups.
   *
   * When `true`, formatter will not sort imports if there is an empty line between them.
   * This helps maintain the defined order of logically separated groups of members.
   *
   * ```js
   * import { b1, b2 } from 'b'
   *
   * import { a } from 'a'
   * import { c } from 'c'
   * ```
   *
   * - Default: `false`
   */
  partitionByNewline?: boolean;
  /**
   * Specifies whether side effect imports should be sorted.
   *
   * By default, sorting side-effect imports is disabled for security reasons.
   *
   * - Default: `false`
   */
  sortSideEffects?: boolean;
  [k: string]: unknown;
}
export interface CustomGroupItemConfig {
  /**
   * List of glob patterns to match import sources for this group.
   */
  elementNamePattern?: string[];
  /**
   * Name of the custom group, used in the `groups` option.
   */
  groupName?: string;
  /**
   * Modifiers to match the import characteristics.
   * All specified modifiers must be present (AND logic).
   *
   * Possible values: `"side_effect"`, `"type"`, `"value"`, `"default"`, `"wildcard"`, `"named"`
   */
  modifiers?: string[];
  /**
   * Selector to match the import kind.
   *
   * Possible values: `"type"`, `"side_effect_style"`, `"side_effect"`, `"style"`, `"index"`,
   * `"sibling"`, `"parent"`, `"subpath"`, `"internal"`, `"builtin"`, `"external"`, `"import"`
   */
  selector?: string;
  [k: string]: unknown;
}
/**
 * A marker object for overriding `newlinesBetween` at a specific group boundary.
 */
export interface NewlinesBetweenMarker {
  newlinesBetween: boolean;
  [k: string]: unknown;
}
export interface SortPackageJsonConfig {
  /**
   * Sort the `scripts` field alphabetically.
   *
   * - Default: `false`
   */
  sortScripts?: boolean;
  [k: string]: unknown;
}
export interface SortTailwindcssConfig {
  /**
   * List of additional attributes to sort beyond `class` and `className` (exact match).
   *
   * NOTE: Regex patterns are not yet supported.
   *
   * - Default: `[]`
   * - Example: `["myClassProp", ":class"]`
   */
  attributes?: string[];
  /**
   * Path to your Tailwind CSS configuration file (v3).
   *
   * NOTE: Paths are resolved relative to the Oxfmt configuration file.
   *
   * - Default: Automatically find `"tailwind.config.js"`
   */
  config?: string;
  /**
   * List of custom function names whose arguments should be sorted (exact match).
   *
   * NOTE: Regex patterns are not yet supported.
   *
   * - Default: `[]`
   * - Example: `["clsx", "cn", "cva", "tw"]`
   */
  functions?: string[];
  /**
   * Preserve duplicate classes.
   *
   * - Default: `false`
   */
  preserveDuplicates?: boolean;
  /**
   * Preserve whitespace around classes.
   *
   * - Default: `false`
   */
  preserveWhitespace?: boolean;
  /**
   * Path to your Tailwind CSS stylesheet (v4).
   *
   * NOTE: Paths are resolved relative to the Oxfmt configuration file.
   *
   * - Default: Installed Tailwind CSS's `theme.css`
   */
  stylesheet?: string;
  [k: string]: unknown;
}
