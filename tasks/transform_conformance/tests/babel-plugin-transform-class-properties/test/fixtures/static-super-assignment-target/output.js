// We don't know what correct output for this should be.
//
// Babel doesn't support this, but ESBuild does.
//
// We'd need to add a new helper function to support this.

class Outer {}

babelHelpers.defineProperty(Outer, "prop", () => {
  // ?????
});
