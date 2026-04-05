import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "language-options-parser-parse-plugin",
  },
  rules: {
    "check-parser-parse": {
      create(context) {
        return {
          Program(node) {
            const parsed = context.languageOptions.parser.parse?.("let foo = 1;", {
              sourceType: "module",
            }) as { body: Array<{ type: string }> };
            const parsedForESLint = context.languageOptions.parser.parseForESLint?.(
              "let bar = 1;",
              { sourceType: "module" },
            ) as { ast: { body: Array<{ type: string }> } };

            context.report({
              message:
                `parse: ${parsed.body[0].type}\n` +
                `parseForESLint: ${parsedForESLint.ast.body[0].type}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
