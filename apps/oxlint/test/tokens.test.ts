import assert from 'node:assert';
import { beforeEach, describe, it, vi } from 'vitest';
import {
  getTokens,
  getTokensBefore,
  getTokenBefore,
  getTokensAfter,
  getTokenAfter,
  getFirstTokens,
  getFirstToken,
  getLastTokens,
  getLastToken,
  getFirstTokensBetween,
  getFirstTokenBetween,
  getLastTokenBetween,
  getLastTokensBetween,
  getTokenByRangeStart,
  getTokensBetween,
  getTokenOrCommentBefore,
  getTokenOrCommentAfter,
} from '../src-js/plugins/tokens.js';
import { resetSourceAndAst } from '../src-js/plugins/source_code.js';
import type { Node } from '../src-js/plugins/types.js';

// Source text used for most tests
const SOURCE_TEXT = '/*A*/var answer/*B*/=/*C*/a/*D*/* b/*E*///F\n    call();\n/*Z*/';

// Mock `source_code.ts` to inject source text from `sourceText` defined here
let sourceText: string;

vi.mock('../src-js/plugins/source_code.ts', async (importOriginal) => {
  const original: any = await importOriginal();
  return {
    ...original,
    get sourceText() {
      return sourceText;
    },
  };
});

// Reset global state and set source text to `SOURCE_TEXT` before each test.
// Individual tests can set `sourceText` to a different value if required before calling token methods.
beforeEach(() => {
  resetSourceAndAst();
  sourceText = SOURCE_TEXT;
});

// TODO: We are lying about `Program`'s range here.
// The range provided by `@typescript-eslint/typescript-estree` does not match the assertions for that of `espree`.
// The deviation is being corrected in upcoming releases of ESLint and TS-ESLint.
// https://eslint.org/blog/2025/10/whats-coming-in-eslint-10.0.0/#updates-to-program-ast-node-range-coverage
// https://github.com/typescript-eslint/typescript-eslint/issues/11026#issuecomment-3421887632
const Program = { range: [5, 55] } as Node;
const BinaryExpression = { range: [26, 35] } as Node;
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
  it('should retrieve zero tokens before a node', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, 0).map((token) => token.value),
      [],
    );
  });

  it('should retrieve one token before a node', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, 1).map((token) => token.value),
      ['='],
    );
  });

  it('should retrieve more than one token before a node', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, 2).map((token) => token.value),
      ['answer', '='],
    );
  });

  it('should retrieve all tokens before a node', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, 9e9).map((token) => token.value),
      ['var', 'answer', '='],
    );
  });

  it('should retrieve more than one token before a node with count option', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, { count: 2 }).map((token) => token.value),
      ['answer', '='],
    );
  });

  it('should retrieve matched tokens before a node with count and filter options', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, {
        count: 1,
        filter: (t) => t.value !== '=',
      }).map((token) => token.value),
      ['answer'],
    );
  });

  it('should retrieve all matched tokens before a node with filter option', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, {
        filter: (t) => t.value !== 'answer',
      }).map((token) => token.value),
      ['var', '='],
    );
  });

  it('should retrieve no tokens before the root node', () => {
    assert.deepStrictEqual(
      getTokensBefore(Program, { count: 1 }).map((token) => token.value),
      [],
    );
  });

  it('should retrieve tokens and comments before a node with count and includeComments option', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, {
        count: 3,
        includeComments: true,
      }).map((token) => token.value),
      ['B', '=', 'C'],
    );
  });

  it('should retrieve all tokens and comments before a node with includeComments option only', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, {
        includeComments: true,
      }).map((token) => token.value),
      ['A', 'var', 'answer', 'B', '=', 'C'],
    );
  });

  it('should retrieve all tokens and comments before a node with includeComments and filter options', () => {
    assert.deepStrictEqual(
      getTokensBefore(BinaryExpression, {
        includeComments: true,
        filter: (t) => t.type.startsWith('Block'),
      }).map((token) => token.value),
      ['A', 'B', 'C'],
    );
  });
});

describe('when calling getTokenBefore', () => {
  it('should retrieve one token before a node', () => {
    assert.strictEqual(getTokenBefore(BinaryExpression)!.value, '=');
  });

  it('should skip a given number of tokens', () => {
    assert.strictEqual(getTokenBefore(BinaryExpression, 1)!.value, 'answer');
    assert.strictEqual(getTokenBefore(BinaryExpression, 2)!.value, 'var');
  });

  it('should skip a given number of tokens with skip option', () => {
    assert.strictEqual(getTokenBefore(BinaryExpression, { skip: 1 })!.value, 'answer');
    assert.strictEqual(getTokenBefore(BinaryExpression, { skip: 2 })!.value, 'var');
  });

  it('should retrieve matched token with filter option', () => {
    assert.strictEqual(getTokenBefore(BinaryExpression, (t) => t.value !== '=')!.value, 'answer');
  });

  it('should retrieve matched token with skip and filter options', () => {
    assert.strictEqual(
      getTokenBefore(BinaryExpression, {
        skip: 1,
        filter: (t) => t.value !== '=',
      })!.value,
      'var',
    );
  });

  it('should retrieve one token or comment before a node with includeComments option', () => {
    assert.strictEqual(
      getTokenBefore(BinaryExpression, {
        includeComments: true,
      })!.value,
      'C',
    );
  });

  it('should retrieve one token or comment before a node with includeComments and skip options', () => {
    assert.strictEqual(
      getTokenBefore(BinaryExpression, {
        includeComments: true,
        skip: 1,
      })!.value,
      '=',
    );
  });

  it('should retrieve one token or comment before a node with includeComments and skip and filter options', () => {
    assert.strictEqual(
      getTokenBefore(BinaryExpression, {
        includeComments: true,
        skip: 1,
        filter: (t) => t.type.startsWith('Block'),
      })!.value,
      'B',
    );
  });

  it('should retrieve the previous node if the comment at the end of source code is specified.', () => {
    sourceText = 'a + b /*comment*/';
    // TODO: this verbatim range should be replaced with `ast.comments[0]`
    const token = getTokenBefore({ range: [6, 17] } as Node);
    assert.strictEqual(token!.value, 'b');
  });

  it('should retrieve the previous comment if the first token is specified.', () => {
    sourceText = '/*comment*/ a + b';
    // TODO: this verbatim range should be replaced with `ast.tokens[0]`
    const token = getTokenBefore({ range: [12, 13] } as Node, { includeComments: true });
    assert.strictEqual(token!.value, 'comment');
  });

  it('should retrieve null if the first comment is specified.', () => {
    sourceText = '/*comment*/ a + b';
    // TODO: this verbatim range should be replaced with `ast.comments[0]`
    const token = getTokenBefore({ range: [0, 11] } as Node, { includeComments: true });
    assert.strictEqual(token, null);
  });
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L461
describe('when calling getTokenAfter', () => {
  it('should retrieve one token after a node', () => {
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier)!.value, '=');
  });

  it('should skip a given number of tokens', () => {
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier, 1)!.value, 'a');
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier, 2)!.value, '*');
  });

  it('should skip a given number of tokens with skip option', () => {
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier, { skip: 1 })!.value, 'a');
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier, { skip: 2 })!.value, '*');
  });

  it('should retrieve matched token with filter option', () => {
    assert.strictEqual(getTokenAfter(VariableDeclaratorIdentifier, (t) => t.type === 'Identifier')!.value, 'a');
    assert.strictEqual(
      getTokenAfter(VariableDeclaratorIdentifier, {
        filter: (t) => t.type === 'Identifier',
      })!.value,
      'a',
    );
  });

  it('should retrieve matched token with filter and skip options', () => {
    assert.strictEqual(
      getTokenAfter(VariableDeclaratorIdentifier, {
        skip: 1,
        filter: (t) => t.type === 'Identifier',
      })!.value,
      'b',
    );
  });

  it('should retrieve one token or comment after a node with includeComments option', () => {
    assert.strictEqual(
      getTokenAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
      })!.value,
      'B',
    );
  });

  it('should retrieve one token or comment after a node with includeComments and skip options', () => {
    assert.strictEqual(
      getTokenAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
        skip: 2,
      })!.value,
      'C',
    );
  });

  it('should retrieve one token or comment after a node with includeComments and skip and filter options', () => {
    assert.strictEqual(
      getTokenAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
        skip: 2,
        filter: (t) => t.type.startsWith('Block'),
      })!.value,
      'D',
    );
  });

  it('should retrieve the next node if the comment at the first of source code is specified.', () => {
    sourceText = '/*comment*/ a + b';
    // TODO: replace this verbatim range with `ast.comments[0]`
    const token = getTokenAfter({ range: [0, 12] } as Node)!;
    assert.strictEqual(token.value, 'a');
  });

  it('should retrieve the next comment if the last token is specified.', () => {
    sourceText = 'a + b /*comment*/';
    // TODO: replace this verbatim range with `ast.tokens[2]`
    const token = getTokenAfter({ range: [4, 5] } as Node, {
      includeComments: true,
    });
    assert.strictEqual(token!.value, 'comment');
  });

  it('should retrieve null if the last comment is specified.', () => {
    sourceText = 'a + b /*comment*/';
    // TODO: replace this verbatim range with `ast.comments[0]`
    const token = getTokenAfter({ range: [6, 17] } as Node, {
      includeComments: true,
    });
    assert.strictEqual(token, null);
  });
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L363-L459
describe('when calling getTokensAfter', () => {
  it('should retrieve zero tokens after a node', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, 0).map((token) => token.value),
      [],
    );
  });

  it('should retrieve one token after a node', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, 1).map((token) => token.value),
      ['='],
    );
  });

  it('should retrieve more than one token after a node', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, 2).map((token) => token.value),
      ['=', 'a'],
    );
  });

  it('should retrieve all tokens after a node', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, 9e9).map((token) => token.value),
      ['=', 'a', '*', 'b', 'call', '(', ')', ';'],
    );
  });

  it('should retrieve more than one token after a node with count option', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, { count: 2 }).map((token) => token.value),
      ['=', 'a'],
    );
  });

  it('should retrieve all matched tokens after a node with filter option', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, {
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['a', 'b', 'call'],
    );
  });

  it('should retrieve matched tokens after a node with count and filter options', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, {
        count: 2,
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['a', 'b'],
    );
  });

  it('should retrieve all tokens and comments after a node with includeComments option', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
      }).map((token) => token.value),
      ['B', '=', 'C', 'a', 'D', '*', 'b', 'E', 'F', 'call', '(', ')', ';', 'Z'],
    );
  });

  it('should retrieve several tokens and comments after a node with includeComments and count options', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
        count: 3,
      }).map((token) => token.value),
      ['B', '=', 'C'],
    );
  });

  it('should retrieve matched tokens and comments after a node with includeComments and count and filter options', () => {
    assert.deepStrictEqual(
      getTokensAfter(VariableDeclaratorIdentifier, {
        includeComments: true,
        count: 3,
        filter: (t) => t.type.startsWith('Block'),
      }).map((token) => token.value),
      ['B', 'C', 'D'],
    );
  });
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

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L675-L849
describe('when calling getFirstToken', () => {
  it("should retrieve the first token of a node's token stream", () => {
    assert.strictEqual(getFirstToken(BinaryExpression)!.value, 'a');
  });

  it('should skip a given number of tokens', () => {
    assert.strictEqual(getFirstToken(BinaryExpression, 1)!.value, '*');
    assert.strictEqual(getFirstToken(BinaryExpression, 2)!.value, 'b');
  });

  it('should skip a given number of tokens with skip option', () => {
    assert.strictEqual(getFirstToken(BinaryExpression, { skip: 1 })!.value, '*');
    assert.strictEqual(getFirstToken(BinaryExpression, { skip: 2 })!.value, 'b');
  });

  it('should retrieve matched token with filter option', () => {
    assert.strictEqual(getFirstToken(BinaryExpression, (t) => t.type === 'Identifier')!.value, 'a');
    assert.strictEqual(
      getFirstToken(BinaryExpression, {
        filter: (t) => t.type === 'Identifier',
      })!.value,
      'a',
    );
  });

  it('should retrieve matched token with filter and skip options', () => {
    assert.strictEqual(
      getFirstToken(BinaryExpression, {
        skip: 1,
        filter: (t) => t.type === 'Identifier',
      })!.value,
      'b',
    );
  });

  it("should retrieve the first token or comment of a node's token stream with includeComments option", () => {
    assert.strictEqual(getFirstToken(BinaryExpression, { includeComments: true })!.value, 'a');
  });

  it("should retrieve the first matched token or comment of a node's token stream with includeComments and skip options", () => {
    assert.strictEqual(
      getFirstToken(BinaryExpression, {
        includeComments: true,
        skip: 1,
      })!.value,
      'D',
    );
  });

  it("should retrieve the first matched token or comment of a node's token stream with includeComments and skip and filter options", () => {
    assert.strictEqual(
      getFirstToken(BinaryExpression, {
        includeComments: true,
        skip: 1,
        filter: (t) => t.value !== 'a',
      })!.value,
      '*',
    );
  });

  it('should retrieve the first comment if the comment is at the last of nodes', () => {
    sourceText = 'a + b\n/*comment*/ c + d';
    /*
     * A node must not start with a token: it can start with a comment or be empty.
     * This test case is needed for completeness.
     */
    assert.strictEqual(
      getFirstToken(
        // TODO: this verbatim range should be replaced with `[ast.comments[0].range[0], ast.tokens[5].range[1]]`
        { range: [6, 23] } as Node,
        { includeComments: true },
      )!.value,
      'comment',
    );
  });

  it('should retrieve the first token (without includeComments option) if the comment is at the last of nodes', () => {
    sourceText = 'a + b\n/*comment*/ c + d';
    /*
     * A node must not start with a token: it can start with a comment or be empty.
     * This test case is needed for completeness.
     */
    assert.strictEqual(
      getFirstToken({
        // TODO: this verbatim range should be replaced with `[ast.comments[0].range[0], ast.tokens[5].range[1]]`
        range: [6, 23],
      } as Node)!.value,
      'c',
    );
  });

  it('should retrieve the first token if the root node contains a trailing comment', () => {
    sourceText = 'foo // comment';
    // TODO: this verbatim range should be replaced with `ast`
    assert.strictEqual(getFirstToken({ range: [0, 14] } as Node)!.value, 'foo');
  });

  it('should return null if the source contains only comments', () => {
    sourceText = '// comment';
    // TODO: this verbatim range should be replaced with `ast`
    assert.strictEqual(
      getFirstToken({ range: [0, 11] } as Node, {
        filter() {
          assert.fail('Unexpected call to filter callback');
        },
      }),
      null,
    );
  });

  it('should return null if the source is empty', () => {
    sourceText = '';
    // TODO: this verbatim range should be replaced with `ast`
    assert.strictEqual(getFirstToken({ range: [0, 0] } as Node), null);
  });
});

// https://github.com/eslint/eslint/blob/v9.39.1/tests/lib/languages/js/source-code/token-store.js#L851-L930
describe('when calling getLastTokens', () => {
  it("should retrieve zero tokens from the end of a node's token stream", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, 0).map((token) => token.value),
      [],
    );
  });

  it("should retrieve one token from the end of a node's token stream", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, 1).map((token) => token.value),
      ['b'],
    );
  });

  it("should retrieve more than one token from the end of a node's token stream", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, 2).map((token) => token.value),
      ['*', 'b'],
    );
  });

  it("should retrieve all tokens from the end of a node's token stream", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, 9e9).map((token) => token.value),
      ['a', '*', 'b'],
    );
  });

  it("should retrieve more than one token from the end of a node's token stream with count option", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, { count: 2 }).map((token) => token.value),
      ['*', 'b'],
    );
  });

  it("should retrieve matched tokens from the end of a node's token stream with filter option", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, (t) => t.type === 'Identifier').map((token) => token.value),
      ['a', 'b'],
    );
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, {
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['a', 'b'],
    );
  });

  it("should retrieve matched tokens from the end of a node's token stream with filter and count options", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, {
        count: 1,
        filter: (t) => t.type === 'Identifier',
      }).map((token) => token.value),
      ['b'],
    );
  });

  it("should retrieve all tokens from the end of a node's token stream with includeComments option", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, {
        includeComments: true,
      }).map((token) => token.value),
      ['a', 'D', '*', 'b'],
    );
  });

  it("should retrieve matched tokens from the end of a node's token stream with includeComments and count options", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, {
        includeComments: true,
        count: 3,
      }).map((token) => token.value),
      ['D', '*', 'b'],
    );
  });

  it("should retrieve matched tokens from the end of a node's token stream with includeComments and count and filter options", () => {
    assert.deepStrictEqual(
      getLastTokens(BinaryExpression, {
        includeComments: true,
        count: 3,
        filter: (t) => t.type !== 'Punctuator',
      }).map((token) => token.value),
      ['a', 'D', 'b'],
    );
  });
});

describe('when calling getLastToken', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getLastToken;
});

describe('when calling getFirstTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getFirstTokensBetween;
});

describe('when calling getFirstTokenBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getFirstTokenBetween;
});

describe('when calling getLastTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getLastTokensBetween;
});

describe('when calling getLastTokenBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getLastTokenBetween;
});

describe('when calling getTokensBetween', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokensBetween;
});

describe('when calling getTokenByRangeStart', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenByRangeStart;
});

describe('when calling getTokenOrCommentBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenOrCommentBefore;
});

describe('when calling getTokenOrCommentAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenOrCommentAfter;
});

describe('when calling getFirstToken & getTokenAfter', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getFirstToken;
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenAfter;
});

describe('when calling getLastToken & getTokenBefore', () => {
  /* oxlint-disable-next-line no-disabled-tests expect-expect */
  it('is to be implemented');
  /* oxlint-disable-next-line no-unused-expressions */
  getLastToken;
  /* oxlint-disable-next-line no-unused-expressions */
  getTokenBefore;
});
