import type { Plugin, Node } from "#oxlint/plugin";

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
    name: "parser-services-plugin",
  },
  rules: {
    "check-parser-services": {
      create(context) {
        context.report({
          message: `typeof context.sourceCode.parserServices: ${typeof context.sourceCode.parserServices}`,
          node: SPAN,
        });

        return {};
      },
    },
  },
};

export default plugin;
