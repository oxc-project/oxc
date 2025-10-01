import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      create(context) {
        return {
          Identifier(node) {
            context.report({
              message: 'Identifier found',
              node,
              fix() {
                throw new Error('Whoops!');
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
