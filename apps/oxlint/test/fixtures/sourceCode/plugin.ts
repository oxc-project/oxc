import assert from 'node:assert';

import type { Program } from '@oxc-project/types';
import type { Plugin, Rule } from '../../../dist/index.js';

const SPAN = { start: 0, end: 0 };

const createRule: Rule = {
  create(context) {
    const { ast } = context.sourceCode;

    context.report({
      message: 'create:\n' +
        `text: ${JSON.stringify(context.sourceCode.text)}\n` +
        `getText(): ${JSON.stringify(context.sourceCode.getText())}\n` +
        `lines: ${JSON.stringify(context.sourceCode.lines)}\n` +
        // @ts-ignore
        `ast: "${ast.body[0].declarations[0].id.name}"\n` +
        `visitorKeys: ${context.sourceCode.visitorKeys.BinaryExpression.join(', ')}`,
      node: SPAN,
    });

    return {
      Program(node) {
        assert(node === ast);
      },
      VariableDeclaration(node) {
        context.report({
          message: `var decl:\nsource: "${context.sourceCode.getText(node)}"`,
          node,
        });
      },
      Identifier(node) {
        context.report({
          message: `ident "${node.name}":\n` +
            `source: "${context.sourceCode.getText(node)}"\n` +
            `source with before: "${context.sourceCode.getText(node, 2)}"\n` +
            `source with after: "${context.sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${context.sourceCode.getText(node, 2, 1)}"`,
          node,
        });
      },
    };
  },
};

const createOnceRule: Rule = {
  createOnce(context) {
    let ast: Program | null = null;

    return {
      before() {
        ast = context.sourceCode.ast;

        context.report({
          message: 'before:\n' +
            `text: ${JSON.stringify(context.sourceCode.text)}\n` +
            `getText(): ${JSON.stringify(context.sourceCode.getText())}\n` +
            `lines: ${JSON.stringify(context.sourceCode.lines)}\n` +
            // @ts-ignore
            `ast: "${ast.body[0].declarations[0].id.name}"\n` +
            `visitorKeys: ${context.sourceCode.visitorKeys.BinaryExpression.join(', ')}`,
          node: SPAN,
        });
      },
      Program(node) {
        assert(node === ast);
      },
      VariableDeclaration(node) {
        context.report({
          message: `var decl:\nsource: "${context.sourceCode.getText(node)}"`,
          node,
        });
      },
      Identifier(node) {
        context.report({
          message: `ident "${node.name}":\n` +
            `source: "${context.sourceCode.getText(node)}"\n` +
            `source with before: "${context.sourceCode.getText(node, 2)}"\n` +
            `source with after: "${context.sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${context.sourceCode.getText(node, 2, 1)}"`,
          node,
        });
      },
      after() {
        assert(context.sourceCode.ast === ast);
        ast = null;

        context.report({
          message: 'after:\n' +
            `source: ${JSON.stringify(context.sourceCode.text)}`,
          node: SPAN,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'source-code-plugin',
  },
  rules: {
    create: createRule,
    'create-once': createOnceRule,
  },
};

export default plugin;
