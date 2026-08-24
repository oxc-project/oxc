// Writing to the output, and the categories which describe what was written last.
//
// These live together because the categories exist only to be stored by `write` -
// `state.last` records what was written last by category, not the last character itself.

import { debugAssert } from "../asserts.ts";

import type { MappableNode, NamedMappableNode, UnnamedMappableNode } from "./types.ts";
import type { State } from "../state.ts";

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

/**
 * Append `code` to the output, and record what it ends with.
 *
 * @param state - Printer state
 * @param code - Text to append, never empty
 * @param last - Category of the last character of `code`
 */
export function write(state: State, code: string, last: Category): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  state.last = last;
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = false;
    state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code` to the output, record what it ends with, and record an unnamed source mapping for `node`.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `write` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param state - Printer state
 * @param code - Text to append, never empty
 * @param last - Category of the last character of `code`
 * @param node - Node this text came from
 */
export function writeWithMap(
  state: State,
  code: string,
  last: Category,
  node: UnnamedMappableNode,
): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  markMapStart(state, node);

  state.last = last;
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = false;
    state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code` to the output, record what it ends with, and record a named source mapping for `node`.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `write` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param state - Printer state
 * @param code - Text to append, never empty
 * @param last - Category of the last character of `code`
 * @param node - Node this text came from
 */
export function writeWithMapNamed(
  state: State,
  code: string,
  last: Category,
  node: NamedMappableNode,
): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  markMapNamed(state, node);

  state.last = last;
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = false;
    state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code` to the output, leaving `state.last` describing whatever came before it.
 *
 * Only sound where the value of `last` is provably dead - another `write` must follow before anything reads it.
 * The readers are the two space functions, the `CAT_INT_DIGIT`, `CAT_LT` and `CAT_QUESTION` adjacency checks,
 * the `CAT_START_OF_*` marker checks, and the `CAT_CLOSE_BRACKET` source map hook - which is the one that runs
 * at the end of printing an expression rather than at the start of one.
 * Every one of them calls `debugAssertLastFresh` first.
 *
 * In practice that means using it for all but the final fragment when one token is written in pieces.
 * Debug builds track the rule and throw if a reader sees a stale `last`.
 *
 * @param state - Printer state
 * @param code - Text to append, which unlike `write` may be empty
 */
export function writeNoLast(state: State, code: string): void {
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = true;
    if (code.length > 0) state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code` and record an unnamed source mapping for `node`, leaving `state.last` alone.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 *
 * `writeNoLast`'s rule about `last` applies here too - another write must follow before anything reads it.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `writeNoLast` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param state - Printer state
 * @param code - Text to append, which unlike `writeWithMap` may be empty
 * @param node - Node this text came from
 */
export function writeWithMapNoLast(state: State, code: string, node: UnnamedMappableNode): void {
  markMapStart(state, node);

  state.output += code;

  if (DEBUG) {
    state.lastIsStale = true;
    if (code.length > 0) state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code` and record a named source mapping for `node`, leaving `state.last` alone.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 *
 * `writeNoLast`'s rule about `last` applies here too - another write must follow before anything reads it.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `writeNoLast` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param state - Printer state
 * @param code - Text to append, which unlike `writeWithMapNamed` may be empty
 * @param node - Node this text came from
 */
export function writeWithMapNamedNoLast(state: State, code: string, node: NamedMappableNode): void {
  markMapNamed(state, node);

  state.output += code;

  if (DEBUG) {
    state.lastIsStale = true;
    if (code.length > 0) state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Append `code`, recording a mapping for the last source character in `node` immediately before it.
 *
 * Rust uses this for emitted closing delimiters, including synthesized ones. The node end offset is
 * exclusive, so move back by one UTF-16 unit to match Rust's byte-span lookup. That can land on
 * the low surrogate of an astral character, which `generateSourceMap` normalizes back to its start.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `write` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param state - Printer state
 * @param code - Text to append, never empty
 * @param last - Category of the last character of `code`
 * @param node - Node whose last source character this text maps to
 */
export function writeWithMapEnd(
  state: State,
  code: string,
  last: Category,
  node: MappableNode,
): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  markMapEnd(state, node);

  state.last = last;
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = false;
    state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Record a mapping for `node`'s start offset at the current output position.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 * Builds without source map support have no use for this. In those builds, minifier removes it.
 *
 * @param state - Printer state
 * @param node - Node the mapping points at
 */
export function markMapStart(state: State, node: MappableNode): void {
  if (SOURCEMAPS && hasMappableSpan(node)) recordMapping(state, node.start);
}

/**
 * Record a mapping for `node`'s end offset at the current output position.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 * Builds without source map support have no use for this. In those builds, minifier removes it.
 *
 * @param state - Printer state
 * @param node - Node whose end offset the mapping points at
 */
export function markMapAfter(state: State, node: MappableNode): void {
  if (SOURCEMAPS && hasMappableSpan(node)) recordMapping(state, node.end);
}

/**
 * Record a mapping a fixed number of characters after `node`'s start offset.
 *
 * The mapping is only recorded where the caller asked for source maps and `node` carries `start` / `end` offsets.
 * Builds without source map support have no use for this. In those builds, minifier removes it.
 *
 * @param state - Printer state
 * @param node - Node the mapping points into
 * @param columnOffset - Number of UTF-16 units to add to `node`'s start offset
 */
export function markMapAtStartOffset(state: State, node: MappableNode, columnOffset: number): void {
  if (SOURCEMAPS && hasMappableSpan(node)) recordMapping(state, node.start + columnOffset);
}

/**
 * Record an unnamed mapping at `node`'s last source character.
 *
 * The end offset is exclusive, so this is one UTF-16 unit back from it, matching Rust's byte-span lookup.
 * `generateSourceMap` normalizes a landing on a low surrogate back to its code point.
 *
 * @param state - Printer state
 * @param node - Node whose last source character the mapping points at
 */
function markMapEnd(state: State, node: MappableNode): void {
  if (SOURCEMAPS && hasMappableSpan(node)) {
    // `hasMappableSpan` ensured span is non-empty, so `end` is at least 1. `end - 1` cannot go negative.
    recordMapping(state, node.end - 1);
  }
}

/**
 * Record a mapping at `node`'s start offset, carrying the name it had in the source.
 *
 * The name is only recorded for the mapping where it differs from the text which is printed.
 *
 * @param state - Printer state
 * @param node - Node the mapping points at
 */
function markMapNamed(state: State, node: NamedMappableNode): void {
  if (!SOURCEMAPS || !hasMappableSpan(node)) return;

  debugAssert(
    state.mapPositions !== null && state.mapNames !== null && state.sourceText !== null,
    "`mapPositions`, `mapNames` and `sourceText` should be defined when source maps are enabled",
  );

  const { start, end } = node;
  const { sourceText } = state;
  if (start > sourceText.length) return;

  // `oxc_codegen` suppresses consecutive source positions as it records them. Do this before
  // recovering a name or retaining the mapping, since member-level marks commonly duplicate keys.
  const { mapPositions } = state;
  if (mapPositions[mapPositions.length - 1] === start) return;

  // A mapping carries a name only when the identifier printed differs from the one in the source.
  // When possible, the mapping records the name from source, but if the source range is invalid,
  // it falls back to the printed name.
  //
  // Almost every identifier is printed exactly as it was in source, so we do a quick check first,
  // and only fall back to expensive scanning with Unicode property regexps in rare cases.
  //
  // A span can reach past the name it begins with, since a TypeScript annotation is absorbed into it,
  // so a match has to be followed by a character which cannot continue an identifier.
  //
  // This check is one-sided.
  // * When `matchesSource === true`, the source definitely has the same name that was printed,
  //   and the mapping needs no name recorded.
  // * When `matchesSource === false`, nothing is settled yet.
  //   It could be a genuine rename, or could be a Unicode escape (`\u0061` printed as `a`),
  //   or a non-ASCII character after the name, which `isDefinitelyIdentifierBoundary` will not classify.
  //
  // A private identifier prints as `#` followed by its name, and its span covers the `#`, so the token is `#name`.
  // That is what the source is compared against, and what gets recorded as the name.
  const printedName = node.name;
  const hashLength = node.type === "PrivateIdentifier" ? 1 : 0;
  const nameStart = start + hashLength;
  const nameEnd = nameStart + printedName.length;
  const matchesSource =
    end <= sourceText.length
    && nameEnd <= end
    && (hashLength === 0 || sourceText.charCodeAt(start) === 35) /* # */
    && sourceText.startsWith(printedName, nameStart)
    && (nameEnd === end || isDefinitelyIdentifierBoundary(sourceText.charCodeAt(nameEnd)));

  if (!matchesSource) {
    // Read the name out of the source.
    // We can use it for a definitive comparison, which the quick check above couldn't.
    const originalName = originalNameFromSource(sourceText, node, start, end);

    // A transformed or hand-authored AST can carry ranges unrelated to `sourceText`.
    // Preserve the existing fallback in that case instead of recording an arbitrary source substring.
    if (originalName === undefined || !isSameToken(originalName, printedName, hashLength)) {
      state.mapNames.push(
        mapPositions.length >> 1,
        originalName === undefined ? printedName : originalName,
      );
    }
  }

  mapPositions.push(state.output.length, start);
}

/**
 * Whether `node` carries a span a mapping can be recorded for.
 *
 * A transformed or hand-authored AST can carry anything at all, so the offsets are checked rather
 * than trusted. Proving `start` and `end` here is what lets each recorder below skip the lower half
 * of its own bounds check - `start` is not negative, and `end` is greater than it.
 *
 * @param node - Node the mapping is for
 * @returns `true` if `node` has a non-empty span of safe integer offsets
 */
function hasMappableSpan(
  node: MappableNode,
): node is MappableNode & { start: number; end: number } {
  const { start, end } = node;
  return (
    typeof start === "number"
    && typeof end === "number"
    && Number.isSafeInteger(start)
    && Number.isSafeInteger(end)
    && start >= 0
    && end > start
  );
}

/**
 * Record source mapping at specified offset.
 *
 * @param state - Printer state
 * @param sourceOffset - Source offset to record mapping for
 */
function recordMapping(state: State, sourceOffset: number): void {
  debugAssert(
    state.mapPositions !== null && state.sourceText !== null,
    "`mapPositions` and `sourceText` should be defined when source maps are enabled",
  );

  if (sourceOffset > state.sourceText.length) return;

  const { mapPositions } = state;
  if (mapPositions[mapPositions.length - 1] === sourceOffset) return;
  mapPositions.push(state.output.length, sourceOffset);
}

/**
 * Is `originalName` the same token as `printedName`, taking into account leading `#` if `hashLength === 1`.
 */
function isSameToken(originalName: string, printedName: string, hashLength: 0 | 1): boolean {
  if (hashLength === 0) return originalName === printedName;

  // Compare with 2 operations rather than `originalName === "#" + printedName`
  // to avoid allocating a temporary string
  return (
    originalName.length === printedName.length + 1
    && originalName.charCodeAt(0) === 35 /* # */
    && originalName.endsWith(printedName)
  );
}

/**
 * Recover the original identifier spelling from validated source offsets.
 *
 * A private identifier's span covers its leading `#`. The scan steps over it to reach the name behind,
 * and the `#` is part of what is returned - the token is `#name`.
 *
 * @param sourceText - Original source text
 * @param node - Node the offsets came from
 * @param start - Start offset of `node`, already validated
 * @param end - End offset of `node`, already validated
 * @returns The identifier as it appears in the source, or `undefined` if the offsets do not span one
 */
function originalNameFromSource(
  sourceText: string,
  node: MappableNode,
  start: number,
  end: number,
): string | undefined {
  if (end > sourceText.length) return undefined;

  // JSX identifiers admit `-`, which is not an ECMAScript identifier character. Their spans do
  // not absorb TypeScript annotations, so the ESTree end offset is already exact.
  if (node.type === "JSXIdentifier") {
    const originalName = sourceText.slice(start, end);
    return JSX_IDENTIFIER_REGEX.test(originalName) ? originalName : undefined;
  }

  let index = start;
  if (index < end && sourceText.charCodeAt(index) === 35) index++; // `#` in a private identifier
  const identifierStart = index;

  while (index < end) {
    const matcher = index === identifierStart ? IDENT_START_REGEX : IDENT_CONTINUE_REGEX;
    if (sourceText.charCodeAt(index) === 92) {
      const length = unicodeEscapeLength(sourceText, index, end);
      if (length === 0) break;
      const codePoint = unicodeEscapeCodePoint(sourceText, index, length);
      if (codePoint > 0x10ffff || !matcher.test(String.fromCodePoint(codePoint))) break;
      index += length;
      continue;
    }

    const codePoint = sourceText.codePointAt(index) as number;
    const char = String.fromCodePoint(codePoint);
    if (!matcher.test(char)) break;
    index += char.length;
  }

  return index === identifierStart ? undefined : sourceText.slice(start, index);
}

/**
 * Return the UTF-16 length of a `\u` identifier escape within `end`, or `0` if invalid.
 *
 * Covers both the fixed 4-digit form and the braced `\u{...}` one.
 *
 * @param sourceText - Original source text
 * @param index - Offset of the `\` starting the escape
 * @param end - Offset the escape must finish within
 * @returns Length of the escape in UTF-16 units, or `0` if it is not a valid one
 */
function unicodeEscapeLength(sourceText: string, index: number, end: number): number {
  if (sourceText.charCodeAt(index + 1) !== 117) return 0; // `u`

  const firstHex = index + 2;
  if (sourceText.charCodeAt(firstHex) === 123) {
    let cursor = firstHex + 1;
    const firstDigit = cursor;
    while (cursor < end && isHexDigit(sourceText.charCodeAt(cursor))) cursor++;
    return cursor > firstDigit && sourceText.charCodeAt(cursor) === 125 ? cursor - index + 1 : 0;
  }

  const escapeEnd = firstHex + 4;
  if (escapeEnd > end) return 0;
  for (let cursor = firstHex; cursor < escapeEnd; cursor++) {
    if (!isHexDigit(sourceText.charCodeAt(cursor))) return 0;
  }
  return escapeEnd - index;
}

/**
 * Decode the code point in a syntactically valid identifier escape.
 *
 * @param sourceText - Original source text
 * @param index - Offset of the `\` starting the escape
 * @param length - Length of the escape, as returned by `unicodeEscapeLength`
 * @returns The code point the escape denotes
 */
function unicodeEscapeCodePoint(sourceText: string, index: number, length: number): number {
  const braced = sourceText.charCodeAt(index + 2) === 123;
  const digitsStart = index + (braced ? 3 : 2);
  const digitsEnd = index + length - (braced ? 1 : 0);
  return Number.parseInt(sourceText.slice(digitsStart, digitsEnd), 16);
}

/**
 * Whether `code` is an ASCII hexadecimal digit.
 *
 * @param code - Character code
 * @returns `true` if `code` is `0`-`9`, `a`-`f` or `A`-`F`
 */
function isHexDigit(code: number): boolean {
  return (code >= 48 && code <= 57) || (code >= 65 && code <= 70) || (code >= 97 && code <= 102);
}

/**
 * Whether an ASCII character definitely cannot continue any supported identifier spelling.
 *
 * @param code - Character code
 * @returns `true` only for an ASCII character which cannot continue an identifier. Every non-ASCII
 *   character gives `false`, since this runtime's tables cannot settle it
 */
function isDefinitelyIdentifierBoundary(code: number): boolean {
  return (
    code <= 0x7f
    && !((code >= 48 && code <= 57) || (code >= 65 && code <= 90) || (code >= 97 && code <= 122))
    && code !== 36 // `$`
    && code !== 45 // `-` in JSX identifiers
    && code !== 92 // `\` starting a Unicode escape
    && code !== 95 // `_`
  );
}

/**
 * Assert that `state.last` still describes what was written last.
 *
 * Every reader of `last` calls this, so a `writeNoLast` whose caller broke the rule is caught by
 * the conformance suites rather than silently incorrectly spacing one construct.
 *
 * Debug builds only - the call and its argument are removed from release builds entirely.
 *
 * @param state - Printer state
 */
export function debugAssertLastFresh(state: State): void {
  debugAssert(!state.lastIsStale, "`last` was read after `writeNoLast` left it stale");
}

/** Matches a character which can continue an identifier. */
const IDENT_CONTINUE_REGEX = /[\p{ID_Continue}$\u200C\u200D]/u;
const IDENT_START_REGEX = /[\p{ID_Start}$_]/u;
const JSX_IDENTIFIER_REGEX = /^[\p{ID_Start}$_](?:[\p{ID_Continue}$-]|\u200C|\u200D)*$/u;

/**
 * Assert that `last` truthfully describes the end of `code`.
 *
 * A category encodes the merge hazard of the trailing token, not the trailing character -
 * a digit legitimately ends both a `CAT_IDENT` and a `CAT_INT_DIGIT` write - so each final character
 * has a set of permitted categories rather than one right answer.
 *
 * Debug builds only. Removed by minifier in release builds.
 *
 * @param state - Printer state
 * @param code - Code being appended to output
 * @param last - Category of the last character of `code`
 * @throws - If `last` and `code` do not match
 */
function debugAssertCategoryMatches(state: State, code: string, last: Category): void {
  if (!DEBUG) return;

  const ch = code.at(-1)!;

  let ok;
  if (ch >= "0" && ch <= "9") {
    // A digit ends an identifier (`x1`), a number some other character already separates
    // (`1e3`, `0x1f`, `.5`) or a written fragment of one (the digits of `15e-8`) - all `CAT_IDENT`.
    // Only a numeric literal which is plain digits throughout is `CAT_INT_DIGIT`.
    // The dangerous converse - a whole plain-digit literal claiming `CAT_IDENT`, which loses the space
    // in `0 .toExponential()` - looks exactly like a fragment write, so it cannot be caught here.
    // The AST fixtures pin the number printer's category decisions instead.
    ok = last === CAT_IDENT || (last === CAT_INT_DIGIT && /^[0-9]+$/.test(code));
  } else if (ch === "+") {
    if (code.endsWith("++")) {
      ok = last === CAT_OP_UPD_INC;
    } else {
      ok = last === CAT_OP_UN_PLUS;
    }
  } else if (ch === "-") {
    if (code.endsWith("--")) {
      ok = last === CAT_OP_UPD_DEC;
    } else {
      ok = last === CAT_OP_UN_NEG;
    }
  } else if (ch === "!") {
    // The two operator categories are told apart by the character before the `!`, which also proves
    // the `CAT_OP_UN_NOT_AFTER_LT` branch in `printUnaryExpression` fires exactly when it should
    if (last === CAT_OP_UN_NOT_AFTER_LT) {
      ok = state.lastCharWritten === "<";
    } else {
      ok = last === CAT_OP_UN_NOT && state.lastCharWritten !== "<";
    }
  } else if (ch === "<") {
    ok = last === CAT_LT;
  } else if (ch === "?") {
    ok = last === CAT_QUESTION;
  } else if (ch === "/") {
    ok = last === CAT_REGEX_SLASH;
  } else if (ch === ")" || ch === "]") {
    ok = last === CAT_CLOSE_BRACKET;
  } else {
    // The final character, whole - for an astral character `ch` is only its low surrogate,
    // which no Unicode property matches
    const isLowSurrogate = ch >= "\uDC00" && ch <= "\uDFFF";
    const finalChar = isLowSurrogate && code.length >= 2 ? code.slice(-2) : ch;

    if (IDENT_CONTINUE_REGEX.test(finalChar)) {
      ok = last === CAT_IDENT;
    } else if (ch <= "\x7F") {
      // ASCII
      ok = last === CAT_OTHER;
    } else {
      // Not an identifier character by this runtime's Unicode tables, but the parser's tables
      // may be a newer Unicode version which does count it, so both claims are believed -
      // test262 holds identifiers whose final characters are new enough to hit this
      ok = last === CAT_IDENT || last === CAT_OTHER;
    }
  }

  debugAssert(ok, () => `Category ${last} does not describe the end of ${JSON.stringify(code)}`);
}
