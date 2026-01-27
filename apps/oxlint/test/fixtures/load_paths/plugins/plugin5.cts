"use strict";

import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "plugin5",
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

module.exports = plugin;
