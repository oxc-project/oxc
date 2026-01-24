// These should NOT error
const a = foo as string;
const f1 = <T,>() => {};  // Trailing comma disambiguates
const f2 = <T extends unknown>() => {};  // Constraint disambiguates
const f3 = <T, U>() => {};  // Multiple params = comma inside disambiguates
const f4 = <T extends unknown = T>() => {};  // Constraint + default
