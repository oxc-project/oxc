import type { Plugin } from "#oxlint/plugins";

// Ported from ESLint's "stops fixing after 10 passes" test.
// This rule's fix never satisfies the rule, so fixes are applied until the maximum number of passes is reached.
const plugin: Plugin = {
  meta: {
    name: "fix-max-passes-plugin",
  },
  rules: {
    "add-spaces": {
      meta: {
        fixable: "whitespace",
      },
      create(context) {
        return {
          Program(node) {
            context.report({
              message: "Add a space before this node.",
              node,
              fix(fixer) {
                return fixer.insertTextBefore(node, " ");
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
