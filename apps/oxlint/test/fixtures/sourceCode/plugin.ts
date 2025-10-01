import type { Plugin, Rule } from '../../../dist/index.js';

const SPAN = { start: 0, end: 0 };

const createRule: Rule = {
  create(context) {
    context.report({
      message: 'create:\n' +
        `sourceCode.text: ${JSON.stringify(context.sourceCode.text)}\n` +
        `sourceCode.getText(): ${JSON.stringify(context.sourceCode.getText())}`,
      node: SPAN,
    });

    return {
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
    return {
      before() {
        context.report({
          message: 'before:\n' +
            `sourceCode.text: ${JSON.stringify(context.sourceCode.text)}\n` +
            `sourceCode.getText(): ${JSON.stringify(context.sourceCode.getText())}`,
          node: SPAN,
        });
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
