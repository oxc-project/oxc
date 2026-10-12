import type { Node, Plugin, Rule } from "#oxlint/plugins";

// Schemas and default options are only validated for rules which are enabled.
// Rules which are not enabled in config can have invalid default options, or a schema which fails to compile,
// without causing an error.

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const reportOptionsRule: Rule = {
  meta: {
    schema: [
      {
        type: "object",
        properties: {
          fromSchema: { type: "number", default: 10 },
          fromDefault: { type: "number" },
          fromConfig: { type: "number" },
        },
        additionalProperties: false,
      },
    ],
    defaultOptions: [{ fromDefault: 20 }],
  },
  create(context) {
    context.report({
      message:
        `\noptions: ${JSON.stringify(context.options, null, 2)}\n`
        + `isDeepFrozen: ${isDeepFrozen(context.options)}`,
      node: SPAN,
    });
    return {};
  },
};

const plugin: Plugin = {
  meta: {
    name: "options-unused-rules-plugin",
  },
  rules: {
    "default-options": reportOptionsRule,
    "config-options": reportOptionsRule,

    // Not enabled. Default options do not match schema.
    "invalid-default-options": {
      meta: {
        schema: [{ enum: ["always", "never"] }],
        defaultOptions: ["something_else"],
      },
      create(_context) {
        return {};
      },
    },

    // Not enabled. Schema contains an invalid regexp, so throws if it's compiled.
    "invalid-schema": {
      meta: {
        schema: [{ type: "string", pattern: "[" }],
      },
      create(_context) {
        return {};
      },
    },
  },
};

export default plugin;

function isDeepFrozen(value: unknown): boolean {
  if (value === null || typeof value !== "object") return true;
  if (!Object.isFrozen(value)) return false;
  if (Array.isArray(value)) return value.every(isDeepFrozen);
  return Object.values(value).every(isDeepFrozen);
}
