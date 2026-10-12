import assert from "node:assert/strict";
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
              context.report({
                message: "Range starts before source",
                loc: { start: { line: 1, column: -1 }, end: { line: 1, column: 1 } },
              });
              context.report({
                message: "Range ends before source",
                loc: { start: { line: 1, column: -2 }, end: { line: 1, column: -1 } },
              });
              context.report({
                message: "Negative shorthand location",
                loc: { line: 1, column: -1 },
              });
              context.report({
                message: "Negative column on later line",
                loc: { start: { line: 2, column: -2 }, end: { line: 2, column: -1 } },
              });
              context.report({
                message: "Range after source",
                loc: { start: { line: 1, column: 999 }, end: { line: 1, column: 1000 } },
              });
              context.report({ message: "Following finding", node: _node });

              assert.throws(
                () => context.report({ message: "Invalid line", loc: { line: -1, column: 0 } }),
                RangeError,
              );
              assert.throws(
                () => context.report({ message: "Invalid column", loc: { line: 1, column: NaN } }),
                TypeError,
              );
              assert.throws(
                () => context.sourceCode.getIndexFromLoc({ line: 1, column: -1 }),
                RangeError,
              );
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
