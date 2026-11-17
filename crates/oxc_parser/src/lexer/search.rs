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
///       handle_eof: {},
///     };
///   }
/// }
/// ```
#[repr(C, align(64))]
pub struct SafeByteMatchTable {
    /// The pattern, plus NUL.
    ///
    /// `byte_search!` searches with this table, so that a search always stops at the NUL padding
    /// which follows the source text, and so never reads past the padding.
    ///
    /// Must be the first field. `byte_search!`'s loop indexes into this table once per byte,
    /// and a table at offset 0 can be indexed without adding a displacement to the address.
    search: [bool; 256],
    /// The pattern.
    table: [bool; 256],
}

impl SafeByteMatchTable {
    // Create new `SafeByteMatchTable`.
    pub const fn new(bytes: [bool; 256]) -> Self {
        let mut table = Self { search: [false; 256], table: [false; 256] };

        // Check if contains either:
        // 1. `true` for all byte values 192..248
        // 2. `false` for all byte values 128..192
        let mut unicode_start_all_match = true;
        let mut unicode_cont_all_no_match = true;

        let mut i = 0;
        loop {
            let matches = bytes[i];
            table.table[i] = matches;
            table.search[i] = matches || i == 0;

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

    /// Declare that using this table for searching is safe.
    /// `byte_search!` calls `.use_table()` on the table it is provided.
    #[expect(clippy::unused_self)]
    #[inline]
    pub const fn use_table(&self) {}

    /// Test a value against this `SafeByteMatchTable`.
    #[inline]
    pub const fn matches(&self, b: u8) -> bool {
        self.table[b as usize]
    }

    /// Test a value against this `SafeByteMatchTable`, also matching NUL.
    /// `byte_search!` uses this to find the end of the source text (see `search` field).
    #[inline]
    pub const fn matches_or_nul(&self, b: u8) -> bool {
        self.search[b as usize]
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
///
/// Source text is followed by at least `SEARCH_BATCH_SIZE` NUL bytes of padding
/// (see `Source`), so a batch can be read from any position in the source without checking
/// how many bytes remain. The search also stops on NUL bytes, so it always stops at the padding.
/// End of source is only tested for when a NUL byte is found.
///
/// How the end of source is reported depends on whether the table matches NUL
/// (this is checked at compile time):
///
/// * Table does not match NUL: `handle_eof` is required, and is evaluated if end of source
///   is reached. NUL bytes within the source text are skipped, like any other non-matching byte.
/// * Table matches NUL: `handle_eof` and `continue_if` cannot be provided. If end of source is
///   reached, the search stops there, and the macro evaluates to `0` - the first byte of the padding.
///   The caller cannot distinguish that from a NUL byte in the source text without checking
///   `lexer.source.is_eof()`, but usually has no need to, as either one ends what was being consumed.
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
/// or, if the table matches NUL, omit `handle_eof`:
///
/// ```rust,ignore
/// static NOT_SPACE_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| b != b' ');
///
/// impl<'a> Lexer<'a> {
///   fn eat_spaces(&mut self) -> u8 {
///     // `lexer.source` is positioned on first byte which isn't a space, or at EOF.
///     // Evaluates to that byte, or `0` if at EOF.
///     byte_search! {
///       lexer: self,
///       table: NOT_SPACE_TABLE,
///     }
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
/// makes the internal calls to `LexerSource`'s unsafe methods sound.
macro_rules! byte_search {
    // Table which matches NUL.
    // `start` is calculated from current position of `lexer.source`.
    (
        lexer: $lexer:ident,
        table: $table:ident $(,)?
    ) => {{
        let start = $lexer.source.position();
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: start,
        }
    }};

    // Table which matches NUL, with provided `start` position
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident $(,)?
    ) => {{
        const {
            assert!($table.matches(0), "`handle_eof` is required when table does not match NUL");
        }
        byte_search! {
            @search
            lexer: $lexer,
            table: $table,
            start: $start,
            continue_if: (byte, pos) false,
            // Never evaluated, because table matches NUL
            handle_eof: unreachable!(),
        }
    }};

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

    // With both `start` and `continue_if`
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident,
        continue_if: ($byte:ident, $pos:ident) $should_continue:expr,
        handle_eof: $eof_handler:expr,
    ) => {{
        // If table matched NUL, `handle_eof` would never be evaluated, and `continue_if` would be
        // evaluated at end of source, where continuing the search would read out of bounds
        const {
            assert!(
                !$table.matches(0),
                "`handle_eof` and `continue_if` cannot be used with a table which matches NUL"
            );
        }
        byte_search! {
            @search
            lexer: $lexer,
            table: $table,
            start: $start,
            continue_if: ($byte, $pos) $should_continue,
            handle_eof: $eof_handler,
        }
    }};

    // Actual implementation.
    // The arms above check at compile time that either table matches NUL, or `handle_eof`
    // is provided - not both.
    (
        @search
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
            // Search a batch of `SEARCH_BATCH_SIZE` bytes.
            //
            // SAFETY: `$pos` is at or before end of source (see below), and source is followed by
            // at least `SEARCH_BATCH_SIZE` NUL bytes of padding, so can read a whole batch
            // from `$pos` without going out of bounds.
            let batch = unsafe { $pos.slice(crate::lexer::search::SEARCH_BATCH_SIZE) };

            // `'inner: loop {}` is not a real loop - it always exits on first turn.
            // Only using `loop {}` so that can use `break 'inner` to get out of it.
            // This allows complex logic of `$should_continue` and `$eof_handler` to be
            // outside the `for` loop, keeping it as minimal as possible, to encourage
            // compiler to unroll it.
            let $byte = 'inner: loop {
                for (i, &byte) in batch.iter().enumerate() {
                    // `matches_or_nul` matches NUL, even if the table itself doesn't
                    if $table.matches_or_nul(byte) {
                        // SAFETY: No byte in batch before this one is NUL, so those bytes are
                        // not padding, so `$pos + i` is at or before end of source.
                        // Also see above about UTF-8 character boundaries invariant.
                        $pos = unsafe { $pos.add(i) };
                        break 'inner byte;
                    }
                }
                // No match in batch - search next batch.
                // SAFETY: No byte in batch is NUL, so none of batch is padding,
                // so `$pos + SEARCH_BATCH_SIZE` is at or before end of source.
                // Also see above about UTF-8 character boundaries invariant.
                $pos = unsafe { $pos.add(crate::lexer::search::SEARCH_BATCH_SIZE) };
                continue 'outer;
            };

            // If the table matches NUL, there's nothing to do for EOF. Search has stopped on
            // 1st byte of padding, and caller handles NUL like any other matching byte.
            // Condition is evaluated at compile time, so this block is removed for those tables.
            if const { !$table.matches(0) } && $byte == 0 {
                if $pos.is_end_of(&$lexer.source) {
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
                // NUL byte in the source text, which the table does not match - continue searching.
                // SAFETY: `$pos` is not at end of source, so safe to advance 1 byte.
                // NUL is ASCII, so after it is a UTF-8 character boundary.
                $pos = unsafe { $pos.add(1) };
                continue;
            }

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
