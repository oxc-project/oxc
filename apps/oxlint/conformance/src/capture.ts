/*
 * `describe` and `it` functions for capturing test results.
 */

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

// Current rule being tested
let currentRule: RuleResult | null = null;

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
    fn();
  } finally {
    describeStack.pop();
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
    testResult.testCase = currentTest;

    if (!(err instanceof Error)) {
      testResult.error = new Error("Unknown error");
    } else if (currentTest === null) {
      testResult.error = new Error("Test case was not run with `RuleTester`");
    } else {
      testResult.error = err;

      const ruleName = describeStack[0];
      if (shouldSkipTest(ruleName, currentTest, code, err)) testResult.isSkipped = true;
    }
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

  // Skip test cases which start with `/* global */`, `/* globals */`, `/* exported */`, or `/* eslint */` comments.
  // Oxlint does not support defining globals inline.
  // `RuleTester` does not support enabling other rules beyond the rule under test.
  if (code.match(/^\s*\/\*\s*(globals?|exported|eslint)\s/)) return true;

  // Skip test cases which include `// eslint-disable` comments.
  // These are not handled by `RuleTester`.
  if (code.match(/\/\/\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

  // Tests rely on directives being parsed as plain `StringLiteral`s in ES3.
  // Oxc parser does not support parsing as ES3.
  if (
    (ruleName === "no-eval" ||
      ruleName === "no-invalid-this" ||
      ruleName === "no-unused-expressions") &&
    test.languageOptions?.ecmaVersion === 3
  ) {
    return true;
  }

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
}

// Add `it.only` property for compatibility.
// `it.only` behaves the same as `it`.
it.only = (name: string, fn: () => void): void => it(name, fn);
