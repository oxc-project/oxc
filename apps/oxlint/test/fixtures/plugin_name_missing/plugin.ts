import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  // No name defined
  rules: {
    rule: {
      create(context) {
        return {
          Program(node) {
            context.report({
              message: `filename: ${context.filename}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
