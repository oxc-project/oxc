// TS7059: Type assertions
const a = <string>foo;
const b = <number><string>foo;

// TS7060: Generic arrow functions (single param, no trailing comma, no constraint)
const f1 = <T>() => {};
const f2 = <T = unknown>() => {};  // Default alone doesn't disambiguate in TypeScript!
