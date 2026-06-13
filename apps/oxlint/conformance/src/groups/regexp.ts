import fs from "node:fs";
import { join as pathJoin } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert";
import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";
import type {
  InvalidTestCase,
  TestCase,
  TestCases,
  Error,
  ErrorSuggestion,
} from "../rule_tester.ts";
import type { Rule } from "#oxlint/plugins";

/**
 * Test case details parsed from snapshot file.
 */
interface SnapshotCase {
  code: string;
  output: string | null;
  errors: (Error | string)[];
}

const SNAPSHOTS_DIR = pathJoin(
  fileURLToPath(import.meta.url),
  "../../../submodules/regexp/tests/lib/rules/__snapshots__",
);

const group: TestGroup = {
  name: "regexp",
  ...repos.regexp,

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
 * Replication of `eslint-snapshot-rule-tester`'s `SnapshotRuleTester`.
 *
 * `SnapshotRuleTester` allows invalid test cases to be plain strings and gets `output` and `errors`
 * from snapshot files instead of them being specified in the test case.
 *
 * This RuleTester class loads snapshot files from the `__snapshots__` directory and extracts `output` and `errors`
 * from snapshots. It adds these properties to invalid test cases, allowing them to be validated.
 */
class SnapshotRuleTesterShim extends RuleTester {
  run(ruleName: string, rule: Rule, tests: TestCases): void {
    // Parse expected outputs from the snapshot file for this rule
    const snapshotCases = parseSnapshot(ruleName);

    assert.equal(
      tests.invalid.length,
      snapshotCases.length,
      "Snapshot file has wrong number of invalid test cases",
    );

    tests = {
      valid: tests.valid,
      invalid: tests.invalid.map((test, index) => {
        if (typeof test === "string") {
          test = { code: test } as unknown as InvalidTestCase;
        }

        // Add `output` and `errors` from snapshot if not specified in test case
        const snapshotCase = snapshotCases[index];
        assert.equal(snapshotCase.code, test.code, "Code in snapshot does not match test case");

        if (test.output === undefined) {
          test.output = snapshotCase.output;
        } else {
          assert.equal(
            test.output,
            snapshotCase.output,
            "Output in snapshot does not match test case",
          );
        }

        if (test.errors === undefined) {
          test.errors = snapshotCase.errors;
        } else {
          assert.deepStrictEqual(
            test.errors,
            snapshotCase.errors,
            "Errors in snapshot do not match test case",
          );
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
 * @param ruleName - Name of the rule
 * @returns Array of test case snapshot details
 */
function parseSnapshot(ruleName: string): SnapshotCase[] {
  // Read snapshot file
  const snapshotPath = pathJoin(SNAPSHOTS_DIR, `${ruleName}.ts.eslintsnap`);

  let content: string;
  try {
    content = fs.readFileSync(snapshotPath, "utf8");
  } catch {
    return [];
  }

  // Strip header and footer
  const HEADER = "# eslint-snapshot-rule-tester format: v1\n";
  assert(content.startsWith(HEADER), "Missing header");
  content = content.slice(HEADER.length).trim();

  if (content === "") return [];

  const FOOTER = "\n---";
  assert(content.endsWith(FOOTER), "Missing footer");
  content = content.slice(0, -FOOTER.length);

  // Parse test cases from snapshot
  return content.split("\n---\n").map((caseText) => {
    const lines = caseText.trimStart().split("\n");

    // Parse header.
    // `Test: rule-name >> invalid` or
    // `Test: rule-name >> invalid >>> ...`
    const firstLineExpected = `Test: ${ruleName} >> invalid`;
    const firstLine = lines[0];
    assert(
      firstLine === firstLineExpected ||
        (firstLine.startsWith(firstLineExpected) &&
          firstLine.slice(firstLineExpected.length).startsWith(" >>> ")),
      "Invalid header",
    );

    // Skip down to code section
    let lineIndex = 1;
    for (; lineIndex < lines.length; lineIndex++) {
      if (lines[lineIndex] === "Code:") break;
    }
    lineIndex++;
    assert(lineIndex < lines.length, "Missing code section");

    // Parse source code.
    // ```
    // Code:
    //   1 | /\x00/
    //     |  ^~~~ [1]
    // ```
    // Can be multiple lines.
    function parseCode(shouldAcceptErrorLines: boolean): string {
      let code = "",
        codeLineNum = 1;
      for (; lineIndex < lines.length; lineIndex++) {
        const match = lines[lineIndex].match(/^\s+(?:(\d+) )?\|(?: (.*))?$/);
        if (!match) break;

        const [, lineNumber, codeLine] = match;
        if (lineNumber === undefined) {
          assert(shouldAcceptErrorLines, "Unexpected error line");
        } else {
          assert.equal(lineNumber, codeLineNum.toString(), "Misnumbered line in code extract");
          if (codeLineNum > 1) code += "\n";
          if (codeLine !== undefined) code += codeLine;
          codeLineNum++;
        }
      }
      return code;
    }

    const code = parseCode(true);

    assert.equal(lines[lineIndex], "", "Expected empty line after code");
    lineIndex++;

    // Parse output.
    // ```
    // Output:
    //   1 | /\0/
    // ```
    // Can be multiple lines.
    let output: string | null = null;
    if (lines[lineIndex] === "Output:") {
      lineIndex++;
      output = parseCode(false);

      assert.equal(lines[lineIndex], "", "Expected empty line after output");
      lineIndex++;
    } else if (lines[lineIndex] === "Output: unchanged") {
      assert.equal(lines[lineIndex + 1], "", "Expected empty line after output");
      lineIndex += 2;
    }

    // Parse errors and their suggestions.
    // ```
    // [1] Unexpected control character escape '\x00' (U+0000). Use '\0' instead.
    // "[2] Don't abuse reptiles under any circumstances."
    // [3] Unexpected gonads on railway track.
    //     Suggestions:
    //       - Remove the gonads.
    //         Output:
    //           1 | const railway = "no gonads here";
    // ```
    const errors: (Error | string)[] = [];
    while (lineIndex < lines.length) {
      // Parse error message
      const match = lines[lineIndex].match(/^(")?\[(\d+)\] (.*?)(")?$/)!;
      assert(match !== null, "Invalid error message");
      lineIndex++;

      // Decode error message if line wrapped in quotes
      let [, quoteStart, errorNumber, message, quoteEnd] = match;
      assert.equal(errorNumber, (errors.length + 1).toString(), "Misnumbered error number");
      if (quoteStart === '"') {
        assert.equal(quoteEnd, '"', "Mismatched quotes");
        message = JSON.parse(`"${message}"`);
      }

      // Pass suggestions for error
      if (/^\s+Suggestions:/.test(lines[lineIndex])) {
        lineIndex++;

        const suggestions: ErrorSuggestion[] = [];

        while (lineIndex < lines.length) {
          const match = lines[lineIndex].match(/^\s+- (.*)$/)!;
          if (!match) break;

          const desc = match[1];
          lineIndex++;

          assert(/^\s+Output:$/.test(lines[lineIndex]), "Expected output section for suggestion");
          lineIndex++;

          const output = parseCode(false);
          suggestions.push({ desc, output });
        }

        errors.push({ message, suggestions });
      } else {
        errors.push(message);
      }
    }

    return { code, output, errors };
  });
}
