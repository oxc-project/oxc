import fs from "node:fs";
import { join as pathJoin } from "node:path";
import { fileURLToPath } from "node:url";
import { RuleTester } from "../rule_tester.ts";

import type { MockFn, TestGroup } from "../index.ts";
import type { InvalidTestCase, TestCase, TestCases } from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

const SNAPSHOTS_DIR = pathJoin(
  fileURLToPath(import.meta.url),
  "../../../submodules/regexp/tests/lib/rules/__snapshots__",
);

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

  shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
    // Skip test cases which start with `/* exported */` comment.
    // Oxlint does not support defining globals inline.
    if (code.match(/^\s*\/\*\s*exported\s/)) return true;

    // Skip test cases which include `// eslint-disable` or `/* eslint-disable` comments.
    // These are not handled by `RuleTester`.
    if (code.match(/\/[/*]\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

    // Skip stand-alone tests that don't use `RuleTester` in test file for `no-useless-flag` rule
    if (
      ruleName === "Don't conflict even if using the rules together." &&
      err.message === "Test case was not run with `RuleTester`"
    ) {
      return true;
    }

    return false;
  },

  ruleTesters: [],

  parsers: [],
};

export default group;

/**
 * Shim for `SnapshotRuleTester`.
 *
 * `SnapshotRuleTester` allows invalid test cases to be plain strings
 * and doesn't require `errors` or `output` properties (both are validated via snapshots).
 * This shim normalizes test cases to the format the conformance `RuleTester` expects:
 * - Converts string invalid test cases to `{ code: string }` objects.
 * - Sets `errors` to `"__unknown__"` when not specified, which tells the conformance
 *   `RuleTester` to skip error count/content validation.
 * - Reads `.eslintsnap` snapshot files to inject the expected `output` for each invalid
 *   test case, enabling proper fix output validation.
 */
class SnapshotRuleTesterShim extends RuleTester {
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    // Parse expected outputs from the snapshot file for this rule
    const snapshotOutputs = parseSnapshotOutputs(ruleName);

    tests = {
      valid: tests.valid,
      invalid: tests.invalid.map((test, index) => {
        if (typeof test === "string") {
          test = { code: test } as unknown as InvalidTestCase;
        }

        // Set errors to __unknown__ if not specified.
        // `SnapshotRuleTester` validates errors via snapshots, not inline.
        if (test.errors == null) {
          (test.errors as unknown as string) = "__unknown__";
        }

        // Inject expected output from snapshot if available and not already specified.
        // `null` means "Output: unchanged" (no fix expected).
        // A string means the expected code after fixing.
        if (!Object.hasOwn(test, "output") && snapshotOutputs != null) {
          const output = snapshotOutputs.get(index);
          if (output !== undefined) {
            test.output = output;
          }
        }

        // Enable recursive fixing to match `SnapshotRuleTester`'s behavior.
        // This is needed for rules whose fixes cascade (e.g. removing duplicates, then simplifying subsets).
        if (test.recursive == null) test.recursive = true;

        return test;
      }),
    };

    super.run(ruleName, rule, tests);
  }
}

/**
 * Parse an `.eslintsnap` snapshot file to extract expected fix output for each invalid test case.
 *
 * The eslintsnap format (v1) looks like:
 * ```
 * Test: rule-name >> invalid
 * Code:
 *   1 | /\x00/
 *     |  ^~~~ [1]
 *
 * Output:
 *   1 | /\0/
 *
 * [1] Error message.
 * ---
 * ```
 *
 * `Output: unchanged` means the rule doesn't produce a fix.
 *
 * @param ruleName - Name of the rule (used to find the snapshot file)
 * @returns Map from invalid test case index to expected output (string or `null` for unchanged),
 *          or `null` if the snapshot file doesn't exist.
 */
function parseSnapshotOutputs(ruleName: string): Map<number, string | null> | null {
  const snapshotPath = pathJoin(SNAPSHOTS_DIR, `${ruleName}.ts.eslintsnap`);

  let content: string;
  try {
    content = fs.readFileSync(snapshotPath, "utf8");
  } catch {
    return null;
  }

  const results = new Map<number, string | null>();
  const lines = content.split("\n");
  let invalidIndex = 0;
  let i = 0;

  while (i < lines.length) {
    if (lines[i].startsWith("Test:") && lines[i].includes(">> invalid")) {
      // Scan forward for the Output section of this test case
      while (i < lines.length && lines[i] !== "---") {
        if (lines[i] === "Output: unchanged") {
          results.set(invalidIndex, null);
          break;
        }

        if (lines[i] === "Output:") {
          i++;
          const outputLines: string[] = [];
          // Collect lines matching the `  <linenum> | <content>` format
          while (i < lines.length && /^\s+\d+\s+\|/.test(lines[i])) {
            const match = lines[i].match(/^\s+\d+\s+\|\s?(.*)/);
            outputLines.push(match ? match[1] : "");
            i++;
          }
          results.set(invalidIndex, outputLines.join("\n"));
          i--; // Compensate for outer i++
          break;
        }

        i++;
      }

      invalidIndex++;
    }

    i++;
  }

  return results;
}
