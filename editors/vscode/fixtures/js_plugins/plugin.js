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
              message: 'Custom name JS Plugin Test Rule.',
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};


export default plugin;
