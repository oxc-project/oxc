const plugin = {
  meta: {
    name: 'js-plugin',
  },
  rules: {
    'no-debugger': {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: 'Unexpected Debugger Statement',
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};

export default plugin;
