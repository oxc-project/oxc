// A suppression comment before a union: on the union's line it covers the whole union,
// ending its line (own-line or not) the first member, and the union lays itself out below.
// Prettier's `handleUnionTypeComments` converges to the same split for `type =`;
// for `let :` and a line-ending block comment it keeps the whole union verbatim instead
// (see DIVERGENCES.md#union-suppression-line-ending-comment).
// alias, inline
type A1 = /* prettier-ignore */ Aaaa<X,Y> | Bbbb<X,Y>;
// annotation, inline
let a2: /* prettier-ignore */ Aaaa<X,Y> | Bbbb<X,Y>;
// as, inline
const a3 = 1 as /* prettier-ignore */ Aaaa<X,Y> | Bbbb<X,Y>;
// alias, own-line (before leading pipe)
type A4 =
  // prettier-ignore
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
// annotation, own-line
let a5:
  // prettier-ignore
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
// alias, own-line, no leading pipe
type A6 =
  // prettier-ignore
  Aaaa<X,Y> | Bbbb<X,Y>;
// alias, inline after leading pipe
type A7 = | /* prettier-ignore */ Aaaa<X,Y> | Bbbb<X,Y>;
// alias, mid-chain
type A8 = Aaaa<X,Y> | /* prettier-ignore */ Bbbb<X,Y>;
// single member alias
type A9 = /* prettier-ignore */ Aaaa<X,Y>;
// annotation, line comment ending the `:` line (divergence: Prettier keeps the whole union verbatim)
let a10: // prettier-ignore
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
// alias, block comment ending its line (divergence: Prettier pulls the whole union up, verbatim)
type A11 = /* prettier-ignore */
  | Aaaa<X,Y>
  | Bbbb<X,Y>;
// own-line before a hand-aligned union: only the first member is protected (Prettier too)
type A12 =
  // prettier-ignore
  | 'aaa'   | 'b'
  | 'ccccc' | 'dd';
