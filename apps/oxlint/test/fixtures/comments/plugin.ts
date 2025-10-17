import { definePlugin } from '../../../dist/index.js';
import type { Rule } from '../../../dist/index.js';

const testCommentsRule: Rule = {
  create(context) {
    const { sourceCode } = context;
    const { ast } = sourceCode;
    const { body } = ast;

    const comments = sourceCode.getAllComments();
    context.report({
      message: `getAllComments() returned ${comments.length} comments:\n` +
        comments.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: context.sourceCode.ast,
    });

    const constX = body.find((n) => n.type === 'VariableDeclaration');
    const before = sourceCode.getCommentsBefore(constX);
    const after = sourceCode.getCommentsAfter(constX);
    context.report({
      message: `getCommentsBefore(x) returned ${before.length} comments:\n` +
        before.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: constX,
    });
    context.report({
      message: `getCommentsAfter(x) returned ${after.length} comments:\n` +
        after.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: constX,
    });

    const functionFoo = body.find((n) => n.type === 'ExportNamedDeclaration');
    const insideFn = sourceCode.getCommentsInside(functionFoo);
    context.report({
      message: `getCommentsInside(foo) returned ${insideFn.length} comments:\n` +
        insideFn.map((c, i) => `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`).join(
          '\n',
        ),
      node: functionFoo,
    });

    const commentsBetween = sourceCode.commentsExistBetween(constX, functionFoo);
    context.report({
      message: `commentsExistBetween(x, foo): ${commentsBetween}`,
      node: constX,
    });

    return {};
  },
};

export default definePlugin({
  meta: {
    name: 'test-comments',
  },
  rules: {
    'test-comments': testCommentsRule,
  },
});
