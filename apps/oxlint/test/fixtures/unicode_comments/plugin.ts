import assert from "node:assert";
import type { Plugin, Rule } from "#oxlint/plugin";

const unicodeCommentsRule: Rule = {
  create(context) {
    const { sourceCode } = context;

    for (const comment of sourceCode.getAllComments()) {
      assert(comment.start === comment.range[0]);
      assert(comment.end === comment.range[1]);
      const { start, end } = comment.loc;
      context.report({
        message: JSON.stringify(
          {
            lines: `${start.line},${start.column}-${end.line},${end.column}`,
            value: comment.value,
          },
          null,
          4,
        ),
        node: comment,
      });
    }

    return {};
  },
};

const plugin: Plugin = {
  meta: {
    name: "unicode-comments",
  },
  rules: {
    "unicode-comments": unicodeCommentsRule,
  },
};

export default plugin;
