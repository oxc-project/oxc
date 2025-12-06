// This file has a tsgolint suggestion (not a fix): no-floating-promises
// With --fix alone, the suggestion should NOT be applied
async function test() {
  Promise.resolve('value');
}

export { test };
