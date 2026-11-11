const x = $abilities ?? ["foo", "bar", "bar"] ?? ["foo", "bar", "bar", "veryVeryVeryVeryVeryVeryVeryVeryVeryLongKey"];

const y = $abilities ?? { a: 1, b: 2 } ?? { a: 1, b: 2, c: 3, veryVeryVeryVeryVeryVeryVeryVeryVeryLongKey: true };

// Not every right side inlines, the chain breaks at an operator and keeps the indent
const z = $abilities ?? foo ?? ["foo", "bar", "bar", "veryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongKey"];
