// https://github.com/oxc-project/oxc/issues/23110
// A trailing comment between the statement content and its semicolon
// is printed after the semicolon, like Prettier >= 3.9
// (paren/chain shapes: trailing-comments-parens.ts,
//  class and interface members: trailing-comments-class-members.ts)
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
export default ((a) => (foo /* l */));
1 as const /* m */;

// Multiple comments move together, comments after the semicolon stay
baz = 3 /* n1 */ /* n2 */;
qux = 4 /* o1 */; /* o2 */

// An own-line comment becomes a leading comment of the next statement,
// staying own-line with its blank lines
// (see DIVERGENCES.md#deferred-own-line-comment-stays-own-line)
bar = 2
/* own line */;
quux();
blankAfter = 3
/* c-blank-after */;

quux();
blankBefore = 4

/* c-blank-before */;
quux();
// ... also when the next statement prints its own leading pass (export)
beforeExport = 5
/* own before export */;

export default quux;

// A trailing suppression comment keeps the statement's original text
suppressed  =  ugly(   1) // prettier-ignore
;
notSuppressed3();

// The `;` on a later line still moves the comment: in the output the semicolon
// directly follows the content, so nothing is crossed
assigned =
  someValue /* moves */
;
// ... also when the content breaks
export type Union =
  | AaaaaaaaaaaaaaaaaaaaaaaaaaaaaaLongMember
  | BbbbbbbbbbbbbbbbbbbbbbbbbbbbbbLongMember /* moves */;

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
