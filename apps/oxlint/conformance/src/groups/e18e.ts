import { RuleTester } from "../rule_tester.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase } from "../rule_tester.ts";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "e18e",

  submoduleName: "e18e",
  testFilesDirPath: "src/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Load the copy of TS-ESLint parser which is used by the test cases
    const tsEslintParser = require("typescript-eslint").parser as TSEslintParser;

    // Mock `@typescript-eslint/rule-tester` to use conformance `RuleTester`
    mock("@typescript-eslint/rule-tester", createTsRuleTester(tsEslintParser));
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Cannot lint JSON files
    if (
      err.message === "Parsing failed" &&
      ((ruleName === "ban-dependencies" && test.filename === "package.json") ||
        (ruleName === "ban-dependencies (JSON)" && (test as any).language === "json/json"))
    ) {
      return true;
    }

    // Type-aware rules are not supported
    if (
      ((ruleName === "prefer-array-at (typed)" &&
        err.message.startsWith("Should have no errors but had 1:")) ||
        (ruleName === "prefer-regex-test (typed)" &&
          err.message.startsWith("Should have 1 error but had 0:")) ||
        (ruleName === "no-indexof-equality" &&
          err.message.startsWith(
            "You have used a rule which requires type information. " +
              "Please ensure you have typescript-eslint setup alongside this plugin " +
              "and configured to enable type-aware linting.",
          ))) &&
      (test.languageOptions?.parserOptions as any)?.projectService != null
    ) {
      return true;
    }

    return false;
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],

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
function createTsRuleTester(tsEslintParser: TSEslintParser): { RuleTester: typeof RuleTester } {
  class TsRuleTester extends RuleTester {
    constructor(config?: { languageOptions?: LanguageOptions } | null) {
      super({
        ...config,
        languageOptions: {
          parser: tsEslintParser,
          ...config?.languageOptions,
        },
      });
    }
  }

  return { RuleTester: TsRuleTester };
}
