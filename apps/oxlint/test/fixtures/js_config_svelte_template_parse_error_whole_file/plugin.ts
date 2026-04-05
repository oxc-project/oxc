import type { Node, Plugin } from "#oxlint/plugins";

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
    name: "real-svelte",
  },
  rules: {
    "should-not-run": {
      create(context) {
        return {
          Program() {
            context.report({
              node: SPAN,
              message: "real parser unexpectedly accepted invalid template",
            });
          },
        };
      },
    },
  },
};

export default plugin;
