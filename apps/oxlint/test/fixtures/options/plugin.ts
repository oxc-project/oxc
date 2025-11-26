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
    options: {
      create(context) {
        context.report({
          message: `options: ${JSON.stringify(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },
    "default-options": {
      meta: {
        defaultOptions: ["string", 123, true, { toBe: false, notToBe: true }],
      },
      create(context) {
        context.report({
          message: `options: ${JSON.stringify(context.options)}`,
          node: SPAN,
        });
        return {};
      },
    },
  },
};

export default plugin;
