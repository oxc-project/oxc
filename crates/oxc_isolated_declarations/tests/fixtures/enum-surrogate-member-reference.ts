export const enum Values {
  "\uD800" = 1,
  "\uDC00" = 2,
  Lead = Values["\uD800"],
  Trail = Values["\uDC00"],
  "a\uD800b" = "value",
  Text = Values[`a\uD800b`],
}
