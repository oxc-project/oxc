// Trailing comments on class and interface members (the statement-level
// basics: trailing-comments.ts).
// Class properties move the same-line comment behind the semicolon;
// an own-line comment before the `;` defers to the next element's leading pass
// (Prettier's first pass cancels the same-line move in that case
// and needs a second pass to settle — we print that fixpoint directly)
class Cls {
  a = 1 /* r */;
  b = 2 // line r
  ;
  declare c: number /* s */;
  d = 4 /* keeps */
  /* own line */;
  e = 5;
  // A comment inside the value's parentheses moves behind the added `;`
  // even without a source `;`, like Prettier
  f = (g = 6 /* moves */)
}

// Under bare ASI a same-line comment sits past the member's span,
// so it still moves behind the added `;` (like statements)
class BareAsi {
  a = 1 /* r */
  b = 2
}

// A definite/optional marker between the content end and the `;`
class Markers {
  x!: number /* m1 */;
  z? /* m2 */;
}

// Bodyless method signatures (overloads, abstract, ambient) move the
// same-line comment behind the semicolon too; own-line ones defer to the
// next element, like every other member
class Overloads {
  m(): void /* w */;
  m(): void {}
  constructor(x: number) /* x */;
  constructor() {}
}
abstract class Abstract {
  abstract am(): void /* y */;
}
declare class Ambient {
  dm(): void /* z */;
  dl(): void // line z
  ;
  down(): void
  /* own line */;
  dmixed(): void /* w */
  /* own2 */;
}

// Note: Prettier keeps interface / type literal member comments before the
// semicolon (member separator, not a statement terminator); so do we,
// also for index signatures — even in classes
interface Iface {
  foo: string /* u */;
  bar(): void /* u2 */;
}
type ObjType = {
  foo: string /* v */;
};
// A LINE comment after a `;`-less member rides the line; the added separator
// lands before it
interface _KeywordDef {
  type?: JSONType | JSONType[] // data types that keyword applies to
}
class WithIndexSignature {
  [key: string]: unknown /* v2 */;
}
