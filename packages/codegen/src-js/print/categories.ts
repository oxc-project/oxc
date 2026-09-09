// The categories which describe what was written last.
//
// They exist only to be stored in `state.last`, which every `write` call sets - see `write.ts`.

import { debugAssert } from "../asserts.ts";

// `state.last` records what was written last, by category, not the last character itself.
//
// Reading last character of `state.output` is disastrous for performance. `state.output` is a "cons string",
// a concatenation of many smaller strings. Reading any character of it causes the string to be flattened,
// which is costly. Given how often it's required to know what was printed last, this produces a 4x slow-down.
//
// No reader needs the character - only its category - so the category is defined statically in `write` calls
// and stored as a small integer, which costs no write barrier to store and one compare to test.
//
// Three properties of the layout are load bearing, so keep them if you add a code:
//
// 1. `CAT_IDENT` through `CAT_REGEX_SLASH` are the classes needing a space before a following identifier,
//    and they are the lowest codes, so `printSpaceBeforeIdentifier` is one compare against `CAT_REGEX_SLASH`.
//    The unary and update operators are likewise contiguous, and the highest codes, so `printSpaceBeforeOperator`
//    is one compare too.
//
// 2. The `CAT_START_OF_*` codes say where the output has reached, not what was written last, and they sit between
//    those two ranges. Both space checks therefore read them as "nothing to separate", which is what the `CAT_OTHER`
//    they displace meant - every position one of them marks follows whitespace or the start of the output.
//    See `state.ts` for what they are for.
//
// 3. `CAT_START_OF_STMT` is odd, with the other two marks either side of it.
//    Each of the two pairs a reader asks about is then one `|`-and-compare instead of two compares.
//
// The whole numbering:
//
// Group 0 to 2: A following identifier needs a space (`printSpaceBeforeIdentifier` checks `last <= CAT_REGEX_SLASH`)
//    0  CAT_IDENT                      Identifier part - letters, digits, `_`, `$`, ID_Continue
//    1  CAT_INT_DIGIT                  Numeric literal of plain digits (`0 .toExponential()`)
//    2  CAT_REGEX_SLASH                Regex closed with no flags
//
// Group 3 to 10: Checked individually
//    3  CAT_OTHER                      Anything else not covered by another category - punctuation, whitespace
//    4  CAT_LT                         `<`
//    5  CAT_QUESTION                   `?`
//    6  CAT_START_OF_DEFAULT_EXPORT    `export default` expression is about to be printed
//    7  CAT_START_OF_STMT              Expression statement's expression is about to be printed
//    8  CAT_START_OF_ARROW_EXPR        Concise arrow body is about to be printed
//    9  CAT_CLOSE_BRACKET              `)` or `]`
//   10  CAT_OP_UN_NOT                  `!`
//
// Group 11 to 15: Operators `printSpaceBeforeOperatorSlow` needs to tell apart (`last >= CAT_OP_UN_NOT_AFTER_LT`)
//   11  CAT_OP_UN_NOT_AFTER_LT         `!` written straight after a `<`
//   12  CAT_OP_UN_PLUS                 `+`
//   13  CAT_OP_UPD_INC                 `++`
//   14  CAT_OP_UN_NEG                  `-`
//   15  CAT_OP_UPD_DEC                 `--`

/**
 * A category code. One of the `CAT_*` constants below, and nothing else -
 * the range checks which read `state.last` are only sound over exactly this set.
 */
export type Category =
  | typeof CAT_IDENT
  | typeof CAT_INT_DIGIT
  | typeof CAT_REGEX_SLASH
  | typeof CAT_OTHER
  | typeof CAT_LT
  | typeof CAT_QUESTION
  | typeof CAT_START_OF_STMT
  | typeof CAT_START_OF_ARROW_EXPR
  | typeof CAT_START_OF_DEFAULT_EXPORT
  | typeof CAT_CLOSE_BRACKET
  | typeof CAT_OP_UN_NOT
  | typeof CAT_OP_UN_NOT_AFTER_LT
  | typeof CAT_OP_UN_PLUS
  | typeof CAT_OP_UPD_INC
  | typeof CAT_OP_UN_NEG
  | typeof CAT_OP_UPD_DEC;

/** Identifier part - letters, digits, `_`, `$`, Unicode `ID_Continue`. */
export const CAT_IDENT = 0;

/**
 * A numeric literal whose text is plain digits, so a following `.` would be read as part of the number -
 * `0 .toExponential()` needs the space, `1e3.x` and `.5.x` do not.
 */
export const CAT_INT_DIGIT = 1;

/**
 * A regex's closing `/`, written only where the regex has no flags.
 *
 * In the identifier range because `/a/ in x` needs the space just as much as `x in y` does -
 * without it the `in` would be read as regex flags. With flags, `CAT_IDENT` says the same thing.
 */
export const CAT_REGEX_SLASH = 2;

/** Anything not covered by another category - punctuation, whitespace, a quote. */
export const CAT_OTHER = 3;

/** `<`, which a following `!` must not merge with into `<!--`. */
export const CAT_LT = 4;

/** `?`, which must not merge with a following `?` into `??` - see `TSJSDocNullableType`. */
export const CAT_QUESTION = 5;

/**
 * An `export default` expression is about to be printed, and nothing has been written since.
 *
 * Read by the function and class printers, which parenthesize themselves here so the declaration forms
 * (`export default function f() {}`) stay distinguishable from the expression ones.
 *
 * Immediately below `CAT_START_OF_STMT`, which is what makes their shared test one `|`.
 */
export const CAT_START_OF_DEFAULT_EXPORT = 6;

/**
 * An expression statement's expression is about to be printed, and nothing has been written since.
 *
 * A node which finds this in `last` is the leftmost token of the statement, however deeply nested it is,
 * so an object literal or an object destructuring assignment there parenthesizes itself.
 *
 * The only mark both readers ask about, so it is the odd one, with the other two either side.
 */
export const CAT_START_OF_STMT = 7;

/**
 * A concise arrow body is about to be printed, and nothing has been written since.
 *
 * Read by the same two printers as `CAT_START_OF_STMT` - `x => ({ a: 1 })` needs the parens for
 * the same reason a statement does, because a leading `{` would be read as a block.
 *
 * Immediately above `CAT_START_OF_STMT`, which is what makes their shared test one `|`.
 */
export const CAT_START_OF_ARROW_EXPR = 8;

/**
 * `)` or `]`, the only two characters a postfix operand can end with.
 *
 * The one reader is the source map hook at the end of `printExpression`, which mirrors Rust's
 * exclusive-end mapping after such an operand. It never asks which of the two was written,
 * so one code serves for both.
 *
 * Neither space check cares about either character, so it sits between the marks and the operator range,
 * where both range compares read it as "nothing to separate".
 */
export const CAT_CLOSE_BRACKET = 9;

/**
 * `!`, written anywhere other than straight after a `<` - both the unary operator and TS's
 * postfix `!` (non-null assertion, definite assignment), which postfix position keeps off a `<`.
 *
 * An operator code, but deliberately below the range `printSpaceBeforeOperator` gates on -
 * no following operator merges with a plain `!`, so storing this never costs the slow path a call.
 */
export const CAT_OP_UN_NOT = 10;

/**
 * `!` written immediately after a `<`, which is the `<!--` hazard.
 * Folding the check on the preceding character into the code saves tracking the second-last character.
 *
 * The first of the operators `printSpaceBeforeOperator` gates on - writing one of these
 * is what records it, so no separate field tracks which operator came last.
 */
export const CAT_OP_UN_NOT_AFTER_LT = 11;

/** `+`, which must not merge with a following `+` or `++`. */
export const CAT_OP_UN_PLUS = 12;

/** `++`, which must not follow a `+` without a space. */
export const CAT_OP_UPD_INC = 13;

/** `-`, which must not merge with a following `-` or `--`. */
export const CAT_OP_UN_NEG = 14;

/** `--`, which must not follow a `-`, nor the `!` of a `<!`. */
export const CAT_OP_UPD_DEC = 15;

/**
 * Every category, in numbering order.
 *
 * Only the debug checks which prove the range compares still match the sets they mean read this,
 * so release builds drop it entirely.
 */
export const ALL_CATEGORIES: Category[] = [
  CAT_IDENT,
  CAT_INT_DIGIT,
  CAT_REGEX_SLASH,
  CAT_OTHER,
  CAT_LT,
  CAT_QUESTION,
  CAT_START_OF_DEFAULT_EXPORT,
  CAT_START_OF_STMT,
  CAT_START_OF_ARROW_EXPR,
  CAT_CLOSE_BRACKET,
  CAT_OP_UN_NOT,
  CAT_OP_UN_NOT_AFTER_LT,
  CAT_OP_UN_PLUS,
  CAT_OP_UPD_INC,
  CAT_OP_UN_NEG,
  CAT_OP_UPD_DEC,
];

// The five marker readers select a pair of marks by position rather than comparing each in turn.
// `printFunction`, `printClass` and `printCallExpression` ask "statement or `export default`",
// `printObjectExpression` and `printAssignmentExpression` ask "statement or concise arrow body":
//
//    (last | 1) === CAT_START_OF_STMT          // Statement or `export default`
//    ((last - 1) | 1) === CAT_START_OF_STMT    // Statement or concise arrow body
//
// Both hold only while `CAT_START_OF_STMT` is odd and its two neighbours sit either side of it.
// Check each against the set it is meant to select, over every category and no others.
if (DEBUG) {
  for (const category of ALL_CATEGORIES) {
    debugAssert(
      ((category | 1) === CAT_START_OF_STMT)
        === (category === CAT_START_OF_STMT || category === CAT_START_OF_DEFAULT_EXPORT),
      `Category ${category} disagrees with \`(last | 1) === CAT_START_OF_STMT\``,
    );
    debugAssert(
      (((category - 1) | 1) === CAT_START_OF_STMT)
        === (category === CAT_START_OF_STMT || category === CAT_START_OF_ARROW_EXPR),
      `Category ${category} disagrees with \`((last - 1) | 1) === CAT_START_OF_STMT\``,
    );
  }
}
