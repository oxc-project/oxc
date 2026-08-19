// Writing to the output, and the categories which describe what was written last.
//
// These live together because the categories exist only to be stored by `write` -
// `state.last` records what was written last by category, not the last character itself.

import { debugAssert } from "../asserts.ts";

import type { MappableNode } from "./types.ts";
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
      ((category | 1) === CAT_START_OF_STMT) ===
        (category === CAT_START_OF_STMT || category === CAT_START_OF_DEFAULT_EXPORT),
      `Category ${category} disagrees with \`(last | 1) === CAT_START_OF_STMT\``,
    );
    debugAssert(
      (((category - 1) | 1) === CAT_START_OF_STMT) ===
        (category === CAT_START_OF_STMT || category === CAT_START_OF_ARROW_EXPR),
      `Category ${category} disagrees with \`((last - 1) | 1) === CAT_START_OF_STMT\``,
    );
  }
}

/**
 * Location of mapping to record.
 */
type Location =
  | typeof LOCATION_NAMED
  | typeof LOCATION_END_MINUS_ONE
  | typeof LOCATION_END
  | typeof LOCATION_START;

const LOCATION_NAMED = 0;
const LOCATION_END_MINUS_ONE = 1;
const LOCATION_END = 2;
const LOCATION_START = 3;

/**
 * Append `code` to the output, and record what it ends with.
 *
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
 * Append `code` to the output, record what it ends with, and record a source mapping for `node`.
 *
 * The mapping is only recorded where the caller asked for source maps, supplied `sourceText`, and
 * `node` carries Oxc `start` / `end` offsets.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `write` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param code - Text to append, never empty
 * @param last - Category of the last character of `code`
 * @param node - Node this text came from
 */
export function writeWithMap(state: State, code: string, last: Category, node: MappableNode): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  recordSourceMapping(state, node, LOCATION_NAMED);

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
 * The readers are `printSpaceBeforeIdentifier` and the `CAT_LT`, `CAT_REGEX_SLASH` and `CAT_QUESTION`
 * adjacency checks, all of which run at the start of printing a construct.
 *
 * In practice that means using it for all but the final fragment when one token is written in pieces.
 * Debug builds track the rule and throw if a reader sees a stale `last`.
 *
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
 * Append `code` and record a source mapping for `node`, leaving `state.last` alone.
 *
 * `writeNoLast`'s rule about `last` applies here too - another write must follow before anything reads it.
 *
 * Builds without source map support have no use for this. For those builds, TSDown plugin rewrites every call
 * into `write` and drops the `node` argument, leaving this unreferenced for the minifier to remove.
 *
 * @param code - Text to append, which unlike `writeWithMap` may be empty
 * @param node - Node this text came from
 */
export function writeWithMapNoLast(state: State, code: string, node: MappableNode): void {
  recordSourceMapping(state, node, LOCATION_NAMED);

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
 * exclusive, so move back by one source code point to match Rust's byte-span lookup.
 */
export function writeWithMapEnd(
  state: State,
  code: string,
  last: Category,
  node: MappableNode,
): void {
  debugAssert(code.length > 0, "`code` should not be an empty string");
  debugAssertCategoryMatches(state, code, last);

  recordSourceMapping(state, node, LOCATION_END_MINUS_ONE);
  state.last = last;
  state.output += code;

  if (DEBUG) {
    state.lastIsStale = false;
    state.lastCharWritten = code[code.length - 1];
  }
}

/**
 * Record a start mapping at the current output position without writing anything.
 */
export function markWithMap(state: State, node: MappableNode): void {
  recordSourceMapping(state, node, LOCATION_NAMED);
}

/**
 * Record a start mapping without attaching an identifier name.
 */
export function markWithMapNoName(state: State, node: MappableNode): void {
  recordSourceMapping(state, node, LOCATION_START);
}

/**
 * Record a mapping for `node`'s end offset at the current output position.
 */
export function markWithMapAfter(state: State, node: MappableNode): void {
  recordSourceMapping(state, node, LOCATION_END);
}

/**
 * Record a mapping a fixed number of columns after `node`'s start offset.
 */
export function markWithMapAtStartOffset(
  state: State,
  node: MappableNode,
  columnOffset: number,
): void {
  if (!SOURCEMAPS) return;

  const { start, end } = node;
  if (
    typeof start !== "number" ||
    typeof end !== "number" ||
    !Number.isSafeInteger(start) ||
    !Number.isSafeInteger(end) ||
    start < 0 ||
    end < start ||
    start === end
  ) {
    return;
  }

  debugAssert(
    state.mapPositions !== null && state.sourceText !== null,
    "`mapPositions` and `sourceText` should be defined when source maps are enabled",
  );

  const sourceOffset = start + columnOffset;
  if (!(sourceOffset >= 0 && sourceOffset <= state.sourceText.length)) return;
  if (state.mapPositions[state.mapPositions.length - 1] === sourceOffset) return;

  state.mapPositions.push(state.output.length, sourceOffset);
}

/**
 * Record one mapping, if source maps and a non-empty location are available.
 */
function recordSourceMapping(state: State, node: MappableNode, location: Location): void {
  if (!SOURCEMAPS) return;

  const { start, end } = node;
  if (
    typeof start !== "number" ||
    typeof end !== "number" ||
    !Number.isSafeInteger(start) ||
    !Number.isSafeInteger(end) ||
    start < 0 ||
    end < start ||
    start === end
  ) {
    return;
  }

  debugAssert(
    state.mapPositions !== null && state.sourceText !== null,
    "`mapPositions` and `sourceText` should be defined when source maps are enabled",
  );

  let sourceOffset: number;
  if (location === LOCATION_END_MINUS_ONE) {
    sourceOffset = end - 1;
  } else if (location === LOCATION_END) {
    sourceOffset = end;
  } else {
    debugAssert(location === LOCATION_START || location === LOCATION_NAMED);
    sourceOffset = start;
  }

  const { sourceText } = state;
  if (!(sourceOffset >= 0 && sourceOffset <= sourceText.length)) return;

  // `oxc_codegen` suppresses consecutive source positions as it records them. Do this before
  // recovering a name or retaining the mapping, since member-level marks commonly duplicate keys.
  if (state.mapPositions[state.mapPositions.length - 1] === sourceOffset) return;

  if (location === LOCATION_NAMED) {
    let name: string | undefined;
    const printedName = typeof node.name === "string" ? node.name : undefined;
    if (printedName !== undefined) {
      // Almost every identifier is printed exactly as it appeared in the source. Avoid scanning it
      // with Unicode property regexps or allocating a source substring in that common case.
      const nameEnd = start + printedName.length;
      const originalName =
        printedName.length > 0 &&
        end <= sourceText.length &&
        nameEnd <= end &&
        sourceText.startsWith(printedName, start) &&
        (nameEnd === end || isDefinitelyIdentifierBoundary(sourceText.charCodeAt(nameEnd)))
          ? printedName
          : originalNameFromSource(sourceText, node, start, end);

      // A transformed or hand-authored AST can carry ranges unrelated to `sourceText`.
      // Preserve the existing fallback in that case instead of recording an arbitrary source substring.
      name =
        originalName === undefined
          ? printedName
          : originalName === printedName
            ? undefined
            : originalName;
    } else {
      name = printedName;
    }

    if (name !== undefined) {
      const mappingIndex = state.mapPositions.length >> 1;
      (state.mapNames ??= []).push(mappingIndex, name);
    }
  }

  state.mapPositions.push(state.output.length, sourceOffset);
}

/**
 * Recover the original identifier spelling from validated source offsets.
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
 * Return the UTF-16 length of a `\\u` identifier escape within `end`, or `0` if invalid.
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
 */
function unicodeEscapeCodePoint(sourceText: string, index: number, length: number): number {
  const braced = sourceText.charCodeAt(index + 2) === 123;
  const digitsStart = index + (braced ? 3 : 2);
  const digitsEnd = index + length - (braced ? 1 : 0);
  return Number.parseInt(sourceText.slice(digitsStart, digitsEnd), 16);
}

/**
 * Whether `code` is an ASCII hexadecimal digit.
 */
function isHexDigit(code: number): boolean {
  return (code >= 48 && code <= 57) || (code >= 65 && code <= 70) || (code >= 97 && code <= 102);
}

/**
 * Whether an ASCII character definitely cannot continue any supported identifier spelling.
 */
function isDefinitelyIdentifierBoundary(code: number): boolean {
  return (
    code <= 0x7f &&
    !((code >= 48 && code <= 57) || (code >= 65 && code <= 90) || (code >= 97 && code <= 122)) &&
    code !== 36 && // `$`
    code !== 45 && // `-` in JSX identifiers
    code !== 92 && // `\` starting a Unicode escape
    code !== 95 // `_`
  );
}

/**
 * Assert that `state.last` still describes what was written last.
 *
 * Every reader of `last` calls this, so a `writeNoLast` whose caller broke the rule is caught by
 * the conformance suites rather than silently incorrectly spacing one construct.
 *
 * Debug builds only - the call and its argument are removed from release builds entirely.
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
