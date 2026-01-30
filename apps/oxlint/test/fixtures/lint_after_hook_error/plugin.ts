import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "error-plugin",
  },
  rules: {
    error: {
      createOnce(_context) {
        return {
          Program(_program) {},
          after() {
            throw new Error("Whoops!");
          },
        };
      },
    },
  },
};

export default plugin;
