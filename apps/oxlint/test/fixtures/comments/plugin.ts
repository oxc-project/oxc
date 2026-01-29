import assert from "node:assert";

import type { Comment, Plugin, Rule } from "#oxlint/plugin";

function formatComments(comments: Comment[]): string {
  let text = `${comments.length} comment${comments.length === 1 ? "" : "s"}`;
  if (comments.length > 0) {
    text += "\n";
    text += comments
      .map(
        (c, i) =>
          `  [${i}] ${c.type}: ${JSON.stringify(c.value)} at [${c.range[0]}, ${c.range[1]}]`,
      )
      .join("\n");
  }
  return text;
}

const testCommentsRule: Rule = {
  create(context) {
    const { sourceCode } = context;
    const { ast } = sourceCode;

    const comments = sourceCode.getAllComments();

    for (const comment of comments) {
      // Check getting `range` / `loc` properties twice results in same objects
      const { range, loc } = comment;
      assert(range === comment.range);
      assert(loc === comment.loc);

      // Check `getRange` and `getLoc` return the same objects too
      assert(sourceCode.getRange(comment) === range);
      assert(sourceCode.getLoc(comment) === loc);

      // Check comment can be converted to a string without an error
      // oxlint-disable-next-line typescript/no-base-to-string, typescript/restrict-template-expressions
      assert.equal(`${comment}`, "[object Object]");
    }

    context.report({
      message: `getAllComments: ${formatComments(comments)}`,
      node: ast,
    });

    const [, topLevelVariable2, topLevelFunctionExport] = ast.body;
    assert(topLevelFunctionExport.type === "ExportNamedDeclaration");
    const topLevelFunction = topLevelFunctionExport.declaration;
    assert(topLevelFunction?.type === "FunctionDeclaration");

    context.report({
      message:
        "commentsExistBetween(topLevelVariable2, topLevelFunction): " +
        sourceCode.commentsExistBetween(topLevelVariable2, topLevelFunction),
      node: topLevelVariable2,
    });

    // Test `commentsExistBetween` returns `false` when start node is after end node
    context.report({
      message:
        "commentsExistBetween(topLevelFunction, topLevelVariable2): " +
        sourceCode.commentsExistBetween(topLevelFunction, topLevelVariable2),
      node: topLevelFunction,
    });

    return {
      VariableDeclaration(node) {
        const { declarations } = node;
        assert(declarations.length >= 1);
        const { id, init } = declarations[0];
        assert(id.type === "Identifier");
        assert(init !== null);

        context.report({
          message:
            `VariableDeclaration(${id.name}):\n` +
            `getCommentsBefore: ${formatComments(sourceCode.getCommentsBefore(node))}\n` +
            `getCommentsInside: ${formatComments(sourceCode.getCommentsInside(node))}\n` +
            `getCommentsAfter: ${formatComments(sourceCode.getCommentsAfter(node))}\n` +
            `commentsExistBetween(id, init): ${sourceCode.commentsExistBetween(id, init)}`,
          node,
        });
      },
      FunctionDeclaration(node) {
        context.report({
          message:
            `FunctionDeclaration(${node.id?.name}):\n` +
            `getCommentsBefore: ${formatComments(sourceCode.getCommentsBefore(node))}\n` +
            `getCommentsInside: ${formatComments(sourceCode.getCommentsInside(node))}\n` +
            `getCommentsAfter: ${formatComments(sourceCode.getCommentsAfter(node))}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "test-comments",
  },
  rules: {
    "test-comments": testCommentsRule,
  },
};

export default plugin;
