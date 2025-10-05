import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'parents',
  },
  rules: {
    check: {
      create(context) {
        function reportAncestry(node: any) {
          context.report({
            message: `${node.type}:\n` +
              `parent: ${node.parent?.type}\n` +
              // @ts-ignore
              `ancestors: [ ${context.sourceCode.getAncestors(node).map(node => node.type).join(', ')} ]`,
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
