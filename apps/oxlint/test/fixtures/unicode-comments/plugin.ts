import type { Plugin, Rule } from '../../../dist/index.js';

const unicodeCommentsRule: Rule = {
  create(context) {
    const { sourceCode } = context;
    const { ast } = sourceCode;

    context.report({
      message: `getAllComments: ${JSON.stringify(
        sourceCode.getAllComments().map((c) => ({ type: c.type, value: c.value })),
        null,
        4,
      )}`,
      node: ast,
    });

    return {};
  },
};

const plugin: Plugin = {
  meta: {
    name: 'unicode-comments',
  },
  rules: {
    'unicode-comments': unicodeCommentsRule,
  },
};

export default plugin;
