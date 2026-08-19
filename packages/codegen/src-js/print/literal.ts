// Literals.

import { typeAssertIs } from "../asserts.ts";
import {
  CAT_CLOSE_BRACKET,
  CAT_IDENT,
  CAT_INT_DIGIT,
  CAT_OP_UN_NEG,
  CAT_OTHER,
  CAT_REGEX_SLASH,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapNoLast,
} from "./write.ts";
import { printSpaceBeforeIdentifier, printSpaceBeforeOperator } from "./space.ts";
import { printNonNegativeFloat } from "./number.ts";
import { CTX_TYPESCRIPT } from "./operators.ts";
import { PREC_PREFIX } from "./precedence.ts";
import { printString } from "./string.ts";

import type { State } from "../state.ts";
import type { LiteralExtras } from "./types.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * The literal types, which all share `type: "Literal"` and are told apart by `typeof value`.
 */
type LiteralNode =
  | ESTree.BooleanLiteral
  | ESTree.NullLiteral
  | ESTree.NumericLiteral
  | ESTree.BigIntLiteral
  | ESTree.RegExpLiteral
  | ESTree.StringLiteral;

/**
 * Print any ESTree `Literal`, which is one node type covering strings, numbers, booleans, `null`,
 * regexes and bigints.
 *
 * The kind is told apart by the type of `value` and by which extra property is present,
 * since the node itself does not say.
 */
export function printLiteral(
  node: LiteralNode,
  state: State,
  precedence: number,
  ctx: number,
): void {
  // Ordered for the hot path - `value` is a `RegExp` / `null` for regexes and a `BigInt` / `null` for bigints,
  // so a string or number `value` identifies a plain string/numeric literal without touching `regex` / `bigint` first.
  const { value } = node;
  switch (typeof value) {
    case "string":
      printString(state, value, node);
      break;
    case "number":
      typeAssertIs<ESTree.NumericLiteral>(node);
      printNumericLiteral(node, state, precedence, ctx);
      break;
    case "boolean":
      printSpaceBeforeIdentifier(state);
      writeWithMap(state, value ? "true" : "false", CAT_IDENT, node);
      break;
    default:
      typeAssertIs<LiteralExtras>(node);
      if (node.regex != null) {
        typeAssertIs<ESTree.RegExpLiteral>(node);
        printRegExpLiteral(node, state);
      } else if (node.bigint != null) {
        typeAssertIs<ESTree.BigIntLiteral>(node);
        printBigIntLiteral(node, state, precedence);
      } else {
        // `null`
        printSpaceBeforeIdentifier(state);
        writeWithMap(state, "null", CAT_IDENT, node);
      }
      break;
  }
}

/**
 * Print a numeric literal, parenthesizing a negative one where the position needs it.
 *
 * A negative number is a unary minus applied to a positive literal, so it needs a space
 * or parentheses in the places any other unary minus would, and `-0` has to keep its sign.
 *
 * In TypeScript positions the source text is printed back rather than the shortest form,
 * so a type keeps the digits it was written with.
 */
function printNumericLiteral(
  node: ESTree.NumericLiteral,
  state: State,
  precedence: number,
  ctx: number,
): void {
  const { value } = node;
  if ((ctx & CTX_TYPESCRIPT) !== 0 && node.raw != null) {
    // A number's raw text can only end 3 ways: a digit, a hex letter, or a trailing `.` as in `1.`.
    // The first two are identifier characters and the third is not, so testing for the `.` decides it -
    // no character classification needed.
    // `CAT_IDENT` rather than `CAT_INT_DIGIT` - raw text only prints inside a TS type,
    // where no member `.` can follow, so there is no `0 .toExponential()` hazard to separate.
    const { raw } = node;
    writeWithMap(state, raw, raw[raw.length - 1] === "." ? CAT_OTHER : CAT_IDENT, node);
    return;
  }

  if (value > 0 && value < Infinity) {
    // Finite and positive: the common case, and the same branch the chain
    // below would reach, without the two builtin calls.
    printSpaceBeforeIdentifier(state);
    printNonNegativeFloat(state, value, node);
  } else if (Number.isNaN(value)) {
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, "NaN", CAT_IDENT, node);
  } else if (!Number.isFinite(value)) {
    const negative = value < 0;
    const wrap = negative && precedence >= PREC_PREFIX;
    if (wrap) write(state, "(", CAT_OTHER);

    if (negative) {
      printSpaceBeforeOperator(state, CAT_OP_UN_NEG);
      writeNoLast(state, "-");
    } else {
      printSpaceBeforeIdentifier(state);
    }
    writeWithMap(state, "Infinity", CAT_IDENT, node);

    if (wrap) write(state, ")", CAT_CLOSE_BRACKET);
  } else if (Object.is(value, 0)) {
    // +0 exactly - `Object.is` rejects `-0`, which the negative paths below print as `-0`
    printSpaceBeforeIdentifier(state);
    writeWithMap(state, "0", CAT_INT_DIGIT, node);
  } else if (precedence >= PREC_PREFIX) {
    writeNoLast(state, "(-");
    printNonNegativeFloat(state, -value, node);
    write(state, ")", CAT_CLOSE_BRACKET);
  } else {
    printSpaceBeforeOperator(state, CAT_OP_UN_NEG);
    writeNoLast(state, "-");
    printNonNegativeFloat(state, -value, node);
  }
}

/**
 * Print a regex literal from its pattern and flags.
 *
 * A regex with no flags ends in `/`, which is recorded as its own category, because a `/`
 * immediately after it would open a comment.
 */
function printRegExpLiteral(node: ESTree.RegExpLiteral, state: State): void {
  // Neither of the separating spaces below can be needed in pretty mode, so the check is not made.
  // Both guard against a regex being written immediately after something, and nothing can be
  // immediately before a regex here: every operator which could put one after a `/` or a `<` is
  // written space padded, so `last` is always the space.
  //
  // A minified mode would stop padding operators and make both reachable again, so this code would
  // need to be restored. `CAT_REGEX_SLASH` is left in place for that - the code is still written
  // after a flagless regex, and it still sits in the range `printSpaceBeforeIdentifier` tests,
  // which costs nothing and is what keeps this a 4 line restoration.
  //
  //   debugAssertLastFresh(state);
  //   const { last } = state;
  //   if (last === CAT_REGEX_SLASH || (last === CAT_LT && /^script/i.test(pattern.slice(0, 6)))) {
  //     write(state, " ", CAT_OTHER);
  //   }
  //
  // * `last === CAT_REGEX_SLASH` keeps `/a//b/` from lexing as a line comment
  // * `CAT_LT` arm keeps `<` followed by `/script...` from closing a host `<script>` element.

  writeWithMapNoLast(state, "/", node);
  writeNoLast(state, node.regex.pattern);

  // `CAT_REGEX_SLASH` rather than `CAT_OTHER`. It means "a regex just closed", which is what the
  // commented-out check above would read. With flags, the flags are what `last` describes instead.
  const { flags } = node.regex;
  if (flags === "") {
    write(state, "/", CAT_REGEX_SLASH);
  } else {
    writeNoLast(state, "/");
    write(state, flags, CAT_IDENT);
  }
}

/**
 * Print a bigint literal, parenthesizing a negative one where the position needs it,
 * same as `printNumericLiteral` does.
 */
function printBigIntLiteral(node: ESTree.BigIntLiteral, state: State, precedence: number): void {
  printSpaceBeforeIdentifier(state);

  const value = node.bigint;
  if (value.startsWith("-") && precedence >= PREC_PREFIX) {
    writeWithMapNoLast(state, "(", node);
    writeNoLast(state, value);
    write(state, "n)", CAT_CLOSE_BRACKET);
  } else {
    writeWithMapNoLast(state, value, node);
    write(state, "n", CAT_IDENT);
  }
}
