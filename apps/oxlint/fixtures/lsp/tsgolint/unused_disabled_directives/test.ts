// Test file for disable directives with type-aware rules
const myPromise = new Promise(() => {})

// Test oxlint-disable-next-line
// oxlint-disable-next-line typescript/no-floating-promises
myPromise

// Test eslint-disable-next-line with @typescript-eslint prefix
// eslint-disable-next-line @typescript-eslint/no-floating-promises
myPromise

// Test oxlint-disable/enable block
/* oxlint-disable typescript/no-floating-promises */
myPromise
myPromise
/* oxlint-enable typescript/no-floating-promises */

// This should still report an error (no disable directive)
myPromise

// Test with different rule name formats
// oxlint-disable-next-line no-floating-promises
myPromise

// eslint-disable-next-line typescript-eslint/no-floating-promises
myPromise

// Test disable-line variant
myPromise // oxlint-disable-line typescript/no-floating-promises

// Multiple promises in one block
/* eslint-disable @typescript-eslint/no-floating-promises */
Promise.resolve(1)
Promise.resolve(2)
Promise.resolve(3)
/* eslint-enable @typescript-eslint/no-floating-promises */

// Should report error
Promise.resolve(4)
