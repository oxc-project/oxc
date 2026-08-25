// A block comment between a clause's `:` and its single-block body prints
// outside the `{`, for case and default alike
// (the conformance suite covers only the default clause).
switch (x) {
  case a: /* c */ {
    break;
  }
  default: /* d */ {
    break;
  }
}
