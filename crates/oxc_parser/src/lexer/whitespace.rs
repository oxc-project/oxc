use super::{
    Kind, Lexer,
    search::{SEARCH_BATCH_SIZE, SafeByteMatchTable, safe_byte_match_table},
};

static NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n'));

impl Lexer<'_> {
    pub(super) fn line_break_handler(&mut self) -> Kind {
        self.token.set_is_on_new_line(true);
        self.trivia_builder.handle_newline();

        // Indentation is common after a line break.
        // Consume it, along with any further line breaks.
        // Irregular line breaks and whitespace are not consumed.
        // They're uncommon, so leave them for the next call to `handle_byte` to take care of.
        {
            // Inlined byte_search! macro
            #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
            NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE.use_table();

            let mut pos = self.source.position();
            // Silence warnings if macro called in unsafe code
            #[allow(unused_unsafe, clippy::unnecessary_safety_comment, clippy::allow_attributes)]
            'outer: loop {
                let byte = if pos.can_read_batch_from(&self.source) {
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
                    let batch = unsafe { pos.slice(SEARCH_BATCH_SIZE) };
                    'inner: loop {
                        for (i, &byte) in batch.iter().enumerate() {
                            if NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE.matches(byte) {
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
                        // SAFETY: `$pos` is before or equal to end of source
                        let remaining = unsafe {
                            let remaining_len = self.source.end().offset_from(pos);
                            pos.slice(remaining_len)
                        };
                        for (i, &byte) in remaining.iter().enumerate() {
                            if NOT_REGULAR_WHITESPACE_OR_LINE_BREAK_TABLE.matches(byte) {
                                // SAFETY: `i` is less than number of bytes remaining after `$pos`,
                                // so `$pos + i` cannot be out of bounds
                                pos = unsafe { pos.add(i) };
                                break 'inner byte;
                            }
                        }

                        // EOF.
                        // Advance `lexer.source`'s position to end of file.
                        self.source.advance_to_end();

                        // Avoid lint errors if `$eof_handler` contains `return` statement
                        #[allow(
                            unused_variables,
                            unreachable_code,
                            clippy::diverging_sub_expression,
                            clippy::allow_attributes
                        )]
                        {
                            let eof_ret = 0; // Fall through to below
                            break 'outer eof_ret;
                        }
                    }
                };

                // Found match. Check if should continue.
                if false {
                    // continue_if: (byte, pos) false
                    // Not a match after all - continue searching.
                    // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                    // See above about UTF-8 character boundaries invariant.
                    pos = unsafe { pos.add(1) };
                    continue;
                }

                // Match confirmed.
                // Advance `lexer.source`'s position up to `$pos`, consuming unmatched bytes.
                // SAFETY: See above about UTF-8 character boundaries invariant.
                self.source.set_position(pos);

                break byte;
            }
        };

        Kind::Skip
    }
}
