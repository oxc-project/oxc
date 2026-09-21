// C pins the conceded extra reverse mapping for an uncached member
// reference; tsc folds it to the string value with no reverse mapping.
enum E {
  A = "\uD800",
  B = "\uD83D" + "\uDE00",
  C = A,
  D = "x\uDC00" + "y",
}
