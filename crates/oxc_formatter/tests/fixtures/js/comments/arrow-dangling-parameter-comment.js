// A dangling comment in an empty parameter list keeps the last-argument hug
// (Prettier's expansion bailout never sees a parameter-less list)
foo((
  // c1
) => {});
foo(a, (
  // c2
) => {});
foo((/* c3 */) => {});
