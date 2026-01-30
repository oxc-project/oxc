import assert from "node:assert";

import type { Plugin, Rule } from "#oxlint/plugin";

const STANDARD_TOKEN_KEYS = new Set(["type", "value", "start", "end", "range", "loc"]);

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    // Get this first to check it works before `sourceText` or `ast` are accessed
    const { tokensAndComments } = sourceCode;

    const { ast } = sourceCode;

    for (const tokenOrComment of tokensAndComments) {
      // Check getting `range` / `loc` properties twice results in same objects
      const { range, loc } = tokenOrComment;
      assert(range === tokenOrComment.range);
      assert(loc === tokenOrComment.loc);

      // Check `getRange` and `getLoc` return the same objects too
      assert(sourceCode.getRange(tokenOrComment) === range);
      assert(sourceCode.getLoc(tokenOrComment) === loc);

      // Check token can be converted to a string without an error
      // oxlint-disable-next-line typescript/no-base-to-string, typescript/restrict-template-expressions
      assert.equal(`${tokenOrComment}`, "[object Object]");
    }

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
      let message = `${token.type} (${JSON.stringify(token.value)})`;
      for (const key of Object.keys(token) as (keyof typeof token)[]) {
        if (!STANDARD_TOKEN_KEYS.has(key)) {
          message += `\n  ${key}: ${JSON.stringify(token[key])}`;
        }
      }

      context.report({ message, node: token });
    }

    return {};
  },
};

const plugin: Plugin = {
  meta: { name: "tokens-plugin" },
  rules: { tokens: rule },
};

export default plugin;
