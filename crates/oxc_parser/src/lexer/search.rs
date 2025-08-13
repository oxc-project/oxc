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

/// Result of continue_if evaluation in byte search
#[derive(Debug)]
pub enum SearchContinuation<'a> {
    /// Stop searching and return the matched byte, with lexer positioned at the given position
    Stop(SourcePosition<'a>),
    /// Continue searching from the given position
    Continue(SourcePosition<'a>),
}

/// Search for bytes matching a table with advanced continuation logic.
/// The `continue_if` function is called with the matched byte and position.
/// It should return a SearchContinuation indicating whether to continue or stop,
/// and what position to use.
/// Returns the matched byte, or calls the eof_handler if EOF is reached.
pub fn byte_search_with_advanced_continue<T>(
    lexer: &mut Lexer<'_>,
    table: &SafeByteMatchTable,
    start_pos: SourcePosition<'_>,
    mut continue_if: impl FnMut(u8, SourcePosition<'_>) -> SearchContinuation<'_>,
    eof_handler: impl FnOnce() -> T,
) -> Result<u8, T> {
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

                // EOF.
                // Advance `lexer.source`'s position to end of file.
                lexer.source.advance_to_end();

                // Avoid lint errors if `eof_handler` contains `return` statement
                #[allow(
                    unused_variables,
                    unreachable_code,
                    clippy::diverging_sub_expression,
                    clippy::allow_attributes
                )]
                {
                    let eof_ret = eof_handler();
                    return Err(eof_ret);
                }
            }
        };

        // Found match. Check if should continue.
        match continue_if(byte, pos) {
            SearchContinuation::Continue(new_pos) => {
                // Continue searching from new position
                pos = new_pos;
                continue;
            }
            SearchContinuation::Stop(final_pos) => {
                // Match confirmed.
                // Advance `lexer.source`'s position to final position.
                // SAFETY: See above about UTF-8 character boundaries invariant.
                lexer.source.set_position(final_pos);
                return Ok(byte);
            }
        }
    }
}

/// Search for bytes matching a table, starting from the current lexer position.
/// Returns the matched byte, or calls the eof_handler if EOF is reached.
pub fn byte_search_simple<T>(
    lexer: &mut Lexer<'_>,
    table: &SafeByteMatchTable,
    eof_handler: impl FnOnce() -> T,
) -> Result<u8, T> {
    byte_search_from_pos_simple(lexer, table, lexer.source.position(), eof_handler)
}

/// Search for bytes matching a table, starting from the specified position.
/// Returns the matched byte, or calls the eof_handler if EOF is reached.
pub fn byte_search_from_pos_simple<T>(
    lexer: &mut Lexer<'_>,
    table: &SafeByteMatchTable,
    start_pos: SourcePosition<'_>,
    eof_handler: impl FnOnce() -> T,
) -> Result<u8, T> {
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

                // EOF.
                // Advance `lexer.source`'s position to end of file.
                lexer.source.advance_to_end();

                // Avoid lint errors if `eof_handler` contains `return` statement
                #[allow(
                    unused_variables,
                    unreachable_code,
                    clippy::diverging_sub_expression,
                    clippy::allow_attributes
                )]
                {
                    let eof_ret = eof_handler();
                    return Err(eof_ret);
                }
            }
        };

        // Match confirmed.
        // Advance `lexer.source`'s position up to `pos`, consuming unmatched bytes.
        // SAFETY: See above about UTF-8 character boundaries invariant.
        lexer.source.set_position(pos);

        return Ok(byte);
    }
}

/// Search for bytes matching a table with conditional logic.
/// The `continue_if` function is called with the matched byte and position.
/// If it returns true, the search continues. If false, the search stops.
/// Returns the matched byte, or calls the eof_handler if EOF is reached.
pub fn byte_search_with_continue<T>(
    lexer: &mut Lexer<'_>,
    table: &SafeByteMatchTable,
    mut continue_if: impl FnMut(u8, SourcePosition<'_>) -> bool,
    eof_handler: impl FnOnce() -> T,
) -> Result<u8, T> {
    byte_search_from_pos_with_continue(
        lexer,
        table,
        lexer.source.position(),
        continue_if,
        eof_handler,
    )
}

/// Search for bytes matching a table with conditional logic, starting from the specified position.
/// The `continue_if` function is called with the matched byte and position.
/// If it returns true, the search continues. If false, the search stops.
/// Returns the matched byte, or calls the eof_handler if EOF is reached.
pub fn byte_search_from_pos_with_continue<T>(
    lexer: &mut Lexer<'_>,
    table: &SafeByteMatchTable,
    start_pos: SourcePosition<'_>,
    mut continue_if: impl FnMut(u8, SourcePosition<'_>) -> bool,
    eof_handler: impl FnOnce() -> T,
) -> Result<u8, T> {
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

                // EOF.
                // Advance `lexer.source`'s position to end of file.
                lexer.source.advance_to_end();

                // Avoid lint errors if `eof_handler` contains `return` statement
                #[allow(
                    unused_variables,
                    unreachable_code,
                    clippy::diverging_sub_expression,
                    clippy::allow_attributes
                )]
                {
                    let eof_ret = eof_handler();
                    return Err(eof_ret);
                }
            }
        };

        // Found match. Check if should continue.
        if continue_if(byte, pos) {
            // Not a match after all - continue searching.
            // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
            // See above about UTF-8 character boundaries invariant.
            pos = unsafe { pos.add(1) };
            continue;
        }

        // Match confirmed.
        // Advance `lexer.source`'s position up to `pos`, consuming unmatched bytes.
        // SAFETY: See above about UTF-8 character boundaries invariant.
        lexer.source.set_position(pos);

        return Ok(byte);
    }
}
