import type { Plugin, Rule } from "#oxlint";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context,
      { ast } = sourceCode;

    // Note: Comments should not appear in `ast.tokens`
    context.report({
      message:
        `Tokens:\n` +
        ast.tokens
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
      node: { range: [0, sourceCode.text.length] },
    });

    return {};
  },
};

const plugin: Plugin = {
  meta: { name: "tokens-plugin" },
  rules: { tokens: rule },
};

export default plugin;
