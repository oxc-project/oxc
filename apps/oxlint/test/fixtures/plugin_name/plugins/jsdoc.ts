import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    // Namespace and name are overridden by alias in config
    name: "jsdoc",
    namespace: "custom-jsdoc",
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
