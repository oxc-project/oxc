// A suppressed statement whose source has no `;` prints none, under `semi: true` too.
// The source parsed the two statements apart and the verbatim text keeps every token,
// so only a first token the reprint introduces can merge them: `a => a` -> `(a) => a`.
// That statement takes the ASI guard whatever `semi` says; Prettier prints the merged
// `foo(  )(a) => a` (DIVERGENCES.md#suppressed-unterminated-asi-guard).

foo(  ) // prettier-ignore
a => a

// The rightmost single-statement body is the unterminated one
if (x) bar(  ) // prettier-ignore
b => b

// prettier-ignore
const c   = 1
d => d

// A `;` from the next line belongs to the guard, not to the verbatim text
foo(  ) // prettier-ignore
;
e => e

// A body's `}` and a keyword statement end are boundaries by grammar: no guard
// prettier-ignore
class K {}
g => g

// prettier-ignore
debugger
h => h

function f() {
  // prettier-ignore
  return   1
  i => i
}

switch (x) {
  case 1:
    foo(  ) // prettier-ignore
    j => j
}

export default   x // prettier-ignore
k => k

// A source `;` terminates: no guard under `semi: true`
foo(  ); // prettier-ignore
l => l

// A leading-suppressed expression ending a `;`-less statement is verbatim as well;
// the guard errs on its side (the statement printed its `;`, the guard is inert)
m = /* prettier-ignore */ foo(  )
n => n
