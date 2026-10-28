import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "test-plugin",
  },
  rules: {
    // Enables the JS plugin execution path required to reproduce issue #25892.
    noop: {
      create() {
        return {};
      },
    },
  },
};

export default plugin;
