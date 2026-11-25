// A typed `this` parameter is not "simple", so the only function argument does not expand its params
const mockFn = vi.fn().mockImplementation(function (
  this: MockWithPrettyLongName,
) {
  return this;
});

// Untyped `this` stays simple
const mockFn2 = vi.fn().mockImplementation(function (this, aaaaaaaaaaaaaaaaaaaaa) {
  return this;
});
