import type { Plugin } from "#oxlint/plugins";

// Plugin with no state shared between files, so output is the same whichever JS thread lints each file
const plugin: Plugin = {
  meta: {
    name: "js-threads-plugin",
  },
  rules: {
    "no-debugger": {
      meta: {
        fixable: "code",
      },
      create(context) {
        return {
          DebuggerStatement(node) {
            context.report({
              message: "Unexpected Debugger Statement",
              node,
              fix: (fixer) => fixer.remove(node),
            });
          },
        };
      },
    },
  },
};

export default plugin;
