// DIVERGES: comments around a hole stay on their side of its `,`, spaced after it (all but `e` and `m` differ from Prettier);
// see DIVERGENCES.md#array-hole-comments
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
// An own-line line comment before the last hole stays before its `,`
const g = [,
  // line
,];
// A comment before a hole stays before its `,`, in every array-like node
const h = [a, /* comment */, b];
[a, /* comment */, b] = x;
const [i, /* comment */, j] = x;
// Own-line comments keep their side of the hole's `,`
const k = [
  a,
  // before
  ,
  // after
  b,
];
// A same-line line comment after a hole keeps its line
const l = [, // line
  ,];
// Comments in a pattern of holes stay inside the brackets
const [, /* comment */ ,] = x;
[, // line
] = x;
// A trailing comment past a JSDoc cast's parens stays outside them
const m = [/** @type {X} */ (a) /* comment */, , b];
// A leading hole, consecutive holes, and comments on both sides of a separator
const n = [/* lead */, 2];
const o = [1, /* c1 */, /* c2 */, 4];
const p = [1 /* t */, /* slot */, 3];
// Right after the `[`, a same-line run ending in a line comment moves to its own line as a whole
const q = [// comment
, b];
const r = [/* a */ // b
, b];
