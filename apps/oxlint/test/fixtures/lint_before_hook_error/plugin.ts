import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: {
      createOnce(_context) {
        return {
          before() {
            throw new Error("Whoops!");
          },
          Program(_program) {},
        };
      },
    },
  },
};

export default plugin;
