import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "fix-plugin",
  },
  rules: {
    // Fixable rule, proving fixes are applied to files parsed by a custom parser
    "no-var": {
      meta: {
        fixable: "code",
      },
      create(context) {
        return {
          VariableDeclaration(node) {
            if ((node as { kind?: string }).kind !== "var") return;
            context.report({
              message: "Unexpected var, use let or const instead",
              node,
              fix(fixer) {
                const varToken = context.sourceCode.getFirstToken(node)!;
                return fixer.replaceText(varToken, "let");
              },
            });
          },
        };
      },
    },
  },
};

export default plugin;
