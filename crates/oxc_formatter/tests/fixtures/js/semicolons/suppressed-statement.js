// A suppressed statement keeps its source text up to the content end;
// the terminator stays the formatter's and follows `semi` (added or removed),
// and comments between the content and the source `;` lead the next statement.
// Prettier re-adds a content-terminated statement's `;` only when the source had one
// (DIVERGENCES.md#suppressed-terminator-per-semi).

// prettier-ignore
do;   while(   1)

;[].sort()

// prettier-ignore
do  {}   while(   two  );

lbl: for (;;) {
  // prettier-ignore
  continue                   lbl

  // breaking comment
  ;(possibleArray || []).sort()
}

lbl2: for (;;) {
  // prettier-ignore
  break                   lbl2;
}

lbl3: for (;;) {
  // prettier-ignore
  break   lbl3
}

// A variable declaration's ignored range ends at the last declarator, source `;` or not,
// behind `export` too.

// prettier-ignore
const noSemi   =   1

;[].sort()

// prettier-ignore
let cast = /** @type {string} */ (   value  );

;[].sort()

// prettier-ignore
var multi   =   1,   multi2   =   2

;[].sort()

// The rightmost-body recursion reaches the declaration
// prettier-ignore
if (cond) var inBody   =   1

;[].sort()

// prettier-ignore
export const exported   =   1

foo()

// A suppressed expression statement also ends its ignored range at the content
// and still gets its `semi: false` ASI guard.

// prettier-ignore
stmt(   );

// prettier-ignore
[breaking].sort();

// A trailing suppression comment suppresses the same way, whether the source `;`
// follows it on the line or sits on the next line (`semi: false` style);
// under `semi: true` the terminator makes the next statement's `(` safe
// where Prettier's output merges into `stmt(  )(a) => a`.
stmt(   ) // prettier-ignore
;[].sort()

stmt(   ) // prettier-ignore
a => a

stmt(   ); // prettier-ignore

// An `if` consequent before `else` goes the same way
if (cond) stmt(   ); // prettier-ignore
else other()

if (cond) stmt(   ) // prettier-ignore
; else other()
