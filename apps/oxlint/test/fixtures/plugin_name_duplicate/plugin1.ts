import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: { name: "@my" },
  rules: {
    rule: {
      create(context) {
        return {
          DebuggerStatement(node) {
            context.report({ message: "plugin 1", node });
          },
        };
      },
    },
  },
};

export default plugin;
