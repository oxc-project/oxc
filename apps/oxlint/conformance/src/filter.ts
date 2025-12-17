/*
 * Test filtering.
 */

type Filter = string | string[];

// Options to filter tests to a specific rule or specific code.
// Useful for debugging a particular test case.
export const FILTER_ONLY_RULE: Filter | null = null;
export const FILTER_ONLY_CODE: Filter | null = null;

// Filter out rules where test failures are expected
export const FILTER_EXCLUDE_RULE: Filter | null = [
  // Rules which use CFG - which we don't support yet
  "array-callback-return",
  "complexity",
  "consistent-return",
  "constructor-super",
  "getter-return",
  "no-constructor-return",
  "no-fallthrough",
  "no-invalid-this",
  "no-promise-executor-return",
  "no-this-before-super",
  "no-unreachable-loop",
  "no-unreachable",
  "no-useless-assignment",
  "no-useless-return",
  "require-atomic-updates",
];
