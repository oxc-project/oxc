import type { Plugin } from "#oxlint/plugins";

export default {
  meta: { name: "hide-fixable" },
  rules: {
    check: {
      meta: { fixable: "code", hasSuggestions: true },
      create(context) {
        return {
          Identifier(node) {
            if (node.name === "fixable") {
              context.report({
                node,
                message: "Automatically fixable",
                fix: (fixer) => fixer.replaceText(node, "fixed"),
              });
            } else if (node.name === "suggestion") {
              context.report({
                node,
                message: "Needs a decision",
                suggest: [{ desc: "Rename", fix: (fixer) => fixer.replaceText(node, "renamed") }],
              });
            } else if (node.name === "noFix") {
              context.report({ node, message: "No fix available", fix: () => null });
            } else {
              context.report({ node, message: "Needs a manual fix" });
            }
          },
        };
      },
    },
  },
} satisfies Plugin;
