// Comments on a SINGLE-member union (no `|` printed in the output).
// The `&` mirror lives in intersection-type/single-member.ts.

// Issue #21792 - same-line comment after the pipe
export type myType = | // Comment
  "A";
type BlockComment = | /* block */
  "B";
type Mixed = | // first
  // second
  "C";
type LongType = | // This is a really long comment that might exceed the print width limit
  "A";

// Issue #21399 - own-line comment after the pipe
export type OwnLine = |
  // Comment
  "A";
type OwnLineBlock = |
  /* block comment */
  "B";
type MultipleComments = |
  // Comment 1
  // Comment 2
  "C";

// Issue #20219 - non-idempotent formatting: leading JSDoc on the single member
export type AuditLogOrderField =
  /** Order audit log entries by timestamp */
  | 'CREATED_AT';
export type MultipleJSDocBlocks =
  /** Comment 1 */
  /** Comment 2 */
  /** Comment 3 */
  | 'CREATED_AT';
export type MultilineJSDoc =
  /**
   * Order audit log entries by timestamp.
   * This is a multiline comment.
   */
  | 'CREATED_AT';
type LineComment =
  // line comment
  | 'VALUE';
// Intersection cross-check with an own-line comment
type IntersectionComment =
  /** JSDoc */
  & 'VALUE';
// Inline comment stays after `=` (not own-line, never moved before `=`)
type InlineComment = /*1*/ | C;
