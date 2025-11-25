import assert from "node:assert";

import type { ESTree, Node, Plugin, Rule } from "#oxlint";

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

const createRule: Rule = {
  create(context) {
    const { ast, lines, text } = context.sourceCode;

    assert(context.getSourceCode() === context.sourceCode);

    let locs = "";
    for (let offset = 0; offset <= text.length; offset++) {
      const loc = context.sourceCode.getLocFromIndex(offset);
      assert(context.sourceCode.getIndexFromLoc(loc) === offset);
      locs +=
        `\n  ${offset} => { line: ${loc.line}, column: ${loc.column} }` +
        `(${JSON.stringify(text[offset] || "<EOF>")})`;
    }

    context.report({
      message:
        "create:\n" +
        `text: ${JSON.stringify(text)}\n` +
        `getText(): ${JSON.stringify(context.sourceCode.getText())}\n` +
        `lines: ${JSON.stringify(lines)}\n` +
        `locs:${locs}\n` +
        // @ts-ignore
        `ast: "${ast.body[0].declarations[0].id.name}"\n` +
        `visitorKeys: ${context.sourceCode.visitorKeys.BinaryExpression.join(", ")}`,
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
        const startLoc = context.sourceCode.getLocFromIndex(node.start);
        const endLoc = context.sourceCode.getLocFromIndex(node.end);
        assert(context.sourceCode.getIndexFromLoc(startLoc) === node.start);
        assert(context.sourceCode.getIndexFromLoc(endLoc) === node.end);

        context.report({
          message:
            `ident "${node.name}":\n` +
            `source: "${context.sourceCode.getText(node)}"\n` +
            `source with before: "${context.sourceCode.getText(node, 2)}"\n` +
            `source with after: "${context.sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${context.sourceCode.getText(node, 2, 1)}"\n` +
            `start loc: ${JSON.stringify(startLoc)}\n` +
            `end loc: ${JSON.stringify(endLoc)}`,
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
        const { lines, text } = context.sourceCode;

        let locs = "";
        for (let offset = 0; offset <= text.length; offset++) {
          const loc = context.sourceCode.getLocFromIndex(offset);
          assert(context.sourceCode.getIndexFromLoc(loc) === offset);
          locs +=
            `\n  ${offset} => { line: ${loc.line}, column: ${loc.column} }` +
            `(${JSON.stringify(text[offset] || "<EOF>")})`;
        }

        context.report({
          message:
            "before:\n" +
            `text: ${JSON.stringify(text)}\n` +
            `getText(): ${JSON.stringify(context.sourceCode.getText())}\n` +
            `lines: ${JSON.stringify(lines)}\n` +
            `locs:${locs}\n` +
            // @ts-ignore
            `ast: "${ast.body[0].declarations[0].id.name}"\n` +
            `visitorKeys: ${context.sourceCode.visitorKeys.BinaryExpression.join(", ")}`,
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
        const startLoc = context.sourceCode.getLocFromIndex(node.start);
        const endLoc = context.sourceCode.getLocFromIndex(node.end);
        assert(context.sourceCode.getIndexFromLoc(startLoc) === node.start);
        assert(context.sourceCode.getIndexFromLoc(endLoc) === node.end);

        context.report({
          message:
            `ident "${node.name}":\n` +
            `source: "${context.sourceCode.getText(node)}"\n` +
            `source with before: "${context.sourceCode.getText(node, 2)}"\n` +
            `source with after: "${context.sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${context.sourceCode.getText(node, 2, 1)}"\n` +
            `start loc: ${JSON.stringify(startLoc)}\n` +
            `end loc: ${JSON.stringify(endLoc)}`,
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
