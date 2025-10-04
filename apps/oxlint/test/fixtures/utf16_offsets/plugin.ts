import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'utf16-plugin',
  },
  rules: {
    'no-debugger': {
      create(context) {
        return {
          Program(program) {
            context.report({
              message: 'program:\n' +
                `start/end: [${program.start},${program.end}]\n` +
                `range: [${program.range}]`,
              node: program,
            });
          },
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: 'debugger:\n' +
                `start/end: [${debuggerStatement.start},${debuggerStatement.end}]\n` +
                `range: [${debuggerStatement.range}]`,
              node: debuggerStatement,
            });
          },
        };
      },
    },
  },
};

export default plugin;
