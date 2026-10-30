//! Structs and macros for searching source for combinations of byte values.
//!
//! * `SafeByteMatchTable` is a lookup table type for byte values.
//! * The `safe_byte_match_table!` macro creates those tables at compile time.
//! * The `byte_search!` macro searches source text for the first byte matching a
//!   `SafeByteMatchTable`.

/// Batch size for searching
pub const SEARCH_BATCH_SIZE: usize = 16;

/// Safe byte matcher lookup table.
///
/// Create table at compile time as a `static` or `const` with `safe_byte_match_table!` macro.
/// Test bytes against table with `SafeByteMatchTable::matches`.
/// Or use `byte_search!` macro to search for first matching byte in source.
///
/// `byte_search!` using this table is guaranteed to leave `lexer.source` positioned on a UTF-8
/// character boundary, provided that its starting position is already on a UTF-8 character
/// boundary.
///
/// To make this guarantee, one of the following must be true:
///
/// 1. Table contains `true` for all byte values 192 - 247
///    i.e. first byte of any multi-byte Unicode character matches.
///    (NB: 248 - 255 cannot occur in UTF-8 strings)
///    e.g.
///    * `safe_byte_match_table!(|b| b >= 192)`
///    * `safe_byte_match_table!(|b| !b.is_ascii())`
///
/// 2. Table contains `false` for all byte values 128 - 191
///    i.e. the continuation bytes of any multi-byte Unicode chars will be consumed in full.
///    e.g.
///    * `safe_byte_match_table!(|b| b < 128 || b >= 192)`
///    * `safe_byte_match_table!(|b| b.is_ascii())`
///    * `safe_byte_match_table!(|b| b == ' ' || b == '\t')`
///
/// This is statically checked by `SafeByteMatchTable::new`, and will fail to compile if match
/// pattern does not satisfy one of the above.
///
/// # Examples
/// ```rust,ignore
/// use crate::lexer::search::{SafeByteMatchTable, safe_byte_match_table};
///
/// static NOT_ASCII: SafeByteMatchTable = safe_byte_match_table!(|b| !b.is_ascii());
/// assert_eq!(NOT_ASCII.matches(b'X'), false);
/// assert_eq!(NOT_ASCII.matches(192), true);
///
/// impl<'a> Lexer<'a> {
///   fn eat_ascii(&mut self) {
///     // NB: Using `byte_search!` macro with a `SafeByteMatchTable` is safe
///     byte_search! {
///       lexer: self,
///       table: NOT_ASCII,
///       handle_match: |matched_byte, start| {},
///       handle_eof: |start| {},
///     };
///   }
/// }
/// ```
#[repr(C, align(64))]
pub struct SafeByteMatchTable {
    bytes: [bool; 256],
    // A nonzero delimiter identifies an exact string-end table. Other tables keep scalar scanning.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
    string_delimiter: u8,
}

impl SafeByteMatchTable {
    #[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
    #[inline]
    pub(super) const fn is_string_end_table(&self) -> bool {
        self.string_delimiter != 0
    }

    /// Return the first string delimiter, escape, or line break, or 16 if none is present.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
    #[inline(always)]
    pub(super) fn first_string_end_in_batch(&self, batch: &[u8; 16]) -> usize {
        use std::arch::aarch64::{
            vceqq_u8, vdupq_n_u8, vget_lane_u64, vld1q_u8, vorrq_u8, vreinterpret_u64_u8,
            vreinterpretq_u16_u8, vshrn_n_u16,
        };

        const { assert!(SEARCH_BATCH_SIZE == 16) };
        debug_assert!(self.is_string_end_table());
        // SAFETY: The target supports NEON, and `batch` contains 16 initialized bytes.
        unsafe {
            let bytes = vld1q_u8(batch.as_ptr());
            let ends = vorrq_u8(
                vceqq_u8(bytes, vdupq_n_u8(self.string_delimiter)),
                vceqq_u8(bytes, vdupq_n_u8(b'\\')),
            );
            let lines =
                vorrq_u8(vceqq_u8(bytes, vdupq_n_u8(b'\n')), vceqq_u8(bytes, vdupq_n_u8(b'\r')));
            // Each matching byte is 0xFF. Narrow pairs into four bits per input byte,
            // then count zero bits to locate the first match on a little-endian target.
            let nibbles = vshrn_n_u16::<4>(vreinterpretq_u16_u8(vorrq_u8(ends, lines)));
            (vget_lane_u64::<0>(vreinterpret_u64_u8(nibbles)).trailing_zeros() / 4) as usize
        }
    }

    // Create new `SafeByteMatchTable`.
    pub const fn new(bytes: [bool; 256]) -> Self {
        let mut table = Self {
            bytes: [false; 256],
            #[cfg(all(
                target_arch = "aarch64",
                target_feature = "neon",
                target_endian = "little"
            ))]
            string_delimiter: 0,
        };

        // Check if contains either:
        // 1. `true` for all byte values 192..248
        // 2. `false` for all byte values 128..192
        let mut unicode_start_all_match = true;
        let mut unicode_cont_all_no_match = true;

        let mut i = 0;
        loop {
            let matches = bytes[i];
            table.bytes[i] = matches;

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

        // Classify the two string-end tables at compile time. Exact equality matters:
        // the SIMD search must preserve the UTF-8 guarantees of the original table.
        #[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
        {
            let mut double_quote = true;
            let mut single_quote = true;
            let mut index = 0;
            while index < 256 {
                // The loop only visits byte values 0 through 255.
                #[expect(clippy::cast_possible_truncation)]
                let byte = index as u8;
                let special = matches!(byte, b'\\' | b'\r' | b'\n');
                double_quote &= bytes[index] == (special || byte == b'"');
                single_quote &= bytes[index] == (special || byte == b'\'');
                index += 1;
            }
            table.string_delimiter = if double_quote {
                b'"'
            } else if single_quote {
                b'\''
            } else {
                0
            };
        }

        table
    }

    /// Declare that using this table for searching is safe.
    /// `byte_search!` calls `.use_table()` on the table it is provided.
    #[expect(clippy::unused_self)]
    #[inline]
    pub const fn use_table(&self) {}

    /// Test a value against this `SafeByteMatchTable`.
    #[inline]
    pub const fn matches(&self, b: u8) -> bool {
        self.bytes[b as usize]
    }
}

/// Macro to create a `SafeByteMatchTable` at compile time.
///
/// `safe_byte_match_table!(|b| !b.is_ascii())` expands to:
///
/// ```rust,ignore
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

/// Macro to search for first byte matching a `SafeByteMatchTable`.
///
/// Search processes source in batches of `SEARCH_BATCH_SIZE` bytes for speed.
/// When not enough bytes remaining in source for a batch, search source byte by byte.
///
/// This is a macro rather than a function because searching is a bit faster when all the code
/// is in a single function, and some parts (e.g. `continue_if`) can be statically removed by
/// the compiler if they're not used.
///
/// Used as follows:
///
/// ```rust,ignore
/// static NOT_STUFF_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| !is_stuff(b));
///
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         // Evaluate to a `u8` which macro call will evaluate to.
///         0xFF
///         // Or can `return` from enclosing function e.g. `return false;`
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table
///     // (or `0xFF` if EOF, because `handle_eof` evaluates to `0xFF`).
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// or provide the `SourcePosition` to start searching from:
///
/// ```rust,ignore
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let start = unsafe { self.source.position().add(1) };
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       start: start,
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         return false;
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table.
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// Can also add a block to decide whether to continue searching for some matches:
///
/// ```rust,ignore
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       continue_if: (matched_byte, pos) {
///         // Matching byte found. Decide whether it's really a match.
///         // Return `true` to continue searching, or `false` to end search.
///         // NB: `lexer.source` has NOT been updated at this point.
///         if matched_byte == 0xE2 {
///           // Only match a specific Unicode char (in this case 0xE2, 0x80, 0xA8)
///           // NB: We don't need to check if `pos` is at EOF here, as 0xE2 is always 1st byte
///           // of a 3-byte Unicode char, but if matching an ASCII char, would need to make sure
///           // don't read out of bounds.
///           unsafe { pos.add(1).read2() != [0x80, 0xA8] }
///         } else {
///           // End search for all other possibilities
///           false
///         }
///       },
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         return false;
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table.
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// # SAFETY
///
/// This macro consumes bytes from `lexer.source` according to the provided `SafeByteMatchTable`.
///
/// The `start` position must be on a UTF-8 character boundary. The overloads without an explicit
/// `start` use the current position of `lexer.source`.
///
/// Using `byte_search!` with the provided `SafeByteMatchTable` is guaranteed to leave
/// `lexer.source` positioned on a UTF-8 character boundary when entering `handle_match`, which
/// makes the internal calls to `Source`'s unsafe methods sound.
macro_rules! byte_search {
    // Simple version.
    // `start` is calculated from current position of `lexer.source`.
    (
        lexer: $lexer:ident,
        table: $table:ident,
        handle_eof: $eof_handler:expr,
    ) => {{
        let start = $lexer.source.position();
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: start,
            continue_if: (byte, pos) false,
            handle_eof: $eof_handler,
        }
    }};

    // With `continue_if`.
    // `start` is calculated from current position of `lexer.source`.
    (
        lexer: $lexer:ident,
        table: $table:ident,
        continue_if: ($byte:ident, $pos:ident) $should_continue:expr,
        handle_eof: $eof_handler:expr,
    ) => {{
        let start = $lexer.source.position();
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: start,
            continue_if: ($byte, $pos) $should_continue,
            handle_eof: $eof_handler,
        }
    }};

    // With provided `start` position
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident,
        handle_eof: $eof_handler:expr,
    ) => {
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: $start,
            continue_if: (byte, pos) false,
            handle_eof: $eof_handler,
        }
    };

    // Actual implementation - with both `start` and `continue_if`
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident,
        continue_if: ($byte:ident, $pos:ident) $should_continue:expr,
        handle_eof: $eof_handler:expr,
    ) => {{
        // SAFETY:
        // The caller must provide `$start` on a UTF-8 character boundary. Given that precondition,
        // `$table` is always a `SafeByteMatchTable`, so it's guaranteed that `lexer.source` will be
        // positioned on a UTF-8 character boundary before `handle_match` is called. Therefore we
        // can assume this is taken care of and wrap the calls to unsafe functions in this macro
        // with `unsafe {}`.
        #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
        $table.use_table();

        let mut $pos = $start;
        // Silence warnings if macro called in unsafe code
        #[allow(unused_unsafe, clippy::unnecessary_safety_comment, clippy::allow_attributes)]
        'outer: loop {
            let $byte = if $pos.can_read_batch_from(&$lexer.source) {
                // Search a batch of `SEARCH_BATCH_SIZE` bytes.
                //
                // `'inner: loop {}` is not a real loop - it always exits on first turn.
                // Only using `loop {}` so that can use `break 'inner` to get out of it.
                // This allows complex logic of `$should_continue` and `$match_handler` to be
                // outside the `for` loop, keeping it as minimal as possible, to encourage
                // compiler to unroll it.
                //
                // SAFETY:
                // `$pos.can_read_batch_from(&$lexer.source)` check above ensures there are
                // at least `SEARCH_BATCH_SIZE` bytes remaining in `lexer.source`.
                // So `$pos.add()` in this loop cannot go out of bounds.
                let batch = unsafe { $pos.slice(crate::lexer::search::SEARCH_BATCH_SIZE) };
                'inner: loop {
                    #[cfg(all(target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
                    if $table.is_string_end_table() {
                        let index = $table.first_string_end_in_batch(batch.try_into().unwrap());
                        if index < crate::lexer::search::SEARCH_BATCH_SIZE {
                            // SAFETY: `index` is inside the readable batch and identifies
                            // an ASCII delimiter, so it is also a UTF-8 character boundary.
                            $pos = unsafe { $pos.add(index) };
                            // SAFETY: The matched index is inside the readable batch.
                            break 'inner unsafe { $pos.read() };
                        }
                        // SAFETY: The complete batch is readable and contains no match.
                        $pos = unsafe { $pos.add(crate::lexer::search::SEARCH_BATCH_SIZE) };
                        continue 'outer;
                    }
                    for (i, &byte) in batch.iter().enumerate() {
                        if $table.matches(byte) {
                            // SAFETY: Cannot go out of bounds (see above).
                            // Also see above about UTF-8 character boundaries invariant.
                            $pos = unsafe { $pos.add(i) };
                            break 'inner byte;
                        }
                    }
                    // No match in batch - search next batch.
                    // SAFETY: Cannot go out of bounds (see above).
                    // Also see above about UTF-8 character boundaries invariant.
                    $pos = unsafe { $pos.add(crate::lexer::search::SEARCH_BATCH_SIZE) };
                    continue 'outer;
                }
            } else {
                // Not enough bytes remaining for a batch. Process byte-by-byte.
                // Same as above, `'inner: loop {}` is not a real loop here - always exits on first turn.
                'inner: loop {
                    // SAFETY: `$pos` is before or equal to end of source
                    let remaining = unsafe {
                        let remaining_len = $lexer.source.end().offset_from($pos);
                        $pos.slice(remaining_len)
                    };
                    for (i, &byte) in remaining.iter().enumerate() {
                        if $table.matches(byte) {
                            // SAFETY: `i` is less than number of bytes remaining after `$pos`,
                            // so `$pos + i` cannot be out of bounds
                            $pos = unsafe { $pos.add(i) };
                            break 'inner byte;
                        }
                    }

                    // EOF.
                    // Advance `lexer.source`'s position to end of file.
                    $lexer.source.advance_to_end();

                    // Avoid lint errors if `$eof_handler` contains `return` statement
                    #[allow(
                        unused_variables,
                        unreachable_code,
                        clippy::diverging_sub_expression,
                        clippy::allow_attributes
                    )]
                    {
                        let eof_ret = $eof_handler;
                        break 'outer eof_ret;
                    }
                }
            };

            // Found match. Check if should continue.
            if $should_continue {
                // Not a match after all - continue searching.
                // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                // See above about UTF-8 character boundaries invariant.
                $pos = unsafe { $pos.add(1) };
                continue;
            }

            // Match confirmed.
            // Advance `lexer.source`'s position up to `$pos`, consuming unmatched bytes.
            // SAFETY: See above about UTF-8 character boundaries invariant.
            $lexer.source.set_position($pos);

            break $byte;
        }
    }};
}
pub(crate) use byte_search;

#[cfg(all(test, target_arch = "aarch64", target_feature = "neon", target_endian = "little"))]
mod tests {
    use super::{SEARCH_BATCH_SIZE, SafeByteMatchTable};

    #[test]
    fn string_batch_matches_scalar_table() {
        for delimiter in [b'"', b'\''] {
            let mut entries = [false; 256];
            for byte in [delimiter, b'\\', b'\r', b'\n'] {
                entries[byte as usize] = true;
            }
            let table = SafeByteMatchTable::new(entries);
            assert!(table.is_string_end_table());
            for lane in 0..SEARCH_BATCH_SIZE {
                for byte in 0..=u8::MAX {
                    let mut batch = [b'a'; SEARCH_BATCH_SIZE];
                    batch[lane] = byte;
                    let expected = batch
                        .iter()
                        .position(|&byte| table.matches(byte))
                        .unwrap_or(SEARCH_BATCH_SIZE);
                    assert_eq!(table.first_string_end_in_batch(&batch), expected);
                }
            }
            assert_eq!(table.first_string_end_in_batch(&[delimiter; SEARCH_BATCH_SIZE]), 0);

            // A table with any additional match must retain its scalar implementation.
            entries[b'a' as usize] = true;
            assert!(!SafeByteMatchTable::new(entries).is_string_end_table());
        }
    }

    #[test]
    fn string_search_crosses_batch_and_utf8_boundaries() {
        use crate::{UniquePromise, lexer::source::Source};

        struct TestLexer<'a> {
            source: Source<'a>,
        }

        for delimiter in [b'"', b'\''] {
            let mut entries = [false; 256];
            for byte in [delimiter, b'\\', b'\r', b'\n'] {
                entries[byte as usize] = true;
            }
            let table = SafeByteMatchTable::new(entries);
            for padding in 0..=SEARCH_BATCH_SIZE * 3 {
                for content in ["", "a", "é", "中", "😀", "aé中😀"] {
                    for end in [None, Some(delimiter), Some(b'\\'), Some(b'\r'), Some(b'\n')] {
                        let mut text = "a".repeat(padding);
                        text.push_str(content);
                        let expected_offset = text.len();
                        if let Some(byte) = end {
                            text.push(char::from(byte));
                            text.push_str("after the first delimiter");
                            text.push(char::from(delimiter));
                        }
                        // Only one Source is alive at a time on this thread.
                        let mut lexer = TestLexer {
                            source: Source::new(
                                &text,
                                UniquePromise::new_for_tests_and_benchmarks(),
                            ),
                        };
                        let matched = byte_search! {
                            lexer: lexer,
                            table: table,
                            handle_eof: { 0xFF },
                        };
                        assert_eq!(matched, end.unwrap_or(0xFF));
                        assert_eq!(lexer.source.remaining(), &text[expected_offset..]);
                    }
                }
            }
        }
    }
}
