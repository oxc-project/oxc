import type { Context } from './plugins/context.ts';
import type { Plugin, Rule } from './plugins/load.ts';

const { defineProperty, getPrototypeOf, setPrototypeOf } = Object;

const dummyOptions: unknown[] = [],
  dummyReport = () => {};

// Define a plugin.
export function definePlugin(plugin: Plugin): Plugin {
  return plugin;
}

// Define a rule.
// If rule has `createOnce` method, add an ESLint-compatible `create` method which delegates to `createOnce`.
export function defineRule(rule: Rule): Rule {
  if (!('createOnce' in rule)) return rule;
  if ('create' in rule) throw new Error('Rules must define only `create` or `createOnce` methods, not both');

  // Run `createOnce` with empty context object.
  // Really, `context` should be an instance of `Context`, which would throw error on accessing e.g. `id`
  // in body of `createOnce`. But any such bugs should have been caught when testing the rule in Oxlint,
  // so should be OK to take this shortcut.
  const context = Object.create(null, {
    id: { value: '', enumerable: true, configurable: true },
    options: { value: dummyOptions, enumerable: true, configurable: true },
    report: { value: dummyReport, enumerable: true, configurable: true },
  });

  const { before: beforeHook, after: afterHook, ...visitor } = rule.createOnce(context as Context);

  // Add `after` hook to `Program:exit` visit fn
  if (afterHook !== null) {
    const programExit = visitor['Program:exit'];
    visitor['Program:exit'] = programExit
      ? (node) => {
        programExit(node);
        afterHook();
      }
      : (_node) => afterHook();
  }

  // Create `create` function
  rule.create = (eslintContext) => {
    // Copy properties from ESLint's context object to `context`.
    // ESLint's context object is an object of form `{ id, options, report }`, with all other properties
    // and methods on another object which is its prototype.
    defineProperty(context, 'id', { value: eslintContext.id });
    defineProperty(context, 'options', { value: eslintContext.options });
    defineProperty(context, 'report', { value: eslintContext.report });
    setPrototypeOf(context, getPrototypeOf(eslintContext));

    // If `before` hook returns `false`, skip rest of traversal by returning an empty object as visitor
    if (beforeHook !== null) {
      const shouldRun = beforeHook();
      if (shouldRun === false) return {};
    }

    // Return same visitor each time
    return visitor;
  };

  return rule;
}
