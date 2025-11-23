import { describe, expect, it, vi } from 'vitest';

import { getTokens } from '../src-js/plugins/tokens.js';
import type { Node } from '../src-js/plugins/types.js';

let sourceText = 'null;';

vi.mock('../src-js/plugins/source_code.ts', () => {
  return {
    get sourceText() {
      return sourceText;
    },
  };
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L62
describe('when calling getTokens', () => {
  sourceText = '/*A*/var answer/*B*/=/*C*/a/*D*/* b/*E*///F\n    call();\n/*Z*/';

  // TODO: We are lying about `Program`'s range here.
  // The range provided by `@typescript-eslint/typescript-estree` does not match the assertions for that of `espree`.
  // The deviation is being corrected in upcoming releases of ESLint and TS-ESLint.
  // https://eslint.org/blog/2025/10/whats-coming-in-eslint-10.0.0/#updates-to-program-ast-node-range-coverage
  // https://github.com/typescript-eslint/typescript-eslint/issues/11026#issuecomment-3421887632
  const Program = { range: [5, 55] } as Node;
  const BinaryExpression = { range: [26, 35] } as Node;

  it('should retrieve all tokens for root node', () => {
    expect(getTokens(Program).map((token) => token.value)).toEqual([
      'var',
      'answer',
      '=',
      'a',
      '*',
      'b',
      'call',
      '(',
      ')',
      ';',
    ]);
  });

  it('should retrieve all tokens for binary expression', () => {
    expect(getTokens(BinaryExpression).map((token) => token.value)).toEqual(['a', '*', 'b']);
  });

  it('should retrieve all tokens plus one before for binary expression', () => {
    expect(getTokens(BinaryExpression, 1).map((token) => token.value)).toEqual(['=', 'a', '*', 'b']);
  });

  it('should retrieve all tokens plus one after for binary expression', () => {
    expect(getTokens(BinaryExpression, 0, 1).map((token) => token.value)).toEqual(['a', '*', 'b', 'call']);
  });

  it('should retrieve all tokens plus two before and one after for binary expression', () => {
    expect(getTokens(BinaryExpression, 2, 1).map((token) => token.value)).toEqual([
      'answer',
      '=',
      'a',
      '*',
      'b',
      'call',
    ]);
  });

  it('should retrieve all matched tokens for root node with filter', () => {
    expect(getTokens(Program, (t) => t.type === 'Identifier').map((token) => token.value)).toEqual([
      'answer',
      'a',
      'b',
      'call',
    ]);
    expect(
      getTokens(Program, {
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
    ).toEqual(['answer', 'a', 'b', 'call']);
  });

  it('should retrieve all tokens and comments in the node for root node with includeComments option', () => {
    expect(getTokens(Program, { includeComments: true }).map((token) => token.value)).toEqual([
      'var',
      'answer',
      'B',
      '=',
      'C',
      'a',
      'D',
      '*',
      'b',
      'E',
      'F',
      'call',
      '(',
      ')',
      ';',
    ]);
  });

  it('should retrieve matched tokens and comments in the node for root node with includeComments and filter options', () => {
    expect(
      getTokens(Program, {
        includeComments: true,
        filter: (t) => t.type.startsWith('Block'),
      }).map((token) => token.value),
    ).toEqual(['B', 'C', 'D', 'E']);
  });

  it('should retrieve all tokens and comments in the node for binary expression with includeComments option', () => {
    expect(getTokens(BinaryExpression, { includeComments: true }).map((token) => token.value)).toEqual([
      'a',
      'D',
      '*',
      'b',
    ]);
  });

  it('should handle out of range nodes gracefully', () => {
    expect(getTokens({ range: [1000, 1001] } as Node).map((token) => token.value)).toEqual([]);
  });
});
