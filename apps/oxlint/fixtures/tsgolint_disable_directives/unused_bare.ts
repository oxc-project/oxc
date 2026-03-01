// Test bare eslint-disable with type-aware rules
const myPromise = new Promise(() => {})

// Case 1: USED bare disable block (suppresses type-aware violation)
/* eslint-disable */
Promise.resolve(1)
/* eslint-enable */

// Case 2: UNUSED bare disable block (no violation inside)
/* eslint-disable */
const _x = 1 + 2
/* eslint-enable */

// Case 3: USED bare disable to end of file
/* eslint-disable */
Promise.resolve(2)

export {}
