// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

/**
 * Total size of the transfer buffer in bytes (block size minus allocator metadata).
 */
export const BUFFER_SIZE = 2147483616;

/**
 * Required alignment of the transfer buffer (4 GiB).
 */
export const BUFFER_ALIGN = 4294967296;

/**
 * Size of the active data area in bytes (buffer size minus raw metadata and chunk footer).
 */
export const ACTIVE_SIZE = 2147483552;

/**
 * Byte offset of the data pointer within the buffer, divided by 4 (for `Uint32Array` indexing).
 */
export const DATA_POINTER_POS_32 = 536870900;

/**
 * Byte offset of the `is_ts` flag within the buffer.
 */
export const IS_TS_FLAG_POS = 2147483612;

/**
 * Byte offset of the `is_jsx` flag within the buffer.
 */
export const IS_JSX_FLAG_POS = 2147483613;

/**
 * Byte offset of the `has_bom` flag within the buffer.
 */
export const HAS_BOM_FLAG_POS = 2147483614;

/**
 * Byte offset of the tokens offset within the buffer, divided by 4 (for `Uint32Array` indexing).
 */
export const TOKENS_OFFSET_POS_32 = 536870901;

/**
 * Byte offset of the tokens length within the buffer, divided by 4 (for `Uint32Array` indexing).
 */
export const TOKENS_LEN_POS_32 = 536870902;

/**
 * Byte offset of the `program` field, relative to start of `RawTransferData`.
 */
export const PROGRAM_OFFSET = 0;

/**
 * Byte offset of pointer to start of source text, relative to start of `Program`.
 */
export const SOURCE_START_OFFSET = 8;

/**
 * Byte offset of length of source text, relative to start of `Program`.
 */
export const SOURCE_LEN_OFFSET = 16;

/**
 * Byte offset of comments `Vec` pointer, relative to start of `Program`.
 */
export const COMMENTS_OFFSET = 24;

/**
 * Byte offset of comments `Vec` length, relative to start of `Program`.
 */
export const COMMENTS_LEN_OFFSET = 32;

/**
 * Size of `Comment` struct in bytes.
 */
export const COMMENT_SIZE = 16;

/**
 * Byte offset of `kind` field, relative to start of `Comment` struct.
 */
export const COMMENT_KIND_OFFSET = 12;

/**
 * Discriminant value for `CommentKind::Line`.
 */
export const COMMENT_LINE_KIND = 0;
