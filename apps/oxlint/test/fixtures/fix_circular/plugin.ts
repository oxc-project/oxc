import type { Plugin } from "#oxlint/plugins";

// Ported from ESLint's "should stop fixing if a circular fix is detected" test.
// Each rule's fix undoes the other's, so fixing stops when the text returns to what it was 2 passes earlier.
const plugin: Plugin = {
  meta: {
    name: "fix-circular-plugin",
  },
  rules: {
    "add-leading-hyphen": {
      meta: {
        fixable: "whitespace",
      },
      create(context) {
        return {
          Program(node) {
            if (context.sourceCode.getText(node).startsWith("-")) return;
            context.report({
              message: "Add leading hyphen.",
              node,
              fix(fixer) {
                return fixer.insertTextBefore(node, "-");
              },
            });
          },
        };
      },
    },
    "remove-leading-hyphen": {
      meta: {
        fixable: "whitespace",
      },
      create(context) {
        return {
          Program(node) {
            if (!context.sourceCode.getText(node).startsWith("-")) return;
            context.report({
              message: "Remove leading hyphen.",
              node,
              fix(fixer) {
                return fixer.removeRange([0, 1]);
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
