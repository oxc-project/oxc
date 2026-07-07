// https://github.com/oxc-project/oxc/issues/23110
// A trailing comment between the statement content and its semicolon
// is printed after the semicolon, like Prettier >= 3.9
foo = 1 /* a */;
const myVar = "asdf" /* b */;
let noInit: string | number /* c */;
// Note: Prettier moves the comment only for an exported type alias and keeps
// `type T = string /* t */;` as-is; oxfmt intentionally applies the rule uniformly
type T = string /* t */;
function f() {
  return foo /* d */;
}
function g() {
  return /* no argument */;
}
function h() {
  throw foo /* e */;
}
import { a } from "mod" /* f */;
import "side-effect" /* g */;
import x, { y } from "mod2" with { type: "json" } /* h */;
export { b } from "mod" /* i */;
export * from "mod" /* j */;
export const exported = 1 /* k */;
export default foo /* l */;
1 as const /* m */;

// Multiple comments move together, comments after the semicolon stay
baz = 3 /* n1 */ /* n2 */;
qux = 4 /* o1 */; /* o2 */

// An own-line comment becomes a leading comment of the next statement
bar = 2
/* own line */;
quux();

// A trailing suppression comment keeps the statement's original text
suppressed  =  ugly(   1) // prettier-ignore
;
notSuppressed3();

// A return/throw argument's parentheses survive in the output, so comments
// inside them stay there (moving them behind the `;` would cross the `)` and,
// when the group breaks, a line boundary too — breaking line directives);
// only comments after the closing paren move behind the semicolon
function multiLineReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition // eslint-disable-line some-rule
  );
}
function ownLineCommentReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition
    /* eslint-enable some-rule */
  );
}
function afterCloseParenReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition
  ) /* moves */;
}
// A comment inside a parenthesized sub-expression belongs to that expression
// and stays attached, like Prettier
parenthesized = (someValue /* stays */);

// The `;` on a later line still moves the comment: in the output the semicolon
// directly follows the content, so nothing is crossed
assigned =
  someValue /* moves */
;
// ... also when the content breaks
export type Union =
  | AaaaaaaaaaaaaaaaaaaaaaaaaaaaaaLongMember
  | BbbbbbbbbbbbbbbbbbbbbbbbbbbbbbLongMember /* moves */;

// No source `;` at all (ASI): nothing to move the comment across
function asiReturn() {
  return (
    aLongLongLongLongLongCondition &&
    anotherLongLongLongLongCondition /* stays */
  )
}

// The declaration below is terminated by the `;` after the own-line comment;
// the comment stays own-line and leads the next statement, not the declaration
export let laterSemi: (callback: () => void, timeout?: number) => Disposable

// Self-invoking function comment
;(function () {})();

// do-while: a comment between `)` and `;` moves behind the semicolon,
// a comment inside the parens stays
do {} while (foo /* in parens */);
do {} while (foo) /* between */;
do {} while (foo) // line between
;

// Labeled break/continue move the comment behind the semicolon
labeled: for (;;) {
  break labeled /* p */;
  continue labeled /* q */;
}

// Class properties move the comment behind the semicolon,
// an own-line comment before the semicolon keeps its own line
class Cls {
  a = 1 /* r */;
  b = 2 // line r
  ;
  declare c: number /* s */;
  d = 4 /* keeps */
  /* own line */;
  e = 5;
}

// Bodyless method signatures (overloads, abstract, ambient) move the comment
// behind the semicolon too; an own-line comment stays in place
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
class WithIndexSignature {
  [key: string]: unknown /* v2 */;
}
