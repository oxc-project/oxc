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
    beforeHook: BeforeHook | null;

  rule.create = (eslintContext) => {
    // Lazily call `createOnce` on first invocation of `create`
    if (context === null) {
      ({ context, visitor, beforeHook } = createContextAndVisitor(rule));
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
    Object.setPrototypeOf(context, Object.getPrototypeOf(eslintContext));

    // If `before` hook returns `false`, skip traversal by returning an empty object as visitor
    if (beforeHook !== null) {
      const shouldRun = beforeHook();
      if (shouldRun === false) return EMPTY_VISITOR;
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
 * @returns Object with `context`, `visitor`, and `beforeHook` properties
 */
function createContextAndVisitor(rule: CreateOnceRule): {
  context: Context;
  visitor: Visitor;
  beforeHook: BeforeHook | null;
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
  // Really, accessing `options` or calling `report` should throw, because they're illegal in `createOnce`.
  // But any such bugs should have been caught when testing the rule in Oxlint, so should be OK to take this shortcut.
  // `FILE_CONTEXT` prototype provides `extends` method, which is available in `createOnce`.
  const context: Context = Object.create(FILE_CONTEXT, {
    id: { value: "", enumerable: true, configurable: true },
    options: { value: null, enumerable: true, configurable: true },
    report: { value: null, enumerable: true, configurable: true },
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

  // Add `after` hook to `Program:exit` visit fn
  if (afterHook != null) {
    if (typeof afterHook !== "function") {
      throw new Error("`after` property of visitor must be a function if defined");
    }

    // We need to make sure that `after` hook is called after all visit fns have been called.
    // Usually this is done by adding a `Program:exit` visit fn, but there's an odd edge case:
    // Other visit fns could be called after `Program:exit` if they're selectors with a higher specificity.
    // e.g. `[body]:exit` would match `Program`, but has higher specificity than `Program:exit`, so would run last.
    //
    // We don't want to parse every visitor key here to calculate their specificity, so we take a shortcut.
    // Selectors which have highest specificity are of types `attribute`, `field`, `nth-child`, and `nth-last-child`.
    //
    // Examples of selectors of these types:
    // * `[id]` (attribute)
    // * `.id` (field)
    // * `:first-child` (nth-child)
    // * `:nth-child(2)` (nth-child)
    // * `:last-child` (nth-last-child)
    // * `:nth-last-child(2)` (nth-last-child)
    //
    // All these contain the characters `[`, `.`, or `:`. So just count these characters in all visitor keys, and create
    // a selector which always matches `Program`, but with a higher specificity than the most specific exit selector.
    //
    // e.g. If visitor has key `[id]:first-child:exit`, that contains 2 special characters (not including `:exit`).
    // So we use a selector `Program[type][type][type]:exit` (3 attributes = more specific than 2).
    //
    // ESLint will recognise that this `Program[type][type][type]` selector can only match `Program` nodes,
    // and will only execute it only on `Program` node. So the additional cost of checking if the selector matches
    // is only paid once per file - insignificant impact on performance.
    // `nodeTypes` for this selector is `["Program"]`, so it only gets added to `exitSelectorsByNodeType` for `Program`.
    // https://github.com/eslint/eslint/blob/4cecf8393ae9af18c4cfd50621115eb23b3d0cb6/lib/linter/esquery.js#L143-L231
    // https://github.com/eslint/eslint/blob/4cecf8393ae9af18c4cfd50621115eb23b3d0cb6/lib/linter/source-code-traverser.js#L93-L125
    //
    // This is blunt tool. We may well create a selector which has a higher specificity than we need.
    // But that doesn't really matter - as long as it's specific *enough*, it'll work correctly.
    const CHAR_CODE_BRACKET = "[".charCodeAt(0);
    const CHAR_CODE_DOT = ".".charCodeAt(0);
    const CHAR_CODE_COLON = ":".charCodeAt(0);

    let maxAttrs = -1;
    for (const key in visitor) {
      if (!Object.hasOwn(visitor, key)) continue;

      // Only `:exit` visit functions matter here
      if (!key.endsWith(":exit")) continue;

      const end = key.length - ":exit".length;
      let count = 0;
      for (let i = 0; i < end; i++) {
        const c = key.charCodeAt(i);
        if (c === CHAR_CODE_BRACKET || c === CHAR_CODE_DOT || c === CHAR_CODE_COLON) count++;
      }
      if (count > maxAttrs) maxAttrs = count;
    }

    const key = `Program${"[type]".repeat(maxAttrs + 1)}:exit`;
    visitor[key] = (_node) => afterHook();
  }

  return { context, visitor, beforeHook };
}
