import assert from "node:assert";

import type { Plugin, Rule } from "#oxlint/plugins";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;
    const { problems, directives } = sourceCode.getDisableDirectives();

    // Test problems (e.g., multi-line disable-line)
    assert.strictEqual(problems.length, 0, `Expected no problems, got ${problems.length}`);

    // Test that we have directives
    assert(directives.length > 0, "Expected some directives");

    // Test directive types
    const [
      blockDirectivesCount,
      lineDirectivesCount,
      nextLineDirectivesCount,
      enableDirectivesCount,
    ] = directives.reduce(
      ([blockDirectives, lineDirectives, nextLineDirectives, enableDirectives], { type }) => [
        blockDirectives + (type === "disable" ? 1 : 0),
        lineDirectives + (type === "disable-line" ? 1 : 0),
        nextLineDirectives + (type === "disable-next-line" ? 1 : 0),
        enableDirectives + (type === "enable" ? 1 : 0),
      ],
      [0, 0, 0, 0],
    );

    assert(blockDirectivesCount > 0, "Expected block directives");
    assert(lineDirectivesCount > 0, "Expected line directives");
    assert(nextLineDirectivesCount > 0, "Expected next-line directives");
    assert(enableDirectivesCount > 0, "Expected enable directives");

    // Test that all directives have required fields
    for (const directive of directives) {
      assert(
        ["disable", "disable-line", "disable-next-line", "enable"].includes(directive.type),
        `Invalid directive type: ${directive.type}`,
      );
      assert(directive.node, "Directive must have a node");
      assert(directive.value !== undefined, "Directive must have a value");
    }

    context.report({
      message:
        `getDisableDirectives:\n` +
        `  total: ${directives.length}\n` +
        `  block: ${blockDirectivesCount}\n` +
        `  line: ${lineDirectivesCount}\n` +
        `  next-line: ${nextLineDirectivesCount}\n` +
        `  enable: ${enableDirectivesCount}`,
      node: sourceCode.ast,
    });

    return {};
  },
};

const plugin: Plugin = {
  meta: { name: "get-disable-directives-plugin" },
  rules: {
    "get-disable-directives": rule,
  },
};

export default plugin;
