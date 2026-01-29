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

const createRule: Rule = {
  create(context) {
    const { sourceCode } = context;

    // Get these first to check they work before `sourceText` or `ast` are accessed
    const { lineStartIndices, lines } = sourceCode;
    const { ast, text } = sourceCode;

    assert(context.getSourceCode() === sourceCode);
    assert(sourceCode.getLines() === lines);

    let locs = "";
    for (let offset = 0; offset <= text.length; offset++) {
      const loc = sourceCode.getLocFromIndex(offset);
      assert(sourceCode.getIndexFromLoc(loc) === offset);
      locs +=
        `\n  ${offset} => { line: ${loc.line}, column: ${loc.column} }` +
        `(${JSON.stringify(text[offset] || "<EOF>")})`;
    }

    const stmt = ast.body[0];
    assert.strictEqual(stmt.type, "VariableDeclaration");
    const { id } = stmt.declarations[0];
    assert.strictEqual(id.type, "Identifier");

    context.report({
      message:
        "create:\n" +
        `text: ${JSON.stringify(text)}\n` +
        `getText(): ${JSON.stringify(sourceCode.getText())}\n` +
        `lines: ${JSON.stringify(lines)}\n` +
        `lineStartIndices: ${JSON.stringify(lineStartIndices)}\n` +
        `locs:${locs}\n` +
        `ast: "${id.name}"\n` +
        `visitorKeys: ${sourceCode.visitorKeys.BinaryExpression.join(", ")}\n` +
        `isESTree: ${sourceCode.isESTree}`,
      node: SPAN,
    });

    return {
      Program(node) {
        assert(node === ast);
      },
      VariableDeclaration(node) {
        context.report({
          message: `var decl:\nsource: "${sourceCode.getText(node)}"`,
          node,
        });
      },
      Identifier(node) {
        const startLoc = sourceCode.getLocFromIndex(node.start);
        const endLoc = sourceCode.getLocFromIndex(node.end);
        assert(sourceCode.getIndexFromLoc(startLoc) === node.start);
        assert(sourceCode.getIndexFromLoc(endLoc) === node.end);

        const { range, loc } = node;

        // Check getting `range` / `loc` properties twice results in same objects
        assert(range === node.range);
        assert(loc === node.loc);
        // Check `getRange` and `getLoc` return the same objects too
        assert(sourceCode.getRange(node) === range);
        assert(sourceCode.getLoc(node) === loc);

        context.report({
          message:
            `ident "${node.name}":\n` +
            `source: "${sourceCode.getText(node)}"\n` +
            `source with before: "${sourceCode.getText(node, 2)}"\n` +
            `source with after: "${sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${sourceCode.getText(node, 2, 1)}"\n` +
            `range: ${JSON.stringify(range)}\n` +
            `loc: ${JSON.stringify(loc)}\n` +
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
        const { sourceCode } = context;

        // Get these first to check they work before `sourceText` or `ast` are accessed
        const { lineStartIndices, lines } = sourceCode;
        ast = sourceCode.ast;
        const { text } = sourceCode;

        let locs = "";
        for (let offset = 0; offset <= text.length; offset++) {
          const loc = sourceCode.getLocFromIndex(offset);
          assert(sourceCode.getIndexFromLoc(loc) === offset);
          locs +=
            `\n  ${offset} => { line: ${loc.line}, column: ${loc.column} }` +
            `(${JSON.stringify(text[offset] || "<EOF>")})`;
        }

        const stmt = ast.body[0];
        assert.strictEqual(stmt.type, "VariableDeclaration");
        const { id } = stmt.declarations[0];
        assert.strictEqual(id.type, "Identifier");

        context.report({
          message:
            "before:\n" +
            `text: ${JSON.stringify(text)}\n` +
            `getText(): ${JSON.stringify(sourceCode.getText())}\n` +
            `lines: ${JSON.stringify(lines)}\n` +
            `lineStartIndices: ${JSON.stringify(lineStartIndices)}\n` +
            `locs:${locs}\n` +
            `ast: "${id.name}"\n` +
            `visitorKeys: ${sourceCode.visitorKeys.BinaryExpression.join(", ")}\n` +
            `isESTree: ${sourceCode.isESTree}`,
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
        const { sourceCode } = context;

        const startLoc = sourceCode.getLocFromIndex(node.start);
        const endLoc = sourceCode.getLocFromIndex(node.end);
        assert(sourceCode.getIndexFromLoc(startLoc) === node.start);
        assert(sourceCode.getIndexFromLoc(endLoc) === node.end);

        context.report({
          message:
            `ident "${node.name}":\n` +
            `source: "${sourceCode.getText(node)}"\n` +
            `source with before: "${sourceCode.getText(node, 2)}"\n` +
            `source with after: "${sourceCode.getText(node, null, 1)}"\n` +
            `source with both: "${sourceCode.getText(node, 2, 1)}"\n` +
            `start loc: ${JSON.stringify(startLoc)}\n` +
            `end loc: ${JSON.stringify(endLoc)}`,
          node,
        });
      },
      after() {
        const { sourceCode } = context;

        assert(sourceCode.ast === ast);
        ast = null;

        context.report({
          message: "after:\n" + `source: ${JSON.stringify(sourceCode.text)}`,
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
