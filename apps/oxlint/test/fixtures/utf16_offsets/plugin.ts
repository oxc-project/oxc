import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'utf16-plugin',
  },
  rules: {
    'no-debugger': {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: `Debugger at ${debuggerStatement.start}-${debuggerStatement.end}`,
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};

export default plugin;
