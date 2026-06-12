import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "print-config-plugin",
  },
  rules: {
    "with-options": {
      create() {
        return {};
      },
    },
  },
};

export default plugin;
