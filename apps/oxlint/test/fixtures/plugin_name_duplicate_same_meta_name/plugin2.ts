import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "shared-name",
  },
  rules: {
    "rule-two": {
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
