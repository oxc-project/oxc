/*
 * `eslintCompatPlugin` function.
 * Converts an Oxlint plugin using `createOnce` to a plugin which will run in ESLint.
 */

import { debugAssertIsNonNull } from "../utils/asserts.ts";

import type { Context, FileContext, LanguageOptions } from "../plugins/context.ts";
import type { CreateOnceRule, Plugin, Rule } from "../plugins/load.ts";
import type { Settings } from "../plugins/settings.ts";
import type { SourceCode } from "../plugins/source_code.ts";
import type { BeforeHook, Visitor, VisitorWithHooks } from "../plugins/types.ts";
import type { Program, Node as ESTreeNode } from "../generated/types.d.ts";
import type { SetNullable } from "../utils/types.ts";

// Empty visitor object, returned by `create` when `before` hook returns `false`.
const EMPTY_VISITOR: Visitor = {};

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

  // Make each rule in the plugin ESLint-compatible by calling `convertRule` on it
  for (const ruleName in rules) {
    if (Object.hasOwn(rules, ruleName)) convertRule(rules[ruleName]);
  }

  return plugin;
}

/**
 * Convert a rule.
 *
 * The `rule` object passed in is mutated in-place.
 *
 * @param rule - Rule to convert
 * @throws {Error} If `rule` is not an object
 */
function convertRule(rule: Rule) {
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
      ({ context, visitor, beforeHook, setupAfterHook } = createContextAndVisitor(rule));
    }
    debugAssertIsNonNull(visitor);

    // Copy properties from ESLint's context object to `context`.
    // ESLint's context object is an object of form `{ id, options, report }`, with all other properties
    // and methods on another object which is its prototype.
    Object.defineProperties(context, {
      id: { value: eslintContext.id },
      options: { value: eslintContext.options },
      report: { value: eslintContext.report },
    });
    const eslintFileContext = Object.getPrototypeOf(eslintContext);
    Object.setPrototypeOf(context, eslintFileContext);

    // If `before` hook returns `false`, skip traversal by returning an empty object as visitor
    if (beforeHook !== null) {
      const shouldRun = beforeHook();
      if (shouldRun === false) return EMPTY_VISITOR;
    }

    // If there's an `after` hook, call `setupAfterHook` with the `Program` node of the current file
    if (setupAfterHook !== null) setupAfterHook(eslintFileContext.sourceCode.ast);

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
 * @returns Object with `context`, `visitor`, and `beforeHook` properties,
 *   and `setupAfterHook` function if visitor has an `after` hook
 */
function createContextAndVisitor(rule: CreateOnceRule): {
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

    // Pass function to populate `program` var back out to the `create` function generated by `convertRule`.
    // `create` will call this function at the start of linting each file.
    // Having `program` in a local var makes the `node === program` check in `onCodePathEnd` as cheap as it can be.
    // Otherwise it'd have to be `node === context.sourceCode.ast`, which would be slower.
    setupAfterHook = (ast: Program) => {
      program = ast;
    };

    // Add `onCodePathEnd` CFG event handler to run `after` hook at end of AST traversal.
    // This fires after all visit fns have been called (after `Program:exit`), and after all other CFG event handlers.
    type CodePathHandler = (this: Visitor, codePath: unknown, node: ESTreeNode) => void;

    const onCodePathEnd = visitor.onCodePathEnd as CodePathHandler | null | undefined;

    (visitor as unknown as { onCodePathEnd: CodePathHandler }).onCodePathEnd =
      onCodePathEnd == null
        ? function (this: Visitor, _codePath: unknown, node: ESTreeNode) {
            if (node === program) {
              program = null;
              afterHook();
            }
          }
        : function (this: Visitor, codePath: unknown, node: ESTreeNode) {
            onCodePathEnd.call(this, codePath, node);

            if (node === program) {
              program = null;
              afterHook();
            }
          };
  }

  return { context, visitor, beforeHook, setupAfterHook };
}
