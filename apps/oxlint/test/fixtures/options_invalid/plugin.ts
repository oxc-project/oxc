import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "options-invalid-plugin",
  },
  rules: {
    options: {
      meta: {
        schema: [
          {
            enum: ["always", "never"],
          },
        ],
      },
      create(_context) {
        return {};
      },
    },
  },
};

export default plugin;
