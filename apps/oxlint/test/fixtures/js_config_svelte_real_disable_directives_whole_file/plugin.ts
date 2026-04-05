import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "real-svelte-disable",
  },
  rules: {
    "markup-visible": {
      create(context) {
        return {
          SvelteElement(node) {
            context.report({
              message: "whole-file custom-parser diagnostics should respect disable directives",
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
