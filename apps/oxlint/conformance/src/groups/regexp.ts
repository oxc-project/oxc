import { RuleTester } from "../rule_tester.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { InvalidTestCase, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

const group: TestGroup = {
  name: "regexp",

  submoduleName: "regexp",
  testFilesDirPath: "tests/lib/rules",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".ts")) return null;
    return filename.slice(0, -".ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Mock `eslint-snapshot-rule-tester` to use conformance `RuleTester`.
    // The plugin's tests import `SnapshotRuleTester` from this package,
    // which is a `RuleTester` subclass that records snapshots.
    // We replace it with a shim that normalizes test cases for the conformance `RuleTester`.
    mock("eslint-snapshot-rule-tester", { SnapshotRuleTester: SnapshotRuleTesterShim });
  },

  ruleTesters: [],

  parsers: [],
};

export default group;

/**
 * Shim for `SnapshotRuleTester`.
 *
 * `SnapshotRuleTester` allows invalid test cases to be plain strings
 * and doesn't require an `errors` property (errors are validated via snapshots).
 * This shim normalizes test cases to the format the conformance `RuleTester` expects:
 * - Converts string invalid test cases to `{ code: string }` objects.
 * - Sets `errors` to `"__unknown__"` when not specified, which tells the conformance
 *   `RuleTester` to skip error count/content validation.
 */
class SnapshotRuleTesterShim extends RuleTester {
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    tests = {
      valid: tests.valid,
      invalid: tests.invalid.map((test) => {
        if (typeof test === "string") test = { code: test } as InvalidTestCase;
        if (test.errors == null) {
          (test.errors as unknown as string) = "__unknown__";
        }
        return test;
      }),
    };

    super.run(ruleName, rule, tests);
  }
}
