import type { Node, Plugin, Rule } from '../../../dist/index.js';

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const rule: Rule = {
  create(context) {
    context.report({ message: `id: ${context.id}`, node: SPAN });
    context.report({ message: `filename: ${context.filename}`, node: SPAN });
    context.report({ message: `physicalFilename: ${context.physicalFilename}`, node: SPAN });
    context.report({ message: `cwd: ${context.cwd}`, node: SPAN });

    if (this !== rule) context.report({ message: 'this !== rule', node: SPAN });

    return {
      VariableDeclaration(node) {
        if (this !== undefined) context.report({ message: 'this !== undefined', node });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'context-plugin',
  },
  rules: {
    'log-context': rule,
  },
};

export default plugin;
