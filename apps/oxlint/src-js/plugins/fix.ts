import { getMessage } from "./report.ts";

import type { RuleDetails } from "./load.ts";
import type { Range, Ranged } from "./location.ts";
import type { Diagnostic, Suggestion, SuggestionReport } from "./report.ts";

// Type of `fix` function.
// `fix` can return a single fix, an array of fixes, or any iterator that yields fixes.
// e.g. `(function*() { yield fix1; yield fix2; })()`
export type FixFn = (
  fixer: Fixer,
) =>
  | Fix
  | Array<Fix | null | undefined>
  | IterableIterator<Fix | null | undefined>
  | null
  | undefined;

/**
 * Fix, as returned by `fix` function.
 *
 * `range` offsets are relative to start of the source text.
 * When the file has a BOM, they are relative to the start of the source text *without* the BOM.
 *
 * To represent a position *before* a BOM, -1 is used to mean "before the BOM".
 * ESLint's `unicode-bom` rule produces a fix `{ range: [-1, 0], text: "" }` to remove a BOM.
 */
export interface Fix {
  range: Range;
  text: string;
}

/**
 * Fix, in form sent to Rust.
 *
 * `start` and `end` are relative to start of the source text.
 * When the file has a BOM, they are relative to the start of the source text *without* the BOM.
 *
 * To represent a position *before* a BOM, -1 is used to mean "before the BOM".
 * ESLint's `unicode-bom` rule produces a fix `{ range: [-1, 0], text: "" }` to remove a BOM.
 *
 * This type's equivalent on Rust side is `JsFix`, which has `start` and `end` properties as `i64`s,
 * to allow negative values.
 */
export interface FixReport {
  start: number;
  end: number;
  text: string;
}

// Fixer, passed as argument to `fix` function passed to `Context#report()`.
//
// Fixer is stateless, so reuse a single object for all fixes.
// Freeze the object to prevent user mutating it.
const FIXER = Object.freeze({
  insertTextBefore(nodeOrToken: Ranged, text: string): Fix {
    const start = nodeOrToken.range[0];
    return { range: [start, start], text };
  },
  insertTextBeforeRange(range: Range, text: string): Fix {
    const start = range[0];
    return { range: [start, start], text };
  },
  insertTextAfter(nodeOrToken: Ranged, text: string): Fix {
    const end = nodeOrToken.range[1];
    return { range: [end, end], text };
  },
  insertTextAfterRange(range: Range, text: string): Fix {
    const end = range[1];
    return { range: [end, end], text };
  },
  remove(nodeOrToken: Ranged): Fix {
    return { range: nodeOrToken.range, text: "" };
  },
  removeRange(range: Range): Fix {
    return { range, text: "" };
  },
  replaceText(nodeOrToken: Ranged, text: string): Fix {
    return { range: nodeOrToken.range, text };
  },
  replaceTextRange(range: Range, text: string): Fix {
    return { range, text };
  },
});

export type Fixer = typeof FIXER;

/**
 * Get fixes from a `Diagnostic`.
 *
 * Returns `null` if no `fix` function, or if it produces no fixes.
 * Throws if rule is not marked as fixable but produces fixes.
 *
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `isFixable` value
 * @returns Non-empty array of `FixReport` objects, or `null` if none
 * @throws {Error} If rule is not marked as fixable but `fix` function returns fixes,
 *   or if `fix` function returns any invalid `Fix` objects
 */
export function getFixes(diagnostic: Diagnostic, ruleDetails: RuleDetails): FixReport[] | null {
  // ESLint silently ignores non-function `fix` values, so we do the same
  const { fix } = diagnostic;
  if (typeof fix !== "function") return null;

  const fixes = getFixesFromFixFn(fix, diagnostic);

  // ESLint does not throw an error if `fix` function returns only falsy values
  if (fixes !== null && ruleDetails.isFixable === false) {
    throw new Error(
      'Fixable rules must set the `meta.fixable` property to "code" or "whitespace".',
    );
  }

  return fixes;
}

/**
 * Get suggestions from a `Diagnostic`.
 *
 * Returns `null` if no `suggest` array, or if it produces no suggestions
 * (e.g. all fix functions return falsy values).
 *
 * Throws if rule is not marked with `meta.hasSuggestions` but produces suggestions.
 *
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific details
 * @returns Non-empty array of `SuggestionReport` objects, or `null` if none
 * @throws {Error} If rule is not marked with `meta.hasSuggestions` but produces suggestions
 * @throws {TypeError} If a suggestion's `fix` is not a function, or message is invalid
 */
export function getSuggestions(
  diagnostic: Diagnostic,
  ruleDetails: RuleDetails,
): SuggestionReport[] | null {
  if (!Object.hasOwn(diagnostic, "suggest")) return null;
  const { suggest } = diagnostic;
  if (suggest == null) return null;

  const suggestLen = suggest.length;
  if (suggestLen === 0) return null;

  const suggestions: SuggestionReport[] = [];
  for (let i = 0; i < suggestLen; i++) {
    const suggestion = suggest[i];

    // Validate fix is a function (matches ESLint)
    const { fix } = suggestion;
    if (typeof fix !== "function") throw new TypeError("Suggestion without a fix function");

    // Get suggestion message
    const { message, messageId } = getMessage(
      Object.hasOwn(suggestion, "desc") ? suggestion.desc : null,
      suggestion,
      ruleDetails,
    );

    // Call fix function - drop suggestion if fix function produces no fixes
    const fixes = getFixesFromFixFn(fix, suggestion);
    if (fixes !== null) suggestions.push({ message, messageId, fixes });
  }

  if (suggestions.length === 0) return null;

  // Check rule has suggestions enabled.
  // This check is skipped if no suggestions are produced, matching what ESLint does.
  if (ruleDetails.hasSuggestions === false) {
    throw new Error("Rules with suggestions must set `meta.hasSuggestions` to `true`.");
  }

  return suggestions;
}

/**
 * Call a `FixFn` and process its return value into an array of `FixReport` objects.
 *
 * Returns `null` if any of:
 *
 * 1. `fixFn` returns a falsy value.
 * 2. `fixFn` returns an empty array/iterator.
 * 3. `fixFn` returns an array/iterator containing only falsy values.
 *
 * Otherwise, returns a non-empty array of `FixReport` objects.
 *
 * `Fix` objects are validated.
 *
 * This function aims to replicate ESLint's behavior as closely as possible.
 *
 * @param fixFn - Fix function to call
 * @param thisArg - `this` value for the fix function call
 * @returns Non-empty array of `FixReport` objects, or `null` if none
 * @throws {Error} If `fixFn` returns any invalid `Fix` objects
 */
function getFixesFromFixFn(fixFn: FixFn, thisArg: Diagnostic | Suggestion): FixReport[] | null {
  // In ESLint, `fix` is called with `this` as a clone of the `Diagnostic` or `Suggestion` object.
  // We just use the original object - that should be close enough.
  const fixes = fixFn.call(thisArg, FIXER);

  // ESLint ignores falsy values
  if (!fixes) return null;

  // `fixes` can be any iterator, not just an array e.g. `fix: function*() { yield fix1; yield fix2; }`
  if (Symbol.iterator in fixes) {
    const fixReports: FixReport[] = [];
    for (const fix of fixes) {
      // ESLint ignores falsy values
      if (fix) fixReports.push(validateAndConvertFix(fix));
    }

    return fixReports.length === 0 ? null : fixReports;
  }

  return [validateAndConvertFix(fixes)];
}

/**
 * Validate that a `Fix` object is well-formed, and convert it to a `FixReport`.
 *
 * Check that `range` has 2 numeric elements, and convert `text` to string if needed.
 *
 * Purpose of validation is to ensure any input which ESLint accepts does not cause an error in `JSON.stringify()`,
 * or in deserializing on Rust side.
 *
 * @param fix - Fix object to validate, possibly malformed
 * @returns `FixReport` object
 * @throws {Error} If `fix` has invalid `range`
 */
function validateAndConvertFix(fix: Fix): FixReport {
  const { range, text } = fix;

  if (range != null) {
    const start = range[0],
      end = range[1];
    if (typeof start === "number" && typeof end === "number") {
      // Converting `text` to string follows ESLint, which does that implicitly.
      return { start, end, text: String(text) };
    }
  }

  throw new Error(`Fix has invalid range: ${JSON.stringify(fix, null, 2)}`);
}
