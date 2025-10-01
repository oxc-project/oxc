import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'test-plugin',
  },
  rules: {
    'no-var': {
      create(context) {
        return {
          VariableDeclaration(node) {
            if (node.kind === 'var') {
              context.report({
                message: 'Use let or const instead of var',
                node: node,
              });
            }
          },
        };
      },
    },
  },
};

export default plugin;
