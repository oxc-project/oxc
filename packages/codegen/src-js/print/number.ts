// Number printing (port of `oxc_ecmascript::number_literal`, Terser-derived).
//
// The shortest text for a number is the shortest of a handful of forms, and which of them are even possible
// follows from what `String` gave - an integer can go hexadecimal or trade trailing zeros for an exponent,
// a fraction can drop its leading zero or trade leading zeros for one, and only a number `String` already wrote
// in exponent notation can fold its point into the exponent. So the forms are tried by separate functions,
// rather than by one which tries all of them on every number.
//
// Each form is written to the output in pieces, so no candidate string is ever built.

import { debugAssert } from "../asserts.ts";
import {
  CAT_IDENT,
  CAT_INT_DIGIT,
  write,
  writeNoLast,
  writeWithMap,
  writeWithMapNoLast,
} from "./write.ts";

import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print a non-negative finite number in its shortest form.
 *
 * Integers below 1000 are the overwhelming majority, and 3 digits are too few for any other form
 * to be shorter, so they print straight from `String` with none of the shortening work in the way.
 *
 * `CAT_INT_DIGIT` goes only on plain digits. It is what forces the space in `0 .toExponential()`,
 * and every other form ends in, or contains, a `.`, an `e` or an `x` which separates it
 * from a following `.` already.
 */
export function printNonNegativeFloat(state: State, value: number, node: ESTree.Node): void {
  if (Number.isInteger(value)) {
    if (value < 1000) {
      writeWithMap(state, String(value), CAT_INT_DIGIT, node);
    } else {
      printShortestInteger(state, value, node);
    }
  } else {
    printShortestFraction(state, value, node);
  }
}

/**
 * Print an integer of 1000 or more in its shortest form.
 *
 * `String` gives plain digits below 1e21 and exponent notation from there up, which have different forms
 * available to them, so they are separate cases. Both can go hexadecimal.
 */
function printShortestInteger(state: State, value: number, node: ESTree.Node): void {
  const formatted = String(value);

  if (value >= 1e21) {
    debugAssert(formatted.includes("e+"), "`String` gives `e+` notation from 1e21 up");
    // The `+` always goes, so the text is one shorter than `formatted`
    if (printHexIfShorter(state, value, formatted.length - 1, node)) return;
    printExponent(state, formatted, formatted.indexOf("e"), node);
    return;
  }

  const { length } = formatted;
  if (printHexIfShorter(state, value, length, node)) return;

  // A run of trailing zeros as an exponent: `1000` -> `1e3`
  if (formatted.charCodeAt(length - 1) === 48 /* 0 */) {
    // The first digit is never a zero, so the run always stops
    let zeros = 1;
    while (formatted.charCodeAt(length - 1 - zeros) === 48 /* 0 */) zeros++;

    // Worth it when the `e` and the exponent cost less than the zeros they replace
    const exponent = String(zeros);
    if (exponent.length + 1 < zeros) {
      writeWithMapNoLast(state, formatted.slice(0, length - zeros), node);
      writeNoLast(state, "e");
      write(state, exponent, CAT_IDENT);
      return;
    }
  }

  writeWithMap(state, formatted, CAT_INT_DIGIT, node);
}

/**
 * Print a non-integer in its shortest form.
 *
 * `String` gives `0.<digits>` from 1e-7 up to 1, `<digits>.<digits>` from 1 up, and exponent notation
 * below 1e-7. The first always loses its leading zero, and may trade a run of zeros after the point
 * for a negative exponent. The second is already as short as it gets.
 */
function printShortestFraction(state: State, value: number, node: ESTree.Node): void {
  const formatted = String(value);

  // The trailing-zero form `printShortestInteger` uses can never win here, so it is not tried.
  // The shortest text for a fraction never ends in a zero digit, and an exponent of at most 3 digits
  // never ends in the 3 zeros it would take to pay for the `e`.
  if (DEBUG) {
    let zeros = 0;
    while (formatted.charCodeAt(formatted.length - 1 - zeros) === 48 /* 0 */) zeros++;
    debugAssert(zeros < 3, () => `Fraction text ends in ${zeros} zeros: ${formatted}`);
  }

  if (formatted.charCodeAt(0) === "0".charCodeAt(0)) {
    // `0.<digits>`, whose leading zero always goes
    if (formatted.charCodeAt(2) === "0".charCodeAt(0)) {
      // A run of zeros straight after the point as a negative exponent: `0.0001` -> `1e-4`.
      // There is always a non-zero digit for the run to stop at, since the text is the shortest
      // which reads back as `value`.
      let start = 3;
      while (formatted.charCodeAt(start) === "0".charCodeAt(0)) start++;

      // The exponent is the count of digits after the point, which is what shifting the point
      // past all of them costs, zeros included
      const exponent = String(formatted.length - 2);

      // Both forms keep the digits, so it comes down to the `e-` and the exponent against the `.`
      // and the zeros they replace
      if (exponent.length + 2 < start - 1) {
        writeWithMapNoLast(state, formatted.slice(start), node);
        writeNoLast(state, "e-");
        write(state, exponent, CAT_IDENT);
        return;
      }
    }

    writeWithMap(state, formatted.slice(1), CAT_IDENT, node);
    return;
  }

  const exponentIndex = formatted.indexOf("e");
  if (exponentIndex === -1) {
    writeWithMap(state, formatted, CAT_IDENT, node);
    return;
  }

  printExponent(state, formatted, exponentIndex, node);
}

/**
 * Print a number `String` gave in exponent notation, in its shortest form.
 *
 * `String` writes exactly one digit before the point, so the digits after it can be folded into the exponent -
 * `1.5e+21` -> `15e20`. That drops the `.`, so it wins whenever the exponent it leaves is no longer to write
 * than the one it replaces. The `+` of a positive exponent always goes.
 *
 * @param formatted - `String(value)`, which is in exponent notation
 * @param exponentIndex - Index of the `e` in `formatted`
 */
function printExponent(
  state: State,
  formatted: string,
  exponentIndex: number,
  node: ESTree.Node,
): void {
  const exponent =
    formatted.charCodeAt(exponentIndex + 1) === "+".charCodeAt(0)
      ? formatted.slice(exponentIndex + 2)
      : formatted.slice(exponentIndex + 1);

  if (exponentIndex > 1) {
    debugAssert(
      formatted.charCodeAt(1) === ".".charCodeAt(0),
      "Exponent notation has 1 digit up front",
    );

    const folded = String(Number(exponent) - (exponentIndex - 2));
    if (folded.length <= exponent.length) {
      writeWithMapNoLast(state, formatted[0], node);
      writeNoLast(state, formatted.slice(2, exponentIndex));
      writeNoLast(state, "e");
      write(state, folded, CAT_IDENT);
      return;
    }
  }

  writeWithMapNoLast(state, formatted.slice(0, exponentIndex + 1), node);
  write(state, exponent, CAT_IDENT);
}

/**
 * Print `value` in hexadecimal if that is shorter than `length` characters, and report whether it was.
 *
 * Hexadecimal has to win back the `0x` in front of it, which takes 13 digits at the very least,
 * so below that the conversion is not attempted at all.
 *
 * That bound is in decimal digits, and the caller with a number in exponent notation passes the length
 * of that instead, which is shorter - but such a number is at least 1e21, whose 18 hex digits make 20 characters,
 * so anything the early return skips there could not have won either.
 *
 * `BigInt` rather than `value.toString(16)`, whose result the specification leaves implementation-approximated
 * for every radix but 10, where the output here has to be exact.
 *
 * @param length - Length of the shortest text found for `value` so far
 */
function printHexIfShorter(
  state: State,
  value: number,
  length: number,
  node: ESTree.Node,
): boolean {
  if (length < 13) return false;

  const hex = BigInt(value).toString(16);
  if (hex.length + 2 >= length) return false;

  writeWithMapNoLast(state, "0x", node);
  write(state, hex, CAT_IDENT);

  return true;
}
