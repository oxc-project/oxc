import assert from "node:assert";

import type { ESTree, Node, Plugin, Rule } from "#oxlint/plugin";

type Program = ESTree.Program;

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

// Purpose of this test fixture is to ensure that AST is not deserialized twice
// if `context.sourceCode.ast` is accessed during AST traversal.
//
// `sourceCode` test fixture tests the opposite case.
// In that fixture `context.sourceCode.ast` is accessed in `create` method or `before` hook
// - which are before AST traversal starts.

const createRule: Rule = {
  create(context) {
    let ast: Program | null = null;

    return {
      Program(node) {
        ast = context.sourceCode.ast;
        assert(ast === node);

        context.report({
          message:
            "program:\n" +
            `text: ${JSON.stringify(context.sourceCode.text)}\n` +
            `getText(): ${JSON.stringify(context.sourceCode.getText())}`,
          node: SPAN,
        });
      },
      VariableDeclaration(node) {
        assert(context.sourceCode.ast === ast);

        context.report({
          message: `var decl:\nsource: "${context.sourceCode.getText(node)}"`,
          node,
        });
      },
      Identifier(node) {
        assert(context.sourceCode.ast === ast);

        context.report({
          message:
            `ident "${node.name}":\n` +
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
      Program(node) {
        ast = context.sourceCode.ast;
        assert(ast === node);

        context.report({
          message:
            "program:\n" +
            `text: ${JSON.stringify(context.sourceCode.text)}\n` +
            `getText(): ${JSON.stringify(context.sourceCode.getText())}`,
          node: SPAN,
        });
      },
      VariableDeclaration(node) {
        assert(context.sourceCode.ast === ast);

        context.report({
          message: `var decl:\nsource: "${context.sourceCode.getText(node)}"`,
          node,
        });
      },
      Identifier(node) {
        assert(context.sourceCode.ast === ast);

        context.report({
          message:
            `ident "${node.name}":\n` +
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
          message: "after:\n" + `source: ${JSON.stringify(context.sourceCode.text)}`,
          node: SPAN,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "source-code-plugin",
  },
  rules: {
    create: createRule,
    "create-once": createOnceRule,
  },
};

export default plugin;
