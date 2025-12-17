import type { Plugin } from "../../../dist/index.js";

const plugin: Plugin = {
  meta: {
    name: "jsdoc",
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
