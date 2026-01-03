import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "first-plugin",
  },
  rules: {
    "rule-one": {
      meta: {
        type: "problem",
      },
      create(context) {
        return {};
      },
    },
  },
};

export default plugin;
