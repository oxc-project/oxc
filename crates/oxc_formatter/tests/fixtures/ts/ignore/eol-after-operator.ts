// A suppression comment on the `=` / `:` line keeps its line (printed right after the operator)
// and targets the right-hand side, at every assignment-like site; a type annotation before the
// operator is not its target.
// Prettier keeps the target only for type aliases and object properties
// (see DIVERGENCES.md#eol-suppression-after-assign-colon).
type T1 = // prettier-ignore
  Foo<X,Y>;
const c1 = // prettier-ignore
  foo( a,b );
c1 = // prettier-ignore
  foo( a,b );
const o = {
  k: // prettier-ignore
    foo( a,b ),
};
class C {
  p = // prettier-ignore
    foo( a,b );
}
const { k: // prettier-ignore
  [a,b] } = x;
// Formatter parens survive
const s = // prettier-ignore
  (a,b);
// A union as a whole (see union/suppression.ts), hugged or not
type U = // prettier-ignore
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
type H = // prettier-ignore
  { a:1 } | null;
// A block suppression comment followed by a riding line comment
const b = /* prettier-ignore */ // c
  foo( a,b );
type UB = /* oxfmt-ignore */ // c
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
let d: Foo = // prettier-ignore
  foo( a,b );
