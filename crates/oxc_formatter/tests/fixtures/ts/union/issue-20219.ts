// Issue #20219 - non-idempotent formatting: leading JSDoc comment on single union member
export type AuditLogOrderField =
  /** Order audit log entries by timestamp */
  | 'CREATED_AT';

// Multiple JSDoc blocks before single union member
export type MultipleJSDocBlocks =
  /** Comment 1 */
  /** Comment 2 */
  /** Comment 3 */
  | 'CREATED_AT';

// Multiline JSDoc block before single union member
export type MultilineJSDoc =
  /**
   * Order audit log entries by timestamp.
   * This is a multiline comment.
   */
  | 'CREATED_AT';
