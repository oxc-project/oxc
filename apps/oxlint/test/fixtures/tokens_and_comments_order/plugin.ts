/**
 * Test that tokens/comments initialization works correctly regardless of the order
 * in which APIs are accessed.
 *
 * Each file in `files/` has the same content (hashbang + comments + tokens).
 * Files are named `001.js` through `120.js` (5! = 120 permutations).
 * The filename number determines the order in which 5 operations are performed.
 *
 * After performing all operations, we verify: 1. Each operation returns correct results. 2. Object
 * identity is preserved - the same token/comment objects are returned by all methods that return
 * them.
 */
import assert from "node:assert";

import type { Plugin, Rule, SourceCode, Token, Comment } from "#oxlint/plugins";

const SOURCE_TEXT = `#!/usr/bin/env node
// Leading comment
let x = /* inline */ 1;
let y = 2;
// Trailing comment
`;

// Expected snapshot strings for each operation.
// Each line is a JSON-serialized `{type, value, range}` for one token/comment.
// prettier-ignore
const TOKENS_SNAPSHOT = [
  '{"type":"Keyword","value":"let","range":[39,42]}',
  '{"type":"Identifier","value":"x","range":[43,44]}',
  '{"type":"Punctuator","value":"=","range":[45,46]}',
  '{"type":"Numeric","value":"1","range":[60,61]}',
  '{"type":"Punctuator","value":";","range":[61,62]}',
  '{"type":"Keyword","value":"let","range":[63,66]}',
  '{"type":"Identifier","value":"y","range":[67,68]}',
  '{"type":"Punctuator","value":"=","range":[69,70]}',
  '{"type":"Numeric","value":"2","range":[71,72]}',
  '{"type":"Punctuator","value":";","range":[72,73]}',
].join("\n");

// prettier-ignore
const COMMENTS_SNAPSHOT = [
  '{"type":"Shebang","value":"/usr/bin/env node","range":[0,19]}',
  '{"type":"Line","value":" Leading comment","range":[20,38]}',
  '{"type":"Block","value":" inline ","range":[47,59]}',
  '{"type":"Line","value":" Trailing comment","range":[74,93]}',
].join("\n");

// prettier-ignore
const TOKENS_AND_COMMENTS_SNAPSHOT = [
  '{"type":"Shebang","value":"/usr/bin/env node","range":[0,19]}',
  '{"type":"Line","value":" Leading comment","range":[20,38]}',
  '{"type":"Keyword","value":"let","range":[39,42]}',
  '{"type":"Identifier","value":"x","range":[43,44]}',
  '{"type":"Punctuator","value":"=","range":[45,46]}',
  '{"type":"Block","value":" inline ","range":[47,59]}',
  '{"type":"Numeric","value":"1","range":[60,61]}',
  '{"type":"Punctuator","value":";","range":[61,62]}',
  '{"type":"Keyword","value":"let","range":[63,66]}',
  '{"type":"Identifier","value":"y","range":[67,68]}',
  '{"type":"Punctuator","value":"=","range":[69,70]}',
  '{"type":"Numeric","value":"2","range":[71,72]}',
  '{"type":"Punctuator","value":";","range":[72,73]}',
  '{"type":"Line","value":" Trailing comment","range":[74,93]}',
].join("\n");

const LAST_TOKEN_SNAPSHOT = '{"type":"Punctuator","value":";","range":[72,73]}';

const LAST_COMMENT_SNAPSHOT = '{"type":"Line","value":" Trailing comment","range":[74,93]}';

// A node-like object covering the entire source text, used for token retrieval methods
const PROGRAM_NODE = {
  type: "Program" as const,
  start: 0,
  end: SOURCE_TEXT.length,
  range: [0, SOURCE_TEXT.length] as [number, number],
  loc: {
    start: { line: 1, column: 0 },
    end: { line: SOURCE_TEXT.split("\n").length, column: 0 },
  },
};

// Operation functions.
// Each returns a value that's stored and later checked for cross-consistency.

interface Results {
  tokens: Token[] | null;
  comments: Comment[] | null;
  tokensAndComments: (Token | Comment)[] | null;
  lastToken: Token | null;
  lastComment: Token | Comment | null;
}

type OpName = keyof Results;

const OP_NAMES: OpName[] = ["tokens", "comments", "tokensAndComments", "lastToken", "lastComment"];

function runOp(op: OpName, sourceCode: SourceCode): Results[OpName] {
  switch (op) {
    case "tokens":
      return opTokens(sourceCode);
    case "comments":
      return opComments(sourceCode);
    case "tokensAndComments":
      return opTokensAndComments(sourceCode);
    case "lastToken":
      return opLastToken(sourceCode);
    case "lastComment":
      return opLastComment(sourceCode);
  }
}

function opTokens(sourceCode: SourceCode): Token[] {
  const { tokens } = sourceCode.ast;

  assert.equal(snapAll(tokens), TOKENS_SNAPSHOT);

  // Calling again should return same array
  assert(sourceCode.ast.tokens === tokens, "`ast.tokens` getter should return cached array");

  return tokens;
}

function opComments(sourceCode: SourceCode): Comment[] {
  const comments = sourceCode.getAllComments();

  assert.equal(snapAll(comments), COMMENTS_SNAPSHOT);

  // Calling again should return same array
  assert(sourceCode.getAllComments() === comments, "`getAllComments()` should return cached array");

  return comments;
}

function opTokensAndComments(sourceCode: SourceCode): (Token | Comment)[] {
  const { tokensAndComments } = sourceCode;

  assert.equal(snapAll(tokensAndComments), TOKENS_AND_COMMENTS_SNAPSHOT);

  // Calling again should return same array
  assert(
    sourceCode.tokensAndComments === tokensAndComments,
    "`tokensAndComments` getter should return cached array",
  );

  return tokensAndComments;
}

function opLastToken(sourceCode: SourceCode): Token {
  const token = sourceCode.getLastToken(PROGRAM_NODE);
  assert(token !== null, "`getLastToken()` should return a token");

  assert.equal(snap(token), LAST_TOKEN_SNAPSHOT);

  return token;
}

function opLastComment(sourceCode: SourceCode): Token | Comment {
  const comment = sourceCode.getLastToken(PROGRAM_NODE, { includeComments: true });
  assert(comment !== null, "`getLastToken({ includeComments: true })` should return a comment");

  assert.equal(snap(comment), LAST_COMMENT_SNAPSHOT);

  return comment;
}

// ---- Rule ----

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;
    const { filename } = context;

    // Parse the number from the filename (e.g. "001.js" -> 0, "120.js" -> 119)
    const match = filename.match(/(\d+)\.js$/);
    if (!match) throw new Error(`Unexpected filename: ${filename}`);
    const permIndex = parseInt(match[1], 10) - 1; // 0-based

    // Determine the order of operations for this file
    const ops = nthPermutation(permIndex, OP_NAMES);

    // Run each operation in the determined order
    const results: Results = {
      tokens: null,
      comments: null,
      tokensAndComments: null,
      lastToken: null,
      lastComment: null,
    };

    for (const op of ops) {
      results[op] = runOp(op, sourceCode) as never;
    }

    // Verify source text.
    // Done AFTER all operations to ensure token/comment methods initialize source text themselves.
    assert.equal(sourceCode.text, SOURCE_TEXT, "Source text should match");

    // ---- Verify object identity across results ----

    const { tokens, comments, tokensAndComments, lastToken, lastComment } = results;

    // `lastToken` should be the same object as the last element of `tokens`
    assert.strictEqual(
      lastToken,
      tokens!.at(-1),
      "`lastToken` should be same object as `tokens[-1]`",
    );

    // `lastComment` should be the same object as the last element of `comments`
    assert.strictEqual(
      lastComment,
      comments!.at(-1),
      "`lastComment` should be same object as `comments[-1]`",
    );

    // Every token in `tokens` should appear in `tokensAndComments` (same object)
    for (const token of tokens!) {
      assert(
        tokensAndComments!.includes(token),
        `Token "${token.value}" at ${token.start} should be in \`tokensAndComments\` (same object)`,
      );
    }

    // Every comment in `comments` should appear in `tokensAndComments` (same object)
    for (const comment of comments!) {
      assert(
        tokensAndComments!.includes(comment),
        `Comment at ${comment.start} should be in \`tokensAndComments\` (same object)`,
      );
    }

    // Total count should match
    assert.equal(
      tokensAndComments!.length,
      tokens!.length + comments!.length,
      "`tokensAndComments` length should equal `tokens` + `comments` combined lengths",
    );

    // `lastToken` should be in `tokensAndComments`
    assert(
      tokensAndComments!.includes(lastToken!),
      "`lastToken` should be in `tokensAndComments` (same object)",
    );

    // `lastComment` should be in `tokensAndComments`
    assert(
      tokensAndComments!.includes(lastComment!),
      "`lastComment` should be in `tokensAndComments` (same object)",
    );

    context.report({
      message: `OK\nPermutation ${permIndex + 1}:\n[${ops.join(", ")}]`,
      node: PROGRAM_NODE,
    });

    return {};
  },
};

const plugin: Plugin = {
  meta: { name: "tokens-and-comments-order-plugin" },
  rules: { "tokens-and-comments-order": rule },
};

export default plugin;

// ---- Helpers ----

/**
 * Decode a 0-based permutation index into a permutation of `items`.
 * Uses the factorial number system (Lehmer code).
 *
 * @param n - Permutation index (0 to items.length! - 1)
 * @param items - Array of items to permute
 * @returns Permuted copy of `items`
 */
function nthPermutation<T>(n: number, items: T[]): T[] {
  const remaining = [...items];
  const result: T[] = [];
  for (let i = remaining.length; i > 0; i--) {
    const index = n % i;
    result.push(remaining[index]);
    remaining.splice(index, 1);
    n = (n - index) / i;
  }
  return result;
}

/**
 * Format a token/comment for snapshot comparison.
 * Only includes `type`, `value`, and `range` - excludes `loc` for brevity.
 */
function snap(entry: Token | Comment): string {
  return JSON.stringify({ type: entry.type, value: entry.value, range: entry.range });
}

/**
 * Format an array of tokens/comments for snapshot comparison.
 */
function snapAll(entries: (Token | Comment)[]): string {
  return entries.map(snap).join("\n");
}
