import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "@tanstack/query",
  },
  rules: {
    "exhaustive-deps": {
      create(context) {
        return {
          CallExpression(callExpression) {
            const callee = callExpression.callee;
            if (callee.type === "Identifier" && callee.name === "useQuery") {
              context.report({
                message: "useQuery must have exhaustive dependencies",
                node: callExpression,
              });
            }
          },
        };
      },
    },
  },
};

export default plugin;
