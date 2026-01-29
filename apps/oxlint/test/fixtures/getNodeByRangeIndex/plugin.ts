import type { Plugin, Rule } from "#oxlint/plugin";

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    // Check no error calling before AST or source code is accessed
    sourceCode.getNodeByRangeIndex(0);

    // Get nodes:
    // * At start of file
    // * Before, at start of, inside, and at end of every token
    const { tokens } = sourceCode.ast;

    const indexes = new Set([0]);
    for (const token of tokens) {
      const [start, end] = token.range;
      indexes.add(start - 1);
      indexes.add(start);
      if (end - start > 1) indexes.add(end - 1);
      indexes.add(end);
    }
    const sourceTextLen = sourceCode.text.length;
    indexes.add(sourceTextLen - 1);
    indexes.add(sourceTextLen);

    for (const index of indexes) {
      const node = sourceCode.getNodeByRangeIndex(index);
      context.report({
        message: `type: ${node === null ? null : node.type}`,
        node: { range: [index, index] },
      });
    }

    // Get node after end of file
    const node = sourceCode.getNodeByRangeIndex(sourceTextLen + 1);
    context.report({
      message: `type: ${node === null ? null : node.type}`,
      node: { range: [sourceTextLen, sourceTextLen] },
    });

    return {};
  },
};

const plugin: Plugin = {
  meta: {
    name: "getNode-plugin",
  },
  rules: {
    getNode: rule,
  },
};

export default plugin;
