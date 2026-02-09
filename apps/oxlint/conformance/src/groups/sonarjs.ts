import { createRequire } from "node:module";
import { join as pathJoin } from "node:path";
import { currentRule } from "../capture.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { TestCase, TestCases, ValidTestCase, InvalidTestCase } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

const SUBMODULE_NAME = "sonarjs";
const TEST_FILES_RELATIVE_DIR_PATH = "packages/jsts/src/rules";
const TEST_FILES_DIR_PATH = pathJoin(
  import.meta.dirname,
  "../../submodules",
  SUBMODULE_NAME,
  TEST_FILES_RELATIVE_DIR_PATH,
);

const group: TestGroup = {
  name: "sonarjs",

  submoduleName: SUBMODULE_NAME,
  testFilesDirPath: TEST_FILES_RELATIVE_DIR_PATH,

  transformTestFilename(filename: string) {
    // Each rule has its own directory in `packages/jsts/src/rules`, each dir named `Sxxx` e.g. `S100`.
    // Test files are in those subdirectories.
    // e.g. `packages/jsts/src/rules/S100/unit.test.ts`
    if (!filename.endsWith(".test.ts")) return null;
    const parts = filename.split("/");
    if (parts.length !== 2) return null;

    // Get rule name from `meta.js` file in test file's directory
    const requireFromTest = createRequire(pathJoin(TEST_FILES_DIR_PATH, filename));
    const meta = requireFromTest(`./meta.js`);
    return `${meta.eslintId} (${parts[0]})`;
  },

  prepare(require: NodeJS.Require, _mock: MockFn) {
    // Patch SonarJS's rule tester classes.
    // Internally they use ESLint's `RuleTester`, which is already patched to use conformance `RuleTester`,
    // but we need to apply further patches.

    // Load module, clone and write clone into module cache.
    // Can't mutate the original module object because it's transformed from ESM and is frozen.
    const path = require.resolve("../../tests/tools/testers/rule-tester.ts");
    let testerModule = require(path);
    testerModule = { ...testerModule };
    Object.defineProperty(testerModule, "__esModule", { value: true });
    require.cache[path]!.exports = testerModule;

    // Patch type-aware rule tester, to add `_isTypeAware` property to test cases.
    // This is used to skip test cases which fail, because Oxlint can't support type-aware rules.
    testerModule.RuleTester = class extends testerModule.RuleTester {
      run(name: string, rule: Rule, tests: TestCases): void {
        tests = {
          valid: tests.valid.map((test) => {
            if (typeof test === "string") test = { code: test };
            return { ...test, _isTypeAware: true } as ValidTestCase;
          }),
          invalid: tests.invalid.map(
            (test) => ({ ...test, _isTypeAware: true }) as InvalidTestCase,
          ),
        };
        super.run(name, rule, tests);
      }
    };

    // Patch non-type-aware rule tester, to fix some test cases which contain JSX syntax with a `.js` filename.
    // Change filename to `.jsx` to allow them to be parsed.
    testerModule.NoTypeCheckingRuleTester = class extends testerModule.NoTypeCheckingRuleTester {
      run(name: string, rule: Rule, tests: TestCases): void {
        if (currentRule?.ruleName === "no-deprecated-react (S6957)") {
          tests = {
            valid: tests.valid.map((test) =>
              fixTestIfJsx(typeof test === "string" ? { code: test } : test),
            ),
            invalid: tests.invalid.map(fixTestIfJsx),
          };
        }

        super.run(name, rule, tests);
      }
    };

    // Fix a bug with `tsx`
    const unicornPath = require.resolve("eslint-plugin-unicorn");
    const unicornRequire = createRequire(unicornPath);
    const validatorIdentifier = unicornRequire("@babel/helper-validator-identifier");
    validatorIdentifier.default = validatorIdentifier;
  },

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Skip test cases which include `// eslint-disable` or `/* eslint-disable` comments.
    // These are not handled by `RuleTester`.
    if (code.match(/\/[/*]\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

    // Faulty test cases - valid test cases with `errors` property
    if (ruleName === "S2755" && err.message === "Valid test case must not have `errors` property") {
      return true;
    }

    // Only valid code in ES3, and Oxc parser does not support ES3
    if (
      ruleName === "S1527" &&
      code.includes("var implements;") &&
      (test.languageOptions?.parserOptions as any)?.ecmaVersion === 3 &&
      err.message === "Parsing failed"
    ) {
      return true;
    }

    // Test cases' options contain `RegExp`s, which are not JSON-serializable.
    // This isn't possible in Oxlint, because Oxlint config is JSON.
    if (ruleName === "S7718" && err.message.startsWith("Options validation failed for rule ")) {
      return true;
    }

    // Unit tests which don't use `RuleTester` - not relevant
    if (
      err.message === "Test case was not run with `RuleTester`" &&
      ((ruleName === "S1116" && code === "S1116 handles null nodes") ||
        (ruleName === "S1172" && code === "should handle incomplete AST") ||
        (ruleName === "S6647" && code === "should crash with decorated rule"))
    ) {
      return true;
    }

    // We cannot support type-aware rules
    if ((test as any)._isTypeAware) return true;

    /*
    // "Sections of code should not be commented out" rule uses `context.languageOptions.parser.parse` to parse code.
    // This is not implemented yet. We should be able to pass these tests once we implement it.
    if (ruleName === "S125") {
      return true;
    }
    */

    return false;
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],

  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;

/**
 * If test case contains JSX syntax with a `.js` filename, convert filename to `.jsx` to allow them to be parsed.
 * @param test - Test case
 * @returns - Amended test case
 */
function fixTestIfJsx<T extends TestCase>(test: T): T {
  if (test.code.match(/<[a-zA-Z_$][a-zA-Z0-9_$]*(\s* \/)?>/) && test.filename?.endsWith(".js")) {
    return { ...test, filename: `${test.filename}x` };
  }
  return test;
}
