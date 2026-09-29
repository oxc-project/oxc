// DIVERGES: a comment after a trailing hole stays after the hole (Prettier moves `f`'s back to the last real element);
// see DIVERGENCES.md#array-hole-trailing-comment
const a = [,, /* comment */];
const b = [
  , /* comment */
];
const c = [
  ,, /* comment */
];
// A trailing line comment forces the array to break
const d = [,, // line
];
// Comments trailing a real element are unaffected
const e = [1, 2 /* t */];
// A real element before the hole (the entry's shape)
const f = [1, , /* comment */];
