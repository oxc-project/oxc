import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "globals-env-builtin-plugin",
  },
  rules: {
    "unresolved-globals": {
      create(context) {
        return {
          Program() {
            const { globalScope } = context.sourceCode.scopeManager;
            for (const ref of globalScope!.through) {
              context.report({
                message: `Unresolved global: ${ref.identifier.name}`,
                node: ref.identifier,
              });
            }
          },
        };
      },
    },
  },
};

export default plugin;
