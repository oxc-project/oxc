import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "eslint-plugin-aliased-settings-source",
  },
  rules: {
    "check-settings": {
      create(context) {
        return {
          Program(node) {
            const settings = context.settings["aliased-settings-source"];
            context.report({
              message: `settings: ${JSON.stringify(settings ?? "missing")}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
