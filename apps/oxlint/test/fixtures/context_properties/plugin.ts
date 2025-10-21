import { sep } from 'node:path';

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

const DIR_PATH_LEN = import.meta.dirname.length + 1;

const relativePath = sep === '/'
  ? (path: string) => path.slice(DIR_PATH_LEN)
  : (path: string) => path.slice(DIR_PATH_LEN).replace(/\\/g, '/');

const rule: Rule = {
  create(context) {
    context.report({
      message: `id: ${context.id}`,
      node: SPAN,
    });

    context.report({
      message: `filename: ${relativePath(context.filename)}`,
      node: SPAN,
    });

    context.report({
      message: `physicalFilename: ${relativePath(context.physicalFilename)}`,
      node: SPAN,
    });

    if (context.cwd.length === 0) {
      context.report({ message: 'cwd.length === 0', node: SPAN });
    }

    if (context.cwd !== process.cwd()) {
      context.report({ message: 'cwd !== process.cwd()', node: SPAN });
    }

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
