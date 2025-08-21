// Test file for tsgolint disable directives functionality

/* eslint-disable typescript-eslint/no-floating-promises */
// This should be suppressed by the block disable directive
Promise.resolve(42);
/* eslint-enable typescript-eslint/no-floating-promises */

// This should trigger a diagnostic (not disabled)
Promise.resolve(84);

// TODO: Fix disable-next-line issue - currently off by one character
// /* eslint-disable-next-line typescript-eslint/no-floating-promises */
// This should be suppressed by the next-line disable directive
// Promise.resolve(168);

// Test with short rule name
/* eslint-disable no-floating-promises */
Promise.resolve(336);
/* eslint-enable no-floating-promises */

// Test with disable-line
Promise.resolve(672); // eslint-disable-line typescript-eslint/no-floating-promises

// This should trigger a diagnostic (not disabled)
Promise.resolve(1344);