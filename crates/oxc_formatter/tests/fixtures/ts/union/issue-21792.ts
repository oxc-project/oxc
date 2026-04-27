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
