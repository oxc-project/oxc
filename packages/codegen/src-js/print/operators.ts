// Operator tables.
//
// The context flags a printer is given about the position it is printing into, the precedence of
// each binary and assignment operator, those operators pre-padded with the spaces pretty mode
// always puts around them, and the category a unary or update operator leaves in `state.last`.

import {
  CAT_OP_UN_NEG,
  CAT_OP_UN_NOT,
  CAT_OP_UN_PLUS,
  CAT_OP_UPD_DEC,
  CAT_OP_UPD_INC,
  CAT_OTHER,
} from "./write.ts";
import { debugAssert } from "../asserts.ts";

import type { Category } from "./write.ts";

// Context flags (`oxc_codegen::Context`).

/** No context. */
export const CTX_NONE = 0;

/** Set in a `for` statement's head, where an unparenthesized `in` would end the head. */
export const CTX_FORBID_IN = 1;

/** Set under a `new`, where an unparenthesized call would be taken as `new`'s own argument list. */
export const CTX_FORBID_CALL = 2;

/** Set inside a TS type, where a numeric literal prints its raw text rather than the shortest form. */
export const CTX_TYPESCRIPT = 4;

/**
 * Binary/logical operator precedences on the Oxc scale.
 */
export const BIN_PRECEDENCE = {
  __proto__: null,
  "**": 17,
  "*": 16,
  "/": 16,
  "%": 16,
  "+": 15,
  "-": 15,
  "<<": 14,
  ">>": 14,
  ">>>": 14,
  "<": 13,
  ">": 13,
  "<=": 13,
  ">=": 13,
  instanceof: 13,
  in: 13,
  "==": 12,
  "!=": 12,
  "===": 12,
  "!==": 12,
  "&": 11,
  "^": 10,
  "|": 9,
  "&&": 8,
  "||": 7,
  "??": 6,
};

/**
 * Space-padded forms of every binary/logical operator.
 *
 * In pretty-print mode an operator is always surrounded by single spaces,
 * so the whole token is one constant string and needs a single `write` call.
 */
export const PADDED_BIN_OPERATORS = {
  __proto__: null,
  "**": " ** ",
  "*": " * ",
  "/": " / ",
  "%": " % ",
  "+": " + ",
  "-": " - ",
  "<<": " << ",
  ">>": " >> ",
  ">>>": " >>> ",
  "<": " < ",
  ">": " > ",
  "<=": " <= ",
  ">=": " >= ",
  instanceof: " instanceof ",
  in: " in ",
  "==": " == ",
  "!=": " != ",
  "===": " === ",
  "!==": " !== ",
  "&": " & ",
  "^": " ^ ",
  "|": " | ",
  "&&": " && ",
  "||": " || ",
  "??": " ?? ",
};

/**
 * Space-padded forms of every assignment operator.
 *
 * In pretty-print mode an operator is always surrounded by single spaces,
 * so the whole token is one constant string and needs a single `write` call.
 */
export const PADDED_ASSIGN_OPERATORS = {
  __proto__: null,
  "=": " = ",
  "+=": " += ",
  "-=": " -= ",
  "*=": " *= ",
  "/=": " /= ",
  "%=": " %= ",
  "**=": " **= ",
  "<<=": " <<= ",
  ">>=": " >>= ",
  ">>>=": " >>>= ",
  "&=": " &= ",
  "^=": " ^= ",
  "|=": " |= ",
  "&&=": " &&= ",
  "||=": " ||= ",
  "??=": " ??= ",
};

/**
 * The category a unary operator leaves in `last`, which is what `printSpaceBeforeOperator` reads
 * to decide whether the next operator needs separating.
 *
 * `typeof`, `void` and `delete` never reach here - they end in an identifier character and are handled as words.
 */
export function unaryOperatorCode(operator: string): Category {
  debugAssert(!["typeof", "void", "delete"].includes(operator));

  switch (operator) {
    case "+":
      return CAT_OP_UN_PLUS;
    case "-":
      return CAT_OP_UN_NEG;
    case "!":
      return CAT_OP_UN_NOT;
    default:
      return CAT_OTHER;
  }
}

/**
 * The category `++` or `--` leaves in `last`, as `unaryOperatorCode` does for unary operators.
 */
export function updateOperatorCode(operator: string): Category {
  return operator === "++" ? CAT_OP_UPD_INC : CAT_OP_UPD_DEC;
}
