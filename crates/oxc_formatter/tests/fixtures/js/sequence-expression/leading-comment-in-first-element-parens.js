// A comment inside the first element's dropped source parentheses sits within
// the sequence's span but leads the sequence, not the element: it prints
// outside the formatter-added parentheses.
// (Prettier 3.9 prints it inside on its first pass and moves it out on the
// second; oxfmt prints that fixpoint directly, see prettier#19894)
((/* c */ a), b);
((/* c1 */ /* c2 */ a), b);
((
  // own line
  a
), b);
const v = ((/* c */ a), b);

// A comment on a later element stays with that element
((a), /* c */ b);

// A cast comment keeps its target
(/** @type {T} */ (a), b);

// Arrow body: the comment leads the sequence and forces the body onto its own line
ret = () => ((/* c */ a), b);

// return keeps its argument parentheses; the comment stays inside them
function g() {
  return ((/* c */ a), b);
}

// for-init adds no parentheses
for ((/* c */ a), b; ; );
