import { definePlugin } from '../../../dist/index.js';
import type { Rule } from '../../../dist/index.js';

const getAllCommentsRule: Rule = {
  create(context) {
    const comments = context.sourceCode.getAllComments();

    context.report({
      message: `getAllComments() returned ${comments.length} comments:\n` +
        comments.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: context.sourceCode.ast,
    });

    return {};
  },
};

export default definePlugin({
  meta: {
    name: 'test-getAllComments',
  },
  rules: {
    'test-getAllComments': getAllCommentsRule,
  },
});
