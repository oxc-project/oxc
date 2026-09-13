// Comments between a method head and its body `{` stay outside the braces
// (head-body comment policy): a same-line block comment inline (m1/m2),
// a line comment keeping its line with the `{` forced onto the next line (m3/m4).
// Known divergence (DIVERGENCES.md "head-body-comment-relocation"):
// Prettier pulls the m3/m4 line comments inside the braces (`{\n  // line comment`).
class A {
  m1(element: Element, key: string, undefined: undefined) /* block comment */ {
    // method body
  }
  m2(element: Element, key: string, undefined: undefined): void /* block comment */ {
    // method body
  }
  m3(tagName: string, rect: number[]): void // line comment
  {
    // method body
  }
  m4(tagName: string, rect: number[]) // line comment
  {
    // method body
  }
}
