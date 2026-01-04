// eslint-plugin-jest code: https://github.com/jest-community/eslint-plugin-jest/blob/45edad2158305d3d5907c00ed698b592379b4490/src/rules/no-large-snapshots.ts
// eslint-plugin-jest license (MIT): https://github.com/jest-community/eslint-plugin-jest/blob/45edad2158305d3d5907c00ed698b592379b4490/LICENSE

import { isAbsolute } from 'path';
import {
  AST_NODE_TYPES,
  type TSESLint,
  type TSESTree,
} from '@typescript-eslint/utils';
import {
  createRule,
  getAccessorValue,
  isSupportedAccessor,
  parseJestFnCall,
} from './utils';

interface RuleOptions {
  maxSize?: number;
  inlineMaxSize?: number;
  allowedSnapshots?: Record<string, Array<string | RegExp>>;
}

type MessageId = 'noSnapshot' | 'tooLongSnapshots';

type RuleContext = TSESLint.RuleContext<MessageId, [RuleOptions]>;

const reportOnViolation = (
  context: RuleContext,
  node: TSESTree.CallExpressionArgument | TSESTree.ExpressionStatement,
  { maxSize: lineLimit = 50, allowedSnapshots = {} }: RuleOptions,
) => {
  const startLine = node.loc.start.line;
  const endLine = node.loc.end.line;
  const lineCount = endLine - startLine;

  const allPathsAreAbsolute = Object.keys(allowedSnapshots).every(isAbsolute);

  if (!allPathsAreAbsolute) {
    throw new Error(
      'All paths for allowedSnapshots must be absolute. You can use JS config and `path.resolve`',
    );
  }

  let isAllowed = false;

  if (
    node.type === AST_NODE_TYPES.ExpressionStatement &&
    'left' in node.expression &&
    node.expression.left.type === AST_NODE_TYPES.MemberExpression &&
    isSupportedAccessor(node.expression.left.property)
  ) {
    const fileName = context.filename;
    const allowedSnapshotsInFile = allowedSnapshots[fileName];

    if (allowedSnapshotsInFile) {
      const snapshotName = getAccessorValue(node.expression.left.property);

      isAllowed = allowedSnapshotsInFile.some(name => {
        if (typeof name === 'string') {
          return snapshotName === name;
        }

        return name.test(snapshotName);
      });
    }
  }

  if (!isAllowed && lineCount > lineLimit) {
    context.report({
      messageId: lineLimit === 0 ? 'noSnapshot' : 'tooLongSnapshots',
      data: { lineLimit, lineCount },
      node,
    });
  }
};

export default createRule<[RuleOptions], MessageId>({
  name: __filename,
  meta: {
    docs: {
      description: 'Disallow large snapshots',
    },
    messages: {
      noSnapshot:
        'Expected to not encounter a Jest snapshot but one was found that is {{ lineCount }} lines long',
      tooLongSnapshots:
        'Expected Jest snapshot to be smaller than {{ lineLimit }} lines but was {{ lineCount }} lines long',
    },
    type: 'suggestion',
    schema: [
      {
        type: 'object',
        properties: {
          maxSize: {
            type: 'number',
          },
          inlineMaxSize: {
            type: 'number',
          },
          allowedSnapshots: {
            type: 'object',
            additionalProperties: { type: 'array' },
          },
        },
        additionalProperties: false,
      },
    ],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    if (context.filename.endsWith('.snap')) {
      return {
        ExpressionStatement(node) {
          reportOnViolation(context, node, options);
        },
      };
    }

    return {
      CallExpression(node) {
        const jestFnCall = parseJestFnCall(node, context);

        if (jestFnCall?.type !== 'expect') {
          return;
        }

        if (
          [
            'toMatchInlineSnapshot',
            'toThrowErrorMatchingInlineSnapshot',
          ].includes(getAccessorValue(jestFnCall.matcher)) &&
          jestFnCall.args.length
        ) {
          reportOnViolation(context, jestFnCall.args[0], {
            ...options,
            maxSize: options.inlineMaxSize ?? options.maxSize,
          });
        }
      },
    };
  },
});
