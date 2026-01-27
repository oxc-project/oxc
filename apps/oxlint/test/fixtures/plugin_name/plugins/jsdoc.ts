import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    // Name is overridden by alias in config
    name: "jsdoc",
  },
  rules: {
    rule: {
      create(context) {
        return {
          FunctionDeclaration(node) {
            context.report({
              message: `id: ${context.id}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
