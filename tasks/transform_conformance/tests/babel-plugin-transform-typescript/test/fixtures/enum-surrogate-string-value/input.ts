// A reference to a string member gets no reverse mapping, even when a lone
// surrogate keeps the value from being evaluated. A reverse mapping for B or
// H.A would overwrite the member named "𐀀".
enum E {
  "𐀀" = 1,
  A = "\uD800" + "\uDC00",
  B = A,
  C = "\uD800",
  D = C,
  G = "x\uDC00" + "y",
}
enum H {
  "𐀀" = 2,
  A = E.A,
}
