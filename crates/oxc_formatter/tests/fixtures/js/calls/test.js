// Test patterns with multiple levels should not break incorrectly
test.describe.serial("My test suite", () => {
  // test content
});

test.describe.serial("Import glossary from JSON", () => {
  // more test content
});

test.describe.parallel("Another test suite", () => {
  // test content
});

test.describe.serial.only("Test with only", () => {});

test.describe.parallel.only("Parallel only", () => {});

test.describe.only("Describe only", () => {});

// These simpler patterns should still work
test.only("Test only", () => {});
describe.only("Describe only", () => {});
it.only("It only", () => {});

test(code.replace((c) => ""), () => {});

expect(content)
  .toMatch(`props: /*@__PURE__*/_mergeDefaults(['foo', 'bar', 'baz'], {
})`)
