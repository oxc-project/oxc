// All of them should be the same result!
[
  /\1(.)\\"'`a/v,
  new RegExp("\\1(.)\\\\\"'`\a","v"),
  new RegExp('\\1(.)\\\\"\'`\a','v'),
  new RegExp(`\\1(.)\\\\"'\`\a`,`v`),
]
