// This file is for testing formatters against invalid disable directives.

// this one should be fine:
// oxlint-disable-next-line eslint/no-debugger
debugger;

// These three should result in diagnostics as they are all unused:

// eslint-disable-next-line no-debugger
const foo = 3;

// oxlint-disable-next-line no-debugger
const bar = 3;

// oxlint-disable-next-line eslint/no-debugger
const baz = 3;

console.log(foo, bar, baz);
