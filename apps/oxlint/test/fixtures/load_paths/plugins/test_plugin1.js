export default {
  meta: {
    name: 'plugin1',
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
