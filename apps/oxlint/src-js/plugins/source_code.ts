import type { Node } from './types.ts';

const { max } = Math;

/**
 * `SourceCode` class.
 *
 * Each rule has its own `SourceCode` object. It is stored in `Context` for that rule.
 *
 * A new `SourceCode` instance is NOT generated for each file.
 * The `SourceCode` instance for the rule is updated for each file.
 */
export class SourceCode {
  // Source text.
  // Initially `null`, but set to source text string before linting each file.
  text: string = null as unknown as string;

  getText(
    node?: Node | null | undefined,
    beforeCount?: number | null | undefined,
    afterCount?: number | null | undefined,
  ): string {
    // ESLint treats all falsy values for `node` as undefined
    if (!node) return this.text;

    // ESLint ignores falsy values for `beforeCount` and `afterCount`
    let { start, end } = node;
    if (beforeCount) start = max(start - beforeCount, 0);
    if (afterCount) end += afterCount;
    return this.text.slice(start, end);
  }

  // TODO: Add more methods
}
