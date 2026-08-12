// Spacing between tokens.
//
// Two adjacent tokens can merge into a different token when written straight up against each other -
// `in` after an identifier, `+` after `+`. Each of these is called before writing the second one,
// and writes a space only where it would otherwise happen.

import { debugAssert } from "../asserts.ts";
import {
  ALL_CATEGORIES,
  CAT_IDENT,
  CAT_INT_DIGIT,
  CAT_OP_UN_NEG,
  CAT_OP_UN_NOT_AFTER_LT,
  CAT_OP_UN_PLUS,
  CAT_OP_UPD_DEC,
  CAT_OP_UPD_INC,
  CAT_OTHER,
  CAT_REGEX_SLASH,
  debugAssertLastFresh,
  write,
} from "./write.ts";

import type { Category } from "./write.ts";
import type { State } from "../state.ts";

// `printSpaceBeforeIdentifier` selects the categories needing a space by their position in `Category` numbering.
// Check `category <= CAT_REGEX_SLASH` matches the intended categories, and no others.
if (DEBUG) {
  for (const category of ALL_CATEGORIES) {
    const expected = [CAT_IDENT, CAT_INT_DIGIT, CAT_REGEX_SLASH].includes(category);
    const actual = category <= CAT_REGEX_SLASH;
    debugAssert(
      actual === expected,
      `Category ${category} disagrees with \`last <= CAT_REGEX_SLASH\``,
    );
  }
}

/**
 * Write a space, if what was written last would otherwise run into an identifier.
 *
 * Call it before writing anything starting with an identifier character - a name, a keyword, or a
 * number. The categories needing the space are the lowest codes in the numbering, so the test is
 * one compare.
 */
export function printSpaceBeforeIdentifier(state: State): void {
  debugAssertLastFresh(state);

  // `last` starts as `CAT_OTHER` (start of output behaves like after whitespace),
  // so no empty-output check is needed.
  // Everything needing a space before an identifier is one of the lowest codes, so this is one compare -
  // identifier characters, a plain-digit number, and a regex closed with no flags.
  if (state.last <= CAT_REGEX_SLASH) write(state, " ", CAT_OTHER);
}

// `printSpaceBeforeOperator` selects the categories needing a space by their position in `Category` numbering.
// Check `category >= CAT_OP_UN_NOT_AFTER_LT` matches the intended categories, and no others.
// `CAT_OP_UN_NOT` is deliberately absent - `printSpaceBeforeOperatorSlow` has no clause for a
// plain `!`, so it sits below the range and storing it costs the slow path nothing.
if (DEBUG) {
  for (const category of ALL_CATEGORIES) {
    const expected = [
      CAT_OP_UN_NOT_AFTER_LT,
      CAT_OP_UN_PLUS,
      CAT_OP_UPD_INC,
      CAT_OP_UN_NEG,
      CAT_OP_UPD_DEC,
    ].includes(category);
    const actual = category >= CAT_OP_UN_NOT_AFTER_LT;
    debugAssert(
      actual === expected,
      `Category ${category} disagrees with \`last >= CAT_OP_UN_NOT_AFTER_LT\``,
    );
  }
}

/**
 * Write a space, if the operator about to be written would otherwise merge with the one before it.
 *
 * `+ +y` and `- -y` are the cases - `++y` and `--y` would be a different operator.
 * An operator category in `last` proves that operator was the immediately preceding token,
 * because every write replaces `last`, so no position has to be recorded alongside it.
 *
 * @param next - Category of the operator about to be written
 */
export function printSpaceBeforeOperator(state: State, next: Category): void {
  debugAssertLastFresh(state);

  // The slow path only runs when an operator it distinguishes was the immediately preceding token,
  // which is rare in pretty output. Keep the hot check inlinable.
  const prev = state.last;
  if (prev >= CAT_OP_UN_NOT_AFTER_LT) printSpaceBeforeOperatorSlow(state, prev, next);
}

/**
 * The pairs of adjacent operators which need separating, kept out of `printSpaceBeforeOperator`
 * so the common path stays small enough to inline.
 *
 * In pretty mode binary operators are written space-padded, so they never leave an operator code in `last`
 * and are never passed as `next` - only unary and update operators reach here.
 * Oxc's `print_space_before_operator` also has clauses for the binary cases, but those were already
 * unreachable here, and there is now no code which could produce a binary operator's category.
 *
 * Only prefix operators appear as `next` - `+ +y`, `- --y` - which is why `prev` being `++` or
 * `--` matches no clause. That is not a gap: in pretty output a postfix `++`/`--` is always
 * followed by punctuation or a padded binary operator, never directly by another operator. (The
 * asymmetry is the Rust original's, where it is live in minified mode - unpadded `(x++)+y` must
 * print `x+++y` with no space, while unary `+ +y` must keep one.)
 *
 * @param prev - Category of the operator written last
 * @param next - Category of the operator about to be written
 */
function printSpaceBeforeOperatorSlow(state: State, prev: Category, next: Category): void {
  if (
    (prev === CAT_OP_UN_PLUS && (next === CAT_OP_UN_PLUS || next === CAT_OP_UPD_INC)) ||
    (prev === CAT_OP_UN_NEG && (next === CAT_OP_UN_NEG || next === CAT_OP_UPD_DEC)) ||
    (prev === CAT_OP_UN_NOT_AFTER_LT && next === CAT_OP_UPD_DEC)
  ) {
    write(state, " ", CAT_OTHER);
  }
}
