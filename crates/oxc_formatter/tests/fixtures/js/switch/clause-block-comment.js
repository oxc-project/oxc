// Comments between a clause's `:` and its single-block body stay outside the
// `{` (head-body comment policy), for case and default alike:
// a block comment inline, a line comment keeping its position with the `{`
// forced onto the next line, an own-line comment keeping its own line.
// Known divergence (js/switch/comments.js): Prettier treats the same shape
// unevenly -- `case b: { // c` (past the `{`) but `default: {` + own-lined
// `// d` inside the block, and an own-line comment inlined to `case c: // own`
// -- attachment artifacts of the kind prettier is currently fixing elsewhere
// (prettier#19894 family).
switch (x) {
  case a: /* c */ {
    break;
  }
  default: /* d */ {
    break;
  }
}
switch (x) {
  case b: // c
  {
    break;
  }
  default: // d
  {
    break;
  }
}
switch (x) {
  case c:
  // own line
  {
    break;
  }
}
// A fallthrough clause's trailing comment stays on its line,
// and a non-block consequent keeps its clause-line comment,
// also after `default:`
switch (x) {
  case d: // fallthrough
  case e: // consequent
    f();
  default: // dangling
    g();
}
