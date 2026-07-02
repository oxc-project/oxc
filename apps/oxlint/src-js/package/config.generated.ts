/*
 * This file is generated from npm/oxlint/configuration_schema.json.
 * Run `just linter-config-ts` to regenerate.
 */

export type AllowWarnDeny = ("allow" | "off" | "warn" | "error" | "deny") | number;
export type GlobalValue = ("readonly" | "writable" | "off") | undefined;
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
export type RuleNoConfig = AllowWarnDeny | [AllowWarnDeny];
export type Mode2 = "as-needed" | "always" | "never";
export type AlwaysNever = "always" | "never";
export type OptionsJsonEnum =
  | CommentConfigJson
  | {
      block?: CommentConfigJson;
      line?: CommentConfigJson;
    };
export type IgnoreClassWithImplements = "all" | "public-fields";
export type ComplexityConfigEnum = number | ComplexityConfig;
export type Variant = "classic" | "modified";
/**
 * The enforcement type for the curly rule.
 */
export type CurlyType = "all" | "multi" | "multi-line" | "multi-or-nest";
/**
 * The optional second element of the curly config array.
 * When set to `"consistent"`, enforces consistent brace usage within if-else chains.
 */
export type CurlyConsistent = "consistent";
export type CompareType = "always" | "smart";
export type NullType = "always" | "never" | "ignore";
export type FuncNameMatchingMode = "always" | "never";
export type FuncNamesConfigType = "always" | "as-needed" | "never";
export type Style = "expression" | "declaration";
export type NamedExports = "ignore" | "expression" | "declaration";
export type PairOrder = "anyOrder" | "getBeforeSet" | "setBeforeGet";
export type Mode = "prefer-top-level" | "prefer-inline";
/**
 * Extension rule configuration; Copy to avoid extra indirection.
 */
export type ExtensionRule = "always" | "never" | "ignorePackages";
export type ImportExtensionsObject =
  | ImportExtensionsConfig
  | {
      [k: string]: ExtensionRule;
    };
/**
 * Action to take for path group overrides.
 *
 * Determines how import extensions are validated for matching bespoke import specifiers.
 */
export type PathGroupAction = "enforce" | "ignore";
export type AbsoluteFirst = "absolute-first" | "disable-absolute-first";
export type MaxDependenciesConfigJson = number | MaxDependenciesConfig;
export type Target = "single" | "any";
export type TestCaseName = "it" | "test";
export type JestFnType = "hook" | "describe" | "test" | "expect" | "jest" | "unknown";
export type DummyRule = AllowWarnDeny | [AllowWarnDeny, ...unknown[]];
export type SnapshotHintMode = "always" | "multi";
export type AltTextElements = "img" | "object" | "area" | 'input[type="image"]';
export type AnchorIsValidAspect = "noHref" | "invalidHref" | "preferButton";
export type Assert = "htmlFor" | "nesting" | "both" | "either";
export type DistractingElement = "marquee" | "blink";
export type MaxClassesPerFileConfigEnum = number | MaxClassesPerFileConfig;
export type MaxDepthConfigEnum = number | MaxDepth;
export type MaxLinesConfigEnum = number | MaxLinesConfig;
export type MaxLinesPerFunctionConfigEnum = number | MaxLinesPerFunctionConfig;
export type MaxNestedCallbacksConfigEnum = number | MaxNestedCallbacks;
export type MaxParamsConfigEnum = number | MaxParamsConfig;
export type CountThis = "always" | "never" | "except-void";
export type MaxStatementsConfigEnum = number | MaxStatementsConfig;
export type NoCondAssignConfig = "except-parens" | "always";
export type CheckLoopsConfig = boolean | CheckLoops;
export type CheckLoops = "all" | "allExceptWhileTrue" | "none";
/**
 * Kinds of functions that can be allowed to be empty.
 */
export type AllowKind =
  | "functions"
  | "arrowFunctions"
  | "generatorFunctions"
  | "methods"
  | "generatorMethods"
  | "getters"
  | "setters"
  | "constructors"
  | "asyncFunctions"
  | "asyncMethods"
  | "privateConstructors"
  | "protectedConstructors"
  | "decoratedFunctions"
  | "overrideMethods";
/**
 * Determines what type of declarations to check.
 */
export type NoInnerDeclarationsConfig = "functions" | "both";
export type BlockScopedFunctions = "allow" | "disallow";
export type NoMagicNumbersNumber = number | string;
export type NoRestrictedImportsConfigEnum = string | RestrictedPath | NoRestrictedImportsConfig;
export type PossiblePaths = string | RestrictedPath;
export type PossiblePatterns = string | RestrictedPattern;
export type NoReturnAssignMode = "always" | "except-parens";
/**
 * Controls how hoisting is handled when checking for shadowing.
 */
export type HoistOption = "all" | "functions" | "functions-and-types" | "never" | "types";
export type LoopType = "WhileStatement" | "DoWhileStatement" | "ForStatement" | "ForInStatement" | "ForOfStatement";
export type NoUnusedVarsConfig = VarsOption | NoUnusedVarsOptions;
export type VarsOption = "all" | "local";
export type ArgsOption = "after-used" | "all" | "none";
export type IgnorePatternFor_String = null | string;
export type CaughtErrorsJson = "all" | "none";
export type NoUnusedVarsFixMode = "off" | "suggestion" | "fix" | "safe-fix";
export type NoUseBeforeDefineConfigJson = Nofunc | NoUseBeforeDefineConfig;
export type Nofunc = "nofunc";
export type Location = "start" | "anywhere";
/**
 * The rule takes a single option - an array of possible callback names - which may include object methods. The default callback names are `callback`, `cb`, `next`.
 */
export type CallbackReturn = string[];
/**
 * The rule takes a single string option: the name of the error parameter.
 *
 * This can be either:
 * - an exact name (e.g. `"err"`, `"error"`)
 * - a regexp pattern (e.g. `"^(err|error)$"`)
 *
 * If the configured name of the error variable begins with a `^` it is considered to be a regexp pattern.
 * Invalid regexp patterns are rejected during configuration parsing.
 *
 * Default: `"err"`.
 */
export type HandleCallbackErrConfig = string;
export type NoMixedRequiresConfig = boolean | NoMixedRequiresOptions;
export type ShorthandType = "always" | "methods" | "properties" | "consistent" | "consistent-as-needed" | "never";
export type Destructuring = "any" | "all";
export type PreferDestructuringOption = PreferDestructuringTargetOption | PreferDestructuringAssignmentConfig;
export type TerminationMethod = string | string[];
export type RadixType = "always" | "as-needed";
export type NativeAllowList = AllKeyword | string[];
export type AllKeyword = "all";
/**
 * A forbidden prop, either as a plain prop name string or with options.
 */
export type ForbidItem = string | ForbidItemObject;
/**
 * A forbidden prop, either as a plain prop name string or with options.
 */
export type ForbidDomPropsItem = string | PropWithOptions;
/**
 * A forbidden element, either as a plain element name or with a custom message.
 */
export type ForbidItem2 =
  | string
  | {
      /**
       * The element name to forbid.
       */
      element: string;
      /**
       * The message to display when this element is found
       */
      message?: string;
    };
export type EnforceBooleanAttribute = "always" | "never";
export type JsxCurlyBracePresenceConfig = JsxCurlyBracePresenceMode | JsxCurlyBracePresence;
export type JsxCurlyBracePresenceMode = "always" | "never" | "ignore";
export type JsxFilenameExtensionAllowMode = "always" | "as-needed";
export type FragmentMode = "syntax" | "element";
export type EnforceDynamicLinksEnum = "always" | "never";
export type IgnoreEnforceOption = "ignore" | "enforce";
export type AllowedOrDisallowInFunc = "allowed" | "disallow-in-func";
export type RequireFlag = "u" | "v";
export type ImportKind = "none" | "all" | "multiple" | "single";
/**
 * Sorting order for keys. Accepts "asc" for ascending or "desc" for descending.
 */
export type SortOrder = "desc" | "asc";
export type ArrayOption = "array" | "array-simple" | "generic";
export type ReadonlyArrayOption = "array" | "array-simple" | "generic";
export type DirectiveConfigSchema =
  | boolean
  | RequireDescription
  | {
      descriptionFormat?: string;
    };
export type RequireDescription = "allow-with-description";
export type ClassLiteralPropertyStyleOption = "fields" | "getters";
export type PreferGenericType = "constructor" | "type-annotation";
export type ConsistentIndexedObjectStyleConfig = "record" | "index-signature";
export type ArrayLiteralTypeAssertions = "allow" | "allow-as-parameter" | "never";
export type AssertionStyle = "as" | "angle-bracket" | "never";
export type ObjectLiteralTypeAssertions = "allow" | "allow-as-parameter" | "never";
export type ConsistentTypeDefinitionsConfig = "interface" | "type";
export type FixStyle = "separate-type-imports" | "inline-type-imports";
export type Prefer = "type-imports" | "no-type-imports";
export type AccessibilityLevel = "explicit" | "no-public" | "off";
export type MethodSignatureStyleConfig = "property" | "method";
/**
 * Type or value specifier for matching specific declarations
 *
 * Supports four types of specifiers:
 *
 * 1. **String specifier** (deprecated): Universal match by name
 * ```json
 * "Promise"
 * ```
 *
 * 2. **File specifier**: Match types/values declared in local files
 * ```json
 * { "from": "file", "name": "MyType" }
 * { "from": "file", "name": ["Type1", "Type2"] }
 * { "from": "file", "name": "MyType", "path": "./types.ts" }
 * ```
 *
 * 3. **Lib specifier**: Match TypeScript built-in lib types
 * ```json
 * { "from": "lib", "name": "Promise" }
 * { "from": "lib", "name": ["Promise", "PromiseLike"] }
 * ```
 *
 * 4. **Package specifier**: Match types/values from npm packages
 * ```json
 * { "from": "package", "name": "Observable", "package": "rxjs" }
 * { "from": "package", "name": ["Observable", "Subject"], "package": "rxjs" }
 * ```
 */
export type TypeOrValueSpecifier = string | FileSpecifier | LibSpecifier | PackageSpecifier;
export type FileFrom = "file";
/**
 * Name specifier that can be a single string or array of strings
 */
export type NameSpecifier = string | string[];
export type LibFrom = "lib";
export type PackageFrom = "package";
export type AllowInterfaces = "never" | "always" | "with-single-extends";
export type AllowObjectTypes = "never" | "always";
export type AllowInGenericTypeArguments = boolean | string[];
export type ChecksVoidReturn = boolean | ChecksVoidReturnOptions;
/**
 * Represents the different ways a ban config can be specified in JSON.
 * Can be:
 * - `true` - ban with default message
 * - A string - ban with custom message
 * - An object with `message` and optional `fixWith` and `suggest`
 */
export type BanConfigValue =
  | (
      | True
      | string
      | {
          /**
           * Replacement type for automatic fixing. Applied directly with `--fix`.
           */
          fixWith?: string;
          /**
           * Custom message explaining why the type is banned.
           */
          message?: string;
          /**
           * Suggested replacement types for manual review. Shown as editor suggestions.
           */
          suggest?: string[];
        }
    )
  | undefined;
export type True = true;
/**
 * Represents the different ways `allowConstantLoopConditions` can be specified in JSON.
 * Can be:
 * - `true` or `false`
 * - A string enum (`"never"`, `"always"`, `"only-allowed-literals"`)
 */
export type AllowConstantLoopConditions = boolean | AllowConstantLoopConditionsMode;
export type AllowConstantLoopConditionsMode = "never" | "always" | "only-allowed-literals";
export type Modifier =
  | "private"
  | "private readonly"
  | "protected"
  | "protected readonly"
  | "public"
  | "public readonly"
  | "readonly";
export type Prefer2 = "class-property" | "parameter-property";
/**
 * Represents the different ways `ignorePrimitives` can be specified in JSON.
 * Can be:
 * - `true` - ignore all primitive types
 * - An object specifying which primitives to ignore
 */
export type IgnorePrimitives = boolean | IgnorePrimitivesOptions;
export type AllowSingleElementEquality = "always" | "never";
export type ReturnAwaitOption = "in-try-catch" | "always" | "error-handling-correctness-only" | "never";
export type PathOption = "always" | "never";
export type TypesOption = "always" | "never" | "prefer-import";
export type BomOptionType = "always" | "never";
export type NonZero = "greater-than" | "not-equal";
export type ModuleStylesOverride =
  | (
      | false
      | {
          /**
           * Whether default imports or whole-module `require()` assignments are allowed for this module.
           *
           * With `{ "styles": { "chalk": { "default": true } } }`, this is valid:
           * ```js
           * import chalk from "chalk";
           * ```
           */
          default?: boolean;
          /**
           * Whether named imports or destructured `require()` calls are allowed for this module.
           *
           * With `{ "styles": { "node:util": { "named": true } } }`, this is valid:
           * ```js
           * import {promisify} from "node:util";
           * ```
           */
          named?: boolean;
          /**
           * Whether namespace imports or whole-module `require()` assignments are allowed for this module.
           *
           * With `{ "styles": { "node:fs": { "namespace": true } } }`, this is valid:
           * ```js
           * import * as fs from "node:fs";
           * ```
           */
          namespace?: boolean;
          /**
           * Whether side-effect imports or unassigned dynamic imports/requires are allowed for this module.
           *
           * With `{ "styles": { "polyfill": { "unassigned": true } } }`, this is valid:
           * ```js
           * import "polyfill";
           * ```
           */
          unassigned?: boolean;
        }
    )
  | undefined;
export type NoInstanceofBuiltinsStrategy = "strict" | "loose";
export type PreferTernaryOption = "always" | "only-single-line";
export type RelativeUrlStyleConfig = "never" | "always";
export type SwitchCaseBracesConfig = "always" | "avoid";
export type MemberNames = "for" | "each";
export type VitestFnName = "vi" | "vitest";
export type CaseType = "PascalCase" | "kebab-case";
export type DeclarationStyle = "type-based" | "type-literal" | "runtime";
export type DeclarationStyle2 = "type-based" | "runtime";
export type Destructure = "only-when-assigned" | "always" | "never";
export type NextTickOption = "promise" | "callback";
export type CaseType2 = "camelCase" | "snake_case";
export type AllowYoda = "never" | "always";
export type OxlintOverrides = OxlintOverride[];
export type JestVersionSchema = number | string;
export type TagNamePreference =
  | (
      | string
      | {
          message: string;
          replacement: string;
          [k: string]: unknown | undefined;
        }
      | {
          message: string;
          [k: string]: unknown | undefined;
        }
      | boolean
    )
  | undefined;
export type OneOrManyFor_String = string | string[];
export type CustomComponent =
  | string
  | {
      attribute: string;
      name: string;
      [k: string]: unknown | undefined;
    }
  | {
      attributes: string[];
      name: string;
      [k: string]: unknown | undefined;
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
  [k: string]: boolean | undefined;
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
  [k: string]: GlobalValue | undefined;
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
  "accessor-pairs"?: RuleNoConfig | [AllowWarnDeny, AccessorPairsConfig];
  "array-callback-return"?: RuleNoConfig | [AllowWarnDeny, ArrayCallbackReturn];
  "arrow-body-style"?: RuleNoConfig | [AllowWarnDeny, Mode2] | [AllowWarnDeny, Mode2, ArrowBodyStyleConfig];
  "block-scoped-var"?: RuleNoConfig;
  "capitalized-comments"?: RuleNoConfig | [AllowWarnDeny, AlwaysNever] | [AllowWarnDeny, AlwaysNever, OptionsJsonEnum];
  "class-methods-use-this"?: RuleNoConfig | [AllowWarnDeny, ClassMethodsUseThisConfig];
  complexity?: RuleNoConfig | [AllowWarnDeny, ComplexityConfigEnum];
  "constructor-super"?: RuleNoConfig;
  curly?: RuleNoConfig | [AllowWarnDeny, CurlyType] | [AllowWarnDeny, CurlyType, CurlyConsistent];
  "default-case"?: RuleNoConfig | [AllowWarnDeny, DefaultCaseConfig];
  "default-case-last"?: RuleNoConfig;
  "default-param-last"?: RuleNoConfig;
  eqeqeq?: RuleNoConfig | [AllowWarnDeny, CompareType] | [AllowWarnDeny, CompareType, EqeqeqOptions];
  "for-direction"?: RuleNoConfig;
  "func-name-matching"?:
    | RuleNoConfig
    | (
        | [AllowWarnDeny, FuncNameMatchingMode]
        | [AllowWarnDeny, FuncNameMatchingMode, FuncNameMatchingConfig]
        | [AllowWarnDeny, FuncNameMatchingMode]
        | [AllowWarnDeny, FuncNameMatchingConfig]
      );
  "func-names"?:
    | RuleNoConfig
    | [AllowWarnDeny, FuncNamesConfigType]
    | [AllowWarnDeny, FuncNamesConfigType, FuncNamesGeneratorsConfig];
  "func-style"?: RuleNoConfig | [AllowWarnDeny, Style] | [AllowWarnDeny, Style, FuncStyleConfig];
  "getter-return"?: RuleNoConfig | [AllowWarnDeny, GetterReturn];
  "grouped-accessor-pairs"?:
    | RuleNoConfig
    | [AllowWarnDeny, PairOrder]
    | [AllowWarnDeny, PairOrder, GroupedAccessorPairsConfig];
  "guard-for-in"?: RuleNoConfig;
  "id-length"?: RuleNoConfig | [AllowWarnDeny, IdLengthConfig];
  "id-match"?: RuleNoConfig | [AllowWarnDeny, string] | [AllowWarnDeny, string, IdMatchOptions];
  "import/consistent-type-specifier-style"?: RuleNoConfig | [AllowWarnDeny, Mode];
  "import/default"?: RuleNoConfig;
  "import/export"?: RuleNoConfig;
  "import/exports-last"?: RuleNoConfig;
  "import/extensions"?:
    | RuleNoConfig
    | (
        | [AllowWarnDeny, ExtensionRule]
        | [AllowWarnDeny, ExtensionRule, ImportExtensionsObject]
        | [AllowWarnDeny, ExtensionRule]
        | [AllowWarnDeny, ImportExtensionsObject]
      );
  "import/first"?: RuleNoConfig | [AllowWarnDeny, AbsoluteFirst];
  "import/group-exports"?: RuleNoConfig;
  "import/max-dependencies"?: RuleNoConfig | [AllowWarnDeny, MaxDependenciesConfigJson];
  "import/named"?: RuleNoConfig;
  "import/namespace"?: RuleNoConfig | [AllowWarnDeny, Namespace];
  "import/newline-after-import"?: RuleNoConfig | [AllowWarnDeny, NewlineAfterImport];
  "import/no-absolute-path"?: RuleNoConfig | [AllowWarnDeny, NoAbsolutePath];
  "import/no-amd"?: RuleNoConfig;
  "import/no-anonymous-default-export"?: RuleNoConfig | [AllowWarnDeny, NoAnonymousDefaultExport];
  "import/no-commonjs"?: RuleNoConfig | [AllowWarnDeny, NoCommonjs];
  "import/no-cycle"?: RuleNoConfig | [AllowWarnDeny, NoCycle];
  "import/no-default-export"?: RuleNoConfig;
  "import/no-duplicates"?: RuleNoConfig | [AllowWarnDeny, NoDuplicates];
  "import/no-dynamic-require"?: RuleNoConfig | [AllowWarnDeny, NoDynamicRequire];
  "import/no-empty-named-blocks"?: RuleNoConfig;
  "import/no-mutable-exports"?: RuleNoConfig;
  "import/no-named-as-default"?: RuleNoConfig;
  "import/no-named-as-default-member"?: RuleNoConfig;
  "import/no-named-default"?: RuleNoConfig;
  "import/no-named-export"?: RuleNoConfig;
  "import/no-namespace"?: RuleNoConfig | [AllowWarnDeny, NoNamespaceConfig];
  "import/no-nodejs-modules"?: RuleNoConfig | [AllowWarnDeny, NoNodejsModulesConfig];
  "import/no-relative-parent-imports"?: RuleNoConfig;
  "import/no-self-import"?: RuleNoConfig;
  "import/no-unassigned-import"?: RuleNoConfig | [AllowWarnDeny, NoUnassignedImportConfig];
  "import/no-webpack-loader-syntax"?: RuleNoConfig;
  "import/prefer-default-export"?: RuleNoConfig | [AllowWarnDeny, PreferDefaultExport];
  "import/unambiguous"?: RuleNoConfig;
  "init-declarations"?:
    | RuleNoConfig
    | [AllowWarnDeny, AlwaysNever]
    | [AllowWarnDeny, AlwaysNever, InitDeclarationsConfig];
  "jest/consistent-test-it"?: RuleNoConfig | [AllowWarnDeny, ConsistentTestItConfig];
  "jest/expect-expect"?: RuleNoConfig | [AllowWarnDeny, ExpectExpectConfig];
  "jest/max-expects"?: RuleNoConfig | [AllowWarnDeny, MaxExpectsConfig];
  "jest/max-nested-describe"?: RuleNoConfig | [AllowWarnDeny, MaxNestedDescribeConfig];
  "jest/no-alias-methods"?: RuleNoConfig;
  "jest/no-commented-out-tests"?: RuleNoConfig;
  "jest/no-conditional-expect"?: RuleNoConfig;
  "jest/no-conditional-in-test"?: RuleNoConfig;
  "jest/no-confusing-set-timeout"?: RuleNoConfig;
  "jest/no-deprecated-functions"?: RuleNoConfig | [AllowWarnDeny, NoDeprecatedFunctionsConfig];
  "jest/no-disabled-tests"?: RuleNoConfig;
  "jest/no-done-callback"?: RuleNoConfig;
  "jest/no-duplicate-hooks"?: RuleNoConfig;
  "jest/no-export"?: RuleNoConfig;
  "jest/no-focused-tests"?: RuleNoConfig;
  "jest/no-hooks"?: RuleNoConfig | [AllowWarnDeny, NoHooksConfig];
  "jest/no-identical-title"?: RuleNoConfig;
  "jest/no-interpolation-in-snapshots"?: RuleNoConfig;
  "jest/no-jasmine-globals"?: RuleNoConfig;
  "jest/no-large-snapshots"?: RuleNoConfig | [AllowWarnDeny, NoLargeSnapshotsConfig];
  "jest/no-mocks-import"?: RuleNoConfig;
  "jest/no-restricted-jest-methods"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedTestMethodsConfig];
  "jest/no-restricted-matchers"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedMatchersConfig];
  "jest/no-standalone-expect"?: RuleNoConfig | [AllowWarnDeny, NoStandaloneExpectConfig];
  "jest/no-test-prefixes"?: RuleNoConfig;
  "jest/no-test-return-statement"?: RuleNoConfig;
  "jest/no-unneeded-async-expect-function"?: RuleNoConfig;
  "jest/no-untyped-mock-factory"?: RuleNoConfig;
  "jest/padding-around-after-all-blocks"?: RuleNoConfig;
  "jest/padding-around-test-blocks"?: RuleNoConfig;
  "jest/prefer-called-with"?: RuleNoConfig;
  "jest/prefer-comparison-matcher"?: RuleNoConfig;
  "jest/prefer-each"?: RuleNoConfig;
  "jest/prefer-ending-with-an-expect"?: RuleNoConfig | [AllowWarnDeny, PreferEndingWithAnExpectConfig];
  "jest/prefer-equality-matcher"?: RuleNoConfig;
  "jest/prefer-expect-assertions"?: RuleNoConfig | [AllowWarnDeny, PreferExpectAssertionsConfig];
  "jest/prefer-expect-resolves"?: RuleNoConfig;
  "jest/prefer-hooks-in-order"?: RuleNoConfig;
  "jest/prefer-hooks-on-top"?: RuleNoConfig;
  "jest/prefer-importing-jest-globals"?: RuleNoConfig | [AllowWarnDeny, PreferImportingJestGlobalsConfig];
  "jest/prefer-jest-mocked"?: RuleNoConfig;
  "jest/prefer-lowercase-title"?: DummyRule;
  "jest/prefer-mock-promise-shorthand"?: RuleNoConfig;
  "jest/prefer-mock-return-shorthand"?: RuleNoConfig;
  "jest/prefer-snapshot-hint"?: RuleNoConfig | [AllowWarnDeny, SnapshotHintMode];
  "jest/prefer-spy-on"?: RuleNoConfig;
  "jest/prefer-strict-equal"?: RuleNoConfig;
  "jest/prefer-to-be"?: RuleNoConfig;
  "jest/prefer-to-contain"?: RuleNoConfig;
  "jest/prefer-to-have-been-called"?: RuleNoConfig;
  "jest/prefer-to-have-been-called-times"?: RuleNoConfig;
  "jest/prefer-to-have-length"?: RuleNoConfig;
  "jest/prefer-todo"?: RuleNoConfig;
  "jest/require-hook"?: RuleNoConfig | [AllowWarnDeny, RequireHookConfig];
  "jest/require-to-throw-message"?: RuleNoConfig;
  "jest/require-top-level-describe"?: RuleNoConfig | [AllowWarnDeny, RequireTopLevelDescribeConfig];
  "jest/valid-describe-callback"?: RuleNoConfig;
  "jest/valid-expect"?: RuleNoConfig | [AllowWarnDeny, ValidExpectConfig];
  "jest/valid-expect-in-promise"?: RuleNoConfig;
  "jest/valid-title"?: DummyRule;
  "jsdoc/check-access"?: RuleNoConfig;
  "jsdoc/check-property-names"?: RuleNoConfig;
  "jsdoc/check-tag-names"?: RuleNoConfig | [AllowWarnDeny, CheckTagNamesConfig];
  "jsdoc/empty-tags"?: RuleNoConfig | [AllowWarnDeny, EmptyTagsConfig];
  "jsdoc/implements-on-classes"?: RuleNoConfig;
  "jsdoc/no-defaults"?: RuleNoConfig | [AllowWarnDeny, NoDefaultsConfig];
  "jsdoc/require-param"?: DummyRule;
  "jsdoc/require-param-description"?: RuleNoConfig | [AllowWarnDeny, RequireParamDescriptionConfig];
  "jsdoc/require-param-name"?: RuleNoConfig;
  "jsdoc/require-param-type"?: RuleNoConfig | [AllowWarnDeny, RequireParamTypeConfig];
  "jsdoc/require-property"?: RuleNoConfig;
  "jsdoc/require-property-description"?: RuleNoConfig;
  "jsdoc/require-property-name"?: RuleNoConfig;
  "jsdoc/require-property-type"?: RuleNoConfig;
  "jsdoc/require-returns"?: RuleNoConfig | [AllowWarnDeny, RequireReturnsConfig];
  "jsdoc/require-returns-description"?: RuleNoConfig;
  "jsdoc/require-returns-type"?: RuleNoConfig;
  "jsdoc/require-throws-description"?: RuleNoConfig;
  "jsdoc/require-throws-type"?: RuleNoConfig;
  "jsdoc/require-yields"?: RuleNoConfig | [AllowWarnDeny, RequireYieldsConfig];
  "jsdoc/require-yields-description"?: RuleNoConfig;
  "jsdoc/require-yields-type"?: RuleNoConfig;
  "jsx-a11y/alt-text"?: RuleNoConfig | [AllowWarnDeny, AltTextConfigSchema];
  "jsx-a11y/anchor-ambiguous-text"?: RuleNoConfig | [AllowWarnDeny, AnchorAmbiguousTextConfig];
  "jsx-a11y/anchor-has-content"?: RuleNoConfig;
  "jsx-a11y/anchor-is-valid"?: RuleNoConfig | [AllowWarnDeny, AnchorIsValidConfig];
  "jsx-a11y/aria-activedescendant-has-tabindex"?: RuleNoConfig;
  "jsx-a11y/aria-props"?: RuleNoConfig;
  "jsx-a11y/aria-proptypes"?: RuleNoConfig;
  "jsx-a11y/aria-role"?: RuleNoConfig | [AllowWarnDeny, AriaRoleConfig];
  "jsx-a11y/aria-unsupported-elements"?: RuleNoConfig;
  "jsx-a11y/autocomplete-valid"?: RuleNoConfig | [AllowWarnDeny, AutocompleteValidConfig];
  "jsx-a11y/click-events-have-key-events"?: RuleNoConfig;
  "jsx-a11y/control-has-associated-label"?: RuleNoConfig | [AllowWarnDeny, ControlHasAssociatedLabelConfig];
  "jsx-a11y/heading-has-content"?: RuleNoConfig | [AllowWarnDeny, HeadingHasContentConfig];
  "jsx-a11y/html-has-lang"?: RuleNoConfig;
  "jsx-a11y/iframe-has-title"?: RuleNoConfig;
  "jsx-a11y/img-redundant-alt"?: RuleNoConfig | [AllowWarnDeny, ImgRedundantAltConfig];
  "jsx-a11y/interactive-supports-focus"?: RuleNoConfig | [AllowWarnDeny, InteractiveSupportsFocusConfig];
  "jsx-a11y/label-has-associated-control"?: RuleNoConfig | [AllowWarnDeny, LabelHasAssociatedControlConfig];
  "jsx-a11y/lang"?: RuleNoConfig;
  "jsx-a11y/media-has-caption"?: RuleNoConfig | [AllowWarnDeny, MediaHasCaptionConfig];
  "jsx-a11y/mouse-events-have-key-events"?: RuleNoConfig | [AllowWarnDeny, MouseEventsHaveKeyEventsConfig];
  "jsx-a11y/no-access-key"?: RuleNoConfig;
  "jsx-a11y/no-aria-hidden-on-focusable"?: RuleNoConfig;
  "jsx-a11y/no-autofocus"?: RuleNoConfig | [AllowWarnDeny, NoAutofocus];
  "jsx-a11y/no-distracting-elements"?: RuleNoConfig | [AllowWarnDeny, NoDistractingElementsConfig];
  "jsx-a11y/no-interactive-element-to-noninteractive-role"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoInteractiveElementToNoninteractiveRoleConfig];
  "jsx-a11y/no-noninteractive-element-interactions"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoNoninteractiveElementInteractionsConfig];
  "jsx-a11y/no-noninteractive-element-to-interactive-role"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoNoninteractiveElementToInteractiveRoleConfig];
  "jsx-a11y/no-noninteractive-tabindex"?: RuleNoConfig | [AllowWarnDeny, NoNoninteractiveTabindexConfig];
  "jsx-a11y/no-redundant-roles"?: RuleNoConfig | [AllowWarnDeny, NoRedundantRolesConfig];
  "jsx-a11y/no-static-element-interactions"?: RuleNoConfig | [AllowWarnDeny, NoStaticElementInteractionsConfig];
  "jsx-a11y/prefer-tag-over-role"?: RuleNoConfig;
  "jsx-a11y/role-has-required-aria-props"?: RuleNoConfig;
  "jsx-a11y/role-supports-aria-props"?: RuleNoConfig;
  "jsx-a11y/scope"?: RuleNoConfig;
  "jsx-a11y/tabindex-no-positive"?: RuleNoConfig;
  "logical-assignment-operators"?:
    | RuleNoConfig
    | [AllowWarnDeny, AlwaysNever]
    | [AllowWarnDeny, AlwaysNever, LogicalAssignmentOperatorsConfig];
  "max-classes-per-file"?: RuleNoConfig | [AllowWarnDeny, MaxClassesPerFileConfigEnum];
  "max-depth"?: RuleNoConfig | [AllowWarnDeny, MaxDepthConfigEnum];
  "max-lines"?: RuleNoConfig | [AllowWarnDeny, MaxLinesConfigEnum];
  "max-lines-per-function"?: RuleNoConfig | [AllowWarnDeny, MaxLinesPerFunctionConfigEnum];
  "max-nested-callbacks"?: RuleNoConfig | [AllowWarnDeny, MaxNestedCallbacksConfigEnum];
  "max-params"?: RuleNoConfig | [AllowWarnDeny, MaxParamsConfigEnum];
  "max-statements"?: RuleNoConfig | [AllowWarnDeny, MaxStatementsConfigEnum];
  "new-cap"?: RuleNoConfig | [AllowWarnDeny, NewCapConfig];
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
  "no-bitwise"?: RuleNoConfig | [AllowWarnDeny, NoBitwiseConfig];
  "no-caller"?: RuleNoConfig;
  "no-case-declarations"?: RuleNoConfig;
  "no-class-assign"?: RuleNoConfig;
  "no-compare-neg-zero"?: RuleNoConfig;
  "no-cond-assign"?: RuleNoConfig | [AllowWarnDeny, NoCondAssignConfig];
  "no-console"?: RuleNoConfig | [AllowWarnDeny, NoConsoleConfig];
  "no-const-assign"?: RuleNoConfig;
  "no-constant-binary-expression"?: RuleNoConfig | [AllowWarnDeny, NoConstantBinaryExpressionConfig];
  "no-constant-condition"?: RuleNoConfig | [AllowWarnDeny, NoConstantCondition];
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
  "no-duplicate-imports"?: RuleNoConfig | [AllowWarnDeny, NoDuplicateImports];
  "no-else-return"?: RuleNoConfig | [AllowWarnDeny, NoElseReturn];
  "no-empty"?: RuleNoConfig | [AllowWarnDeny, NoEmpty];
  "no-empty-character-class"?: RuleNoConfig;
  "no-empty-function"?: RuleNoConfig | [AllowWarnDeny, NoEmptyFunctionConfig];
  "no-empty-pattern"?: RuleNoConfig | [AllowWarnDeny, NoEmptyPattern];
  "no-empty-static-block"?: RuleNoConfig;
  "no-eq-null"?: RuleNoConfig;
  "no-eval"?: RuleNoConfig | [AllowWarnDeny, NoEval];
  "no-ex-assign"?: RuleNoConfig;
  "no-extend-native"?: RuleNoConfig | [AllowWarnDeny, NoExtendNativeConfig];
  "no-extra-bind"?: RuleNoConfig;
  "no-extra-boolean-cast"?: RuleNoConfig | [AllowWarnDeny, NoExtraBooleanCast];
  "no-extra-label"?: RuleNoConfig;
  "no-fallthrough"?: RuleNoConfig | [AllowWarnDeny, NoFallthroughConfig];
  "no-func-assign"?: RuleNoConfig;
  "no-global-assign"?: RuleNoConfig | [AllowWarnDeny, NoGlobalAssignConfig];
  "no-implicit-coercion"?: RuleNoConfig | [AllowWarnDeny, NoImplicitCoercionConfig];
  "no-implicit-globals"?: RuleNoConfig | [AllowWarnDeny, NoImplicitGlobalsConfig];
  "no-implied-eval"?: RuleNoConfig;
  "no-import-assign"?: RuleNoConfig;
  "no-inline-comments"?: RuleNoConfig | [AllowWarnDeny, NoInlineCommentsConfig];
  "no-inner-declarations"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoInnerDeclarationsConfig]
    | [AllowWarnDeny, NoInnerDeclarationsConfig, NoInnerDeclarationsOptions];
  "no-invalid-regexp"?: RuleNoConfig | [AllowWarnDeny, NoInvalidRegexpConfig];
  "no-irregular-whitespace"?: RuleNoConfig | [AllowWarnDeny, NoIrregularWhitespaceConfig];
  "no-iterator"?: RuleNoConfig;
  "no-label-var"?: RuleNoConfig;
  "no-labels"?: RuleNoConfig | [AllowWarnDeny, NoLabels];
  "no-lone-blocks"?: RuleNoConfig;
  "no-lonely-if"?: RuleNoConfig;
  "no-loop-func"?: RuleNoConfig;
  "no-loss-of-precision"?: RuleNoConfig;
  "no-magic-numbers"?: RuleNoConfig | [AllowWarnDeny, NoMagicNumbersConfig];
  "no-misleading-character-class"?: RuleNoConfig | [AllowWarnDeny, NoMisleadingCharacterClass];
  "no-multi-assign"?: RuleNoConfig | [AllowWarnDeny, NoMultiAssign];
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
  "no-param-reassign"?: RuleNoConfig | [AllowWarnDeny, NoParamReassignConfig];
  "no-plusplus"?: RuleNoConfig | [AllowWarnDeny, NoPlusplus];
  "no-promise-executor-return"?: RuleNoConfig | [AllowWarnDeny, NoPromiseExecutorReturnConfig];
  "no-proto"?: RuleNoConfig;
  "no-prototype-builtins"?: RuleNoConfig;
  "no-redeclare"?: RuleNoConfig | [AllowWarnDeny, NoRedeclare];
  "no-regex-spaces"?: RuleNoConfig;
  "no-restricted-exports"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedExportsConfig];
  "no-restricted-globals"?: DummyRule;
  "no-restricted-imports"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoRestrictedImportsConfigEnum, ...NoRestrictedImportsConfigEnum[]];
  "no-restricted-properties"?: RuleNoConfig | [AllowWarnDeny, PropertyDetails, ...PropertyDetails[]];
  "no-return-assign"?: RuleNoConfig | [AllowWarnDeny, NoReturnAssignMode];
  "no-script-url"?: RuleNoConfig;
  "no-self-assign"?: RuleNoConfig | [AllowWarnDeny, NoSelfAssign];
  "no-self-compare"?: RuleNoConfig;
  "no-sequences"?: RuleNoConfig | [AllowWarnDeny, NoSequences];
  "no-setter-return"?: RuleNoConfig;
  "no-shadow"?: RuleNoConfig | [AllowWarnDeny, NoShadowConfig];
  "no-shadow-restricted-names"?: RuleNoConfig | [AllowWarnDeny, NoShadowRestrictedNamesConfig];
  "no-sparse-arrays"?: RuleNoConfig;
  "no-template-curly-in-string"?: RuleNoConfig;
  "no-ternary"?: RuleNoConfig;
  "no-this-before-super"?: RuleNoConfig;
  "no-throw-literal"?: RuleNoConfig;
  "no-unassigned-vars"?: RuleNoConfig;
  "no-undef"?: RuleNoConfig | [AllowWarnDeny, NoUndef];
  "no-undefined"?: RuleNoConfig;
  "no-underscore-dangle"?: RuleNoConfig | [AllowWarnDeny, NoUnderscoreDangleConfig];
  "no-unexpected-multiline"?: RuleNoConfig;
  "no-unmodified-loop-condition"?: RuleNoConfig;
  "no-unneeded-ternary"?: RuleNoConfig | [AllowWarnDeny, NoUnneededTernary];
  "no-unreachable"?: RuleNoConfig;
  "no-unreachable-loop"?: RuleNoConfig | [AllowWarnDeny, NoUnreachableLoopConfig];
  "no-unsafe-finally"?: RuleNoConfig;
  "no-unsafe-negation"?: RuleNoConfig | [AllowWarnDeny, NoUnsafeNegation];
  "no-unsafe-optional-chaining"?: RuleNoConfig | [AllowWarnDeny, NoUnsafeOptionalChaining];
  "no-unused-expressions"?: RuleNoConfig | [AllowWarnDeny, NoUnusedExpressionsConfig];
  "no-unused-labels"?: RuleNoConfig;
  "no-unused-private-class-members"?: RuleNoConfig;
  "no-unused-vars"?: RuleNoConfig | [AllowWarnDeny, NoUnusedVarsConfig];
  "no-use-before-define"?: RuleNoConfig | [AllowWarnDeny, NoUseBeforeDefineConfigJson];
  "no-useless-assignment"?: RuleNoConfig;
  "no-useless-backreference"?: RuleNoConfig;
  "no-useless-call"?: RuleNoConfig;
  "no-useless-catch"?: RuleNoConfig;
  "no-useless-computed-key"?: RuleNoConfig | [AllowWarnDeny, NoUselessComputedKey];
  "no-useless-concat"?: RuleNoConfig;
  "no-useless-constructor"?: RuleNoConfig;
  "no-useless-escape"?: RuleNoConfig | [AllowWarnDeny, NoUselessEscapeConfig];
  "no-useless-rename"?: RuleNoConfig | [AllowWarnDeny, NoUselessRenameConfig];
  "no-useless-return"?: RuleNoConfig;
  "no-var"?: RuleNoConfig;
  "no-void"?: RuleNoConfig | [AllowWarnDeny, NoVoid];
  "no-warning-comments"?: RuleNoConfig | [AllowWarnDeny, NoWarningCommentsConfigJson];
  "no-with"?: RuleNoConfig;
  "node/callback-return"?: RuleNoConfig | [AllowWarnDeny, CallbackReturn];
  "node/global-require"?: RuleNoConfig;
  "node/handle-callback-err"?: RuleNoConfig | [AllowWarnDeny, HandleCallbackErrConfig];
  "node/no-exports-assign"?: RuleNoConfig;
  "node/no-mixed-requires"?: RuleNoConfig | [AllowWarnDeny, NoMixedRequiresConfig];
  "node/no-new-require"?: RuleNoConfig;
  "node/no-path-concat"?: RuleNoConfig;
  "node/no-process-env"?: RuleNoConfig | [AllowWarnDeny, NoProcessEnvConfig];
  "node/no-sync"?: RuleNoConfig | [AllowWarnDeny, NoSyncConfig];
  "object-shorthand"?:
    | RuleNoConfig
    | [AllowWarnDeny, ShorthandType]
    | [AllowWarnDeny, ShorthandType, ObjectShorthandOptions];
  "operator-assignment"?: RuleNoConfig | [AllowWarnDeny, AlwaysNever];
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
  "oxc/no-async-endpoint-handlers"?: RuleNoConfig | [AllowWarnDeny, NoAsyncEndpointHandlersConfig];
  "oxc/no-barrel-file"?: RuleNoConfig | [AllowWarnDeny, NoBarrelFile];
  "oxc/no-const-enum"?: RuleNoConfig;
  "oxc/no-map-spread"?: RuleNoConfig | [AllowWarnDeny, NoMapSpreadConfig];
  "oxc/no-optional-chaining"?: RuleNoConfig | [AllowWarnDeny, NoOptionalChainingConfig];
  "oxc/no-rest-spread-properties"?: RuleNoConfig | [AllowWarnDeny, NoRestSpreadPropertiesOptions];
  "oxc/no-this-in-exported-function"?: RuleNoConfig;
  "oxc/number-arg-out-of-range"?: RuleNoConfig;
  "oxc/only-used-in-recursion"?: RuleNoConfig;
  "oxc/uninvoked-array-callback"?: RuleNoConfig;
  "prefer-arrow-callback"?: RuleNoConfig | [AllowWarnDeny, PreferArrowCallbackConfig];
  "prefer-const"?: RuleNoConfig | [AllowWarnDeny, PreferConstConfig];
  "prefer-destructuring"?:
    | RuleNoConfig
    | [AllowWarnDeny, PreferDestructuringOption]
    | [AllowWarnDeny, PreferDestructuringOption, PreferDestructuringRenamedPropertiesConfig];
  "prefer-exponentiation-operator"?: RuleNoConfig;
  "prefer-named-capture-group"?: RuleNoConfig;
  "prefer-numeric-literals"?: RuleNoConfig;
  "prefer-object-has-own"?: RuleNoConfig;
  "prefer-object-spread"?: RuleNoConfig;
  "prefer-promise-reject-errors"?: RuleNoConfig | [AllowWarnDeny, PreferPromiseRejectErrors];
  "prefer-regex-literals"?: RuleNoConfig | [AllowWarnDeny, PreferRegexLiteralsConfig];
  "prefer-rest-params"?: RuleNoConfig;
  "prefer-spread"?: RuleNoConfig;
  "prefer-template"?: RuleNoConfig;
  "preserve-caught-error"?: RuleNoConfig | [AllowWarnDeny, PreserveCaughtErrorOptions];
  "promise/always-return"?: RuleNoConfig | [AllowWarnDeny, AlwaysReturnConfig];
  "promise/avoid-new"?: RuleNoConfig;
  "promise/catch-or-return"?: RuleNoConfig | [AllowWarnDeny, CatchOrReturnConfig];
  "promise/no-callback-in-promise"?: RuleNoConfig | [AllowWarnDeny, NoCallbackInPromiseConfig];
  "promise/no-multiple-resolved"?: RuleNoConfig;
  "promise/no-nesting"?: RuleNoConfig;
  "promise/no-new-statics"?: RuleNoConfig;
  "promise/no-promise-in-callback"?: RuleNoConfig | [AllowWarnDeny, NoPromiseInCallbackConfig];
  "promise/no-return-in-finally"?: RuleNoConfig;
  "promise/no-return-wrap"?: RuleNoConfig | [AllowWarnDeny, NoReturnWrap];
  "promise/param-names"?: RuleNoConfig | [AllowWarnDeny, ParamNamesConfig];
  "promise/prefer-await-to-callbacks"?: RuleNoConfig;
  "promise/prefer-await-to-then"?: RuleNoConfig | [AllowWarnDeny, PreferAwaitToThenConfig];
  "promise/prefer-catch"?: RuleNoConfig;
  "promise/spec-only"?: RuleNoConfig | [AllowWarnDeny, SpecOnlyConfig];
  "promise/valid-params"?: RuleNoConfig;
  radix?: RuleNoConfig | [AllowWarnDeny, RadixType];
  "react-perf/jsx-no-jsx-as-prop"?: RuleNoConfig | [AllowWarnDeny, ReactPerfConfig];
  "react-perf/jsx-no-new-array-as-prop"?: RuleNoConfig | [AllowWarnDeny, ReactPerfConfig];
  "react-perf/jsx-no-new-function-as-prop"?: RuleNoConfig | [AllowWarnDeny, ReactPerfConfig];
  "react-perf/jsx-no-new-object-as-prop"?: RuleNoConfig | [AllowWarnDeny, ReactPerfConfig];
  "react/button-has-type"?: RuleNoConfig | [AllowWarnDeny, ButtonHasType];
  "react/checked-requires-onchange-or-readonly"?: RuleNoConfig | [AllowWarnDeny, CheckedRequiresOnchangeOrReadonly];
  "react/display-name"?: RuleNoConfig | [AllowWarnDeny, DisplayNameConfig];
  "react/exhaustive-deps"?: RuleNoConfig | [AllowWarnDeny, ExhaustiveDepsConfig];
  "react/forbid-component-props"?: RuleNoConfig | [AllowWarnDeny, ForbidComponentPropsConfig];
  "react/forbid-dom-props"?: RuleNoConfig | [AllowWarnDeny, ForbidDomPropsConfig];
  "react/forbid-elements"?: RuleNoConfig | [AllowWarnDeny, ForbidElementsConfig];
  "react/forward-ref-uses-ref"?: RuleNoConfig;
  "react/hook-use-state"?: RuleNoConfig | [AllowWarnDeny, HookUseStateConfig];
  "react/iframe-missing-sandbox"?: RuleNoConfig;
  "react/jsx-boolean-value"?:
    | RuleNoConfig
    | [AllowWarnDeny, EnforceBooleanAttribute]
    | [AllowWarnDeny, EnforceBooleanAttribute, JsxBooleanValueOptions];
  "react/jsx-curly-brace-presence"?: RuleNoConfig | [AllowWarnDeny, JsxCurlyBracePresenceConfig];
  "react/jsx-filename-extension"?: RuleNoConfig | [AllowWarnDeny, JsxFilenameExtensionConfig];
  "react/jsx-fragments"?: RuleNoConfig | [AllowWarnDeny, FragmentMode];
  "react/jsx-handler-names"?: RuleNoConfig | [AllowWarnDeny, JsxHandlerNamesConfig];
  "react/jsx-key"?: RuleNoConfig | [AllowWarnDeny, JsxKeyConfig];
  "react/jsx-max-depth"?: RuleNoConfig | [AllowWarnDeny, JsxMaxDepthConfig];
  "react/jsx-no-comment-textnodes"?: RuleNoConfig;
  "react/jsx-no-constructed-context-values"?: RuleNoConfig;
  "react/jsx-no-duplicate-props"?: RuleNoConfig;
  "react/jsx-no-literals"?: RuleNoConfig | [AllowWarnDeny, JsxNoLiteralsConfig];
  "react/jsx-no-script-url"?:
    | RuleNoConfig
    | (
        | [AllowWarnDeny, JsxNoScriptUrlComponent[]]
        | [AllowWarnDeny, JsxNoScriptUrlComponent[], JsxNoScriptUrlOptions]
        | [AllowWarnDeny, JsxNoScriptUrlOptions]
      );
  "react/jsx-no-target-blank"?: RuleNoConfig | [AllowWarnDeny, JsxNoTargetBlank];
  "react/jsx-no-undef"?: RuleNoConfig;
  "react/jsx-no-useless-fragment"?: RuleNoConfig | [AllowWarnDeny, JsxNoUselessFragment];
  "react/jsx-pascal-case"?: RuleNoConfig | [AllowWarnDeny, JsxPascalCaseConfig];
  "react/jsx-props-no-spread-multi"?: RuleNoConfig;
  "react/jsx-props-no-spreading"?: RuleNoConfig | [AllowWarnDeny, JsxPropsNoSpreadingConfig];
  "react/no-array-index-key"?: RuleNoConfig;
  "react/no-children-prop"?: RuleNoConfig;
  "react/no-clone-element"?: RuleNoConfig;
  "react/no-danger"?: RuleNoConfig;
  "react/no-danger-with-children"?: RuleNoConfig;
  "react/no-did-mount-set-state"?: RuleNoConfig | [AllowWarnDeny, AllowedOrDisallowInFunc];
  "react/no-did-update-set-state"?: RuleNoConfig | [AllowWarnDeny, AllowedOrDisallowInFunc];
  "react/no-direct-mutation-state"?: RuleNoConfig;
  "react/no-find-dom-node"?: RuleNoConfig;
  "react/no-is-mounted"?: RuleNoConfig;
  "react/no-multi-comp"?: RuleNoConfig | [AllowWarnDeny, NoMultiCompConfig];
  "react/no-namespace"?: RuleNoConfig;
  "react/no-object-type-as-default-prop"?: RuleNoConfig;
  "react/no-react-children"?: RuleNoConfig;
  "react/no-redundant-should-component-update"?: RuleNoConfig;
  "react/no-render-return-value"?: RuleNoConfig;
  "react/no-set-state"?: RuleNoConfig;
  "react/no-string-refs"?: RuleNoConfig | [AllowWarnDeny, NoStringRefs];
  "react/no-this-in-sfc"?: RuleNoConfig;
  "react/no-unescaped-entities"?: RuleNoConfig;
  "react/no-unknown-property"?: RuleNoConfig | [AllowWarnDeny, NoUnknownPropertyConfig];
  "react/no-unsafe"?: RuleNoConfig | [AllowWarnDeny, NoUnsafeConfig];
  "react/no-unstable-nested-components"?: RuleNoConfig | [AllowWarnDeny, NoUnstableNestedComponentsConfig];
  "react/no-will-update-set-state"?: RuleNoConfig | [AllowWarnDeny, AllowedOrDisallowInFunc];
  "react/only-export-components"?: RuleNoConfig | [AllowWarnDeny, OnlyExportComponentsConfig];
  "react/prefer-es6-class"?: RuleNoConfig | [AllowWarnDeny, AlwaysNever];
  "react/prefer-function-component"?: RuleNoConfig | [AllowWarnDeny, PreferFunctionComponent];
  "react/react-compiler"?: RuleNoConfig | [AllowWarnDeny, ReactCompilerConfig];
  "react/react-in-jsx-scope"?: RuleNoConfig;
  "react/require-render-return"?: RuleNoConfig;
  "react/rules-of-hooks"?: RuleNoConfig;
  "react/self-closing-comp"?: RuleNoConfig | [AllowWarnDeny, SelfClosingComp];
  "react/state-in-constructor"?: RuleNoConfig | [AllowWarnDeny, AlwaysNever];
  "react/style-prop-object"?: RuleNoConfig | [AllowWarnDeny, StylePropObjectConfig];
  "react/void-dom-elements-no-children"?: RuleNoConfig;
  "require-await"?: RuleNoConfig;
  "require-unicode-regexp"?: RuleNoConfig | [AllowWarnDeny, RequireUnicodeRegexpConfig];
  "require-yield"?: RuleNoConfig;
  "sort-imports"?: RuleNoConfig | [AllowWarnDeny, SortImportsOptions];
  "sort-keys"?: RuleNoConfig | [AllowWarnDeny, SortOrder] | [AllowWarnDeny, SortOrder, SortKeysOptions];
  "sort-vars"?: RuleNoConfig | [AllowWarnDeny, SortVars];
  "symbol-description"?: RuleNoConfig;
  "typescript/adjacent-overload-signatures"?: RuleNoConfig;
  "typescript/array-type"?: RuleNoConfig | [AllowWarnDeny, ArrayTypeConfig];
  "typescript/await-thenable"?: RuleNoConfig;
  "typescript/ban-ts-comment"?: RuleNoConfig | [AllowWarnDeny, BanTsCommentConfig];
  "typescript/ban-tslint-comment"?: RuleNoConfig;
  "typescript/ban-types"?: RuleNoConfig;
  "typescript/class-literal-property-style"?: RuleNoConfig | [AllowWarnDeny, ClassLiteralPropertyStyleOption];
  "typescript/consistent-generic-constructors"?: RuleNoConfig | [AllowWarnDeny, PreferGenericType];
  "typescript/consistent-indexed-object-style"?: RuleNoConfig | [AllowWarnDeny, ConsistentIndexedObjectStyleConfig];
  "typescript/consistent-return"?: RuleNoConfig | [AllowWarnDeny, ConsistentReturnConfig];
  "typescript/consistent-type-assertions"?: RuleNoConfig | [AllowWarnDeny, ConsistentTypeAssertionsConfig];
  "typescript/consistent-type-definitions"?: RuleNoConfig | [AllowWarnDeny, ConsistentTypeDefinitionsConfig];
  "typescript/consistent-type-exports"?: RuleNoConfig | [AllowWarnDeny, ConsistentTypeExportsConfig];
  "typescript/consistent-type-imports"?: RuleNoConfig | [AllowWarnDeny, ConsistentTypeImportsConfig];
  "typescript/dot-notation"?: RuleNoConfig | [AllowWarnDeny, DotNotationConfig];
  "typescript/explicit-function-return-type"?: RuleNoConfig | [AllowWarnDeny, ExplicitFunctionReturnTypeConfig];
  "typescript/explicit-member-accessibility"?: RuleNoConfig | [AllowWarnDeny, ExplicitMemberAccessibilityConfig];
  "typescript/explicit-module-boundary-types"?: RuleNoConfig | [AllowWarnDeny, ExplicitModuleBoundaryTypesConfig];
  "typescript/method-signature-style"?: RuleNoConfig | [AllowWarnDeny, MethodSignatureStyleConfig];
  "typescript/no-array-delete"?: RuleNoConfig;
  "typescript/no-base-to-string"?: RuleNoConfig | [AllowWarnDeny, NoBaseToStringConfig];
  "typescript/no-confusing-non-null-assertion"?: RuleNoConfig;
  "typescript/no-confusing-void-expression"?: RuleNoConfig | [AllowWarnDeny, NoConfusingVoidExpressionConfig];
  "typescript/no-deprecated"?: RuleNoConfig | [AllowWarnDeny, NoDeprecatedConfig];
  "typescript/no-duplicate-enum-values"?: RuleNoConfig;
  "typescript/no-duplicate-type-constituents"?: RuleNoConfig | [AllowWarnDeny, NoDuplicateTypeConstituentsConfig];
  "typescript/no-dynamic-delete"?: RuleNoConfig;
  "typescript/no-empty-interface"?: RuleNoConfig | [AllowWarnDeny, NoEmptyInterface];
  "typescript/no-empty-object-type"?: RuleNoConfig | [AllowWarnDeny, NoEmptyObjectTypeConfig];
  "typescript/no-explicit-any"?: RuleNoConfig | [AllowWarnDeny, NoExplicitAny];
  "typescript/no-extra-non-null-assertion"?: RuleNoConfig;
  "typescript/no-extraneous-class"?: RuleNoConfig | [AllowWarnDeny, NoExtraneousClass];
  "typescript/no-floating-promises"?: RuleNoConfig | [AllowWarnDeny, NoFloatingPromisesConfig];
  "typescript/no-for-in-array"?: RuleNoConfig;
  "typescript/no-implied-eval"?: RuleNoConfig;
  "typescript/no-import-type-side-effects"?: RuleNoConfig;
  "typescript/no-inferrable-types"?: RuleNoConfig | [AllowWarnDeny, NoInferrableTypes];
  "typescript/no-invalid-void-type"?: RuleNoConfig | [AllowWarnDeny, NoInvalidVoidTypeConfig];
  "typescript/no-meaningless-void-operator"?: RuleNoConfig | [AllowWarnDeny, NoMeaninglessVoidOperatorConfig];
  "typescript/no-misused-new"?: RuleNoConfig;
  "typescript/no-misused-promises"?: RuleNoConfig | [AllowWarnDeny, NoMisusedPromisesConfig];
  "typescript/no-misused-spread"?: RuleNoConfig | [AllowWarnDeny, NoMisusedSpreadConfig];
  "typescript/no-mixed-enums"?: RuleNoConfig;
  "typescript/no-namespace"?: RuleNoConfig | [AllowWarnDeny, NoNamespace];
  "typescript/no-non-null-asserted-nullish-coalescing"?: RuleNoConfig;
  "typescript/no-non-null-asserted-optional-chain"?: RuleNoConfig;
  "typescript/no-non-null-assertion"?: RuleNoConfig;
  "typescript/no-redundant-type-constituents"?: RuleNoConfig;
  "typescript/no-require-imports"?: RuleNoConfig | [AllowWarnDeny, NoRequireImportsConfig];
  "typescript/no-restricted-types"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedTypesConfig];
  "typescript/no-this-alias"?: RuleNoConfig | [AllowWarnDeny, NoThisAliasConfig];
  "typescript/no-unnecessary-boolean-literal-compare"?:
    | RuleNoConfig
    | [AllowWarnDeny, NoUnnecessaryBooleanLiteralCompareConfig];
  "typescript/no-unnecessary-condition"?: RuleNoConfig | [AllowWarnDeny, NoUnnecessaryConditionConfig];
  "typescript/no-unnecessary-parameter-property-assignment"?: RuleNoConfig;
  "typescript/no-unnecessary-qualifier"?: RuleNoConfig;
  "typescript/no-unnecessary-template-expression"?: RuleNoConfig;
  "typescript/no-unnecessary-type-arguments"?: RuleNoConfig;
  "typescript/no-unnecessary-type-assertion"?: RuleNoConfig | [AllowWarnDeny, NoUnnecessaryTypeAssertionConfig];
  "typescript/no-unnecessary-type-constraint"?: RuleNoConfig;
  "typescript/no-unnecessary-type-conversion"?: RuleNoConfig;
  "typescript/no-unnecessary-type-parameters"?: RuleNoConfig;
  "typescript/no-unsafe-argument"?: RuleNoConfig;
  "typescript/no-unsafe-assignment"?: RuleNoConfig;
  "typescript/no-unsafe-call"?: RuleNoConfig;
  "typescript/no-unsafe-declaration-merging"?: RuleNoConfig;
  "typescript/no-unsafe-enum-comparison"?: RuleNoConfig;
  "typescript/no-unsafe-function-type"?: RuleNoConfig;
  "typescript/no-unsafe-member-access"?: RuleNoConfig | [AllowWarnDeny, NoUnsafeMemberAccessConfig];
  "typescript/no-unsafe-return"?: RuleNoConfig;
  "typescript/no-unsafe-type-assertion"?: RuleNoConfig;
  "typescript/no-unsafe-unary-minus"?: RuleNoConfig;
  "typescript/no-useless-default-assignment"?: RuleNoConfig;
  "typescript/no-useless-empty-export"?: RuleNoConfig;
  "typescript/no-var-requires"?: RuleNoConfig;
  "typescript/no-wrapper-object-types"?: RuleNoConfig;
  "typescript/non-nullable-type-assertion-style"?: RuleNoConfig;
  "typescript/only-throw-error"?: RuleNoConfig | [AllowWarnDeny, OnlyThrowErrorConfig];
  "typescript/parameter-properties"?: RuleNoConfig | [AllowWarnDeny, ParameterPropertiesConfig];
  "typescript/prefer-as-const"?: RuleNoConfig;
  "typescript/prefer-enum-initializers"?: RuleNoConfig;
  "typescript/prefer-find"?: RuleNoConfig;
  "typescript/prefer-for-of"?: RuleNoConfig;
  "typescript/prefer-function-type"?: RuleNoConfig;
  "typescript/prefer-includes"?: RuleNoConfig;
  "typescript/prefer-literal-enum-member"?: RuleNoConfig | [AllowWarnDeny, PreferLiteralEnumMember];
  "typescript/prefer-namespace-keyword"?: RuleNoConfig;
  "typescript/prefer-nullish-coalescing"?: RuleNoConfig | [AllowWarnDeny, PreferNullishCoalescingConfig];
  "typescript/prefer-optional-chain"?: RuleNoConfig | [AllowWarnDeny, PreferOptionalChainConfig];
  "typescript/prefer-promise-reject-errors"?: RuleNoConfig | [AllowWarnDeny, PreferPromiseRejectErrorsConfig];
  "typescript/prefer-readonly"?: RuleNoConfig | [AllowWarnDeny, PreferReadonlyConfig];
  "typescript/prefer-readonly-parameter-types"?: RuleNoConfig | [AllowWarnDeny, PreferReadonlyParameterTypesConfig];
  "typescript/prefer-reduce-type-parameter"?: RuleNoConfig;
  "typescript/prefer-regexp-exec"?: RuleNoConfig;
  "typescript/prefer-return-this-type"?: RuleNoConfig;
  "typescript/prefer-string-starts-ends-with"?: RuleNoConfig | [AllowWarnDeny, PreferStringStartsEndsWithConfig];
  "typescript/prefer-ts-expect-error"?: RuleNoConfig;
  "typescript/promise-function-async"?: RuleNoConfig | [AllowWarnDeny, PromiseFunctionAsyncConfig];
  "typescript/related-getter-setter-pairs"?: RuleNoConfig;
  "typescript/require-array-sort-compare"?: RuleNoConfig | [AllowWarnDeny, RequireArraySortCompareConfig];
  "typescript/require-await"?: RuleNoConfig;
  "typescript/restrict-plus-operands"?: RuleNoConfig | [AllowWarnDeny, RestrictPlusOperandsConfig];
  "typescript/restrict-template-expressions"?: RuleNoConfig | [AllowWarnDeny, RestrictTemplateExpressionsConfig];
  "typescript/return-await"?: RuleNoConfig | [AllowWarnDeny, ReturnAwaitOption];
  "typescript/strict-boolean-expressions"?: RuleNoConfig | [AllowWarnDeny, StrictBooleanExpressionsConfig];
  "typescript/strict-void-return"?: RuleNoConfig | [AllowWarnDeny, StrictVoidReturnConfig];
  "typescript/switch-exhaustiveness-check"?: RuleNoConfig | [AllowWarnDeny, SwitchExhaustivenessCheckConfig];
  "typescript/triple-slash-reference"?: RuleNoConfig | [AllowWarnDeny, TripleSlashReferenceConfig];
  "typescript/unbound-method"?: RuleNoConfig | [AllowWarnDeny, UnboundMethodConfig];
  "typescript/unified-signatures"?: RuleNoConfig | [AllowWarnDeny, UnifiedSignaturesOptions];
  "typescript/use-unknown-in-catch-callback-variable"?: RuleNoConfig;
  "unicode-bom"?: RuleNoConfig | [AllowWarnDeny, BomOptionType];
  "unicorn/catch-error-name"?: RuleNoConfig | [AllowWarnDeny, CatchErrorNameConfig];
  "unicorn/consistent-assert"?: RuleNoConfig;
  "unicorn/consistent-date-clone"?: RuleNoConfig;
  "unicorn/consistent-empty-array-spread"?: RuleNoConfig;
  "unicorn/consistent-existence-index-check"?: RuleNoConfig;
  "unicorn/consistent-function-scoping"?: RuleNoConfig | [AllowWarnDeny, ConsistentFunctionScoping];
  "unicorn/consistent-template-literal-escape"?: RuleNoConfig;
  "unicorn/custom-error-definition"?: RuleNoConfig;
  "unicorn/empty-brace-spaces"?: RuleNoConfig;
  "unicorn/error-message"?: RuleNoConfig;
  "unicorn/escape-case"?: RuleNoConfig;
  "unicorn/explicit-length-check"?: RuleNoConfig | [AllowWarnDeny, ExplicitLengthCheck];
  "unicorn/filename-case"?: DummyRule;
  "unicorn/import-style"?: RuleNoConfig | [AllowWarnDeny, ImportStyleConfig];
  "unicorn/max-nested-calls"?: RuleNoConfig | [AllowWarnDeny, MaxNestedCalls];
  "unicorn/new-for-builtins"?: RuleNoConfig;
  "unicorn/no-abusive-eslint-disable"?: RuleNoConfig;
  "unicorn/no-accessor-recursion"?: RuleNoConfig;
  "unicorn/no-anonymous-default-export"?: RuleNoConfig;
  "unicorn/no-array-callback-reference"?: RuleNoConfig;
  "unicorn/no-array-fill-with-reference-type"?: RuleNoConfig;
  "unicorn/no-array-for-each"?: RuleNoConfig;
  "unicorn/no-array-method-this-argument"?: RuleNoConfig;
  "unicorn/no-array-reduce"?: RuleNoConfig | [AllowWarnDeny, NoArrayReduce];
  "unicorn/no-array-reverse"?: RuleNoConfig | [AllowWarnDeny, NoArrayReverse];
  "unicorn/no-array-sort"?: RuleNoConfig | [AllowWarnDeny, NoArraySort];
  "unicorn/no-await-expression-member"?: RuleNoConfig;
  "unicorn/no-await-in-promise-methods"?: RuleNoConfig;
  "unicorn/no-console-spaces"?: RuleNoConfig;
  "unicorn/no-document-cookie"?: RuleNoConfig;
  "unicorn/no-empty-file"?: RuleNoConfig;
  "unicorn/no-hex-escape"?: RuleNoConfig;
  "unicorn/no-immediate-mutation"?: RuleNoConfig;
  "unicorn/no-instanceof-array"?: RuleNoConfig;
  "unicorn/no-instanceof-builtins"?: RuleNoConfig | [AllowWarnDeny, NoInstanceofBuiltinsConfig];
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
  "unicorn/no-null"?: RuleNoConfig | [AllowWarnDeny, NoNull];
  "unicorn/no-object-as-default-parameter"?: RuleNoConfig;
  "unicorn/no-process-exit"?: RuleNoConfig;
  "unicorn/no-single-promise-in-promise-methods"?: RuleNoConfig;
  "unicorn/no-static-only-class"?: RuleNoConfig;
  "unicorn/no-thenable"?: RuleNoConfig;
  "unicorn/no-this-assignment"?: RuleNoConfig;
  "unicorn/no-typeof-undefined"?: RuleNoConfig | [AllowWarnDeny, NoTypeofUndefined];
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
  "unicorn/no-useless-promise-resolve-reject"?: RuleNoConfig | [AllowWarnDeny, NoUselessPromiseResolveRejectOptions];
  "unicorn/no-useless-spread"?: RuleNoConfig;
  "unicorn/no-useless-switch-case"?: RuleNoConfig;
  "unicorn/no-useless-undefined"?: RuleNoConfig | [AllowWarnDeny, NoUselessUndefined];
  "unicorn/no-zero-fractions"?: RuleNoConfig;
  "unicorn/number-literal-case"?: RuleNoConfig;
  "unicorn/numeric-separators-style"?: RuleNoConfig | [AllowWarnDeny, NumericSeparatorsStyleConfig];
  "unicorn/prefer-add-event-listener"?: RuleNoConfig;
  "unicorn/prefer-array-find"?: RuleNoConfig;
  "unicorn/prefer-array-flat"?: RuleNoConfig;
  "unicorn/prefer-array-flat-map"?: RuleNoConfig;
  "unicorn/prefer-array-index-of"?: RuleNoConfig;
  "unicorn/prefer-array-some"?: RuleNoConfig;
  "unicorn/prefer-at"?: RuleNoConfig | [AllowWarnDeny, PreferAtConfig];
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
  "unicorn/prefer-export-from"?: RuleNoConfig | [AllowWarnDeny, PreferExportFrom];
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
  "unicorn/prefer-number-coercion"?: RuleNoConfig;
  "unicorn/prefer-number-properties"?: RuleNoConfig | [AllowWarnDeny, PreferNumberPropertiesConfig];
  "unicorn/prefer-object-from-entries"?: RuleNoConfig | [AllowWarnDeny, PreferObjectFromEntriesConfig];
  "unicorn/prefer-optional-catch-binding"?: RuleNoConfig;
  "unicorn/prefer-prototype-methods"?: RuleNoConfig;
  "unicorn/prefer-query-selector"?: RuleNoConfig;
  "unicorn/prefer-reflect-apply"?: RuleNoConfig;
  "unicorn/prefer-regexp-test"?: RuleNoConfig;
  "unicorn/prefer-response-static-json"?: RuleNoConfig;
  "unicorn/prefer-set-has"?: RuleNoConfig;
  "unicorn/prefer-set-size"?: RuleNoConfig;
  "unicorn/prefer-single-call"?: RuleNoConfig | [AllowWarnDeny, PreferSingleCallConfig];
  "unicorn/prefer-spread"?: RuleNoConfig;
  "unicorn/prefer-string-raw"?: RuleNoConfig;
  "unicorn/prefer-string-replace-all"?: RuleNoConfig;
  "unicorn/prefer-string-slice"?: RuleNoConfig;
  "unicorn/prefer-string-starts-ends-with"?: RuleNoConfig;
  "unicorn/prefer-string-trim-start-end"?: RuleNoConfig;
  "unicorn/prefer-structured-clone"?: RuleNoConfig | [AllowWarnDeny, PreferStructuredCloneConfig];
  "unicorn/prefer-ternary"?: RuleNoConfig | [AllowWarnDeny, PreferTernaryOption];
  "unicorn/prefer-top-level-await"?: RuleNoConfig;
  "unicorn/prefer-type-error"?: RuleNoConfig;
  "unicorn/relative-url-style"?: RuleNoConfig | [AllowWarnDeny, RelativeUrlStyleConfig];
  "unicorn/require-array-join-separator"?: RuleNoConfig;
  "unicorn/require-module-attributes"?: RuleNoConfig;
  "unicorn/require-module-specifiers"?: RuleNoConfig;
  "unicorn/require-number-to-fixed-digits-argument"?: RuleNoConfig;
  "unicorn/require-post-message-target-origin"?: RuleNoConfig;
  "unicorn/switch-case-braces"?: RuleNoConfig | [AllowWarnDeny, SwitchCaseBracesConfig];
  "unicorn/switch-case-break-position"?: RuleNoConfig;
  "unicorn/text-encoding-identifier-case"?: RuleNoConfig | [AllowWarnDeny, TextEncodingIdentifierCase];
  "unicorn/throw-new-error"?: RuleNoConfig;
  "use-isnan"?: RuleNoConfig | [AllowWarnDeny, UseIsnan];
  "valid-typeof"?: RuleNoConfig | [AllowWarnDeny, ValidTypeof];
  "vars-on-top"?: RuleNoConfig;
  "vitest/consistent-each-for"?: RuleNoConfig | [AllowWarnDeny, ConsistentEachForJson];
  "vitest/consistent-test-filename"?: RuleNoConfig | [AllowWarnDeny, ConsistentTestFilenameConfig];
  "vitest/consistent-test-it"?: RuleNoConfig | [AllowWarnDeny, ConsistentTestItConfig];
  "vitest/consistent-vitest-vi"?: RuleNoConfig | [AllowWarnDeny, ConsistentVitestConfig];
  "vitest/expect-expect"?: RuleNoConfig | [AllowWarnDeny, ExpectExpectConfig];
  "vitest/hoisted-apis-on-top"?: RuleNoConfig;
  "vitest/max-expects"?: RuleNoConfig | [AllowWarnDeny, MaxExpectsConfig];
  "vitest/max-nested-describe"?: RuleNoConfig | [AllowWarnDeny, MaxNestedDescribeConfig];
  "vitest/no-alias-methods"?: RuleNoConfig;
  "vitest/no-commented-out-tests"?: RuleNoConfig;
  "vitest/no-conditional-expect"?: RuleNoConfig;
  "vitest/no-conditional-in-test"?: RuleNoConfig;
  "vitest/no-conditional-tests"?: RuleNoConfig;
  "vitest/no-disabled-tests"?: RuleNoConfig;
  "vitest/no-duplicate-hooks"?: RuleNoConfig;
  "vitest/no-focused-tests"?: RuleNoConfig;
  "vitest/no-hooks"?: RuleNoConfig | [AllowWarnDeny, NoHooksConfig];
  "vitest/no-identical-title"?: RuleNoConfig;
  "vitest/no-import-node-test"?: RuleNoConfig;
  "vitest/no-importing-vitest-globals"?: RuleNoConfig;
  "vitest/no-interpolation-in-snapshots"?: RuleNoConfig;
  "vitest/no-large-snapshots"?: RuleNoConfig | [AllowWarnDeny, NoLargeSnapshotsConfig];
  "vitest/no-mocks-import"?: RuleNoConfig;
  "vitest/no-restricted-matchers"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedMatchersConfig];
  "vitest/no-restricted-vi-methods"?: RuleNoConfig | [AllowWarnDeny, NoRestrictedTestMethodsConfig];
  "vitest/no-standalone-expect"?: RuleNoConfig | [AllowWarnDeny, NoStandaloneExpectConfig];
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
  "vitest/prefer-expect-assertions"?: RuleNoConfig | [AllowWarnDeny, PreferExpectAssertionsConfig];
  "vitest/prefer-expect-resolves"?: RuleNoConfig;
  "vitest/prefer-expect-type-of"?: RuleNoConfig;
  "vitest/prefer-hooks-in-order"?: RuleNoConfig;
  "vitest/prefer-hooks-on-top"?: RuleNoConfig;
  "vitest/prefer-import-in-mock"?: RuleNoConfig | [AllowWarnDeny, PreferImportInMockConfig];
  "vitest/prefer-importing-vitest-globals"?: RuleNoConfig;
  "vitest/prefer-lowercase-title"?: DummyRule;
  "vitest/prefer-mock-promise-shorthand"?: RuleNoConfig;
  "vitest/prefer-mock-return-shorthand"?: RuleNoConfig;
  "vitest/prefer-snapshot-hint"?: RuleNoConfig | [AllowWarnDeny, SnapshotHintMode];
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
  "vitest/require-hook"?: RuleNoConfig | [AllowWarnDeny, RequireHookConfig];
  "vitest/require-local-test-context-for-concurrent-snapshots"?: RuleNoConfig;
  "vitest/require-mock-type-parameters"?: RuleNoConfig | [AllowWarnDeny, RequireMockTypeParametersConfig];
  "vitest/require-test-timeout"?: RuleNoConfig;
  "vitest/require-to-throw-message"?: RuleNoConfig;
  "vitest/require-top-level-describe"?: RuleNoConfig | [AllowWarnDeny, RequireTopLevelDescribeConfig];
  "vitest/valid-describe-callback"?: RuleNoConfig;
  "vitest/valid-expect"?: RuleNoConfig | [AllowWarnDeny, ValidExpectConfig];
  "vitest/valid-expect-in-promise"?: RuleNoConfig;
  "vitest/valid-title"?: DummyRule;
  "vitest/warn-todo"?: RuleNoConfig;
  "vue/component-definition-name-casing"?: RuleNoConfig | [AllowWarnDeny, CaseType];
  "vue/define-emits-declaration"?: RuleNoConfig | [AllowWarnDeny, DeclarationStyle];
  "vue/define-props-declaration"?: RuleNoConfig | [AllowWarnDeny, DeclarationStyle2];
  "vue/define-props-destructuring"?: RuleNoConfig | [AllowWarnDeny, DefinePropsDestructuring];
  "vue/max-props"?: RuleNoConfig | [AllowWarnDeny, MaxProps];
  "vue/next-tick-style"?: RuleNoConfig | [AllowWarnDeny, NextTickOption];
  "vue/no-arrow-functions-in-watch"?: RuleNoConfig;
  "vue/no-async-in-computed-properties"?: RuleNoConfig | [AllowWarnDeny, NoAsyncInComputedPropertiesConfig];
  "vue/no-computed-properties-in-data"?: RuleNoConfig;
  "vue/no-deprecated-data-object-declaration"?: RuleNoConfig;
  "vue/no-deprecated-delete-set"?: RuleNoConfig;
  "vue/no-deprecated-destroyed-lifecycle"?: RuleNoConfig;
  "vue/no-deprecated-events-api"?: RuleNoConfig;
  "vue/no-deprecated-model-definition"?: RuleNoConfig | [AllowWarnDeny, NoDeprecatedModelDefinitionConfig];
  "vue/no-deprecated-props-default-this"?: RuleNoConfig;
  "vue/no-deprecated-vue-config-keycodes"?: RuleNoConfig;
  "vue/no-dupe-keys"?: RuleNoConfig | [AllowWarnDeny, NoDupeKeysConfig];
  "vue/no-export-in-script-setup"?: RuleNoConfig;
  "vue/no-expose-after-await"?: RuleNoConfig;
  "vue/no-import-compiler-macros"?: RuleNoConfig;
  "vue/no-lifecycle-after-await"?: RuleNoConfig;
  "vue/no-multiple-slot-args"?: RuleNoConfig;
  "vue/no-required-prop-with-default"?: RuleNoConfig;
  "vue/no-reserved-component-names"?: RuleNoConfig | [AllowWarnDeny, NoReservedComponentNames];
  "vue/no-reserved-keys"?: RuleNoConfig | [AllowWarnDeny, NoReservedKeysConfig];
  "vue/no-reserved-props"?: RuleNoConfig | [AllowWarnDeny, NoReservedPropsConfig];
  "vue/no-shared-component-data"?: RuleNoConfig;
  "vue/no-side-effects-in-computed-properties"?: RuleNoConfig;
  "vue/no-this-in-before-route-enter"?: RuleNoConfig;
  "vue/no-watch-after-await"?: RuleNoConfig;
  "vue/prefer-import-from-vue"?: RuleNoConfig;
  "vue/prop-name-casing"?: RuleNoConfig | [AllowWarnDeny, CaseType2] | [AllowWarnDeny, CaseType2, Options];
  "vue/require-default-export"?: RuleNoConfig;
  "vue/require-default-prop"?: RuleNoConfig;
  "vue/require-direct-export"?: RuleNoConfig | [AllowWarnDeny, RequireDirectExport];
  "vue/require-prop-type-constructor"?: RuleNoConfig;
  "vue/require-prop-types"?: RuleNoConfig;
  "vue/require-render-return"?: RuleNoConfig;
  "vue/require-slots-as-functions"?: RuleNoConfig;
  "vue/require-typed-ref"?: RuleNoConfig;
  "vue/return-in-computed-property"?: RuleNoConfig | [AllowWarnDeny, ReturnInComputedPropertyConfig];
  "vue/return-in-emits-validator"?: RuleNoConfig;
  "vue/valid-define-emits"?: RuleNoConfig;
  "vue/valid-define-options"?: RuleNoConfig;
  "vue/valid-define-props"?: RuleNoConfig;
  "vue/valid-next-tick"?: RuleNoConfig;
  yoda?: RuleNoConfig | [AllowWarnDeny, AllowYoda] | [AllowWarnDeny, AllowYoda, YodaOptions];
  [k: string]: DummyRule | undefined;
}
export interface AccessorPairsConfig {
  /**
   * Enforce the rule for class members.
   */
  enforceForClassMembers?: boolean;
  /**
   * Enforce the rule for TypeScript interfaces and types.
   */
  enforceForTSTypes?: boolean;
  /**
   * Report a getter without a setter.
   */
  getWithoutSet?: boolean;
  /**
   * Report a setter without a getter.
   */
  setWithoutGet?: boolean;
}
export interface ArrayCallbackReturn {
  /**
   * When set to true, allows callbacks of methods that require a return value to
   * implicitly return undefined with a return statement containing no expression.
   */
  allowImplicit?: boolean;
  /**
   * When set to true, rule will not report the return value with a void operator.
   * Works only if `checkForEach` option is set to true.
   */
  allowVoid?: boolean;
  /**
   * When set to true, rule will also report forEach callbacks that return a value.
   */
  checkForEach?: boolean;
}
export interface ArrowBodyStyleConfig {
  /**
   * Requires braces and an explicit return for object literals. This option only applies when
   * the first option is `"as-needed"`.
   */
  requireReturnForObjectLiteral?: boolean;
}
export interface CommentConfigJson {
  /**
   * If true, consecutive comments will be ignored after the first comment.
   */
  ignoreConsecutiveComments?: boolean;
  /**
   * If true, inline comments (comments in the middle of code) will be ignored.
   */
  ignoreInlineComments?: boolean;
  /**
   * A regex pattern. Comments that match the pattern will not cause violations.
   */
  ignorePattern?: string;
}
export interface ClassMethodsUseThisConfig {
  /**
   * Enforce this rule for class fields that are functions.
   */
  enforceForClassFields?: boolean;
  /**
   * List of method names to exempt from this rule. Names can include the hash for private methods.
   * Example: `save`, `#rerender`
   */
  exceptMethods?: string[];
  /**
   * Whether to ignore classes that implement interfaces.
   */
  ignoreClassesWithImplements?: IgnoreClassWithImplements;
  /**
   * Whether to ignore methods that are overridden.
   */
  ignoreOverrideMethods?: boolean;
}
export interface ComplexityConfig {
  /**
   * Maximum amount of cyclomatic complexity
   */
  max?: number;
  /**
   * The cyclomatic complexity variant to use
   */
  variant?: Variant;
}
export interface DefaultCaseConfig {
  /**
   * A regex pattern used to detect comments that mark the absence
   * of a `default` case as intentional.
   *
   * Default value: `no default`.
   *
   * Examples of **incorrect** code for this rule with the `{ "commentPattern": "^skip\\sdefault" }` option:
   * ```js
   * switch (a) {
   * case 1:
   * break;
   * // no default
   * }
   * ```
   *
   * Examples of **correct** code for this rule with the `{ "commentPattern": "^skip\\sdefault" }` option:
   * ```js
   * switch (a) {
   * case 1:
   * break;
   * // skip default
   * }
   * ```
   */
  commentPattern?: string;
}
export interface EqeqeqOptions {
  /**
   * Configuration for whether to allow/disallow comparisons against `null`,
   * e.g. `foo == null` or `foo != null`
   */
  null?: NullType;
}
export interface FuncNameMatchingConfig {
  /**
   * If `considerPropertyDescriptor` is set to `true`, the check will take into account the use of `Object.create`, `Object.defineProperty`, `Object.defineProperties`, and `Reflect.defineProperty`.
   */
  considerPropertyDescriptor?: boolean;
  /**
   * If `includeCommonJSModuleExports` is set to `true`, `module.exports` and `module["exports"]` will be checked by this rule.
   */
  includeCommonJSModuleExports?: boolean;
}
export interface FuncNamesGeneratorsConfig {
  /**
   * Configuration for generator function expressions. If not specified, uses the
   * primary configuration.
   *
   * Accepts `always`, `as-needed`, or `never`.
   *
   * Generator functions are those defined using the `function*` syntax.
   * ```js
   * function* foobar(i) {
   * yield i;
   * yield i + 10;
   * }
   * ```
   */
  generators?: FuncNamesConfigType;
}
export interface FuncStyleConfig {
  /**
   * When true, arrow functions are allowed regardless of the style setting.
   */
  allowArrowFunctions?: boolean;
  /**
   * When true, functions with type annotations are allowed regardless of the style setting.
   */
  allowTypeAnnotation?: boolean;
  /**
   * Override the style specifically for named exports. Can be "expression", "declaration", or "ignore" (default).
   */
  overrides?: Override;
}
export interface Override {
  namedExports?: NamedExports;
}
export interface GetterReturn {
  /**
   * When set to `true`, allows getters to implicitly return `undefined` with a `return` statement containing no expression.
   */
  allowImplicit?: boolean;
}
export interface GroupedAccessorPairsConfig {
  /**
   * When `enforceForTSTypes` is enabled, this rule also applies to TypeScript interfaces
   * and type aliases.
   *
   * Examples of **incorrect** TypeScript code:
   * ```ts
   * interface Foo {
   * get a(): string;
   * someProperty: string;
   * set a(value: string);
   * }
   *
   * type Bar = {
   * get b(): string;
   * someProperty: string;
   * set b(value: string);
   * };
   * ```
   *
   * Examples of **correct** TypeScript code:
   * ```ts
   * interface Foo {
   * get a(): string;
   * set a(value: string);
   * someProperty: string;
   * }
   *
   * type Bar = {
   * get b(): string;
   * set b(value: string);
   * someProperty: string;
   * };
   * ```
   */
  enforceForTSTypes?: boolean;
}
export interface IdLengthConfig {
  /**
   * Whether to check TypeScript generic type parameter names.
   * Defaults to `true`.
   */
  checkGeneric?: boolean;
  /**
   * An array of regex patterns for identifiers to exclude from the rule.
   * For example, `["^x.*"]` would exclude all identifiers starting with "x".
   */
  exceptionPatterns?: string[];
  /**
   * An array of identifier names that are excluded from the rule.
   * For example, `["x", "y", "z"]` would allow single-letter identifiers "x", "y", and "z".
   */
  exceptions?: string[];
  /**
   * The maximum number of graphemes allowed in an identifier.
   * Defaults to no maximum (effectively unlimited).
   */
  max?: number;
  /**
   * The minimum number of graphemes required in an identifier.
   */
  min?: number;
  /**
   * Whether to check property names for length.
   */
  properties?: AlwaysNever;
}
export interface IdMatchOptions {
  /**
   * Whether class field names are checked, including public fields,
   * accessor properties, and private field names.
   */
  classFields?: boolean;
  /**
   * Whether to ignore shorthand and aliased bindings introduced by object
   * destructuring, such as `foo` in `const { foo } = obj` and `alias` in
   * `const { foo: alias } = obj`. This does not suppress computed key
   * references such as `const { [key]: value } = obj`.
   */
  ignoreDestructuring?: boolean;
  /**
   * Whether to check only variable and function declaration names.
   * References, member names, labels, class names, TypeScript declarations,
   * and function or arrow parameters are skipped.
   */
  onlyDeclarations?: boolean;
  /**
   * Whether object literal property names, class method names, and assigned
   * member names such as `obj.prop = value` are checked.
   */
  properties?: boolean;
}
export interface ImportExtensionsConfig {
  /**
   * Whether to check type imports when enforcing extension rules.
   */
  checkTypeImports?: boolean;
  /**
   * Whether to ignore package imports when enforcing extension rules.
   */
  ignorePackages?: boolean;
  /**
   * Path group overrides for bespoke import specifiers.
   */
  pathGroupOverrides?: PathGroupOverrideConfig[];
  /**
   * Per-extension rules.
   */
  pattern?: {
    [k: string]: ExtensionRule;
  };
}
export interface PathGroupOverrideConfig {
  /**
   * Action to take when pattern matches.
   */
  action: PathGroupAction;
  /**
   * Glob pattern to match import specifiers.
   */
  pattern: string;
}
export interface MaxDependenciesConfig {
  /**
   * Whether to ignore type imports when counting dependencies.
   *
   * ```ts
   * // Neither of these count as dependencies if `ignoreTypeImports` is true:
   * import type { Foo } from './foo';
   * import { type Foo } from './foo';
   * ```
   */
  ignoreTypeImports?: boolean;
  /**
   * Maximum number of dependencies allowed in a file.
   */
  max?: number;
}
export interface Namespace {
  /**
   * Whether to allow computed references to an imported namespace.
   */
  allowComputed?: boolean;
}
export interface NewlineAfterImport {
  considerComments?: boolean;
  count?: number;
  exactCount?: boolean;
}
export interface NoAbsolutePath {
  /**
   * If set to `true`, dependency paths for AMD-style define and require calls will be resolved:
   *
   * ```js
   * /* import/no-absolute-path: ["error", { "commonjs": false, "amd": true }] * /
   * define(['/foo'], function (foo) { /*...* / }) // reported
   * require(['/foo'], function (foo) { /*...* / }) // reported
   *
   * const foo = require('/foo') // ignored because of explicit `commonjs: false`
   * ```
   */
  amd?: boolean;
  /**
   * If set to `true`, dependency paths for CommonJS-style require calls will be resolved:
   *
   * ```js
   * var foo = require('/foo'); // reported
   * ```
   */
  commonjs?: boolean;
  /**
   * If set to `true`, dependency paths for ES module import statements will be resolved:
   *
   * ```js
   * import foo from '/foo'; // reported
   * ```
   */
  esmodule?: boolean;
}
export interface NoAnonymousDefaultExport {
  /**
   * Allow anonymous class as default export.
   */
  allowAnonymousClass?: boolean;
  /**
   * Allow anonymous function as default export.
   */
  allowAnonymousFunction?: boolean;
  /**
   * Allow anonymous array as default export.
   */
  allowArray?: boolean;
  /**
   * Allow anonymous arrow function as default export.
   */
  allowArrowFunction?: boolean;
  /**
   * Allow anonymous call expression as default export.
   */
  allowCallExpression?: boolean;
  /**
   * Allow anonymous literal as default export.
   */
  allowLiteral?: boolean;
  /**
   * Allow anonymous new expression as default export.
   */
  allowNew?: boolean;
  /**
   * Allow anonymous object as default export.
   */
  allowObject?: boolean;
}
export interface NoCommonjs {
  /**
   * When set to `true`, allows conditional `require()` calls (e.g., inside `if` statements or try-catch blocks).
   * This is useful for places where you need to conditionally load via commonjs requires if ESM imports are not supported.
   */
  allowConditionalRequire?: boolean;
  /**
   * If `allowPrimitiveModules` option is set to true, the following is valid:
   *
   * ```js
   * module.exports = "foo";
   * module.exports = function rule(context) {
   * return { /* ... * / };
   * };
   * ```
   *
   * but this is still reported:
   *
   * ```js
   * module.exports = { x: "y" };
   * exports.z = function bark() { /* ... * / };
   * ```
   */
  allowPrimitiveModules?: boolean;
  /**
   * If set to `true`, `require` calls are valid:
   *
   * ```js
   * var mod = require("./mod");
   * ```
   *
   * but `module.exports` is reported as usual.
   */
  allowRequire?: boolean;
}
export interface NoCycle {
  /**
   * Allow cyclic dependency if there is at least one dynamic import in the chain
   */
  allowUnsafeDynamicCyclicDependency?: boolean;
  /**
   * Ignore external modules
   */
  ignoreExternal?: boolean;
  /**
   * Ignore type-only imports
   */
  ignoreTypes?: boolean;
  /**
   * Maximum dependency depth to traverse
   */
  maxDepth?: number;
}
export interface NoDuplicates {
  /**
   * When set to `true`, the rule will consider the query string part of the import path
   * when determining if imports are duplicates. This is useful when using loaders like
   * webpack that use query strings to configure how a module should be loaded.
   *
   * Examples of **correct** code with this option set to `true`:
   * ```javascript
   * import x from './bar?optionX';
   * import y from './bar?optionY';
   * ```
   */
  considerQueryString?: boolean;
  /**
   * When set to `true`, prefer inline type imports instead of separate type import
   * statements for TypeScript code.
   *
   * Examples of **correct** code with this option set to `true`:
   * ```typescript
   * import { Foo, type Bar } from './module';
   * ```
   */
  preferInline?: boolean;
}
export interface NoDynamicRequire {
  /**
   * When `true`, also check `import()` expressions for dynamic module specifiers.
   */
  esmodule?: boolean;
}
export interface NoNamespaceConfig {
  /**
   * An array of glob strings for modules that should be ignored by the rule.
   * For example, `["*.json"]` will ignore all JSON imports.
   */
  ignore?: string[];
}
export interface NoNodejsModulesConfig {
  /**
   * Array of names of allowed modules. Defaults to an empty array.
   */
  allow: string[];
}
export interface NoUnassignedImportConfig {
  /**
   * A list of glob patterns to allow unassigned imports for specific modules.
   * For example:
   * `{ "allow": ["** /*.css"] }` will allow unassigned imports for any module ending with `.css`.
   */
  allow?: string[];
}
export interface PreferDefaultExport {
  /**
   * Configuration option to specify the target type for preferring default exports.
   */
  target?: Target;
}
export interface InitDeclarationsConfig {
  /**
   * When set to `true`, allows uninitialized variables in the init expression of `for`, `for-in`, and `for-of` loops.
   * Only applies when mode is set to `"never"`.
   */
  ignoreForLoopInit?: boolean;
}
export interface ConsistentTestItConfig {
  /**
   * Decides whether to use `test` or `it`.
   *
   * Examples of **incorrect** code for `{ "fn": "test" }`:
   * ```javascript
   * it('foo');
   * it.only('foo');
   * ```
   *
   * Examples of **correct** code for `{ "fn": "test" }`:
   * ```javascript
   * test('foo');
   * test.only('foo');
   * ```
   *
   * Examples of **incorrect** code for `{ "fn": "it" }`:
   * ```javascript
   * test('foo');
   * test.only('foo');
   * ```
   *
   * Examples of **correct** code for `{ "fn": "it" }`:
   * ```javascript
   * it('foo');
   * it.only('foo');
   * ```
   */
  fn?: TestCaseName;
  /**
   * Decides whether to use `test` or `it` within a `describe` scope.
   * If only `fn` is provided, this will default to the value of `fn`.
   *
   * Examples of **incorrect** code for `{ "withinDescribe": "test" }`:
   * ```javascript
   * describe('foo', function () {
   * it('bar');
   * });
   * ```
   *
   * Examples of **correct** code for `{ "withinDescribe": "test" }`:
   * ```javascript
   * describe('foo', function () {
   * test('bar');
   * });
   * ```
   */
  withinDescribe?: TestCaseName;
}
export interface ExpectExpectConfig {
  /**
   * An array of function names that should also be treated as test blocks.
   */
  additionalTestBlockFunctions?: string[];
  /**
   * A list of function names that should be treated as assertion functions.
   *
   * NOTE: The default value is `["expect"]` for Jest and
   * `["expect", "expectTypeOf", "assert", "assertType"]` for Vitest.
   */
  assertFunctionNames?: string[];
}
export interface MaxExpectsConfig {
  /**
   * Maximum number of `expect()` assertion calls allowed within a single test.
   */
  max?: number;
}
export interface MaxNestedDescribeConfig {
  /**
   * Maximum allowed depth of nested describe calls.
   */
  max?: number;
}
export interface NoDeprecatedFunctionsConfig {
  /**
   * Jest configuration options.
   * Deprecated config, it will be removed in future versions.
   * Use please instead { "settings": { "jest": {"version": 29 } } } in `Oxlint config file`.
   * Beware the value from the config have higher priority than the rule config.
   */
  jest?: JestConfigJson;
}
export interface JestConfigJson {
  /**
   * The version of Jest being used.
   */
  version?: string;
}
export interface NoHooksConfig {
  /**
   * An array of hook function names that are permitted for use.
   */
  allow?: string[];
}
export interface NoLargeSnapshotsConfig {
  /**
   * A map of snapshot file paths to arrays of snapshot names that are allowed to exceed the size limit.
   * Snapshot names can be specified as regular expressions.
   */
  allowedSnapshots?: {
    [k: string]: string[] | undefined;
  };
  /**
   * Maximum number of lines allowed for inline snapshots.
   */
  inlineMaxSize?: number;
  /**
   * Maximum number of lines allowed for external snapshot files.
   */
  maxSize?: number;
}
export interface NoRestrictedTestMethodsConfig {
  [k: string]: (string | null) | undefined;
}
export interface NoRestrictedMatchersConfig {
  [k: string]: (string | null) | undefined;
}
export interface NoStandaloneExpectConfig {
  /**
   * An array of function names that should also be treated as test blocks.
   */
  additionalTestBlockFunctions?: string[];
}
export interface PreferEndingWithAnExpectConfig {
  /**
   * An array of function names that should also be treated as test blocks.
   */
  additionalTestBlockFunctions?: string[];
  /**
   * A list of function names that should be treated as assertion functions.
   */
  assertFunctionNames?: string[];
}
export interface PreferExpectAssertionsConfig {
  onlyFunctionsWithAsyncKeyword?: boolean;
  onlyFunctionsWithExpectInCallback?: boolean;
  onlyFunctionsWithExpectInLoop?: boolean;
}
export interface PreferImportingJestGlobalsConfig {
  /**
   * Jest function types to enforce importing for.
   */
  types?: JestFnType[];
}
export interface RequireHookConfig {
  /**
   * An array of function names that are allowed to be called outside of hooks.
   */
  allowedFunctionCalls?: string[];
}
export interface RequireTopLevelDescribeConfig {
  /**
   * The maximum number of top-level `describe` blocks allowed in a test file.
   */
  maxNumberOfTopLevelDescribes?: number;
}
export interface ValidExpectConfig {
  /**
   * When `true`, async assertions must be awaited in all contexts (not just return statements).
   */
  alwaysAwait?: boolean;
  /**
   * List of matchers that are considered async and therefore require awaiting (e.g. `toResolve`, `toReject`).
   */
  asyncMatchers?: string[];
  /**
   * Maximum number of arguments `expect` should be called with.
   */
  maxArgs?: number;
  /**
   * Minimum number of arguments `expect` should be called with.
   */
  minArgs?: number;
}
export interface CheckTagNamesConfig {
  /**
   * Additional tag names to allow.
   */
  definedTags?: string[];
  /**
   * Whether to allow JSX-related tags:
   * - `jsx`
   * - `jsxFrag`
   * - `jsxImportSource`
   * - `jsxRuntime`
   */
  jsxTags?: boolean;
  /**
   * If typed is `true`, disallow tags that are unnecessary/duplicative of TypeScript functionality.
   */
  typed?: boolean;
}
export interface EmptyTagsConfig {
  /**
   * Additional tags to check for their descriptions.
   */
  tags?: string[];
}
export interface NoDefaultsConfig {
  /**
   * If true, report the presence of optional param names (square brackets) on `@param` tags.
   */
  noOptionalParamNames?: boolean;
}
export interface RequireParamDescriptionConfig {
  /**
   * The description string to set by default for destructured roots. Defaults to "The root object".
   */
  defaultDestructuredRootDescription?: string;
  /**
   * Whether to set a default destructured root description.
   * For example, you may wish to avoid manually having to set the description for a @param corresponding to a destructured root object as it should always be the same type of object.
   * Uses `defaultDestructuredRootDescription` for the description string. Defaults to `false`.
   */
  setDefaultDestructuredRootDescription?: boolean;
}
export interface RequireParamTypeConfig {
  /**
   * The type string to set by default for destructured roots. Defaults to "object".
   */
  defaultDestructuredRootType?: string;
  /**
   * Whether to set a default destructured root type. For example, you may wish to avoid manually having to set the type for a `@param` corresponding to a destructured root object as it is always going to be an object. Uses `defaultDestructuredRootType` for the type string. Defaults to `false`.
   */
  setDefaultDestructuredRootType?: boolean;
}
export interface RequireReturnsConfig {
  /**
   * Whether to check constructor methods.
   */
  checkConstructors?: boolean;
  /**
   * Whether to check getter methods.
   */
  checkGetters?: boolean;
  /**
   * Tags that exempt functions from requiring `@returns`.
   */
  exemptedBy?: string[];
  /**
   * Whether to require a `@returns` tag even if the function doesn't return a value.
   */
  forceRequireReturn?: boolean;
  /**
   * Whether to require a `@returns` tag for async functions.
   */
  forceReturnsWithAsync?: boolean;
}
export interface RequireYieldsConfig {
  /**
   * Functions with these tags will be exempted from the lint rule.
   */
  exemptedBy?: string[];
  /**
   * When `true`, all generator functions must have a `@yields` tag, even if they don't yield a value or have an empty body.
   */
  forceRequireYields?: boolean;
  /**
   * When `true`, require `@yields` when a `@generator` tag is present.
   */
  withGeneratorTag?: boolean;
}
export interface AltTextConfigSchema {
  /**
   * Custom components to check for alt text on `area` elements.
   */
  area?: string[];
  /**
   * Custom components to check for alt text on any of the supported elements.
   */
  elements?: AltTextElements[];
  /**
   * Custom components to check for alt text on `img` elements.
   */
  img?: string[];
  /**
   * Custom components to check for alt text on `input[type="image"]` elements.
   */
  'input[type="image"]'?: string[];
  /**
   * Custom components to check for alt text on `object` elements.
   */
  object?: string[];
}
export interface AnchorAmbiguousTextConfig {
  /**
   * List of ambiguous words or phrases that should be flagged in anchor text.
   */
  words?: string[];
}
export interface AnchorIsValidConfig {
  /**
   * Sub-rule aspects to run.
   */
  aspects?: AnchorIsValidAspect[];
  /**
   * Custom components to treat as anchor elements.
   */
  components?: string[];
  /**
   * Custom prop names to treat as link destinations.
   */
  specialLink?: string[];
}
export interface AriaRoleConfig {
  /**
   * Custom roles that should be allowed in addition to the ARIA spec.
   */
  allowedInvalidRoles?: string[];
  /**
   * Determines if developer-created components are checked.
   */
  ignoreNonDOM?: boolean;
}
export interface AutocompleteValidConfig {
  /**
   * List of custom component names that should be treated as input elements.
   */
  inputComponents?: string[];
}
export interface ControlHasAssociatedLabelConfig {
  /**
   * Custom JSX components to be treated as interactive controls.
   */
  controlComponents?: string[];
  /**
   * Maximum depth to search for an accessible label within the element.
   * Defaults to `2`.
   */
  depth?: number;
  /**
   * Elements to ignore.
   * Defaults to `["audio", "canvas", "embed", "input", "textarea", "tr", "video"]`.
   */
  ignoreElements?: string[];
  /**
   * Interactive roles to ignore.
   * Defaults to `["grid", "listbox", "menu", "menubar", "radiogroup", "row", "tablist", "toolbar", "tree", "treegrid"]`.
   */
  ignoreRoles?: string[];
  /**
   * Additional attributes to check for accessible label text.
   */
  labelAttributes?: string[];
}
export interface HeadingHasContentConfig {
  /**
   * Additional custom component names to treat as heading elements.
   * These will be validated in addition to the standard h1-h6 elements.
   */
  components?: string[];
}
export interface ImgRedundantAltConfig {
  /**
   * JSX element types to validate (component names) where the rule applies.
   * For example, `["img", "Image"]`.
   */
  components?: string[];
  /**
   * Words considered redundant in alt text that should trigger a warning.
   */
  words?: string[];
}
export interface InteractiveSupportsFocusConfig {
  /**
   * An array of interactive ARIA roles that should be considered tabbable (require `tabIndex={0}`).
   * Interactive roles not in this list are only required to be focusable (`tabIndex={-1}` is sufficient).
   * Defaults to `["button", "checkbox", "link", "searchbox", "spinbutton", "switch", "textbox"]`.
   */
  tabbable?: string[];
}
export interface LabelHasAssociatedControlConfig {
  /**
   * The type of association required between the label and the control.
   */
  assert?: Assert;
  /**
   * Custom JSX components to be treated as form controls.
   */
  controlComponents?: string[];
  /**
   * Maximum depth to search for a nested control.
   */
  depth?: number;
  /**
   * Attributes to check for accessible label text.
   */
  labelAttributes?: string[];
  /**
   * Custom JSX components to be treated as labels.
   */
  labelComponents?: string[];
}
export interface MediaHasCaptionConfig {
  /**
   * Element names to treat as `<audio>` elements
   */
  audio?: string[];
  /**
   * Element names to treat as `<track>` elements
   */
  track?: string[];
  /**
   * Element names to treat as `<video>` elements
   */
  video?: string[];
}
export interface MouseEventsHaveKeyEventsConfig {
  /**
   * List of hover-in mouse event handlers that require corresponding keyboard event handlers.
   */
  hoverInHandlers?: string[];
  /**
   * List of hover-out mouse event handlers that require corresponding keyboard event handlers.
   */
  hoverOutHandlers?: string[];
}
export interface NoAutofocus {
  /**
   * Determines if developer-created components are checked.
   */
  ignoreNonDOM?: boolean;
}
export interface NoDistractingElementsConfig {
  /**
   * List of distracting elements to check for.
   */
  elements?: DistractingElement[];
}
export interface NoInteractiveElementToNoninteractiveRoleConfig {
  [k: string]: string[] | undefined;
}
export interface NoNoninteractiveElementInteractionsConfig {
  /**
   * An array of event handler names that should trigger this rule.
   */
  handlers?: string[];
  [k: string]: string[] | undefined;
}
export interface NoNoninteractiveElementToInteractiveRoleConfig {
  [k: string]: string[] | undefined;
}
export interface NoNoninteractiveTabindexConfig {
  /**
   * If `true`, allows tabIndex values to be expression values (e.g., variables, ternaries). If `false`, only string literal values are allowed.
   */
  allowExpressionValues?: boolean;
  /**
   * An array of ARIA roles that should be considered interactive.
   */
  roles?: string[];
  /**
   * An array of custom HTML elements that should be considered interactive.
   */
  tags?: string[];
}
export interface NoRedundantRolesConfig {
  [k: string]: string[] | undefined;
}
export interface NoStaticElementInteractionsConfig {
  /**
   * If `true`, role attribute values that are JSX expressions (e.g., `role={ROLE}`) are allowed.
   * If `false`, only string literal role values are permitted.
   */
  allowExpressionValues?: boolean;
  /**
   * An array of event handler names that should trigger this rule (e.g., `onClick`, `onKeyDown`).
   */
  handlers?: string[];
}
export interface LogicalAssignmentOperatorsConfig {
  /**
   * This option checks for additional patterns with if statements which could be expressed with the logical assignment operator.
   * Only available if string option is set to `always`.
   *
   * Examples of **incorrect** code for this rule with the `["always", { enforceForIfStatements: true }]` option:
   * ```js
   * if (a) a = b // <=> a &&= b
   * if (!a) a = b // <=> a ||= b
   *
   * if (a == null) a = b // <=> a ??= b
   * if (a === null || a === undefined) a = b // <=> a ??= b
   * ```
   *
   * Examples of **correct** code for this rule with the `["always", { enforceForIfStatements: true }]` option:
   * ```js
   * if (a) b = c
   * if (a === 0) a = b
   * ```
   */
  enforceForIfStatements?: boolean;
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
export interface NewCapConfig {
  /**
   * `true` to require that all functions with names starting with an uppercase letter to be called with `new`.
   */
  capIsNew?: boolean;
  /**
   * A regex pattern to match exceptions for functions with names starting with an uppercase letter.
   */
  capIsNewExceptionPattern?: string;
  /**
   * Exceptions to ignore for functions with names starting with an uppercase letter.
   */
  capIsNewExceptions?: string[];
  /**
   * `true` to require that all constructor names start with an uppercase letter, e.g. `new Person()`.
   */
  newIsCap?: boolean;
  /**
   * A regex pattern to match exceptions for constructor names starting with an uppercase letter.
   */
  newIsCapExceptionPattern?: string;
  /**
   * Exceptions to ignore for constructor names starting with an uppercase letter.
   */
  newIsCapExceptions?: string[];
  /**
   * `true` to require capitalization for object properties (e.g., `new obj.Method()`).
   */
  properties?: boolean;
}
export interface NoBitwiseConfig {
  /**
   * The `allow` option permits the given list of bitwise operators to be used
   * as exceptions to this rule.
   *
   * For example `{ "allow": ["~"] }` would allow the use of the bitwise operator
   * `~` without restriction. Such as in the following:
   *
   * ```javascript
   * ~[1,2,3].indexOf(1) === -1;
   * ```
   */
  allow?: string[];
  /**
   * When set to `true` the `int32Hint` option allows the use of bitwise OR in |0
   * pattern for type casting.
   *
   * For example with `{ "int32Hint": true }` the following is permitted:
   *
   * ```javascript
   * const b = a|0;
   * ```
   */
  int32Hint?: boolean;
}
export interface NoConsoleConfig {
  /**
   * The `allow` option permits the given list of console methods to be used as exceptions to
   * this rule.
   *
   * Say the option was configured as `{ "allow": ["info"] }` then the rule would behave as
   * follows:
   *
   * Example of **incorrect** code for this option:
   * ```javascript
   * console.log('foo');
   * ```
   *
   * Example of **correct** code for this option:
   * ```javascript
   * console.info('foo');
   * ```
   */
  allow?: string[];
}
export interface NoConstantBinaryExpressionConfig {
  checkRelationalComparisons?: boolean;
}
export interface NoConstantCondition {
  /**
   * Configuration option to specify whether to check for constant conditions in loops.
   *
   * - `"all"` or `true` disallows constant expressions in loops
   * - `"allExceptWhileTrue"` disallows constant expressions in loops except while loops with expression `true`
   * - `"none"` or `false` allows constant expressions in loops
   */
  checkLoops?: CheckLoopsConfig;
}
export interface NoDuplicateImports {
  /**
   * When `true`, imports with only type specifiers (inline types or type imports) are
   * considered separate from imports with value specifiers, so they can be imported from the
   * same module on separate import statements.
   *
   * Examples of **correct** code when `allowSeparateTypeImports` is set to `true`:
   * ```js
   * import { foo } from "module";
   * import type { Bar } from "module";
   * ```
   *
   * ```js
   * import { type Foo } from "module";
   * import type { Bar } from "module";
   * ```
   */
  allowSeparateTypeImports?: boolean;
  /**
   * When `true` this rule will also look at exports to see if there is both a re-export of a
   * module as in `export ... from 'module'` and also a standard import statement for the same
   * module. This would count as a rule violation because there are in a sense two statements
   * importing from the same module.
   *
   * Examples of **incorrect** code when `includeExports` is set to `true`:
   * ```js
   * import { merge } from 'module';
   *
   * export { find } from 'module'; // re-export which is an import and an export.
   * ```
   *
   * Examples of **correct** code when `includeExports` is set to `true`:
   *
   * If re-exporting from an imported module, you should add the imports to the
   * `import` statement, and export that directly, not use `export ... from`.
   * ```js
   * import { merge } from "lodash-es";
   * export { merge as lodashMerge }
   * ```
   *
   * ```js
   * import { merge, find } from 'module';
   *
   * // cannot be merged with the above import
   * export * as something from 'module';
   *
   * // cannot be written differently
   * export * from 'module';
   * ```
   */
  includeExports?: boolean;
}
export interface NoElseReturn {
  /**
   * Whether to allow `else if` blocks after a return statement.
   *
   * Examples of **incorrect** code for this rule with `allowElseIf: false`:
   * ```javascript
   * function foo() {
   * if (error) {
   * return 'It failed';
   * } else if (loading) {
   * return "It's still loading";
   * }
   * }
   * ```
   *
   * Examples of **correct** code for this rule with `allowElseIf: false`:
   * ```javascript
   * function foo() {
   * if (error) {
   * return 'It failed';
   * }
   *
   * if (loading) {
   * return "It's still loading";
   * }
   * }
   * ```
   */
  allowElseIf?: boolean;
}
export interface NoEmpty {
  /**
   * If set to `true`, allows an empty `catch` block without triggering the linter.
   */
  allowEmptyCatch?: boolean;
}
export interface NoEmptyFunctionConfig {
  /**
   * Types of functions that are allowed to be empty.
   *
   * By default, no function kinds are allowed to be empty, but this option can be used to
   * permit specific kinds of functions.
   *
   * Example:
   * ```json
   * {
   *   "no-empty-function": [
   *     "error",
   *     {
   *       "allow": [
   *         "constructors"
   *       ]
   *     }
   *   ]
   * }
   * ```
   */
  allow?: AllowKind[];
}
export interface NoEmptyPattern {
  /**
   * When set to `true`, this rule allows empty object patterns used directly as function
   * parameters, including parameters defaulted to an empty object literal.
   */
  allowObjectPatternsAsParameters?: boolean;
}
export interface NoEval {
  /**
   * This `allowIndirect` option allows indirect `eval()` calls.
   *
   * Indirect calls to `eval`(e.g., `window['eval']`) are less dangerous
   * than direct calls because they cannot dynamically change the scope.
   * Indirect `eval()` calls also typically have less impact on performance
   * compared to direct calls, as they do not invoke JavaScript's scope chain.
   */
  allowIndirect?: boolean;
}
export interface NoExtendNativeConfig {
  /**
   * A list of objects which are allowed to be exceptions to the rule.
   */
  exceptions?: string[];
}
export interface NoExtraBooleanCast {
  /**
   * when set to `true`, in addition to checking default contexts, checks
   * whether extra boolean casts are present in expressions whose result is
   * used in a boolean context. See examples below. Default is `false`,
   * meaning that this rule by default does not warn about extra booleans
   * cast inside inner expressions.
   */
  enforceForInnerExpressions?: boolean;
}
export interface NoFallthroughConfig {
  /**
   * Whether to allow empty case clauses to fall through.
   */
  allowEmptyCase?: boolean;
  /**
   * Custom regex pattern to match fallthrough comments.
   */
  commentPattern?: string;
  /**
   * Whether to report unused fallthrough comments.
   */
  reportUnusedFallthroughComment?: boolean;
}
export interface NoGlobalAssignConfig {
  /**
   * List of global variable names to exclude from this rule.
   * Globals listed here can be assigned to without triggering warnings.
   */
  exceptions?: string[];
}
export interface NoImplicitCoercionConfig {
  /**
   * List of operators to allow. Valid values: `"!!"`, `"~"`, `"+"`, `"-"`, `"- -"`, `"*"`
   */
  allow?: string[];
  /**
   * When `true`, warns on implicit boolean coercion (e.g., `!!foo`).
   */
  boolean?: boolean;
  /**
   * When `true`, disallows using template literals for string coercion (e.g., `` `${foo}` ``).
   */
  disallowTemplateShorthand?: boolean;
  /**
   * When `true`, warns on implicit number coercion (e.g., `+foo`).
   */
  number?: boolean;
  /**
   * When `true`, warns on implicit string coercion (e.g., `"" + foo`).
   */
  string?: boolean;
}
export interface NoImplicitGlobalsConfig {
  lexicalBindings?: boolean;
}
export interface NoInlineCommentsConfig {
  /**
   * A regex pattern to ignore certain inline comments.
   *
   * Comments matching this pattern will not be reported.
   *
   * Example configuration:
   * ```json
   * {
   *   "no-inline-comments": [
   *     "error",
   *     {
   *       "ignorePattern": "webpackChunkName"
   *     }
   *   ]
   * }
   * ```
   */
  ignorePattern?: string;
}
export interface NoInnerDeclarationsOptions {
  /**
   * Controls whether function declarations in nested blocks are allowed in strict mode (ES6+ behavior).
   */
  blockScopedFunctions?: BlockScopedFunctions;
}
export interface NoInvalidRegexpConfig {
  /**
   * Case-sensitive array of flags that will be allowed.
   */
  allowConstructorFlags?: string[];
}
export interface NoIrregularWhitespaceConfig {
  /**
   * Whether to skip irregular whitespace in comments.
   */
  skipComments?: boolean;
  /**
   * Whether to skip irregular whitespace in JSX text.
   */
  skipJSXText?: boolean;
  /**
   * Whether to skip irregular whitespace in regular expression literals.
   */
  skipRegExps?: boolean;
  /**
   * Whether to skip irregular whitespace in string literals.
   */
  skipStrings?: boolean;
  /**
   * Whether to skip irregular whitespace in template literals.
   */
  skipTemplates?: boolean;
}
export interface NoLabels {
  /**
   * If set to `true`, this rule ignores labels which are sticking to loop statements.
   * Examples of **correct** code with this option set to `true`:
   * ```js
   * label:
   * while (true) {
   * break label;
   * }
   * ```
   */
  allowLoop?: boolean;
  /**
   * If set to `true`, this rule ignores labels which are sticking to switch statements.
   * Examples of **correct** code with this option set to `true`:
   * ```js
   * label:
   * switch (a) {
   * case 0:
   * break label;
   * }
   * ```
   */
  allowSwitch?: boolean;
}
export interface NoMagicNumbersConfig {
  /**
   * When true, numeric literals used in object properties are considered magic numbers.
   */
  detectObjects?: boolean;
  /**
   * When true, enforces that number constants must be declared using `const` instead of `let` or `var`.
   */
  enforceConst?: boolean;
  /**
   * An array of numbers to ignore if used as magic numbers. Can include floats or BigInt strings.
   */
  ignore?: NoMagicNumbersNumber[];
  /**
   * When true, numeric literals used as array indexes are ignored.
   */
  ignoreArrayIndexes?: boolean;
  /**
   * When true, numeric literals used as initial values in class fields are ignored.
   */
  ignoreClassFieldInitialValues?: boolean;
  /**
   * When true, numeric literals used as default values in function parameters and destructuring are ignored.
   */
  ignoreDefaultValues?: boolean;
  /**
   * When true, numeric literals in TypeScript enums are ignored.
   */
  ignoreEnums?: boolean;
  /**
   * When true, numeric literals used as TypeScript numeric literal types are ignored.
   */
  ignoreNumericLiteralTypes?: boolean;
  /**
   * When true, numeric literals in readonly class properties are ignored.
   */
  ignoreReadonlyClassProperties?: boolean;
  /**
   * When true, numeric literals used to index TypeScript types are ignored.
   */
  ignoreTypeIndexes?: boolean;
}
export interface NoMisleadingCharacterClass {
  /**
   * When set to `true`, the rule allows any grouping of code points
   * inside a character class as long as they are written using escape sequences.
   *
   * Examples of **incorrect** code for this rule with `{ "allowEscape": true }`:
   * ```javascript
   * /[\uD83D]/; // backslash can be omitted
   * new RegExp("[\ud83d" + "\udc4d]");
   * ```
   *
   * Examples of **correct** code for this rule with `{ "allowEscape": true }`:
   * ```javascript
   * /[\ud83d\udc4d]/;
   * /[\u00B7\u0300-\u036F]/u;
   * /[👨\u200d👩]/u;
   * new RegExp("[\x41\u0301]");
   * new RegExp(`[\u{1F1EF}\u{1F1F5}]`, "u");
   * new RegExp("[\\u{1F1EF}\\u{1F1F5}]", "u");
   * ```
   */
  allowEscape?: boolean;
}
export interface NoMultiAssign {
  /**
   * When set to `true`, the rule allows chains that don't include initializing a variable in a declaration or initializing a class field.
   *
   * Examples of **correct** code for this option set to `true`:
   * ```js
   * let a;
   * let b;
   * a = b = "baz";
   *
   * const x = {};
   * const y = {};
   * x.one = y.one = 1;
   * ```
   *
   * Examples of **incorrect** code for this option set to `true`:
   * ```js
   * let a = b = "baz";
   *
   * const foo = bar = 1;
   *
   * class Foo {
   * a = b = 10;
   * }
   * ```
   */
  ignoreNonDeclaration?: boolean;
}
export interface NoParamReassignConfig {
  /**
   * An array of parameter names whose property modifications should be ignored.
   */
  ignorePropertyModificationsFor?: string[];
  /**
   * An array of regex patterns (as strings) for parameter names whose property modifications should be ignored.
   * Note that this uses [Rust regex syntax](https://docs.rs/regex/latest/regex/) and so may not have all features
   * available to JavaScript regexes.
   */
  ignorePropertyModificationsForRegex?: string[];
  /**
   * When true, also check for modifications to properties of parameters.
   */
  props?: boolean;
}
export interface NoPlusplus {
  /**
   * Whether to allow `++` and `--` in for loop afterthoughts.
   */
  allowForLoopAfterthoughts?: boolean;
}
export interface NoPromiseExecutorReturnConfig {
  /**
   * If `true`, allows returning `void` expressions (e.g., `return void resolve()`).
   */
  allowVoid?: boolean;
}
export interface NoRedeclare {
  /**
   * When set `true`, it flags redeclaring built-in globals (e.g., `let Object = 1;`).
   */
  builtinGlobals?: boolean;
}
export interface NoRestrictedExportsConfig {
  /**
   * An object with boolean properties to restrict certain default export
   * declarations. This option works only if the `restrictedNamedExports`
   * option does not contain the `"default"` value.
   */
  restrictDefaultExports?: RestrictDefaultExports;
  /**
   * An array of strings, where each string is a name to be restricted.
   *
   * Example of **incorrect** code for `"restrictedNamedExports": ["foo"]`:
   *
   * ```ts
   * export const foo = 1;
   * ```
   *
   * Example of **correct** code for `"restrictedNamedExports": ["foo"]`:
   *
   * ```ts
   * export const bar = 1;
   * ```
   *
   * By design, this option doesn't disallow export default declarations. If
   * you configure `default` as a restricted name, that restriction will apply
   * only to named export declarations.
   *
   * Example of **incorrect** code for `"restrictedNamedExports": ["default"]`:
   *
   * ```ts
   * function foo() {}
   * export { foo as default };
   *
   * export { default } from "some_module";
   * ```
   */
  restrictedNamedExports?: string[];
  /**
   * A string representing a regular expression pattern. Named exports
   * matching this pattern will be restricted. This option does not apply to
   * default named exports.
   *
   * Example of **incorrect** code for `"restrictedNamedExportsPattern": "bar$":
   *
   * ```ts
   * export const foobar = 1;
   * ```
   *
   * Example of **correct** code for `"restrictedNamedExportsPattern": "bar$":
   *
   * ```ts
   * export const foo = 1;
   * ```
   */
  restrictedNamedExportsPattern?: string;
}
export interface RestrictDefaultExports {
  /**
   * Whether to restrict `export { default } from` declarations.
   *
   * Example of **incorrect** code for `"restrictDefaultExports": { "defaultFrom": true }`:
   *
   * ```js
   * export { default } from 'foo';
   * ```
   */
  defaultFrom?: boolean;
  /**
   * Whether to restrict `export default` declarations.
   *
   * Example of **incorrect** code for `"restrictDefaultExports": { "direct": true }`:
   *
   * ```js
   * const foo = 123;
   * export default foo;
   * ```
   */
  direct?: boolean;
  /**
   * Whether to restrict `export { foo as default }` declarations.
   *
   * Example of **incorrect** code for `"restrictDefaultExports": { "named": true }`:
   *
   * ```js
   * const foo = 123;
   * export { foo as default };
   * ```
   */
  named?: boolean;
  /**
   * Whether to restrict `export { foo as default } from` declarations.
   *
   * Example of **incorrect** code for `"restrictDefaultExports": { "namedFrom": true }`:
   *
   * ```js
   * export { foo as default } from 'foo';
   * ```
   */
  namedFrom?: boolean;
  /**
   * Whether to restrict `export * as default from` declarations.
   *
   * Example of **incorrect** code for `"restrictDefaultExports": { "namespaceFrom": true }`:
   *
   * ```js
   * export * as default from 'foo';
   * ```
   */
  namespaceFrom?: boolean;
}
export interface RestrictedPath {
  allowImportNames?: string[];
  allowTypeImports?: boolean;
  importNames?: string[];
  message?: string;
  name: string;
}
export interface NoRestrictedImportsConfig {
  paths?: PossiblePaths[];
  patterns?: PossiblePatterns[];
}
export interface RestrictedPattern {
  allowImportNamePattern?: string;
  allowImportNames?: string[];
  allowTypeImports?: boolean;
  caseSensitive?: boolean;
  group?: string[];
  importNamePattern?: string;
  importNames?: string[];
  message?: string;
  regex?: string;
}
export interface PropertyDetails {
  /**
   * Objects where property access should be allowed. This must be used with `property` and
   * cannot be used with `object`.
   */
  allowObjects?: string[];
  /**
   * Properties where property access should be allowed. This must be used with `object` and
   * cannot be used with `property`.
   */
  allowProperties?: string[];
  /**
   * A custom message to display.
   */
  message?: string;
  /**
   * The object on which the property is being accessed.
   */
  object?: string;
  /**
   * The property being accessed. If `object` is not specified, this applies to the named
   * property on all objects.
   */
  property?: string;
}
export interface NoSelfAssign {
  /**
   * The `props` option when set to `false`, disables the checking of properties.
   *
   * With `props` set to `false` the following are examples of correct code:
   * ```javascript
   * obj.a = obj.a;
   * obj.a.b = obj.a.b;
   * obj["a"] = obj["a"];
   * obj[a] = obj[a];
   * ```
   */
  props?: boolean;
}
export interface NoSequences {
  /**
   * If this option is set to `false`, this rule disallows the comma operator
   * even when the expression sequence is explicitly wrapped in parentheses.
   */
  allowInParentheses?: boolean;
}
export interface NoShadowConfig {
  /**
   * List of variable names that are allowed to shadow.
   */
  allow?: string[];
  /**
   * Whether to report shadowing of built-in global variables.
   */
  builtinGlobals?: boolean;
  /**
   * Controls how hoisting is handled.
   */
  hoist?: HoistOption;
  /**
   * If `true`, ignore when a function type parameter shadows a value.
   * Example: `const T = 1; function foo<T>() {}`
   */
  ignoreFunctionTypeParameterNameValueShadow?: boolean;
  /**
   * Whether to ignore the variable initializers when the shadowed variable is presumably still uninitialized.
   */
  ignoreOnInitialization?: boolean;
  /**
   * If `true`, ignore when a type and a value have the same name.
   * This is common in TypeScript: `type Foo = ...; const Foo = ...;`
   */
  ignoreTypeValueShadow?: boolean;
}
export interface NoShadowRestrictedNamesConfig {
  /**
   * If true, also report shadowing of `globalThis`.
   */
  reportGlobalThis?: boolean;
}
export interface NoUndef {
  /**
   * When set to `true`, warns on undefined variables used in a `typeof` expression.
   */
  typeof?: boolean;
}
export interface NoUnderscoreDangleConfig {
  /**
   * An array of variable names that are allowed to have dangling underscores.
   */
  allow?: string[];
  /**
   * Whether to allow dangling underscores in members of the `super` object.
   */
  allowAfterSuper?: boolean;
  /**
   * Whether to allow dangling underscores in members of the `this` object.
   */
  allowAfterThis?: boolean;
  /**
   * Whether to allow dangling underscores in members of the `this.constructor` object.
   */
  allowAfterThisConstructor?: boolean;
  /**
   * Whether to allow dangling underscores in function parameter names.
   */
  allowFunctionParams?: boolean;
  /**
   * Whether to allow dangling underscores in variable names assigned by array destructuring.
   */
  allowInArrayDestructuring?: boolean;
  /**
   * Whether to allow dangling underscores in variable names assigned by object destructuring.
   */
  allowInObjectDestructuring?: boolean;
  /**
   * Whether to allow dangling underscores in `using` and `await using` declarations.
   */
  allowInUsingDeclarations?: boolean;
  /**
   * Whether to enforce dangling underscores in class field names.
   */
  enforceInClassFields?: boolean;
  /**
   * Whether to enforce dangling underscores in method names.
   */
  enforceInMethodNames?: boolean;
}
export interface NoUnneededTernary {
  /**
   * Whether to allow the default assignment pattern `x ? x : y`.
   *
   * When set to `false`, the rule also flags cases like `x ? x : y` and suggests using
   * the logical OR form `x || y` instead. When `true` (default), such default assignments
   * are allowed and not reported.
   */
  defaultAssignment?: boolean;
}
export interface NoUnreachableLoopConfig {
  ignore?: LoopType[];
}
export interface NoUnsafeNegation {
  /**
   * The `enforceForOrderingRelations` option determines whether negation is allowed
   * on the left-hand side of ordering relational operators (<, >, <=, >=).
   *
   * The purpose is to avoid expressions such as `!a < b` (which is equivalent to `(a ? 0 : 1) < b`)
   * when what is really intended is `!(a < b)`.
   */
  enforceForOrderingRelations?: boolean;
}
export interface NoUnsafeOptionalChaining {
  /**
   * Disallow arithmetic operations on optional chaining expressions.
   * If this is true, this rule warns arithmetic operations on optional chaining expressions, which possibly result in NaN.
   */
  disallowArithmeticOperators?: boolean;
}
export interface NoUnusedExpressionsConfig {
  /**
   * When set to `true`, allows short circuit evaluations in expressions.
   */
  allowShortCircuit?: boolean;
  /**
   * When set to `true`, allows tagged template literals in expressions.
   */
  allowTaggedTemplates?: boolean;
  /**
   * When set to `true`, allows ternary operators in expressions.
   */
  allowTernary?: boolean;
  /**
   * When set to `true`, enforces the rule for unused JSX expressions also.
   */
  enforceForJSX?: boolean;
  /**
   * When set to `true`, allows directive prologues.
   */
  ignoreDirectives?: boolean;
}
export interface NoUnusedVarsOptions {
  /**
   * Controls how unused arguments are checked.
   */
  args?: ArgsOption;
  /**
   * Specifies exceptions to this rule for unused arguments. Arguments whose
   * names match this pattern will be ignored.
   *
   * By default, this pattern is `^_` unless options are configured with an
   * object. In this case it will default to [`None`]. Note that this
   * behavior deviates from both ESLint and TypeScript-ESLint, which never
   * provide a default pattern.
   *
   * #### Example
   *
   * Examples of **correct** code for this option when the pattern is `^_`:
   *
   * ```javascript
   * function foo(_a, b) {
   * console.log(b);
   * }
   * foo(1, 2);
   * ```
   */
  argsIgnorePattern?: IgnorePatternFor_String;
  /**
   * Used for `catch` block validation.
   */
  caughtErrors?: CaughtErrorsJson;
  /**
   * Specifies exceptions to this rule for errors caught within a `catch` block.
   * Variables declared within a `catch` block whose names match this pattern
   * will be ignored.
   *
   * #### Example
   *
   * Examples of **correct** code when the pattern is `^ignore`:
   *
   * ```javascript
   * try {
   * // ...
   * } catch (ignoreErr) {
   * console.error("Error caught in catch block");
   * }
   * ```
   */
  caughtErrorsIgnorePattern?: IgnorePatternFor_String;
  /**
   * This option specifies exceptions within destructuring patterns that will
   * not be checked for usage. Variables declared within array destructuring
   * whose names match this pattern will be ignored.
   *
   * By default this pattern is unset.
   *
   * #### Example
   *
   * Examples of **correct** code for this option, when the pattern is `^_`:
   * ```javascript
   * const [a, _b, c] = ["a", "b", "c"];
   * console.log(a + c);
   *
   * const { x: [_a, foo] } = bar;
   * console.log(foo);
   *
   * let _m, n;
   * foo.forEach(item => {
   * [_m, n] = item;
   * console.log(n);
   * });
   * ```
   */
  destructuredArrayIgnorePattern?: IgnorePatternFor_String;
  /**
   * Controls which `no-unused-vars` auto-fixes are emitted.
   *
   * When omitted, both `imports` and `variables` default to `"suggestion"`,
   * preserving the current behavior.
   *
   * NOTE: This option is experimental and may change based on feedback.
   */
  fix?: NoUnusedVarsFixOptions;
  /**
   * The `ignoreClassWithStaticInitBlock` option is a boolean. Static
   * initialization blocks allow you to initialize static variables and
   * execute code during the evaluation of a class definition, meaning
   * the static block code is executed without creating a new instance
   * of the class. When set to `true`, this option ignores classes
   * containing static initialization blocks.
   *
   * #### Example
   *
   * Examples of **incorrect** code for the `{ "ignoreClassWithStaticInitBlock": true }` option
   *
   * ```javascript
   * /* no-unused-vars: ["error", { "ignoreClassWithStaticInitBlock": true }]* /
   *
   * class Foo {
   * static myProperty = "some string";
   * static mymethod() {
   * return "some string";
   * }
   * }
   *
   * class Bar {
   * static {
   * let baz; // unused variable
   * }
   * }
   * ```
   *
   * Examples of **correct** code for the `{ "ignoreClassWithStaticInitBlock": true }` option
   *
   * ```javascript
   * /* no-unused-vars: ["error", { "ignoreClassWithStaticInitBlock": true }]* /
   *
   * class Foo {
   * static {
   * let bar = "some string";
   *
   * console.log(bar);
   * }
   * }
   * ```
   */
  ignoreClassWithStaticInitBlock?: boolean;
  /**
   * Using a Rest property it is possible to "omit" properties from an
   * object, but by default the sibling properties are marked as "unused".
   * With this option enabled the rest property's siblings are ignored.
   *
   *
   * #### Example
   * Examples of **correct** code when this option is set to `true`:
   * ```js
   * // 'foo' and 'bar' were ignored because they have a rest property sibling.
   * var { foo, ...coords } = data;
   *
   * var bar;
   * ({ bar, ...coords } = data);
   * ```
   */
  ignoreRestSiblings?: boolean;
  /**
   * When set to `true`, the rule will ignore variables declared with
   * `using` or `await using` declarations, even if they are unused.
   *
   * This is useful when working with resources that need to be disposed
   * via the explicit resource management proposal, where the primary
   * purpose is the disposal side effect rather than using the resource.
   *
   * #### Example
   *
   * Examples of **correct** code for the `{ "ignoreUsingDeclarations": true }` option:
   *
   * ```javascript
   * /* no-unused-vars: ["error", { "ignoreUsingDeclarations": true }]* /
   *
   * using resource = getResource();
   * await using anotherResource = getAnotherResource();
   * ```
   */
  ignoreUsingDeclarations?: boolean;
  /**
   * The `reportUsedIgnorePattern` option is a boolean.
   * Using this option will report variables that match any of the valid
   * ignore pattern options (`varsIgnorePattern`, `argsIgnorePattern`,
   * `caughtErrorsIgnorePattern`, or `destructuredArrayIgnorePattern`) if
   * they have been used.
   *
   * #### Example
   *
   * Examples of **incorrect** code for the `{ "reportUsedIgnorePattern": true }` option:
   *
   * ```javascript
   * /* no-unused-vars: ["error", { "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]* /
   *
   * var firstVarIgnored = 1;
   * var secondVar = 2;
   * console.log(firstVarIgnored, secondVar);
   * ```
   *
   * Examples of **correct** code for the `{ "reportUsedIgnorePattern": true }` option:
   *
   * ```javascript
   * /* no-unused-vars: ["error", { "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]* /
   *
   * var firstVar = 1;
   * var secondVar = 2;
   * console.log(firstVar, secondVar);
   * ```
   */
  reportUsedIgnorePattern?: boolean;
  /**
   * The `reportVarsOnlyUsedAsTypes` option is a boolean.
   *
   * If `true`, the rule will also report variables that are only used as types.
   *
   * #### Examples
   *
   * Examples of **incorrect** code for the `{ "reportVarsOnlyUsedAsTypes": true }` option:
   *
   * ```javascript
   * /*  no-unused-vars: ["error", { "reportVarsOnlyUsedAsTypes": true }] * /
   *
   * const myNumber: number = 4;
   * export type MyNumber = typeof myNumber
   * ```
   *
   * Examples of **correct** code for the `{ "reportVarsOnlyUsedAsTypes": true }` option:
   *
   * ```javascript
   * export type MyNumber = number;
   * ```
   *
   * Note: even with `{ "reportVarsOnlyUsedAsTypes": false }`, cases where the value is
   * only used a type within itself will still be reported:
   * ```javascript
   * function foo(): typeof foo {}
   * ```
   */
  reportVarsOnlyUsedAsTypes?: boolean;
  /**
   * Controls how usage of a variable in the global scope is checked.
   */
  vars?: VarsOption;
  /**
   * Specifies exceptions to this rule for unused variables. Variables whose
   * names match this pattern will be ignored.
   *
   * By default, this pattern is `^_` unless options are configured with an
   * object. In this case it will default to [`None`]. Note that this
   * behavior deviates from both ESLint and TypeScript-ESLint, which never
   * provide a default pattern.
   *
   * #### Example
   *
   * Examples of **correct** code for this option when the pattern is `^_`:
   * ```javascript
   * var _a = 10;
   * var b = 10;
   * console.log(b);
   * ```
   */
  varsIgnorePattern?: IgnorePatternFor_String;
}
/**
 * Fine-grained auto-fix controls for `no-unused-vars`.
 */
export interface NoUnusedVarsFixOptions {
  /**
   * Controls auto-fixes for unused imports.
   */
  imports?: NoUnusedVarsFixMode;
  /**
   * Controls auto-fixes for unused variables (including catch bindings).
   */
  variables?: NoUnusedVarsFixMode;
}
export interface NoUseBeforeDefineConfig {
  /**
   * Allow named exports that appear before declaration.
   */
  allowNamedExports?: boolean;
  /**
   * Check class declarations.
   */
  classes?: boolean;
  /**
   * Check enum declarations.
   */
  enums?: boolean;
  /**
   * Check function declarations.
   */
  functions?: boolean;
  /**
   * Ignore usages that are type-only references.
   */
  ignoreTypeReferences?: boolean;
  /**
   * Check type aliases, interfaces, and type parameters.
   */
  typedefs?: boolean;
  /**
   * Check variable declarations.
   */
  variables?: boolean;
}
export interface NoUselessComputedKey {
  /**
   * The `enforceForClassMembers` option controls whether the rule applies to
   * class members (methods and properties).
   *
   * Examples of **correct** code for this rule with the `{ "enforceForClassMembers": false }` option:
   * ```js
   * class SomeClass {
   * ["foo"] = "bar";
   * [42] = "baz";
   * get ['b']() {}
   * set ['c'](value) {}
   * static ["foo"] = "bar";
   * }
   * ```
   */
  enforceForClassMembers?: boolean;
}
export interface NoUselessEscapeConfig {
  /**
   * An array of characters that are allowed to be escaped unnecessarily in regexes.
   * For example, setting this to `["#"]` allows `\#` in regexes.
   *
   * Each string in this array must be a single character.
   */
  allowRegexCharacters?: string[];
}
export interface NoUselessRenameConfig {
  /**
   * When set to `true`, allows using the same name in destructurings.
   */
  ignoreDestructuring?: boolean;
  /**
   * When set to `true`, allows renaming exports to the same name.
   */
  ignoreExport?: boolean;
  /**
   * When set to `true`, allows renaming imports to the same name.
   */
  ignoreImport?: boolean;
}
export interface NoVoid {
  /**
   * If set to `true`, using `void` as a standalone statement is allowed.
   */
  allowAsStatement?: boolean;
}
export interface NoWarningCommentsConfigJson {
  decoration?: string[];
  location?: Location;
  terms?: string[];
}
export interface NoMixedRequiresOptions {
  allowCall?: boolean;
  grouping?: boolean;
}
export interface NoProcessEnvConfig {
  /**
   * Variable names which are allowed to be accessed on `process.env`.
   */
  allowedVariables?: string[];
}
export interface NoSyncConfig {
  /**
   * Whether synchronous methods should be allowed at the top level of a file.
   */
  allowAtRootLevel?: boolean;
  /**
   * Function names to ignore.
   */
  ignores?: string[];
}
export interface ObjectShorthandOptions {
  avoidExplicitReturnArrows?: boolean;
  avoidQuotes?: boolean;
  ignoreConstructors?: boolean;
  methodsIgnorePattern?: string;
}
export interface NoAsyncEndpointHandlersConfig {
  /**
   * An array of names that are allowed to be async.
   */
  allowedNames?: string[];
}
export interface NoBarrelFile {
  /**
   * The maximum number of modules that can be re-exported via `export *`
   * before the rule is triggered.
   */
  threshold?: number;
}
export interface NoMapSpreadConfig {
  /**
   * Ignore maps on arrays passed as parameters to a function.
   *
   * This option is enabled by default to better avoid false positives. It
   * comes at the cost of potentially missing spreads that are inefficient.
   * We recommend turning this off in your `.oxlintrc.json` files.
   *
   * #### Examples
   *
   * Examples of **incorrect** code for this rule when `ignoreArgs` is `true`:
   * ```ts
   * /* "oxc/no-map-spread": ["error", { "ignoreArgs": true }] * /
   * function foo(arr) {
   * let arr2 = arr.filter(x => x.a > 0);
   * return arr2.map(x => ({ ...x }));
   * }
   * ```
   *
   * Examples of **correct** code for this rule when `ignoreArgs` is `true`:
   * ```ts
   * /* "oxc/no-map-spread": ["error", { "ignoreArgs": true }] * /
   * function foo(arr) {
   * return arr.map(x => ({ ...x }));
   * }
   * ```
   */
  ignoreArgs?: boolean;
  /**
   * Ignore mapped arrays that are re-read after the `map` call.
   *
   * Re-used arrays may rely on shallow copying behavior to avoid mutations.
   * In these cases, `Object.assign` is not really more performant than spreads.
   */
  ignoreRereads?: boolean;
}
export interface NoOptionalChainingConfig {
  /**
   * A custom help message to display when optional chaining is found.
   * For example, "Our output target is ES2016, and optional chaining results in verbose
   * helpers and should be avoided."
   */
  message?: string;
}
export interface NoRestSpreadPropertiesOptions {
  /**
   * A message to display when object rest properties are found.
   */
  objectRestMessage?: string;
  /**
   * A message to display when object spread properties are found.
   */
  objectSpreadMessage?: string;
}
export interface PreferArrowCallbackConfig {
  /**
   * If this option is set to `true`, named function expressions are allowed.
   */
  allowNamedFunctions?: boolean;
  /**
   * If this option is set to `false`, function expressions that reference `this` are reported even when they are not bound to a `this` value.
   */
  allowUnboundThis?: boolean;
}
export interface PreferConstConfig {
  /**
   * Configures how destructuring assignments are handled.
   */
  destructuring?: Destructuring;
  /**
   * If `true`, the rule will not report variables that are read before their initial assignment.
   * This is mainly useful for preventing conflicts with the `typescript/no-use-before-define` rule.
   */
  ignoreReadBeforeAssign?: boolean;
}
export interface PreferDestructuringTargetOption {
  array?: boolean;
  object?: boolean;
}
export interface PreferDestructuringAssignmentConfig {
  AssignmentExpression?: PreferDestructuringTargetOption;
  VariableDeclarator?: PreferDestructuringTargetOption;
}
export interface PreferDestructuringRenamedPropertiesConfig {
  enforceForRenamedProperties?: boolean;
}
export interface PreferPromiseRejectErrors {
  /**
   * Whether to allow calls to `Promise.reject()` with no arguments.
   */
  allowEmptyReject?: boolean;
}
export interface PreferRegexLiteralsConfig {
  /**
   * By default, this rule doesn’t check when a regex literal is unnecessarily wrapped in a `RegExp` constructor call.
   * When the option `disallowRedundantWrapping` is set to `true`, the rule will also disallow such unnecessary patterns.
   *
   * Examples of **incorrect** code for `{ "disallowRedundantWrapping": true }`:
   * ```js
   * new RegExp(/abc/);
   * new RegExp(/abc/, 'u');
   * ```
   *
   * Examples of **correct** code for `{ "disallowRedundantWrapping": true }`:
   * ```js
   * /abc/;
   * /abc/u;
   * new RegExp(/abc/, flags);
   * ```
   */
  disallowRedundantWrapping?: boolean;
}
export interface PreserveCaughtErrorOptions {
  /**
   * When set to `true`, requires that catch clauses always have a parameter.
   */
  requireCatchParameter?: boolean;
}
export interface AlwaysReturnConfig {
  /**
   * You can pass an `{ ignoreAssignmentVariable: [] }` as an option to this rule
   * with a list of variable names so that the last `then()` callback in a promise
   * chain does not warn if it does an assignment to a global variable. Default is
   * `["globalThis"]`.
   *
   * ```javascript
   * /* promise/always-return: ["error", { ignoreAssignmentVariable: ["globalThis"] }] * /
   *
   * // OK
   * promise.then((x) => {
   * globalThis = x
   * })
   *
   * promise.then((x) => {
   * globalThis.x = x
   * })
   *
   * // OK
   * promise.then((x) => {
   * globalThis.x.y = x
   * })
   *
   * // NG
   * promise.then((x) => {
   * anyOtherVariable = x
   * })
   *
   * // NG
   * promise.then((x) => {
   * anyOtherVariable.x = x
   * })
   *
   * // NG
   * promise.then((x) => {
   * x()
   * })
   * ```
   */
  ignoreAssignmentVariable?: string[];
  /**
   * You can pass an `{ ignoreLastCallback: true }` as an option to this rule so that
   * the last `then()` callback in a promise chain does not warn if it does not have
   * a `return`. Default is `false`.
   *
   * ```javascript
   * // OK
   * promise.then((x) => {
   * console.log(x)
   * })
   * // OK
   * void promise.then((x) => {
   * console.log(x)
   * })
   * // OK
   * await promise.then((x) => {
   * console.log(x)
   * })
   *
   * promise
   * // NG
   * .then((x) => {
   * console.log(x)
   * })
   * // OK
   * .then((x) => {
   * console.log(x)
   * })
   *
   * // NG
   * const v = promise.then((x) => {
   * console.log(x)
   * })
   * // NG
   * const v = await promise.then((x) => {
   * console.log(x)
   * })
   * function foo() {
   * // NG
   * return promise.then((x) => {
   * console.log(x)
   * })
   * }
   * ```
   */
  ignoreLastCallback?: boolean;
}
export interface CatchOrReturnConfig {
  /**
   * Whether to allow `finally()` as a termination method.
   */
  allowFinally?: boolean;
  /**
   * Whether to allow `then()` with two arguments as a termination method.
   */
  allowThen?: boolean;
  /**
   * Whether to allow `then(null, handler)` as a termination method.
   */
  allowThenStrict?: boolean;
  /**
   * List of allowed termination methods (e.g., `catch`, `done`).
   */
  terminationMethod?: TerminationMethod;
}
export interface NoCallbackInPromiseConfig {
  /**
   * List of callback function names to allow within Promise `then` and `catch` methods.
   */
  exceptions?: string[];
  /**
   * Boolean as to whether callbacks in timeout functions like `setTimeout` will err.
   */
  timeoutsErr?: boolean;
}
export interface NoPromiseInCallbackConfig {
  /**
   * Whether or not to exempt function declarations. Defaults to `false`.
   */
  exemptDeclarations?: boolean;
}
export interface NoReturnWrap {
  /**
   * `allowReject` allows returning `Promise.reject` inside a promise handler.
   *
   * With `allowReject` set to `true` the following are examples of correct code:
   *
   * ```js
   * myPromise().then(
   * function() {
   * return Promise.reject(0)
   * })
   * ```
   *
   * ```js
   * myPromise().then().catch(() => Promise.reject("err"))
   * ```
   */
  allowReject?: boolean;
}
export interface ParamNamesConfig {
  /**
   * Regex pattern used to validate the `reject` parameter name. If provided, this pattern
   * is used instead of the default `^_?reject$` check.
   */
  rejectPattern?: string;
  /**
   * Regex pattern used to validate the `resolve` parameter name. If provided, this pattern
   * is used instead of the default `^_?resolve$` check.
   */
  resolvePattern?: string;
}
export interface PreferAwaitToThenConfig {
  /**
   * If true, enforces the rule even after an `await` or `yield` expression.
   */
  strict?: boolean;
}
export interface SpecOnlyConfig {
  /**
   * List of Promise static methods that are allowed to be used.
   */
  allowedMethods?: string[];
}
export interface ReactPerfConfig {
  /**
   * Controls whether native elements (lowercase-first-letter tags such as `div`)
   * are ignored by the rule. Set to `"all"` to ignore every attribute on native
   * elements, or to an array of attribute names to ignore only those attributes
   * on native elements.
   */
  nativeAllowList?: NativeAllowList;
}
export interface ButtonHasType {
  /**
   * If true, allow `type="button"`.
   */
  button?: boolean;
  /**
   * If true, allow `type="reset"`.
   */
  reset?: boolean;
  /**
   * If true, allow `type="submit"`.
   */
  submit?: boolean;
}
export interface CheckedRequiresOnchangeOrReadonly {
  /**
   * Ignore the restriction that `checked` and `defaultChecked` should not be used together.
   */
  ignoreExclusiveCheckedAttribute?: boolean;
  /**
   * Ignore the requirement to provide either `onChange` or `readOnly` when the `checked` prop is present.
   */
  ignoreMissingProperties?: boolean;
}
export interface DisplayNameConfig {
  /**
   * When `true`, this rule will warn on context objects
   * without a `displayName`.
   *
   * `displayName` allows you to [name your context](https://reactjs.org/docs/context.html#contextdisplayname) object.
   * This name is used in the React DevTools for the context's `Provider` and `Consumer`.
   */
  checkContextObjects?: boolean;
  /**
   * When `true`, the rule will ignore the name set by the transpiler
   * and require a `displayName` property in this case.
   */
  ignoreTranspilerName?: boolean;
}
export interface ExhaustiveDepsConfig {
  /**
   * Optionally provide a regex of additional hooks to check.
   */
  additionalHooks?: string;
}
export interface ForbidComponentPropsConfig {
  /**
   * An array specifying the names of props that are forbidden.
   *
   * The default value is `["className", "style"]`.
   *
   * Each array element can be a string with the property name, or an object with `propName` / `propNamePattern`,
   * `allowedFor` / `allowedForPatterns`, `disallowedFor` / `disallowedForPatterns`, optional custom `message`
   *
   * **Pattern matching**: Uses glob patterns to match prop names and component names.
   * For example, a `propNamePattern` of `"**-**"` would match any prop name that contains a hyphen, and an `allowedForPatterns` entry of `"*Icon"` would match component names like `SomeIcon` and `AnotherIcon`.
   * Note that the pattern matching is done in Rust with the fast-glob library, and so may differ
   * from the JavaScript glob library used by the original ESLint rule.
   *
   * Examples:
   *
   * - `["error", { "forbid": ["className", "style"] }]`
   * - `["error", { "forbid": [{ "propName": "className", "message": "Use variant instead" }] }]`
   * - `["error", { "forbid": [{ "propName": "className", "allowedFor": ["ReactModal"] }] }]`
   * - `["error", { "forbid": [{ "propNamePattern": "**-**", "disallowedFor": ["Foo"] }] }]`
   */
  forbid?: ForbidItem[];
}
export interface ForbidItemObject {
  /**
   * Component names for which this prop is **allowed** (all others are
   * forbidden).
   */
  allowedFor?: string[];
  /**
   * Glob patterns for component names where the prop is **allowed**.
   */
  allowedForPatterns?: string[];
  /**
   * Component names for which this prop is **disallowed** (all others are
   * allowed).
   */
  disallowedFor?: string[];
  /**
   * Glob patterns for component names where the prop is **disallowed**.
   */
  disallowedForPatterns?: string[];
  /**
   * Custom message to display.
   */
  message?: string;
  /**
   * Exact prop name to forbid.
   */
  propName?: string;
  /**
   * Glob pattern to match prop names against.
   */
  propNamePattern?: string;
}
/**
 * Configuration for the `forbid-dom-props` rule.
 */
export interface ForbidDomPropsConfig {
  /**
   * An array of prop names or objects that are forbidden on DOM elements.
   *
   * Each array element can be a string with the property name, or an object
   * with `propName`, optional `disallowedFor` and `disallowedValues` arrays,
   * and an optional custom `message`.
   *
   * Examples:
   *
   * - `["error", { "forbid": ["id", "style"] }]`
   * - `["error", { "forbid": [{ "propName": "className", "message": "Use class instead" }] }]`
   * - `["error", { "forbid": [{ "propName": "style", "disallowedFor": ["div", "span"] }] }]`
   * - `["error", { "forbid": [{ "propName": "type", "disallowedValues": ["button"] }] }]`
   */
  forbid?: ForbidDomPropsItem[];
}
/**
 * A prop with optional `disallowedFor` DOM node list, optional `disallowedValues`
 * value list, and custom `message`.
 */
export interface PropWithOptions {
  /**
   * A list of DOM element names (e.g. `["div", "span"]`) on which this
   * prop is forbidden. If empty or omitted, the prop is forbidden on all
   * DOM elements.
   */
  disallowedFor?: string[];
  /**
   * A list of string literal values for which this prop is forbidden. If
   * omitted, the prop is forbidden for all values.
   */
  disallowedValues?: string[];
  /**
   * A custom message to display when this prop is used.
   */
  message?: string;
  /**
   * The name of the prop to forbid.
   */
  propName: string;
}
export interface ForbidElementsConfig {
  /**
   * List of forbidden elements, with optional messages for display with lint violations.
   *
   * Examples:
   *
   * - `["error, { "forbid": ["button"] }]`
   * - `["error, { "forbid": [{ "element": "button", "message": "Use <Button> instead." }] }]`
   * - `["error, { "forbid": [{ "element": "input" }] }]`
   */
  forbid?: ForbidItem2[];
}
export interface HookUseStateConfig {
  /**
   * When true the rule will ignore the name of the destructured value.
   */
  allowDestructuredState?: boolean;
}
export interface JsxBooleanValueOptions {
  /**
   * List of attribute names that should always have explicit boolean values.
   * Only necessary when main mode is `"never"`.
   */
  always?: string[];
  /**
   * If `true`, treats `prop={false}` as equivalent to the prop being `undefined`.
   * When combined with `"never"` mode, this will enforce that the attribute is omitted entirely.
   *
   * ```jsx
   * // With "assumeUndefinedIsFalse": true
   * <App foo={false} />; // Incorrect
   * <App />;             // Correct
   * ```
   *
   * This option does nothing in `"always"` mode.
   */
  assumeUndefinedIsFalse?: boolean;
  /**
   * List of attribute names that should never have explicit boolean values.
   * Only necessary when main mode is `"always"`.
   */
  never?: string[];
}
export interface JsxCurlyBracePresence {
  /**
   * Whether to enforce or disallow curly braces for child content of a JSX element.
   *
   * - `never` will disallow unnecessary curly braces, e.g. this will be preferred: `<Foo>I love oxlint</Foo>`
   * - `always` will force the usage of curly braces like this, in all cases: `<Foo>{'I love oxlint'}</Foo>`
   * - `ignore` will allow either style for child content.
   */
  children?: JsxCurlyBracePresenceMode;
  /**
   * When set to `ignore` or `never`, this JSX code is allowed (or enforced):
   * `<App prop=<div /> />;`
   *
   * When set to `always`, the curly braces are required for prop values that are
   * JSX elements: `<App prop={<div />} />;`
   *
   * **Note**: it is _highly_ recommended that you set `propElementValues` to `always`.
   * The ability to omit curly braces around prop values that are JSX elements is obscure, and
   * intentionally undocumented, and should not be relied upon.
   */
  propElementValues?: JsxCurlyBracePresenceMode;
  /**
   * Whether to enforce or disallow curly braces for props on JSX elements.
   *
   * - `never` will disallow unnecessary curly braces, e.g. this will be preferred: `<Foo foo="bar" />`
   * - `always` will force the usage of curly braces like this, in all cases: `<Foo foo={'bar'} />`
   * - `ignore` will allow either style for prop values.
   */
  props?: JsxCurlyBracePresenceMode;
}
export interface JsxFilenameExtensionConfig {
  /**
   * When to allow a JSX filename extension. By default all files may have a JSX extension.
   * Set this to `as-needed` to only allow JSX file extensions in files that contain JSX syntax.
   */
  allow?: JsxFilenameExtensionAllowMode;
  /**
   * The set of allowed file extensions.
   * Can include or exclude the leading dot (e.g., "jsx" and ".jsx" are both valid).
   */
  extensions?: string[];
  /**
   * If enabled, files that do not contain code (i.e. are empty, contain only whitespaces or comments) will not be rejected.
   */
  ignoreFilesWithoutCode?: boolean;
}
export interface JsxHandlerNamesConfig {
  /**
   * Whether to check for inline functions in JSX attributes.
   */
  checkInlineFunction?: boolean;
  /**
   * Whether to check for local variables in JSX attributes.
   */
  checkLocalVariables?: boolean;
  /**
   * Event handler prefixes to check against.
   */
  eventHandlerPrefix?: string | false;
  /**
   * Event handler prop prefixes to check against.
   */
  eventHandlerPropPrefix?: string | false;
  /**
   * Component names to ignore when checking for event handler prefixes.
   */
  ignoreComponentNames?: string[];
}
export interface JsxKeyConfig {
  /**
   * When true, check fragment shorthand `<>` for keys
   */
  checkFragmentShorthand?: boolean;
  /**
   * When true, require key prop to be placed before any spread props
   */
  checkKeyMustBeforeSpread?: boolean;
  /**
   * When true, warn on duplicate key values
   */
  warnOnDuplicates?: boolean;
}
export interface JsxMaxDepthConfig {
  /**
   * The maximum allowed depth of nested JSX elements and fragments.
   */
  max?: number;
}
/**
 * The options shared between the top-level config and each `elementOverrides` entry.
 */
export interface JsxNoLiteralsConfig {
  /**
   * An array of unique string values that would otherwise warn, but will be ignored.
   */
  allowedStrings?: string[];
  /**
   * An object where the keys are the element names and the values are objects with the same options as above. This allows you to specify different options for different elements.
   */
  elementOverrides?: {
    [k: string]: ElementOverrideOptions | undefined;
  };
  /**
   * (default: false) - When true the rule ignores literals used in props, wrapped or unwrapped.
   */
  ignoreProps?: boolean;
  /**
   * (default: false) - Enforces no string literals used in attributes when set to true.
   */
  noAttributeStrings?: boolean;
  /**
   * (default: false) - Enforces no string literals used as children, wrapped or unwrapped.
   */
  noStrings?: boolean;
  /**
   * An array of unique attribute names where string literals should be restricted. Only the specified attributes will be checked for string literals when this option is used. Note: When noAttributeStrings is true, this option is ignored at the root level.
   */
  restrictedAttributes?: string[];
}
/**
 * One entry in `elementOverrides`: the base options plus override-only fields.
 */
export interface ElementOverrideOptions {
  /**
   * (default: false) - When true the rule will allow the specified element to have string literals as children, wrapped or unwrapped without warning.
   */
  allowElement?: boolean;
  /**
   * An array of unique string values that would otherwise warn, but will be ignored.
   */
  allowedStrings?: string[];
  /**
   * (default: true) - When false the rule will not apply the current options set to nested elements. This is useful when you want to apply the rule to a specific element, but not to its children.
   */
  applyToNestedElements?: boolean;
  /**
   * (default: false) - When true the rule ignores literals used in props, wrapped or unwrapped.
   */
  ignoreProps?: boolean;
  /**
   * (default: false) - Enforces no string literals used in attributes when set to true.
   */
  noAttributeStrings?: boolean;
  /**
   * (default: false) - Enforces no string literals used as children, wrapped or unwrapped.
   */
  noStrings?: boolean;
  /**
   * An array of unique attribute names where string literals should be restricted. Only the specified attributes will be checked for string literals when this option is used. Note: When noAttributeStrings is true, this option is ignored at the root level.
   */
  restrictedAttributes?: string[];
}
export interface JsxNoScriptUrlComponent {
  /**
   * Component name.
   */
  name: string;
  /**
   * List of properties that should be validated.
   */
  props: string[];
}
export interface JsxNoScriptUrlOptions {
  /**
   * Whether to include components from settings.
   */
  includeFromSettings?: boolean;
}
export interface JsxNoTargetBlank {
  /**
   * Whether to allow referrers.
   */
  allowReferrer?: boolean;
  /**
   * Whether to enforce dynamic links or enforce static links.
   */
  enforceDynamicLinks?: EnforceDynamicLinksEnum;
  /**
   * Whether to check form elements.
   */
  forms?: boolean;
  /**
   * Whether to check link elements.
   */
  links?: boolean;
  /**
   * Whether to warn when spread attributes are used.
   */
  warnOnSpreadAttributes?: boolean;
}
export interface JsxNoUselessFragment {
  /**
   * Allow fragments with a single expression child.
   */
  allowExpressions?: boolean;
}
export interface JsxPascalCaseConfig {
  /**
   * Whether to allow all-caps component names.
   */
  allowAllCaps?: boolean;
  /**
   * Whether to allow leading underscores in component names.
   */
  allowLeadingUnderscore?: boolean;
  /**
   * Whether to allow namespaced component names.
   */
  allowNamespace?: boolean;
  /**
   * List of component names to ignore.
   */
  ignore?: string[];
}
export interface JsxPropsNoSpreadingConfig {
  /**
   * `custom` set to `ignore` will ignore all custom jsx tags like `App`, `MyCustomComponent` etc. Default is set to `enforce`.
   */
  custom?: IgnoreEnforceOption;
  /**
   * Exceptions flip the enforcement behavior for specific components.
   * For example:
   * - If `html` is set to `ignore`, an exception for `div` will enforce the rule on `<div>` elements.
   * - If `custom` is set to `enforce`, an exception for `Foo` will ignore the rule on `<Foo>` components.
   *
   * This allows you to override the general setting for individual components.
   */
  exceptions?: string[];
  /**
   * `explicitSpread` set to `ignore` will ignore spread operators that are explicitly listing all object properties within that spread. Default is set to `enforce`.
   */
  explicitSpread?: IgnoreEnforceOption;
  /**
   * `html` set to `ignore` will ignore all html jsx tags like `div`, `img` etc. Default is set to `enforce`.
   */
  html?: IgnoreEnforceOption;
}
export interface NoMultiCompConfig {
  /**
   * When `true`, the rule will ignore stateless components and will allow you to have multiple
   * stateless components in the same file. Or one stateful component and one-or-more stateless
   * components in the same file.
   *
   * Stateless basically just means function components, including those created via
   * `memo` and `forwardRef`.
   */
  ignoreStateless?: boolean;
}
export interface NoStringRefs {
  /**
   * Disallow template literals in addition to string literals.
   */
  noTemplateLiterals?: boolean;
}
export interface NoUnknownPropertyConfig {
  /**
   * List of properties to ignore.
   */
  ignore?: string[];
  /**
   * Require `data-*` attributes to be lowercase, e.g. `data-foobar` instead of `data-fooBar`.
   */
  requireDataLowercase?: boolean;
}
export interface NoUnsafeConfig {
  /**
   * Whether to check for the non-prefixed lifecycle methods.
   * If `true`, this means `componentWillMount`, `componentWillReceiveProps`,
   * and `componentWillUpdate` will also be flagged, rather than just the
   * UNSAFE_ versions. It is recommended to set this to `true` to fully
   * avoid unsafe lifecycle methods.
   */
  checkAliases?: boolean;
}
export interface NoUnstableNestedComponentsConfig {
  /**
   * Allow component definitions in props.
   */
  allowAsProps?: boolean;
  /**
   * Optional custom propTypes validators accepted for eslint-plugin-react compatibility.
   */
  customValidators?: string[];
  /**
   * Glob pattern for render-prop names that may receive inline component definitions.
   */
  propNamePattern?: string;
}
export interface OnlyExportComponentsConfig {
  /**
   * Allow exporting primitive constants (string/number/boolean/template literal)
   * alongside component exports without triggering a violation. Recommended when your
   * bundler’s Fast Refresh integration supports this (enabled by the plugin’s `vite`
   * preset).
   *
   * ```jsx
   * // Allowed when allowConstantExport: true
   * export const VERSION = "3";
   * export const Foo = () => null;
   * ```
   */
  allowConstantExport?: boolean;
  /**
   * Treat specific named exports as HMR-safe (useful for frameworks that hot-replace
   * certain exports). For example, in Remix:
   * `{ "allowExportNames": ["meta", "links", "headers", "loader", "action"] }`
   */
  allowExportNames?: string[];
  /**
   * Check `.js` files that contain JSX (in addition to `.tsx`/`.jsx`). To reduce
   * false positives, only files that import React are checked when this is enabled.
   */
  checkJS?: boolean;
  /**
   * If you export components wrapped in custom higher-order components, list their
   * identifiers here to avoid false positives.
   */
  customHOCs?: string[];
}
export interface PreferFunctionComponent {
  /**
   * If `true`, error boundary classes (those implementing `componentDidCatch`
   * or `static getDerivedStateFromError`) are allowed as class components.
   *
   * This is because these classes are not easily converted to function components,
   * and so they are exempted from this rule by default.
   */
  allowErrorBoundary?: boolean;
  /**
   * If `true`, classes that contain JSX but do not extend `Component` or
   * `PureComponent` are allowed.
   */
  allowJsxUtilityClass?: boolean;
}
export interface ReactCompilerConfig {
  /**
   * Also report compiler bail-outs — places where React Compiler skipped a
   * component or hook (for example because of unsupported syntax) without
   * finding a rule violation. These do not indicate incorrect code, only
   * code that the compiler declined to optimize.
   */
  reportAllBailouts?: boolean;
}
export interface SelfClosingComp {
  /**
   * Whether to enforce self-closing for custom components.
   */
  component?: boolean;
  /**
   * Whether to enforce self-closing for native HTML elements.
   */
  html?: boolean;
}
export interface StylePropObjectConfig {
  /**
   * List of component names on which to allow `style` prop values of any type.
   */
  allow?: string[];
}
export interface RequireUnicodeRegexpConfig {
  /**
   * The `u` flag may be preferred in environments that do not support the `v` flag.
   *
   * Examples of **incorrect** code for this rule with the `{ "requireFlag": "u" }` option:
   * ```js
   * const fooEmpty = /foo/;
   * const fooEmptyRegexp = new RegExp('foo');
   * const foo = /foo/v;
   * const fooRegexp = new RegExp('foo', 'v');
   * ```
   *
   * Examples of **correct** code for this rule with the `{ "requireFlag": "u" }` option:
   * ```js
   * const foo = /foo/u;
   * const fooRegexp = new RegExp('foo', 'u');
   * ```
   *
   * The `v` flag may be a better choice when it is supported because it has more features than the `u` flag (e.g., the ability to test Unicode properties of strings).
   * It does have a stricter syntax, however (e.g., the need to escape certain characters within character classes).
   *
   * Examples of **incorrect** code for this rule with the `{ "requireFlag": "v" }` option:
   * ```js
   * const fooEmpty = /foo/;
   * const fooEmptyRegexp = new RegExp('foo');
   * const foo = /foo/u;
   * const fooRegexp = new RegExp('foo', 'u');
   * ```
   *
   * Examples of **correct** code for this rule with the `{ "requireFlag": "v" }` option:
   * ```js
   * const foo = /foo/v;
   * const fooRegexp = new RegExp('foo', 'v');
   * ```
   */
  requireFlag?: RequireFlag;
}
export interface SortImportsOptions {
  /**
   * When `true`, the rule allows import groups separated by blank lines to be treated independently.
   */
  allowSeparatedGroups?: boolean;
  /**
   * When `true`, the rule ignores case-sensitivity when sorting import names.
   */
  ignoreCase?: boolean;
  /**
   * When `true`, the rule ignores the sorting of import declarations (the order of `import` statements).
   */
  ignoreDeclarationSort?: boolean;
  /**
   * When `true`, the rule ignores the sorting of import members within a single import declaration.
   */
  ignoreMemberSort?: boolean;
  /**
   * Specifies the sort order of different import syntaxes.
   * Must include all 4 kinds!
   */
  memberSyntaxSortOrder?: ImportKind[];
}
export interface SortKeysOptions {
  /**
   * When true, groups of properties separated by a blank line are sorted independently.
   */
  allowLineSeparatedGroups?: boolean;
  /**
   * Whether the sort comparison is case-sensitive (A < a when true).
   */
  caseSensitive?: boolean;
  /**
   * Minimum number of properties required in an object before sorting is enforced.
   */
  minKeys?: number;
  /**
   * Use natural sort order so that, for example, "a2" comes before "a10".
   */
  natural?: boolean;
}
export interface SortVars {
  /**
   * When `true`, the rule ignores case-sensitivity when sorting variables.
   */
  ignoreCase?: boolean;
}
export interface ArrayTypeConfig {
  /**
   * The array type expected for mutable cases.
   */
  default?: ArrayOption;
  /**
   * The array type expected for readonly cases. If omitted, the value for `default` will be used.
   */
  readonly?: ReadonlyArrayOption;
}
/**
 * This rule allows you to specify how different TypeScript directive comments
 * should be handled.
 *
 * For each directive (`@ts-expect-error`, `@ts-ignore`, `@ts-nocheck`, `@ts-check`), you can choose one of the following options:
 * - `true`: Disallow the directive entirely, preventing its use in the entire codebase.
 * - `false`: Allow the directive without any restrictions.
 * - `"allow-with-description"`: Allow the directive only if it is followed by a description explaining its use. The description must meet the minimum length specified by `minimumDescriptionLength`.
 * - `{ "descriptionFormat": "<regex>" }`: Allow the directive only if the description matches the specified regex pattern.
 *
 * For example:
 * ```json
 * {
 *   "ts-expect-error": "allow-with-description",
 *   "ts-ignore": true,
 *   "ts-nocheck": {
 *     "descriptionFormat": "^: TS\\d+ because .+$"
 *   },
 *   "ts-check": false,
 *   "minimumDescriptionLength": 3
 * }
 * ```
 */
export interface BanTsCommentConfig {
  /**
   * Minimum description length required when using directives with `allow-with-description`.
   */
  minimumDescriptionLength?: number;
  /**
   * How to handle the `@ts-check` directive.
   */
  "ts-check"?: DirectiveConfigSchema;
  /**
   * How to handle the `@ts-expect-error` directive.
   */
  "ts-expect-error"?: DirectiveConfigSchema;
  /**
   * How to handle the `@ts-ignore` directive.
   */
  "ts-ignore"?: DirectiveConfigSchema;
  /**
   * How to handle the `@ts-nocheck` directive.
   */
  "ts-nocheck"?: DirectiveConfigSchema;
}
export interface ConsistentReturnConfig {
  /**
   * Treat explicit `return undefined` as equivalent to an unspecified return.
   */
  treatUndefinedAsUnspecified?: boolean;
}
export interface ConsistentTypeAssertionsConfig {
  /**
   * Whether array literal type assertions are allowed, allowed only as parameters, or disallowed.
   */
  arrayLiteralTypeAssertions?: ArrayLiteralTypeAssertions;
  /**
   * Which assertion syntax is enforced.
   */
  assertionStyle?: AssertionStyle;
  /**
   * Whether object literal type assertions are allowed, allowed only as parameters, or disallowed.
   */
  objectLiteralTypeAssertions?: ObjectLiteralTypeAssertions;
}
export interface ConsistentTypeExportsConfig {
  /**
   * Enables an autofix strategy that rewrites mixed exports using inline `type` specifiers.
   */
  fixMixedExportsWithInlineTypeSpecifier?: boolean;
}
export interface ConsistentTypeImportsConfig {
  /**
   * Disallow using `import()` in type annotations, like `type T = import('foo')`
   */
  disallowTypeAnnotations?: boolean;
  /**
   * Control how type imports are added when auto-fixing.
   */
  fixStyle?: FixStyle;
  /**
   * Control whether to enforce type imports or value imports.
   */
  prefer?: Prefer;
}
export interface DotNotationConfig {
  /**
   * Allow bracket notation for properties covered by an index signature.
   */
  allowIndexSignaturePropertyAccess?: boolean;
  /**
   * Allow bracket notation for ES3 keyword property names (for example `obj["class"]`).
   */
  allowKeywords?: boolean;
  /**
   * Regex pattern for property names that are allowed to use bracket notation.
   */
  allowPattern?: string;
  /**
   * Allow bracket notation for private class members.
   */
  allowPrivateClassPropertyAccess?: boolean;
  /**
   * Allow bracket notation for protected class members.
   */
  allowProtectedClassPropertyAccess?: boolean;
}
export interface ExplicitFunctionReturnTypeConfig {
  /**
   * Whether to allow concise arrow functions that start with the `void` keyword.
   */
  allowConciseArrowFunctionExpressionsStartingWithVoid?: boolean;
  /**
   * Whether to allow arrow functions that use `as const` assertion on their return value.
   */
  allowDirectConstAssertionInArrowFunctions?: boolean;
  /**
   * Whether to allow expressions as function return types. When `true`, allows functions that immediately return an expression without a return type annotation.
   */
  allowExpressions?: boolean;
  /**
   * Whether to allow functions that do not have generic type parameters.
   */
  allowFunctionsWithoutTypeParameters?: boolean;
  /**
   * Whether to allow higher-order functions (functions that return another function) without return type annotations.
   */
  allowHigherOrderFunctions?: boolean;
  /**
   * Whether to allow immediately invoked function expressions (IIFEs) without return type annotations.
   */
  allowIIFEs?: boolean;
  /**
   * Whether to allow typed function expressions. When `true`, allows function expressions that are assigned to a typed variable or parameter.
   */
  allowTypedFunctionExpressions?: boolean;
  /**
   * Array of function names that are exempt from requiring return type annotations.
   */
  allowedNames?: string[];
}
export interface ExplicitMemberAccessibilityConfig {
  /**
   * Which accessibility modifier is required to exist or not exist.
   */
  accessibility?: AccessibilityLevel;
  /**
   * Specific method names that may be ignored.
   */
  ignoredMethodNames?: string[];
  /**
   * Changes to required accessibility modifiers for specific kinds of class members.
   */
  overrides?: AccessibilityOverrides;
}
export interface AccessibilityOverrides {
  /**
   * Which member accessibility modifier requirements to apply for accessors (getters/setters).
   */
  accessors?: AccessibilityLevel;
  /**
   * Which member accessibility modifier requirements to apply for constructors.
   */
  constructors?: AccessibilityLevel;
  /**
   * Which member accessibility modifier requirements to apply for methods.
   */
  methods?: AccessibilityLevel;
  /**
   * Which member accessibility modifier requirements to apply for parameter properties.
   */
  parameterProperties?: AccessibilityLevel;
  /**
   * Which member accessibility modifier requirements to apply for properties.
   */
  properties?: AccessibilityLevel;
}
export interface ExplicitModuleBoundaryTypesConfig {
  /**
   * Whether to ignore arguments that are explicitly typed as `any`.
   */
  allowArgumentsExplicitlyTypedAsAny?: boolean;
  /**
   * Whether to ignore return type annotations on body-less arrow functions
   * that return an `as const` type assertion. You must still type the
   * parameters of the function.
   */
  allowDirectConstAssertionInArrowFunctions?: boolean;
  /**
   * Whether to ignore return type annotations on functions immediately
   * returning another function expression. You must still type the
   * parameters of the function.
   */
  allowHigherOrderFunctions?: boolean;
  /**
   * Whether to ignore return type annotations on functions with overload
   * signatures.
   */
  allowOverloadFunctions?: boolean;
  /**
   * Whether to ignore type annotations on the variable of a function
   * expression.
   */
  allowTypedFunctionExpressions?: boolean;
  /**
   * An array of function/method names that will not have their arguments or
   * return values checked.
   */
  allowedNames?: string[];
}
export interface NoBaseToStringConfig {
  /**
   * Whether to also check values of type `unknown`.
   * When `true`, calling toString on `unknown` values will be flagged.
   * Default is `false`.
   */
  checkUnknown?: boolean;
  /**
   * A list of type names to ignore when checking for unsafe toString usage.
   * These types are considered safe to call toString on even if they don't
   * provide a custom implementation.
   */
  ignoredTypeNames?: string[];
}
export interface NoConfusingVoidExpressionConfig {
  /**
   * Whether to ignore arrow function shorthand that returns void.
   * When true, allows expressions like `() => someVoidFunction()`.
   */
  ignoreArrowShorthand?: boolean;
  /**
   * Whether to ignore expressions using the void operator.
   * When true, allows `void someExpression`.
   */
  ignoreVoidOperator?: boolean;
  /**
   * Whether to ignore calling functions that are declared to return void.
   * When true, allows expressions like `x = voidReturningFunction()`.
   */
  ignoreVoidReturningFunctions?: boolean;
}
export interface NoDeprecatedConfig {
  /**
   * An array of type or value specifiers that are allowed to be used even if deprecated.
   * Use this to allow specific deprecated APIs that you intentionally want to continue using.
   */
  allow?: TypeOrValueSpecifier[];
}
/**
 * Describes specific types or values declared in local files.
 */
export interface FileSpecifier {
  /**
   * Must be "file"
   */
  from: FileFrom;
  /**
   * The name(s) of the type or value to match
   */
  name: NameSpecifier;
  /**
   * Optional file path to specify where the types or values must be declared.
   * If omitted, all files will be matched.
   */
  path?: string;
}
/**
 * Describes specific types or values declared in TypeScript's built-in lib.*.d.ts types.
 */
export interface LibSpecifier {
  /**
   * Must be "lib"
   */
  from: LibFrom;
  /**
   * The name(s) of the lib type or value to match
   */
  name: NameSpecifier;
}
/**
 * Describes specific types or values imported from packages.
 */
export interface PackageSpecifier {
  /**
   * Must be "package"
   */
  from: PackageFrom;
  /**
   * The name(s) of the type or value to match
   */
  name: NameSpecifier;
  /**
   * The package name to match
   */
  package: string;
}
export interface NoDuplicateTypeConstituentsConfig {
  /**
   * Whether to ignore duplicate types in intersection types.
   * When true, allows `type T = A & A`.
   */
  ignoreIntersections?: boolean;
  /**
   * Whether to ignore duplicate types in union types.
   * When true, allows `type T = A | A`.
   */
  ignoreUnions?: boolean;
}
export interface NoEmptyInterface {
  /**
   * When set to `true`, allows empty interfaces that extend a single interface.
   */
  allowSingleExtends?: boolean;
}
export interface NoEmptyObjectTypeConfig {
  /**
   * Whether to allow empty interfaces.
   */
  allowInterfaces?: AllowInterfaces;
  /**
   * Whether to allow empty object type literals.
   */
  allowObjectTypes?: AllowObjectTypes;
  /**
   * A stringified regular expression to allow interfaces and object type aliases with the configured name.
   *
   * This can be useful if your existing code style includes a pattern of declaring empty types with `{}` instead of `object`.
   *
   * Example of **incorrect** code for this rule with `{ allowWithName: 'Props$' }`:
   * ```ts
   * interface InterfaceValue {}
   * type TypeValue = {};
   * ```
   *
   * Example of **correct** code for this rule with `{ allowWithName: 'Props$' }`:
   * ```ts
   * interface InterfaceProps {}
   * type TypeProps = {};
   * ```
   */
  allowWithName?: string;
}
export interface NoExplicitAny {
  /**
   * Whether to enable auto-fixing in which the `any` type is converted to the `unknown` type.
   */
  fixToUnknown?: boolean;
  /**
   * Whether to ignore rest parameter arrays.
   */
  ignoreRestArgs?: boolean;
}
export interface NoExtraneousClass {
  /**
   * Allow classes that only have a constructor.
   */
  allowConstructorOnly?: boolean;
  /**
   * Allow empty classes.
   */
  allowEmpty?: boolean;
  /**
   * Allow classes with only static members.
   */
  allowStaticOnly?: boolean;
  /**
   * Allow classes with decorators.
   */
  allowWithDecorator?: boolean;
}
export interface NoFloatingPromisesConfig {
  /**
   * Allows specific calls to be ignored, specified as type or value specifiers.
   */
  allowForKnownSafeCalls?: TypeOrValueSpecifier[];
  /**
   * Allows specific Promise types to be ignored, specified as type or value specifiers.
   */
  allowForKnownSafePromises?: TypeOrValueSpecifier[];
  /**
   * Check for thenable objects that are not necessarily Promises.
   */
  checkThenables?: boolean;
  /**
   * Ignore immediately invoked function expressions (IIFEs).
   */
  ignoreIIFE?: boolean;
  /**
   * Ignore Promises that are void expressions.
   */
  ignoreVoid?: boolean;
}
export interface NoInferrableTypes {
  /**
   * When set to `true`, ignores type annotations on function parameters.
   */
  ignoreParameters?: boolean;
  /**
   * When set to `true`, ignores type annotations on class properties.
   */
  ignoreProperties?: boolean;
}
export interface NoInvalidVoidTypeConfig {
  /**
   * Whether a `this` parameter of a function may be `void`.
   */
  allowAsThisParameter?: boolean;
  /**
   * Whether `void` can be used as generic type arguments.
   * Can be `true` / `false`, or an allowlist of generic type names.
   */
  allowInGenericTypeArguments?: AllowInGenericTypeArguments;
}
export interface NoMeaninglessVoidOperatorConfig {
  /**
   * Whether to check `void` applied to expressions of type `never`.
   */
  checkNever?: boolean;
}
export interface NoMisusedPromisesConfig {
  /**
   * Whether to check if Promises are used in conditionals.
   * When true, disallows using Promises in conditions where a boolean is expected.
   */
  checksConditionals?: boolean;
  /**
   * Whether to check if Promises are used in spread syntax.
   * When true, disallows spreading Promise values.
   */
  checksSpreads?: boolean;
  /**
   * Configuration for checking if Promises are returned in contexts expecting void.
   * Can be a boolean to enable/disable all checks, or an object for granular control.
   */
  checksVoidReturn?: ChecksVoidReturn;
}
export interface ChecksVoidReturnOptions {
  /**
   * Whether to check Promise-returning functions passed as arguments to void-returning functions.
   */
  arguments?: boolean;
  /**
   * Whether to check Promise-returning functions in JSX attributes expecting void.
   */
  attributes?: boolean;
  /**
   * Whether to check Promise-returning methods that override void-returning inherited methods.
   */
  inheritedMethods?: boolean;
  /**
   * Whether to check Promise-returning functions assigned to object properties expecting void.
   */
  properties?: boolean;
  /**
   * Whether to check Promise values returned from void-returning functions.
   */
  returns?: boolean;
  /**
   * Whether to check Promise-returning functions assigned to variables typed as void-returning.
   */
  variables?: boolean;
}
export interface NoMisusedSpreadConfig {
  /**
   * An array of type or value specifiers that are allowed to be spread
   * even if they would normally be flagged as misused.
   */
  allow?: TypeOrValueSpecifier[];
}
export interface NoNamespace {
  /**
   * Whether to allow declare with custom TypeScript namespaces.
   *
   * Examples of **incorrect** code for this rule when `{ "allowDeclarations": true }`
   * ```typescript
   * module foo {}
   * namespace foo {}
   * ```
   *
   * Examples of **correct** code for this rule when `{ "allowDeclarations": true }`
   * ```typescript
   * declare module 'foo' {}
   * declare module foo {}
   * declare namespace foo {}
   *
   * declare global {
   * namespace foo {}
   * }
   *
   * declare module foo {
   * namespace foo {}
   * }
   * ```
   *
   * Examples of **incorrect** code for this rule when `{ "allowDeclarations": false }`
   * ```typescript
   * module foo {}
   * namespace foo {}
   * declare module foo {}
   * declare namespace foo {}
   * ```
   *
   * Examples of **correct** code for this rule when `{ "allowDeclarations": false }`
   * ```typescript
   * declare module 'foo' {}
   * ```
   */
  allowDeclarations?: boolean;
  /**
   * Examples of **incorrect** code for this rule when `{ "allowDefinitionFiles": true }`
   * ```typescript
   * // if outside a d.ts file
   * module foo {}
   * namespace foo {}
   *
   * // if outside a d.ts file
   * module foo {}
   * namespace foo {}
   * declare module foo {}
   * declare namespace foo {}
   * ```
   *
   * Examples of **correct** code for this rule when `{ "allowDefinitionFiles": true }`
   * ```typescript
   * declare module 'foo' {}
   * // anything inside a d.ts file
   * ```
   */
  allowDefinitionFiles?: boolean;
}
export interface NoRequireImportsConfig {
  /**
   * These strings will be compiled into regular expressions with the u flag and be used to test against the imported path.
   * A common use case is to allow importing `package.json`. This is because `package.json` commonly lives outside of the TS root directory,
   * so statically importing it would lead to root directory conflicts, especially with `resolveJsonModule` enabled.
   * You can also use it to allow importing any JSON if your environment doesn't support JSON modules, or use it for other cases where `import` statements cannot work.
   *
   * With `{ allow: ['/package\\.json$'] }`:
   *
   * Examples of **correct** code for this rule:
   * ```ts
   * console.log(require('../package.json').version);
   * ```
   */
  allow?: string[];
  /**
   * When set to `true`, `import ... = require(...)` declarations won't be reported.
   * This is useful if you use certain module options that require strict CommonJS interop semantics.
   *
   * When set to `true`:
   *
   * Examples of **incorrect** code for this rule:
   * ```ts
   * var foo = require('foo');
   * const foo = require('foo');
   * let foo = require('foo');
   * ```
   * Examples of **correct** code for this rule:
   * ```ts
   * import foo = require('foo');
   * import foo from 'foo';
   * ```
   */
  allowAsImport?: boolean;
}
export interface NoRestrictedTypesConfig {
  /**
   * A mapping of type names to ban configurations.
   */
  types?: {
    [k: string]: BanConfigValue | undefined;
  };
}
export interface NoThisAliasConfig {
  /**
   * Whether to allow destructuring of `this` to local variables.
   */
  allowDestructuring?: boolean;
  /**
   * An array of variable names that are allowed to alias `this`.
   */
  allowedNames?: string[];
}
export interface NoUnnecessaryBooleanLiteralCompareConfig {
  /**
   * Whether to allow comparing nullable boolean expressions to `false`.
   * When false, `x === false` where x is `boolean | null` will be flagged.
   */
  allowComparingNullableBooleansToFalse?: boolean;
  /**
   * Whether to allow comparing nullable boolean expressions to `true`.
   * When false, `x === true` where x is `boolean | null` will be flagged.
   */
  allowComparingNullableBooleansToTrue?: boolean;
}
export interface NoUnnecessaryConditionConfig {
  /**
   * Whether to allow constant loop conditions.
   * `true` is treated as `"always"`, `false` as `"never"`.
   */
  allowConstantLoopConditions?: AllowConstantLoopConditions;
  /**
   * Whether to check type predicate functions.
   */
  checkTypePredicates?: boolean;
}
export interface NoUnnecessaryTypeAssertionConfig {
  /**
   * Whether to check literal const assertions like `'foo' as const`.
   * When `false` (default), const assertions on literal types are not flagged.
   * When `true`, these will be reported as unnecessary since the type is already a literal.
   */
  checkLiteralConstAssertions?: boolean;
  /**
   * A list of type names to ignore when checking for unnecessary assertions.
   * Type assertions to these types will not be flagged even if they appear unnecessary.
   * Example: `["Foo", "Bar"]` to allow `x as Foo` or `x as Bar`.
   */
  typesToIgnore?: string[];
}
export interface NoUnsafeMemberAccessConfig {
  /**
   * Whether to allow `?.` optional chains on `any` values.
   * When `true`, optional chaining on `any` values will not be flagged.
   * Default is `false`.
   */
  allowOptionalChaining?: boolean;
}
export interface OnlyThrowErrorConfig {
  /**
   * An array of type or value specifiers for additional types that are allowed to be thrown.
   * Use this to allow throwing custom error types.
   */
  allow?: TypeOrValueSpecifier[];
  /**
   * Whether to allow rethrowing caught values that are not Error objects.
   */
  allowRethrowing?: boolean;
  /**
   * Whether to allow throwing values typed as `any`.
   */
  allowThrowingAny?: boolean;
  /**
   * Whether to allow throwing values typed as `unknown`.
   */
  allowThrowingUnknown?: boolean;
}
export interface ParameterPropertiesConfig {
  /**
   * Modifiers that are allowed to be used with parameter properties or class properties, depending on the `prefer` option.
   */
  allow?: Modifier[];
  /**
   * Whether to prefer parameter properties or class properties.
   */
  prefer?: Prefer2;
}
export interface PreferLiteralEnumMember {
  /**
   * When set to `true`, allows bitwise expressions in enum member initializers.
   * This includes bitwise NOT (`~`), AND (`&`), OR (`|`), XOR (`^`), and shift operators (`<<`, `>>`, `>>>`).
   */
  allowBitwiseExpressions?: boolean;
}
export interface PreferNullishCoalescingConfig {
  /**
   * Whether to ignore arguments to the `Boolean` constructor.
   */
  ignoreBooleanCoercion?: boolean;
  /**
   * Whether to ignore cases that are located within a conditional test.
   */
  ignoreConditionalTests?: boolean;
  /**
   * Whether to ignore any if statements that could be simplified by using
   * the nullish coalescing operator.
   */
  ignoreIfStatements?: boolean;
  /**
   * Whether to ignore any logical or expressions that are part of a mixed
   * logical expression (with `&&`).
   */
  ignoreMixedLogicalExpressions?: boolean;
  /**
   * Whether to ignore all (`true`) or some (an object with properties) primitive types.
   */
  ignorePrimitives?: IgnorePrimitives;
  /**
   * Whether to ignore any ternary expressions that could be simplified by
   * using the nullish coalescing operator.
   */
  ignoreTernaryTests?: boolean;
}
/**
 * Options for ignoring specific primitive types.
 */
export interface IgnorePrimitivesOptions {
  /**
   * Ignore bigint primitive types.
   */
  bigint?: boolean;
  /**
   * Ignore boolean primitive types.
   */
  boolean?: boolean;
  /**
   * Ignore number primitive types.
   */
  number?: boolean;
  /**
   * Ignore string primitive types.
   */
  string?: boolean;
}
export interface PreferOptionalChainConfig {
  /**
   * Allow autofixers that will change the return type of the expression.
   * This option is considered unsafe as it may break the build.
   */
  allowPotentiallyUnsafeFixesThatModifyTheReturnTypeIKnowWhatImDoing?: boolean;
  /**
   * Check operands that are typed as `any` when inspecting "loose boolean" operands.
   */
  checkAny?: boolean;
  /**
   * Check operands that are typed as `bigint` when inspecting "loose boolean" operands.
   */
  checkBigInt?: boolean;
  /**
   * Check operands that are typed as `boolean` when inspecting "loose boolean" operands.
   */
  checkBoolean?: boolean;
  /**
   * Check operands that are typed as `number` when inspecting "loose boolean" operands.
   */
  checkNumber?: boolean;
  /**
   * Check operands that are typed as `string` when inspecting "loose boolean" operands.
   */
  checkString?: boolean;
  /**
   * Check operands that are typed as `unknown` when inspecting "loose boolean" operands.
   */
  checkUnknown?: boolean;
  /**
   * Skip operands that are not typed with `null` and/or `undefined` when inspecting
   * "loose boolean" operands.
   */
  requireNullish?: boolean;
}
export interface PreferPromiseRejectErrorsConfig {
  /**
   * An array of type or value specifiers for additional types that are allowed
   * as Promise rejection reasons.
   */
  allow?: TypeOrValueSpecifier[];
  /**
   * Whether to allow calling `Promise.reject()` with no arguments.
   */
  allowEmptyReject?: boolean;
  /**
   * Whether to allow rejecting Promises with values typed as `any`.
   */
  allowThrowingAny?: boolean;
  /**
   * Whether to allow rejecting Promises with values typed as `unknown`.
   */
  allowThrowingUnknown?: boolean;
}
export interface PreferReadonlyConfig {
  /**
   * Restrict checks to members immediately initialized with inline lambda values.
   */
  onlyInlineLambdas?: boolean;
}
export interface PreferReadonlyParameterTypesConfig {
  /**
   * Type/value specifiers that should be exempt from this rule.
   */
  allow?: TypeOrValueSpecifier[];
  /**
   * Whether to check constructor parameter properties.
   */
  checkParameterProperties?: boolean;
  /**
   * Whether to ignore parameters without explicit type annotations.
   */
  ignoreInferredTypes?: boolean;
  /**
   * Whether mutable methods should be treated as readonly members.
   */
  treatMethodsAsReadonly?: boolean;
}
export interface PreferStringStartsEndsWithConfig {
  /**
   * Whether equality checks against the first/last character are allowed.
   */
  allowSingleElementEquality?: AllowSingleElementEquality;
}
export interface PromiseFunctionAsyncConfig {
  /**
   * Whether to allow functions returning `any` type without requiring `async`.
   */
  allowAny?: boolean;
  /**
   * A list of Promise type names that are allowed without requiring `async`.
   * Example: `["SpecialPromise"]` to allow functions returning `SpecialPromise` without `async`.
   */
  allowedPromiseNames?: string[];
  /**
   * Whether to check arrow functions for missing `async` keyword.
   */
  checkArrowFunctions?: boolean;
  /**
   * Whether to check function declarations for missing `async` keyword.
   */
  checkFunctionDeclarations?: boolean;
  /**
   * Whether to check function expressions for missing `async` keyword.
   */
  checkFunctionExpressions?: boolean;
  /**
   * Whether to check method declarations for missing `async` keyword.
   */
  checkMethodDeclarations?: boolean;
}
export interface RequireArraySortCompareConfig {
  /**
   * Whether to ignore arrays in which all elements are strings.
   */
  ignoreStringArrays?: boolean;
}
export interface RestrictPlusOperandsConfig {
  /**
   * Whether to allow `any` type in plus operations.
   */
  allowAny?: boolean;
  /**
   * Whether to allow `boolean` types in plus operations.
   */
  allowBoolean?: boolean;
  /**
   * Whether to allow nullish types (`null` or `undefined`) in plus operations.
   */
  allowNullish?: boolean;
  /**
   * Whether to allow mixed number and string operands in plus operations.
   */
  allowNumberAndString?: boolean;
  /**
   * Whether to allow `RegExp` types in plus operations.
   */
  allowRegExp?: boolean;
  /**
   * Whether to skip compound assignments (e.g., `a += b`).
   */
  skipCompoundAssignments?: boolean;
}
export interface RestrictTemplateExpressionsConfig {
  /**
   * An array of type or value specifiers for additional types that are allowed in template expressions.
   * Defaults include Error, URL, and URLSearchParams from lib.
   */
  allow?: TypeOrValueSpecifier[];
  /**
   * Whether to allow `any` typed values in template expressions.
   */
  allowAny?: boolean;
  /**
   * Whether to allow array types in template expressions.
   */
  allowArray?: boolean;
  /**
   * Whether to allow boolean types in template expressions.
   */
  allowBoolean?: boolean;
  /**
   * Whether to allow `never` type in template expressions.
   */
  allowNever?: boolean;
  /**
   * Whether to allow nullish types (`null` or `undefined`) in template expressions.
   */
  allowNullish?: boolean;
  /**
   * Whether to allow number and bigint types in template expressions.
   */
  allowNumber?: boolean;
  /**
   * Whether to allow RegExp values in template expressions.
   */
  allowRegExp?: boolean;
}
export interface StrictBooleanExpressionsConfig {
  /**
   * Whether to allow `any` type in boolean contexts.
   */
  allowAny?: boolean;
  /**
   * Whether to allow nullable boolean types (e.g., `boolean | null`) in boolean contexts.
   */
  allowNullableBoolean?: boolean;
  /**
   * Whether to allow nullable enum types in boolean contexts.
   */
  allowNullableEnum?: boolean;
  /**
   * Whether to allow nullable number types (e.g., `number | null`) in boolean contexts.
   */
  allowNullableNumber?: boolean;
  /**
   * Whether to allow nullable object types in boolean contexts.
   */
  allowNullableObject?: boolean;
  /**
   * Whether to allow nullable string types (e.g., `string | null`) in boolean contexts.
   */
  allowNullableString?: boolean;
  /**
   * Whether to allow number types in boolean contexts (checks for non-zero numbers).
   */
  allowNumber?: boolean;
  /**
   * Whether to allow string types in boolean contexts (checks for non-empty strings).
   */
  allowString?: boolean;
}
export interface StrictVoidReturnConfig {
  /**
   * Allow callbacks that return `any` in places that expect a `void` callback.
   */
  allowReturnAny?: boolean;
}
export interface SwitchExhaustivenessCheckConfig {
  /**
   * Whether to allow default cases on switches that are not exhaustive.
   * When false, requires exhaustive switch statements without default cases.
   */
  allowDefaultCaseForExhaustiveSwitch?: boolean;
  /**
   * Whether to consider `default` cases exhaustive for union types.
   * When true, a switch statement with a `default` case is considered exhaustive
   * even if not all union members are handled explicitly.
   */
  considerDefaultExhaustiveForUnions?: boolean;
  /**
   * Regular expression pattern that when matched in a default case comment,
   * will suppress the exhaustiveness check.
   * Example: `"@skip-exhaustive-check"` to allow `default: // @skip-exhaustive-check`
   */
  defaultCaseCommentPattern?: string;
  /**
   * Whether to require default cases on switches over union types that are not exhaustive.
   * When true, switches with non-exhaustive union types must have a default case.
   */
  requireDefaultForNonUnion?: boolean;
}
export interface TripleSlashReferenceConfig {
  /**
   * What to enforce for `/// <reference lib="..." />` references.
   */
  lib?: AlwaysNever;
  /**
   * What to enforce for `/// <reference path="..." />` references.
   */
  path?: PathOption;
  /**
   * What to enforce for `/// <reference types="..." />` references.
   */
  types?: TypesOption;
}
export interface UnboundMethodConfig {
  /**
   * Whether to ignore unbound methods that are static.
   * When true, static methods can be referenced without binding.
   */
  ignoreStatic?: boolean;
}
export interface UnifiedSignaturesOptions {
  /**
   * Whether to ignore parameter name differences when comparing signatures. If `false`, signatures
   * will not be considered unifiable if they have parameters in the same position with different
   * names, even if the parameter types are the same.
   */
  ignoreDifferentlyNamedParameters?: boolean;
  /**
   * Whether to ignore JSDoc differences when comparing signatures. If `false`, signatures will not
   * be considered unifiable if the closest leading block comments for the signatures are different,
   * even if the signatures themselves are identical.
   */
  ignoreOverloadsWithDifferentJSDoc?: boolean;
}
export interface CatchErrorNameConfig {
  /**
   * A list of patterns to ignore when checking `catch` variable names. The pattern
   * can be a string or regular expression.
   */
  ignore?: string[];
  /**
   * The name to use for error variables in `catch` blocks. You can customize it
   * to something other than `'error'` (e.g., `'exception'`).
   */
  name?: string;
}
export interface ConsistentFunctionScoping {
  /**
   * Whether to check scoping with arrow functions.
   */
  checkArrowFunctions?: boolean;
}
export interface ExplicitLengthCheck {
  /**
   * Configuration option to specify how non-zero length checks should be enforced.
   */
  "non-zero"?: NonZero;
}
export interface ImportStyleConfig {
  /**
   * Whether dynamic import expressions are checked.
   *
   * Set this to `false` to skip calls such as `await import("module")`.
   *
   * With the default configuration, examples of **incorrect** code:
   * ```js
   * async () => {
   * const {red} = await import("chalk");
   * };
   * ```
   *
   * Examples of **correct** code:
   * ```js
   * async () => {
   * const {default: chalk} = await import("chalk");
   * };
   * ```
   */
  checkDynamicImport?: boolean;
  /**
   * Whether export-from declarations are checked.
   *
   * This is disabled by default. Set this to `true` to check declarations like
   * `export ... from "module"`.
   *
   * With `{ "checkExportFrom": true }`, examples of **incorrect** code:
   * ```js
   * export * from "node:util";
   * ```
   *
   * Examples of **correct** code:
   * ```js
   * export {promisify} from "node:util";
   * ```
   */
  checkExportFrom?: boolean;
  /**
   * Whether static import declarations are checked.
   *
   * Set this to `false` to skip `import ... from "module"` and side-effect imports like
   * `import "module"`.
   *
   * With the default configuration, examples of **incorrect** code:
   * ```js
   * import {red} from "chalk";
   * ```
   *
   * Examples of **correct** code:
   * ```js
   * import chalk from "chalk";
   * ```
   */
  checkImport?: boolean;
  /**
   * Whether CommonJS `require()` calls are checked.
   *
   * Set this to `false` to skip `require("module")` calls completely.
   *
   * With the default configuration, examples of **incorrect** code:
   * ```js
   * const util = require("node:util");
   * ```
   *
   * Examples of **correct** code:
   * ```js
   * const {promisify} = require("node:util");
   * ```
   */
  checkRequire?: boolean;
  /**
   * Whether `styles` extends or replaces the built-in module preferences.
   *
   * When this is `true`, entries in `styles` are merged with the default preferences. For
   * example, `{ "styles": { "path": { "named": true } } }` allows named imports from
   * `path` while leaving its default import style allowed. When this is `false`, only modules
   * configured in `styles` are checked.
   *
   * With `{ "extendDefaultStyles": false, "styles": {} }`, examples of **correct** code:
   * ```js
   * import {red} from "chalk";
   * ```
   */
  extendDefaultStyles?: boolean;
  /**
   * Per-module import style preferences.
   *
   * Each key is a module specifier. Set the value to `false` to disable checking for the
   * module, or to an object that allows one or more import styles. The available styles are
   * `unassigned`, `default`, `namespace`, and `named`. When `extendDefaultStyles` is `true`,
   * these entries extend the built-in defaults instead of replacing them.
   *
   * The default module preferences are default imports for `chalk`, `path`, and `node:path`,
   * and named imports for `util` and `node:util`.
   *
   * With `{ "styles": { "node:util": { "named": true, "default": false } } }`,
   * examples of **incorrect** code:
   * ```js
   * import util from "node:util";
   * ```
   *
   * Examples of **correct** code:
   * ```js
   * import {promisify} from "node:util";
   * ```
   */
  styles?: {
    [k: string]: ModuleStylesOverride | undefined;
  };
}
export interface MaxNestedCalls {
  /**
   * The maximum allowed nested call depth.
   */
  max?: number;
}
export interface NoArrayReduce {
  /**
   * When set to `true`, allows simple operations (like summing numbers) in `reduce` and `reduceRight` calls.
   */
  allowSimpleOperations?: boolean;
}
export interface NoArrayReverse {
  /**
   * This rule allows `array.reverse()` as an expression statement by default.
   * Set to `false` to forbid `Array#reverse()` even if it's an expression statement.
   *
   * Examples of **incorrect** code for this rule with this option set to `false`:
   * ```js
   * array.reverse();
   * ```
   */
  allowExpressionStatement?: boolean;
}
export interface NoArraySort {
  /**
   * When set to `true`, allows sorting a fresh array created by a spread, e.g. `[...iterable].sort()`.
   * This avoids the double allocation of `toSorted()` when sorting an iterable such as a `Set`.
   *
   * Example of **correct** code for this rule with `allowAfterSpread` set to `true`:
   * ```js
   * const sorted = [...mySet].sort();
   * ```
   */
  allowAfterSpread?: boolean;
  /**
   * When set to `true` (default), allows `array.sort()` as an expression statement.
   * Set to `false` to forbid `Array#sort()` even if it's an expression statement.
   *
   * Example of **incorrect** code for this rule with `allowExpressionStatement` set to `false`:
   * ```js
   * array.sort();
   * ```
   */
  allowExpressionStatement?: boolean;
}
export interface NoInstanceofBuiltinsConfig {
  /**
   * Constructor names to exclude from checking.
   */
  exclude?: string[];
  /**
   * Additional constructor names to check beyond the default set.
   * Use this to extend the rule with additional constructors.
   */
  include?: string[];
  /**
   * Controls which built-in constructors are checked.
   */
  strategy?: NoInstanceofBuiltinsStrategy;
  /**
   * When `true`, checks `instanceof Error` and suggests using `Error.isError()` instead.
   * Requires [the `Error.isError()` function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/isError)
   * to be available.
   */
  useErrorIsError?: boolean;
}
export interface NoNull {
  /**
   * When set to `true`, disallow the use of `null` as a direct function call or constructor argument.
   */
  checkArguments?: boolean;
  /**
   * When set to `true`, the rule will also check strict equality/inequality comparisons (`===` and `!==`) against `null`.
   */
  checkStrictEquality?: boolean;
}
export interface NoTypeofUndefined {
  /**
   * If set to `true`, also report `typeof x === "undefined"` when `x` may be a global
   * variable that is not declared (commonly checked via `typeof foo === "undefined"`).
   */
  checkGlobalVariables?: boolean;
}
export interface NoUselessPromiseResolveRejectOptions {
  /**
   * If set to `true`, allows the use of `Promise.reject` in async functions and promise callbacks.
   */
  allowReject?: boolean;
}
export interface NoUselessUndefined {
  /**
   * Whether to check for useless `undefined` in function call arguments.
   */
  checkArguments?: boolean;
  /**
   * Whether to check for useless `undefined` in arrow function bodies.
   */
  checkArrowFunctionBody?: boolean;
}
export interface NumericSeparatorsStyleConfig {
  /**
   * Configuration for binary literals (e.g. `0b1010_0001` and bigint variants).
   * Controls how digits are grouped and when separators are applied.
   */
  binary?: NumericBaseConfig;
  /**
   * Configuration for hexadecimal literals (e.g. `0xAB_CD`, `0Xab_cd`, and bigint variants).
   * Controls how digits are grouped and when separators are applied.
   */
  hexadecimal?: NumericBaseConfig;
  /**
   * Configuration for decimal numbers (integers, fraction parts, and exponents).
   * Controls how digits are grouped and when separators are applied.
   */
  number?: NumericNumberConfig;
  /**
   * Configuration for octal literals (e.g. `0o1234_5670` and bigint variants).
   * Controls how digits are grouped and when separators are applied.
   */
  octal?: NumericBaseConfig;
  /**
   * Only enforce the rule when the numeric literal already contains a separator (`_`).
   *
   * When `true`, numbers without separators are left as-is; when `false` (default),
   * grouping will be enforced for eligible numbers even if they don't include separators yet.
   */
  onlyIfContainsSeparator?: boolean;
}
export interface NumericBaseConfig {
  /**
   * The number of digits per group when inserting numeric separators.
   * For example, a `groupLength` of 3 formats `1234567` as `1_234_567`.
   */
  groupLength?: number;
  /**
   * The minimum number of digits required before grouping is applied.
   * Values with fewer digits than this threshold will not be grouped.
   */
  minimumDigits?: number;
  /**
   * Only enforce the rule when the numeric literal already contains a separator (`_`).
   *
   * When `true`, numbers without separators are left as-is; when `false` (default),
   * grouping will be enforced for eligible numbers even if they don't include separators yet.
   */
  onlyIfContainsSeparator?: boolean;
}
export interface NumericNumberConfig {
  /**
   * The size a group of digits in the fractional part (after the decimal point) should be.
   */
  fractionGroupLength?: number;
  /**
   * The number of digits per group when inserting numeric separators.
   * For example, a `groupLength` of 3 formats `1234567` as `1_234_567`.
   */
  groupLength?: number;
  /**
   * The minimum number of digits required before grouping is applied.
   * Values with fewer digits than this threshold will not be grouped.
   */
  minimumDigits?: number;
  /**
   * Only enforce the rule when the numeric literal already contains a separator (`_`).
   *
   * When `true`, numbers without separators are left as-is; when `false` (default),
   * grouping will be enforced for eligible numbers even if they don't include separators yet.
   */
  onlyIfContainsSeparator?: boolean;
}
export interface PreferAtConfig {
  /**
   * Check all index access, not just special patterns like `array.length - 1`.
   * When enabled, `array[0]`, `array[1]`, etc. will also be flagged.
   */
  checkAllIndexAccess?: boolean;
  /**
   * List of function names to treat as "get last element" functions.
   * These functions will be checked for `.at(-1)` usage.
   */
  getLastElementFunctions?: string[];
}
export interface PreferExportFrom {
  /**
   * When false, if any import binding is used somewhere other than a re-export, all variables in the import declaration are ignored.
   */
  checkUsedVariables?: boolean;
}
export interface PreferNumberPropertiesConfig {
  /**
   * If set to `true`, checks for usage of `Infinity` and `-Infinity` as global variables.
   */
  checkInfinity?: boolean;
  /**
   * If set to `true`, checks for usage of `NaN` as a global variable.
   */
  checkNaN?: boolean;
}
export interface PreferObjectFromEntriesConfig {
  /**
   * Additional functions to treat as equivalents to `Object.fromEntries`.
   */
  functions?: string[];
}
export interface PreferSingleCallConfig {
  /**
   * Methods to ignore.
   */
  ignore?: string[];
}
export interface PreferStructuredCloneConfig {
  /**
   * List of functions that are allowed to be used for deep cloning instead of structuredClone.
   */
  functions?: string[];
}
export interface TextEncodingIdentifierCase {
  /**
   * If `true`, prefer `utf-8` over `utf8`.
   */
  withDash?: boolean;
}
export interface UseIsnan {
  /**
   * Whether to disallow NaN as arguments of `indexOf` and `lastIndexOf`
   */
  enforceForIndexOf?: boolean;
  /**
   * Whether to disallow NaN in switch cases and discriminants
   */
  enforceForSwitchCase?: boolean;
}
export interface ValidTypeof {
  /**
   * The `requireStringLiterals` option when set to `true`, allows the comparison of `typeof`
   * expressions with only string literals or other `typeof` expressions, and disallows
   * comparisons to any other value. Default is `false`.
   *
   * With `requireStringLiterals` set to `true`, the following are examples of **incorrect** code:
   * ```js
   * typeof foo === undefined
   * typeof bar == Object
   * typeof baz === "strnig"
   * typeof qux === "some invalid type"
   * typeof baz === anotherVariable
   * typeof foo == 5
   * ```
   *
   * With `requireStringLiterals` set to `true`, the following are examples of **correct** code:
   * ```js
   * typeof foo === "undefined"
   * typeof bar == "object"
   * typeof baz === "string"
   * typeof bar === typeof qux
   * ```
   */
  requireStringLiterals?: boolean;
}
export interface ConsistentEachForJson {
  /**
   * Preferred method to create parameterized tests for `describe` blocks.
   */
  describe?: MemberNames;
  /**
   * Preferred method to create parameterized tests for `it` blocks.
   */
  it?: MemberNames;
  /**
   * Preferred method to create parameterized tests for `suite` blocks.
   */
  suite?: MemberNames;
  /**
   * Preferred method to create parameterized tests for `test` blocks.
   */
  test?: MemberNames;
}
export interface ConsistentTestFilenameConfig {
  /**
   * Regex pattern to ensure we are linting only test filenames.
   * Decides whether a file is a testing file.
   */
  allTestPattern?: string;
  /**
   * Required regex to check if a test filename have a valid formart.
   * Pattern doesn't have a default value, you must provide one.
   */
  pattern?: string;
}
export interface ConsistentVitestConfig {
  /**
   * Decides whether to prefer vitest function accessor
   */
  fn?: VitestFnName;
}
export interface PreferImportInMockConfig {
  /**
   * Whether the rule should generate fixes or not.
   */
  fixable: boolean;
}
export interface RequireMockTypeParametersConfig {
  /**
   * Also require type parameters for `importActual` and `importMock`.
   */
  checkImportFunctions?: boolean;
}
export interface DefinePropsDestructuring {
  /**
   * Require or prohibit destructuring.
   */
  destructure?: Destructure;
}
export interface MaxProps {
  /**
   * The maximum number of props allowed in a Vue SFC.
   */
  maxProps?: number;
}
export interface NoAsyncInComputedPropertiesConfig {
  /**
   * Names of identifiers whose member-call chains (`.then` / `.catch` / `.finally`)
   * should be ignored. Useful for libraries like Zod where `.catch(default)` is
   * a builder API, not a Promise method.
   */
  ignoredObjectNames?: string[];
}
export interface NoDeprecatedModelDefinitionConfig {
  /**
   * Allow `model: { prop: 'modelValue', event: 'update:modelValue' }` (or
   * the kebab-case `model-value` variant) which is forwards-compatible with
   * Vue 3's `v-model`.
   */
  allowVue3Compat?: boolean;
}
export interface NoDupeKeysConfig {
  /**
   * Additional group names to search for duplicate keys in, on top of the
   * built-in `props`, `computed`, `data`, `methods` and `setup` groups.
   */
  groups?: string[];
}
export interface NoReservedComponentNames {
  /**
   * Disallow Vue 3 built-in component names (e.g. `Teleport`, `Suspense`).
   * Note: this also catches Vue 2 built-ins because Vue 3's set includes them.
   */
  disallowVue3BuiltInComponents?: boolean;
  /**
   * Disallow Vue 2 built-in component names (e.g. `Transition`, `KeepAlive`).
   */
  disallowVueBuiltInComponents?: boolean;
  /**
   * Match HTML / SVG element names case-sensitively. When `false` (default),
   * the capitalized form of an HTML element (e.g. `Div`) is also reported.
   */
  htmlElementCaseSensitive?: boolean;
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
export interface ReturnInComputedPropertyConfig {
  /**
   * When `true` (default), `return;` (without a value) is treated as a missing return.
   * Set to `false` to allow bare `return;` as if it returned a value.
   */
  treatUndefinedAsUnspecified?: boolean;
}
export interface YodaOptions {
  /**
   * If the `"exceptRange"` property is `true`, the rule *allows* yoda conditions
   * in range comparisons which are wrapped directly in parentheses, including the
   * parentheses of an `if` or `while` condition.
   * A *range* comparison tests whether a variable is inside or outside the range
   * between two literal values.
   */
  exceptRange?: boolean;
  /**
   * If the `"onlyEquality"` property is `true`, the rule reports yoda
   * conditions *only* for the equality operators `==` and `===`. The `onlyEquality`
   * option allows a superset of the exceptions which `exceptRange` allows, thus
   * both options are not useful together.
   */
  onlyEquality?: boolean;
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
  [k: string]: unknown | undefined;
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
  [k: string]: unknown | undefined;
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
    [k: string]: TagNamePreference | undefined;
  };
  [k: string]: unknown | undefined;
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
    [k: string]: string[] | undefined;
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
    [k: string]: string | undefined;
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
  [k: string]: unknown | undefined;
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
  [k: string]: unknown | undefined;
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
  [k: string]: unknown | undefined;
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
  [k: string]: unknown | undefined;
}
