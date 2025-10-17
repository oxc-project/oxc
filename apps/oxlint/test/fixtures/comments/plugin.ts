import assert from 'node:assert';
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

    const [topLevelVariable1, topLevelVariable2, topLevelFunctionExport, topLevelVariable3] = body;
    assert(topLevelFunctionExport.type === 'ExportNamedDeclaration');
    const topLevelFunction = topLevelFunctionExport.declaration;
    assert(topLevelFunction.type === 'FunctionDeclaration');
    const [functionScopedVariable] = topLevelFunction.body.body;

    const commentsBetween = sourceCode.commentsExistBetween(topLevelVariable2, topLevelFunction);
    context.report({
      message: `commentsExistBetween(topLevelVariable, topLevelFunction): ${commentsBetween}`,
      node: topLevelVariable2,
    });

    const commentsBeforeFunctionScopedVariable = sourceCode.getCommentsBefore(functionScopedVariable);
    context.report({
      message:
        `getCommentsBefore(functionScopedVariable) returned ${commentsBeforeFunctionScopedVariable.length} comments:\n` +
        commentsBeforeFunctionScopedVariable.map((c, i) =>
          `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`
        ).join(
          '\n',
        ),
      node: functionScopedVariable,
    });

    const commentsBeforeTopLevelVariable1 = sourceCode.getCommentsBefore(topLevelVariable1);
    context.report({
      message: `getCommentsBefore(topLevelVariable1) returned ${commentsBeforeTopLevelVariable1.length} comments:\n`,
      node: topLevelVariable1,
    });

    const commentsAfterFunctionScopedVariable = sourceCode.getCommentsAfter(functionScopedVariable);
    context.report({
      message:
        `getCommentsAfter(functionScopedVariable) returned ${commentsAfterFunctionScopedVariable.length} comments:\n` +
        commentsAfterFunctionScopedVariable.map((c, i) =>
          `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`
        ).join(
          '\n',
        ),
      node: functionScopedVariable,
    });

    const commentsAfterTopLevelVariable3 = sourceCode.getCommentsAfter(topLevelVariable3);
    context.report({
      message: `getCommentsAfter(topLevelVariable3) returned ${commentsAfterTopLevelVariable3.length} comments:\n`,
      node: topLevelVariable3,
    });

    const commentsInsideTopLevelFunction = sourceCode.getCommentsInside(topLevelFunction);
    context.report({
      message: `getCommentsInside(topLevelFunction) returned ${commentsInsideTopLevelFunction.length} comments:\n` +
        commentsInsideTopLevelFunction.map((c, i) =>
          `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`
        ).join(
          '\n',
        ),
      node: topLevelFunction,
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
