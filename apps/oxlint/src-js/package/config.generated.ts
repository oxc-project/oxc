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
       * - jsx-a11y (includes jsx-a11y-x)
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
 * Patterns are matched against paths relative to the configuration file's directory.
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
export type DummyRule = AllowWarnDeny | [AllowWarnDeny, ...unknown[]];
export type RuleNoConfig = AllowWarnDeny | [AllowWarnDeny];
export type CountThis = "always" | "never" | "except-void";
export type CaseType = "camelCase" | "snake_case";
export type OxlintOverrides = OxlintOverride[];
export type JestVersionSchema = number | string;
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
 * Usage: `oxlint -c oxlintrc.json`
 *
 * Example
 *
 * `.oxlintrc.json`
 *
 * ```json
 * {
 *   "$schema": "./node_modules/oxlint/configuration_schema.json",
 *   "plugins": [
 *     "import",
 *     "typescript",
 *     "unicorn"
 *   ],
 *   "env": {
 *     "browser": true
 *   },
 *   "globals": {
 *     "foo": "readonly"
 *   },
 *   "settings": {
 *     "react": {
 *       "version": "18.2.0"
 *     },
 *     "custom": {
 *       "option": true
 *     }
 *   },
 *   "rules": {
 *     "eqeqeq": "warn",
 *     "import/no-cycle": "error",
 *     "react/self-closing-comp": [
 *       "error",
 *       {
 *         "html": false
 *       }
 *     ]
 *   },
 *   "overrides": [
 *     {
 *       "files": [
 *         "*.test.ts",
 *         "*.spec.ts"
 *       ],
 *       "rules": {
 *         "@typescript-eslint/no-explicit-any": "off"
 *       }
 *     }
 *   ]
 * }
 * ```
 *
 * `oxlint.config.ts`
 *
 * ```ts
 * import { defineConfig } from "oxlint";
 *
 * export default defineConfig({
 * plugins: ["import", "typescript", "unicorn"],
 * env: {
 * "browser": true
 * },
 * globals: {
 * "foo": "readonly"
 * },
 * settings: {
 * react: {
 * version: "18.2.0"
 * },
 * custom: { option: true }
 * },
 * rules: {
 * "eqeqeq": "warn",
 * "import/no-cycle": "error",
 * "react/self-closing-comp": ["error", { "html": false }]
 * },
 * overrides: [
 * {
 * files: ["*.test.ts", "*.spec.ts"],
 * rules: {
 * "@typescript-eslint/no-explicit-any": "off"
 * }
 * }
 * ]
 * });
 * ```
 */
export interface Oxlintrc {
  /**
   * Schema URI for editor tooling.
   */
  $schema?: string;
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
   * Note: JS plugins are in alpha and not subject to semver.
   *
   * Examples:
   *
   * Basic usage with a local plugin path.
   *
   * ```json
   * {
   *   "jsPlugins": [
   *     "./custom-plugin.js"
   *   ],
   *   "rules": {
   *     "custom/rule-name": "warn"
   *   }
   * }
   * ```
   *
   * Basic usage with a TypeScript plugin and a local plugin path.
   *
   * TypeScript plugin files are supported in the following environments:
   * - Deno and Bun: TypeScript files are supported natively.
   * - Node.js >=22.18.0 and Node.js ^20.19.0: TypeScript files are supported natively with built-in
   * type-stripping enabled by default.
   *
   * For older Node.js versions, TypeScript plugins are not supported. Please use JavaScript plugins or upgrade your Node version.
   *
   * ```json
   * {
   *   "jsPlugins": [
   *     "./custom-plugin.ts"
   *   ],
   *   "rules": {
   *     "custom/rule-name": "warn"
   *   }
   * }
   * ```
   *
   * Using a built-in Rust plugin alongside a JS plugin with the same name
   * by giving the JS plugin an alias.
   *
   * ```json
   * {
   *   "plugins": [
   *     "import"
   *   ],
   *   "jsPlugins": [
   *     {
   *       "name": "import-js",
   *       "specifier": "eslint-plugin-import"
   *     }
   *   ],
   *   "rules": {
   *     "import/no-cycle": "error",
   *     "import-js/no-unresolved": "warn"
   *   }
   * }
   * ```
   */
  jsPlugins?: null | ExternalPluginEntry[];
  /**
   * Oxlint config options.
   */
  options?: OxlintOptions;
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
  plugins?: LintPlugins;
  /**
   * Example
   *
   * `.oxlintrc.json`
   *
   * ```json
   * {
   *   "$schema": "./node_modules/oxlint/configuration_schema.json",
   *   "rules": {
   *     "eqeqeq": "warn",
   *     "import/no-cycle": "error",
   *     "prefer-const": [
   *       "error",
   *       {
   *         "ignoreReadBeforeAssign": true
   *       }
   *     ]
   *   }
   * }
   * ```
   *
   * See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html) for the list of
   * rules.
   */
  rules?: DummyRuleMap;
  /**
   * Plugin-specific configuration for both built-in and custom plugins.
   * This includes settings for built-in plugins such as `react` and `jsdoc`
   * as well as configuring settings for JS custom plugins loaded via `jsPlugins`.
   */
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
 *   "$schema": "./node_modules/oxlint/configuration_schema.json",
 *   "categories": {
 *     "correctness": "warn"
 *   },
 *   "rules": {
 *     "eslint/no-unused-vars": "error"
 *   }
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
 * Available environments:
 * - amd - require() and define() globals.
 * - applescript - AppleScript globals.
 * - astro - Astro globals.
 * - atomtest - Atom test globals.
 * - audioworklet - AudioWorklet globals.
 * - browser - browser globals.
 * - builtin - Latest ECMAScript globals, equivalent to es2026.
 * - commonjs - CommonJS globals and scoping.
 * - embertest - Ember test globals.
 * - es2015 - ECMAScript 2015 globals.
 * - es2016 - ECMAScript 2016 globals.
 * - es2017 - ECMAScript 2017 globals.
 * - es2018 - ECMAScript 2018 globals.
 * - es2019 - ECMAScript 2019 globals.
 * - es2020 - ECMAScript 2020 globals.
 * - es2021 - ECMAScript 2021 globals.
 * - es2022 - ECMAScript 2022 globals.
 * - es2023 - ECMAScript 2023 globals.
 * - es2024 - ECMAScript 2024 globals.
 * - es2025 - ECMAScript 2025 globals.
 * - es2026 - ECMAScript 2026 globals.
 * - es6 - ECMAScript 6 globals except modules.
 * - greasemonkey - GreaseMonkey globals.
 * - jasmine - Jasmine globals.
 * - jest - Jest globals.
 * - jquery - jQuery globals.
 * - meteor - Meteor globals.
 * - mocha - Mocha globals.
 * - mongo - MongoDB globals.
 * - nashorn - Java 8 Nashorn globals.
 * - node - Node.js globals and scoping.
 * - phantomjs - PhantomJS globals.
 * - prototypejs - Prototype.js globals.
 * - protractor - Protractor globals.
 * - qunit - QUnit globals.
 * - serviceworker - Service Worker globals.
 * - shared-node-browser - Node.js and Browser common globals.
 * - shelljs - ShellJS globals.
 * - svelte - Svelte globals.
 * - vitest - Vitest globals.
 * - vue - Vue globals.
 * - webextensions - WebExtensions globals.
 * - worker - Web Workers globals.
 */
export interface OxlintEnv {
  [k: string]: boolean;
}
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
 * {
 *   "$schema": "./node_modules/oxlint/configuration_schema.json",
 *   "env": {
 *     "es6": true
 *   },
 *   "globals": {
 *     "Promise": "off"
 *   }
 * }
 * ```
 *
 * You may also use `"readable"` or `false` to represent `"readonly"`, and
 * `"writeable"` or `true` to represent `"writable"`.
 */
export interface OxlintGlobals {
  [k: string]: GlobalValue;
}
/**
 * Options for the linter.
 */
export interface OxlintOptions {
  /**
   * Ensure warnings produce a non-zero exit code.
   *
   * Equivalent to passing `--deny-warnings` on the CLI.
   */
  denyWarnings?: boolean;
  /**
   * Specify a warning threshold. Exits with an error status if warnings exceed this value.
   *
   * Equivalent to passing `--max-warnings` on the CLI.
   */
  maxWarnings?: number;
  /**
   * Report unused disable directives (e.g. `// oxlint-disable-line` or `// eslint-disable-line`).
   *
   * Equivalent to passing `--report-unused-disable-directives-severity` on the CLI.
   * CLI flags take precedence over this value when both are set.
   * Only supported in the root configuration file.
   */
  reportUnusedDisableDirectives?: AllowWarnDeny;
  /**
   * Whether oxlint should respect `eslint-disable*` and `eslint-enable*`
   * directives in addition to its native `oxlint-*` directives.
   *
   * Defaults to `true`.
   * Only supported in the root configuration file.
   */
  respectEslintDisableDirectives?: boolean;
  /**
   * Enable rules that require type information.
   *
   * Equivalent to passing `--type-aware` on the CLI.
   *
   * Note that this requires the `oxlint-tsgolint` package to be installed.
   */
  typeAware?: boolean;
  /**
   * Enable experimental type checking (includes TypeScript compiler diagnostics).
   *
   * Equivalent to passing `--type-check` on the CLI.
   *
   * Note that this requires the `oxlint-tsgolint` package to be installed.
   */
  typeCheck?: boolean;
}
export interface OxlintOverride {
  /**
   * Environments enable and disable collections of global variables.
   */
  env?: OxlintEnv;
  /**
   * A list of glob patterns to exclude from this override.
   *
   * Files matching these patterns are not globally ignored; this override
   * simply does not apply to them.
   *
   * ## Example
   * `[ "*.generated.ts", "fixtures/**" ]`
   */
  excludeFiles?: GlobSet;
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
  globals?: OxlintGlobals;
  /**
   * JS plugins for this override, allows usage of ESLint plugins with Oxlint.
   *
   * Read more about JS plugins in
   * [the docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
   *
   * Note: JS plugins are in alpha and not subject to semver.
   */
  jsPlugins?: null | ExternalPluginEntry[];
  /**
   * Optionally change what plugins are enabled for this override. When
   * omitted, the base config's plugins are used.
   */
  plugins?: LintPlugins;
  rules?: DummyRuleMap;
}
/**
 * See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html)
 */
export interface DummyRuleMap {
  "accessor-pairs"?: DummyRule;
  "array-callback-return"?: DummyRule;
  "arrow-body-style"?: DummyRule;
  "block-scoped-var"?: RuleNoConfig;
  "capitalized-comments"?: DummyRule;
  "class-methods-use-this"?: DummyRule;
  complexity?: DummyRule;
  "constructor-super"?: RuleNoConfig;
  curly?: DummyRule;
  "default-case"?: DummyRule;
  "default-case-last"?: RuleNoConfig;
  "default-param-last"?: RuleNoConfig;
  eqeqeq?: DummyRule;
  "for-direction"?: RuleNoConfig;
  "func-name-matching"?: DummyRule;
  "func-names"?: DummyRule;
  "func-style"?: DummyRule;
  "getter-return"?: DummyRule;
  "grouped-accessor-pairs"?: DummyRule;
  "guard-for-in"?: RuleNoConfig;
  "id-length"?: DummyRule;
  "id-match"?: DummyRule;
  "import/consistent-type-specifier-style"?: DummyRule;
  "import/default"?: RuleNoConfig;
  "import/export"?: RuleNoConfig;
  "import/exports-last"?: RuleNoConfig;
  "import/extensions"?: DummyRule;
  "import/first"?: DummyRule;
  "import/group-exports"?: RuleNoConfig;
  "import/max-dependencies"?: DummyRule;
  "import/named"?: RuleNoConfig;
  "import/namespace"?: DummyRule;
  "import/newline-after-import"?: DummyRule;
  "import/no-absolute-path"?: DummyRule;
  "import/no-amd"?: RuleNoConfig;
  "import/no-anonymous-default-export"?: DummyRule;
  "import/no-commonjs"?: DummyRule;
  "import/no-cycle"?: DummyRule;
  "import/no-default-export"?: RuleNoConfig;
  "import/no-duplicates"?: DummyRule;
  "import/no-dynamic-require"?: DummyRule;
  "import/no-empty-named-blocks"?: RuleNoConfig;
  "import/no-mutable-exports"?: RuleNoConfig;
  "import/no-named-as-default"?: RuleNoConfig;
  "import/no-named-as-default-member"?: RuleNoConfig;
  "import/no-named-default"?: RuleNoConfig;
  "import/no-named-export"?: RuleNoConfig;
  "import/no-namespace"?: DummyRule;
  "import/no-nodejs-modules"?: DummyRule;
  "import/no-relative-parent-imports"?: RuleNoConfig;
  "import/no-self-import"?: RuleNoConfig;
  "import/no-unassigned-import"?: DummyRule;
  "import/no-webpack-loader-syntax"?: RuleNoConfig;
  "import/prefer-default-export"?: DummyRule;
  "import/unambiguous"?: RuleNoConfig;
  "init-declarations"?: DummyRule;
  "jest/consistent-test-it"?: DummyRule;
  "jest/expect-expect"?: DummyRule;
  "jest/max-expects"?: DummyRule;
  "jest/max-nested-describe"?: DummyRule;
  "jest/no-alias-methods"?: RuleNoConfig;
  "jest/no-commented-out-tests"?: RuleNoConfig;
  "jest/no-conditional-expect"?: RuleNoConfig;
  "jest/no-conditional-in-test"?: RuleNoConfig;
  "jest/no-confusing-set-timeout"?: RuleNoConfig;
  "jest/no-deprecated-functions"?: DummyRule;
  "jest/no-disabled-tests"?: RuleNoConfig;
  "jest/no-done-callback"?: RuleNoConfig;
  "jest/no-duplicate-hooks"?: RuleNoConfig;
  "jest/no-export"?: RuleNoConfig;
  "jest/no-focused-tests"?: RuleNoConfig;
  "jest/no-hooks"?: DummyRule;
  "jest/no-identical-title"?: RuleNoConfig;
  "jest/no-interpolation-in-snapshots"?: RuleNoConfig;
  "jest/no-jasmine-globals"?: RuleNoConfig;
  "jest/no-large-snapshots"?: DummyRule;
  "jest/no-mocks-import"?: RuleNoConfig;
  "jest/no-restricted-jest-methods"?: DummyRule;
  "jest/no-restricted-matchers"?: DummyRule;
  "jest/no-standalone-expect"?: DummyRule;
  "jest/no-test-prefixes"?: RuleNoConfig;
  "jest/no-test-return-statement"?: RuleNoConfig;
  "jest/no-unneeded-async-expect-function"?: RuleNoConfig;
  "jest/no-untyped-mock-factory"?: RuleNoConfig;
  "jest/padding-around-after-all-blocks"?: RuleNoConfig;
  "jest/padding-around-test-blocks"?: RuleNoConfig;
  "jest/prefer-called-with"?: RuleNoConfig;
  "jest/prefer-comparison-matcher"?: RuleNoConfig;
  "jest/prefer-each"?: RuleNoConfig;
  "jest/prefer-ending-with-an-expect"?: DummyRule;
  "jest/prefer-equality-matcher"?: RuleNoConfig;
  "jest/prefer-expect-assertions"?: DummyRule;
  "jest/prefer-expect-resolves"?: RuleNoConfig;
  "jest/prefer-hooks-in-order"?: RuleNoConfig;
  "jest/prefer-hooks-on-top"?: RuleNoConfig;
  "jest/prefer-importing-jest-globals"?: DummyRule;
  "jest/prefer-jest-mocked"?: RuleNoConfig;
  "jest/prefer-lowercase-title"?: DummyRule;
  "jest/prefer-mock-promise-shorthand"?: RuleNoConfig;
  "jest/prefer-mock-return-shorthand"?: RuleNoConfig;
  "jest/prefer-snapshot-hint"?: DummyRule;
  "jest/prefer-spy-on"?: RuleNoConfig;
  "jest/prefer-strict-equal"?: RuleNoConfig;
  "jest/prefer-to-be"?: RuleNoConfig;
  "jest/prefer-to-contain"?: RuleNoConfig;
  "jest/prefer-to-have-been-called"?: RuleNoConfig;
  "jest/prefer-to-have-been-called-times"?: RuleNoConfig;
  "jest/prefer-to-have-length"?: RuleNoConfig;
  "jest/prefer-todo"?: RuleNoConfig;
  "jest/require-hook"?: DummyRule;
  "jest/require-to-throw-message"?: RuleNoConfig;
  "jest/require-top-level-describe"?: DummyRule;
  "jest/valid-describe-callback"?: RuleNoConfig;
  "jest/valid-expect"?: DummyRule;
  "jest/valid-expect-in-promise"?: RuleNoConfig;
  "jest/valid-title"?: DummyRule;
  "jsdoc/check-access"?: RuleNoConfig;
  "jsdoc/check-property-names"?: RuleNoConfig;
  "jsdoc/check-tag-names"?: DummyRule;
  "jsdoc/empty-tags"?: DummyRule;
  "jsdoc/implements-on-classes"?: RuleNoConfig;
  "jsdoc/no-defaults"?: DummyRule;
  "jsdoc/require-param"?: DummyRule;
  "jsdoc/require-param-description"?: RuleNoConfig;
  "jsdoc/require-param-name"?: RuleNoConfig;
  "jsdoc/require-param-type"?: RuleNoConfig;
  "jsdoc/require-property"?: RuleNoConfig;
  "jsdoc/require-property-description"?: RuleNoConfig;
  "jsdoc/require-property-name"?: RuleNoConfig;
  "jsdoc/require-property-type"?: RuleNoConfig;
  "jsdoc/require-returns"?: DummyRule;
  "jsdoc/require-returns-description"?: RuleNoConfig;
  "jsdoc/require-returns-type"?: RuleNoConfig;
  "jsdoc/require-throws-description"?: RuleNoConfig;
  "jsdoc/require-throws-type"?: RuleNoConfig;
  "jsdoc/require-yields"?: DummyRule;
  "jsdoc/require-yields-description"?: RuleNoConfig;
  "jsdoc/require-yields-type"?: RuleNoConfig;
  "jsx-a11y/alt-text"?: DummyRule;
  "jsx-a11y/anchor-ambiguous-text"?: DummyRule;
  "jsx-a11y/anchor-has-content"?: RuleNoConfig;
  "jsx-a11y/anchor-is-valid"?: DummyRule;
  "jsx-a11y/aria-activedescendant-has-tabindex"?: RuleNoConfig;
  "jsx-a11y/aria-props"?: RuleNoConfig;
  "jsx-a11y/aria-proptypes"?: RuleNoConfig;
  "jsx-a11y/aria-role"?: DummyRule;
  "jsx-a11y/aria-unsupported-elements"?: RuleNoConfig;
  "jsx-a11y/autocomplete-valid"?: DummyRule;
  "jsx-a11y/click-events-have-key-events"?: RuleNoConfig;
  "jsx-a11y/control-has-associated-label"?: DummyRule;
  "jsx-a11y/heading-has-content"?: DummyRule;
  "jsx-a11y/html-has-lang"?: RuleNoConfig;
  "jsx-a11y/iframe-has-title"?: RuleNoConfig;
  "jsx-a11y/img-redundant-alt"?: DummyRule;
  "jsx-a11y/interactive-supports-focus"?: DummyRule;
  "jsx-a11y/label-has-associated-control"?: DummyRule;
  "jsx-a11y/lang"?: RuleNoConfig;
  "jsx-a11y/media-has-caption"?: DummyRule;
  "jsx-a11y/mouse-events-have-key-events"?: DummyRule;
  "jsx-a11y/no-access-key"?: RuleNoConfig;
  "jsx-a11y/no-aria-hidden-on-focusable"?: RuleNoConfig;
  "jsx-a11y/no-autofocus"?: DummyRule;
  "jsx-a11y/no-distracting-elements"?: DummyRule;
  "jsx-a11y/no-interactive-element-to-noninteractive-role"?: DummyRule;
  "jsx-a11y/no-noninteractive-element-interactions"?: DummyRule;
  "jsx-a11y/no-noninteractive-element-to-interactive-role"?: DummyRule;
  "jsx-a11y/no-noninteractive-tabindex"?: DummyRule;
  "jsx-a11y/no-redundant-roles"?: RuleNoConfig;
  "jsx-a11y/no-static-element-interactions"?: DummyRule;
  "jsx-a11y/prefer-tag-over-role"?: RuleNoConfig;
  "jsx-a11y/role-has-required-aria-props"?: RuleNoConfig;
  "jsx-a11y/role-supports-aria-props"?: RuleNoConfig;
  "jsx-a11y/scope"?: RuleNoConfig;
  "jsx-a11y/tabindex-no-positive"?: RuleNoConfig;
  "logical-assignment-operators"?: DummyRule;
  "max-classes-per-file"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxClassesPerFileConfig];
  "max-depth"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxDepth];
  "max-lines"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxLinesConfig];
  "max-lines-per-function"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxLinesPerFunctionConfig];
  "max-nested-callbacks"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxNestedCallbacks];
  "max-params"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxParamsConfig];
  "max-statements"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, number | MaxStatementsConfig];
  "new-cap"?: DummyRule;
  "nextjs/google-font-display"?: RuleNoConfig;
  "nextjs/google-font-preconnect"?: RuleNoConfig;
  "nextjs/inline-script-id"?: RuleNoConfig;
  "nextjs/next-script-for-ga"?: RuleNoConfig;
  "nextjs/no-assign-module-variable"?: RuleNoConfig;
  "nextjs/no-async-client-component"?: RuleNoConfig;
  "nextjs/no-before-interactive-script-outside-document"?: RuleNoConfig;
  "nextjs/no-css-tags"?: RuleNoConfig;
  "nextjs/no-document-import-in-page"?: RuleNoConfig;
  "nextjs/no-duplicate-head"?: RuleNoConfig;
  "nextjs/no-head-element"?: RuleNoConfig;
  "nextjs/no-head-import-in-document"?: RuleNoConfig;
  "nextjs/no-html-link-for-pages"?: RuleNoConfig;
  "nextjs/no-img-element"?: RuleNoConfig;
  "nextjs/no-page-custom-font"?: RuleNoConfig;
  "nextjs/no-script-component-in-head"?: RuleNoConfig;
  "nextjs/no-styled-jsx-in-document"?: RuleNoConfig;
  "nextjs/no-sync-scripts"?: RuleNoConfig;
  "nextjs/no-title-in-document-head"?: RuleNoConfig;
  "nextjs/no-typos"?: RuleNoConfig;
  "nextjs/no-unwanted-polyfillio"?: RuleNoConfig;
  "no-alert"?: RuleNoConfig;
  "no-array-constructor"?: RuleNoConfig;
  "no-async-promise-executor"?: RuleNoConfig;
  "no-await-in-loop"?: RuleNoConfig;
  "no-bitwise"?: DummyRule;
  "no-caller"?: RuleNoConfig;
  "no-case-declarations"?: RuleNoConfig;
  "no-class-assign"?: RuleNoConfig;
  "no-compare-neg-zero"?: RuleNoConfig;
  "no-cond-assign"?: DummyRule;
  "no-console"?: DummyRule;
  "no-const-assign"?: RuleNoConfig;
  "no-constant-binary-expression"?: RuleNoConfig;
  "no-constant-condition"?: DummyRule;
  "no-constructor-return"?: RuleNoConfig;
  "no-continue"?: RuleNoConfig;
  "no-control-regex"?: RuleNoConfig;
  "no-debugger"?: RuleNoConfig;
  "no-delete-var"?: RuleNoConfig;
  "no-div-regex"?: RuleNoConfig;
  "no-dupe-class-members"?: RuleNoConfig;
  "no-dupe-else-if"?: RuleNoConfig;
  "no-dupe-keys"?: RuleNoConfig;
  "no-duplicate-case"?: RuleNoConfig;
  "no-duplicate-imports"?: DummyRule;
  "no-else-return"?: DummyRule;
  "no-empty"?: DummyRule;
  "no-empty-character-class"?: RuleNoConfig;
  "no-empty-function"?: DummyRule;
  "no-empty-pattern"?: DummyRule;
  "no-empty-static-block"?: RuleNoConfig;
  "no-eq-null"?: RuleNoConfig;
  "no-eval"?: DummyRule;
  "no-ex-assign"?: RuleNoConfig;
  "no-extend-native"?: DummyRule;
  "no-extra-bind"?: RuleNoConfig;
  "no-extra-boolean-cast"?: DummyRule;
  "no-extra-label"?: RuleNoConfig;
  "no-fallthrough"?: DummyRule;
  "no-func-assign"?: RuleNoConfig;
  "no-global-assign"?: DummyRule;
  "no-implicit-coercion"?: DummyRule;
  "no-implicit-globals"?: DummyRule;
  "no-implied-eval"?: RuleNoConfig;
  "no-import-assign"?: RuleNoConfig;
  "no-inline-comments"?: DummyRule;
  "no-inner-declarations"?: DummyRule;
  "no-invalid-regexp"?: DummyRule;
  "no-irregular-whitespace"?: DummyRule;
  "no-iterator"?: RuleNoConfig;
  "no-label-var"?: RuleNoConfig;
  "no-labels"?: DummyRule;
  "no-lone-blocks"?: RuleNoConfig;
  "no-lonely-if"?: RuleNoConfig;
  "no-loop-func"?: RuleNoConfig;
  "no-loss-of-precision"?: RuleNoConfig;
  "no-magic-numbers"?: DummyRule;
  "no-misleading-character-class"?: DummyRule;
  "no-multi-assign"?: DummyRule;
  "no-multi-str"?: RuleNoConfig;
  "no-negated-condition"?: RuleNoConfig;
  "no-nested-ternary"?: RuleNoConfig;
  "no-new"?: RuleNoConfig;
  "no-new-func"?: RuleNoConfig;
  "no-new-native-nonconstructor"?: RuleNoConfig;
  "no-new-wrappers"?: RuleNoConfig;
  "no-nonoctal-decimal-escape"?: RuleNoConfig;
  "no-obj-calls"?: RuleNoConfig;
  "no-object-constructor"?: RuleNoConfig;
  "no-param-reassign"?: DummyRule;
  "no-plusplus"?: DummyRule;
  "no-promise-executor-return"?: DummyRule;
  "no-proto"?: RuleNoConfig;
  "no-prototype-builtins"?: RuleNoConfig;
  "no-redeclare"?: DummyRule;
  "no-regex-spaces"?: RuleNoConfig;
  "no-restricted-exports"?: DummyRule;
  "no-restricted-globals"?: DummyRule;
  "no-restricted-imports"?: DummyRule;
  "no-restricted-properties"?: DummyRule;
  "no-return-assign"?: DummyRule;
  "no-script-url"?: RuleNoConfig;
  "no-self-assign"?: DummyRule;
  "no-self-compare"?: RuleNoConfig;
  "no-sequences"?: DummyRule;
  "no-setter-return"?: RuleNoConfig;
  "no-shadow"?: DummyRule;
  "no-shadow-restricted-names"?: DummyRule;
  "no-sparse-arrays"?: RuleNoConfig;
  "no-template-curly-in-string"?: RuleNoConfig;
  "no-ternary"?: RuleNoConfig;
  "no-this-before-super"?: RuleNoConfig;
  "no-throw-literal"?: RuleNoConfig;
  "no-unassigned-vars"?: RuleNoConfig;
  "no-undef"?: DummyRule;
  "no-undefined"?: RuleNoConfig;
  "no-underscore-dangle"?: DummyRule;
  "no-unexpected-multiline"?: RuleNoConfig;
  "no-unmodified-loop-condition"?: RuleNoConfig;
  "no-unneeded-ternary"?: DummyRule;
  "no-unreachable"?: RuleNoConfig;
  "no-unsafe-finally"?: RuleNoConfig;
  "no-unsafe-negation"?: DummyRule;
  "no-unsafe-optional-chaining"?: DummyRule;
  "no-unused-expressions"?: DummyRule;
  "no-unused-labels"?: RuleNoConfig;
  "no-unused-private-class-members"?: RuleNoConfig;
  "no-unused-vars"?: DummyRule;
  "no-use-before-define"?: DummyRule;
  "no-useless-assignment"?: RuleNoConfig;
  "no-useless-backreference"?: RuleNoConfig;
  "no-useless-call"?: RuleNoConfig;
  "no-useless-catch"?: RuleNoConfig;
  "no-useless-computed-key"?: DummyRule;
  "no-useless-concat"?: RuleNoConfig;
  "no-useless-constructor"?: RuleNoConfig;
  "no-useless-escape"?: DummyRule;
  "no-useless-rename"?: DummyRule;
  "no-useless-return"?: RuleNoConfig;
  "no-var"?: RuleNoConfig;
  "no-void"?: DummyRule;
  "no-warning-comments"?: DummyRule;
  "no-with"?: RuleNoConfig;
  "node/callback-return"?: DummyRule;
  "node/global-require"?: RuleNoConfig;
  "node/handle-callback-err"?: DummyRule;
  "node/no-exports-assign"?: RuleNoConfig;
  "node/no-new-require"?: RuleNoConfig;
  "node/no-path-concat"?: RuleNoConfig;
  "node/no-process-env"?: DummyRule;
  "object-shorthand"?: DummyRule;
  "operator-assignment"?: DummyRule;
  "oxc/approx-constant"?: RuleNoConfig;
  "oxc/bad-array-method-on-arguments"?: RuleNoConfig;
  "oxc/bad-bitwise-operator"?: RuleNoConfig;
  "oxc/bad-char-at-comparison"?: RuleNoConfig;
  "oxc/bad-comparison-sequence"?: RuleNoConfig;
  "oxc/bad-min-max-func"?: RuleNoConfig;
  "oxc/bad-object-literal-comparison"?: RuleNoConfig;
  "oxc/bad-replace-all-arg"?: RuleNoConfig;
  "oxc/branches-sharing-code"?: RuleNoConfig;
  "oxc/const-comparisons"?: RuleNoConfig;
  "oxc/double-comparisons"?: RuleNoConfig;
  "oxc/erasing-op"?: RuleNoConfig;
  "oxc/misrefactored-assign-op"?: RuleNoConfig;
  "oxc/missing-throw"?: RuleNoConfig;
  "oxc/no-accumulating-spread"?: RuleNoConfig;
  "oxc/no-async-await"?: RuleNoConfig;
  "oxc/no-async-endpoint-handlers"?: DummyRule;
  "oxc/no-barrel-file"?: DummyRule;
  "oxc/no-const-enum"?: RuleNoConfig;
  "oxc/no-map-spread"?: DummyRule;
  "oxc/no-optional-chaining"?: DummyRule;
  "oxc/no-rest-spread-properties"?: DummyRule;
  "oxc/no-this-in-exported-function"?: RuleNoConfig;
  "oxc/number-arg-out-of-range"?: RuleNoConfig;
  "oxc/only-used-in-recursion"?: RuleNoConfig;
  "oxc/uninvoked-array-callback"?: RuleNoConfig;
  "prefer-arrow-callback"?: DummyRule;
  "prefer-const"?: DummyRule;
  "prefer-destructuring"?: DummyRule;
  "prefer-exponentiation-operator"?: RuleNoConfig;
  "prefer-named-capture-group"?: RuleNoConfig;
  "prefer-numeric-literals"?: RuleNoConfig;
  "prefer-object-has-own"?: RuleNoConfig;
  "prefer-object-spread"?: RuleNoConfig;
  "prefer-promise-reject-errors"?: DummyRule;
  "prefer-regex-literals"?: DummyRule;
  "prefer-rest-params"?: RuleNoConfig;
  "prefer-spread"?: RuleNoConfig;
  "prefer-template"?: RuleNoConfig;
  "preserve-caught-error"?: DummyRule;
  "promise/always-return"?: DummyRule;
  "promise/avoid-new"?: RuleNoConfig;
  "promise/catch-or-return"?: DummyRule;
  "promise/no-callback-in-promise"?: DummyRule;
  "promise/no-multiple-resolved"?: RuleNoConfig;
  "promise/no-nesting"?: RuleNoConfig;
  "promise/no-new-statics"?: RuleNoConfig;
  "promise/no-promise-in-callback"?: DummyRule;
  "promise/no-return-in-finally"?: RuleNoConfig;
  "promise/no-return-wrap"?: DummyRule;
  "promise/param-names"?: DummyRule;
  "promise/prefer-await-to-callbacks"?: RuleNoConfig;
  "promise/prefer-await-to-then"?: DummyRule;
  "promise/prefer-catch"?: RuleNoConfig;
  "promise/spec-only"?: DummyRule;
  "promise/valid-params"?: RuleNoConfig;
  radix?: DummyRule;
  "react-perf/jsx-no-jsx-as-prop"?: RuleNoConfig;
  "react-perf/jsx-no-new-array-as-prop"?: RuleNoConfig;
  "react-perf/jsx-no-new-function-as-prop"?: RuleNoConfig;
  "react-perf/jsx-no-new-object-as-prop"?: DummyRule;
  "react/button-has-type"?: DummyRule;
  "react/checked-requires-onchange-or-readonly"?: DummyRule;
  "react/display-name"?: DummyRule;
  "react/exhaustive-deps"?: DummyRule;
  "react/forbid-component-props"?: DummyRule;
  "react/forbid-dom-props"?: DummyRule;
  "react/forbid-elements"?: DummyRule;
  "react/forward-ref-uses-ref"?: RuleNoConfig;
  "react/hook-use-state"?: DummyRule;
  "react/iframe-missing-sandbox"?: RuleNoConfig;
  "react/jsx-boolean-value"?: DummyRule;
  "react/jsx-curly-brace-presence"?: DummyRule;
  "react/jsx-filename-extension"?: DummyRule;
  "react/jsx-fragments"?: DummyRule;
  "react/jsx-handler-names"?: DummyRule;
  "react/jsx-key"?: DummyRule;
  "react/jsx-max-depth"?: DummyRule;
  "react/jsx-no-comment-textnodes"?: RuleNoConfig;
  "react/jsx-no-constructed-context-values"?: RuleNoConfig;
  "react/jsx-no-duplicate-props"?: RuleNoConfig;
  "react/jsx-no-script-url"?: DummyRule;
  "react/jsx-no-target-blank"?: DummyRule;
  "react/jsx-no-undef"?: RuleNoConfig;
  "react/jsx-no-useless-fragment"?: DummyRule;
  "react/jsx-pascal-case"?: DummyRule;
  "react/jsx-props-no-spread-multi"?: RuleNoConfig;
  "react/jsx-props-no-spreading"?: DummyRule;
  "react/no-array-index-key"?: RuleNoConfig;
  "react/no-children-prop"?: RuleNoConfig;
  "react/no-clone-element"?: RuleNoConfig;
  "react/no-danger"?: RuleNoConfig;
  "react/no-danger-with-children"?: RuleNoConfig;
  "react/no-did-mount-set-state"?: DummyRule;
  "react/no-did-update-set-state"?: DummyRule;
  "react/no-direct-mutation-state"?: RuleNoConfig;
  "react/no-find-dom-node"?: RuleNoConfig;
  "react/no-is-mounted"?: RuleNoConfig;
  "react/no-multi-comp"?: DummyRule;
  "react/no-namespace"?: RuleNoConfig;
  "react/no-object-type-as-default-prop"?: DummyRule;
  "react/no-react-children"?: RuleNoConfig;
  "react/no-redundant-should-component-update"?: RuleNoConfig;
  "react/no-render-return-value"?: RuleNoConfig;
  "react/no-set-state"?: RuleNoConfig;
  "react/no-string-refs"?: DummyRule;
  "react/no-this-in-sfc"?: RuleNoConfig;
  "react/no-unescaped-entities"?: RuleNoConfig;
  "react/no-unknown-property"?: DummyRule;
  "react/no-unsafe"?: DummyRule;
  "react/no-unstable-nested-components"?: DummyRule;
  "react/no-will-update-set-state"?: DummyRule;
  "react/only-export-components"?: DummyRule;
  "react/prefer-es6-class"?: DummyRule;
  "react/prefer-function-component"?: DummyRule;
  "react/react-in-jsx-scope"?: RuleNoConfig;
  "react/require-render-return"?: RuleNoConfig;
  "react/rules-of-hooks"?: RuleNoConfig;
  "react/self-closing-comp"?: DummyRule;
  "react/state-in-constructor"?: DummyRule;
  "react/style-prop-object"?: DummyRule;
  "react/void-dom-elements-no-children"?: RuleNoConfig;
  "require-await"?: RuleNoConfig;
  "require-unicode-regexp"?: DummyRule;
  "require-yield"?: RuleNoConfig;
  "sort-imports"?: DummyRule;
  "sort-keys"?: DummyRule;
  "sort-vars"?: DummyRule;
  "symbol-description"?: RuleNoConfig;
  "typescript/adjacent-overload-signatures"?: RuleNoConfig;
  "typescript/array-type"?: DummyRule;
  "typescript/await-thenable"?: RuleNoConfig;
  "typescript/ban-ts-comment"?: DummyRule;
  "typescript/ban-tslint-comment"?: RuleNoConfig;
  "typescript/ban-types"?: RuleNoConfig;
  "typescript/class-literal-property-style"?: DummyRule;
  "typescript/consistent-generic-constructors"?: DummyRule;
  "typescript/consistent-indexed-object-style"?: DummyRule;
  "typescript/consistent-return"?: DummyRule;
  "typescript/consistent-type-assertions"?: DummyRule;
  "typescript/consistent-type-definitions"?: DummyRule;
  "typescript/consistent-type-exports"?: DummyRule;
  "typescript/consistent-type-imports"?: DummyRule;
  "typescript/dot-notation"?: DummyRule;
  "typescript/explicit-function-return-type"?: DummyRule;
  "typescript/explicit-member-accessibility"?: DummyRule;
  "typescript/explicit-module-boundary-types"?: DummyRule;
  "typescript/method-signature-style"?: DummyRule;
  "typescript/no-array-delete"?: RuleNoConfig;
  "typescript/no-base-to-string"?: DummyRule;
  "typescript/no-confusing-non-null-assertion"?: RuleNoConfig;
  "typescript/no-confusing-void-expression"?: DummyRule;
  "typescript/no-deprecated"?: DummyRule;
  "typescript/no-duplicate-enum-values"?: RuleNoConfig;
  "typescript/no-duplicate-type-constituents"?: DummyRule;
  "typescript/no-dynamic-delete"?: RuleNoConfig;
  "typescript/no-empty-interface"?: DummyRule;
  "typescript/no-empty-object-type"?: DummyRule;
  "typescript/no-explicit-any"?: DummyRule;
  "typescript/no-extra-non-null-assertion"?: RuleNoConfig;
  "typescript/no-extraneous-class"?: DummyRule;
  "typescript/no-floating-promises"?: DummyRule;
  "typescript/no-for-in-array"?: RuleNoConfig;
  "typescript/no-implied-eval"?: RuleNoConfig;
  "typescript/no-import-type-side-effects"?: RuleNoConfig;
  "typescript/no-inferrable-types"?: DummyRule;
  "typescript/no-invalid-void-type"?: DummyRule;
  "typescript/no-meaningless-void-operator"?: DummyRule;
  "typescript/no-misused-new"?: RuleNoConfig;
  "typescript/no-misused-promises"?: DummyRule;
  "typescript/no-misused-spread"?: DummyRule;
  "typescript/no-mixed-enums"?: RuleNoConfig;
  "typescript/no-namespace"?: DummyRule;
  "typescript/no-non-null-asserted-nullish-coalescing"?: RuleNoConfig;
  "typescript/no-non-null-asserted-optional-chain"?: RuleNoConfig;
  "typescript/no-non-null-assertion"?: RuleNoConfig;
  "typescript/no-redundant-type-constituents"?: RuleNoConfig;
  "typescript/no-require-imports"?: DummyRule;
  "typescript/no-restricted-types"?: DummyRule;
  "typescript/no-this-alias"?: DummyRule;
  "typescript/no-unnecessary-boolean-literal-compare"?: DummyRule;
  "typescript/no-unnecessary-condition"?: DummyRule;
  "typescript/no-unnecessary-parameter-property-assignment"?: RuleNoConfig;
  "typescript/no-unnecessary-qualifier"?: RuleNoConfig;
  "typescript/no-unnecessary-template-expression"?: RuleNoConfig;
  "typescript/no-unnecessary-type-arguments"?: RuleNoConfig;
  "typescript/no-unnecessary-type-assertion"?: DummyRule;
  "typescript/no-unnecessary-type-constraint"?: RuleNoConfig;
  "typescript/no-unnecessary-type-conversion"?: RuleNoConfig;
  "typescript/no-unnecessary-type-parameters"?: RuleNoConfig;
  "typescript/no-unsafe-argument"?: RuleNoConfig;
  "typescript/no-unsafe-assignment"?: RuleNoConfig;
  "typescript/no-unsafe-call"?: RuleNoConfig;
  "typescript/no-unsafe-declaration-merging"?: RuleNoConfig;
  "typescript/no-unsafe-enum-comparison"?: RuleNoConfig;
  "typescript/no-unsafe-function-type"?: RuleNoConfig;
  "typescript/no-unsafe-member-access"?: DummyRule;
  "typescript/no-unsafe-return"?: RuleNoConfig;
  "typescript/no-unsafe-type-assertion"?: RuleNoConfig;
  "typescript/no-unsafe-unary-minus"?: RuleNoConfig;
  "typescript/no-useless-default-assignment"?: RuleNoConfig;
  "typescript/no-useless-empty-export"?: RuleNoConfig;
  "typescript/no-var-requires"?: RuleNoConfig;
  "typescript/no-wrapper-object-types"?: RuleNoConfig;
  "typescript/non-nullable-type-assertion-style"?: RuleNoConfig;
  "typescript/only-throw-error"?: DummyRule;
  "typescript/parameter-properties"?: DummyRule;
  "typescript/prefer-as-const"?: RuleNoConfig;
  "typescript/prefer-enum-initializers"?: RuleNoConfig;
  "typescript/prefer-find"?: RuleNoConfig;
  "typescript/prefer-for-of"?: RuleNoConfig;
  "typescript/prefer-function-type"?: RuleNoConfig;
  "typescript/prefer-includes"?: RuleNoConfig;
  "typescript/prefer-literal-enum-member"?: DummyRule;
  "typescript/prefer-namespace-keyword"?: RuleNoConfig;
  "typescript/prefer-nullish-coalescing"?: DummyRule;
  "typescript/prefer-optional-chain"?: DummyRule;
  "typescript/prefer-promise-reject-errors"?: DummyRule;
  "typescript/prefer-readonly"?: DummyRule;
  "typescript/prefer-readonly-parameter-types"?: DummyRule;
  "typescript/prefer-reduce-type-parameter"?: RuleNoConfig;
  "typescript/prefer-regexp-exec"?: RuleNoConfig;
  "typescript/prefer-return-this-type"?: RuleNoConfig;
  "typescript/prefer-string-starts-ends-with"?: DummyRule;
  "typescript/prefer-ts-expect-error"?: RuleNoConfig;
  "typescript/promise-function-async"?: DummyRule;
  "typescript/related-getter-setter-pairs"?: RuleNoConfig;
  "typescript/require-array-sort-compare"?: DummyRule;
  "typescript/require-await"?: RuleNoConfig;
  "typescript/restrict-plus-operands"?: DummyRule;
  "typescript/restrict-template-expressions"?: DummyRule;
  "typescript/return-await"?: DummyRule;
  "typescript/strict-boolean-expressions"?: DummyRule;
  "typescript/strict-void-return"?: DummyRule;
  "typescript/switch-exhaustiveness-check"?: DummyRule;
  "typescript/triple-slash-reference"?: DummyRule;
  "typescript/unbound-method"?: DummyRule;
  "typescript/unified-signatures"?: DummyRule;
  "typescript/use-unknown-in-catch-callback-variable"?: RuleNoConfig;
  "unicode-bom"?: DummyRule;
  "unicorn/catch-error-name"?: DummyRule;
  "unicorn/consistent-assert"?: RuleNoConfig;
  "unicorn/consistent-date-clone"?: RuleNoConfig;
  "unicorn/consistent-empty-array-spread"?: RuleNoConfig;
  "unicorn/consistent-existence-index-check"?: RuleNoConfig;
  "unicorn/consistent-function-scoping"?: DummyRule;
  "unicorn/consistent-template-literal-escape"?: RuleNoConfig;
  "unicorn/custom-error-definition"?: RuleNoConfig;
  "unicorn/empty-brace-spaces"?: RuleNoConfig;
  "unicorn/error-message"?: RuleNoConfig;
  "unicorn/escape-case"?: RuleNoConfig;
  "unicorn/explicit-length-check"?: DummyRule;
  "unicorn/filename-case"?: DummyRule;
  "unicorn/import-style"?: DummyRule;
  "unicorn/new-for-builtins"?: RuleNoConfig;
  "unicorn/no-abusive-eslint-disable"?: RuleNoConfig;
  "unicorn/no-accessor-recursion"?: RuleNoConfig;
  "unicorn/no-anonymous-default-export"?: RuleNoConfig;
  "unicorn/no-array-callback-reference"?: RuleNoConfig;
  "unicorn/no-array-for-each"?: RuleNoConfig;
  "unicorn/no-array-method-this-argument"?: RuleNoConfig;
  "unicorn/no-array-reduce"?: DummyRule;
  "unicorn/no-array-reverse"?: DummyRule;
  "unicorn/no-array-sort"?: DummyRule;
  "unicorn/no-await-expression-member"?: RuleNoConfig;
  "unicorn/no-await-in-promise-methods"?: RuleNoConfig;
  "unicorn/no-console-spaces"?: RuleNoConfig;
  "unicorn/no-document-cookie"?: RuleNoConfig;
  "unicorn/no-empty-file"?: RuleNoConfig;
  "unicorn/no-hex-escape"?: RuleNoConfig;
  "unicorn/no-immediate-mutation"?: RuleNoConfig;
  "unicorn/no-instanceof-array"?: RuleNoConfig;
  "unicorn/no-instanceof-builtins"?: DummyRule;
  "unicorn/no-invalid-fetch-options"?: RuleNoConfig;
  "unicorn/no-invalid-remove-event-listener"?: RuleNoConfig;
  "unicorn/no-length-as-slice-end"?: RuleNoConfig;
  "unicorn/no-lonely-if"?: RuleNoConfig;
  "unicorn/no-magic-array-flat-depth"?: RuleNoConfig;
  "unicorn/no-negated-condition"?: RuleNoConfig;
  "unicorn/no-negation-in-equality-check"?: RuleNoConfig;
  "unicorn/no-nested-ternary"?: RuleNoConfig;
  "unicorn/no-new-array"?: RuleNoConfig;
  "unicorn/no-new-buffer"?: RuleNoConfig;
  "unicorn/no-null"?: DummyRule;
  "unicorn/no-object-as-default-parameter"?: RuleNoConfig;
  "unicorn/no-process-exit"?: RuleNoConfig;
  "unicorn/no-single-promise-in-promise-methods"?: RuleNoConfig;
  "unicorn/no-static-only-class"?: RuleNoConfig;
  "unicorn/no-thenable"?: RuleNoConfig;
  "unicorn/no-this-assignment"?: RuleNoConfig;
  "unicorn/no-typeof-undefined"?: DummyRule;
  "unicorn/no-unnecessary-array-flat-depth"?: RuleNoConfig;
  "unicorn/no-unnecessary-array-splice-count"?: RuleNoConfig;
  "unicorn/no-unnecessary-await"?: RuleNoConfig;
  "unicorn/no-unnecessary-slice-end"?: RuleNoConfig;
  "unicorn/no-unreadable-array-destructuring"?: RuleNoConfig;
  "unicorn/no-unreadable-iife"?: RuleNoConfig;
  "unicorn/no-useless-collection-argument"?: RuleNoConfig;
  "unicorn/no-useless-error-capture-stack-trace"?: RuleNoConfig;
  "unicorn/no-useless-fallback-in-spread"?: RuleNoConfig;
  "unicorn/no-useless-iterator-to-array"?: RuleNoConfig;
  "unicorn/no-useless-length-check"?: RuleNoConfig;
  "unicorn/no-useless-promise-resolve-reject"?: DummyRule;
  "unicorn/no-useless-spread"?: RuleNoConfig;
  "unicorn/no-useless-switch-case"?: RuleNoConfig;
  "unicorn/no-useless-undefined"?: DummyRule;
  "unicorn/no-zero-fractions"?: RuleNoConfig;
  "unicorn/number-literal-case"?: RuleNoConfig;
  "unicorn/numeric-separators-style"?: DummyRule;
  "unicorn/prefer-add-event-listener"?: RuleNoConfig;
  "unicorn/prefer-array-find"?: RuleNoConfig;
  "unicorn/prefer-array-flat"?: RuleNoConfig;
  "unicorn/prefer-array-flat-map"?: RuleNoConfig;
  "unicorn/prefer-array-index-of"?: RuleNoConfig;
  "unicorn/prefer-array-some"?: RuleNoConfig;
  "unicorn/prefer-at"?: DummyRule;
  "unicorn/prefer-bigint-literals"?: RuleNoConfig;
  "unicorn/prefer-blob-reading-methods"?: RuleNoConfig;
  "unicorn/prefer-class-fields"?: RuleNoConfig;
  "unicorn/prefer-classlist-toggle"?: RuleNoConfig;
  "unicorn/prefer-code-point"?: RuleNoConfig;
  "unicorn/prefer-date-now"?: RuleNoConfig;
  "unicorn/prefer-default-parameters"?: RuleNoConfig;
  "unicorn/prefer-dom-node-append"?: RuleNoConfig;
  "unicorn/prefer-dom-node-dataset"?: RuleNoConfig;
  "unicorn/prefer-dom-node-remove"?: RuleNoConfig;
  "unicorn/prefer-dom-node-text-content"?: RuleNoConfig;
  "unicorn/prefer-event-target"?: RuleNoConfig;
  "unicorn/prefer-global-this"?: RuleNoConfig;
  "unicorn/prefer-import-meta-properties"?: RuleNoConfig;
  "unicorn/prefer-includes"?: RuleNoConfig;
  "unicorn/prefer-keyboard-event-key"?: RuleNoConfig;
  "unicorn/prefer-logical-operator-over-ternary"?: RuleNoConfig;
  "unicorn/prefer-math-min-max"?: RuleNoConfig;
  "unicorn/prefer-math-trunc"?: RuleNoConfig;
  "unicorn/prefer-modern-dom-apis"?: RuleNoConfig;
  "unicorn/prefer-modern-math-apis"?: RuleNoConfig;
  "unicorn/prefer-module"?: RuleNoConfig;
  "unicorn/prefer-native-coercion-functions"?: RuleNoConfig;
  "unicorn/prefer-negative-index"?: RuleNoConfig;
  "unicorn/prefer-node-protocol"?: RuleNoConfig;
  "unicorn/prefer-number-properties"?: DummyRule;
  "unicorn/prefer-object-from-entries"?: DummyRule;
  "unicorn/prefer-optional-catch-binding"?: RuleNoConfig;
  "unicorn/prefer-prototype-methods"?: RuleNoConfig;
  "unicorn/prefer-query-selector"?: RuleNoConfig;
  "unicorn/prefer-reflect-apply"?: RuleNoConfig;
  "unicorn/prefer-regexp-test"?: RuleNoConfig;
  "unicorn/prefer-response-static-json"?: RuleNoConfig;
  "unicorn/prefer-set-has"?: RuleNoConfig;
  "unicorn/prefer-set-size"?: RuleNoConfig;
  "unicorn/prefer-spread"?: RuleNoConfig;
  "unicorn/prefer-string-raw"?: RuleNoConfig;
  "unicorn/prefer-string-replace-all"?: RuleNoConfig;
  "unicorn/prefer-string-slice"?: RuleNoConfig;
  "unicorn/prefer-string-starts-ends-with"?: RuleNoConfig;
  "unicorn/prefer-string-trim-start-end"?: RuleNoConfig;
  "unicorn/prefer-structured-clone"?: DummyRule;
  "unicorn/prefer-ternary"?: DummyRule;
  "unicorn/prefer-top-level-await"?: RuleNoConfig;
  "unicorn/prefer-type-error"?: RuleNoConfig;
  "unicorn/relative-url-style"?: DummyRule;
  "unicorn/require-array-join-separator"?: RuleNoConfig;
  "unicorn/require-module-attributes"?: RuleNoConfig;
  "unicorn/require-module-specifiers"?: RuleNoConfig;
  "unicorn/require-number-to-fixed-digits-argument"?: RuleNoConfig;
  "unicorn/require-post-message-target-origin"?: RuleNoConfig;
  "unicorn/switch-case-braces"?: DummyRule;
  "unicorn/switch-case-break-position"?: RuleNoConfig;
  "unicorn/text-encoding-identifier-case"?: DummyRule;
  "unicorn/throw-new-error"?: RuleNoConfig;
  "use-isnan"?: DummyRule;
  "valid-typeof"?: DummyRule;
  "vars-on-top"?: RuleNoConfig;
  "vitest/consistent-each-for"?: DummyRule;
  "vitest/consistent-test-filename"?: DummyRule;
  "vitest/consistent-test-it"?: DummyRule;
  "vitest/consistent-vitest-vi"?: DummyRule;
  "vitest/expect-expect"?: DummyRule;
  "vitest/hoisted-apis-on-top"?: RuleNoConfig;
  "vitest/max-expects"?: DummyRule;
  "vitest/max-nested-describe"?: DummyRule;
  "vitest/no-alias-methods"?: RuleNoConfig;
  "vitest/no-commented-out-tests"?: RuleNoConfig;
  "vitest/no-conditional-expect"?: RuleNoConfig;
  "vitest/no-conditional-in-test"?: RuleNoConfig;
  "vitest/no-conditional-tests"?: RuleNoConfig;
  "vitest/no-disabled-tests"?: RuleNoConfig;
  "vitest/no-duplicate-hooks"?: RuleNoConfig;
  "vitest/no-focused-tests"?: RuleNoConfig;
  "vitest/no-hooks"?: DummyRule;
  "vitest/no-identical-title"?: RuleNoConfig;
  "vitest/no-import-node-test"?: RuleNoConfig;
  "vitest/no-importing-vitest-globals"?: RuleNoConfig;
  "vitest/no-interpolation-in-snapshots"?: RuleNoConfig;
  "vitest/no-large-snapshots"?: DummyRule;
  "vitest/no-mocks-import"?: RuleNoConfig;
  "vitest/no-restricted-matchers"?: DummyRule;
  "vitest/no-restricted-vi-methods"?: DummyRule;
  "vitest/no-standalone-expect"?: DummyRule;
  "vitest/no-test-prefixes"?: RuleNoConfig;
  "vitest/no-test-return-statement"?: RuleNoConfig;
  "vitest/no-unneeded-async-expect-function"?: RuleNoConfig;
  "vitest/padding-around-after-all-blocks"?: RuleNoConfig;
  "vitest/prefer-called-exactly-once-with"?: RuleNoConfig;
  "vitest/prefer-called-once"?: RuleNoConfig;
  "vitest/prefer-called-times"?: RuleNoConfig;
  "vitest/prefer-called-with"?: RuleNoConfig;
  "vitest/prefer-comparison-matcher"?: RuleNoConfig;
  "vitest/prefer-describe-function-title"?: RuleNoConfig;
  "vitest/prefer-each"?: RuleNoConfig;
  "vitest/prefer-equality-matcher"?: RuleNoConfig;
  "vitest/prefer-expect-assertions"?: DummyRule;
  "vitest/prefer-expect-resolves"?: RuleNoConfig;
  "vitest/prefer-expect-type-of"?: RuleNoConfig;
  "vitest/prefer-hooks-in-order"?: RuleNoConfig;
  "vitest/prefer-hooks-on-top"?: RuleNoConfig;
  "vitest/prefer-import-in-mock"?: DummyRule;
  "vitest/prefer-importing-vitest-globals"?: RuleNoConfig;
  "vitest/prefer-lowercase-title"?: DummyRule;
  "vitest/prefer-mock-promise-shorthand"?: RuleNoConfig;
  "vitest/prefer-mock-return-shorthand"?: RuleNoConfig;
  "vitest/prefer-snapshot-hint"?: DummyRule;
  "vitest/prefer-spy-on"?: RuleNoConfig;
  "vitest/prefer-strict-boolean-matchers"?: RuleNoConfig;
  "vitest/prefer-strict-equal"?: RuleNoConfig;
  "vitest/prefer-to-be"?: RuleNoConfig;
  "vitest/prefer-to-be-falsy"?: RuleNoConfig;
  "vitest/prefer-to-be-object"?: RuleNoConfig;
  "vitest/prefer-to-be-truthy"?: RuleNoConfig;
  "vitest/prefer-to-contain"?: RuleNoConfig;
  "vitest/prefer-to-have-been-called-times"?: RuleNoConfig;
  "vitest/prefer-to-have-length"?: RuleNoConfig;
  "vitest/prefer-todo"?: RuleNoConfig;
  "vitest/require-awaited-expect-poll"?: RuleNoConfig;
  "vitest/require-hook"?: DummyRule;
  "vitest/require-local-test-context-for-concurrent-snapshots"?: RuleNoConfig;
  "vitest/require-mock-type-parameters"?: DummyRule;
  "vitest/require-test-timeout"?: RuleNoConfig;
  "vitest/require-to-throw-message"?: RuleNoConfig;
  "vitest/require-top-level-describe"?: DummyRule;
  "vitest/valid-describe-callback"?: RuleNoConfig;
  "vitest/valid-expect"?: DummyRule;
  "vitest/valid-expect-in-promise"?: RuleNoConfig;
  "vitest/valid-title"?: DummyRule;
  "vitest/warn-todo"?: RuleNoConfig;
  "vue/component-definition-name-casing"?: DummyRule;
  "vue/define-emits-declaration"?: DummyRule;
  "vue/define-props-declaration"?: DummyRule;
  "vue/define-props-destructuring"?: DummyRule;
  "vue/max-props"?: DummyRule;
  "vue/no-arrow-functions-in-watch"?: RuleNoConfig;
  "vue/no-computed-properties-in-data"?: RuleNoConfig;
  "vue/no-deprecated-data-object-declaration"?: RuleNoConfig;
  "vue/no-deprecated-delete-set"?: RuleNoConfig;
  "vue/no-deprecated-destroyed-lifecycle"?: RuleNoConfig;
  "vue/no-deprecated-events-api"?: RuleNoConfig;
  "vue/no-deprecated-model-definition"?: DummyRule;
  "vue/no-deprecated-props-default-this"?: RuleNoConfig;
  "vue/no-deprecated-vue-config-keycodes"?: RuleNoConfig;
  "vue/no-export-in-script-setup"?: RuleNoConfig;
  "vue/no-expose-after-await"?: RuleNoConfig;
  "vue/no-import-compiler-macros"?: RuleNoConfig;
  "vue/no-lifecycle-after-await"?: RuleNoConfig;
  "vue/no-multiple-slot-args"?: RuleNoConfig;
  "vue/no-required-prop-with-default"?: RuleNoConfig;
  "vue/no-reserved-component-names"?: DummyRule;
  "vue/no-reserved-keys"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, NoReservedKeysConfig];
  "vue/no-reserved-props"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, NoReservedPropsConfig];
  "vue/no-shared-component-data"?: RuleNoConfig;
  "vue/no-this-in-before-route-enter"?: RuleNoConfig;
  "vue/no-watch-after-await"?: RuleNoConfig;
  "vue/prefer-import-from-vue"?: RuleNoConfig;
  "vue/prop-name-casing"?:
    | AllowWarnDeny
    | [AllowWarnDeny]
    | [AllowWarnDeny, CaseType]
    | [AllowWarnDeny, CaseType, Options];
  "vue/require-default-export"?: RuleNoConfig;
  "vue/require-direct-export"?: AllowWarnDeny | [AllowWarnDeny] | [AllowWarnDeny, RequireDirectExport];
  "vue/require-prop-type-constructor"?: RuleNoConfig;
  "vue/require-prop-types"?: RuleNoConfig;
  "vue/require-render-return"?: RuleNoConfig;
  "vue/require-slots-as-functions"?: RuleNoConfig;
  "vue/require-typed-ref"?: RuleNoConfig;
  "vue/return-in-computed-property"?: DummyRule;
  "vue/return-in-emits-validator"?: RuleNoConfig;
  "vue/valid-define-emits"?: RuleNoConfig;
  "vue/valid-define-options"?: RuleNoConfig;
  "vue/valid-define-props"?: RuleNoConfig;
  "vue/valid-next-tick"?: RuleNoConfig;
  yoda?: DummyRule;
  [k: string]: DummyRule | undefined;
}
export interface MaxClassesPerFileConfig {
  /**
   * Whether to ignore class expressions when counting classes.
   */
  ignoreExpressions?: boolean;
  /**
   * The maximum number of classes allowed per file.
   */
  max?: number;
}
export interface MaxDepth {
  /**
   * The `max` enforces a maximum depth that blocks can be nested
   */
  max?: number;
}
export interface MaxLinesConfig {
  /**
   * Maximum number of lines allowed per file.
   */
  max?: number;
  /**
   * Whether to ignore blank lines when counting.
   */
  skipBlankLines?: boolean;
  /**
   * Whether to ignore comments when counting.
   */
  skipComments?: boolean;
}
export interface MaxLinesPerFunctionConfig {
  /**
   * The `IIFEs` option controls whether IIFEs are included in the line count.
   * By default, IIFEs are not considered, but when set to `true`, they will
   * be included in the line count for the function.
   */
  IIFEs?: boolean;
  /**
   * Maximum number of lines allowed in a function.
   */
  max?: number;
  /**
   * Skip lines made up purely of whitespace.
   */
  skipBlankLines?: boolean;
  /**
   * Skip lines containing just comments.
   */
  skipComments?: boolean;
}
export interface MaxNestedCallbacks {
  /**
   * The `max` enforces a maximum depth that callbacks can be nested.
   */
  max?: number;
}
export interface MaxParamsConfig {
  /**
   * This option controls when to count a `this` parameter.
   *
   * - "always": always count `this`
   * - "never": never count `this`
   * - "except-void": count `this` only when it is not type `void`
   */
  countThis?: CountThis;
  /**
   * Deprecated alias for `countThis`.
   *
   * For example `{ "countVoidThis": true }` would mean that having a function
   * take a `this` parameter of type `void` is counted towards the maximum number of parameters.
   */
  countVoidThis?: boolean;
  /**
   * Maximum number of parameters allowed in function definitions.
   */
  max?: number;
}
export interface MaxStatementsConfig {
  /**
   * Whether to ignore top-level functions.
   */
  ignoreTopLevelFunctions?: boolean;
  /**
   * Maximum number of statements allowed per function.
   */
  max?: number;
}
export interface NoReservedKeysConfig {
  /**
   * Extra component option groups to inspect, on top of the built-in
   * `props` / `computed` / `data` / `asyncData` / `methods` / `setup`.
   */
  groups?: string[];
  /**
   * Extra reserved key names to disallow, on top of the built-in list.
   */
  reserved?: string[];
}
export interface NoReservedPropsConfig {
  /**
   * Vue major version whose reserved attribute set is applied. Vue 2 reserves
   * more names (`is`, `slot`, `class`, `style`, ...) than Vue 3.
   */
  vueVersion?: number;
}
export interface Options {
  ignoreProps?: string[];
}
export interface RequireDirectExport {
  /**
   * When set `true`, disallow functional component functions.
   */
  disallowFunctionalComponentFunction?: boolean;
}
/**
 * Configure the behavior of linter plugins.
 *
 * Here's an example if you're using Next.js in a monorepo:
 *
 * ```json
 * {
 *   "settings": {
 *     "next": {
 *       "rootDir": "apps/dashboard/"
 *     },
 *     "react": {
 *       "linkComponents": [
 *         {
 *           "name": "Link",
 *           "linkAttribute": "to"
 *         }
 *       ]
 *     },
 *     "jsx-a11y": {
 *       "components": {
 *         "Link": "a",
 *         "Button": "button"
 *       }
 *     }
 *   }
 * }
 * ```
 */
export interface OxlintPluginSettings {
  jest?: JestPluginSettings;
  jsdoc?: JSDocPluginSettings;
  "jsx-a11y"?: JSXA11YPluginSettings;
  next?: NextPluginSettings;
  react?: ReactPluginSettings;
  vitest?: VitestPluginSettings;
  [k: string]: unknown;
}
/**
 * Configure Jest plugin rules.
 *
 * See [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest)'s
 * configuration for a full reference.
 */
export interface JestPluginSettings {
  /**
   * Jest version — accepts a number (`29`) or a semver string (`"29.1.0"` or `"v29.1.0"`),
   * storing only the major version.
   * ::: warning
   * Using this config will override the `no-deprecated-functions` config set.
   * :::
   */
  version?: JestVersionSchema;
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
  tagNamePreference?: {
    [k: string]: TagNamePreference;
  };
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
   *   "settings": {
   *     "jsx-a11y": {
   *       "attributes": {
   *         "for": [
   *           "htmlFor",
   *           "for"
   *         ]
   *       }
   *     }
   *   }
   * }
   * ```
   */
  attributes?: {
    [k: string]: string[];
  };
  /**
   * To have your custom components be checked as DOM elements, you can
   * provide a mapping of your component names to the DOM element name.
   *
   * Example:
   *
   * ```json
   * {
   *   "settings": {
   *     "jsx-a11y": {
   *       "components": {
   *         "Link": "a",
   *         "IconButton": "button"
   *       }
   *     }
   *   }
   * }
   * ```
   */
  components?: {
    [k: string]: string;
  };
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
  polymorphicPropName?: string;
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
   *   "settings": {
   *     "next": {
   *       "rootDir": "apps/dashboard/"
   *     }
   *   }
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
  version?: string;
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
