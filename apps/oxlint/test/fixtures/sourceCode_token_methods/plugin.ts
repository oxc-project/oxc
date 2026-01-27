import type { Plugin, Rule } from "#oxlint/plugin";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    return {
      Program(program) {
        for (const node of program.body) {
          const tokens = sourceCode.getTokens(node);
          context.report({
            message:
              `Tokens for ${node.type}:\n` +
              tokens
                .map(
                  ({ type, loc, range, value }) =>
                    `${type.padEnd(17)} ` +
                    `loc=${loc.start.line}:${loc.start.column}-${loc.end.line}:${loc.end.column} `.padEnd(
                      16,
                    ) +
                    `range=${range[0]}-${range[1]} `.padEnd(10) +
                    `"${value}"`,
                )
                .join("\n"),
            node,
          });
          const tokensWithComments = sourceCode.getTokens(node, { includeComments: true });
          if (tokensWithComments.length > tokens.length) {
            context.report({
              message:
                `Tokens for ${node.type}, including comments:\n` +
                tokensWithComments
                  .map(
                    ({ type, loc, range, value }) =>
                      `${type.padEnd(17)} ` +
                      `loc=${loc.start.line}:${loc.start.column}-${loc.end.line}:${loc.end.column} `.padEnd(
                        16,
                      ) +
                      `range=${range[0]}-${range[1]} `.padEnd(10) +
                      `"${value}"`,
                  )
                  .join("\n"),
              node,
            });
          }
        }
      },
    };
  },
};

const plugin: Plugin = {
  meta: { name: "token-plugin" },
  rules: { token: rule },
};

export default plugin;
