import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "real-svelte-unused-disable",
  },
  rules: {
    noop: {
      create() {
        return {};
      },
    },
  },
};

export default plugin;
