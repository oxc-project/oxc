use std::slice;

use oxc_ast::ast::StringLiteral;
use oxc_data_structures::{assert_unchecked, slice_iter::SliceIter};
use oxc_syntax::{
    identifier::NBSP,
    line_terminator::{LS_LAST_2_BYTES, PS_LAST_2_BYTES},
};
use oxc_wtf8::Wtf8;

use crate::Codegen;

/// Quote character.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Quote {
    Single = b'\'',
    Double = b'"',
    Backtick = b'`',
}

impl Quote {
    #[inline]
    pub fn print(self, codegen: &mut Codegen<'_>) {
        // SAFETY: All variants of `Quote` are ASCII bytes
        unsafe { codegen.code.print_byte_unchecked(self as u8) };
    }
}

impl Codegen<'_> {
    /// Print a [`StringLiteral`].
    pub(crate) fn print_string_literal(&mut self, s: &StringLiteral<'_>, allow_backtick: bool) {
        self.print_property_key_annotation(s.span.start);
        self.add_source_mapping(s.span);
        self.print_wtf8_string_impl(s.value.as_wtf8(), allow_backtick);
    }

    /// Print a [`StringLiteral`] as a template literal, whatever its contents.
    ///
    /// A template literal is never a directive, which a string literal in the first statement position would be.
    /// See `print_directives_and_statements`.
    pub(crate) fn print_string_literal_as_template(&mut self, s: &StringLiteral<'_>) {
        self.print_property_key_annotation(s.span.start);
        self.add_source_mapping(s.span);
        Quote::Backtick.print(self);
        self.print_wtf8_string_body(s.value.as_wtf8(), Some(Quote::Backtick), true);
        Quote::Backtick.print(self);
    }

    pub(super) fn print_string_impl(&mut self, s: &str, allow_backtick: bool) {
        // If `minify` option enabled, quote will be chosen depending on what produces shortest output.
        // What is the best quote to use will be determined when first character needing escape is found.
        // This avoids iterating through the string twice if it contains no quotes (common case).
        // Don't print opening quote now, because we don't know what it is yet.
        //
        // If not in `minify` mode, print the quote requested in options.
        let quote = if self.options.minify {
            None
        } else {
            let quote = self.quote;
            quote.print(self);
            Some(quote)
        };

        self.print_string_body(s, quote, allow_backtick);
    }

    /// Print the contents of a string, and its closing quote.
    ///
    /// `quote` is `None` where it has yet to be chosen - it is then calculated from the contents,
    /// and the opening quote printed, when the first character needing an escape is found.
    fn print_string_body(&mut self, s: &str, quote: Option<Quote>, allow_backtick: bool) {
        // Loop through bytes, looking for any which need to be escaped.
        // String is written to buffer in chunks.
        let bytes = s.as_bytes().iter();
        let mut state = PrintStringState { chunk_start: bytes.ptr(), bytes, quote, allow_backtick };

        // Loop through bytes.
        while let Some(b) = state.peek() {
            // Look up whether byte needs escaping
            let escape = ESCAPES.0[b as usize];
            if escape == Escape::__ {
                // No escape required.
                // SAFETY: We just checked there's a byte to consume.
                // If byte is not ASCII, this will temporarily leave `bytes` iterator not on a UTF-8
                // character boundary, but if so next turns of the loop will consume the rest of
                // the Unicode character.
                // All bytes which produce an `Escape` which isn't `Escape::__` are 1st byte
                // of a UTF-8 character sequence.
                unsafe { state.consume_byte_unchecked() };
            } else {
                // Escape may be required. Execute byte handler.
                // Characters requiring escape are relatively rare, so cold branch.
                cold_branch(|| {
                    // SAFETY: We just peeked a byte, so there is a byte ready to consume.
                    // `escape` corresponds to that byte.
                    // Checked that `escape != Escape::__` above.
                    unsafe { handle_escape(escape, self, &mut state) };
                });
            }
        }

        // Flush any remaining bytes
        state.flush(self);

        // Print closing quote.
        // SAFETY: `flush` calls `calculate_quote` which ensures `state.quote` is `Some`.
        let quote = unsafe { state.quote.unwrap_unchecked() };
        quote.print(self);
    }

    pub(super) fn print_wtf8_string_impl(&mut self, s: &Wtf8, allow_backtick: bool) {
        if let Some(s_str) = s.as_str() {
            self.print_string_impl(s_str, allow_backtick);
            return;
        }
        // Contains lone surrogates – need WTF-8 aware printing
        let quote = if self.options.minify {
            if allow_backtick {
                // Replicate calculate_quote_maybe_backtick logic over CodePoints
                let mut single_cost: isize = 0;
                let mut double_cost: isize = 0;
                let mut backtick_cost: isize = 0;
                let mut iter = s.code_points().peekable();
                while let Some(cp) = iter.next() {
                    if let Some(c) = cp.to_char() {
                        match c {
                            '\n' => backtick_cost -= 1,
                            '\'' => single_cost += 1,
                            '"' => double_cost += 1,
                            '`' => backtick_cost += 1,
                            '$' => {
                                if matches!(
                                    iter.peek().and_then(|a: &oxc_wtf8::CodePoint| {
                                        oxc_wtf8::CodePoint::to_char(*a)
                                    }),
                                    Some('{')
                                ) {
                                    backtick_cost += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if backtick_cost <= double_cost {
                    if backtick_cost <= single_cost { Quote::Backtick } else { Quote::Single }
                } else if double_cost <= single_cost {
                    Quote::Double
                } else {
                    Quote::Single
                }
            } else {
                // No backtick option – count single vs double
                let mut single_cost: isize = 0;
                for cp in s.code_points() {
                    if let Some(c) = cp.to_char() {
                        match c {
                            '\'' => single_cost += 1,
                            '"' => single_cost -= 1,
                            _ => {}
                        }
                    }
                }
                if single_cost < 0 { Quote::Single } else { Quote::Double }
            }
        } else {
            self.quote
        };
        quote.print(self);
        self.print_wtf8_string_body(s, Some(quote), allow_backtick);
        quote.print(self);
    }

    fn print_wtf8_string_body(&mut self, s: &Wtf8, quote: Option<Quote>, _allow_backtick: bool) {
        // Iterate over code points; surrogates are emitted as \uXXXX, valid chars are escaped as needed
        let mut iter = s.code_points().peekable();
        while let Some(cp) = iter.next() {
            if let Some(c) = cp.to_char() {
                // Handle escapes for valid scalar values
                match c {
                    '"' => {
                        if quote == Some(Quote::Double) {
                            self.code.print_str("\\\"");
                        } else {
                            self.code.print_str("\"");
                        }
                    }
                    '\'' => {
                        if quote == Some(Quote::Single) {
                            self.code.print_str("\\'");
                        } else {
                            self.code.print_str("'");
                        }
                    }
                    '\\' => self.code.print_str("\\\\"),
                    '\n' => {
                        if quote == Some(Quote::Backtick) {
                            self.code.print_str("\n");
                        } else {
                            self.code.print_str("\\n");
                        }
                    }
                    '\r' => self.code.print_str("\\r"),
                    '\t' => self.code.print_str("\\t"),
                    '\x08' => self.code.print_str("\\b"),
                    '\x0C' => self.code.print_str("\\f"),
                    '\x0B' => self.code.print_str("\\v"),
                    '\0' => {
                        // Check if next char is digit (0-9) – if so, use \x00, else \0
                        let next_is_digit = iter
                            .peek()
                            .and_then(|a: &oxc_wtf8::CodePoint| oxc_wtf8::CodePoint::to_char(*a))
                            .is_some_and(|next_c| next_c.is_ascii_digit());
                        if next_is_digit {
                            self.code.print_str("\\x00");
                        } else {
                            self.code.print_str("\\0");
                        }
                    }
                    '`' => {
                        if quote == Some(Quote::Backtick) {
                            self.code.print_str("\\`");
                        } else {
                            self.code.print_str("`");
                        }
                    }
                    '$' => {
                        if quote == Some(Quote::Backtick)
                            && matches!(
                                iter.peek().and_then(|a: &oxc_wtf8::CodePoint| {
                                    oxc_wtf8::CodePoint::to_char(*a)
                                }),
                                Some('{')
                            )
                        {
                            self.code.print_str("\\$");
                        } else {
                            self.code.print_str("$");
                        }
                    }
                    '<' => {
                        // Escape `</script` and `<!--` as \x3C
                        // Check if following code points are "/script" or "!--"
                        #[expect(clippy::unused_peekable)]
                        // The lookahead clone only calls `next()`, never `peek()`
                        let mut lookahead = iter.clone();
                        let mut is_script = false;
                        let mut is_comment = false;
                        if let Some(cp1) = lookahead.next().and_then(oxc_wtf8::CodePoint::to_char) {
                            if cp1 == '/' {
                                let s: String = lookahead
                                    .by_ref()
                                    .take(6)
                                    .filter_map(oxc_wtf8::CodePoint::to_char)
                                    .collect();
                                if s.eq_ignore_ascii_case("script") {
                                    is_script = true;
                                }
                            } else if cp1 == '!' {
                                let mut s = String::new();
                                if let Some(cp2) =
                                    lookahead.next().and_then(oxc_wtf8::CodePoint::to_char)
                                {
                                    s.push(cp2);
                                    if let Some(cp3) =
                                        lookahead.next().and_then(oxc_wtf8::CodePoint::to_char)
                                    {
                                        s.push(cp3);
                                        if s == "--" {
                                            is_comment = true;
                                        }
                                    }
                                }
                            }
                        }
                        if is_script || is_comment {
                            self.code.print_str("\\x3C");
                        } else {
                            self.code.print_str("<");
                        }
                    }
                    '\u{2028}' => self.code.print_str("\\u2028"),
                    '\u{2029}' => self.code.print_str("\\u2029"),
                    c if c.is_control() => {
                        self.code.print_str(format!("\\u{:04x}", c as u32));
                    }
                    _ => {
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        self.code.print_str(s);
                    }
                }
            } else {
                // Lone surrogate – emit \uXXXX (lowercase per spec)
                self.code.print_str(format!("\\u{:04x}", cp.to_u32()));
            }
        }
    }
}

/// String printer state.
///
/// Main purpose is to contain `bytes` iterator.
/// This iterator must always be positioned on a UTF-8 character boundary.
struct PrintStringState<'s> {
    chunk_start: *const u8,
    bytes: slice::Iter<'s, u8>,
    quote: Option<Quote>,
    allow_backtick: bool,
}

impl PrintStringState<'_> {
    /// Peek next byte in `bytes` iterator.
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.bytes.peek_copy()
    }

    /// Advance the `bytes` iterator by 1 byte.
    ///
    /// # SAFETY
    ///
    /// * There must be at least 1 more byte in the `bytes` iterator.
    /// * After this call, `bytes` iterator must be left on a UTF-8 character boundary
    ///   (i.e. the current byte is ASCII), or if byte is not ASCII, then further calls to
    ///   `consume_byte_unchecked` / `consume_bytes_unchecked` consume the rest of the Unicode character
    ///   before calling other methods e.g. `flush`.
    #[inline]
    unsafe fn consume_byte_unchecked(&mut self) {
        // SAFETY: Caller guarantees there is a byte to consume in `bytes` iterator,
        // and that consuming it leaves the iterator on a UTF-8 char boundary
        unsafe { self.bytes.next_unchecked() };
    }

    /// Advance the `bytes` iterator by `count` bytes.
    ///
    /// # SAFETY
    ///
    /// * There must be at least `count` more bytes in the `bytes` iterator.
    /// * After this call, `bytes` iterator must be left on a UTF-8 character boundary.
    #[inline]
    unsafe fn consume_bytes_unchecked(&mut self, count: usize) {
        // SAFETY: Caller guarantees there are `count` bytes to consume in `bytes` iterator,
        // and that consuming them leaves the iterator on a UTF-8 char boundary.
        unsafe { self.bytes.advance_unchecked(count) };
    }

    /// Set the start of next chunk to be current position of `bytes` iterator.
    #[inline]
    fn start_chunk(&mut self) {
        self.chunk_start = self.bytes.ptr();
    }

    /// Flush current chunk to buffer, consume 1 byte, and start next chunk after that byte.
    ///
    /// # SAFETY
    ///
    /// * There must be at least 1 more byte in the `bytes` iterator.
    /// * After this call, `bytes` iterator must be left on a UTF-8 character boundary.
    ///   i.e. the current byte is ASCII.
    #[inline]
    unsafe fn flush_and_consume_byte(&mut self, codegen: &mut Codegen) {
        // SAFETY: Caller guarantees `flush_and_consume_bytes`'s requirements are met
        unsafe { self.flush_and_consume_bytes(codegen, 1) };
    }

    /// Flush current chunk to buffer, consume `count` bytes, and start next chunk after those bytes.
    ///
    /// # SAFETY
    ///
    /// * There must be at least `count` more bytes in the `bytes` iterator.
    /// * After this call, `bytes` iterator must be left on a UTF-8 character boundary.
    #[inline]
    unsafe fn flush_and_consume_bytes(&mut self, codegen: &mut Codegen, count: usize) {
        self.flush(codegen);

        // SAFETY: Caller guarantees there are `count` bytes to consume in `bytes` iterator,
        // and that consuming them leaves the iterator on a UTF-8 char boundary
        unsafe { self.consume_bytes_unchecked(count) };

        self.start_chunk();
    }

    /// Flush all bytes from `chunk_start` up to current position of `bytes` iterator into buffer.
    ///
    /// If what quote character to use has not been decided yet, calculate the best quote character,
    /// and print it before flushing.
    fn flush(&mut self, codegen: &mut Codegen) {
        // If which quote character to use is not already known, calculate it and print opening quote
        self.calculate_quote(codegen);

        // SAFETY: `chunk_start` is pointer to current position of `bytes` iterator at some point,
        // and the iterator only advances, so current position of `bytes` must be on or after `chunk_start`
        let len = unsafe {
            let bytes_ptr = self.bytes.ptr();
            bytes_ptr.offset_from_unsigned(self.chunk_start)
        };

        // SAFETY: `chunk_start` is within bounds of original `&str`.
        // `bytes` iter cannot go past end of `&str` either.
        // So a slice of `len` bytes starting at `chunk_start` must be within bounds of the `&str`.
        // `bytes` iterator is always positioned on a UTF-8 character boundary, as is `chunk_start`.
        // Therefore the slice between these two must be a valid UTF-8 string.
        unsafe {
            let slice = slice::from_raw_parts(self.chunk_start, len);
            codegen.code.print_bytes_unchecked(slice);
        }
    }

    /// Calculate optimum quote character to use, and print that quote (as opening quote).
    ///
    /// Actual logic in separate `calculate_quote_impl` method, so that `calculate_quote` itself
    /// is inlined, to create a fast path for when `quote` is `Some`.
    #[inline]
    fn calculate_quote(&mut self, codegen: &mut Codegen) -> Quote {
        if let Some(quote) = self.quote { quote } else { self.calculate_quote_impl(codegen) }
    }

    fn calculate_quote_impl(&mut self, codegen: &mut Codegen) -> Quote {
        let quote = if self.allow_backtick {
            self.calculate_quote_maybe_backtick()
        } else {
            self.calculate_quote_no_backtick()
        };

        quote.print(codegen);

        self.quote = Some(quote);

        quote
    }

    /// Calculate optimum quote character to use, when backtick (`) is an option.
    fn calculate_quote_maybe_backtick(&self) -> Quote {
        // Max string length is:
        // * 64-bit platforms: `u32::MAX`.
        // * 32-bit platforms: `i32::MAX`.
        // In either case, `isize` is sufficient to make overflow impossible.
        let mut single_cost: isize = 0;
        let mut double_cost: isize = 0;
        let mut backtick_cost: isize = 0;
        let mut bytes = self.bytes.clone();
        while let Some(b) = bytes.next() {
            match b {
                b'\n' => backtick_cost -= 1,
                b'\'' => single_cost += 1,
                b'"' => double_cost += 1,
                b'`' => backtick_cost += 1,
                b'$' if bytes.peek() == Some(&b'{') => {
                    backtick_cost += 1;
                }
                _ => {}
            }
        }

        // If equal cost for different quotes prefer, in order:
        // 1. Backtick
        // 2. Double quote
        // 3. Single quote
        #[rustfmt::skip]
        let quote = if backtick_cost <= double_cost {
            if backtick_cost <= single_cost {
                Quote::Backtick
            } else {
                Quote::Single
            }
        } else if double_cost <= single_cost {
            Quote::Double
        } else {
            Quote::Single
        };
        quote
    }

    /// Calculate optimum quote character to use, when backtick (`) is not an option.
    fn calculate_quote_no_backtick(&self) -> Quote {
        // Max string length is:
        // * 64-bit platforms: `u32::MAX`.
        // * 32-bit platforms: `i32::MAX`.
        // In either case, `isize` is sufficient to make overflow impossible.
        let mut single_cost: isize = 0;
        for &b in self.bytes.clone() {
            match b {
                b'\'' => single_cost += 1,
                b'"' => single_cost -= 1,
                _ => {}
            }
        }

        // Prefer double quote over single quote if cost is the same
        if single_cost < 0 { Quote::Single } else { Quote::Double }
    }
}

/// Convert `char` to UTF-8 bytes array.
const fn to_bytes<const N: usize>(ch: char) -> [u8; N] {
    assert!(ch.len_utf8() == N);
    let mut bytes = [0u8; N];
    ch.encode_utf8(&mut bytes);
    bytes
}

/// `NBSP` character as UTF-8 bytes.
const NBSP_BYTES: [u8; 2] = to_bytes(NBSP);
const _: () = assert!(NBSP_BYTES[0] == 0xC2);
const NBSP_LAST_BYTE: u8 = NBSP_BYTES[1];

/// Escape codes.
///
/// Discriminant - 1 is used as index into `BYTE_HANDLERS` (except for `__` variant).
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Escape {
    __ = 0,  // No escape required
    NU = 1,  // \x00  - Null byte
    BE = 2,  // \x07  - Bell
    BK = 3,  // \b    - Backspace
    VT = 4,  // \v    - Vertical tab
    FF = 5,  // \f    - Form feed
    NL = 6,  // \n    - New line
    CR = 7,  // \r    - Carriage return
    ES = 8,  // \x1B  - Escape
    BS = 9,  // \\    - Backslash
    SQ = 10, // '     - Single quote
    DQ = 11, // "     - Double quote
    BQ = 12, // `     - Backtick quote
    DO = 13, // $     - Dollar sign
    LT = 14, // <     - Less-than sign
    LS = 15, // LS/PS - U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR (first byte)
    NB = 16, // NBSP  - Non-breaking space (first byte)
}

/// Struct which ensures content is aligned on 128.
#[repr(C, align(128))]
struct Aligned128<T>(T);

/// Table mapping bytes to `Escape`s.
///
/// Aligned on 128, so top half (ASCII chars) occupies a pair of L1 cache lines.
/// Bottom half (non-ASCII chars) also occupies a pair of L1 cache lines,
/// but will not be accessed for strings which only contain ASCII (common case).
static ESCAPES: Aligned128<[Escape; 256]> = {
    #[allow(clippy::enum_glob_use, clippy::allow_attributes)]
    use Escape::*;
    Aligned128([
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        NU, __, __, __, __, __, __, BE, BK, __, NL, VT, FF, CR, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, ES, __, __, __, __, // 1
        __, __, DQ, __, DO, __, __, SQ, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, LT, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        BQ, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, NB, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, LS, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ])
};

type ByteHandler = unsafe fn(&mut Codegen, &mut PrintStringState);

/// Byte handlers.
///
/// Indexed by `escape as usize - 1` (where `escape` is not `Escape::__`).
/// Must be in same order as discriminants in `Escape`.
///
/// Function pointers are 8 bytes each, so `BYTE_HANDLERS` is 136 bytes in total.
/// Aligned on 128, so first 16 occupy a pair of L1 cache lines.
/// The last will be in separate cache line, but it should be vanishingly rare that it's accessed.
static BYTE_HANDLERS: Aligned128<[ByteHandler; 16]> = Aligned128([
    print_null,
    print_bell,
    print_backspace,
    print_vertical_tab,
    print_form_field,
    print_new_line,
    print_carriage_return,
    print_escape,
    print_backslash,
    print_single_quote,
    print_double_quote,
    print_backtick,
    print_dollar,
    print_less_than,
    print_ls_or_ps,
    print_non_breaking_space,
]);

/// Call byte handler for byte which needs escaping.
///
/// # SAFETY
///
/// * There must be another byte to consume in `bytes` iterator.
/// * `escape` must correspond to the next byte.
/// * `escape` must not be `Escape::__`.
unsafe fn handle_escape(escape: Escape, codegen: &mut Codegen, state: &mut PrintStringState) {
    // Inform compiler that `escape` is not `Escape::__`.
    // This removes the bounds check from `BYTE_HANDLERS.0[escape as usize - 1]`.
    // SAFETY: Caller guarantees `escape` is not `Escape::__`.
    unsafe { assert_unchecked!(escape != Escape::__) };

    let byte_handler = BYTE_HANDLERS.0[escape as usize - 1];
    // SAFETY: Caller guarantees there is a byte to consume in `bytes` iterator,
    // and that `escape` corresponds to the next byte, so we are calling the correct byte handler
    unsafe { byte_handler(codegen, state) };
}

// Byte handlers for bytes which need escaping.
//
// # SAFETY
//
// All byte handlers have safety invariants:
// * There must be at least 1 byte remaining in `bytes` iterator.
// * The byte handler is only called for the byte it expects.

// \x00
unsafe fn print_null(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x00));

    // SAFETY: Next byte is `\x00`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    if state.peek().is_some_and(|b| b.is_ascii_digit()) {
        codegen.print_str("\\x00");
    } else {
        codegen.print_str("\\0");
    }
}

// \x07
unsafe fn print_bell(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x07));
    // SAFETY: Next byte is `\x07`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\x07");
}

// \b
unsafe fn print_backspace(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x08));
    // SAFETY: Next byte is `\b`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\b");
}

// \v
unsafe fn print_vertical_tab(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x0B));
    // SAFETY: Next byte is `\v`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\v");
}

// \f
unsafe fn print_form_field(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x0C));
    // SAFETY: Next byte is `\f`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\f");
}

// \n
unsafe fn print_new_line(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'\n'));

    if state.calculate_quote(codegen) == Quote::Backtick {
        // No need to escape.
        // SAFETY: Next byte is `\n`, which is ASCII.
        unsafe { state.consume_byte_unchecked() };
    } else {
        // SAFETY: Next byte is `\n`, which is ASCII
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\n");
    }
}

// \r
unsafe fn print_carriage_return(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'\r'));
    // SAFETY: Next byte is `\r`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\r");
}

// \x1B
unsafe fn print_escape(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0x1B));
    // SAFETY: Next byte is `\x1B`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\x1B");
}

// \
unsafe fn print_backslash(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'\\'));
    // SAFETY: Next byte is `\`, which is ASCII
    unsafe { state.flush_and_consume_byte(codegen) };
    codegen.print_str("\\\\");
}

// '
unsafe fn print_single_quote(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'\''));

    if state.calculate_quote(codegen) == Quote::Single {
        // SAFETY: Next byte is `'`, which is ASCII
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\'");
    } else {
        // No need to escape.
        // SAFETY: Next byte is `'`, which is ASCII.
        unsafe { state.consume_byte_unchecked() };
    }
}

// "
unsafe fn print_double_quote(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'"'));

    if state.calculate_quote(codegen) == Quote::Double {
        // SAFETY: Next byte is `"`, which is ASCII
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\\"");
    } else {
        // No need to escape.
        // SAFETY: Next byte is `"`, which is ASCII.
        unsafe { state.consume_byte_unchecked() };
    }
}

// `
unsafe fn print_backtick(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'`'));

    if state.calculate_quote(codegen) == Quote::Backtick {
        // SAFETY: Next byte is `, which is ASCII
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\`");
    } else {
        // No need to escape.
        // SAFETY: Next byte is `, which is ASCII.
        unsafe { state.consume_byte_unchecked() };
    }
}

// $
unsafe fn print_dollar(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'$'));

    // Note: Check next byte is `{` first, to avoid calculating quote unless have to
    let next = state.bytes.as_slice().get(1);
    if next == Some(&b'{') && state.calculate_quote(codegen) == Quote::Backtick {
        // SAFETY: Next byte is `$`, which is ASCII
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\$");
    } else {
        // No need to escape.
        // SAFETY: Next byte is `$`, which is ASCII.
        unsafe { state.consume_byte_unchecked() };
    }
}

// <
unsafe fn print_less_than(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(b'<'));

    // Get slice of remaining bytes, including leading `<`
    let slice = state.bytes.as_slice();

    // SAFETY: Next byte is `<`, which is ASCII
    unsafe { state.consume_byte_unchecked() };

    if slice.len() >= 8 && is_script_close_tag(&slice[0..8]) {
        // Flush up to and including `<`. Skip `/`. Write `\/` instead. Then skip over `script`.
        // Next chunk starts with `script`.
        // SAFETY: We already consumed `<`. Next byte is `/`, which is ASCII.
        unsafe { state.flush_and_consume_byte(codegen) };
        codegen.print_str("\\/");
        // SAFETY: The check above ensures there are 6 bytes left, after consuming 2 already.
        // `script` / `SCRIPT` is all ASCII bytes, so skipping them leaves `bytes` iterator
        // positioned on UTF-8 char boundary.
        unsafe { state.consume_bytes_unchecked(6) };
    }
}

// 0xE2 - first byte of <LS> or <PS>
unsafe fn print_ls_or_ps(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0xE2));

    let next2: [u8; 2] = {
        // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
        // so there must be 2 more bytes available to consume
        let next2 = unsafe { state.bytes.as_slice().get_unchecked(1..3) };
        next2.try_into().unwrap()
    };

    let replacement = match next2 {
        LS_LAST_2_BYTES => "\\u2028",
        PS_LAST_2_BYTES => "\\u2029",
        _ => {
            // Some other character starting with 0xE2. Advance past it.
            // SAFETY: 0xE2 is always the start of a 3-byte Unicode character
            unsafe { state.consume_bytes_unchecked(3) };
            return;
        }
    };

    // SAFETY: 0xE2 is always the start of a 3-byte Unicode character
    unsafe { state.flush_and_consume_bytes(codegen, 3) };
    codegen.print_str(replacement);
}

// 0xC2 - first byte of <NBSP>
unsafe fn print_non_breaking_space(codegen: &mut Codegen, state: &mut PrintStringState) {
    debug_assert_eq!(state.peek(), Some(0xC2));

    // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
    // so there must be 1 more byte available to consume
    let next = unsafe { *state.bytes.as_slice().get_unchecked(1) };
    if next == NBSP_LAST_BYTE {
        // Character is NBSP.
        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character.
        unsafe { state.flush_and_consume_bytes(codegen, 2) };
        codegen.print_str("\\xA0");
    } else {
        // Some other character starting with 0xC2. Advance past it.
        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character.
        unsafe { state.consume_bytes_unchecked(2) };
    }
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
#[cold]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}

/// Check if `slice` is `</script`, regardless of case.
///
/// `slice.len()` must be 8.
//
// `#[inline(always)]` so that compiler can see from caller that `slice.len() == 8`
// and so `slice.try_into().unwrap()` cannot fail. This function is only 4 instructions.
#[expect(clippy::inline_always)]
#[inline(always)]
pub fn is_script_close_tag(slice: &[u8]) -> bool {
    // Compiler condenses these operations to an 8-byte read, u64 AND, and u64 compare.
    // https://godbolt.org/z/K8q68WGn6
    let mut bytes: [u8; 8] = slice.try_into().unwrap();
    for byte in bytes.iter_mut().skip(2) {
        // `| 32` converts ASCII upper case letters to lower case.
        *byte |= 32;
    }
    bytes == *b"</script"
}
