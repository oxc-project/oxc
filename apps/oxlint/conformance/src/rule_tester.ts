/*
 * Shim of `RuleTester` class.
 */

import { createRequire } from "node:module";
import { RuleTester } from "#oxlint";
import { describe, it } from "./capture.ts";
import { ESLINT_RULES_TESTS_DIR_PATH } from "./run.ts";
import { FILTER_ONLY_CODE } from "./filter.ts";

import type { Rule } from "#oxlint";

type DescribeFn = RuleTester.DescribeFn;
type ItFn = RuleTester.ItFn;
type TestCases = RuleTester.TestCases;
type ValidTestCase = RuleTester.ValidTestCase;
type InvalidTestCase = RuleTester.InvalidTestCase;
export type TestCase = ValidTestCase | InvalidTestCase;

const { isArray } = Array;

// Get `@typescript-eslint/parser` module.
// Load the instance which would be loaded by files in ESLint's `tests/lib/rules` directory.
const require = createRequire(ESLINT_RULES_TESTS_DIR_PATH);
const tsEslintParser = require("@typescript-eslint/parser");

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

  // Apply filter to test cases.
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    if (FILTER_ONLY_CODE !== null) {
      const codeMatchesFilter = isArray(FILTER_ONLY_CODE)
        ? (code: string) => FILTER_ONLY_CODE!.includes(code)
        : (code: string) => code === FILTER_ONLY_CODE;

      tests = {
        valid: tests.valid.filter((test) => {
          const code = typeof test === "string" ? test : test.code;
          return codeMatchesFilter(code);
        }),
        invalid: tests.invalid.filter((test) => codeMatchesFilter(test.code)),
      };
    }

    super.run(ruleName, rule, tests);
  }
}

// Register hook to modify test cases before they are run.
// `registerModifyTestCaseHook` is only present in debug builds, so it's not part of the `RuleTester` type def.
(RuleTester as any).registerModifyTestCaseHook(modifyTestCase);

function modifyTestCase(test: TestCase): void {
  // Enable ESLint compat mode.
  // This makes `RuleTester` adjust column indexes in diagnostics to match ESLint's behavior,
  // and enables `sourceType: "commonjs"`.
  test.eslintCompat = true;

  // If test case uses `@typescript-eslint/parser` as parser, set `parserOptions.lang = "ts"`
  let { languageOptions } = test;
  if (languageOptions?.parser === tsEslintParser) {
    languageOptions = { ...languageOptions };
    test.languageOptions = languageOptions;

    delete languageOptions.parser;

    const parserOptions = { ...languageOptions.parserOptions };
    languageOptions.parserOptions = parserOptions;

    parserOptions.lang = parserOptions.ecmaFeatures?.jsx === true ? "tsx" : "ts";
  }
}

export { RuleTesterShim as RuleTester };
