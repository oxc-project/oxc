/**
 * Test plugin for custom parser integration.
 *
 * This plugin contains rules that verify the custom parser AST is properly
 * passed to JS rules and can be traversed correctly.
 */

import type { Plugin, Rule, Node } from "#oxlint";

/**
 * Rule that reports when a variable named "foo" is declared.
 * This tests that VariableDeclaration nodes from custom parser are visited.
 */
const noFooVarRule: Rule = {
  create(context) {
    return {
      VariableDeclarator(node) {
        // Type assertion since this comes from custom parser
        const declarator = node as unknown as {
          id: { type: string; name: string } & Node;
        } & Node;

        if (declarator.id.type === "Identifier" && declarator.id.name === "foo") {
          context.report({
            message: `Variable "foo" is not allowed in custom DSL files`,
            node: declarator.id,
          });
        }
      },
    };
  },
};

/**
 * Rule that counts all identifiers in the file and reports at the end.
 * This tests that all nodes are traversed and createOnce works with custom parser.
 */
const countIdentifiersRule: Rule = {
  createOnce(context) {
    let identifierCount = 0;

    return {
      before() {
        identifierCount = 0;
        context.report({
          message: `Starting to lint: ${context.filename}`,
          node: {
            start: 0,
            end: 0,
            range: [0, 0],
            loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 0 } },
          },
        });
      },
      Identifier(node) {
        identifierCount++;
        const identifier = node as unknown as { name: string } & Node;
        context.report({
          message: `Found identifier: ${identifier.name}`,
          node,
        });
      },
      after() {
        context.report({
          message: `Total identifiers found: ${identifierCount}`,
          node: {
            start: 0,
            end: 0,
            range: [0, 0],
            loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 0 } },
          },
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "custom-parser-plugin",
  },
  rules: {
    "no-foo-var": noFooVarRule,
    "count-identifiers": countIdentifiersRule,
  },
};

export default plugin;
