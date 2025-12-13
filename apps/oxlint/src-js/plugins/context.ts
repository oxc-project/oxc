/*
 * Context objects for rules.
 *
 * Context objects are in 2 layers:
 * 1. Context object for each rule.
 * 2. File context object, shared across all rules.
 *
 * This mirrors ESLint's `RuleContext` and `FileContext` types (with `RuleContext` inheriting from `FileContext`).
 * Some ESLint plugins rely on this 2-layer structure. https://github.com/oxc-project/oxc/issues/15325
 *
 * The difference is that we don't create new file context and rule context objects for each file, but instead reuse
 * the same objects over and over. After plugin loading is complete, no further `Context` objects are created.
 * This reduces pressure on garbage collector, and is required to support `createOnce` API.
 *
 * ## Rule context
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` and `createOnce` functions.
 * `Context` objects are created during plugin loading for each rule.
 * For each file, the same `Context` object is reused over and over.
 *
 * ## File context
 *
 * All `Context` objects have `FILE_CONTEXT` as their prototype, which provides getters for file-specific properties.
 * `FILE_CONTEXT` is a singleton object, shared across all rules.
 * `FILE_CONTEXT` contains no state, only getters which return other singletons (`SOURCE_CODE`),
 * and global variables (`filePath`, `settings`, `cwd`).
 */

import { ast, initAst, SOURCE_CODE } from "./source_code.ts";
import { report } from "./report.ts";
import { settings, initSettings } from "./settings.ts";
import visitorKeys from "../generated/keys.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";
import { EMPTY_GLOBALS, Globals, globals, initGlobals } from "./globals.ts";

import type { RuleDetails } from "./load.ts";
import type { Options } from "./options.ts";
import type { Diagnostic } from "./report.ts";
import type { Settings } from "./settings.ts";
import type { SourceCode } from "./source_code.ts";
import type { ModuleKind, Program } from "../generated/types.d.ts";

const { freeze, assign: ObjectAssign, create: ObjectCreate } = Object;

// Cached current working directory
let cwd: string | null = null;

// Absolute path of file being linted.
// When `null`, indicates that no file is currently being linted (in `createOnce`, or between linting files).
export let filePath: string | null = null;

/**
 * Set up context for linting a file.
 * @param filePathInput - Absolute path of file being linted
 */
export function setupFileContext(filePathInput: string): void {
  filePath = filePathInput;
}

/**
 * Reset file context.
 *
 * This disables all getters on `Context` objects, and `FILE_CONTEXT`.
 * Only way user could trigger a getter if this wasn't done is to store a `Context` object, and then access one of its
 * properties in next tick, in between linting files (highly unlikely). But it's cheap to do, so we cover this odd case.
 */
export function resetFileContext(): void {
  filePath = null;
}

// ECMAScript version. This matches ESLint's default.
export const ECMA_VERSION = 2026;
const ECMA_VERSION_NUMBER = 17;

// Supported ECMAScript versions. This matches ESLint's default.
const SUPPORTED_ECMA_VERSIONS = freeze([3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]);

// Singleton object for parser's `Syntax` property. Generated lazily.
let Syntax: Record<string, string> | null = null;

// Singleton object for parser.
const PARSER = freeze({
  /**
   * Parser name.
   */
  name: "oxc",

  /**
   * Parser version.
   */
  // TODO: This can be statically defined, but need it be to be updated when we make a new release.
  version: "0.0.0",

  /**
   * Parse code into an AST.
   * @param code - Code to parse
   * @param options? - Parser options
   * @returns AST
   */
  // oxlint-disable-next-line no-unused-vars
  parse(code: string, options?: Record<string, unknown>): Program {
    throw new Error("`context.languageOptions.parser.parse` not implemented yet."); // TODO
  },

  /**
   * Visitor keys for AST nodes.
   */
  VisitorKeys: visitorKeys,

  /**
   * Ast node types.
   */
  get Syntax(): Readonly<Record<string, string>> {
    // Construct lazily, as it's probably rarely used
    if (Syntax === null) {
      Syntax = ObjectCreate(null);
      for (const key in visitorKeys) {
        Syntax![key] = key;
      }
      freeze(Syntax);
    }
    return Syntax!;
  },

  /**
   * Latest ECMAScript version supported by parser.
   */
  latestEcmaVersion: ECMA_VERSION_NUMBER,

  /**
   * ECMAScript versions supported by parser.
   */
  supportedEcmaVersions: SUPPORTED_ECMA_VERSIONS,
});

// Singleton object for parser options.
// TODO: `sourceType` is the only property ESLint provides. But does TS-ESLint provide any further properties?
const PARSER_OPTIONS = freeze({
  /**
   * Source type of the file being linted.
   */
  get sourceType(): ModuleKind {
    // TODO: Would be better to get `sourceType` without deserializing whole AST,
    // in case it's used in `create` to return an empty visitor if wrong type.
    // TODO: ESLint also has `commonjs` option.
    if (ast === null) initAst();
    debugAssertIsNonNull(ast);

    return ast.sourceType;
  },
});

// Singleton object for language options.
const LANGUAGE_OPTIONS = {
  /**
   * Source type of the file being linted.
   */
  get sourceType(): ModuleKind {
    // TODO: Would be better to get `sourceType` without deserializing whole AST,
    // in case it's used in `create` to return an empty visitor if wrong type.
    // TODO: ESLint also has `commonjs` option.
    if (ast === null) initAst();
    debugAssertIsNonNull(ast);

    return ast.sourceType;
  },

  /**
   * ECMAScript version of the file being linted.
   */
  ecmaVersion: ECMA_VERSION,

  /**
   * Parser used to parse the file being linted.
   */
  parser: PARSER,

  /**
   * Parser options used to parse the file being linted.
   */
  // Note: If we change this implementation, also change `parserOptions` getter on `FILE_CONTEXT` below
  parserOptions: PARSER_OPTIONS,

  /**
   * Globals defined for the file being linted.
   */
  get globals(): Readonly<Globals> | null {
    if (globals === null) initGlobals();
    debugAssertIsNonNull(globals);

    // ESLint has `globals` as `null`, not empty object, if no globals are defined
    return globals === EMPTY_GLOBALS ? null : globals;
  },
};

// In conformance build, replace `LANGUAGE_OPTIONS.ecmaVersion` with a getter which returns value of local var.
// This is to allow changing the ECMAScript version in conformance tests.
// Some of ESLint's rules change behavior based on the version, and ESLint's tests rely on this.
let ecmaVersion = ECMA_VERSION;

export function setEcmaVersion(version: number): void {
  if (!CONFORMANCE) throw new Error("Should be unreachable in release or debug builds");
  ecmaVersion = version;
}

if (CONFORMANCE) {
  Object.defineProperty(LANGUAGE_OPTIONS, "ecmaVersion", {
    get(): number {
      return ecmaVersion;
    },
  });
}

freeze(LANGUAGE_OPTIONS);

/**
 * Language options used when parsing a file.
 */
export type LanguageOptions = Readonly<typeof LANGUAGE_OPTIONS>;

// Singleton object for file-specific properties.
//
// Only one file is linted at a time, so we reuse a single object for all files.
// This object is used as the prototype for `Context` objects for each rule.
// It has no state, only getters which return other singletons, or global variables.
//
// IMPORTANT: Getters must not use `this`, to support wrapped context objects.
// https://github.com/oxc-project/oxc/issues/15325
//
// # Deprecated methods
//
// Some methods and getters are deprecated. They are all marked `@deprecated` below.
// These are present in ESLint 9, but are being removed in ESLint 10.
// https://eslint.org/blog/2025/10/whats-coming-in-eslint-10.0.0/#removing-deprecated-rule-context-members
//
// We have decided to keep them, as some existing ESLint plugins use them, and those plugins won't work with Oxlint
// without these methods/getters.
// Our hope is that Oxlint will remain on v1.x for a long time, so we'll be stuck with these deprecated methods
// long after ESlint has removed them.
// We don't think this is a problem, because the implementations are trivial, and no maintenance burden.
//
// However, we still want to discourage using these deprecated methods/getters in rules, because such rules
// will not work in ESLint 10 in compatibility mode.
//
// TODO: When we write a rule tester, throw an error in the tester if the rule uses deprecated methods/getters.
// We'll need to offer an option to opt out of these errors, for rules which delegate to another rule whose code
// the author doesn't control.
const FILE_CONTEXT = freeze({
  /**
   * Absolute path of the file being linted.
   */
  get filename(): string {
    // Note: If we change this implementation, also change `getFilename` method below
    if (filePath === null) throw new Error("Cannot access `context.filename` in `createOnce`");
    return filePath;
  },

  /**
   * Get absolute path of the file being linted.
   * @returns Absolute path of the file being linted.
   * @deprecated Use `context.filename` property instead.
   */
  getFilename(): string {
    if (filePath === null) throw new Error("Cannot call `context.getFilename` in `createOnce`");
    return filePath;
  },

  /**
   * Physical absolute path of the file being linted.
   */
  // TODO: Unclear how this differs from `filename`.
  get physicalFilename(): string {
    // Note: If we change this implementation, also change `getPhysicalFilename` method below
    if (filePath === null) {
      throw new Error("Cannot access `context.physicalFilename` in `createOnce`");
    }
    return filePath;
  },

  /**
   * Get physical absolute path of the file being linted.
   * @returns Physical absolute path of the file being linted.
   * @deprecated Use `context.physicalFilename` property instead.
   */
  getPhysicalFilename(): string {
    if (filePath === null) {
      throw new Error("Cannot call `context.getPhysicalFilename` in `createOnce`");
    }
    return filePath;
  },

  /**
   * Current working directory.
   */
  get cwd(): string {
    // Note: We can allow accessing `cwd` in `createOnce`, as it's global.
    // Note: If we change this implementation, also change `getCwd` method below,
    // and `cwd` getter + `getCwd` method in `index.ts` (`createOnce` shim for ESLint).
    if (cwd === null) cwd = process.cwd();
    return cwd;
  },

  /**
   * Get current working directory.
   * @returns The current working directory.
   * @deprecated Use `context.cwd` property instead.
   */
  getCwd(): string {
    if (cwd === null) cwd = process.cwd();
    return cwd;
  },

  /**
   * Source code of the file being linted.
   */
  get sourceCode(): SourceCode {
    // Note: If we change this implementation, also change `getSourceCode` method below
    if (filePath === null) throw new Error("Cannot access `context.sourceCode` in `createOnce`");
    return SOURCE_CODE;
  },

  /**
   * Get source code of the file being linted.
   * @returns Source code of the file being linted.
   * @deprecated Use `context.sourceCode` property instead.
   */
  getSourceCode(): SourceCode {
    if (filePath === null) throw new Error("Cannot call `context.getSourceCode` in `createOnce`");
    return SOURCE_CODE;
  },

  /**
   * Language options used when parsing this file.
   */
  get languageOptions(): LanguageOptions {
    if (filePath === null) {
      throw new Error("Cannot access `context.languageOptions` in `createOnce`");
    }
    return LANGUAGE_OPTIONS;
  },

  /**
   * Settings for the file being linted.
   */
  get settings(): Readonly<Settings> {
    if (filePath === null) throw new Error("Cannot access `context.settings` in `createOnce`");

    if (settings === null) initSettings();
    debugAssertIsNonNull(settings);

    return settings;
  },

  /**
   * Create a new object with the current object as the prototype and
   * the specified properties as its own properties.
   * @param extension - The properties to add to the new object.
   * @returns A new object with the current object as the prototype
   *   and the specified properties as its own properties.
   */
  extend(this: FileContext, extension: Record<string | number | symbol, unknown>): FileContext {
    // Note: We can allow calling `extend` in `createOnce`, as it involves no file-specific state
    return freeze(ObjectAssign(ObjectCreate(this), extension));
  },

  /**
   * Parser options used to parse the file being linted.
   * @deprecated Use `languageOptions.parserOptions` instead.
   */
  get parserOptions(): Record<string, unknown> {
    if (filePath === null) throw new Error("Cannot access `context.parserOptions` in `createOnce`");
    return PARSER_OPTIONS;
  },

  /**
   * The path to the parser used to parse this file.
   * @deprecated No longer supported.
   */
  get parserPath(): string {
    // TODO: Implement this?
    throw new Error("`context.parserPath` is unsupported at present (and deprecated)");
  },
});

/**
 * Context object for a file.
 * Is the prototype for `Context` objects for each rule.
 */
export type FileContext = typeof FILE_CONTEXT;

/**
 * Context object for a rule.
 * Passed to `create` and `createOnce` functions.
 */
export interface Context extends FileContext {
  /**
   * Rule ID, in form `<plugin>/<rule>`.
   */
  id: string;
  /**
   * Rule options for this rule on this file.
   */
  options: Readonly<Options>;
  /**
   * Report an error/warning.
   */
  report(this: void, diagnostic: Diagnostic): void;
}

/**
 * Create `Context` object for a rule.
 * @param fullRuleName - Full rule name, including plugin name e.g. `my-plugin/my-rule`
 * @param ruleDetails - `RuleDetails` object
 * @returns `Context` object
 */
export function createContext(fullRuleName: string, ruleDetails: RuleDetails): Readonly<Context> {
  // Create `Context` object for rule.
  //
  // All properties are enumerable, to support a pattern which some ESLint plugins use:
  // ```
  // function create(context) {
  //   const wrappedContext = {
  //     __proto__: Object.getPrototypeOf(context),
  //     ...context,
  //     report = (diagnostic) => {
  //       doSomethingBeforeReporting(diagnostic);
  //       context.report(diagnostic);
  //     },
  //   };
  //   return baseRule.create(wrappedContext);
  // }
  // ```
  //
  // Object is frozen to prevent user mutating it.
  //
  // IMPORTANT: Methods/getters must not use `this`, to support wrapped context objects
  // or e.g. `const { report } = context; report(diagnostic);`.
  // https://github.com/oxc-project/oxc/issues/15325
  return freeze({
    // Inherit from `FILE_CONTEXT`, which provides getters for file-specific properties
    __proto__: FILE_CONTEXT,
    // Rule ID, in form `<plugin>/<rule>`
    get id(): string {
      // It's not possible to allow access to `id` in `createOnce` in ESLint compatibility mode, so we don't
      // allow it here either. It's probably not very useful anyway - a rule should know what its own name is!
      if (filePath === null) throw new Error("Cannot access `context.id` in `createOnce`");
      return fullRuleName;
    },
    // Getter for rule options for this rule on this file
    get options(): Readonly<Options> {
      if (filePath === null) throw new Error("Cannot access `context.options` in `createOnce`");
      debugAssertIsNonNull(ruleDetails.options);
      return ruleDetails.options;
    },
    /**
     * Report error.
     * @param diagnostic - Diagnostic object
     * @throws {TypeError} If `diagnostic` is invalid
     */
    report(this: void, diagnostic: Diagnostic): void {
      // Delegate to `report` implementation shared between all rules, passing rule-specific details (`RuleDetails`)
      report(diagnostic, ruleDetails);
    },
  } as unknown as Context); // It seems TS can't understand `__proto__: FILE_CONTEXT`
}
