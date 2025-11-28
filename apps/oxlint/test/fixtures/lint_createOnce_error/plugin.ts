import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: {
      createOnce(_context) {
        throw new Error("Whoops!");
      },
    },
  },
};

export default plugin;
