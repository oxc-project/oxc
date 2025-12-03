// This file has a fixable tsgolint suggestion: no-floating-promises
// The floating promise should be handled with void or await
async function test() {
  Promise.resolve('value');
}

export { test };
