import { definePlugin } from '../../../dist/index.js';
import type { Node, Rule } from '../../../dist/index.js';

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const getAllCommentsRule: Rule = {
  create(context) {
    const comments = context.sourceCode.getAllComments();

    context.report({
      message: `getAllComments() returned ${comments.length} comments:\n` +
        comments.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: SPAN,
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
