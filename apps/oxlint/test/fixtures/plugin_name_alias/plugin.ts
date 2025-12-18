import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "legal-plugin-name",
  },
  rules: {
    "no-debugger": {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: "Unexpected Debugger Statement",
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};

export default plugin;
