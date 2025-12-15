import { typeAssertIs } from "../utils/asserts.ts";

import type { RuleDetails } from "./load.ts";
import type { Range, Ranged } from "./location.ts";
import type { Diagnostic } from "./report.ts";

const { prototype: ArrayPrototype, from: ArrayFrom } = Array,
  { getPrototypeOf, hasOwn, prototype: ObjectPrototype } = Object,
  { ownKeys } = Reflect,
  IteratorSymbol = Symbol.iterator;

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

// Type of a fix, as returned by `fix` function.
export type Fix = { range: Range; text: string };

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
 * Returns `null` if any of:
 *
 * 1. No `fix` function was provided in `diagnostic`.
 * 2. `fix` function returns a falsy value.
 * 3. `fix` function returns an empty array/iterator.
 * 4. `fix` function returns an array/iterator containing only falsy values.
 *
 * Otherwise, returns a non-empty array of `Fix` objects.
 *
 * `Fix` objects are validated and conformed to expected shape.
 * Does not mutate the `fixes` array returned by `fix` function, but avoids cloning if possible.
 *
 * This function aims to replicate ESLint's behavior as closely as possible.
 *
 * TODO: Are prototype checks, and checks for `toJSON` methods excessive?
 * We're not handling all possible edge cases e.g. `fixes` or individual `Fix` objects being `Proxy`s or objects
 * with getters. As we're not managing to be 100% bulletproof anyway, maybe we don't need to be quite so defensive.
 *
 * @param diagnostic - Diagnostic object
 * @param ruleDetails - `RuleDetails` object, containing rule-specific `isFixable` value
 * @returns Non-empty array of `Fix` objects, or `null` if none
 * @throws {Error} If rule is not marked as fixable but `fix` function returns fixes,
 *   or if `fix` function returns any invalid `Fix` objects
 */
export function getFixes(diagnostic: Diagnostic, ruleDetails: RuleDetails): Fix[] | null {
  // ESLint silently ignores non-function `fix` values, so we do the same
  const { fix } = diagnostic;
  if (typeof fix !== "function") return null;

  // In ESLint, `fix` is called with `this` as a clone of the `diagnostic` object.
  // We just use the original `diagnostic` object - that should be close enough.
  let fixes = fix.call(diagnostic, FIXER);

  // ESLint ignores falsy values
  if (!fixes) return null;

  // `fixes` can be any iterator, not just an array e.g. `fix: function*() { yield fix1; yield fix2; }`
  if (IteratorSymbol in fixes) {
    let isCloned = false;

    // Check prototype instead of using `Array.isArray()`, to ensure it is a native `Array`,
    // not a subclass which may have overridden `toJSON()` in a way which could make `JSON.stringify()` throw
    if (getPrototypeOf(fixes) !== ArrayPrototype || hasOwn(fixes, "toJSON")) {
      fixes = ArrayFrom(fixes);
      isCloned = true;
    }

    const fixesLen = fixes.length;
    if (fixesLen === 0) return null;

    for (let i = 0; i < fixesLen; i++) {
      const fix = fixes[i];

      // ESLint ignores falsy values.
      // Filter them out. This branch can only be taken once.
      if (!fix) {
        fixes = fixes.filter(Boolean);
        if (fixes.length === 0) return null;
        isCloned = true;
        i--;
        continue;
      }

      const conformedFix = validateAndConformFix(fix);
      if (conformedFix !== fix) {
        // Don't mutate `fixes` array
        if (isCloned === false) {
          fixes = fixes.slice();
          isCloned = true;
        }
        fixes[i] = conformedFix;
      }
    }
  } else {
    fixes = [validateAndConformFix(fixes)];
  }

  // ESLint does not throw this error if `fix` function returns only falsy values.
  // We've already exited if that is the case, so we're reproducing that behavior.
  if (ruleDetails.isFixable === false) {
    throw new Error(
      'Fixable rules must set the `meta.fixable` property to "code" or "whitespace".',
    );
  }

  return fixes;
}

/**
 * Validate that a `Fix` object is well-formed, and conform it to expected shape.
 *
 * - Convert `text` to string if needed.
 * - Shorten `range` to 2 elements if it has extra elements.
 * - Remove any additional properties on the object.
 *
 * Purpose is to ensure any input which ESLint accepts does not cause an error in `JSON.stringify()`,
 * or in deserializing on Rust side.
 *
 * @param fix - Fix object to validate, possibly malformed
 * @returns `Fix` object
 */
function validateAndConformFix(fix: unknown): Fix {
  typeAssertIs<Fix>(fix);
  const { range, text } = fix;

  // These checks follow ESLint, which throws if `range` is missing or invalid
  if (!range || typeof range[0] !== "number" || typeof range[1] !== "number") {
    throw new Error(`Fix has invalid range: ${JSON.stringify(fix, null, 2)}`);
  }

  // If `fix` is already well-formed, return it as-is.
  // Note: `ownKeys(fix).length === 2` rules out `fix` having a custom `toJSON` method.
  const fixPrototype = getPrototypeOf(fix);
  if (
    (fixPrototype === ObjectPrototype || fixPrototype === null) &&
    ownKeys(fix).length === 2 &&
    getPrototypeOf(range) === ArrayPrototype &&
    !hasOwn(range, "toJSON") &&
    range.length === 2 &&
    typeof text === "string"
  ) {
    return fix;
  }

  // Conform fix object to expected shape.
  // Converting `text` to string follows ESLint, which does that implicitly.
  return { range: [range[0], range[1]], text: String(text) };
}
