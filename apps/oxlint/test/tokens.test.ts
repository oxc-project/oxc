import assert from 'node:assert';
import { describe, it, vi } from 'vitest';
import {
  getTokens,
  getTokensBefore,
  getTokenBefore,
  getTokensAfter,
  getTokenAfter,
  getFirstTokens,
} from '../src-js/plugins/tokens.js';
import { resetSourceAndAst } from '../src-js/plugins/source_code.js';
import type { Node } from '../src-js/plugins/types.js';

let sourceText = '/*A*/var answer/*B*/=/*C*/a/*D*/* b/*E*///F\n    call();\n/*Z*/';

vi.mock('../src-js/plugins/source_code.ts', async (importOriginal) => {
  const original: any = await importOriginal();
  return {
    ...original,
    get sourceText() {
      return sourceText;
    },
  };
});

// TODO: We are lying about `Program`'s range here.
// The range provided by `@typescript-eslint/typescript-estree` does not match the assertions for that of `espree`.
// The deviation is being corrected in upcoming releases of ESLint and TS-ESLint.
// https://eslint.org/blog/2025/10/whats-coming-in-eslint-10.0.0/#updates-to-program-ast-node-range-coverage
// https://github.com/typescript-eslint/typescript-eslint/issues/11026#issuecomment-3421887632
const Program = { range: [5, 55] } as Node;
const BinaryExpression = { range: [26, 35] } as Node;
/* oxlint-disable-next-line no-unused-vars */
const VariableDeclaratorIdentifier = { range: [9, 15] } as Node;

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L62
describe('when calling getTokens', () => {
  it('should retrieve all tokens for root node', () => {
    assert.deepStrictEqual(
      getTokens(Program).map((token) => token.value),
      ['var', 'answer', '=', 'a', '*', 'b', 'call', '(', ')', ';'],
    );
  });

  it('should retrieve all tokens for binary expression', () => {
    assert.deepStrictEqual(
      getTokens(BinaryExpression).map((token) => token.value),
      ['a', '*', 'b'],
    );
  });

  it('should retrieve all tokens plus one before for binary expression', () => {
    assert.deepStrictEqual(
      getTokens(BinaryExpression, 1).map((token) => token.value),
      ['=', 'a', '*', 'b'],
    );
  });

  it('should retrieve all tokens plus one after for binary expression', () => {
    assert.deepStrictEqual(
      getTokens(BinaryExpression, 0, 1).map((token) => token.value),
      ['a', '*', 'b', 'call'],
    );
  });

  it('should retrieve all tokens plus two before and one after for binary expression', () => {
    assert.deepStrictEqual(
      getTokens(BinaryExpression, 2, 1).map((token) => token.value),
      ['answer', '=', 'a', '*', 'b', 'call'],
    );
  });

  it('should retrieve all matched tokens for root node with filter', () => {
    assert.deepStrictEqual(
      getTokens(Program, (t) => t.type === 'Identifier').map((token) => token.value),
      ['answer', 'a', 'b', 'call'],
    );
    assert.deepStrictEqual(
      getTokens(Program, {
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['answer', 'a', 'b', 'call'],
    );
  });

  it('should retrieve all tokens and comments in the node for root node with includeComments option', () => {
    assert.deepStrictEqual(
      getTokens(Program, { includeComments: true }).map((token) => token.value),
      ['var', 'answer', 'B', '=', 'C', 'a', 'D', '*', 'b', 'E', 'F', 'call', '(', ')', ';'],
    );
  });

  it('should retrieve matched tokens and comments in the node for root node with includeComments and filter options', () => {
    assert.deepStrictEqual(
      getTokens(Program, {
        includeComments: true,
        filter: (t) => t.type.startsWith('Block'),
      }).map((token) => token.value),
      ['B', 'C', 'D', 'E'],
    );
  });

  it('should retrieve all tokens and comments in the node for binary expression with includeComments option', () => {
    assert.deepStrictEqual(
      getTokens(BinaryExpression, { includeComments: true }).map((token) => token.value),
      ['a', 'D', '*', 'b'],
    );
  });

  it('should handle out of range nodes gracefully', () => {
    assert.deepStrictEqual(
      getTokens({ range: [1000, 1001] } as Node).map((token) => token.value),
      [],
    );
  });
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L157
describe('when calling getTokensBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokensBefore;
});

describe('when calling getTokenBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenBefore;
  /* oxlint-disable-next-line no-unused-expressions */
  resetSourceAndAst;
});

describe('when calling getTokenAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenAfter;
});

describe('when calling getTokensAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokensAfter;
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L594-L673
describe('when calling getFirstTokens', () => {
  it("should retrieve zero tokens from a node's token stream", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, 0).map((token) => token.value),
      [],
    );
  });

  it("should retrieve one token from a node's token stream", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, 1).map((token) => token.value),
      ['a'],
    );
  });

  it("should retrieve more than one token from a node's token stream", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, 2).map((token) => token.value),
      ['a', '*'],
    );
  });

  it("should retrieve all tokens from a node's token stream", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, 9e9).map((token) => token.value),
      ['a', '*', 'b'],
    );
  });

  it("should retrieve more than one token from a node's token stream with count option", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, { count: 2 }).map((token) => token.value),
      ['a', '*'],
    );
  });

  it("should retrieve matched tokens from a node's token stream with filter option", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, (t) => t.type === 'Identifier').map((token) => token.value),
      ['a', 'b'],
    );
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, {
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['a', 'b'],
    );
  });

  it("should retrieve matched tokens from a node's token stream with filter and count options", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, {
        count: 1,
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['a'],
    );
  });

  it("should retrieve all tokens and comments from a node's token stream with includeComments option", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, {
        includeComments: true,
      }).map((token) => token.value),
      ['a', 'D', '*', 'b'],
    );
  });

  it("should retrieve several tokens and comments from a node's token stream with includeComments and count options", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, {
        includeComments: true,
        count: 3,
      }).map((token) => token.value),
      ['a', 'D', '*'],
    );
  });

  it("should retrieve several tokens and comments from a node's token stream with includeComments and count and filter options", () => {
    assert.deepStrictEqual(
      getFirstTokens(BinaryExpression, {
        includeComments: true,
        count: 3,
        filter: (t) => t.value !== 'a',
      }).map((token) => token.value),
      ['D', '*', 'b'],
    );
  });
});

describe('when calling getFirstToken', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getLastTokens', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getLastToken', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getFirstTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getFirstTokenBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getLastTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getLastTokenBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getTokenByRangeStart', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getTokenOrCommentBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getTokenOrCommentAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getFirstToken & getTokenAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});

describe('when calling getLastToken & getTokenBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
});
