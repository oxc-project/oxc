import type { Plugin } from '../../../dist/index.js';

const plugin: Plugin = {
  meta: {
    name: 'parents',
  },
  rules: {
    check: {
      create(context) {
        return {
          Program(node) {
            context.report({ message: `${node.type} -> ${node.parent}`, node });
          },
          VariableDeclaration(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          VariableDeclarator(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          Identifier(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          ObjectExpression(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          Property(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          ArrayExpression(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
          SpreadElement(node) {
            context.report({ message: `${node.type} -> ${node.parent.type}`, node });
          },
        };
      },
    },
  },
};

export default plugin;
