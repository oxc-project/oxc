import type { Node, Plugin } from "#oxlint";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const plugin: Plugin = {
  meta: {
    name: "options-plugin",
  },
  rules: {
    "no-options": {
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    options: {
      meta: {
        schema: false,
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    "default-options": {
      meta: {
        defaultOptions: [
          "string",
          123,
          true,
          { toBe: false, notToBe: true },
          { deep: [{ deeper: { evenDeeper: [{ soDeep: { soSoDeep: true } }] } }] },
        ],
        schema: false,
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    "merge-options": {
      meta: {
        defaultOptions: [
          { fromDefault: 1, overrideDefault: 2, nested: { fromDefault: 3, overrideDefault: 4 } },
          { fromDefault: 5 },
          { fromDefault: 6 },
          7,
        ],
        schema: false,
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    "empty-default-options": {
      meta: {
        defaultOptions: [],
        schema: false,
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    // Rule with schema defaults only (no `defaultOptions`)
    "schema-defaults": {
      meta: {
        schema: [
          {
            type: "object",
            default: {},
            properties: {
              fromSchema: { type: "number", default: 10 },
              overrideSchema: { type: "number", default: 20 },
            },
            additionalProperties: true,
          },
          {
            type: "string",
            default: "schema-default-string",
          },
        ],
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    // Rule with both schema defaults AND `defaultOptions`.
    // Order: `defaultOptions` merged first, then schema defaults applied after.
    "schema-and-default-options": {
      meta: {
        schema: [
          {
            type: "object",
            default: {},
            properties: {
              fromSchema: { type: "number", default: 10 },
              overrideSchemaByDefaultOptions: { type: "number", default: 20 },
              overrideSchemaByConfig: { type: "number", default: 30 },
              overrideBothByConfig: { type: "number", default: 40 },
            },
            additionalProperties: true,
          },
        ],
        defaultOptions: [
          {
            fromDefaultOptions: 51,
            overrideDefaultOptionsByConfig: 61,
            overrideSchemaByDefaultOptions: 21,
            overrideBothByConfig: 41,
          },
        ],
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },

    // Rule with both schema defaults AND `defaultOptions`, with `defaultOptions` empty
    "schema-and-empty-default-options": {
      meta: {
        schema: [
          {
            type: "object",
            default: {},
            properties: {
              fromSchema: { type: "number", default: 10 },
              overrideSchemaByDefaultOptions: { type: "number", default: 20 },
              overrideSchemaByConfig: { type: "number", default: 30 },
              overrideBothByConfig: { type: "number", default: 40 },
            },
            additionalProperties: true,
          },
        ],
        defaultOptions: [],
      },
      create(context) {
        context.report({
          message:
            `\noptions: ${JSON.stringify(context.options, null, 2)}\n` +
            `isDeepFrozen: ${isDeepFrozen(context.options)}`,
          node: SPAN,
        });
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
