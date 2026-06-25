import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "playwright",
  ...repos.playwright,

  submoduleName: "playwright",
  testFilesDirPath: "src/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    // `rules.test.ts` is a meta-test for plugin exports/README, not a rule test.
    if (filename === "rules.test.ts") return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Load the TS-ESLint parser used by some test cases (via `runTSRuleTester`)
    const tsEslintParser = require("@typescript-eslint/parser") as TSEslintParser;

    // Mock the plugin's `src/utils/rule-tester` module.
    // The original imports `describe` and `it` from `vitest` (ESM-only, can't be `require()`-ed)
    // and sets `RuleTester.describe`/`.it`/`.itOnly`, which the conformance `RuleTester` prevents.
    // So we replace the entire module with conformance-compatible wrapper functions.
    mock("../utils/rule-tester", createRuleTesterModule(tsEslintParser));
  },

  ruleTesters: [],

  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;

/**
 * Create a module to replace the plugin's `src/utils/rule-tester` module.
 * Provides `runRuleTester`, `runTSRuleTester`, and `test` functions
 * that use the conformance `RuleTester`.
 *
 * The original module creates `RuleTester` instances with specific language options
 * and calls `.run()`. We replicate that behavior using the conformance `RuleTester`.
 *
 * @param tsEslintParser - TSESLint parser module
 * @returns Module to replace `src/utils/rule-tester` module with
 */
function createRuleTesterModule(tsEslintParser: TSEslintParser) {
  function runRuleTester(name: string, rule: Rule, tests: TestCases) {
    const languageOptions: LanguageOptions = {
      ecmaVersion: 2022,
      sourceType: "module",
    };

    return new RuleTester({ languageOptions }).run(name, rule, tests);
  }

  function runTSRuleTester(name: string, rule: Rule, tests: TestCases) {
    const languageOptions: LanguageOptions = {
      ecmaVersion: 2022,
      sourceType: "module",
      parser: tsEslintParser,
    };

    return new RuleTester({ languageOptions }).run(name, rule, tests);
  }

  const test = (input: string) => `test('test', async () => { ${input} })`;

  return { runRuleTester, runTSRuleTester, test };
}
