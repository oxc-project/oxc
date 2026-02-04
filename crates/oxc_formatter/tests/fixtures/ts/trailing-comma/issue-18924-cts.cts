// Single type parameter in arrow function requires trailing comma in .cts files
const fn = <T,>() => {};

// With constraint - no trailing comma needed
const fn2 = <T extends string>() => {};

// Multiple type parameters - no special handling needed
const fn3 = <T, U>() => {};
