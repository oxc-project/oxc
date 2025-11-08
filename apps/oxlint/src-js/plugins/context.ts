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

import { getFixes } from './fix.js';
import { getOffsetFromLineColumn } from './location.js';
import { ast, initAst, SOURCE_CODE } from './source_code.js';
import { settings, initSettings } from './settings.js';

import type { Fix, FixFn } from './fix.ts';
import type { RuleDetails } from './load.ts';
import type { SourceCode } from './source_code.ts';
import type { Location, Ranged } from './types.ts';
import type { ModuleKind } from '../generated/types.d.ts';

const { hasOwn, keys: ObjectKeys, freeze, assign: ObjectAssign, create: ObjectCreate } = Object;

// Diagnostic in form passed by user to `Context#report()`
export type Diagnostic = DiagnosticWithNode | DiagnosticWithLoc;

export interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  data?: Record<string, string | number> | null | undefined;
  fix?: FixFn;
}

export interface DiagnosticWithNode extends DiagnosticBase {
  node: Ranged;
}

export interface DiagnosticWithLoc extends DiagnosticBase {
  loc: Location;
}

// Diagnostic in form sent to Rust
interface DiagnosticReport {
  message: string;
  start: number;
  end: number;
  ruleIndex: number;
  fixes: Fix[] | null;
}

// Diagnostics array. Reused for every file.
export const diagnostics: DiagnosticReport[] = [];

// Cached current working directory
let cwd: string | null = null;

// Absolute path of file being linted.
// When `null`, indicates that no file is currently being linted (in `createOnce`, or between linting files).
let filePath: string | null = null;

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
const ECMA_VERSION = 2026;

// Singleton object for parser options.
const PARSER_OPTIONS = freeze({
  /**
   * Source type of the file being linted.
   */
  get sourceType(): ModuleKind {
    // TODO: Would be better to get `sourceType` without deserializing whole AST,
    // in case it's used in `create` to return an empty visitor if wrong type.
    // TODO: ESLint also has `commonjs` option.
    if (ast === null) initAst();
    return ast.sourceType;
  },
});

// Singleton object for language options.
const LANGUAGE_OPTIONS = freeze({
  /**
   * Source type of the file being linted.
   */
  get sourceType(): ModuleKind {
    // TODO: Would be better to get `sourceType` without deserializing whole AST,
    // in case it's used in `create` to return an empty visitor if wrong type.
    // TODO: ESLint also has `commonjs` option.
    if (ast === null) initAst();
    return ast.sourceType;
  },

  /**
   * ECMAScript version of the file being linted.
   */
  ecmaVersion: ECMA_VERSION,

  /**
   * Parser used to parse the file being linted.
   */
  get parser(): Record<string, unknown> {
    throw new Error('`context.languageOptions.parser` is not implemented yet.'); // TODO
  },

  /**
   * Parser options used to parse the file being linted.
   */
  parserOptions: PARSER_OPTIONS,

  /**
   * Globals defined for the file being linted.
   */
  // ESLint has `globals` as `null`, not empty object, if no globals are defined.
  get globals(): Record<string, 'readonly' | 'writable' | 'off'> | null {
    // TODO: Get globals from Rust side.
    // Note: ESLint's type is "writable", whereas Oxlint's is "writeable" (misspelled with extra "e").
    // Probably we should fix that on Rust side (while still allowing "writeable").
    return null;
  },
});

/**
 * Language options used when parsing a file.
 */
export type LanguageOptions = typeof LANGUAGE_OPTIONS;

// Singleton object for file-specific properties.
//
// Only one file is linted at a time, so we reuse a single object for all files.
// This object is used as the prototype for `Context` objects for each rule.
// It has no state, only getters which return other singletons, or global variables.
//
// IMPORTANT: Getters must not use `this`, to support wrapped context objects.
// https://github.com/oxc-project/oxc/issues/15325
const FILE_CONTEXT = freeze({
  /**
   * Absolute path of the file being linted.
   */
  get filename(): string {
    if (filePath === null) throw new Error('Cannot access `context.filename` in `createOnce`');
    return filePath;
  },

  /**
   * Physical absolute path of the file being linted.
   */
  // TODO: Unclear how this differs from `filename`.
  get physicalFilename(): string {
    if (filePath === null) throw new Error('Cannot access `context.physicalFilename` in `createOnce`');
    return filePath;
  },

  /**
   * Current working directory.
   */
  get cwd(): string {
    // Note: We can allow accessing `cwd` in `createOnce`, as it's global
    if (cwd === null) cwd = process.cwd();
    return cwd;
  },

  /**
   * Source code of the file being linted.
   */
  get sourceCode(): SourceCode {
    if (filePath === null) throw new Error('Cannot access `context.sourceCode` in `createOnce`');
    return SOURCE_CODE;
  },

  /**
   * Language options used when parsing this file.
   */
  get languageOptions(): LanguageOptions {
    if (filePath === null) throw new Error('Cannot access `context.languageOptions` in `createOnce`');
    return LANGUAGE_OPTIONS;
  },

  /**
   * Settings for the file being linted.
   */
  get settings(): Record<string, unknown> {
    if (filePath === null) throw new Error('Cannot access `context.settings` in `createOnce`');
    if (settings === null) initSettings();
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
    return freeze(ObjectAssign(ObjectCreate(this), extension));
  },

  // ---------------
  // Deprecated APIs
  // ---------------

  /**
   * Parser options for the file being linted.
   * @deprecated Use `languageOptions` instead.
   */
  get parserOptions(): Record<string, unknown> {
    // TODO: Implement this?
    throw new Error('`context.parserOptions` is deprecated. Use `languageOptions` instead.');
  },

  /**
   * The path to the parser used to parse this file.
   * @deprecated No longer supported.
   */
  get parserPath(): string {
    // TODO: Implement this?
    throw new Error('`context.parserPath` is deprecated. No longer supported.');
  },

  /**
   * Get current working directory.
   * @returns The current working directory.
   * @deprecated Use `cwd` instead.
   */
  getCwd(): string {
    // TODO: Implement this?
    throw new Error('`context.getCwd` is deprecated. Use `cwd` instead.');
  },

  /**
   * Get absolute path of the file being linted.
   * @returns Absolute path of the file being linted.
   * @deprecated Use `filename` instead.
   */
  getFilename(): string {
    // TODO: Implement this?
    throw new Error('`context.getFilename` is deprecated. Use `filename` instead.');
  },

  /**
   * Get physical absolute path of the file being linted.
   * @returns Physical absolute path of the file being linted.
   * @deprecated Use `physicalFilename` instead.
   */
  getPhysicalFilename(): string {
    // TODO: Implement this?
    throw new Error('`context.getPhysicalFilename` is deprecated. Use `physicalFilename` instead.');
  },

  /**
   * Get source code of the file being linted.
   * @returns Source code of the file being linted.
   * @deprecated Use `sourceCode` instead.
   */
  getSourceCode(): SourceCode {
    // TODO: Implement this?
    throw new Error('`context.getSourceCode` is deprecated. Use `sourceCode` instead.');
  },
});

/**
 * Context object for a file.
 * Is the prototype for `Context` objects for each rule.
 */
type FileContext = typeof FILE_CONTEXT;

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
  options: unknown[];
  /**
   * Report an error/warning.
   */
  report(diagnostic: Diagnostic): void;
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
    id: fullRuleName,
    // Getter for rule options for this rule on this file
    get options(): Readonly<unknown[]> {
      if (filePath === null) throw new Error('Cannot access `context.options` in `createOnce`');
      return ruleDetails.options;
    },
    /**
     * Report error.
     * @param diagnostic - Diagnostic object
     * @throws {TypeError} If `diagnostic` is invalid
     */
    report(diagnostic: Diagnostic): void {
      // Delegate to `reportImpl`, passing rule-specific details (`RuleDetails`)
      reportImpl(diagnostic, ruleDetails);
    },
  } as unknown as Context); // It seems TS can't understand `__proto__: FILE_CONTEXT`
}

/**
 * Report error.
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific details e.g. `isFixable`
 * @throws {TypeError} If `diagnostic` is invalid
 */
function reportImpl(diagnostic: Diagnostic, ruleDetails: RuleDetails): void {
  if (filePath === null) throw new Error('Cannot report errors in `createOnce`');

  // Get message, resolving message from `messageId` if present
  let message = getMessage(diagnostic, ruleDetails);

  // Interpolate placeholders {{key}} with data values
  if (hasOwn(diagnostic, 'data')) {
    const { data } = diagnostic;
    if (data != null) {
      message = message.replace(/\{\{([^}]+)\}\}/g, (match, key) => {
        key = key.trim();
        const value = data[key];
        return value !== undefined ? String(value) : match;
      });
    }
  }

  // TODO: Validate `diagnostic`
  let start: number, end: number, loc: Location;

  if (hasOwn(diagnostic, 'loc') && (loc = (diagnostic as DiagnosticWithLoc).loc) != null) {
    // `loc`
    if (typeof loc !== 'object') throw new TypeError('`loc` must be an object');
    start = getOffsetFromLineColumn(loc.start);
    end = getOffsetFromLineColumn(loc.end);
  } else {
    // `node`
    const { node } = diagnostic as DiagnosticWithNode;
    if (node == null) throw new TypeError('Either `node` or `loc` is required');
    if (typeof node !== 'object') throw new TypeError('`node` must be an object');

    // ESLint uses `loc` here instead of `range`.
    // We can't do that because AST nodes don't have `loc` property yet. In any case, `range` is preferable,
    // as otherwise we have to convert `loc` to `range` which is expensive at present.
    // TODO: Revisit this once we have `loc` support in AST, and a fast translation table to convert `loc` to `range`.
    const { range } = node;
    if (range === null || typeof range !== 'object') throw new TypeError('`node.range` must be present');
    start = range[0];
    end = range[1];

    // Do type validation checks here, to ensure no error in serialization / deserialization.
    // Range validation happens on Rust side.
    if (
      typeof start !== 'number' ||
      typeof end !== 'number' ||
      start < 0 ||
      end < 0 ||
      (start | 0) !== start ||
      (end | 0) !== end
    ) {
      throw new TypeError('`node.range[0]` and `node.range[1]` must be non-negative integers');
    }
  }

  diagnostics.push({
    message,
    start,
    end,
    ruleIndex: ruleDetails.ruleIndex,
    fixes: getFixes(diagnostic, ruleDetails),
  });
}

/**
 * Get message from diagnostic.
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `messages`
 * @returns Message string
 * @throws {Error|TypeError} If neither `message` nor `messageId` provided, or of wrong type
 */
function getMessage(diagnostic: Diagnostic, ruleDetails: RuleDetails): string {
  if (hasOwn(diagnostic, 'messageId')) {
    const { messageId } = diagnostic as { messageId: string | null | undefined };
    if (messageId != null) return resolveMessageFromMessageId(messageId, ruleDetails);
  }

  if (hasOwn(diagnostic, 'message')) {
    const { message } = diagnostic;
    if (typeof message === 'string') return message;
    if (message != null) throw new TypeError('`message` must be a string');
  }

  throw new Error('Either `message` or `messageId` is required');
}

/**
 * Resolve a message ID to its message string, with optional data interpolation.
 * @param messageId - The message ID to resolve
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `messages`
 * @returns Resolved message string
 * @throws {Error} If `messageId` is not found in `messages`
 */
function resolveMessageFromMessageId(messageId: string, ruleDetails: RuleDetails): string {
  const { messages } = ruleDetails;
  if (messages === null) {
    throw new Error(`Cannot use messageId '${messageId}' - rule does not define any messages in \`meta.messages\``);
  }

  if (!hasOwn(messages, messageId)) {
    throw new Error(
      `Unknown messageId '${messageId}'. Available \`messageIds\`: ${ObjectKeys(messages)
        .map((msg) => `'${msg}'`)
        .join(', ')}`,
    );
  }

  return messages[messageId];
}
