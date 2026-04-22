/*
 * Test filtering.
 *
 * Alter the constants below (`FILTER_ONLY_GROUP` etc) to apply a filter to tests.
 * These should only be used temporarily for debugging a certain test case.
 * In committed code, these should all be `null`.
 */

type Filter = string | string[];
type ShouldSkipFn = (name: string) => boolean;

// Options to filter tests to a specific group, rule, or specific code.
// Useful for debugging a particular test case.
const FILTER_ONLY_GROUP: Filter | null = null;
const FILTER_ONLY_RULE: Filter | null = null;
const FILTER_ONLY_CODE: Filter | null = null;

// Filtering functions, generated from the above constants
export const SHOULD_SKIP_GROUP = createShouldSkipFn(FILTER_ONLY_GROUP);
export const SHOULD_SKIP_RULE = createShouldSkipFn(FILTER_ONLY_RULE);
export const SHOULD_SKIP_CODE = createShouldSkipFn(FILTER_ONLY_CODE);

/**
 * Create a function which determines if should skip based on a filter.
 * @param filter - Filter
 * @returns Filter function
 */
function createShouldSkipFn(filter: Filter | null): ShouldSkipFn {
  if (filter === null) return returnFalse;
  if (Array.isArray(filter)) return (name) => !filter.includes(name);
  return (name) => name !== filter;
}

function returnFalse(): boolean {
  return false;
}
