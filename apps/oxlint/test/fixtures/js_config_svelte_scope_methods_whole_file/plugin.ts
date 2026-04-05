import type { Node, Plugin } from "#oxlint/plugins";

type ScopeVariable = {
  eslintUsed?: boolean;
};

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte-scope",
  },
  rules: {
    methods: {
      create(context) {
        return {
          Program(node) {
            const declared = context.sourceCode.getDeclaredVariables(node);
            const scope = context.sourceCode.getScope(node);
            const scopeManager = context.sourceCode.scopeManager;
            const firstExpression = (context.sourceCode.ast.body[0] as { expression?: Node } | undefined)
              ?.expression;
            const globalVariable = scopeManager.globalScope?.set.get("externalGlobal") as
              | ScopeVariable
              | undefined;

            context.report({
              message: [
                `text=${context.sourceCode.text.includes("<h1>Hello</h1>")}`,
                `sameScope=${scopeManager.acquire(node) === scope}`,
                `scopeType=${scope.type}`,
                `declared=${declared.map((variable) => variable.name).join(",")}`,
                `globalRef=${
                  firstExpression == null
                    ? "<missing>"
                    : String(context.sourceCode.isGlobalReference(firstExpression))
                }`,
                `markUsed=${context.sourceCode.markVariableAsUsed("externalGlobal", node)}`,
                `used=${globalVariable?.eslintUsed === true}`,
              ].join("; "),
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
