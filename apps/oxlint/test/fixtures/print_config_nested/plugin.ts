import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "print-config-plugin",
  },
  rules: {
    "no-foo": {
      create() {
        return {};
      },
    },
    "no-bar": {
      create() {
        return {};
      },
    },
  },
};

export default plugin;
