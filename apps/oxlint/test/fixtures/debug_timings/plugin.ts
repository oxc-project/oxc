import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "timing-plugin",
  },
  rules: {
    "count-identifiers": {
      create() {
        return {
          Identifier() {},
          onCodePathStart() {},
        };
      },
    },
  },
};

export default plugin;
