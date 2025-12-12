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
    name: "globals-plugin",
  },
  rules: {
    globals: {
      create(context) {
        context.report({
          message: JSON.stringify(context.languageOptions.globals, null, 2),
          node: SPAN,
        });
        return {};
      },
    },
  },
};

export default plugin;
