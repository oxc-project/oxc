/*
 * `describe` and `it` functions for capturing test results.
 */

import type { TestGroup } from "./index.ts";
import type { TestCase } from "./rule_tester.ts";

/**
 * Result of running all tests in a rule file.
 */
export interface RuleResult {
  ruleName: string;
  isLoadError: boolean;
  loadError: Error | null;
  tests: TestResult[];
}

/**
 * Result of running a single test.
 */
export interface TestResult {
  groupName: string;
  code: string;
  isPassed: boolean;
  isSkipped: boolean;
  error: Error | null;
  testCase: TestCase | null;
}

// Tracks what nested `describe` blocks currently in
const describeStack: string[] = [];

// Current test group
export let currentGroup: TestGroup | null = null;

/**
 * Set the current group being tested.
 * Call before starting running tests for a test group.
 * @param group - `TestGroup` object
 */
export function setCurrentGroup(group: TestGroup): void {
  currentGroup = group;
}

// Current rule being tested
export let currentRule: RuleResult | null = null;

/**
 * Set the current rule being tested.
 * Call before loading a file containing tests.
 * @param rule - `RuleResult` object
 */
export function setCurrentRule(rule: RuleResult): void {
  currentRule = rule;
}

/**
 * Reset the current rule being tested.
 * Call after loading a file containing tests.
 */
export function resetCurrentRule(): void {
  currentRule = null;
}

// Current test case being tested
let currentTest: TestCase | null = null;

/**
 * Set the current test being tested.
 * Call before running linter for a test case (in `modifyTestCase` hook).
 * @param test - `TestCase` object
 */
export function setCurrentTest(test: TestCase): void {
  currentTest = test;
}

/**
 * `describe` function that tracks the test hierarchy.
 * @param name - Name of the test group
 * @param fn - Function to run tests in the group
 */
export function describe(name: string, fn: () => void): void {
  describeStack.push(name);
  try {
    const res = fn() as any;

    // If returned a promise, ignore the promise's rejection, and create a test case which throws an error,
    // so it appears in the snapshot. This can only happen if `describe` is used manually.
    if (res instanceof Promise) {
      res.catch(() => {});
      throw new Error("Test case returned a promise");
    }

    describeStack.pop();
  } catch (err) {
    // Error. Treat it as a test case (`it`), so it ends up in snapshot and doesn't cause the file to fail to load.
    // This is useful for test files which use `describe` and `it` manually.
    describeStack.pop();
    it(name, () => {
      throw err;
    });
  }
}

(globalThis as any).describe = describe;

/**
 * `it` function that runs and records individual tests.
 * @param name - Name of the test
 * @param fn - Function to run test
 */
export function it(code: string, fn: () => void): void {
  const testResult: TestResult = {
    groupName: describeStack.join(" > "),
    code,
    isPassed: false,
    isSkipped: false,
    error: null,
    testCase: null,
  };

  try {
    fn();

    // Check that the test case was actually run
    if (currentTest === null) throw new Error("Test case was not run with `RuleTester`");

    testResult.isPassed = true;
  } catch (err) {
    if (currentTest === null) {
      // oxlint-disable-next-line no-ex-assign
      err = new Error("Test case was not run with `RuleTester`");
      currentTest = { code };
    } else if (!(err instanceof Error)) {
      // oxlint-disable-next-line no-ex-assign
      err = new Error("Unknown error");
    }

    testResult.testCase = currentTest;
    testResult.error = err;

    const ruleName = describeStack[0];
    if (shouldSkipTest(ruleName, currentTest, code, err)) testResult.isSkipped = true;
  } finally {
    // Reset current test
    currentTest = null;
  }

  currentRule!.tests.push(testResult);
}

(globalThis as any).it = it;

/**
 * Determine if failing test case should be skipped.
 * @param ruleName - Rule name
 * @param test - Test case
 * @param code - Code for test case
 * @param err - Error thrown during test case
 * @returns `true` if test should be skipped
 */
function shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
  // We cannot support custom parsers
  if (err.message === "Custom parsers are not supported") return true;

  // Defer to `TestGroup`'s `shouldSkipTest` method to determine if test should be skipped
  const { shouldSkipTest } = currentGroup!;
  if (shouldSkipTest != null) return shouldSkipTest(ruleName, test, code, err);

  return false;
}

// Add `it.only` property for compatibility.
// `it.only` behaves the same as `it`.
it.only = (name: string, fn: () => void): void => it(name, fn);
