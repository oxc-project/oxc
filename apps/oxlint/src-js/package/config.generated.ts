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
  "block-scoped-var"?: DummyRule;
  "capitalized-comments"?: DummyRule;
  "class-methods-use-this"?: DummyRule;
  complexity?: DummyRule;
  "constructor-super"?: DummyRule;
  curly?: DummyRule;
  "default-case"?: DummyRule;
  "default-case-last"?: DummyRule;
  "default-param-last"?: DummyRule;
  eqeqeq?: DummyRule;
  "for-direction"?: DummyRule;
  "func-name-matching"?: DummyRule;
  "func-names"?: DummyRule;
  "func-style"?: DummyRule;
  "getter-return"?: DummyRule;
  "grouped-accessor-pairs"?: DummyRule;
  "guard-for-in"?: DummyRule;
  "id-length"?: DummyRule;
  "import/consistent-type-specifier-style"?: DummyRule;
  "import/default"?: DummyRule;
  "import/export"?: DummyRule;
  "import/exports-last"?: DummyRule;
  "import/extensions"?: DummyRule;
  "import/first"?: DummyRule;
  "import/group-exports"?: DummyRule;
  "import/max-dependencies"?: DummyRule;
  "import/named"?: DummyRule;
  "import/namespace"?: DummyRule;
  "import/no-absolute-path"?: DummyRule;
  "import/no-amd"?: DummyRule;
  "import/no-anonymous-default-export"?: DummyRule;
  "import/no-commonjs"?: DummyRule;
  "import/no-cycle"?: DummyRule;
  "import/no-default-export"?: DummyRule;
  "import/no-duplicates"?: DummyRule;
  "import/no-dynamic-require"?: DummyRule;
  "import/no-empty-named-blocks"?: DummyRule;
  "import/no-mutable-exports"?: DummyRule;
  "import/no-named-as-default"?: DummyRule;
  "import/no-named-as-default-member"?: DummyRule;
  "import/no-named-default"?: DummyRule;
  "import/no-named-export"?: DummyRule;
  "import/no-namespace"?: DummyRule;
  "import/no-nodejs-modules"?: DummyRule;
  "import/no-relative-parent-imports"?: DummyRule;
  "import/no-self-import"?: DummyRule;
  "import/no-unassigned-import"?: DummyRule;
  "import/no-webpack-loader-syntax"?: DummyRule;
  "import/prefer-default-export"?: DummyRule;
  "import/unambiguous"?: DummyRule;
  "init-declarations"?: DummyRule;
  "jest/consistent-test-it"?: DummyRule;
  "jest/expect-expect"?: DummyRule;
  "jest/max-expects"?: DummyRule;
  "jest/max-nested-describe"?: DummyRule;
  "jest/no-alias-methods"?: DummyRule;
  "jest/no-commented-out-tests"?: DummyRule;
  "jest/no-conditional-expect"?: DummyRule;
  "jest/no-conditional-in-test"?: DummyRule;
  "jest/no-confusing-set-timeout"?: DummyRule;
  "jest/no-deprecated-functions"?: DummyRule;
  "jest/no-disabled-tests"?: DummyRule;
  "jest/no-done-callback"?: DummyRule;
  "jest/no-duplicate-hooks"?: DummyRule;
  "jest/no-export"?: DummyRule;
  "jest/no-focused-tests"?: DummyRule;
  "jest/no-hooks"?: DummyRule;
  "jest/no-identical-title"?: DummyRule;
  "jest/no-interpolation-in-snapshots"?: DummyRule;
  "jest/no-jasmine-globals"?: DummyRule;
  "jest/no-large-snapshots"?: DummyRule;
  "jest/no-mocks-import"?: DummyRule;
  "jest/no-restricted-jest-methods"?: DummyRule;
  "jest/no-restricted-matchers"?: DummyRule;
  "jest/no-standalone-expect"?: DummyRule;
  "jest/no-test-prefixes"?: DummyRule;
  "jest/no-test-return-statement"?: DummyRule;
  "jest/no-unneeded-async-expect-function"?: DummyRule;
  "jest/no-untyped-mock-factory"?: DummyRule;
  "jest/padding-around-after-all-blocks"?: DummyRule;
  "jest/padding-around-test-blocks"?: DummyRule;
  "jest/prefer-called-with"?: DummyRule;
  "jest/prefer-comparison-matcher"?: DummyRule;
  "jest/prefer-each"?: DummyRule;
  "jest/prefer-ending-with-an-expect"?: DummyRule;
  "jest/prefer-equality-matcher"?: DummyRule;
  "jest/prefer-expect-assertions"?: DummyRule;
  "jest/prefer-expect-resolves"?: DummyRule;
  "jest/prefer-hooks-in-order"?: DummyRule;
  "jest/prefer-hooks-on-top"?: DummyRule;
  "jest/prefer-importing-jest-globals"?: DummyRule;
  "jest/prefer-jest-mocked"?: DummyRule;
  "jest/prefer-lowercase-title"?: DummyRule;
  "jest/prefer-mock-promise-shorthand"?: DummyRule;
  "jest/prefer-mock-return-shorthand"?: DummyRule;
  "jest/prefer-snapshot-hint"?: DummyRule;
  "jest/prefer-spy-on"?: DummyRule;
  "jest/prefer-strict-equal"?: DummyRule;
  "jest/prefer-to-be"?: DummyRule;
  "jest/prefer-to-contain"?: DummyRule;
  "jest/prefer-to-have-been-called"?: DummyRule;
  "jest/prefer-to-have-been-called-times"?: DummyRule;
  "jest/prefer-to-have-length"?: DummyRule;
  "jest/prefer-todo"?: DummyRule;
  "jest/require-hook"?: DummyRule;
  "jest/require-to-throw-message"?: DummyRule;
  "jest/require-top-level-describe"?: DummyRule;
  "jest/valid-describe-callback"?: DummyRule;
  "jest/valid-expect"?: DummyRule;
  "jest/valid-expect-in-promise"?: DummyRule;
  "jest/valid-title"?: DummyRule;
  "jsdoc/check-access"?: DummyRule;
  "jsdoc/check-property-names"?: DummyRule;
  "jsdoc/check-tag-names"?: DummyRule;
  "jsdoc/empty-tags"?: DummyRule;
  "jsdoc/implements-on-classes"?: DummyRule;
  "jsdoc/no-defaults"?: DummyRule;
  "jsdoc/require-param"?: DummyRule;
  "jsdoc/require-param-description"?: DummyRule;
  "jsdoc/require-param-name"?: DummyRule;
  "jsdoc/require-param-type"?: DummyRule;
  "jsdoc/require-property"?: DummyRule;
  "jsdoc/require-property-description"?: DummyRule;
  "jsdoc/require-property-name"?: DummyRule;
  "jsdoc/require-property-type"?: DummyRule;
  "jsdoc/require-returns"?: DummyRule;
  "jsdoc/require-returns-description"?: DummyRule;
  "jsdoc/require-returns-type"?: DummyRule;
  "jsdoc/require-throws-type"?: DummyRule;
  "jsdoc/require-yields"?: DummyRule;
  "jsdoc/require-yields-type"?: DummyRule;
  "jsx-a11y/alt-text"?: DummyRule;
  "jsx-a11y/anchor-ambiguous-text"?: DummyRule;
  "jsx-a11y/anchor-has-content"?: DummyRule;
  "jsx-a11y/anchor-is-valid"?: DummyRule;
  "jsx-a11y/aria-activedescendant-has-tabindex"?: DummyRule;
  "jsx-a11y/aria-props"?: DummyRule;
  "jsx-a11y/aria-proptypes"?: DummyRule;
  "jsx-a11y/aria-role"?: DummyRule;
  "jsx-a11y/aria-unsupported-elements"?: DummyRule;
  "jsx-a11y/autocomplete-valid"?: DummyRule;
  "jsx-a11y/click-events-have-key-events"?: DummyRule;
  "jsx-a11y/control-has-associated-label"?: DummyRule;
  "jsx-a11y/heading-has-content"?: DummyRule;
  "jsx-a11y/html-has-lang"?: DummyRule;
  "jsx-a11y/iframe-has-title"?: DummyRule;
  "jsx-a11y/img-redundant-alt"?: DummyRule;
  "jsx-a11y/interactive-supports-focus"?: DummyRule;
  "jsx-a11y/label-has-associated-control"?: DummyRule;
  "jsx-a11y/lang"?: DummyRule;
  "jsx-a11y/media-has-caption"?: DummyRule;
  "jsx-a11y/mouse-events-have-key-events"?: DummyRule;
  "jsx-a11y/no-access-key"?: DummyRule;
  "jsx-a11y/no-aria-hidden-on-focusable"?: DummyRule;
  "jsx-a11y/no-autofocus"?: DummyRule;
  "jsx-a11y/no-distracting-elements"?: DummyRule;
  "jsx-a11y/no-interactive-element-to-noninteractive-role"?: DummyRule;
  "jsx-a11y/no-noninteractive-element-interactions"?: DummyRule;
  "jsx-a11y/no-noninteractive-element-to-interactive-role"?: DummyRule;
  "jsx-a11y/no-noninteractive-tabindex"?: DummyRule;
  "jsx-a11y/no-redundant-roles"?: DummyRule;
  "jsx-a11y/no-static-element-interactions"?: DummyRule;
  "jsx-a11y/prefer-tag-over-role"?: DummyRule;
  "jsx-a11y/role-has-required-aria-props"?: DummyRule;
  "jsx-a11y/role-supports-aria-props"?: DummyRule;
  "jsx-a11y/scope"?: DummyRule;
  "jsx-a11y/tabindex-no-positive"?: DummyRule;
  "logical-assignment-operators"?: DummyRule;
  "max-classes-per-file"?: DummyRule;
  "max-depth"?: DummyRule;
  "max-lines"?: DummyRule;
  "max-lines-per-function"?: DummyRule;
  "max-nested-callbacks"?: DummyRule;
  "max-params"?: DummyRule;
  "max-statements"?: DummyRule;
  "new-cap"?: DummyRule;
  "nextjs/google-font-display"?: DummyRule;
  "nextjs/google-font-preconnect"?: DummyRule;
  "nextjs/inline-script-id"?: DummyRule;
  "nextjs/next-script-for-ga"?: DummyRule;
  "nextjs/no-assign-module-variable"?: DummyRule;
  "nextjs/no-async-client-component"?: DummyRule;
  "nextjs/no-before-interactive-script-outside-document"?: DummyRule;
  "nextjs/no-css-tags"?: DummyRule;
  "nextjs/no-document-import-in-page"?: DummyRule;
  "nextjs/no-duplicate-head"?: DummyRule;
  "nextjs/no-head-element"?: DummyRule;
  "nextjs/no-head-import-in-document"?: DummyRule;
  "nextjs/no-html-link-for-pages"?: DummyRule;
  "nextjs/no-img-element"?: DummyRule;
  "nextjs/no-page-custom-font"?: DummyRule;
  "nextjs/no-script-component-in-head"?: DummyRule;
  "nextjs/no-styled-jsx-in-document"?: DummyRule;
  "nextjs/no-sync-scripts"?: DummyRule;
  "nextjs/no-title-in-document-head"?: DummyRule;
  "nextjs/no-typos"?: DummyRule;
  "nextjs/no-unwanted-polyfillio"?: DummyRule;
  "no-alert"?: DummyRule;
  "no-array-constructor"?: DummyRule;
  "no-async-promise-executor"?: DummyRule;
  "no-await-in-loop"?: DummyRule;
  "no-bitwise"?: DummyRule;
  "no-caller"?: DummyRule;
  "no-case-declarations"?: DummyRule;
  "no-class-assign"?: DummyRule;
  "no-compare-neg-zero"?: DummyRule;
  "no-cond-assign"?: DummyRule;
  "no-console"?: DummyRule;
  "no-const-assign"?: DummyRule;
  "no-constant-binary-expression"?: DummyRule;
  "no-constant-condition"?: DummyRule;
  "no-constructor-return"?: DummyRule;
  "no-continue"?: DummyRule;
  "no-control-regex"?: DummyRule;
  "no-debugger"?: DummyRule;
  "no-delete-var"?: DummyRule;
  "no-div-regex"?: DummyRule;
  "no-dupe-class-members"?: DummyRule;
  "no-dupe-else-if"?: DummyRule;
  "no-dupe-keys"?: DummyRule;
  "no-duplicate-case"?: DummyRule;
  "no-duplicate-imports"?: DummyRule;
  "no-else-return"?: DummyRule;
  "no-empty"?: DummyRule;
  "no-empty-character-class"?: DummyRule;
  "no-empty-function"?: DummyRule;
  "no-empty-pattern"?: DummyRule;
  "no-empty-static-block"?: DummyRule;
  "no-eq-null"?: DummyRule;
  "no-eval"?: DummyRule;
  "no-ex-assign"?: DummyRule;
  "no-extend-native"?: DummyRule;
  "no-extra-bind"?: DummyRule;
  "no-extra-boolean-cast"?: DummyRule;
  "no-extra-label"?: DummyRule;
  "no-fallthrough"?: DummyRule;
  "no-func-assign"?: DummyRule;
  "no-global-assign"?: DummyRule;
  "no-implicit-coercion"?: DummyRule;
  "no-implicit-globals"?: DummyRule;
  "no-import-assign"?: DummyRule;
  "no-inline-comments"?: DummyRule;
  "no-inner-declarations"?: DummyRule;
  "no-invalid-regexp"?: DummyRule;
  "no-irregular-whitespace"?: DummyRule;
  "no-iterator"?: DummyRule;
  "no-label-var"?: DummyRule;
  "no-labels"?: DummyRule;
  "no-lone-blocks"?: DummyRule;
  "no-lonely-if"?: DummyRule;
  "no-loop-func"?: DummyRule;
  "no-loss-of-precision"?: DummyRule;
  "no-magic-numbers"?: DummyRule;
  "no-misleading-character-class"?: DummyRule;
  "no-multi-assign"?: DummyRule;
  "no-multi-str"?: DummyRule;
  "no-negated-condition"?: DummyRule;
  "no-nested-ternary"?: DummyRule;
  "no-new"?: DummyRule;
  "no-new-func"?: DummyRule;
  "no-new-native-nonconstructor"?: DummyRule;
  "no-new-wrappers"?: DummyRule;
  "no-nonoctal-decimal-escape"?: DummyRule;
  "no-obj-calls"?: DummyRule;
  "no-object-constructor"?: DummyRule;
  "no-param-reassign"?: DummyRule;
  "no-plusplus"?: DummyRule;
  "no-promise-executor-return"?: DummyRule;
  "no-proto"?: DummyRule;
  "no-prototype-builtins"?: DummyRule;
  "no-redeclare"?: DummyRule;
  "no-regex-spaces"?: DummyRule;
  "no-restricted-exports"?: DummyRule;
  "no-restricted-globals"?: DummyRule;
  "no-restricted-imports"?: DummyRule;
  "no-restricted-properties"?: DummyRule;
  "no-return-assign"?: DummyRule;
  "no-script-url"?: DummyRule;
  "no-self-assign"?: DummyRule;
  "no-self-compare"?: DummyRule;
  "no-sequences"?: DummyRule;
  "no-setter-return"?: DummyRule;
  "no-shadow"?: DummyRule;
  "no-shadow-restricted-names"?: DummyRule;
  "no-sparse-arrays"?: DummyRule;
  "no-template-curly-in-string"?: DummyRule;
  "no-ternary"?: DummyRule;
  "no-this-before-super"?: DummyRule;
  "no-throw-literal"?: DummyRule;
  "no-unassigned-vars"?: DummyRule;
  "no-undef"?: DummyRule;
  "no-undefined"?: DummyRule;
  "no-underscore-dangle"?: DummyRule;
  "no-unexpected-multiline"?: DummyRule;
  "no-unmodified-loop-condition"?: DummyRule;
  "no-unneeded-ternary"?: DummyRule;
  "no-unreachable"?: DummyRule;
  "no-unsafe-finally"?: DummyRule;
  "no-unsafe-negation"?: DummyRule;
  "no-unsafe-optional-chaining"?: DummyRule;
  "no-unused-expressions"?: DummyRule;
  "no-unused-labels"?: DummyRule;
  "no-unused-private-class-members"?: DummyRule;
  "no-unused-vars"?: DummyRule;
  "no-use-before-define"?: DummyRule;
  "no-useless-assignment"?: DummyRule;
  "no-useless-backreference"?: DummyRule;
  "no-useless-call"?: DummyRule;
  "no-useless-catch"?: DummyRule;
  "no-useless-computed-key"?: DummyRule;
  "no-useless-concat"?: DummyRule;
  "no-useless-constructor"?: DummyRule;
  "no-useless-escape"?: DummyRule;
  "no-useless-rename"?: DummyRule;
  "no-useless-return"?: DummyRule;
  "no-var"?: DummyRule;
  "no-void"?: DummyRule;
  "no-warning-comments"?: DummyRule;
  "no-with"?: DummyRule;
  "node/global-require"?: DummyRule;
  "node/handle-callback-err"?: DummyRule;
  "node/no-exports-assign"?: DummyRule;
  "node/no-new-require"?: DummyRule;
  "node/no-path-concat"?: DummyRule;
  "node/no-process-env"?: DummyRule;
  "object-shorthand"?: DummyRule;
  "operator-assignment"?: DummyRule;
  "oxc/approx-constant"?: DummyRule;
  "oxc/bad-array-method-on-arguments"?: DummyRule;
  "oxc/bad-bitwise-operator"?: DummyRule;
  "oxc/bad-char-at-comparison"?: DummyRule;
  "oxc/bad-comparison-sequence"?: DummyRule;
  "oxc/bad-min-max-func"?: DummyRule;
  "oxc/bad-object-literal-comparison"?: DummyRule;
  "oxc/bad-replace-all-arg"?: DummyRule;
  "oxc/branches-sharing-code"?: DummyRule;
  "oxc/const-comparisons"?: DummyRule;
  "oxc/double-comparisons"?: DummyRule;
  "oxc/erasing-op"?: DummyRule;
  "oxc/misrefactored-assign-op"?: DummyRule;
  "oxc/missing-throw"?: DummyRule;
  "oxc/no-accumulating-spread"?: DummyRule;
  "oxc/no-async-await"?: DummyRule;
  "oxc/no-async-endpoint-handlers"?: DummyRule;
  "oxc/no-barrel-file"?: DummyRule;
  "oxc/no-const-enum"?: DummyRule;
  "oxc/no-map-spread"?: DummyRule;
  "oxc/no-optional-chaining"?: DummyRule;
  "oxc/no-rest-spread-properties"?: DummyRule;
  "oxc/no-this-in-exported-function"?: DummyRule;
  "oxc/number-arg-out-of-range"?: DummyRule;
  "oxc/only-used-in-recursion"?: DummyRule;
  "oxc/uninvoked-array-callback"?: DummyRule;
  "prefer-const"?: DummyRule;
  "prefer-destructuring"?: DummyRule;
  "prefer-exponentiation-operator"?: DummyRule;
  "prefer-numeric-literals"?: DummyRule;
  "prefer-object-has-own"?: DummyRule;
  "prefer-object-spread"?: DummyRule;
  "prefer-promise-reject-errors"?: DummyRule;
  "prefer-regex-literals"?: DummyRule;
  "prefer-rest-params"?: DummyRule;
  "prefer-spread"?: DummyRule;
  "prefer-template"?: DummyRule;
  "preserve-caught-error"?: DummyRule;
  "promise/always-return"?: DummyRule;
  "promise/avoid-new"?: DummyRule;
  "promise/catch-or-return"?: DummyRule;
  "promise/no-callback-in-promise"?: DummyRule;
  "promise/no-multiple-resolved"?: DummyRule;
  "promise/no-nesting"?: DummyRule;
  "promise/no-new-statics"?: DummyRule;
  "promise/no-promise-in-callback"?: DummyRule;
  "promise/no-return-in-finally"?: DummyRule;
  "promise/no-return-wrap"?: DummyRule;
  "promise/param-names"?: DummyRule;
  "promise/prefer-await-to-callbacks"?: DummyRule;
  "promise/prefer-await-to-then"?: DummyRule;
  "promise/prefer-catch"?: DummyRule;
  "promise/spec-only"?: DummyRule;
  "promise/valid-params"?: DummyRule;
  radix?: DummyRule;
  "react-perf/jsx-no-jsx-as-prop"?: DummyRule;
  "react-perf/jsx-no-new-array-as-prop"?: DummyRule;
  "react-perf/jsx-no-new-function-as-prop"?: DummyRule;
  "react-perf/jsx-no-new-object-as-prop"?: DummyRule;
  "react/button-has-type"?: DummyRule;
  "react/checked-requires-onchange-or-readonly"?: DummyRule;
  "react/display-name"?: DummyRule;
  "react/exhaustive-deps"?: DummyRule;
  "react/forbid-component-props"?: DummyRule;
  "react/forbid-dom-props"?: DummyRule;
  "react/forbid-elements"?: DummyRule;
  "react/forward-ref-uses-ref"?: DummyRule;
  "react/hook-use-state"?: DummyRule;
  "react/iframe-missing-sandbox"?: DummyRule;
  "react/jsx-boolean-value"?: DummyRule;
  "react/jsx-curly-brace-presence"?: DummyRule;
  "react/jsx-filename-extension"?: DummyRule;
  "react/jsx-fragments"?: DummyRule;
  "react/jsx-handler-names"?: DummyRule;
  "react/jsx-key"?: DummyRule;
  "react/jsx-max-depth"?: DummyRule;
  "react/jsx-no-comment-textnodes"?: DummyRule;
  "react/jsx-no-constructed-context-values"?: DummyRule;
  "react/jsx-no-duplicate-props"?: DummyRule;
  "react/jsx-no-script-url"?: DummyRule;
  "react/jsx-no-target-blank"?: DummyRule;
  "react/jsx-no-undef"?: DummyRule;
  "react/jsx-no-useless-fragment"?: DummyRule;
  "react/jsx-pascal-case"?: DummyRule;
  "react/jsx-props-no-spread-multi"?: DummyRule;
  "react/jsx-props-no-spreading"?: DummyRule;
  "react/no-array-index-key"?: DummyRule;
  "react/no-children-prop"?: DummyRule;
  "react/no-clone-element"?: DummyRule;
  "react/no-danger"?: DummyRule;
  "react/no-danger-with-children"?: DummyRule;
  "react/no-did-mount-set-state"?: DummyRule;
  "react/no-did-update-set-state"?: DummyRule;
  "react/no-direct-mutation-state"?: DummyRule;
  "react/no-find-dom-node"?: DummyRule;
  "react/no-is-mounted"?: DummyRule;
  "react/no-multi-comp"?: DummyRule;
  "react/no-namespace"?: DummyRule;
  "react/no-react-children"?: DummyRule;
  "react/no-redundant-should-component-update"?: DummyRule;
  "react/no-render-return-value"?: DummyRule;
  "react/no-set-state"?: DummyRule;
  "react/no-string-refs"?: DummyRule;
  "react/no-this-in-sfc"?: DummyRule;
  "react/no-unescaped-entities"?: DummyRule;
  "react/no-unknown-property"?: DummyRule;
  "react/no-unsafe"?: DummyRule;
  "react/no-will-update-set-state"?: DummyRule;
  "react/only-export-components"?: DummyRule;
  "react/prefer-es6-class"?: DummyRule;
  "react/prefer-function-component"?: DummyRule;
  "react/react-in-jsx-scope"?: DummyRule;
  "react/require-render-return"?: DummyRule;
  "react/rules-of-hooks"?: DummyRule;
  "react/self-closing-comp"?: DummyRule;
  "react/state-in-constructor"?: DummyRule;
  "react/style-prop-object"?: DummyRule;
  "react/void-dom-elements-no-children"?: DummyRule;
  "require-await"?: DummyRule;
  "require-unicode-regexp"?: DummyRule;
  "require-yield"?: DummyRule;
  "sort-imports"?: DummyRule;
  "sort-keys"?: DummyRule;
  "sort-vars"?: DummyRule;
  "symbol-description"?: DummyRule;
  "typescript/adjacent-overload-signatures"?: DummyRule;
  "typescript/array-type"?: DummyRule;
  "typescript/await-thenable"?: DummyRule;
  "typescript/ban-ts-comment"?: DummyRule;
  "typescript/ban-tslint-comment"?: DummyRule;
  "typescript/ban-types"?: DummyRule;
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
  "typescript/no-array-delete"?: DummyRule;
  "typescript/no-base-to-string"?: DummyRule;
  "typescript/no-confusing-non-null-assertion"?: DummyRule;
  "typescript/no-confusing-void-expression"?: DummyRule;
  "typescript/no-deprecated"?: DummyRule;
  "typescript/no-duplicate-enum-values"?: DummyRule;
  "typescript/no-duplicate-type-constituents"?: DummyRule;
  "typescript/no-dynamic-delete"?: DummyRule;
  "typescript/no-empty-interface"?: DummyRule;
  "typescript/no-empty-object-type"?: DummyRule;
  "typescript/no-explicit-any"?: DummyRule;
  "typescript/no-extra-non-null-assertion"?: DummyRule;
  "typescript/no-extraneous-class"?: DummyRule;
  "typescript/no-floating-promises"?: DummyRule;
  "typescript/no-for-in-array"?: DummyRule;
  "typescript/no-implied-eval"?: DummyRule;
  "typescript/no-import-type-side-effects"?: DummyRule;
  "typescript/no-inferrable-types"?: DummyRule;
  "typescript/no-invalid-void-type"?: DummyRule;
  "typescript/no-meaningless-void-operator"?: DummyRule;
  "typescript/no-misused-new"?: DummyRule;
  "typescript/no-misused-promises"?: DummyRule;
  "typescript/no-misused-spread"?: DummyRule;
  "typescript/no-mixed-enums"?: DummyRule;
  "typescript/no-namespace"?: DummyRule;
  "typescript/no-non-null-asserted-nullish-coalescing"?: DummyRule;
  "typescript/no-non-null-asserted-optional-chain"?: DummyRule;
  "typescript/no-non-null-assertion"?: DummyRule;
  "typescript/no-redundant-type-constituents"?: DummyRule;
  "typescript/no-require-imports"?: DummyRule;
  "typescript/no-restricted-types"?: DummyRule;
  "typescript/no-this-alias"?: DummyRule;
  "typescript/no-unnecessary-boolean-literal-compare"?: DummyRule;
  "typescript/no-unnecessary-condition"?: DummyRule;
  "typescript/no-unnecessary-parameter-property-assignment"?: DummyRule;
  "typescript/no-unnecessary-qualifier"?: DummyRule;
  "typescript/no-unnecessary-template-expression"?: DummyRule;
  "typescript/no-unnecessary-type-arguments"?: DummyRule;
  "typescript/no-unnecessary-type-assertion"?: DummyRule;
  "typescript/no-unnecessary-type-constraint"?: DummyRule;
  "typescript/no-unnecessary-type-conversion"?: DummyRule;
  "typescript/no-unnecessary-type-parameters"?: DummyRule;
  "typescript/no-unsafe-argument"?: DummyRule;
  "typescript/no-unsafe-assignment"?: DummyRule;
  "typescript/no-unsafe-call"?: DummyRule;
  "typescript/no-unsafe-declaration-merging"?: DummyRule;
  "typescript/no-unsafe-enum-comparison"?: DummyRule;
  "typescript/no-unsafe-function-type"?: DummyRule;
  "typescript/no-unsafe-member-access"?: DummyRule;
  "typescript/no-unsafe-return"?: DummyRule;
  "typescript/no-unsafe-type-assertion"?: DummyRule;
  "typescript/no-unsafe-unary-minus"?: DummyRule;
  "typescript/no-useless-default-assignment"?: DummyRule;
  "typescript/no-useless-empty-export"?: DummyRule;
  "typescript/no-var-requires"?: DummyRule;
  "typescript/no-wrapper-object-types"?: DummyRule;
  "typescript/non-nullable-type-assertion-style"?: DummyRule;
  "typescript/only-throw-error"?: DummyRule;
  "typescript/parameter-properties"?: DummyRule;
  "typescript/prefer-as-const"?: DummyRule;
  "typescript/prefer-enum-initializers"?: DummyRule;
  "typescript/prefer-find"?: DummyRule;
  "typescript/prefer-for-of"?: DummyRule;
  "typescript/prefer-function-type"?: DummyRule;
  "typescript/prefer-includes"?: DummyRule;
  "typescript/prefer-literal-enum-member"?: DummyRule;
  "typescript/prefer-namespace-keyword"?: DummyRule;
  "typescript/prefer-nullish-coalescing"?: DummyRule;
  "typescript/prefer-optional-chain"?: DummyRule;
  "typescript/prefer-promise-reject-errors"?: DummyRule;
  "typescript/prefer-readonly"?: DummyRule;
  "typescript/prefer-readonly-parameter-types"?: DummyRule;
  "typescript/prefer-reduce-type-parameter"?: DummyRule;
  "typescript/prefer-regexp-exec"?: DummyRule;
  "typescript/prefer-return-this-type"?: DummyRule;
  "typescript/prefer-string-starts-ends-with"?: DummyRule;
  "typescript/prefer-ts-expect-error"?: DummyRule;
  "typescript/promise-function-async"?: DummyRule;
  "typescript/related-getter-setter-pairs"?: DummyRule;
  "typescript/require-array-sort-compare"?: DummyRule;
  "typescript/require-await"?: DummyRule;
  "typescript/restrict-plus-operands"?: DummyRule;
  "typescript/restrict-template-expressions"?: DummyRule;
  "typescript/return-await"?: DummyRule;
  "typescript/strict-boolean-expressions"?: DummyRule;
  "typescript/strict-void-return"?: DummyRule;
  "typescript/switch-exhaustiveness-check"?: DummyRule;
  "typescript/triple-slash-reference"?: DummyRule;
  "typescript/unbound-method"?: DummyRule;
  "typescript/unified-signatures"?: DummyRule;
  "typescript/use-unknown-in-catch-callback-variable"?: DummyRule;
  "unicode-bom"?: DummyRule;
  "unicorn/catch-error-name"?: DummyRule;
  "unicorn/consistent-assert"?: DummyRule;
  "unicorn/consistent-date-clone"?: DummyRule;
  "unicorn/consistent-empty-array-spread"?: DummyRule;
  "unicorn/consistent-existence-index-check"?: DummyRule;
  "unicorn/consistent-function-scoping"?: DummyRule;
  "unicorn/consistent-template-literal-escape"?: DummyRule;
  "unicorn/custom-error-definition"?: DummyRule;
  "unicorn/empty-brace-spaces"?: DummyRule;
  "unicorn/error-message"?: DummyRule;
  "unicorn/escape-case"?: DummyRule;
  "unicorn/explicit-length-check"?: DummyRule;
  "unicorn/filename-case"?: DummyRule;
  "unicorn/new-for-builtins"?: DummyRule;
  "unicorn/no-abusive-eslint-disable"?: DummyRule;
  "unicorn/no-accessor-recursion"?: DummyRule;
  "unicorn/no-anonymous-default-export"?: DummyRule;
  "unicorn/no-array-callback-reference"?: DummyRule;
  "unicorn/no-array-for-each"?: DummyRule;
  "unicorn/no-array-method-this-argument"?: DummyRule;
  "unicorn/no-array-reduce"?: DummyRule;
  "unicorn/no-array-reverse"?: DummyRule;
  "unicorn/no-array-sort"?: DummyRule;
  "unicorn/no-await-expression-member"?: DummyRule;
  "unicorn/no-await-in-promise-methods"?: DummyRule;
  "unicorn/no-console-spaces"?: DummyRule;
  "unicorn/no-document-cookie"?: DummyRule;
  "unicorn/no-empty-file"?: DummyRule;
  "unicorn/no-hex-escape"?: DummyRule;
  "unicorn/no-immediate-mutation"?: DummyRule;
  "unicorn/no-instanceof-array"?: DummyRule;
  "unicorn/no-instanceof-builtins"?: DummyRule;
  "unicorn/no-invalid-fetch-options"?: DummyRule;
  "unicorn/no-invalid-remove-event-listener"?: DummyRule;
  "unicorn/no-length-as-slice-end"?: DummyRule;
  "unicorn/no-lonely-if"?: DummyRule;
  "unicorn/no-magic-array-flat-depth"?: DummyRule;
  "unicorn/no-negated-condition"?: DummyRule;
  "unicorn/no-negation-in-equality-check"?: DummyRule;
  "unicorn/no-nested-ternary"?: DummyRule;
  "unicorn/no-new-array"?: DummyRule;
  "unicorn/no-new-buffer"?: DummyRule;
  "unicorn/no-null"?: DummyRule;
  "unicorn/no-object-as-default-parameter"?: DummyRule;
  "unicorn/no-process-exit"?: DummyRule;
  "unicorn/no-single-promise-in-promise-methods"?: DummyRule;
  "unicorn/no-static-only-class"?: DummyRule;
  "unicorn/no-thenable"?: DummyRule;
  "unicorn/no-this-assignment"?: DummyRule;
  "unicorn/no-typeof-undefined"?: DummyRule;
  "unicorn/no-unnecessary-array-flat-depth"?: DummyRule;
  "unicorn/no-unnecessary-array-splice-count"?: DummyRule;
  "unicorn/no-unnecessary-await"?: DummyRule;
  "unicorn/no-unnecessary-slice-end"?: DummyRule;
  "unicorn/no-unreadable-array-destructuring"?: DummyRule;
  "unicorn/no-unreadable-iife"?: DummyRule;
  "unicorn/no-useless-collection-argument"?: DummyRule;
  "unicorn/no-useless-error-capture-stack-trace"?: DummyRule;
  "unicorn/no-useless-fallback-in-spread"?: DummyRule;
  "unicorn/no-useless-iterator-to-array"?: DummyRule;
  "unicorn/no-useless-length-check"?: DummyRule;
  "unicorn/no-useless-promise-resolve-reject"?: DummyRule;
  "unicorn/no-useless-spread"?: DummyRule;
  "unicorn/no-useless-switch-case"?: DummyRule;
  "unicorn/no-useless-undefined"?: DummyRule;
  "unicorn/no-zero-fractions"?: DummyRule;
  "unicorn/number-literal-case"?: DummyRule;
  "unicorn/numeric-separators-style"?: DummyRule;
  "unicorn/prefer-add-event-listener"?: DummyRule;
  "unicorn/prefer-array-find"?: DummyRule;
  "unicorn/prefer-array-flat"?: DummyRule;
  "unicorn/prefer-array-flat-map"?: DummyRule;
  "unicorn/prefer-array-index-of"?: DummyRule;
  "unicorn/prefer-array-some"?: DummyRule;
  "unicorn/prefer-at"?: DummyRule;
  "unicorn/prefer-bigint-literals"?: DummyRule;
  "unicorn/prefer-blob-reading-methods"?: DummyRule;
  "unicorn/prefer-class-fields"?: DummyRule;
  "unicorn/prefer-classlist-toggle"?: DummyRule;
  "unicorn/prefer-code-point"?: DummyRule;
  "unicorn/prefer-date-now"?: DummyRule;
  "unicorn/prefer-default-parameters"?: DummyRule;
  "unicorn/prefer-dom-node-append"?: DummyRule;
  "unicorn/prefer-dom-node-dataset"?: DummyRule;
  "unicorn/prefer-dom-node-remove"?: DummyRule;
  "unicorn/prefer-dom-node-text-content"?: DummyRule;
  "unicorn/prefer-event-target"?: DummyRule;
  "unicorn/prefer-global-this"?: DummyRule;
  "unicorn/prefer-import-meta-properties"?: DummyRule;
  "unicorn/prefer-includes"?: DummyRule;
  "unicorn/prefer-keyboard-event-key"?: DummyRule;
  "unicorn/prefer-logical-operator-over-ternary"?: DummyRule;
  "unicorn/prefer-math-min-max"?: DummyRule;
  "unicorn/prefer-math-trunc"?: DummyRule;
  "unicorn/prefer-modern-dom-apis"?: DummyRule;
  "unicorn/prefer-modern-math-apis"?: DummyRule;
  "unicorn/prefer-module"?: DummyRule;
  "unicorn/prefer-native-coercion-functions"?: DummyRule;
  "unicorn/prefer-negative-index"?: DummyRule;
  "unicorn/prefer-node-protocol"?: DummyRule;
  "unicorn/prefer-number-properties"?: DummyRule;
  "unicorn/prefer-object-from-entries"?: DummyRule;
  "unicorn/prefer-optional-catch-binding"?: DummyRule;
  "unicorn/prefer-prototype-methods"?: DummyRule;
  "unicorn/prefer-query-selector"?: DummyRule;
  "unicorn/prefer-reflect-apply"?: DummyRule;
  "unicorn/prefer-regexp-test"?: DummyRule;
  "unicorn/prefer-response-static-json"?: DummyRule;
  "unicorn/prefer-set-has"?: DummyRule;
  "unicorn/prefer-set-size"?: DummyRule;
  "unicorn/prefer-spread"?: DummyRule;
  "unicorn/prefer-string-raw"?: DummyRule;
  "unicorn/prefer-string-replace-all"?: DummyRule;
  "unicorn/prefer-string-slice"?: DummyRule;
  "unicorn/prefer-string-starts-ends-with"?: DummyRule;
  "unicorn/prefer-string-trim-start-end"?: DummyRule;
  "unicorn/prefer-structured-clone"?: DummyRule;
  "unicorn/prefer-ternary"?: DummyRule;
  "unicorn/prefer-top-level-await"?: DummyRule;
  "unicorn/prefer-type-error"?: DummyRule;
  "unicorn/relative-url-style"?: DummyRule;
  "unicorn/require-array-join-separator"?: DummyRule;
  "unicorn/require-module-attributes"?: DummyRule;
  "unicorn/require-module-specifiers"?: DummyRule;
  "unicorn/require-number-to-fixed-digits-argument"?: DummyRule;
  "unicorn/require-post-message-target-origin"?: DummyRule;
  "unicorn/switch-case-braces"?: DummyRule;
  "unicorn/switch-case-break-position"?: DummyRule;
  "unicorn/text-encoding-identifier-case"?: DummyRule;
  "unicorn/throw-new-error"?: DummyRule;
  "use-isnan"?: DummyRule;
  "valid-typeof"?: DummyRule;
  "vars-on-top"?: DummyRule;
  "vitest/consistent-each-for"?: DummyRule;
  "vitest/consistent-test-filename"?: DummyRule;
  "vitest/consistent-test-it"?: DummyRule;
  "vitest/consistent-vitest-vi"?: DummyRule;
  "vitest/expect-expect"?: DummyRule;
  "vitest/hoisted-apis-on-top"?: DummyRule;
  "vitest/max-expects"?: DummyRule;
  "vitest/max-nested-describe"?: DummyRule;
  "vitest/no-alias-methods"?: DummyRule;
  "vitest/no-commented-out-tests"?: DummyRule;
  "vitest/no-conditional-expect"?: DummyRule;
  "vitest/no-conditional-in-test"?: DummyRule;
  "vitest/no-conditional-tests"?: DummyRule;
  "vitest/no-disabled-tests"?: DummyRule;
  "vitest/no-duplicate-hooks"?: DummyRule;
  "vitest/no-focused-tests"?: DummyRule;
  "vitest/no-hooks"?: DummyRule;
  "vitest/no-identical-title"?: DummyRule;
  "vitest/no-import-node-test"?: DummyRule;
  "vitest/no-importing-vitest-globals"?: DummyRule;
  "vitest/no-interpolation-in-snapshots"?: DummyRule;
  "vitest/no-large-snapshots"?: DummyRule;
  "vitest/no-mocks-import"?: DummyRule;
  "vitest/no-restricted-matchers"?: DummyRule;
  "vitest/no-restricted-vi-methods"?: DummyRule;
  "vitest/no-standalone-expect"?: DummyRule;
  "vitest/no-test-prefixes"?: DummyRule;
  "vitest/no-test-return-statement"?: DummyRule;
  "vitest/no-unneeded-async-expect-function"?: DummyRule;
  "vitest/prefer-called-exactly-once-with"?: DummyRule;
  "vitest/prefer-called-once"?: DummyRule;
  "vitest/prefer-called-times"?: DummyRule;
  "vitest/prefer-called-with"?: DummyRule;
  "vitest/prefer-comparison-matcher"?: DummyRule;
  "vitest/prefer-describe-function-title"?: DummyRule;
  "vitest/prefer-each"?: DummyRule;
  "vitest/prefer-equality-matcher"?: DummyRule;
  "vitest/prefer-expect-assertions"?: DummyRule;
  "vitest/prefer-expect-resolves"?: DummyRule;
  "vitest/prefer-expect-type-of"?: DummyRule;
  "vitest/prefer-hooks-in-order"?: DummyRule;
  "vitest/prefer-hooks-on-top"?: DummyRule;
  "vitest/prefer-import-in-mock"?: DummyRule;
  "vitest/prefer-importing-vitest-globals"?: DummyRule;
  "vitest/prefer-lowercase-title"?: DummyRule;
  "vitest/prefer-mock-promise-shorthand"?: DummyRule;
  "vitest/prefer-mock-return-shorthand"?: DummyRule;
  "vitest/prefer-snapshot-hint"?: DummyRule;
  "vitest/prefer-spy-on"?: DummyRule;
  "vitest/prefer-strict-boolean-matchers"?: DummyRule;
  "vitest/prefer-strict-equal"?: DummyRule;
  "vitest/prefer-to-be"?: DummyRule;
  "vitest/prefer-to-be-falsy"?: DummyRule;
  "vitest/prefer-to-be-object"?: DummyRule;
  "vitest/prefer-to-be-truthy"?: DummyRule;
  "vitest/prefer-to-contain"?: DummyRule;
  "vitest/prefer-to-have-been-called-times"?: DummyRule;
  "vitest/prefer-to-have-length"?: DummyRule;
  "vitest/prefer-todo"?: DummyRule;
  "vitest/require-awaited-expect-poll"?: DummyRule;
  "vitest/require-hook"?: DummyRule;
  "vitest/require-local-test-context-for-concurrent-snapshots"?: DummyRule;
  "vitest/require-mock-type-parameters"?: DummyRule;
  "vitest/require-test-timeout"?: DummyRule;
  "vitest/require-to-throw-message"?: DummyRule;
  "vitest/require-top-level-describe"?: DummyRule;
  "vitest/valid-describe-callback"?: DummyRule;
  "vitest/valid-expect"?: DummyRule;
  "vitest/valid-expect-in-promise"?: DummyRule;
  "vitest/valid-title"?: DummyRule;
  "vitest/warn-todo"?: DummyRule;
  "vue/define-emits-declaration"?: DummyRule;
  "vue/define-props-declaration"?: DummyRule;
  "vue/define-props-destructuring"?: DummyRule;
  "vue/max-props"?: DummyRule;
  "vue/no-arrow-functions-in-watch"?: DummyRule;
  "vue/no-deprecated-data-object-declaration"?: DummyRule;
  "vue/no-deprecated-delete-set"?: DummyRule;
  "vue/no-deprecated-destroyed-lifecycle"?: DummyRule;
  "vue/no-deprecated-events-api"?: DummyRule;
  "vue/no-deprecated-model-definition"?: DummyRule;
  "vue/no-deprecated-vue-config-keycodes"?: DummyRule;
  "vue/no-export-in-script-setup"?: DummyRule;
  "vue/no-import-compiler-macros"?: DummyRule;
  "vue/no-lifecycle-after-await"?: DummyRule;
  "vue/no-multiple-slot-args"?: DummyRule;
  "vue/no-required-prop-with-default"?: DummyRule;
  "vue/no-this-in-before-route-enter"?: DummyRule;
  "vue/prefer-import-from-vue"?: DummyRule;
  "vue/require-default-export"?: DummyRule;
  "vue/require-typed-ref"?: DummyRule;
  "vue/return-in-computed-property"?: DummyRule;
  "vue/valid-define-emits"?: DummyRule;
  "vue/valid-define-props"?: DummyRule;
  yoda?: DummyRule;
  [k: string]: DummyRule;
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
