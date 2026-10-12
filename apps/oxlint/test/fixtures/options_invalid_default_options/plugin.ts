import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "options-invalid-default-options-plugin",
  },
  rules: {
    options: {
      meta: {
        schema: [{ enum: ["always", "never"] }],
        defaultOptions: ["something_else"],
      },
      create(_context) {
        return {};
      },
    },
  },
};

export default plugin;
