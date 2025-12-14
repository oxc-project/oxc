import assert from "node:assert";

import type { Plugin, Rule } from "#oxlint";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    // Get this first to check it works before `sourceText` or `ast` are accessed
    const { tokensAndComments } = sourceCode;

    const { ast } = sourceCode;

    // `ast.tokens` does not include comments
    context.report({
      message:
        `Tokens:\n` +
        ast.tokens
          .map((token) => {
            const { range } = token;
            assert(token.start === range[0]);
            assert(token.end === range[1]);

            const { start, end } = token.loc;

            return (
              `${token.type.padEnd(17)} ` +
              `loc= ${start.line}:${start.column} - ${end.line}:${end.column} `.padEnd(18) +
              `range= ${range[0]}-${range[1]} `.padEnd(15) +
              `${JSON.stringify(token.value)}`
            );
          })
          .join("\n"),
      node: { range: [0, sourceCode.text.length] },
    });

    // `sourceCode.tokensAndComments` does include comments
    context.report({
      message:
        `Tokens and comments:\n` +
        tokensAndComments
          .map((token) => {
            const { range } = token;
            assert(token.start === range[0]);
            assert(token.end === range[1]);

            const { start, end } = token.loc;

            return (
              `${token.type.padEnd(17)} ` +
              `loc= ${start.line}:${start.column} - ${end.line}:${end.column} `.padEnd(18) +
              `range= ${range[0]}-${range[1]} `.padEnd(15) +
              `${JSON.stringify(token.value)}`
            );
          })
          .join("\n"),
      node: { range: [0, sourceCode.text.length] },
    });

    // Report each token / comment separately
    for (const token of tokensAndComments) {
      context.report({
        message: `${token.type} (${JSON.stringify(token.value)})`,
        node: token,
      });
    }

    return {};
  },
};

const plugin: Plugin = {
  meta: { name: "tokens-plugin" },
  rules: { tokens: rule },
};

export default plugin;
