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
    testResult.isPassed = true;
  } catch (err) {
    if (err instanceof Error && err.message === "Custom parsers are not supported") {
      testResult.isSkipped = true;
    }

    testResult.error = err as Error;
    testResult.testCase = err?.__testCase ?? null;
  }

  currentRule!.tests.push(testResult);
}

// Add `it.only` property for compatibility.
// `it.only` behaves the same as `it`.
it.only = (name: string, fn: () => void): void => it(name, fn);
