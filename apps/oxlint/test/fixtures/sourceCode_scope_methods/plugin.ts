import type { Plugin, Rule } from "#oxlint/plugin";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    return {
      VariableDeclaration(node) {
        const variables = sourceCode.getDeclaredVariables(node);
        context.report({
          message: `getDeclaredVariables(): ${variables.map((v) => v.name).join(", ")}`,
          node,
        });
      },
      Identifier(node) {
        context.report({
          message: `isGlobalReference(${node.name}): ${sourceCode.isGlobalReference(node)}`,
          node,
        });
      },
      FunctionDeclaration(node) {
        const scope = sourceCode.getScope(node);
        context.report({
          message:
            `getScope(${node.id?.name}):\n` +
            `type: ${scope.type}\n` +
            `isStrict: ${scope.isStrict}\n` +
            `variables: [${scope.variables.map((v) => v.name).join(", ")}]\n` +
            `through: [${scope.through.map((r) => r.identifier.name).join(", ")}]\n` +
            `upper type: ${scope.upper?.type}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: { name: "scope-plugin" },
  rules: { scope: rule },
};

export default plugin;
