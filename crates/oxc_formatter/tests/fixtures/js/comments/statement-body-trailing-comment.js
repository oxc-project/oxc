// A same-line comment before a single-statement body's distant `;` trails the whole
// statement (Prettier's statement `locEnd` stops at the content): printed past the body's
// indent, it no longer breaks the body onto its own line.
for (;;) continue // c1
;
for (;;) break /* c2 */
;
while (x) foo() // c3
;
if (x) foo() // c4
;
if (a) if (b) foo() // c5
;
with (a) foo() // c6
;
for (;;) return // c7
;
while (x) var a = 1 // c8
;
label: for (;;) continue label // c9
;
if (x) foo() /* c10 */ // c11
;
// A do-while body keeps it as well: the statement continues past the body
do foo() // c13
; while (x);
