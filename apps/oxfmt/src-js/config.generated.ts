/*
 * This file is generated from npm/oxfmt/configuration_schema.json.
 * Run `just formatter-config-ts` to regenerate.
 */

export type ArrowParensConfig = "always" | "avoid";
export type AstroUserConfig = boolean | AstroConfig;
export type AstroCompressHtmlConfig = "jsx" | "html" | "none";
export type EmbeddedLanguageFormattingConfig = "auto" | "off";
export type EndOfLineConfig = "lf" | "crlf" | "cr";
export type OperatorPositionConfig = "start" | "end";
export type HtmlWhitespaceSensitivityConfig = "css" | "strict" | "ignore";
export type JsdocUserConfig = boolean | JsdocConfig;
export type CommentLineStrategyConfig = "singleLine" | "multiline" | "keep";
export type LineWrappingStyleConfig = "greedy" | "balance";
export type ObjectWrapConfig = "preserve" | "collapse";
/**
 * A set of glob patterns.
 * Patterns are matched against paths relative to the configuration file's directory.
 */
export type GlobSet = string[];
export type ProseWrapConfig = "always" | "never" | "preserve";
export type QuotePropsConfig = "as-needed" | "consistent" | "preserve";
export type SortImportsUserConfig = boolean | SortImportsConfig;
/**
 * Modifier matching the import characteristics in `customGroups` (see `sortImports.groups` for semantics).
 */
export type ImportModifierConfig = "side_effect" | "type" | "value" | "default" | "wildcard" | "named";
/**
 * Selector matching the import kind in `customGroups` (see `sortImports.groups` for semantics).
 */
export type ImportSelectorConfig =
  | "type"
  | "side_effect_style"
  | "side_effect"
  | "style"
  | "index"
  | "sibling"
  | "parent"
  | "subpath"
  | "internal"
  | "builtin"
  | "external"
  | "import";
export type SortGroupItemConfig = NewlinesBetweenMarker | string | string[];
export type SortOrderConfig = "asc" | "desc";
export type SortPackageJsonUserConfig = boolean | SortPackageJsonConfig;
export type SortTailwindcssUserConfig = boolean | SortTailwindcssConfig;
export type SvelteUserConfig = boolean | SvelteConfig;
export type TrailingCommaConfig = "all" | "es5" | "none";

/**
 * Configuration options for Oxfmt.
 *
 * Most options are the same as Prettier's options, but not all of them.
 * In addition, some options are our own extensions.
 */
export interface Oxfmtrc {
  /**
   * Include parentheses around a sole arrow function parameter.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"always"`
   */
  arrowParens?: ArrowParensConfig;
  /**
   * Options for `prettier-plugin-astro`.
   *
   * Pass `true` or an object to enable `.astro` file formatting, `false` to disable (handy in overrides).
   * Setting `true` resets to defaults, dropping options inherited from a parent scope.
   *
   * NOTE: `prettier-plugin-astro` requires the `@astrojs/compiler-rs` package at runtime.
   * Oxfmt does NOT bundle it, so install it in your project, otherwise formatting fails.
   *
   * - Languages: Astro
   * - Default: Disabled
   */
  astro?: AstroUserConfig;
  /**
   * Put the `>` of a multi-line element at the end of the last line,
   * instead of being alone on the next line (does not apply to self-closing elements).
   *
   * - Languages: JSX, TSX, HTML, Angular, Vue, MJML, Svelte, Astro
   * - Default: `false`
   */
  bracketSameLine?: boolean;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Languages: JS, JSX, TS, TSX, JSON, JSONC, JSON5, GraphQL, YAML
   * - Default: `true`
   */
  bracketSpacing?: boolean;
  /**
   * Control whether to format embedded parts in the file.
   * For example:
   * - CSS-in-JS: template literal
   * - JS-in-Vue: `<script>` block
   * - JS-in-Markdown: code fence
   * - YAML-in-CSS/Markdown: front matter
   *
   * With `"off"`, these parts are kept as-is.
   *
   * NOTE: Some languages behave differently.
   * For Svelte, formatting fails with `"off"`, a limitation of `prettier-plugin-svelte`.
   * For Astro, the frontmatter is still formatted, but by Prettier instead of Oxfmt (use `astro.skipFrontmatter` to keep it as-is).
   *
   * - Languages: JS, JSX, TS, TSX, CSS, SCSS, Less, HTML, Vue, Angular, Svelte, Astro, Markdown, MDX
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Languages: All
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig;
  /**
   * When expressions wrap lines, print operators at the start of new lines (`"start"`)
   * or at the end of previous lines (`"end"`).
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"end"`
   */
  experimentalOperatorPosition?: OperatorPositionConfig;
  /**
   * Specify the global whitespace sensitivity.
   *
   * - Languages: HTML, Angular, Vue, Handlebars, Svelte
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig;
  /**
   * Ignore files matching these glob patterns.
   * Patterns use gitignore-style matching, rooted at the directory containing the configuration file.
   * Files outside that directory cannot be matched; patterns containing `..` are rejected as a configuration error.
   *
   * - Default: `[]`
   */
  ignorePatterns?: string[];
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Languages: All
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean;
  /**
   * Enable JSDoc comment formatting.
   *
   * Normalizes JSDoc comments: tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: Disabled
   */
  jsdoc?: JsdocUserConfig;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Languages: JSX, TSX
   * - Default: `false`
   */
  jsxSingleQuote?: boolean;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Languages: JS, JSX, TS, TSX, JSON, JSONC, JSON5
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
   * - Languages: All
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number;
  /**
   * How to wrap prose.
   *
   * - `"always"`: Wrap prose to the print width
   * - `"never"`: Put each prose block on a single line, relying on editor/viewer soft wrapping
   * - `"preserve"`: Keep wrapping as-is
   *
   * By default, wrapping is preserved, since some services use a linebreak-sensitive renderer (e.g. GitHub comments, BitBucket).
   *
   * - Languages: Markdown, MDX, YAML
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig;
  /**
   * Change when properties in objects are quoted.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `true`
   */
  semi?: boolean;
  /**
   * Enforce single attribute per line.
   *
   * - Languages: JSX, TSX, HTML, Angular, Vue, MJML, Svelte, Astro
   * - Default: `false`
   */
  singleAttributePerLine?: boolean;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Languages: JS, JSX, TS, TSX, CSS, Less, SCSS, Markdown, MDX, YAML, Handlebars, Svelte, Astro
   * - Default: `false`
   * - Overrides `.editorconfig.quote_type`
   */
  singleQuote?: boolean;
  /**
   * Sort import statements.
   *
   * Uses a similar algorithm to [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: Disabled
   */
  sortImports?: SortImportsUserConfig;
  /**
   * Sort `package.json` keys.
   *
   * The order is NOT compatible with [prettier-plugin-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson),
   * but we believe it is clearer and easier to navigate.
   *
   * - Languages: JSON (`package.json` only)
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig;
  /**
   * Sort Tailwind CSS classes.
   *
   * Uses the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g. `config` instead of `tailwindConfig`).
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX, HTML, Vue, Angular, Handlebars, CSS, SCSS, Less, Svelte, Astro
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssUserConfig;
  /**
   * Options for `prettier-plugin-svelte`.
   *
   * Pass `true` or an object to enable `.svelte` file formatting, `false` to disable (handy in overrides).
   * Setting `true` resets to defaults, dropping options inherited from a parent scope.
   *
   * NOTE: `prettier-plugin-svelte` requires the `svelte` package (`svelte/compiler`) at runtime.
   * Oxfmt does NOT bundle it, so install it in your project, otherwise formatting fails.
   *
   * - Languages: Svelte
   * - Default: Disabled
   */
  svelte?: SvelteUserConfig;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Languages: All
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size` (falls back to `.editorconfig.tab_width`)
   */
  tabWidth?: number;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Languages: JS, JSX, TS, TSX, JSONC, JSON5, TOML, CSS, Less, SCSS, YAML
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Languages: All
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Languages: Vue
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean;
  [k: string]: unknown;
}
export interface AstroConfig {
  /**
   * Whether to normalize matching identifier attributes to shorthand or explicit form.
   * When unset, the form that was written stays as-is.
   *
   * - Default: Unset
   */
  allowShorthand?: boolean;
  /**
   * Mirror of Astro's `compressHTML` config.
   * Tells the formatter which whitespace the compiler will collapse.
   *
   * - Default: `"jsx"`
   */
  compressHTML?: AstroCompressHtmlConfig;
  /**
   * Whether to skip formatting the frontmatter.
   *
   * - Default: `false`
   */
  skipFrontmatter?: boolean;
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
   * - `"singleLine"`: Convert to single-line `/** content * /` when possible
   * - `"multiline"`: Always use multi-line format
   * - `"keep"`: Preserve original formatting
   *
   * By default, comments are collapsed to a single line when possible.
   *
   * - Default: `"singleLine"`
   */
  commentLineStrategy?: CommentLineStrategyConfig;
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
   * - `"greedy"`: Always re-wrap text to fit within print width
   * - `"balance"`: Preserve original line breaks if all lines fit within print width
   *
   * By default, description lines are always re-wrapped.
   *
   * - Default: `"greedy"`
   */
  lineWrappingStyle?: LineWrappingStyleConfig;
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
   * Accepts the same options as the top-level format options.
   */
  options?: FormatConfig;
  [k: string]: unknown;
}
export interface FormatConfig {
  /**
   * Include parentheses around a sole arrow function parameter.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"always"`
   */
  arrowParens?: ArrowParensConfig;
  /**
   * Options for `prettier-plugin-astro`.
   *
   * Pass `true` or an object to enable `.astro` file formatting, `false` to disable (handy in overrides).
   * Setting `true` resets to defaults, dropping options inherited from a parent scope.
   *
   * NOTE: `prettier-plugin-astro` requires the `@astrojs/compiler-rs` package at runtime.
   * Oxfmt does NOT bundle it, so install it in your project, otherwise formatting fails.
   *
   * - Languages: Astro
   * - Default: Disabled
   */
  astro?: AstroUserConfig;
  /**
   * Put the `>` of a multi-line element at the end of the last line,
   * instead of being alone on the next line (does not apply to self-closing elements).
   *
   * - Languages: JSX, TSX, HTML, Angular, Vue, MJML, Svelte, Astro
   * - Default: `false`
   */
  bracketSameLine?: boolean;
  /**
   * Print spaces between brackets in object literals.
   *
   * - Languages: JS, JSX, TS, TSX, JSON, JSONC, JSON5, GraphQL, YAML
   * - Default: `true`
   */
  bracketSpacing?: boolean;
  /**
   * Control whether to format embedded parts in the file.
   * For example:
   * - CSS-in-JS: template literal
   * - JS-in-Vue: `<script>` block
   * - JS-in-Markdown: code fence
   * - YAML-in-CSS/Markdown: front matter
   *
   * With `"off"`, these parts are kept as-is.
   *
   * NOTE: Some languages behave differently.
   * For Svelte, formatting fails with `"off"`, a limitation of `prettier-plugin-svelte`.
   * For Astro, the frontmatter is still formatted, but by Prettier instead of Oxfmt (use `astro.skipFrontmatter` to keep it as-is).
   *
   * - Languages: JS, JSX, TS, TSX, CSS, SCSS, Less, HTML, Vue, Angular, Svelte, Astro, Markdown, MDX
   * - Default: `"auto"`
   */
  embeddedLanguageFormatting?: EmbeddedLanguageFormattingConfig;
  /**
   * Which end of line characters to apply.
   *
   * NOTE: `"auto"` is not supported.
   *
   * - Languages: All
   * - Default: `"lf"`
   * - Overrides `.editorconfig.end_of_line`
   */
  endOfLine?: EndOfLineConfig;
  /**
   * When expressions wrap lines, print operators at the start of new lines (`"start"`)
   * or at the end of previous lines (`"end"`).
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"end"`
   */
  experimentalOperatorPosition?: OperatorPositionConfig;
  /**
   * Specify the global whitespace sensitivity.
   *
   * - Languages: HTML, Angular, Vue, Handlebars, Svelte
   * - Default: `"css"`
   */
  htmlWhitespaceSensitivity?: HtmlWhitespaceSensitivityConfig;
  /**
   * Whether to insert a final newline at the end of the file.
   *
   * - Languages: All
   * - Default: `true`
   * - Overrides `.editorconfig.insert_final_newline`
   */
  insertFinalNewline?: boolean;
  /**
   * Enable JSDoc comment formatting.
   *
   * Normalizes JSDoc comments: tag aliases are canonicalized, descriptions are capitalized,
   * long lines are wrapped, and short comments are collapsed to single-line.
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: Disabled
   */
  jsdoc?: JsdocUserConfig;
  /**
   * Use single quotes instead of double quotes in JSX.
   *
   * - Languages: JSX, TSX
   * - Default: `false`
   */
  jsxSingleQuote?: boolean;
  /**
   * How to wrap object literals when they could fit on one line or span multiple lines.
   *
   * By default, formats objects as multi-line if there is a newline prior to the first property.
   * Authors can use this heuristic to contextually improve readability, though it has some downsides.
   *
   * - Languages: JS, JSX, TS, TSX, JSON, JSONC, JSON5
   * - Default: `"preserve"`
   */
  objectWrap?: ObjectWrapConfig;
  /**
   * Specify the line length that the printer will wrap on.
   *
   * If you don't want line wrapping when formatting Markdown, you can set the `proseWrap` option to disable it.
   *
   * - Languages: All
   * - Default: `100`
   * - Overrides `.editorconfig.max_line_length`
   */
  printWidth?: number;
  /**
   * How to wrap prose.
   *
   * - `"always"`: Wrap prose to the print width
   * - `"never"`: Put each prose block on a single line, relying on editor/viewer soft wrapping
   * - `"preserve"`: Keep wrapping as-is
   *
   * By default, wrapping is preserved, since some services use a linebreak-sensitive renderer (e.g. GitHub comments, BitBucket).
   *
   * - Languages: Markdown, MDX, YAML
   * - Default: `"preserve"`
   */
  proseWrap?: ProseWrapConfig;
  /**
   * Change when properties in objects are quoted.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `"as-needed"`
   */
  quoteProps?: QuotePropsConfig;
  /**
   * Print semicolons at the ends of statements.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: `true`
   */
  semi?: boolean;
  /**
   * Enforce single attribute per line.
   *
   * - Languages: JSX, TSX, HTML, Angular, Vue, MJML, Svelte, Astro
   * - Default: `false`
   */
  singleAttributePerLine?: boolean;
  /**
   * Use single quotes instead of double quotes.
   *
   * For JSX, you can set the `jsxSingleQuote` option.
   *
   * - Languages: JS, JSX, TS, TSX, CSS, Less, SCSS, Markdown, MDX, YAML, Handlebars, Svelte, Astro
   * - Default: `false`
   * - Overrides `.editorconfig.quote_type`
   */
  singleQuote?: boolean;
  /**
   * Sort import statements.
   *
   * Uses a similar algorithm to [eslint-plugin-perfectionist/sort-imports](https://perfectionist.dev/rules/sort-imports).
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX
   * - Default: Disabled
   */
  sortImports?: SortImportsUserConfig;
  /**
   * Sort `package.json` keys.
   *
   * The order is NOT compatible with [prettier-plugin-packagejson](https://github.com/matzkoh/prettier-plugin-packagejson),
   * but we believe it is clearer and easier to navigate.
   *
   * - Languages: JSON (`package.json` only)
   * - Default: `true`
   */
  sortPackageJson?: SortPackageJsonUserConfig;
  /**
   * Sort Tailwind CSS classes.
   *
   * Uses the same algorithm as [prettier-plugin-tailwindcss](https://github.com/tailwindlabs/prettier-plugin-tailwindcss).
   * Option names omit the `tailwind` prefix used in the original plugin (e.g. `config` instead of `tailwindConfig`).
   *
   * Pass `true` or an object to enable, `false` to disable.
   *
   * - Languages: JS, JSX, TS, TSX, HTML, Vue, Angular, Handlebars, CSS, SCSS, Less, Svelte, Astro
   * - Default: Disabled
   */
  sortTailwindcss?: SortTailwindcssUserConfig;
  /**
   * Options for `prettier-plugin-svelte`.
   *
   * Pass `true` or an object to enable `.svelte` file formatting, `false` to disable (handy in overrides).
   * Setting `true` resets to defaults, dropping options inherited from a parent scope.
   *
   * NOTE: `prettier-plugin-svelte` requires the `svelte` package (`svelte/compiler`) at runtime.
   * Oxfmt does NOT bundle it, so install it in your project, otherwise formatting fails.
   *
   * - Languages: Svelte
   * - Default: Disabled
   */
  svelte?: SvelteUserConfig;
  /**
   * Specify the number of spaces per indentation-level.
   *
   * - Languages: All
   * - Default: `2`
   * - Overrides `.editorconfig.indent_size` (falls back to `.editorconfig.tab_width`)
   */
  tabWidth?: number;
  /**
   * Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
   *
   * A single-line array, for example, never gets trailing commas.
   *
   * - Languages: JS, JSX, TS, TSX, JSONC, JSON5, TOML, CSS, Less, SCSS, YAML
   * - Default: `"all"`
   */
  trailingComma?: TrailingCommaConfig;
  /**
   * Indent lines with tabs instead of spaces.
   *
   * - Languages: All
   * - Default: `false`
   * - Overrides `.editorconfig.indent_style`
   */
  useTabs?: boolean;
  /**
   * Whether or not to indent the code inside `<script>` and `<style>` tags in Vue files.
   *
   * - Languages: Vue
   * - Default: `false`
   */
  vueIndentScriptAndStyle?: boolean;
  [k: string]: unknown;
}
export interface SortImportsConfig {
  /**
   * Define your own groups for matching very specific imports.
   *
   * The first matching definition is used, and custom groups take precedence over predefined groups.
   * To give a predefined group precedence, define an equivalent custom group and put it first.
   *
   * When multiple conditions (`elementNamePattern`, `selector`, `modifiers`) are specified,
   * all of them must match.
   *
   * NOTE: Predefined group names (e.g. `side_effect`, `external`) and `unknown` are reserved and cannot be used as `groupName`.
   *
   * - Default: `[]`
   */
  customGroups?: CustomGroupItemConfig[];
  /**
   * List of import groups for sorting.
   *
   * Each import is assigned to a single group (or `unknown` if none matches), and groups are ordered as listed.
   * Within a group, imports are sorted according to `order`, `ignoreCase`, etc.
   *
   * A predefined group name is a single selector with optional modifiers, joined by `-` (e.g. `type-external`).
   * Modifiers can be in any order, but the selector must come last.
   *
   * Selectors, from most to least important:
   * - `type`: TypeScript type imports.
   * - `side_effect_style`: Side effect style imports.
   * - `side_effect`: Side effect imports.
   * - `style`: Style imports.
   * - `index`: Main file from the current directory.
   * - `sibling`: Modules from the same directory.
   * - `parent`: Modules from the parent directory.
   * - `subpath`: Node.js subpath imports.
   * - `internal`: Your internal modules.
   * - `builtin`: Node.js Built-in Modules.
   * - `external`: External modules installed in the project.
   * - `import`: Any import.
   *
   * Modifiers, from most to least important:
   * - `side_effect`: Side effect imports.
   * - `type`: TypeScript type imports.
   * - `value`: Value imports.
   * - `default`: Imports containing the default specifier.
   * - `wildcard`: Imports containing the wildcard (`* as`) specifier.
   * - `named`: Imports containing at least one named specifier.
   *
   * Wrap groups in an array to sort them together as a single group (order within the array does not matter).
   *
   * To override `newlinesBetween` at a specific group boundary,
   * put a `{ "newlinesBetween": boolean }` marker object at that position.
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
   */
  groups?: SortGroupItemConfig[];
  /**
   * Ignore case when sorting.
   *
   * - Default: `true`
   */
  ignoreCase?: boolean;
  /**
   * Prefixes for identifying internal imports.
   *
   * - Default: `["~/", "@/", "#"]`
   */
  internalPattern?: string[];
  /**
   * Add newlines between groups.
   *
   * - Default: `true`
   */
  newlinesBetween?: boolean;
  /**
   * Sort in ascending or descending order.
   *
   * - Default: `"asc"`
   */
  order?: SortOrderConfig;
  /**
   * Use comments to separate imports into logical groups.
   *
   * When `true`, all comments are treated as delimiters.
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
   * Use empty lines to separate imports into logical groups.
   *
   * When `true`, imports are not sorted across an empty line.
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
   * Sort side effect imports.
   *
   * Disabled by default for safety reasons.
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
   * All specified modifiers must match.
   */
  modifiers?: ImportModifierConfig[];
  /**
   * Selector to match the import kind.
   */
  selector?: ImportSelectorConfig;
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
export interface SvelteConfig {
  /**
   * Whether to allow attribute shorthand if attribute name and expression are the same.
   *
   * - Default: `true`
   */
  allowShorthand?: boolean;
  /**
   * Whether to indent code inside `<script>` and `<style>` tags.
   *
   * - Default: `true`
   */
  indentScriptAndStyle?: boolean;
  /**
   * The order in which Svelte component sections are printed.
   * Join `options`, `scripts`, `markup`, `styles` with `-` in the desired order,
   * or use `none` to keep the original order.
   *
   * - Default: `"options-scripts-markup-styles"`
   */
  sortOrder?: string;
  [k: string]: unknown;
}
