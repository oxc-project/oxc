// A leading multi-line `*`-aligned block comment breaks after `=` for type aliases too,
// a union included (it hands its indent over); an interface member is not an assignment-like and keeps it inline.
type A = /* multi
 * line */ string;
type C = /* multi
 * line */ { a: 1 } | null;
interface I {
  p: /* multi
   * line */ string;
}
