// A comment after a trailing hole stays inside the brackets, like Prettier
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
