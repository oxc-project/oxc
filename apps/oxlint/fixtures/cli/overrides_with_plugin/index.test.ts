describe("", () => {
  // ^ jest/no-valid-title error as explicitly set in the `.test.ts` override

  it("", () => {});
  // ^ jest/no-valid-title error as explicitly set in the `.test.ts` override
  // ^ jest/expect-expect as `jest` plugin is enabled and `jest/expect-expect` is a correctness rule
});

const foo = 123;
// no no-unused-vars error as override disables this rule for `.test.ts` files
