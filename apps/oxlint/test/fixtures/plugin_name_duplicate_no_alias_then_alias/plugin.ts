import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "foo",
  },
  rules: {
    "test-rule": {
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
