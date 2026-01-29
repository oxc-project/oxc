import type { Plugin } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "parents",
  },
  rules: {
    check: {
      create(context) {
        function reportAncestry(node: any) {
          context.report({
            message:
              `${node.type}:\n` +
              `parent: ${node.parent?.type}\n` +
              `ancestors: [ ${context.sourceCode
                .getAncestors(node)
                // @ts-expect-error - TODO: Shouldn't be an error. We need to fix our types.
                .map((node) => node.type)
                .join(", ")} ]`,
            node,
          });
        }

        return {
          Program: reportAncestry,
          VariableDeclaration: reportAncestry,
          VariableDeclarator: reportAncestry,
          Identifier: reportAncestry,
          ObjectExpression: reportAncestry,
          Property: reportAncestry,
          ArrayExpression: reportAncestry,
          SpreadElement: reportAncestry,
        };
      },
    },
  },
};

export default plugin;
