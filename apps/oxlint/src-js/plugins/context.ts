import { getFixes } from './fix.js';
import { SourceCode } from './source_code.js';

import type { Fix, FixFn } from './fix.ts';
import type { Node } from './types.ts';

// Diagnostic in form passed by user to `Context#report()`
export interface Diagnostic {
  message: string;
  node: Node;
  fix?: FixFn;
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

/**
 * Update a `Context` with file-specific data.
 *
 * We have to define this function within class body, as it's not possible to access private property
 * `#internal` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 *
 * @param context - `Context` object
 * @param ruleIndex - Index of this rule within `ruleIds` passed from Rust
 * @param filePath - Absolute path of file being linted
 * @param sourceText - Source text of file being linted
 */
export let setupContextForFile: (
  context: Context,
  ruleIndex: number,
  filePath: string,
  sourceText: string,
) => void;

/**
 * Get internal data from `Context`.
 *
 * Throws an `Error` if `Context` has not been set up for a file (in body of `createOnce`).
 *
 * We have to define this function within class body, as it's not possible to access private property
 * `#internal` from outside the class.
 * We don't use a normal class method, because we don't want to expose this to user.
 * We don't use a private class method, because private property/method accesses are somewhat expensive.
 *
 * @param context - `Context` object
 * @param actionDescription - Description of the action being attempted. Used in error message if context is not set up.
 * @returns `InternalContext` object
 * @throws {Error} If context has not been set up
 */
let getInternal: (context: Context, actionDescription: string) => InternalContext;

// Internal data within `Context` that don't want to expose to plugins.
// Stored as `#internal` property of `Context`.
export interface InternalContext {
  // Full rule name, including plugin name e.g. `my-plugin/my-rule`.
  id: string;
  // Index into `ruleIds` sent from Rust
  ruleIndex: number;
  // Absolute path of file being linted
  filePath: string;
  // `SourceCode` class instance for this rule.
  // Rule has single `SourceCode` instance that is updated for each file
  // (NOT new `SourceCode` instance for each file).
  sourceCode: SourceCode;
  // Options
  options: unknown[];
  // `true` if rule can provide fixes (`meta.fixable` in `RuleMeta` is 'code' or 'whitespace')
  isFixable: boolean;
}

/**
 * Context class.
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` function.
 */
export class Context {
  // Internal data.
  // Initialized in constructor, updated by `setupContextForFile` before running visitor on file.
  #internal: InternalContext;

  /**
   * @class
   * @param fullRuleName - Rule name, in form `<plugin>/<rule>`
   */
  constructor(fullRuleName: string, isFixable: boolean) {
    this.#internal = {
      id: fullRuleName,
      filePath: '',
      sourceCode: new SourceCode(),
      ruleIndex: -1,
      options: [],
      isFixable,
    };
  }

  // Getter for full rule name, in form `<plugin>/<rule>`
  get id() {
    return getInternal(this, 'access `context.id`').id;
  }

  // Getter for absolute path of file being linted.
  get filename() {
    return getInternal(this, 'access `context.filename`').filePath;
  }

  // Getter for absolute path of file being linted.
  // TODO: Unclear how this differs from `filename`.
  get physicalFilename() {
    return getInternal(this, 'access `context.physicalFilename`').filePath;
  }

  // Getter for options for file being linted.
  get options() {
    return getInternal(this, 'access `context.options`').options;
  }

  // Getter for `SourceCode` for file being linted.
  get sourceCode() {
    return getInternal(this, 'access `context.sourceCode`').sourceCode;
  }

  /**
   * Report error.
   * @param diagnostic - Diagnostic object
   */
  report(diagnostic: Diagnostic): void {
    const internal = getInternal(this, 'report errors');

    // TODO: Validate `diagnostic`
    const { node } = diagnostic;
    diagnostics.push({
      message: diagnostic.message,
      start: node.start,
      end: node.end,
      ruleIndex: internal.ruleIndex,
      fixes: getFixes(diagnostic, internal),
    });
  }

  static {
    setupContextForFile = (context, ruleIndex, filePath, sourceText) => {
      // TODO: Support `options`
      const internal = context.#internal;
      internal.ruleIndex = ruleIndex;
      internal.filePath = filePath;
      internal.sourceCode.text = sourceText;
    };

    getInternal = (context, actionDescription) => {
      const internal = context.#internal;
      if (internal.ruleIndex === -1) throw new Error(`Cannot ${actionDescription} in \`createOnce\``);
      return internal;
    };
  }
}
