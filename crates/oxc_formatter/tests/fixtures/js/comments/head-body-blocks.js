// Comments between a head and its body `{` stay outside the braces for
// function/arrow/method/class/labeled heads too (head-body comment policy).
// Known divergences (js/comments/function*, js/class-comment/*, js/label/comment.js):
// Prettier pulls line and own-line comments inside the braces, hoists a labeled
// statement's comment above the label, and breaks an `if` consequent whose
// trailing line comment its attachment marks as multiline -- artifacts of the
// kind prettier is currently fixing elsewhere (prettier#19894 family,
// open prettier#7745 / #5900).
// An arrow's block body pushed down by its head comments indents under the
// arrow (own-line converges with Prettier; for a same-line line comment
// Prettier own-lines the comment, we keep its position).
function f1() /* c */ {}
function f2() // c
{}
function f3()
// c
{}

g1 = () => /* c */ {};
g2 = () => // c
{};
g3 = () =>
// c
{};

class C {
  m1() /* c */ {}
  m2() // c
  {}
}

class A1 /* c */ {}
class A2 // c
{}
class A3
// c
{}
class A4 extends B // c
{}

foo1: /* c */ {
  break foo1;
}
foo2: // c
{
  break foo2;
}
foo3: // c
bar();

// An `if` consequent's trailing line comment rides the line without forcing
// the consequent onto its own line, and `else` comments keep their positions
if (a) doSomething(); // c
else if (b) other(); // d
else {}

if (a) {} /* b4 */ else {}
if (a) {} // b4
else {}
if (a) {}
// own
else {}
if (a) {} else /* c */ if (b) {} else /* d */ {}

// A trailing suppression comment keeps the consequent's original text
if (a) ugly(  1  ) // prettier-ignore
else b();
