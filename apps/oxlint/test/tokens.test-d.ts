/**
 * Type tests for token method return types.
 *
 * These are compile-time only — they don't run any code, they just verify that the conditional return types
 * resolve correctly for various calling patterns.
 *
 * Enforced by the type-checker: If any `satisfies` check fails, type check will error.
 *
 * `getTokens` and `getFirstToken` are tested exhaustively because they represent the two return type patterns:
 * 1. Array - `TokenResult<Options>[]`
 * 2. Single - `TokenResult<Options> | null`
 *
 * All other conditional-return methods use the same `TokenResult` type, so they only get minimal tests:
 * 1. No options -> `Token`
 * 2. `{ includeComments: true }` -> `TokenOrComment`
 * This guards against a method accidentally missing the `TokenResult` return type.
 */

import {
  getFirstToken,
  getFirstTokenBetween,
  getFirstTokens,
  getFirstTokensBetween,
  getLastToken,
  getLastTokenBetween,
  getLastTokens,
  getLastTokensBetween,
  getTokenAfter,
  getTokenBefore,
  getTokenByRangeStart,
  getTokens,
  getTokensAfter,
  getTokensBefore,
  getTokensBetween,
} from "../src-js/plugins/tokens_methods.ts";

import type { Node } from "../src-js/plugins/types.ts";
import type { Token } from "../src-js/plugins/tokens.ts";
import type { TokenOrComment } from "../src-js/plugins/tokens_and_comments.ts";
import type { CountOptions, SkipOptions } from "../src-js/plugins/tokens_methods.ts";

type IsExact<T, U> = [T] extends [U] ? ([U] extends [T] ? true : false) : false;

declare const node: Node;

// --- `getTokens` ---

// No options -> `Token[]`
{
  const result = getTokens(node);
  true satisfies IsExact<typeof result, Token[]>;
}

// `null` options -> `Token[]`
{
  const result = getTokens(node, null);
  true satisfies IsExact<typeof result, Token[]>;
}

// `undefined` options -> `Token[]`
{
  const result = getTokens(node, undefined);
  true satisfies IsExact<typeof result, Token[]>;
}

// Empty options object -> `Token[]`
{
  const opts = {};
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, Token[]>;
}

// `{ includeComments: true }` -> `TokenOrComment[]`
{
  const result = getTokens(node, { includeComments: true });
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// `{ includeComments: false }` -> `Token[]`
{
  const result = getTokens(node, { includeComments: false });
  true satisfies IsExact<typeof result, Token[]>;
}

// Variable boolean (widened) -> `TokenOrComment[]` (conservative)
{
  const flag: boolean = true;
  const result = getTokens(node, { includeComments: flag });
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options object assigned to variable (`true` widened to `boolean`) -> `TokenOrComment[]`
{
  const opts = { includeComments: true };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options object assigned to variable (`false` widened to `boolean`) -> `TokenOrComment[]` (conservative)
{
  const opts = { includeComments: false };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options object with `as const` (true) -> `TokenOrComment[]`
{
  const opts = { includeComments: true } as const;
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options object with `as const` (false) -> `Token[]`
{
  const opts = { includeComments: false } as const;
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, Token[]>;
}

// Options with `count` only -> `Token[]`
{
  const opts = { count: 3 };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, Token[]>;
}

// Options with `filter` only -> `Token[]`
{
  const opts = { filter: (token: TokenOrComment) => token.type === "Keyword" };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, Token[]>;
}

// Options with `count` and `includeComments: true` -> `TokenOrComment[]`
{
  const opts = { count: 3, includeComments: true };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options with `count` and `includeComments: false` inline -> `Token[]`
{
  const result = getTokens(node, { count: 3, includeComments: false });
  true satisfies IsExact<typeof result, Token[]>;
}

// Options with `count` and `includeComments: false` -> `TokenOrComment[]` (conservative)
{
  const opts = { count: 3, includeComments: false };
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options with `count` and `includeComments: true` with `as const` -> `TokenOrComment[]`
{
  const opts = { count: 3, includeComments: true } as const;
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Options with `count` and `includeComments: false` with `as const` -> `Token[]`
{
  const opts = { count: 3, includeComments: false } as const;
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, Token[]>;
}

// Number option -> `Token[]`
{
  const result = getTokens(node, 5);
  true satisfies IsExact<typeof result, Token[]>;
}

// Filter function -> `Token[]`
{
  const result = getTokens(node, (token) => token.type === "Keyword");
  true satisfies IsExact<typeof result, Token[]>;
}

// Variable typed as `CountOptions` -> `TokenOrComment[]` (conservative)
{
  const opts = {} as CountOptions;
  const result = getTokens(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// Variable typed as the full union -> should be `TokenOrComment[]` but resolves to `Token[]`.
// The union distributes `MayIncludeComments` to `true | false` = `boolean`,
// which doesn't extend `true`, so it falls to `Token[]`.
// This is a known limitation, but this calling pattern is unrealistic in practice.
{
  const opts = {} as CountOptions | number | null | undefined;
  const result = getTokens(node, opts);
  // @ts-expect-error — see above
  true satisfies IsExact<typeof result, TokenOrComment[]>;
}

// --- `getFirstToken` ---

// No options -> `Token | null`
{
  const result = getFirstToken(node);
  true satisfies IsExact<typeof result, Token | null>;
}

// `null` options -> `Token | null`
{
  const result = getFirstToken(node, null);
  true satisfies IsExact<typeof result, Token | null>;
}

// `undefined` options -> `Token | null`
{
  const result = getFirstToken(node, undefined);
  true satisfies IsExact<typeof result, Token | null>;
}

// Empty options object -> `Token | null`
{
  const opts = {};
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, Token | null>;
}

// `{ includeComments: true }` -> `TokenOrComment | null`
{
  const result = getFirstToken(node, { includeComments: true });
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// `{ includeComments: false }` -> `Token | null`
{
  const result = getFirstToken(node, { includeComments: false });
  true satisfies IsExact<typeof result, Token | null>;
}

// Variable boolean (widened) -> `TokenOrComment | null` (conservative)
{
  const flag: boolean = true;
  const result = getFirstToken(node, { includeComments: flag });
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options object assigned to variable (`true` widened to `boolean`) -> `TokenOrComment | null`
{
  const opts = { includeComments: true };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options object assigned to variable (`false` widened to `boolean`) -> `TokenOrComment | null` (conservative)
{
  const opts = { includeComments: false };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options object with `as const` (true) -> `TokenOrComment | null`
{
  const opts = { includeComments: true } as const;
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options object with `as const` (false) -> `Token | null`
{
  const opts = { includeComments: false } as const;
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, Token | null>;
}

// Options with `skip` only -> `Token | null`
{
  const opts = { skip: 1 };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, Token | null>;
}

// Options with `filter` only -> `Token | null`
{
  const opts = { filter: (token: TokenOrComment) => token.type === "Keyword" };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, Token | null>;
}

// Options with `skip` and `includeComments: true` -> `TokenOrComment | null`
{
  const opts = { skip: 1, includeComments: true };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options with `skip` and `includeComments: false` inline -> `Token | null`
{
  const result = getFirstToken(node, { skip: 1, includeComments: false });
  true satisfies IsExact<typeof result, Token | null>;
}

// Options with `skip` and `includeComments: false` -> `TokenOrComment | null` (conservative)
{
  const opts = { skip: 1, includeComments: false };
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options with `skip` and `includeComments: true` with `as const` -> `TokenOrComment | null`
{
  const opts = { skip: 1, includeComments: true } as const;
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// Options with `skip` and `includeComments: false` with `as const` -> `Token | null`
{
  const opts = { skip: 1, includeComments: false } as const;
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, Token | null>;
}

// Number option -> `Token | null`
{
  const result = getFirstToken(node, 5);
  true satisfies IsExact<typeof result, Token | null>;
}

// Filter function -> `Token | null`
{
  const result = getFirstToken(node, (token) => token.type === "Keyword");
  true satisfies IsExact<typeof result, Token | null>;
}

// Variable typed as `SkipOptions` -> `TokenOrComment | null` (conservative)
{
  const opts = {} as SkipOptions;
  const result = getFirstToken(node, opts);
  true satisfies IsExact<typeof result, TokenOrComment | null>;
}

// --- Minimal tests for remaining methods ---

// Each method gets two tests:
// 1. No options -> `Token`
// 2. `{ includeComments: true }` -> `TokenOrComment`

// `getFirstTokens`
{
  const noOptions = getFirstTokens(node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getFirstTokens(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getLastToken`
{
  const noOptions = getLastToken(node);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getLastToken(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}

// `getLastTokens`
{
  const noOptions = getLastTokens(node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getLastTokens(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getTokenBefore`
{
  const noOptions = getTokenBefore(node);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getTokenBefore(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}

// `getTokenAfter`
{
  const noOptions = getTokenAfter(node);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getTokenAfter(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}

// `getTokensBefore`
{
  const noOptions = getTokensBefore(node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getTokensBefore(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getTokensAfter`
{
  const noOptions = getTokensAfter(node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getTokensAfter(node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getTokensBetween`
{
  const noOptions = getTokensBetween(node, node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getTokensBetween(node, node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getFirstTokenBetween`
{
  const noOptions = getFirstTokenBetween(node, node);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getFirstTokenBetween(node, node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}

// `getFirstTokensBetween`
{
  const noOptions = getFirstTokensBetween(node, node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getFirstTokensBetween(node, node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getLastTokenBetween`
{
  const noOptions = getLastTokenBetween(node, node);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getLastTokenBetween(node, node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}

// `getLastTokensBetween`
{
  const noOptions = getLastTokensBetween(node, node);
  true satisfies IsExact<typeof noOptions, Token[]>;
  const withOptions = getLastTokensBetween(node, node, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment[]>;
}

// `getTokenByRangeStart`
{
  const noOptions = getTokenByRangeStart(0);
  true satisfies IsExact<typeof noOptions, Token | null>;
  const withOptions = getTokenByRangeStart(0, { includeComments: true });
  true satisfies IsExact<typeof withOptions, TokenOrComment | null>;
}
