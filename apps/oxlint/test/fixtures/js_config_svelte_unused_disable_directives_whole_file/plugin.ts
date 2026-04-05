import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte-unused-disable",
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
