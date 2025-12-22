/*
 * Test filtering.
 */

type Filter = string | string[];

// Options to filter tests to a specific rule or specific code.
// Useful for debugging a particular test case.
export const FILTER_ONLY_RULE: Filter | null = null;
export const FILTER_ONLY_CODE: Filter | null = null;

// Filter out rules where test failures are expected
export const FILTER_EXCLUDE_RULE: Filter | null = [];
