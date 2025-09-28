import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'error-plugin',
  },
  rules: {
    error: {
      createOnce(_context) {
        return {
          after() {
            throw new Error('Whoops!');
          },
        };
      },
    },
  },
};

export default plugin;
