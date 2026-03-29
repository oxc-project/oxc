/*
 * `eslintCompatPlugin` function.
 * Converts an Oxlint plugin using `createOnce` to a plugin which will run in ESLint.
 */

import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Context, FileContext, LanguageOptions } from "../plugins/context.ts";
import type { CreateOnceRule, Plugin, Rule } from "../plugins/load.ts";
import type { Settings } from "../plugins/settings.ts";
import type { SourceCode } from "../plugins/source_code.ts";
import type { BeforeHook, Visitor, VisitorWithHooks } from "../plugins/types.ts";
import type { Program, Node as ESTreeNode } from "../generated/types.d.ts";
import type { SetNullable } from "../utils/types.ts";

// Empty visitor object, returned by `create` when `before` hook returns `false`.
const EMPTY_VISITOR: Visitor = {};

// State of an `after` hook.
// `AFTER_HOOK_INACTIVE` = doesn't need to run, `AFTER_HOOK_PENDING` = needs to run.
// This is a "poor man's enum" which minifies better than a TS enum.
type PendingState = typeof AFTER_HOOK_INACTIVE | typeof AFTER_HOOK_PENDING;
const AFTER_HOOK_INACTIVE = 0;
const AFTER_HOOK_PENDING = 1;

/**
 * Convert a plugin which used Oxlint's `createOnce` API to also work with ESLint.
 *
 * If any of the plugin's rules use the Oxlint alternative `createOnce` API,
 * add ESLint-compatible `create` methods to those rules, which delegate to `createOnce`.
 * This makes the plugin compatible with ESLint.
 *
 * The `plugin` object passed in is mutated in-place.
 *
 * @param plugin - Plugin to convert
 * @returns Plugin with all rules having `create` method
 * @throws {Error} If `plugin` is not an object, or `plugin.rules` is not an object
 */
export function eslintCompatPlugin(plugin: Plugin): Plugin {
  // Validate type of `plugin`
  if (plugin === null || typeof plugin !== "object") throw new Error("Plugin must be an object");

  const { rules } = plugin;
  if (rules === null || typeof rules !== "object") {
    throw new Error("Plugin must have an object as `rules` property");
  }

  // Set up state for tracking if any `after` hooks need to be called
  const afterHooksState = new AfterHooksState();

  // Make each rule in the plugin ESLint-compatible by calling `convertRule` on it
  for (const ruleName in rules) {
    if (Object.hasOwn(rules, ruleName)) convertRule(rules[ruleName], afterHooksState);
  }

  return plugin;
}

/**
 * Class containing state for tracking if any `after` hooks for a plugin's rules need to be called.
 *
 * # Aims
 *
 * Aims are:
 * 1. `after` hook of each rule runs after all other AST visit functions, and CFG event handlers.
 * 2. `after` hooks for *all* a plugin's rules run after *all* that plugin's rules have completed visiting AST.
 * 3. `after` hooks for *all* a plugin's rules run before *any* of plugin's rules begin linting another file.
 * 4. In the case of an error during AST traversal, `after` hooks are always still run.
 *
 * The above exactly matches the behavior when running a `createOnce` rule in Oxlint.
 *
 * # Why this is important
 *
 * All the complication comes from ensuring `after` hooks run even after an error during AST traversal.
 *
 * In ESLint CLI, an error will crash the process, so it doesn't particularly matter if `after` hooks run or not,
 * but language servers will typically swallow errors, and keep the process running.
 *
 * Rules using `before` and `after` hooks will often rely on both hooks running in a predictable order,
 * to maintain some internal state. For example, they may use `before` and `after` hooks to maintain a per-file
 * cache of data which is shared between rules. The cache use case is why rule (2) above is important.
 *
 * Below is an example of using `before` and `after` hooks to maintain a per-file cache, shared between rules.
 * It relies on all `before` hooks running before any rule starts visiting the AST,
 * and all `after` hooks running after all rules have finished visiting the AST.
 *
 * ```ts
 * let cache: Data | null = null;
 *
 * let numRunningRules = 0;
 *
 * const setupCache = (context) => {
 *   if (cache === null) cache = new Data(context);
 *   numRunningRules++;
 * };
 *
 * const teardownCache = () => {
 *   numRunningRules--;
 *   if (numRunningRules === 0) cache = null;
 * };
 *
 * const rule1 = {
 *   createOnce(context) {
 *     return {
 *       before() {
 *         setupCache(context);
 *       },
 *       Identifier(node) {
 *         // Use `cache`
 *       },
 *       after: teardownCache,
 *     };
 *   },
 * };
 *
 * const rule2 = {
 *   // Same as above
 * };
 *
 * const rule3 = {
 *   // Same as above
 * };
 * ```
 *
 * If `after` hooks did not always run, the next lint run could get stale state, and malfunction.
 * If `after` hooks ran in the wrong order (e.g. after some `before` hooks for next file),
 * `numRunningRules` would never get to 0, and cache would never be cleared.
 *
 * Note that because all rules run together in a single AST traversal, if a rule from plugin X throws an error,
 * it can disrupt rules from plugin Y. This would make it hard to debug.
 *
 * # Mechanism
 *
 * ## Initialization
 *
 * Rules with an `after` hook register themselves by:
 *
 * 1. Calling `registerResetFunction` to register a function to run `after` hook and clean up internal state.
 *    This call adds the reset fn to `resetFunctions`, and adds `AFTER_HOOK_INACTIVE` to `pendingStates`.
 * 2. Adding an `onCodePathEnd` CFG event handler to the visitor which calls `ruleFinished` at end of AST traversal.
 *
 * ## Per-file setup
 *
 * Before linting a file, `create` will call `setupAfterHook` which is created by `createContextAndVisitor`.
 * This registers that the `after` hook for the rule needs to run, by setting `pendingStates[ruleIndex]`
 * to `AFTER_HOOK_PENDING`, and incrementing `pendingCount`.
 *
 * If a cleanup microtask has not been scheduled yet, one is scheduled now (see reason below).
 *
 * ## Normal operation
 *
 * AST traversal for each rule ends with `ruleFinished` hook being called from `onCodePathEnd` CFG event handler.
 * It increments `lintFinishedCount`. If `lintFinishedCount` equals `pendingCount`, all rules have finished linting
 * the file, and `reset` is called, which calls all the pending `after` hooks.
 *
 * ## Error handling
 *
 * If an error is thrown during AST traversal, we ensure that `after` hooks are still run by 2 mechanisms:
 *
 * ### 1. Next microtick
 *
 * Before any rules began linting files, a microtask was scheduled, which runs on next micro-tick.
 * All language servers we're aware of run each lint task in a separate tick, so this microtask will run in next tick
 * after a linting run, before the next lint task starts.
 *
 * If the linting run completed successfully, the microtask does nothing.
 *
 * But if an error was thrown during AST traversal, this will be visible from the state of `pendingCount`.
 * The microtask will run any `after` hooks which need to be run, and reset state to reflect that there are
 * no more pending `after` hooks.
 *
 * ### 2. Fallback: Next lint run
 *
 * Before linting any file, the state of `pendingCount` is checked.
 * If any `after` hooks are still pending, they are run immediately.
 * They're run before the `context` objects in `createOnce` closures are updated to the next file,
 * so they run with access to the old `context` object from the last file.
 *
 * This fallback should not be required, but it's included as "belt and braces", to handle if any language server
 * or other environment running ESLint programmatically, does not pause a tick between linting runs.
 */
class AfterHooksState {
  // Array of functions to call to clean up state and run `after` hook for rules which have an `after` hook.
  resetFunctions: (() => void)[] = [];

  // Array of flags indicating if reset functions needs to be called.
  // Each entry corresponds to an entry in `resetFunctions`.
  pendingStates: PendingState[] = [];

  // Count of reset functions which need to be called.
  // Equal to number of `AFTER_HOOK_PENDING` entries in `pendingStates`.
  pendingCount: number = 0;

  // Count of rules with `after` hooks which have completed linting current file.
  lintFinishedCount: number = 0;

  // `true` if a microtask has been scheduled to call `reset` yet.
  resetIsScheduled: boolean = false;

  // `SourceCode` object for file currently being linted.
  // Used to determine at start of `create` whether we're linting a new file, or still on current one.
  sourceCode: SourceCode | null = null;

  // Function which is scheduled as the cleanup microtask.
  private resetMicrotask: () => void = this.resetMicrotaskImpl.bind(this);

  /**
   * Register a function to run `after` hook for a rule, and reset state.
   * @param reset - Function to run `after` hook and reset state
   * @returns Index of rule
   */
  registerResetFunction(reset: () => void): number {
    const { pendingStates } = this;
    const index = pendingStates.length;
    pendingStates.push(AFTER_HOOK_INACTIVE);
    this.resetFunctions.push(reset);
    return index;
  }

  /**
   * Register that a rule with `after` hook has completed linting a file.
   * Called by `onCodePathEnd` CFG event handler which is added to visitor for rules with `after` hooks.
   *
   * If all rules with an `after` hook which needs to be run have completed linting the file, run all `after` hooks.
   */
  ruleFinished(): void {
    this.lintFinishedCount++;
    if (this.lintFinishedCount === this.pendingCount) {
      // All rules with `after` hooks have finished linting the file. Run all `after` hooks.
      // `false` to throw any errors which occur in `after` hooks.
      this.reset(false);
    }
  }

  /**
   * Call all reset functions where corresponding entry in `pendingStates` is `AFTER_HOOK_PENDING`.
   * Should only be called when some `after` hooks are pending.
   *
   * @param ignoreErrors - `true` to catch and silently ignore any errors which occur in `after` hooks.
   *   `false` to throw them,
   * @throws {unknown} If `ignoreErrors` is `false` and an error occurs in any `after` hooks.
   */
  reset(ignoreErrors: boolean): void {
    debugAssert(this.pendingCount > 0, "`pendingCount` should be > 0");

    const { resetFunctions, pendingStates } = this,
      hooksLen = pendingStates.length;

    let hasError = false,
      error: unknown;

    for (let i = 0; i < hooksLen; i++) {
      if (pendingStates[i] !== AFTER_HOOK_INACTIVE) {
        pendingStates[i] = AFTER_HOOK_INACTIVE;

        // Run reset function for rule.
        // Capture any errors - make sure all rules are reset, before throwing.
        try {
          resetFunctions[i]();
        } catch (e) {
          if (hasError === false) {
            hasError = true;
            error = e;
          }
        }
      }
    }

    // Reset state
    this.pendingCount = 0;
    this.lintFinishedCount = 0;

    // Clear `sourceCode` to free it for garbage collection
    this.sourceCode = null;

    // Throw error if there was one, unless `ignoreErrors` is `true`
    if (hasError === true && ignoreErrors === false) throw error;
  }

  /**
   * Schedule a microtask to run `reset` functions.
   */
  scheduleReset(): void {
    queueMicrotask(this.resetMicrotask);
    this.resetIsScheduled = true;
  }

  /**
   * Function which is scheduled as the cleanup microtask.
   * `scheduleReset` uses `resetMicrotask` which is this method bound to `this`.
   */
  private resetMicrotaskImpl(): void {
    this.resetIsScheduled = false;

    if (this.pendingCount !== 0) {
      // `true` to ignore errors. We're not in main "thread" of execution.
      this.reset(true);
    }
  }
}

/**
 * Convert a rule.
 *
 * The `rule` object passed in is mutated in-place.
 *
 * @param rule - Rule to convert
 * @param afterHooksState - State of `after` hooks
 * @throws {Error} If `rule` is not an object
 */
function convertRule(rule: Rule, afterHooksState: AfterHooksState) {
  // Validate type of `rule`
  if (rule === null || typeof rule !== "object") throw new Error("Rule must be an object");

  // If rule already has `create` method, no need to convert
  if ("create" in rule) return;

  // Add `create` function to `rule`
  let context: Context | null = null,
    visitor: Visitor | undefined,
    beforeHook: BeforeHook | null,
    setupAfterHook: ((program: Program) => void) | null;

  rule.create = (eslintContext) => {
    // Lazily call `createOnce` on first invocation of `create`
    if (context === null) {
      ({ context, visitor, beforeHook, setupAfterHook } = createContextAndVisitor(
        rule,
        afterHooksState,
      ));
    }
    debugAssertIsNonNull(visitor);

    const eslintFileContext = Object.getPrototypeOf(eslintContext);

    // If this rule has an `after` hook, check if `sourceCode` has changed.
    // If it has, this is first rule with an `after` hook to run on this file.
    // If any `after` hooks were not run at end of last file, run them now.
    if (setupAfterHook !== null) {
      const { sourceCode } = eslintFileContext;
      if (afterHooksState.sourceCode !== sourceCode) {
        afterHooksState.sourceCode = sourceCode;

        if (afterHooksState.pendingCount !== 0) {
          // `true` to ignore errors - any errors relate to *previous* file, so not appropriate to throw here
          afterHooksState.reset(true);
        }
      }
    }

    // Copy properties from ESLint's context object to `context`.
    // ESLint's context object is an object of form `{ id, options, report }`, with all other properties
    // and methods on another object which is its prototype.
    Object.defineProperties(context, {
      id: { value: eslintContext.id },
      options: { value: eslintContext.options },
      report: { value: eslintContext.report },
    });
    Object.setPrototypeOf(context, eslintFileContext);

    // If `before` hook returns `false`, skip traversal by returning an empty object as visitor
    if (beforeHook !== null) {
      const shouldRun = beforeHook();
      if (shouldRun === false) return EMPTY_VISITOR;
    }

    // If there's an `after` hook, call `setupAfterHook` with the `Program` node of the current file.
    // Schedule a microtask to run `after` hooks functions, if one hasn't already been scheduled.
    if (setupAfterHook !== null) {
      setupAfterHook(eslintFileContext.sourceCode.ast);
      if (afterHooksState.resetIsScheduled === false) afterHooksState.scheduleReset();
    }

    // Return same visitor each time
    return visitor;
  };
}

// File context object. Used as prototype for `Context` objects for each rule during `createOnce` call.
// When running the rules, ESLint's `context` object's prototype is switched in as prototype for `Context` objects.
//
// Only `extends` method is available in `createOnce`, so only that is implemented here.
// All other getters/methods throw, same as they do in main implementation.
//
// See `FILE_CONTEXT` in `plugins/context.ts` for details of all the getters/methods.
const FILE_CONTEXT: FileContext = Object.freeze({
  get filename(): string {
    throw new Error("Cannot access `context.filename` in `createOnce`");
  },

  getFilename(): string {
    throw new Error("Cannot call `context.getFilename` in `createOnce`");
  },

  get physicalFilename(): string {
    throw new Error("Cannot access `context.physicalFilename` in `createOnce`");
  },

  getPhysicalFilename(): string {
    throw new Error("Cannot call `context.getPhysicalFilename` in `createOnce`");
  },

  get cwd(): string {
    throw new Error("Cannot access `context.cwd` in `createOnce`");
  },

  getCwd(): string {
    throw new Error("Cannot call `context.getCwd` in `createOnce`");
  },

  get sourceCode(): SourceCode {
    throw new Error("Cannot access `context.sourceCode` in `createOnce`");
  },

  getSourceCode(): SourceCode {
    throw new Error("Cannot call `context.getSourceCode` in `createOnce`");
  },

  get languageOptions(): LanguageOptions {
    throw new Error("Cannot access `context.languageOptions` in `createOnce`");
  },

  get settings(): Readonly<Settings> {
    throw new Error("Cannot access `context.settings` in `createOnce`");
  },

  extend(this: FileContext, extension: Record<string | number | symbol, unknown>): FileContext {
    // Note: We can allow calling `extend` in `createOnce`, as it involves no file-specific state
    return Object.freeze(Object.assign(Object.create(this), extension));
  },

  get parserOptions(): Record<string, unknown> {
    throw new Error("Cannot access `context.parserOptions` in `createOnce`");
  },

  get parserPath(): string | undefined {
    throw new Error("Cannot access `context.parserPath` in `createOnce`");
  },
});

/**
 * Call `createOnce` method of rule, and return `Context`, `Visitor`, and `beforeHook` (if any).
 *
 * @param rule - Rule with `createOnce` method
 * @param afterHooksState - State of `after` hooks
 * @returns Object with `context`, `visitor`, and `beforeHook` properties,
 *   and `setupAfterHook` function if visitor has an `after` hook
 */
function createContextAndVisitor(
  rule: CreateOnceRule,
  afterHooksState: AfterHooksState,
): {
  context: Context;
  visitor: Visitor;
  beforeHook: BeforeHook | null;
  setupAfterHook: ((program: Program) => void) | null;
} {
  // Validate type of `createOnce`
  const { createOnce } = rule;
  if (createOnce == null) {
    throw new Error("Rules must define either a `create` or `createOnce` method");
  }
  if (typeof createOnce !== "function") {
    throw new Error("Rule `createOnce` property must be a function");
  }

  // Call `createOnce` with empty context object.
  // We don't use `Object.preventExtensions` on the context object, as we need to change its prototype for each file.
  const context: Context = Object.create(FILE_CONTEXT, {
    id: { value: null, enumerable: true, configurable: true },
    options: { value: null, enumerable: true, configurable: true },
    report: {
      value() {
        throw new Error("Cannot report errors in `createOnce`");
      },
      enumerable: true,
      configurable: true,
    },
  });

  let {
    before: beforeHook,
    after: afterHook,
    ...visitor
  } = createOnce.call(rule, context) as SetNullable<VisitorWithHooks, "before" | "after">;

  if (beforeHook === undefined) {
    beforeHook = null;
  } else if (beforeHook !== null && typeof beforeHook !== "function") {
    throw new Error("`before` property of visitor must be a function if defined");
  }

  // Handle `after` hook
  let setupAfterHook: ((program: Program) => void) | null = null;

  if (afterHook != null) {
    if (typeof afterHook !== "function") {
      throw new Error("`after` property of visitor must be a function if defined");
    }

    let program: Program | null = null;

    // Register a reset function. This calls `after` hook for the rule.
    // Its called after all rules with `after` hooks have finished linting the file, or after an error occurs.
    const ruleIndex = afterHooksState.registerResetFunction(() => {
      // Clear `program` to free the AST for garbage collection
      program = null;
      // Call `after` hook
      afterHook();
    });

    // Create setup function.
    // It is called by `create` before linting a file, passing `Program` node for the file.
    // Having `program` in a local var makes the `node === program` check in `onCodePathEnd` as cheap as it can be.
    // Otherwise it'd have to be `node === context.sourceCode.ast`, which would be slower.
    setupAfterHook = (ast: Program) => {
      // Store `Program` in local var, for use in `onCodePathEnd` CFG event handler
      program = ast;
      // Mark `after` hook for this rule as needing to be run
      afterHooksState.pendingStates[ruleIndex] = AFTER_HOOK_PENDING;
      afterHooksState.pendingCount++;
    };

    // Add `onCodePathEnd` CFG event handler to detect when this rule completes AST traversal.
    // This fires after all visit fns have been called (after `Program:exit`), and after all other CFG event handlers.
    type CodePathHandler = (this: Visitor, codePath: unknown, node: ESTreeNode) => void;

    const onCodePathEnd = visitor.onCodePathEnd as CodePathHandler | null | undefined;

    (visitor as unknown as { onCodePathEnd: CodePathHandler }).onCodePathEnd =
      onCodePathEnd == null
        ? function (this: Visitor, _codePath: unknown, node: ESTreeNode) {
            if (node === program) afterHooksState.ruleFinished();
          }
        : function (this: Visitor, codePath: unknown, node: ESTreeNode) {
            onCodePathEnd.call(this, codePath, node);
            if (node === program) afterHooksState.ruleFinished();
          };
  }

  return { context, visitor, beforeHook, setupAfterHook };
}
