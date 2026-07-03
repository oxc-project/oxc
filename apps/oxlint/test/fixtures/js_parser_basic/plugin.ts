import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "custom-parser-plugin",
  },
  rules: {
    // Rule visiting the non-standard `CustomTemplate` node type produced by the custom parser,
    // proving unknown-node-type dispatch works (bare type key + selector key)
    "no-template": {
      create(context) {
        return {
          CustomTemplate(node) {
            context.report({
              message: "Unexpected template",
              node,
            });
          },
          "VariableDeclarator > CustomTemplate"(node) {
            context.report({
              message: "Template initializer",
              node,
            });
          },
        };
      },
    },
    // Rule using `SourceCode` methods, proving `context.sourceCode` works for custom parser ASTs
    "source-code-check": {
      create(context) {
        const { sourceCode } = context;
        return {
          FunctionDeclaration(node) {
            const scope = sourceCode.getScope(node);
            const name = node.id === null ? "?" : sourceCode.getText(node.id);
            const firstToken = sourceCode.getFirstToken(node)!;
            const tokenAfterFirst = sourceCode.getTokenAfter(firstToken)!;
            context.report({
              message:
                `fn \`${name}\`: scope=${scope.type}, ` +
                `firstToken=${firstToken.value}, tokenAfterFirst=${tokenAfterFirst.value}, ` +
                `parserServices.isToyParser=${String(sourceCode.parserServices.isToyParser)}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
