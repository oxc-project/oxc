import assert from 'node:assert';

import type { Plugin, Rule, Node } from '../../../dist/index.js';

const testRule: Rule = {
  create(context) {
    return {
      AssignmentExpression(node) {
        const { isSpaceBetween } = context.sourceCode,
          { left, right } = node;

        context.report({
          message:
            '\n' +
            // Test where 2 nodes are separated, maybe with whitespace in between
            `isSpaceBetween(left, right): ${isSpaceBetween(left, right)}\n` +
            `isSpaceBetween(right, left): ${isSpaceBetween(right, left)}\n` +
            // Test where 1 node is inside another, sharing same `start` or `end`
            `isSpaceBetween(left, node): ${isSpaceBetween(left, node)}\n` +
            `isSpaceBetween(node, left): ${isSpaceBetween(node, left)}\n` +
            `isSpaceBetween(right, node): ${isSpaceBetween(right, node)}\n` +
            `isSpaceBetween(node, right): ${isSpaceBetween(node, right)}`,
          node,
        });

        // Test where 1 node is inside another, not sharing same `start` or `end`
        if (right.type === 'BinaryExpression') {
          const binaryLeft = right.left;
          context.report({
            message:
              '\n' +
              `isSpaceBetween(node, binaryLeft): ${isSpaceBetween(node, binaryLeft)}\n` +
              `isSpaceBetween(binaryLeft, node): ${isSpaceBetween(binaryLeft, node)}`,
            node,
          });
        }

        // Test where 2 nodes are completely adjacent to each other.
        // We don't have tokens yet, so adjust ranges of 1 node so they touch.
        assert(left.type === 'Identifier');
        if (left.name === 'noSpace') {
          const leftExtended: Node = { ...left, end: left.end + 1, range: [left.range[0], left.range[1] + 1] };
          assert(leftExtended.end === right.start);
          assert(leftExtended.range[1] === right.range[0]);

          context.report({
            message:
              '\n' +
              `isSpaceBetween(leftExtended, right): ${isSpaceBetween(leftExtended, right)}\n` +
              `isSpaceBetween(right, leftExtended): ${isSpaceBetween(right, leftExtended)}`,
            node,
          });
        }
      },

      SequenceExpression(node) {
        const { isSpaceBetween } = context.sourceCode,
          [beforeString, , afterString] = node.expressions;

        // We get this wrong. Should be `false`, but we get `true`.
        context.report({
          message:
            '\n' +
            `isSpaceBetween(beforeString, afterString): ${isSpaceBetween(beforeString, afterString)}\n` +
            `isSpaceBetween(afterString, beforeString): ${isSpaceBetween(afterString, beforeString)}`,
          node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'test-plugin',
  },
  rules: {
    'is-space-between': testRule,
  },
};

export default plugin;
