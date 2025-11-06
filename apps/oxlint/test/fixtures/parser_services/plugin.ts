import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'parser-services-plugin',
  },
  rules: {
    'check-parser-services': {
      create(context) {
        if (typeof context.sourceCode.parserServices?.defineTemplateBodyVisitor === 'function') {
        }

        return {};
      },
    },
  },
};

export default plugin;
