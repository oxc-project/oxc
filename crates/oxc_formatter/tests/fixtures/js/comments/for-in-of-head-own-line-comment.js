// An own-line comment between a for-in/for-of head's left side and its right side
// keeps its own line, above the statement.
// Divergence (js/for-of/comments.js): Prettier makes it lead the right side
// (`for (x in // c` + break + `y)`), a shape its second pass moves behind the `)`
// and the body's `{`; not a fixpoint (prettier#12880 family).
for (x
// c1
in y);
for (x in
// c2
y) {}
for (x of
/* c3 */
y) {}
for (const [a, b] of
// c4
entries) {}
