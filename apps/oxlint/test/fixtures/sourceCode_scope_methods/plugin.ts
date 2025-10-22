import assert from 'node:assert';

import type { ESTree, Node, Plugin, Rule, Scope, Variable } from '../../../dist/index.js';

type Program = ESTree.Program;

type Identifier =
  | ESTree.IdentifierName
  | ESTree.IdentifierReference
  | ESTree.BindingIdentifier
  | ESTree.LabelIdentifier;

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    return {
      VariableDeclaration(node) {
        const variables = sourceCode.getDeclaredVariables(node);
        context.report({
          message: `getDeclaredVariables(): ${variables.map(v => v.name).join(', ')}`,
          node,
        });
      },
      Identifier(node) {
        const { name } = node;
        const isGlobal = sourceCode.isGlobalReference(node);
        context.report({
          message: `isGlobalReference(${name}): ${isGlobal}`,
          node,
        });
      },
      FunctionDeclaration(node) {
        const scope = sourceCode.getScope(node);
        let text = '';
        text += `type: ${scope.type}\n`;
        text += `isStrict: ${scope.isStrict}\n`;
        text += `vars: [${scope.variables.map(v => v.name).join(', ')}]\n`;
        text += `through: [${scope.through.map(r => (r.identifier as any).name).join(', ')}]\n`;
        if (scope.upper) text += `upper: ${scope.upper.type}\n`;

        context.report({
          message: `getScope(${node.id.name}): ${text}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: { name: 'scope-plugin' },
  rules: { scope: rule },
};

export default plugin;
