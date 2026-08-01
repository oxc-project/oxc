// Issue #21792 - same-line comment after pipe in single member union
export type myType = | // Comment
  "A";

// Block comment same-line after pipe
type BlockComment = | /* block */
  "B";

// Same-line comment then own-line comment
type Mixed = | // first
  // second
  "C";

// Long line comment that exceeds printWidth on the `=` line
type LongType = | // This is a really long comment that might exceed the print width limit
  "A";
