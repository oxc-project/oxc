import type { Plugin, Node } from '../../../dist/index.js';

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const plugin: Plugin = {
  meta: {
    name: 'language-options-plugin',
  },
  rules: {
    lang: {
      create(context) {
        const { languageOptions } = context;

        context.report({
          message:
            'languageOptions:\n' +
            `sourceType: ${languageOptions.sourceType}\n` +
            `ecmaVersion: ${languageOptions.ecmaVersion}\n` +
            `parserOptions: ${JSON.stringify(languageOptions.parserOptions)}\n` +
            `globals: ${JSON.stringify(languageOptions.globals)}`,
          node: SPAN,
        });

        return {};
      },
    },
  },
};

export default plugin;
