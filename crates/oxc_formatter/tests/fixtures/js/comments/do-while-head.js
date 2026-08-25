// Comments between a do-body's `}` and the `while` keyword keep their
// positions (head-body comment policy), like the `else`/`catch`/`finally` gaps.
// Known divergence (js/comments/between-head-and-body/between-head-and-body.js):
// Prettier flushes a line comment past the whole `while (x);` head and pulls an
// own-line comment into the parens (`while (\n// a\nx)`) -- attachment
// artifacts of the kind prettier is currently fixing elsewhere
// (prettier#19894 family).
do {} /* a */ while (x);
do {} // a
while (x);
do {}
// a
while (x);
do x();
// a
while (y);
do {} while (x); // real trailing
