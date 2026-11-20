// A comment leading the right-hand side on its own line breaks after the operator,
// before a complex destructuring target would break its left-hand side (as Prettier).
const { a, b: c, d } =
  // c
  foo;
const { a = 1, b, d } =
  // c
  foo;
const { a, b: c, d } =
  /* c */
  foo;
({ a, b: c, d } =
  // c
  foo);
const { id1, method: isMethod1, methodId1 } =
  // c
  privateNamesMap.get(name);
