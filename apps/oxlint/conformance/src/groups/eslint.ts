import type { TestGroup } from "../index.ts";
import type { TestCase } from "../rule_tester.ts";

const group: TestGroup = {
  name: "eslint",

  submoduleName: "eslint",
  testFilesDirPath: "tests/lib/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".js")) return null;
    if (filename.startsWith("utils/")) return null;
    return filename.slice(0, -3);
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Skip test cases which start with `/* global */`, `/* globals */`, `/* exported */`, or `/* eslint */` comments.
    // Oxlint does not support defining globals inline.
    // `RuleTester` does not support enabling other rules beyond the rule under test.
    if (code.match(/^\s*\/\*\s*(globals?|exported|eslint)\s/)) return true;

    // Skip test cases which include `// eslint-disable` comments.
    // These are not handled by `RuleTester`.
    if (code.match(/\/\/\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

    // Test relies on scope analysis to follow ES5 semantics where function declarations in blocks are bound in parent scope.
    // TS-ESLint scope manager does not support ES5. Oxc also doesn't support parsing/semantic as ES5.
    if (
      ruleName === "no-use-before-define" &&
      code === '"use strict"; a(); { function a() {} }' &&
      test.languageOptions?.ecmaVersion === 5
    ) {
      return true;
    }

    // Code contains unrecoverable syntax error - `function (x, this: context) {}`
    if (
      ruleName === "no-invalid-this" &&
      code.includes("function (x, this: context) {") &&
      err?.message === "Parsing failed"
    ) {
      return true;
    }

    // TypeScript parser does not support HTML comments
    if (ruleName === "prefer-object-spread" && code.includes("<!--")) return true;

    return false;
  },

  ruleTesters: [{ specifier: "../../../lib/rule-tester/rule-tester.js", propName: null }],
  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;
