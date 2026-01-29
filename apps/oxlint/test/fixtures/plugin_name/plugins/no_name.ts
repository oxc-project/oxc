import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  // No name defined
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
