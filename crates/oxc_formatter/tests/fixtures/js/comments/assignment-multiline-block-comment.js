// A leading multi-line block comment whose lines all start with `*` breaks after the operator,
// at every assignment-like site and ahead of the never-break shapes (`require`, literals).
const a = /* multi
 * line */ foo;
a = /* multi
 * line */ foo;
const o = { k: /* multi
 * line */ foo };
class C {
  p = /* multi
   * line */ foo;
}
const r = /**
 * jsdoc
 */ require("mod");
const n = /* multi
 * line */ 1;
// A cast comment is the same rule: the multi-line one breaks, the single-line one stays inline.
const { aaa, bbb } = /** @type {{
 *   aaa: () => void,
 *   bbb: () => null,
 * }} */ (require("mod"));
const cast = /** @type {T} */ (foo);
// Not alignable (a line without `*`) and single-line comments stay inline.
const b = /* multi
line */ foo;
const c = /* single */ foo;
// Own-line comments already break after the operator.
const d = /* multi
 * line */
  foo;
