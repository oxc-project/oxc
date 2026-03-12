import { RuleTester } from "../rule_tester.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "storybook",

  submoduleName: "storybook",
  testFilesDirPath: "code/lib/eslint-plugin/src/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Load the copy of TS-ESLint parser which is used by the test cases
    const tsEslintParser = require("@typescript-eslint/parser") as TSEslintParser;

    // Mock `@typescript-eslint/rule-tester` to use conformance `RuleTester`
    mock("@typescript-eslint/rule-tester", createTsRuleTester(tsEslintParser));

    // Mock `vitest` - it's ESM-only and can't be `require()`-ed.
    // `no-uninstalled-addons` test uses `vi.mock()` which can't work in CJS context anyway.
    mock("vitest", { vi: { mock: () => { }, importActual: () => ({}) } });
  },

  shouldSkipTest(ruleName: string): boolean {
    // `no-uninstalled-addons` relies on `vi.mock('fs')` and reading `package.json` files,
    // which can't work in the CJS conformance context.
    if (ruleName === "no-uninstalled-addons") {
      return true;
    }

    return false;
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],

  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
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

    // Patch test case filenames from `.js` to `.tsx` so oxlint parses as TypeScript.
    // Storybook's `StorybookRuleTester` sets `filename: 'MyComponent.stories.js'` as default,
    // but many test cases contain TypeScript and/or JSX syntax.
    run(ruleName: string, rule: Rule, tests: TestCases): void {
      tests = {
        valid: tests.valid.map((test) => {
          if (typeof test === "string") return test;
          return patchFilename(test);
        }),
        invalid: tests.invalid.map((test) => patchFilename(test)),
      };
      super.run(ruleName, rule, tests);
    }
  }

  return { RuleTester: TsRuleTester };
}

/**
 * Change `.js` file extension to `.tsx` so oxlint parses as TypeScript + JSX.
 * Storybook test cases may contain both TS syntax (type annotations) and JSX.
 */
function patchFilename<T extends TestCase>(test: T): T {
  const { filename } = test;
  if (filename != null && filename.endsWith(".js")) {
    return { ...test, filename: `${filename.slice(0, -".js".length)}.tsx` };
  }
  return test;
}
