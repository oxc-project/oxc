export default {
  meta: {
    name: "plugin7",
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
