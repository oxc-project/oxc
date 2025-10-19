import type { Context } from './plugins/context.ts';
import type { CreateOnceRule, Plugin, Rule } from './plugins/load.ts';
import type { BeforeHook, Visitor, VisitorWithHooks } from './plugins/types.ts';

export type * as ESTree from './generated/types.d.ts';
export type { Context, Diagnostic, DiagnosticBase, DiagnosticWithLoc, DiagnosticWithNode } from './plugins/context.ts';
export type { Fix, Fixer, FixFn } from './plugins/fix.ts';
export type { CreateOnceRule, CreateRule, Plugin, Rule } from './plugins/load.ts';
export type {
  Definition,
  DefinitionType,
  Reference,
  Scope,
  ScopeManager,
  ScopeType,
  Variable,
} from './plugins/scope.ts';
export type { SourceCode } from './plugins/source_code.ts';
export type { CountOptions, FilterFn, RangeOptions, SkipOptions } from './plugins/tokens.ts';
export type {
  AfterHook,
  BeforeHook,
  Comment,
  LineColumn,
  Location,
  Node,
  NodeOrToken,
  Range,
  Ranged,
  RuleMeta,
  Span,
  Token,
  Visitor,
  VisitorWithHooks,
} from './plugins/types.ts';

const { defineProperty, getPrototypeOf, hasOwn, setPrototypeOf, create: ObjectCreate } = Object;

/**
 * Define a plugin.
 *
 * If any of the plugin's rules use the Oxlint alternative `createOnce` API,
 * add ESLint-compatible `create` methods to those rules, which delegate to `createOnce`.
 * This makes the plugin compatible with ESLint.
 *
 * The `plugin` object passed in is mutated in-place.
 *
 * @param plugin - Plugin to define
 * @returns Plugin with all rules having `create` method
 * @throws {Error} If `plugin` is not an object, or `plugin.rules` is not an object
 */
export function definePlugin(plugin: Plugin): Plugin {
  // Validate type of `plugin`
  if (plugin === null || typeof plugin !== 'object') throw new Error('Plugin must be an object');

  const { rules } = plugin;
  if (rules === null || typeof rules !== 'object') throw new Error('Plugin must have an object as `rules` property');

  // Make each rule in the plugin ESLint-compatible by calling `defineRule` on it
  for (const ruleName in rules) {
    if (hasOwn(rules, ruleName)) {
      rules[ruleName] = defineRule(rules[ruleName]);
    }
  }

  return plugin;
}

/**
 * Define a rule.
 *
 * If `rule` uses the Oxlint alternative `createOnce` API, add an ESLint-compatible
 * `create` method to the rule, which delegates to `createOnce`.
 * This makes the rule compatible with ESLint.
 *
 * The `rule` object passed in is mutated in-place.
 *
 * @param rule - Rule to define
 * @returns Rule with `create` method
 * @throws {Error} If `rule` is not an object
 */
export function defineRule(rule: Rule): Rule {
  // Validate type of `rule`
  if (rule === null || typeof rule !== 'object') throw new Error('Rule must be an object');

  // If rule already has `create` method, return it as is
  if ('create' in rule) return rule;

  // Add `create` function to `rule`
  let context: Context = null, visitor: Visitor, beforeHook: BeforeHook | null;

  rule.create = (eslintContext) => {
    // Lazily call `createOnce` on first invocation of `create`
    if (context === null) {
      ({ context, visitor, beforeHook } = createContextAndVisitor(rule));
    }

    // Copy properties from ESLint's context object to `context`.
    // ESLint's context object is an object of form `{ id, options, report }`, with all other properties
    // and methods on another object which is its prototype.
    defineProperty(context, 'id', { value: eslintContext.id });
    defineProperty(context, 'options', { value: eslintContext.options });
    defineProperty(context, 'report', { value: eslintContext.report });
    setPrototypeOf(context, getPrototypeOf(eslintContext));

    // If `before` hook returns `false`, skip traversal by returning an empty object as visitor
    if (beforeHook !== null) {
      const shouldRun = beforeHook();
      if (shouldRun === false) return {};
    }

    // Return same visitor each time
    return visitor;
  };

  return rule;
}

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
  if (createOnce == null) throw new Error('Rules must define either a `create` or `createOnce` method');
  if (typeof createOnce !== 'function') throw new Error('Rule `createOnce` property must be a function');

  // Call `createOnce` with empty context object.
  // Really, `context` should be an instance of `Context`, which would throw error on accessing e.g. `id`
  // in body of `createOnce`. But any such bugs should have been caught when testing the rule in Oxlint,
  // so should be OK to take this shortcut.
  const context = ObjectCreate(null, {
    id: { value: '', enumerable: true, configurable: true },
    options: { value: null, enumerable: true, configurable: true },
    report: { value: null, enumerable: true, configurable: true },
  });

  let { before: beforeHook, after: afterHook, ...visitor } = createOnce.call(rule, context) as VisitorWithHooks;

  if (beforeHook === void 0) {
    beforeHook = null;
  } else if (beforeHook !== null && typeof beforeHook !== 'function') {
    throw new Error('`before` property of visitor must be a function if defined');
  }

  // Add `after` hook to `Program:exit` visit fn
  if (afterHook != null) {
    if (typeof afterHook !== 'function') throw new Error('`after` property of visitor must be a function if defined');

    const programExit = visitor['Program:exit'];
    visitor['Program:exit'] = programExit == null
      ? (_node) => afterHook()
      : (node) => {
        programExit(node);
        afterHook();
      };
  }

  return { context, visitor, beforeHook };
}
