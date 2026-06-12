import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "negative-loc",
  },
  rules: {
    "no-bugger": {
      create(context) {
        return {
          Program() {
            context.report({
              message: "Column before file start!",
              loc: {
                start: { line: 1, column: -1 },
                end: { line: 1, column: 1 },
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
