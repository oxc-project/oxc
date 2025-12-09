/*
 * Shim of `RuleTester` class.
 */

import { RuleTester } from "#oxlint";
import { describe, it } from "./capture.ts";

type DescribeFn = RuleTester.DescribeFn;
type ItFn = RuleTester.ItFn;
type ValidTestCase = RuleTester.ValidTestCase;
type InvalidTestCase = RuleTester.InvalidTestCase;
type TestCase = ValidTestCase | InvalidTestCase;

// Set up `RuleTester` to use our hooks
RuleTester.describe = describe;
RuleTester.it = it;

/**
 * Shim of `RuleTester` class.
 * Prevents overriding `describe` and `it` properties.
 */
class RuleTesterShim extends RuleTester {
  // Prevent changing `describe` or `it` properties

  static get describe(): DescribeFn {
    return describe;
  }

  static set describe(_value: DescribeFn) {
    throw new Error("Cannot override `describe` property");
  }

  static get it(): ItFn {
    return it;
  }

  static set it(_value: ItFn) {
    throw new Error("Cannot override `it` property");
  }

  static get itOnly(): ItFn {
    return it.only;
  }

  static set itOnly(_value: ItFn) {
    throw new Error("Cannot override `itOnly` property");
  }
}

// Register hook to modify test cases before they are run.
// `registerModifyTestCaseHook` is only present in debug builds, so it's not part of the `RuleTester` type def.
(RuleTester as any).registerModifyTestCaseHook(modifyTestCase);

function modifyTestCase(test: TestCase): void {
  // Enable ESLint compat mode.
  // This makes `RuleTester` adjust column indexes in diagnostics to match ESLint's behavior.
  test.eslintCompat = true;
}

export { RuleTesterShim as RuleTester };
