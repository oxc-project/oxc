// Test unused disable directives with type-aware rules
const myPromise = new Promise(() => {})

// This disable directive is USED (suppresses the error below)
// oxlint-disable-next-line typescript/no-floating-promises
myPromise

// This disable directive is UNUSED (no error to suppress)
// oxlint-disable-next-line typescript/no-floating-promises
console.log("hello")

// This disable directive is UNUSED (wrong rule name)
// oxlint-disable-next-line typescript/no-unsafe-assignment
myPromise

// This disable directive is USED
/* eslint-disable @typescript-eslint/no-floating-promises */
myPromise
/* eslint-enable @typescript-eslint/no-floating-promises */

// This disable directive is UNUSED (no floating promise here)
/* eslint-disable @typescript-eslint/no-floating-promises */
const x = 1 + 2
/* eslint-enable @typescript-eslint/no-floating-promises */
