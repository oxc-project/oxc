import { dirname } from "node:path";
import { createRequire } from "node:module";
import assert from "node:assert";

import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const require = createRequire(import.meta.url);
const fs = require("fs");

// Test for `no-uninstalled-addons` rule mocks `fs.readFileSync` using Vitest's `vi.mock()`
// to return a fake `package.json` file.
//
// We replicate this behavior by:
// 1. Mocking `vitest` and capturing the `package.json` file contents in a variable.
// 2. Tracking when we're in a test case for this plugin by adding `before` and `after` hooks to all test cases,
//    which set/unset `isInTestCase` flag.
// 3. Mocking `fs.readFileSync` to return the `package.json` file contents when `isInTestCase === true`.
let readFileSyncResult: string | null = null;

const vitestMock = {
  vi: {
    mock(moduleName: string, getMockedMethods: () => Record<string, Function>) {
      assert.equal(moduleName, "fs");
      const mockedMethods = getMockedMethods();
      assert(Object.keys(mockedMethods).length === 1);
      readFileSyncResult = mockedMethods.readFileSync();
    },
    importActual(moduleName: string): null {
      assert.equal(moduleName, "fs");
      return null;
    },
  },
};

let isInTestCase = false;

const readFileSyncOriginal = fs.readFileSync;
fs.readFileSync = function (...args: unknown[]): unknown {
  if (isInTestCase && readFileSyncResult !== null) return readFileSyncResult;
  return readFileSyncOriginal.apply(this, args);
};

// Test group definition
const group: TestGroup = {
  name: "storybook",
  ...repos.storybook,

  submoduleName: "storybook",
  testFilesDirPath: "code/lib/eslint-plugin/src/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Load the copy of TS-ESLint parser which is used by the test cases
    const tsEslintParser = require("@typescript-eslint/parser") as TSEslintParser;

    // Get path to `code` directory in submodule
    const codeDirPath = dirname(require.resolve("../../../../package.json"));

    // Mock `vitest` to capture call to `vi.mock()` which `no-uninstalled-addons` rule test uses
    mock("vitest", vitestMock);

    // Mock `@typescript-eslint/rule-tester` to use conformance `RuleTester`,
    // and add `before` and `after` hooks to track when we're in a test case for this plugin.
    mock("@typescript-eslint/rule-tester", createTsRuleTester(tsEslintParser, codeDirPath));
  },

  ruleTesters: [],

  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;

/**
 * Create a module to replace `@typescript-eslint/rule-tester`,
 * which presents the same API, but uses conformance `RuleTester` with TS-ESLint parser.
 * It also adds `before` and `after` hooks to all test cases to track when we're in a test case for this plugin.
 *
 * @param tsEslintParser - TSESLint parser module
 * @param codeDirPath - Path to `code` directory in submodule
 * @returns Module to replace `@typescript-eslint/rule-tester` module with
 */
function createTsRuleTester(
  tsEslintParser: TSEslintParser,
  codeDirPath: string,
): { RuleTester: typeof RuleTester } {
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
          if (typeof test === "string") test = { code: test };
          return patchTestCase(test, codeDirPath);
        }),
        invalid: tests.invalid.map((test) => patchTestCase(test, codeDirPath)),
      };
      super.run(ruleName, rule, tests);
    }
  }

  return { RuleTester: TsRuleTester };
}

/**
 * Change `.js` file extension to `.tsx` so oxlint parses as TypeScript + JSX.
 * Storybook test cases may contain both TS syntax (type annotations) and JSX.
 *
 * Track when in a test case for this plugin by adding `before` and `after` hooks to all test cases.
 * Set CWD to `codeDirPath` (`code` directory in submodule) while running a test case.
 * The last 2 are required for `no-uninstalled-addons` rule test to work (see above).
 *
 * @param test - Test case
 * @param codeDirPath - Path to `code` directory in submodule
 * @returns Test case with `filename` property patched, and `before` and `after` hooks added
 */
function patchTestCase<T extends TestCase>(test: T, codeDirPath: string): T {
  test = { ...test };

  const { filename } = test;
  if (filename != null && filename.endsWith(".js")) {
    test.filename = `${filename.slice(0, -".js".length)}.tsx`;
  }

  let cwd: string | undefined;

  // oxlint-disable-next-line typescript/unbound-method
  const { before, after } = test;
  test.before = function () {
    isInTestCase = true;
    cwd = process.cwd();
    process.chdir(codeDirPath);
    if (before != null) before.call(this);
  };
  test.after = function () {
    if (after != null) after.call(this);
    process.chdir(cwd!);
    isInTestCase = false;
  };

  return test;
}
