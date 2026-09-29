// An end-of-line line comment right after `=`/`:` keeps its position
// (`= // c` + mandatory break). Prettier flushes it past a FITTING right-hand
// side (`const v1 = 1; // c`, crossing the value) while keeping the order when
// the right-hand side breaks — one rule for both shapes
// (Known divergence, see DIVERGENCES.md#eol-comment-after-assign-colon).

const v1 = // c
  1;

v2 = // c
  1;

const o = {
  p: // c
    1,
};

class C {
  f = // c
    1;
}

// Object, array and template values too (Prettier own-lines it above them)
const h1 = // c
  { a: 1 };
const h2 = // c
  [1];
const h3 = // c
  `x`;
h4 = // c
  { a: 1 };
const h5 // c
  = { a: 1 };
const h6 = // prettier-ignore
  { a:   1 };

// Breaking right-hand side: Prettier keeps the order here too (no divergence)
const v3 = // c
  someLongFunctionCall(argumentOne, argumentTwo, argumentThree, argumentFours);

// Block comments glued after the operator stay after it, in order, before the
// riding line comment (Prettier moves the block across the line comment:
// `const b1 = // d` + `/* c */ foo(a, b)`)
const b1 = /* c */ // d
  foo(a, b);
b2 = /* c */ /* e */ // d
  foo(a, b);
const o2 = {
  p: /* c */ // d
    1,
};
class C2 {
  f = /* c */ // d
    1;
}
// Before the operator they trail the left side, also when ending its line
// (Prettier moves a line-ending one across the operator: `const b6 = /* c */ 1; // d`)
const b3 /* c */ = // d
  1;
const b6 /* c */
= // d
  1;
b7 /* c */
= // d
  1;
const b8 /* c */
= 1;
class C4 {
  f /* c */
  = // d
    1;
}
const o4 = {
  p /* c */
  : // d
    1,
};
const b10 /* c */
= // oxfmt-ignore
  [1,   2];
// A line comment before the operator trails the left side and breaks after the operator the same way
// (Prettier flushes it past a fitting value)
const b4 // c
  = 1;
const b5 // c
  = foo(b);
const o3 = {
  p // c
    : 1,
};
class C3 {
  f // c
    = 1;
}
// Own-line comments the left side defers lead the right-hand side, the line comment follows them in source order
// (Prettier prints the line comment first: `const b9 = // d`)
const b9
// c
= // d
  1;
a.b
// c
= // d
  1;
