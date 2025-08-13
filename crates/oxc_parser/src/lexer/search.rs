//! Structs and macros for searching source for combinations of byte values.
//!
//! * `ByteMatchTable` and `SafeByteMatchTable` are lookup table types for byte values.
//! * `byte_match_table!` and `safe_byte_match_table!` macros create those tables at compile time.
//! * `byte_search!` macro searches source text for first byte matching a byte table.

/// Batch size for searching
pub const SEARCH_BATCH_SIZE: usize = 32;

/// Safe byte matcher lookup table.
///
/// Create table at compile time as a `static` or `const` with `safe_byte_match_table!` macro.
/// Test bytes against table with `SafeByteMatchTable::matches`.
///
/// Only difference between this and the original `ByteMatchTable` is that for `SafeByteMatchTable`,
/// it must be guaranteed that byte searching using this table will always end up with `lexer.source`
/// positioned on a UTF-8 character boundary.
///
/// Usage is safe and does not require an `unsafe {}` block.
///
/// To make this guarantee, one of the following must be true:
///
/// 1. Table contains `true` for all byte values 192 - 247
///   i.e. first byte of any multi-byte Unicode character matches.
///   (NB: 248 - 255 cannot occur in UTF-8 strings)
///   e.g.
///   `safe_byte_match_table!(|b| b >= 192)`
///   `safe_byte_match_table!(|b| !b.is_ascii())`
///
/// 2. Table contains `false` for all byte values 128 - 191
///   i.e. the continuation bytes of any multi-byte Unicode chars will be consumed in full.
///   e.g.
///   `safe_byte_match_table!(|b| b < 128 || b >= 192)`
///   `safe_byte_match_table!(|b| b.is_ascii())`
///   `safe_byte_match_table!(|b| b == ' ' || b == '\t')`
///
/// This is statically checked by `SafeByteMatchTable::new`, and will fail to compile if match
/// pattern does not satisfy one of the above.
///
/// # Examples
/// ```
/// use crate::lexer::search::{SafeByteMatchTable, safe_byte_match_table};
///
/// static NOT_ASCII: SafeByteMatchTable = safe_byte_match_table!(|b| !b.is_ascii());
/// assert_eq!(NOT_ASCII.matches(b'X'), false);
/// assert_eq!(NOT_ASCII.matches(192), true);
/// ```
#[repr(C, align(64))]
pub struct SafeByteMatchTable([bool; 256]);

impl SafeByteMatchTable {
    // Create new `SafeByteMatchTable`.
    pub const fn new(bytes: [bool; 256]) -> Self {
        let mut table = Self([false; 256]);

        // Check if contains either:
        // 1. `true` for all byte values 192..248
        // 2. `false` for all byte values 128..192
        let mut unicode_start_all_match = true;
        let mut unicode_cont_all_no_match = true;

        let mut i = 0;
        loop {
            let matches = bytes[i];
            table.0[i] = matches;

            if matches {
                if i >= 128 && i < 192 {
                    unicode_cont_all_no_match = false;
                }
            } else if i >= 192 && i < 248 {
                unicode_start_all_match = false;
            }

            i += 1;
            if i == 256 {
                break;
            }
        }

        assert!(
            unicode_start_all_match || unicode_cont_all_no_match,
            "Cannot create a `SafeByteMatchTable` with an unsafe pattern"
        );

        table
    }

    /// Declare that using this table for searching.
    /// A safe function.
    #[expect(clippy::unused_self)]
    #[inline]
    pub const fn use_table(&self) {}

    /// Test a value against this `SafeByteMatchTable`.
    #[inline]
    pub const fn matches(&self, b: u8) -> bool {
        self.0[b as usize]
    }
}

/// Macro to create a `SafeByteMatchTable` at compile time.
///
/// `safe_byte_match_table!(|b| !b.is_ascii())` expands to:
///
/// ```
/// {
///   use crate::lexer::search::SafeByteMatchTable;
///   #[allow(clippy::eq_op, clippy::allow_attributes)]
///   const TABLE: SafeByteMatchTable = SafeByteMatchTable::new([
///     (!0u8.is_ascii()),
///     (!1u8.is_ascii()),
///     /* ... */
///     (!255u8.is_ascii()),
///   ]);
///   TABLE
/// }
/// ```
macro_rules! safe_byte_match_table {
    (|$byte:ident| $res:expr) => {{
        use crate::lexer::search::SafeByteMatchTable;
        // Clippy creates warnings because e.g. `safe_byte_match_table!(|b| b == 0)`
        // is expanded to `SafeByteMatchTable([0 == 0, ... ])`
        #[allow(clippy::eq_op, clippy::allow_attributes)]
        const TABLE: SafeByteMatchTable = seq_macro::seq!($byte in 0u8..=255 {
            SafeByteMatchTable::new([#($res,)*])
        });
        TABLE
    }};
}
pub(crate) use safe_byte_match_table;

use super::{Lexer, SourcePosition};

/// Simple byte search that doesn't take any closures to avoid borrowing issues.
/// Returns the matched byte and the position where it was found, or None if EOF reached.
pub fn byte_search_raw<'a>(
    lexer: &Lexer<'a>,
    table: &SafeByteMatchTable,
    start_pos: SourcePosition<'a>,
) -> Option<(u8, SourcePosition<'a>)> {
    #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
    table.use_table();

    let mut pos = start_pos;
    // Silence warnings if macro called in unsafe code
    #[allow(unused_unsafe, clippy::unnecessary_safety_comment, clippy::allow_attributes)]
    'outer: loop {
        let byte = if pos.can_read_batch_from(&lexer.source) {
            // Search a batch of `SEARCH_BATCH_SIZE` bytes.
            //
            // `'inner: loop {}` is not a real loop - it always exits on first turn.
            // Only using `loop {}` so that can use `break 'inner` to get out of it.
            // This allows complex logic to be outside the `for` loop, keeping it as minimal
            // as possible, to encourage compiler to unroll it.
            //
            // SAFETY:
            // `pos.can_read_batch_from(&lexer.source)` check above ensures there are
            // at least `SEARCH_BATCH_SIZE` bytes remaining in `lexer.source`.
            // So `pos.add()` in this loop cannot go out of bounds.
            let batch = unsafe { pos.slice(SEARCH_BATCH_SIZE) };
            'inner: loop {
                for (i, &byte) in batch.iter().enumerate() {
                    if table.matches(byte) {
                        // SAFETY: Cannot go out of bounds (see above).
                        // Also see above about UTF-8 character boundaries invariant.
                        pos = unsafe { pos.add(i) };
                        break 'inner byte;
                    }
                }
                // No match in batch - search next batch.
                // SAFETY: Cannot go out of bounds (see above).
                // Also see above about UTF-8 character boundaries invariant.
                pos = unsafe { pos.add(SEARCH_BATCH_SIZE) };
                continue 'outer;
            }
        } else {
            // Not enough bytes remaining for a batch. Process byte-by-byte.
            // Same as above, `'inner: loop {}` is not a real loop here - always exits on first turn.
            'inner: loop {
                // SAFETY: `pos` is before or equal to end of source
                let remaining = unsafe {
                    let remaining_len = lexer.source.end().offset_from(pos);
                    pos.slice(remaining_len)
                };
                for (i, &byte) in remaining.iter().enumerate() {
                    if table.matches(byte) {
                        // SAFETY: `i` is less than number of bytes remaining after `pos`,
                        // so `pos + i` cannot be out of bounds
                        pos = unsafe { pos.add(i) };
                        break 'inner byte;
                    }
                }

                // EOF - no match found
                return None;
            }
        };

        // Found a match
        return Some((byte, pos));
    }
}