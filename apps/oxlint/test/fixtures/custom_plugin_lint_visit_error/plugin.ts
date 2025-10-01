import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      create(_context) {
        return {
          Identifier(_node) {
            throw new Error('Whoops!');
          },
        };
      },
    },
  },
};

export default plugin;
