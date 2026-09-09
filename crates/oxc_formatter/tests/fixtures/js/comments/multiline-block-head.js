// A multiline block comment between a head and its `{` stays inline like any
// other same-line block comment (head-body comment policy, no exception for
// multiline content).
// Known divergence (js/comments/between-head-and-body/between-head-and-body.js):
// Prettier moves it onto its own line for while/do/else heads only, keeping it
// inline everywhere else -- an internal inconsistency, one uniform rule instead.
while (x) /* one
two */ {}
do /*
c */ {} while (y);
if (x) {} else /* p
q */ {}

// Inline everywhere else in both formatters (control)
function f() /* a
b */ {}
class A /* a
b */ {}
try /* a
b */ {} catch (e) /* c
d */ {} finally /* e
f */ {}
foo: /* a
b */ {
  break foo;
}
while (x) /* a
b */;
do {} /* a
b */ while (y);
