const plugin = {
  meta: {
    name: 'js-plugin',
  },
  rules: {
    'test-rule': {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: 'Workspace 2 Plugin',
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};


export default plugin;
