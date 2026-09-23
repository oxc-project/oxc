import type { Plugin } from "#oxlint/plugins";

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
            if (context.filename.endsWith("negative-loc.js")) {
              context.report({
                message: "Negative location",
                loc: { start: { line: 1, column: -1 } },
              });
              return;
            }

            if (context.filename.endsWith("negative-node.js")) {
              context.report({
                message: "Negative node",
                node: { range: [-1, 0] } as never,
              });
              return;
            }

            if (context.filename.endsWith("out-of-range-node.js")) {
              context.report({
                message: "Out-of-range node",
                node: { range: [999, 1000] } as never,
              });
              return;
            }

            context.report({
              message: "Misaligned location",
              loc: {
                start: { line: 1, column: 3 },
                end: { line: 1, column: 1 },
              },
            });
            context.report({
              message: "Bugger debugger debug!",
              loc: {
                start: { line: 1, column: 2 },
                end: { line: 3, column: 5 },
              },
            });
          },
          DebuggerStatement(_node) {
            if (context.filename.endsWith("out-of-range-node.js")) return;

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
