/*
 * This file is generated from npm/oxlint/configuration_schema.json.
 * Run `just linter-config-ts` to regenerate.
 */

export type AllowWarnDeny = ("allow" | "off" | "warn" | "error" | "deny") | number;
export type GlobalValue = "readonly" | "writable" | "off";
export type ExternalPluginEntry =
  | string
  | {
      /**
       * Custom name/alias for the plugin.
       *
       * Note: The following plugin names are reserved because they are implemented natively in Rust within oxlint and cannot be used for JS plugins:
       * - react (includes react-hooks)
       * - unicorn
       * - typescript (includes @typescript-eslint)
       * - oxc
       * - import (includes import-x)
       * - jsdoc
       * - jest
       * - vitest
       * - jsx-a11y
       * - nextjs
       * - react-perf
       * - promise
       * - node
       * - vue
       * - eslint
       *
       * If you need to use the JavaScript version of any of these plugins, provide a custom alias to avoid conflicts.
       */
      name: string;
      /**
       * Path or package name of the plugin
       */
      specifier: string;
    };
/**
 * A set of glob patterns.
 */
export type GlobSet = string[];
export type LintPluginOptionsSchema =
  | "eslint"
  | "react"
  | "unicorn"
  | "typescript"
  | "oxc"
  | "import"
  | "jsdoc"
  | "jest"
  | "vitest"
  | "jsx-a11y"
  | "nextjs"
  | "react-perf"
  | "promise"
  | "node"
  | "vue";
export type LintPlugins = LintPluginOptionsSchema[];
export type DummyRule = AllowWarnDeny | unknown[];
export type OxlintOverrides = OxlintOverride[];
export type TagNamePreference =
  | string
  | {
      message: string;
      replacement: string;
      [k: string]: unknown;
    }
  | {
      message: string;
      [k: string]: unknown;
    }
  | boolean;
export type OneOrManyFor_String = string | string[];
export type CustomComponent =
  | string
  | {
      attribute: string;
      name: string;
      [k: string]: unknown;
    }
  | {
      attributes: string[];
      name: string;
      [k: string]: unknown;
    };

/**
 * Oxlint Configuration File
 *
 * This configuration is aligned with ESLint v8's configuration schema (`eslintrc.json`).
 *
 * Usage: `oxlint -c oxlintrc.json --import-plugin`
 *
 * ::: danger NOTE
 *
 * Only the `.json` format is supported. You can use comments in configuration files.
 *
 * :::
 *
 * Example
 *
 * `.oxlintrc.json`
 *
 * ```json
 * {
 * "$schema": "./node_modules/oxlint/configuration_schema.json",
 * "plugins": ["import", "typescript", "unicorn"],
 * "env": {
 * "browser": true
 * },
 * "globals": {
 * "foo": "readonly"
 * },
 * "settings": {
 * },
 * "rules": {
 * "eqeqeq": "warn",
 * "import/no-cycle": "error",
 * "react/self-closing-comp": ["error", { "html": false }]
 * },
 * "overrides": [
 * {
 * "files": ["*.test.ts", "*.spec.ts"],
 * "rules": {
 * "@typescript-eslint/no-explicit-any": "off"
 * }
 * }
 * ]
 * }
 * ```
 */
export interface Oxlintrc {
  /**
   * Schema URI for editor tooling.
   */
  $schema?: string | null;
  categories?: RuleCategories;
  /**
   * Environments enable and disable collections of global variables.
   */
  env?: OxlintEnv;
  /**
   * Paths of configuration files that this configuration file extends (inherits from). The files
   * are resolved relative to the location of the configuration file that contains the `extends`
   * property. The configuration files are merged from the first to the last, with the last file
   * overriding the previous ones.
   */
  extends?: string[];
  /**
   * Enabled or disabled specific global variables.
   */
  globals?: OxlintGlobals;
  /**
   * Globs to ignore during linting. These are resolved from the configuration file path.
   */
  ignorePatterns?: string[];
  /**
   * JS plugins, allows usage of ESLint plugins with Oxlint.
   *
   * Read more about JS plugins in
   * [the docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
   *
   * Note: JS plugins are experimental and not subject to semver.
   * They are not supported in the language server (and thus editor integrations) at present.
   */
  jsPlugins?: null | ExternalPluginEntry[];
  /**
   * Add, remove, or otherwise reconfigure rules for specific files or groups of files.
   */
  overrides?: OxlintOverrides;
  /**
   * Enabled built-in plugins for Oxlint.
   * You can view the list of available plugins on
   * [the website](https://oxc.rs/docs/guide/usage/linter/plugins.html#supported-plugins).
   *
   * NOTE: Setting the `plugins` field will overwrite the base set of plugins.
   * The `plugins` array should reflect all of the plugins you want to use.
   */
  plugins?: LintPlugins | null;
  /**
   * Example
   *
   * `.oxlintrc.json`
   *
   * ```json
   * {
   * "$schema": "./node_modules/oxlint/configuration_schema.json",
   * "rules": {
   * "eqeqeq": "warn",
   * "import/no-cycle": "error",
   * "prefer-const": ["error", { "ignoreReadBeforeAssign": true }]
   * }
   * }
   * ```
   *
   * See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html) for the list of
   * rules.
   */
  rules?: DummyRuleMap;
  settings?: OxlintPluginSettings;
}
/**
 * Configure an entire category of rules all at once.
 *
 * Rules enabled or disabled this way will be overwritten by individual rules in the `rules` field.
 *
 * Example
 * ```json
 * {
 *     "$schema": "./node_modules/oxlint/configuration_schema.json",
 *     "categories": {
 *         "correctness": "warn"
 *     },
 *     "rules": {
 *         "eslint/no-unused-vars": "error"
 *     }
 * }
 * ```
 */
export interface RuleCategories {
  correctness?: AllowWarnDeny;
  nursery?: AllowWarnDeny;
  pedantic?: AllowWarnDeny;
  perf?: AllowWarnDeny;
  restriction?: AllowWarnDeny;
  style?: AllowWarnDeny;
  suspicious?: AllowWarnDeny;
}
/**
 * Predefine global variables.
 *
 * Environments specify what global variables are predefined.
 * See [ESLint's list of environments](https://eslint.org/docs/v8.x/use/configure/language-options#specifying-environments)
 * for what environments are available and what each one provides.
 */
export type OxlintEnv = Record<string, boolean>;
/**
 * Add or remove global variables.
 *
 * For each global variable, set the corresponding value equal to `"writable"`
 * to allow the variable to be overwritten or `"readonly"` to disallow overwriting.
 *
 * Globals can be disabled by setting their value to `"off"`. For example, in
 * an environment where most Es2015 globals are available but `Promise` is unavailable,
 * you might use this config:
 *
 * ```json
 *
 * {
 * "$schema": "./node_modules/oxlint/configuration_schema.json",
 * "env": {
 * "es6": true
 * },
 * "globals": {
 * "Promise": "off"
 * }
 * }
 *
 * ```
 *
 * You may also use `"readable"` or `false` to represent `"readonly"`, and
 * `"writeable"` or `true` to represent `"writable"`.
 */
export type OxlintGlobals = Record<string, GlobalValue>;
export interface OxlintOverride {
  /**
   * Environments enable and disable collections of global variables.
   */
  env?: OxlintEnv | null;
  /**
   * A list of glob patterns to override.
   *
   * ## Example
   * `[ "*.test.ts", "*.spec.ts" ]`
   */
  files: GlobSet;
  /**
   * Enabled or disabled specific global variables.
   */
  globals?: OxlintGlobals | null;
  /**
   * JS plugins for this override, allows usage of ESLint plugins with Oxlint.
   *
   * Read more about JS plugins in
   * [the docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
   *
   * Note: JS plugins are experimental and not subject to semver.
   * They are not supported in the language server (and thus editor integrations) at present.
   */
  jsPlugins?: null | ExternalPluginEntry[];
  /**
   * Optionally change what plugins are enabled for this override. When
   * omitted, the base config's plugins are used.
   */
  plugins?: LintPlugins | null;
  rules?: DummyRuleMap;
}
/**
 * See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html)
 */
export type DummyRuleMap = Record<string, DummyRule>;
/**
 * Configure the behavior of linter plugins.
 *
 * Here's an example if you're using Next.js in a monorepo:
 *
 * ```json
 * {
 * "settings": {
 * "next": {
 * "rootDir": "apps/dashboard/"
 * },
 * "react": {
 * "linkComponents": [
 * { "name": "Link", "linkAttribute": "to" }
 * ]
 * },
 * "jsx-a11y": {
 * "components": {
 * "Link": "a",
 * "Button": "button"
 * }
 * }
 * }
 * }
 * ```
 */
export interface OxlintPluginSettings {
  jsdoc?: JSDocPluginSettings;
  "jsx-a11y"?: JSXA11YPluginSettings;
  next?: NextPluginSettings;
  react?: ReactPluginSettings;
  vitest?: VitestPluginSettings;
  [k: string]: unknown;
}
export interface JSDocPluginSettings {
  /**
   * Only for `require-(yields|returns|description|example|param|throws)` rule
   */
  augmentsExtendsReplacesDocs?: boolean;
  /**
   * Only for `require-param-type` and `require-param-description` rule
   */
  exemptDestructuredRootsFromChecks?: boolean;
  /**
   * For all rules but NOT apply to `empty-tags` rule
   */
  ignoreInternal?: boolean;
  /**
   * For all rules but NOT apply to `check-access` and `empty-tags` rule
   */
  ignorePrivate?: boolean;
  /**
   * Only for `require-(yields|returns|description|example|param|throws)` rule
   */
  ignoreReplacesDocs?: boolean;
  /**
   * Only for `require-(yields|returns|description|example|param|throws)` rule
   */
  implementsReplacesDocs?: boolean;
  /**
   * Only for `require-(yields|returns|description|example|param|throws)` rule
   */
  overrideReplacesDocs?: boolean;
  tagNamePreference?: Record<string, TagNamePreference>;
  [k: string]: unknown;
}
/**
 * Configure JSX A11y plugin rules.
 *
 * See
 * [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations)'s
 * configuration for a full reference.
 */
export interface JSXA11YPluginSettings {
  /**
   * Map of attribute names to their DOM equivalents.
   * This is useful for non-React frameworks that use different attribute names.
   *
   * Example:
   *
   * ```json
   * {
   * "settings": {
   * "jsx-a11y": {
   * "attributes": {
   * "for": ["htmlFor", "for"]
   * }
   * }
   * }
   * }
   * ```
   */
  attributes?: Record<string, string[]>;
  /**
   * To have your custom components be checked as DOM elements, you can
   * provide a mapping of your component names to the DOM element name.
   *
   * Example:
   *
   * ```json
   * {
   * "settings": {
   * "jsx-a11y": {
   * "components": {
   * "Link": "a",
   * "IconButton": "button"
   * }
   * }
   * }
   * }
   * ```
   */
  components?: Record<string, string>;
  /**
   * An optional setting that define the prop your code uses to create polymorphic components.
   * This setting will be used to determine the element type in rules that
   * require semantic context.
   *
   * For example, if you set the `polymorphicPropName` to `as`, then this element:
   *
   * ```jsx
   * <Box as="h3">Hello</Box>
   * ```
   *
   * Will be treated as an `h3`. If not set, this component will be treated
   * as a `Box`.
   */
  polymorphicPropName?: string | null;
  [k: string]: unknown;
}
/**
 * Configure Next.js plugin rules.
 */
export interface NextPluginSettings {
  /**
   * The root directory of the Next.js project.
   *
   * This is particularly useful when you have a monorepo and your Next.js
   * project is in a subfolder.
   *
   * Example:
   *
   * ```json
   * {
   * "settings": {
   * "next": {
   * "rootDir": "apps/dashboard/"
   * }
   * }
   * }
   * ```
   */
  rootDir?: OneOrManyFor_String;
  [k: string]: unknown;
}
/**
 * Configure React plugin rules.
 *
 * Derived from [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-)
 */
export interface ReactPluginSettings {
  /**
   * Functions that wrap React components and should be treated as HOCs.
   *
   * Example:
   *
   * ```jsonc
   * {
   * "settings": {
   * "react": {
   * "componentWrapperFunctions": ["observer", "withRouter"]
   * }
   * }
   * }
   * ```
   */
  componentWrapperFunctions?: string[];
  /**
   * Components used as alternatives to `<form>` for forms, such as `<Formik>`.
   *
   * Example:
   *
   * ```jsonc
   * {
   * "settings": {
   * "react": {
   * "formComponents": [
   * "CustomForm",
   * // OtherForm is considered a form component and has an endpoint attribute
   * { "name": "OtherForm", "formAttribute": "endpoint" },
   * // allows specifying multiple properties if necessary
   * { "name": "Form", "formAttribute": ["registerEndpoint", "loginEndpoint"] }
   * ]
   * }
   * }
   * }
   * ```
   */
  formComponents?: CustomComponent[];
  /**
   * Components used as alternatives to `<a>` for linking, such as `<Link>`.
   *
   * Example:
   *
   * ```jsonc
   * {
   * "settings": {
   * "react": {
   * "linkComponents": [
   * "HyperLink",
   * // Use `linkAttribute` for components that use a different prop name
   * // than `href`.
   * { "name": "MyLink", "linkAttribute": "to" },
   * // allows specifying multiple properties if necessary
   * { "name": "Link", "linkAttribute": ["to", "href"] }
   * ]
   * }
   * }
   * }
   * ```
   */
  linkComponents?: CustomComponent[];
  /**
   * React version to use for version-specific rules.
   *
   * Accepts semver versions (e.g., "18.2.0", "17.0").
   *
   * Example:
   *
   * ```jsonc
   * {
   * "settings": {
   * "react": {
   * "version": "18.2.0"
   * }
   * }
   * }
   * ```
   */
  version?: string | null;
  [k: string]: unknown;
}
/**
 * Configure Vitest plugin rules.
 *
 * See [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest)'s
 * configuration for a full reference.
 */
export interface VitestPluginSettings {
  /**
   * Whether to enable typecheck mode for Vitest rules.
   * When enabled, some rules will skip certain checks for describe blocks
   * to accommodate TypeScript type checking scenarios.
   */
  typecheck?: boolean;
  [k: string]: unknown;
}
