const rule = {
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
};

const plugin = {
  meta: {
    name: "best-plugin-ever",
  },
  rules: {
    "no-debug": rule,
  },
};

export default plugin;
