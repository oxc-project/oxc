import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "basic-custom-plugin",
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
    "no-debugger-2": {
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
    "no-identifiers-named-foo": {
      create(context) {
        return {
          Identifier(ident) {
            if (ident.name == "foo") {
              context.report({ message: "Unexpected Identifier named foo", node: ident });
            }
          },
        };
      },
    },
  },
};

export default plugin;
