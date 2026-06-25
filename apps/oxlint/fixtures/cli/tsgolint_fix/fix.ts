// This file has a fixable tsgolint error: no-unnecessary-type-assertion
// The type assertion `as string` is unnecessary because str is already a string
const str: string = 'hello';
const redundant = str as string;

export { redundant };
