import { RuleTester } from "../rule_tester.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "testing_library",

  submoduleName: "testing_library",
  testFilesDirPath: "tests/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Load the copy of TS-ESLint parser which is used by the test cases
    const tsEslintParser = require("typescript-eslint").parser as TSEslintParser;

    // Mock `@typescript-eslint/rule-tester` to use conformance `RuleTester` with patches
    mock("@typescript-eslint/rule-tester", createRuleTesterModule(tsEslintParser));
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Test case defines an option as `undefined`. We can't support that as `undefined` is not JSON-serializable.
    if (
      ruleName === "no-debugging-utils" &&
      code.trim().replace(/\n\s+/g, "\n") ===
        "import { screen } from '@testing-library/dom'\nscreen.logTestingPlaygroundURL()" &&
      err.message.startsWith("Should have no errors but had 1:")
    ) {
      const { options } = test;
      if (options != null && options.length === 1) {
        const firstOption = options[0];
        if (
          typeof firstOption === "object" &&
          firstOption !== null &&
          !Array.isArray(firstOption) &&
          Object.hasOwn(firstOption, "utilsToCheckFor") &&
          firstOption.utilsToCheckFor === undefined
        ) {
          return true;
        }
      }
    }

    return false;
  },

  ruleTesters: [],

  parsers: [{ specifier: "typescript-eslint", propName: "parser", lang: "ts" }],
};

export default group;

/**
 * Create a module to replace `@typescript-eslint/rule-tester`,
 * which presents the same API, but uses conformance `RuleTester` with TS-ESLint parser.
 *
 * @param tsEslintParser - TSESLint parser module
 * @returns Module to replace `@typescript-eslint/rule-tester` module with
 */
function createRuleTesterModule(tsEslintParser: TSEslintParser): { RuleTester: typeof RuleTester } {
  class PatchedRuleTester extends RuleTester {
    #languageOptions: LanguageOptions;

    constructor(config?: { languageOptions?: LanguageOptions } | null) {
      // Set parser as TS-ESLint parser
      const languageOptions = {
        parser: tsEslintParser,
        ...config?.languageOptions,
      };

      super({ ...config, languageOptions });

      // Store config `languageOptions` for use in `patchTestCase`
      this.#languageOptions = languageOptions;
    }

    // Patch test cases
    run(ruleName: string, rule: Rule, tests: TestCases): void {
      const languageOptions = this.#languageOptions;

      tests = {
        valid: tests.valid.map((test) => {
          if (typeof test === "string") test = { code: test };

          // Some valid test cases have `errors: []` property, which is illegal. Remove it.
          const { errors, ...rest } = test as typeof test & { errors?: unknown[] };
          if (Array.isArray(errors)) test = rest;

          // Alter file extension for JSX files
          return patchTestCase(test, languageOptions);
        }),
        invalid: tests.invalid.map((test) => {
          // TS-ESlint `RuleTester` accepts `output` as an array of strings,
          // where first string is the output after 1 round of fixes
          const output = test.output as string | string[] | null | undefined;
          if (Array.isArray(output) && output.length > 0) test = { ...test, output: output[0] };

          // Alter file extension for JSX files
          return patchTestCase(test, languageOptions);
        }),
      };
      super.run(ruleName, rule, tests);
    }
  }

  return { RuleTester: PatchedRuleTester };
}

/**
 * Patch test case to make `filename`'s extension `.tsx` if `languageOptions.parserOptions.ecmaFeatures.jsx` is `true`.
 * Otherwise, `filename: "whatever.test.js"` takes precedence over `jsx: true`, and fails to parse.
 * @param test - Test case
 * @param languageOptions - Language options from config
 * @returns Patched test case
 */
function patchTestCase<T extends TestCase>(test: T, languageOptions: LanguageOptions): T {
  // If no `filename` specified, no patching is required - `RuleTester` will determine language as TSX
  const { filename } = test;
  if (filename == null) return test;

  // Get `jsx` option from config or test case
  let isJsx = languageOptions.parserOptions?.ecmaFeatures?.jsx ?? false;
  const testIsJsx = test.languageOptions?.parserOptions?.ecmaFeatures?.jsx;
  if (testIsJsx != null) isJsx = testIsJsx;

  // Change file extension
  if (isJsx && filename.endsWith(".js")) {
    return { ...test, filename: `${filename.slice(0, -2)}tsx` };
  }

  return test;
}
