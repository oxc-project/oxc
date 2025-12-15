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
