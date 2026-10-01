import type { Plugin } from "#oxlint/plugins";

// Reports envs, and global references which are not resolved to a variable
const plugin: Plugin = {
  meta: {
    name: "globals-env-builtin-plugin",
  },
  rules: {
    "unresolved-globals": {
      create(context) {
        return {
          Program(node) {
            const { through } = context.sourceCode.scopeManager.globalScope!;
            const unresolved = through.map((ref) => ref.identifier.name).join(", ") || "none";
            context.report({
              message: `env: ${JSON.stringify(context.languageOptions.env)}, unresolved: ${unresolved}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
