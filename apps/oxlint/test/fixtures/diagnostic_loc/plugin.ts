import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "loc-plugin",
  },
  rules: {
    "no-bugger": {
      create(context) {
        let debuggerCount = 0;
        return {
          Program(_node) {
            context.report({
              message: "Bugger debugger debug!",
              loc: {
                start: { line: 1, column: 2 },
                end: { line: 3, column: 5 },
              },
            });
          },
          DebuggerStatement(_node) {
            debuggerCount++;
            context.report({
              message: "Bugger!",
              loc: {
                start: { line: debuggerCount, column: 2 },
                end: { line: debuggerCount, column: 8 },
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
