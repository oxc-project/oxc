/*
 * Function to generate a report of test results as markdown.
 */

import { join as pathJoin, sep as pathSep } from "node:path";
import { pathToFileURL } from "node:url";
import { CONFORMANCE_DIR_PATH } from "./run.ts";

import type { RuleResult, TestResult } from "./capture.ts";
import type { TestCase } from "./rule_tester.ts";

// Number of lines of stack trace to show in report for each error
const STACK_TRACE_LINES = 4;

const ROOT_DIR_PATH = pathJoin(CONFORMANCE_DIR_PATH, "../../../");
const ROOT_DIR_URL = pathToFileURL(ROOT_DIR_PATH).href;
const DIST_DIR_SUBPATH = "apps/oxlint/dist";

// Replace backslashes with forward slashes on Windows. Do nothing on Mac/Linux.
const normalizeSlashes =
  pathSep === "\\" ? (path: string) => path.replaceAll("\\", "/") : (path: string) => path;

/**
 * Generate report of test results as markdown.
 * @param results - Results of running tests
 * @returns Report as markdown
 */
export function generateReport(results: RuleResult[]): string {
  // Categorize rules
  const loadErrorRules: RuleResult[] = [],
    noTestRules: RuleResult[] = [],
    passingRules: { ruleName: string; testCount: number; skippedCount: number }[] = [],
    failingRules: {
      ruleName: string;
      testCount: number;
      skippedCount: number;
      failingTests: TestResult[];
    }[] = [];
  let fullyFailingRuleCount = 0,
    totalTestCount = 0,
    failingTestCount = 0,
    skippedTestCount = 0;

  for (const result of results) {
    if (result.isLoadError) {
      loadErrorRules.push(result);
      continue;
    }

    const { ruleName, tests } = result,
      testCount = tests.length;
    if (testCount === 0) {
      noTestRules.push(result);
      continue;
    }

    totalTestCount += testCount;

    const failingTests = [];
    let ruleSkippedCount = 0;
    for (const test of tests) {
      if (test.isPassed) continue;
      if (test.isSkipped) {
        ruleSkippedCount++;
      } else {
        failingTests.push(test);
      }
    }

    skippedTestCount += ruleSkippedCount;

    if (failingTests.length === 0) {
      passingRules.push({ ruleName, testCount, skippedCount: ruleSkippedCount });
      continue;
    }

    failingTestCount += failingTests.length;

    if (failingTests.length === tests.length) {
      fullyFailingRuleCount++;
    }

    failingRules.push({
      ruleName,
      testCount,
      skippedCount: ruleSkippedCount,
      failingTests,
    });
  }

  let report = "";

  function line(str: string): void {
    report += str;
    report += "\n";
  }

  function lineBreak(): void {
    report += "\n";
  }

  function block(str: string): void {
    report += str
      .trimStart()
      .split("\n")
      .map((line) => line.trimStart())
      .join("\n");
  }

  // Summary statistics
  const totalRuleCount = results.length,
    loadErrorRuleCount = loadErrorRules.length,
    fullyPassingRuleCount = passingRules.length,
    partiallyPassingRuleCount = failingRules.length - fullyFailingRuleCount,
    passingTestCount = totalTestCount - failingTestCount - skippedTestCount,
    noTestsRuleCount = noTestRules.length;

  function countAndPercent(count: number, total: number): string {
    return `${String(count).padStart(5)} | ${formatPercent(count, total).padStart(6)}`;
  }

  block(`
    # ESLint Rule Tester Conformance Results

    ## Summary

    ### Rules

    | Status            | Count | %      |
    | ----------------- | ----- | ------ |
    | Total rules       | ${countAndPercent(totalRuleCount, totalRuleCount)} |
    | Fully passing     | ${countAndPercent(fullyPassingRuleCount, totalRuleCount)} |
    | Partially passing | ${countAndPercent(partiallyPassingRuleCount, totalRuleCount)} |
    | Fully failing     | ${countAndPercent(fullyFailingRuleCount, totalRuleCount)} |
    | Load errors       | ${countAndPercent(loadErrorRuleCount, totalRuleCount)} |
    | No tests run      | ${countAndPercent(noTestsRuleCount, totalRuleCount)} |

    ### Tests

    | Status      | Count | %      |
    | ----------- | ----- | ------ |
    | Total tests | ${countAndPercent(totalTestCount, totalTestCount)} |
    | Passing     | ${countAndPercent(passingTestCount, totalTestCount)} |
    | Failing     | ${countAndPercent(failingTestCount, totalTestCount)} |
    | Skipped     | ${countAndPercent(skippedTestCount, totalTestCount)} |

  `);

  // Fully passing rules
  line("## Fully Passing Rules\n");
  if (passingRules.length === 0) {
    line("No rules fully passing");
  } else {
    for (const rule of passingRules) {
      let out = `- \`${rule.ruleName}\` (${rule.testCount} tests)`;
      if (rule.skippedCount > 0) out += ` (${rule.skippedCount} skipped)`;
      line(out);
    }
  }
  lineBreak();

  // Rules with failures
  report += "## Rules with Failures\n\n";
  if (failingRules.length === 0) {
    line("No rules with failures\n");
  } else {
    // Summary
    for (const rule of failingRules) {
      const { testCount } = rule,
        passedCount = testCount - rule.failingTests.length;
      line(`- \`${rule.ruleName}\` - ${formatProportion(passedCount, testCount)}`);
    }
    lineBreak();

    // Details
    line("## Rules with Failures Detail\n");

    for (const rule of failingRules) {
      const { testCount, failingTests, skippedCount } = rule,
        failedCount = failingTests.length,
        passedCount = testCount - failedCount - skippedCount;

      block(`
        ### \`${rule.ruleName}\`

        Pass: ${formatProportion(passedCount, testCount)}
        Fail: ${formatProportion(failedCount, testCount)}
        Skip: ${formatProportion(skippedCount, testCount)}

      `);

      // List failed tests
      for (const test of failingTests) {
        line(`#### ${test.groupName}\n`);
        line("```js");
        line(test.code);
        line("```\n");

        const testCaseStr = formatTestCase(test.testCase, test.code);
        if (testCaseStr !== null) {
          line("```json");
          line(testCaseStr);
          line("```\n");
        }

        line(formatError(test.error));
        lineBreak();
      }
    }
  }

  // Load errors
  if (loadErrorRules.length > 0) {
    line("## Load Errors\n");
    for (const rule of loadErrorRules) {
      line(`### \`${rule.ruleName}\`\n`);
      line(formatError(rule.loadError));
      lineBreak();
    }
    lineBreak();
  }

  // Rules with no tests
  if (noTestRules.length > 0) {
    line("## Rules with no tests run\n");
    for (const rule of noTestRules) {
      line(`- \`${rule.ruleName}\``);
    }
    lineBreak();
  }

  return report.slice(0, -1);
}

// Regex to match ANSI escape sequences (colors, formatting, etc.)
// Matches ESC[ followed by parameters and command letter
// eslint-disable-next-line no-control-regex
const ANSI_ESCAPE_REGEX = /\x1B\[[0-9;]*[a-zA-Z]/gu;

// Regex to match other control characters (except tab, newline, carriage return)
// eslint-disable-next-line no-control-regex
const CONTROL_CHAR_REGEX = /[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/gu;

/**
 * Clean a string for output.
 * Remove ANSI escape sequences and control characters from a string.
 * Keeps tab, newline, and carriage return.
 * @param str - String to clean
 * @returns Cleaned string
 */
function cleanString(str: string): string {
  return str.replace(ANSI_ESCAPE_REGEX, "").replace(CONTROL_CHAR_REGEX, "");
}

const STACK_LINE_REGEX = /^    at ([^ ]+ \()(.+)(:\d+:\d+)\)$/u;
const STACK_LINE_REGEX2 = /^    at (.+)(:\d+:\d+)$/u;

/**
 * Format an error for markdown output.
 * Includes error message followed by stack trace lines.
 * Number of lines of stack trace is limited to max `STACK_TRACE_LINES`.
 *
 * @param err - Error to format
 * @returns Error formatted as string
 */
function formatError(err: Error | null): string {
  if (!err) return "Unknown error";

  const stack = err.stack || "";
  const lines = stack.split("\n");

  // Print error message
  let out = "",
    lineIndex = 0;
  for (; lineIndex < lines.length; lineIndex++) {
    const line = lines[lineIndex];
    if (line.startsWith("    at ")) break;
    // These `      ^` diff marker lines appear to be produced by `AssertionError` non-deterministically.
    // This appears to be a bug in NodeJS's `assert` module. Remove them, to avoid churn in snapshot.
    if (line.trimStart() === "^") continue;
    out += `${cleanString(line)}\n`;
  }

  // Print stack trace.
  // Limit to first `STACK_TRACE_LINES` lines.
  // Convert paths to relative to repo root, and forward slashes on Windows.
  // For files in `apps/oxlint/dist`, remove line column - it produces churn in report when Oxc's code is altered.
  for (let i = 0; i < STACK_TRACE_LINES && lineIndex < lines.length; i++) {
    const line = lines[lineIndex];

    let prefix, path, lineCol, postfix;
    let match = line.match(STACK_LINE_REGEX);
    if (match) {
      [, prefix, path, lineCol] = match;
      postfix = ")";
    } else {
      match = line.match(STACK_LINE_REGEX2);
      if (!match) break;
      [, path, lineCol] = match;
      prefix = "";
      postfix = "";
    }

    if (path.startsWith("file://")) {
      if (path.startsWith(ROOT_DIR_URL)) {
        path = path.slice(ROOT_DIR_URL.length);
        if (path.startsWith(DIST_DIR_SUBPATH)) lineCol = "";
      }
    } else {
      if (path.startsWith(ROOT_DIR_PATH)) {
        path = normalizeSlashes(path.slice(ROOT_DIR_PATH.length));
        if (path.startsWith(DIST_DIR_SUBPATH)) lineCol = "";
      } else {
        path = normalizeSlashes(path);
      }
    }

    out += `    at ${prefix}${path}${lineCol}${postfix}\n`;
    lineIndex++;
  }

  return out;
}

/**
 * Format a test case as JSON.
 * @param testCase - Test case to format
 * @returns Test case formatted as JSON string, or `null` if not present, or could not format
 */
function formatTestCase(testCase: TestCase | null, code: string): string | null {
  if (!testCase) return null;

  testCase = { ...testCase };

  // Remove `eslintCompat` option - it's always `true`
  testCase.eslintCompat = undefined;

  // Remove `code` property if it's the same as the test case's code
  if (testCase.code === code) (testCase as { code?: string }).code = undefined;

  try {
    return JSON.stringify(testCase, null, 2);
  } catch {
    return null;
  }
}

/**
 * Produce a string of the form `count / total (percent%)`.
 * @param count - Count
 * @param total - Total count
 * @returns Formatted proportion
 */
function formatProportion(count: number, total: number): string {
  return `${count} / ${total} (${formatPercent(count, total)})`;
}

/**
 * Produce a string representing `count / total` as a percentage.
 * @param count - Count
 * @param total - Total count
 * @returns Formatted percent
 */
function formatPercent(count: number, total: number): string {
  return `${((count / total) * 100).toFixed(1)}%`;
}
