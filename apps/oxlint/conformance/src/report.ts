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
    passingRules: RuleResult[] = [],
    failingRules: { ruleName: string; testCount: number; failingTests: TestResult[] }[] = [];
  let fullyFailingRuleCount = 0,
    totalTestCount = 0,
    failingTestCount = 0;

  for (const result of results) {
    if (result.isLoadError) {
      loadErrorRules.push(result);
      continue;
    }

    const { tests } = result,
      testCount = tests.length;
    if (testCount === 0) {
      noTestRules.push(result);
      continue;
    }

    totalTestCount += testCount;

    const failingTests = tests.filter((test) => !test.isPassed);
    if (failingTests.length === 0) {
      passingRules.push(result);
      continue;
    }

    failingTestCount += failingTests.length;

    if (failingTests.length === tests.length) {
      fullyFailingRuleCount++;
    }

    failingRules.push({
      ruleName: result.ruleName,
      testCount,
      failingTests,
    });
  }

  // Header
  const lines: string[] = [];
  lines.push("# ESLint Rule Tester Conformance Results");
  lines.push("");

  // Summary statistics
  const totalRuleCount = results.length,
    loadErrorRuleCount = loadErrorRules.length,
    fullyPassingRuleCount = passingRules.length,
    partiallyPassingRuleCount = failingRules.length - fullyFailingRuleCount,
    passingTestCount = totalTestCount - failingTestCount,
    noTestsRuleCount = noTestRules.length;

  lines.push("## Summary");
  lines.push("");
  lines.push("### Rules");
  lines.push("");
  lines.push(`| Status            | Count |`);
  lines.push(`| ----------------- | ----- |`);
  lines.push(`| Total rules       | ${String(totalRuleCount).padStart(5)} |`);
  lines.push(`| Fully passing     | ${String(fullyPassingRuleCount).padStart(5)} |`);
  lines.push(`| Partially passing | ${String(partiallyPassingRuleCount).padStart(5)} |`);
  lines.push(`| Fully failing     | ${String(fullyFailingRuleCount).padStart(5)} |`);
  lines.push(`| Load errors       | ${String(loadErrorRuleCount).padStart(5)} |`);
  lines.push(`| No tests run      | ${String(noTestsRuleCount).padStart(5)} |`);
  lines.push("");

  lines.push("### Tests");
  lines.push("");
  lines.push(`| Status      | Count |`);
  lines.push(`| ----------- | ----- |`);
  lines.push(`| Total tests | ${String(totalTestCount).padStart(5)} |`);
  lines.push(`| Passing     | ${String(passingTestCount).padStart(5)} |`);
  lines.push(`| Failing     | ${String(failingTestCount).padStart(5)} |`);
  lines.push("");

  // Fully passing rules
  lines.push("## Fully Passing Rules");
  lines.push("");
  if (passingRules.length === 0) {
    lines.push("No rules fully passing");
  } else {
    for (const rule of passingRules) {
      lines.push(`- \`${rule.ruleName}\` (${rule.tests.length} tests)`);
    }
  }
  lines.push("");

  // Rules with failures
  lines.push("## Rules with Failures");
  lines.push("");
  if (failingRules.length === 0) {
    lines.push("No rules with failures");
  } else {
    // Summary
    for (const rule of failingRules) {
      const { testCount } = rule,
        passedCount = testCount - rule.failingTests.length;
      lines.push(`- \`${rule.ruleName}\` - ${formatProportion(passedCount, testCount)}`);
    }
    lines.push("");

    // Details
    lines.push("## Rules with Failures Detail");
    lines.push("");

    for (const rule of failingRules) {
      const { testCount, failingTests } = rule,
        failedCount = failingTests.length,
        passedCount = testCount - failedCount;

      lines.push(`### \`${rule.ruleName}\``);
      lines.push("");
      lines.push(`Pass: ${formatProportion(passedCount, testCount)}`);
      lines.push(`Fail: ${formatProportion(failedCount, testCount)}`);
      lines.push("");

      // List failed tests
      for (const test of failingTests) {
        lines.push(`#### ${test.groupName}`);
        lines.push("");

        lines.push("```js");
        lines.push(test.code);
        lines.push("```");
        lines.push("");

        const testCaseStr = formatTestCase(test.testCase, test.code);
        if (testCaseStr !== null) {
          lines.push("```json");
          lines.push(testCaseStr);
          lines.push("```");
          lines.push("");
        }

        lines.push(formatError(test.error));
        lines.push("");
      }
    }
  }

  // Load errors
  if (loadErrorRules.length > 0) {
    lines.push("## Load Errors");
    lines.push("");

    for (const rule of loadErrorRules) {
      lines.push(`### \`${rule.ruleName}\``);
      lines.push("");
      lines.push(formatError(rule.loadError));
      lines.push("");
    }

    lines.push("");
  }

  // Rules with no tests
  if (noTestRules.length > 0) {
    lines.push("## Rules with no tests run");
    lines.push("");

    for (const rule of noTestRules) {
      lines.push(`- \`${rule.ruleName}\``);
    }

    lines.push("");
  }

  return lines.join("\n");
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
  const percent = ((count / total) * 100).toFixed(1);
  return `${count} / ${total} (${percent}%)`;
}
