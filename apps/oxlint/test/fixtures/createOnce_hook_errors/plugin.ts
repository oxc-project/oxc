import { sep as pathSep } from "node:path";

import type { Node, Plugin, Rule, Context } from "#oxlint/plugins";

// Aim of this test is to check:
//
// 1. Errors thrown during `before` and `after` hooks are handled correctly and shown to user as diagnostics.
//
// 2. All rules whose `before` hooks runs have their `after` hook run even if an error is thrown
//    during other rules' hooks, or during AST visitation.
//
// 3. Global state is reset after an error in hooks, so it's in correct initial state when Oxlint starts linting
//    the next file.
//
// The last two are tricky to test because usually the order Oxlint lints files in is non-deterministic.
// To make this test deterministic, we run it with `oxlint --threads 1`
// (`options.json` file for this fixture contains `"singleThread": true`).
// This guarantees that files are linted in alphabetical order.
//
// In tests, rules are run in the order they're defined in the plugin, so we can control at what point
// errors are thrown.

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

// Record of events per file.
// Key of outer `Map` is directory name.
// Key of inner object is file name.
// Value of inner object is array of events.
const events = new Map<string, Record<string, string[]>>();

/**
 * Record an event.
 *
 * @param context - Context object
 * @param event - Event name
 */
function addEvent(context: Context, event: string) {
  const pathParts = context.filename.split(pathSep);
  const dirname = pathParts.at(-2)!;
  const filename = pathParts.at(-1)!;

  let dir = events.get(dirname);
  if (!dir) {
    dir = {};
    events.set(dirname, dir);
  }

  let fileEvents = dir[filename];
  if (!fileEvents) {
    fileEvents = [];
    dir[filename] = fileEvents;
  }

  const ruleName = context.id.split("/")[1];
  fileEvents.push(`${event}: ${ruleName}`);
}

/**
 * Get recorded events for a directory, serialized as pretty-printed JSON.
 *
 * @param path - File path
 * @returns Events for directory as JSON string
 */
function getEvents(path: string): string {
  const dirname = path.split(pathSep).at(-2)!;
  return JSON.stringify(events.get(dirname)!, null, 2);
}

// Rule which throws in `before` hook
const throwInBeforeRule: Rule = {
  createOnce(context) {
    return {
      before() {
        addEvent(context, "before");

        throw new Error("`before` hook threw");
      },
      Identifier(_node) {
        addEvent(context, "visit");
      },
      after() {
        // Should not be called because `before` hook threw
        addEvent(context, "after");
      },
    };
  },
};

// Rule which throws in `after` hook
const throwInAfterRule: Rule = {
  createOnce(context) {
    return {
      before() {
        addEvent(context, "before");
      },
      Identifier(_node) {
        addEvent(context, "visit");
      },
      after() {
        addEvent(context, "after");

        throw new Error("`after` hook threw");
      },
    };
  },
};

// Rule which throws during AST visitation
const throwInVisitRule: Rule = {
  createOnce(context) {
    return {
      before() {
        addEvent(context, "before");
      },
      Identifier(_node) {
        addEvent(context, "visit");

        throw new Error("`Identifier` visit function threw");
      },
      after() {
        // Should be called because `before` hook succeeded
        addEvent(context, "after");
      },
    };
  },
};

// Rule which records events in `before` and `after` hooks, and during AST visitation.
// This rule is run before the rules which throw.
const beforeAndAfterRule: Rule = {
  createOnce(context) {
    return {
      before() {
        addEvent(context, "before");
      },
      Identifier(_node) {
        addEvent(context, "visit");
      },
      after() {
        addEvent(context, "after");
      },
    };
  },
};

// Rule which records events in `before` and `after` hooks, and during AST visitation (like `beforeAndAfterRule`).
// `after` hook creates a diagnostic containing all events for the directory.
// This rule is run after the rules which throw.
const beforeAndAfterLateRule: Rule = {
  createOnce(context) {
    return {
      before() {
        addEvent(context, "before");
      },
      Identifier(_node) {
        addEvent(context, "visit");
      },
      after() {
        // Should be called unless `before` hook of another rule (`throwInBeforeRule`) threw in its `before` hook
        // so this rule's `before` hook was not run either
        addEvent(context, "after");

        context.report({
          message:
            "after hook:\n" +
            `id: ${context.id}\n` +
            `filename: ${context.filename}\n` +
            `events: ${getEvents(context.filename)}`,
          node: SPAN,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "create-once-errors-plugin",
  },
  rules: {
    "before-and-after": beforeAndAfterRule,
    "throw-in-before": throwInBeforeRule,
    "throw-in-after": throwInAfterRule,
    "throw-in-visit": throwInVisitRule,
    "before-and-after-late": beforeAndAfterLateRule,
  },
};

export default plugin;
