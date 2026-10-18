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
  CAT_OP_LT_THEN_UN_NOT,
  CAT_OP_UN_PLUS,
  CAT_OP_UPD_DEC,
  CAT_OP_UPD_INC,
  CAT_OTHER,
  CAT_REGEX_SLASH,
} from "./categories.ts";
import { debugAssertLastFresh, write } from "./write.ts";

import type { Category } from "./categories.ts";
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
 *
 * @param state - Printer state
 */
export function printSpaceBeforeIdentifier(state: State): void {
  debugAssertLastFresh(state);

  // Pretty output never needs this space, so this function is always a no-op.
  // Only a minifying printer needs the check.
  //
  // `last` is never one of the categories the check looks for:
  //
  // - The next thing written after a keyword is always either a space (`return x`, `function f`),
  //   or punctuation (`return;`, `default:`, `function(`, `function<`).
  //   A space replaces the keyword's `CAT_IDENT` with `CAT_OTHER` before anything which calls this.
  //   Punctuation does not start with an identifier character, so is never preceded by a call to this.
  //
  // - Everything else which writes one of the relevant categories ends a name, an expression or a type:
  //   - an identifier, `this` or `true` (`CAT_IDENT`)
  //   - a number (`CAT_INT_DIGIT`)
  //   - a regex (`CAT_REGEX_SLASH`)
  //   Nothing starting with an identifier character is ever written straight after one of those.
  //   The keywords which can follow them (`in`, `of`, `as`, `from`, `extends` etc)
  //   are always written with a space before them, so do not call this function.
  //
  // Release builds remove the calls entirely. Debug builds keep the assertion above,
  // to ensure `last` stays fresh at every call site, ready for a minify mode.
  if (!MINIFY) {
    debugAssert(state.last > CAT_REGEX_SLASH);
    return;
  }

  // `last` starts as `CAT_OTHER` (start of output behaves like after whitespace),
  // so no empty-output check is needed.
  // Everything needing a space before an identifier is one of the lowest codes, so this is one compare -
  // identifier characters, a plain-digit number, and a regex closed with no flags.
  if (state.last <= CAT_REGEX_SLASH) write(state, " ", CAT_OTHER);
}

// `printSpaceBeforeOperator` selects the categories needing a space by their position in `Category` numbering.
// Check `category >= CAT_OP_LT_THEN_UN_NOT` matches the intended categories, and no others.
// `CAT_OP_UN_NOT` is deliberately absent - `printSpaceBeforeOperatorSlow` has no clause for a
// plain `!`, so it sits below the range and storing it costs the slow path nothing.
if (DEBUG) {
  for (const category of ALL_CATEGORIES) {
    const expected = [
      CAT_OP_LT_THEN_UN_NOT,
      CAT_OP_UN_PLUS,
      CAT_OP_UPD_INC,
      CAT_OP_UN_NEG,
      CAT_OP_UPD_DEC,
    ].includes(category);
    const actual = category >= CAT_OP_LT_THEN_UN_NOT;
    debugAssert(
      actual === expected,
      `Category ${category} disagrees with \`last >= CAT_OP_LT_THEN_UN_NOT\``,
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
 * @param state - Printer state
 * @param next - Category of the operator about to be written
 */
export function printSpaceBeforeOperator(state: State, next: Category): void {
  debugAssertLastFresh(state);

  // The slow path only runs when an operator it distinguishes was the immediately preceding token,
  // which is rare in pretty output. Keep the hot check inlinable.
  const prev = state.last;
  if (prev >= CAT_OP_LT_THEN_UN_NOT) printSpaceBeforeOperatorSlow(state, prev, next);
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
 * @param state - Printer state
 * @param prev - Category of the operator written last
 * @param next - Category of the operator about to be written
 */
function printSpaceBeforeOperatorSlow(state: State, prev: Category, next: Category): void {
  if (
    (prev === CAT_OP_UN_PLUS && (next === CAT_OP_UN_PLUS || next === CAT_OP_UPD_INC))
    || (prev === CAT_OP_UN_NEG && (next === CAT_OP_UN_NEG || next === CAT_OP_UPD_DEC))
    || (prev === CAT_OP_LT_THEN_UN_NOT && next === CAT_OP_UPD_DEC)
  ) {
    write(state, " ", CAT_OTHER);
  }
}
