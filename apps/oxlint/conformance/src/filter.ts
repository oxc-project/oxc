// Options to filter tests to a specific rule or specific code.
// Useful for debugging a particular test case.
type Filter = string | string[];

export const FILTER_ONLY_RULE: Filter | null = null;
export const FILTER_ONLY_CODE: Filter | null = null;
