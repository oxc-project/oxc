/*
 * This file is generated from npm/oxfmt/configuration_schema.json.
 * Run `just formatter-config-ts` to regenerate.
 */

export type ArrowParensConfig = "always" | "avoid";
export type EmbeddedLanguageFormattingConfig = "auto" | "off";
export type EndOfLineConfig = "lf" | "crlf" | "cr";
export type HtmlWhitespaceSensitivityConfig = "css" | "strict" | "ignore";
export type ObjectWrapConfig = "preserve" | "collapse";
export type ProseWrapConfig = "always" | "never" | "preserve";
export type QuotePropsConfig = "as-needed" | "consistent" | "preserve";
export type SortGroupItemConfig = NewlinesBetweenMarker | string | string[];
export type SortOrderConfig = "asc" | "desc";
export type SortPackageJsonUserConfig = boolean | SortPackageJsonConfig;
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
  arrowParens?: ArrowParensConfig | null;
  /**
   * Put the `>` of a multi-line HTML (HTML, JSX, Vue, Angular) element at the end of the last line,
   * instead of being alone on the next line (does not apply to self closing elements).
   *
   * - Default: `false`
   */
  bracketSameLine?: boolean | null;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Default: `true`
   */
  bracketSpacing?: boolean | null;
  /**
   * Control whether to format embedded parts (For example, CSS-in-JS, or JS-in-Vue, etc.) in the file.
   *
   * NOTE: XXX-in-JS support is incomplete.
   *
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig | null;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig | null;
  /**
   * Specify the global whitespace sensitivity for HTML, Vue, Angular, and Handlebars.
   *
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig | null;
  /**
   * Ignore files matching these glob patterns.
   * Patterns are based on the location of the Oxfmt configuration file.
   *
   * - Default: `[]`
   */
  ignorePatterns?: string[] | null;
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean | null;
  /**
   * Enable JSDoc comment formatting.
   *
   * When enabled, JSDoc comments are normalized and reformatted:
   * tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Can be `true` (enable with defaults), `false` (disable), or an object with options.
   *
   * - Default: Disabled
   */
  jsdoc?: JsdocConfig | null;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Default: `false`
   */
  jsxSingleQuote?: boolean | null;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Default: `"preserve"`
   */
  objectWrap?: ObjectWrapConfig | null;
  /**
   * File-specific overrides.
   * When a file matches multiple overrides, the later override takes precedence (array order matters).
   *
   * - Default: `[]`
   */
  overrides?: OxfmtOverrideConfig[] | null;
  /**
   * Specify the line length that the printer will wrap on.
   *
   * If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
   *
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number | null;
  /**
   * How to wrap prose.
   *
   * By default, formatter will not change wrapping in markdown text since some services use a linebreak-sensitive renderer, e.g. GitHub comments and BitBucket.
   * To wrap prose to the print width, change this option to "always".
   * If you want to force all prose blocks to be on a single line and rely on editor/viewer soft wrapping instead, you can use "never".
   *
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig | null;
  /**
   * Change when properties in objects are quoted.
   *
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig | null;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Default: `true`
   */
  semi?: boolean | null;
  /**
   * Enforce single attribute per line in HTML, Vue, and JSX.
   *
   * - Default: `false`
   */
  singleAttributePerLine?: boolean | null;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Default: `false`
   */
  singleQuote?: boolean | null;
  /**
   * Sort import statements.
   *
   * Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   * For details, see each field's documentation.
   *
   * - Default: Disabled
   */
  sortImports?: SortImportsConfig | null;
  /**
   * Sort `package.json` keys.
   *
   * The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
   * But we believe it is clearer and easier to navigate.
   * For details, see each field's documentation.
   *
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig | null;
  /**
   * Sort Tailwind CSS classes.
   *
   * Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
   * For details, see each field's documentation.
   *
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssConfig | null;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size`
   */
  tabWidth?: number | null;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig | null;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean | null;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean | null;
  [k: string]: unknown;
}
/**
 * JSDoc configuration: either `true`/`false` or an object with fine-grained options.
 */
export interface JsdocConfig {
  add_default_to_description?: boolean | null;
  bracket_spacing?: boolean | null;
  capitalize_descriptions?: boolean | null;
  comment_line_strategy?: string | null;
  description_tag?: boolean | null;
  description_with_dot?: boolean | null;
  keep_unparsable_example_indent?: boolean | null;
  line_wrapping_style?: string | null;
  prefer_code_fences?: boolean | null;
  separate_returns_from_param?: boolean | null;
  separate_tag_groups?: boolean | null;
  [k: string]: unknown;
}
export interface OxfmtOverrideConfig {
  /**
   * Glob patterns to exclude from this override.
   */
  excludeFiles?: string[] | null;
  /**
   * Glob patterns to match files for this override.
   * All patterns are relative to the Oxfmt configuration file.
   */
  files: string[];
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
  arrowParens?: ArrowParensConfig | null;
  /**
   * Put the `>` of a multi-line HTML (HTML, JSX, Vue, Angular) element at the end of the last line,
   * instead of being alone on the next line (does not apply to self closing elements).
   *
   * - Default: `false`
   */
  bracketSameLine?: boolean | null;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Default: `true`
   */
  bracketSpacing?: boolean | null;
  /**
   * Control whether to format embedded parts (For example, CSS-in-JS, or JS-in-Vue, etc.) in the file.
   *
   * NOTE: XXX-in-JS support is incomplete.
   *
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig | null;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig | null;
  /**
   * Specify the global whitespace sensitivity for HTML, Vue, Angular, and Handlebars.
   *
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig | null;
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean | null;
  /**
   * Enable JSDoc comment formatting.
   *
   * When enabled, JSDoc comments are normalized and reformatted:
   * tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Can be `true` (enable with defaults), `false` (disable), or an object with options.
   *
   * - Default: Disabled
   */
  jsdoc?: JsdocConfig | null;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Default: `false`
   */
  jsxSingleQuote?: boolean | null;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Default: `"preserve"`
   */
  objectWrap?: ObjectWrapConfig | null;
  /**
   * Specify the line length that the printer will wrap on.
   *
   * If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
   *
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number | null;
  /**
   * How to wrap prose.
   *
   * By default, formatter will not change wrapping in markdown text since some services use a linebreak-sensitive renderer, e.g. GitHub comments and BitBucket.
   * To wrap prose to the print width, change this option to "always".
   * If you want to force all prose blocks to be on a single line and rely on editor/viewer soft wrapping instead, you can use "never".
   *
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig | null;
  /**
   * Change when properties in objects are quoted.
   *
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig | null;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Default: `true`
   */
  semi?: boolean | null;
  /**
   * Enforce single attribute per line in HTML, Vue, and JSX.
   *
   * - Default: `false`
   */
  singleAttributePerLine?: boolean | null;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Default: `false`
   */
  singleQuote?: boolean | null;
  /**
   * Sort import statements.
   *
   * Using the similar algorithm as [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   * For details, see each field's documentation.
   *
   * - Default: Disabled
   */
  sortImports?: SortImportsConfig | null;
  /**
   * Sort `package.json` keys.
   *
   * The algorithm is NOT compatible with [prettier-plugin-sort-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson).
   * But we believe it is clearer and easier to navigate.
   * For details, see each field's documentation.
   *
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig | null;
  /**
   * Sort Tailwind CSS classes.
   *
   * Using the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g., `config` instead of `tailwindConfig`).
   * For details, see each field's documentation.
   *
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssConfig | null;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size`
   */
  tabWidth?: number | null;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig | null;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean | null;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean | null;
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
  customGroups?: CustomGroupItemConfig[] | null;
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
  groups?: SortGroupItemConfig[] | null;
  /**
   * Specifies whether sorting should be case-sensitive.
   *
   * - Default: `true`
   */
  ignoreCase?: boolean | null;
  /**
   * Specifies a prefix for identifying internal imports.
   *
   * This is useful for distinguishing your own modules from external dependencies.
   *
   * - Default: `["~/", "@/"]`
   */
  internalPattern?: string[] | null;
  /**
   * Specifies whether to add newlines between groups.
   *
   * When `false`, no newlines are added between groups.
   *
   * - Default: `true`
   */
  newlinesBetween?: boolean | null;
  /**
   * Specifies whether to sort items in ascending or descending order.
   *
   * - Default: `"asc"`
   */
  order?: SortOrderConfig | null;
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
  partitionByComment?: boolean | null;
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
  partitionByNewline?: boolean | null;
  /**
   * Specifies whether side effect imports should be sorted.
   *
   * By default, sorting side-effect imports is disabled for security reasons.
   *
   * - Default: `false`
   */
  sortSideEffects?: boolean | null;
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
  modifiers?: string[] | null;
  /**
   * Selector to match the import kind.
   *
   * Possible values: `"type"`, `"side_effect_style"`, `"side_effect"`, `"style"`, `"index"`,
   * `"sibling"`, `"parent"`, `"subpath"`, `"internal"`, `"builtin"`, `"external"`, `"import"`
   */
  selector?: string | null;
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
  sortScripts?: boolean | null;
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
  attributes?: string[] | null;
  /**
   * Path to your Tailwind CSS configuration file (v3).
   *
   * NOTE: Paths are resolved relative to the Oxfmt configuration file.
   *
   * - Default: Automatically find `"tailwind.config.js"`
   */
  config?: string | null;
  /**
   * List of custom function names whose arguments should be sorted (exact match).
   *
   * NOTE: Regex patterns are not yet supported.
   *
   * - Default: `[]`
   * - Example: `["clsx", "cn", "cva", "tw"]`
   */
  functions?: string[] | null;
  /**
   * Preserve duplicate classes.
   *
   * - Default: `false`
   */
  preserveDuplicates?: boolean | null;
  /**
   * Preserve whitespace around classes.
   *
   * - Default: `false`
   */
  preserveWhitespace?: boolean | null;
  /**
   * Path to your Tailwind CSS stylesheet (v4).
   *
   * NOTE: Paths are resolved relative to the Oxfmt configuration file.
   *
   * - Default: Installed Tailwind CSS's `theme.css`
   */
  stylesheet?: string | null;
  [k: string]: unknown;
}
