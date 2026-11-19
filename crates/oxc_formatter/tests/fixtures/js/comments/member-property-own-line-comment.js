// DIVERGENCES.md#member-property-own-line-comment
class A {
  #b;
  m() {
    x = a
      // comment
      .b;
    x = a
      // comment
      .#b;
    return a
      // comment
      .#b;
  }
  n() {
    return a
      // comment
      .b;
  }
}
