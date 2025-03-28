use std::slice;

use oxc_ast::ast::StringLiteral;
use oxc_syntax::identifier::{LS, NBSP, PS};

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

/// String printer state.
struct PrintStringState<'s> {
    chunk_start: *const u8,
    bytes: slice::Iter<'s, u8>,
    quote: Quote,
    quote_is_unknown: bool,
    lone_surrogates: bool,
    allow_backtick: bool,
}

impl PrintStringState<'_> {
    /// Peek next byte in `bytes` iterator.
    #[inline]
    fn peek(&self) -> Option<u8> {
        self.bytes.clone().next().copied()
    }

    /// Peek next byte in `bytes` iterator, without checking not at end.
    ///
    /// # SAFETY
    /// `bytes` iterator must not be at end.
    #[inline]
    unsafe fn peek_unchecked(&self) -> u8 {
        debug_assert!(self.bytes.clone().next().is_some());
        // SAFETY: Caller guarantees `bytes` iterator is not at end
        unsafe { *self.bytes.clone().next().unwrap_unchecked() }
    }

    /// TODO
    ///
    /// # SAFETY
    ///
    /// TODO
    unsafe fn consume_byte(&mut self) {
        debug_assert!(!self.bytes.as_slice().is_empty());
        // SAFETY: TODO
        unsafe { self.bytes.next().unwrap_unchecked() };
    }

    /// TODO
    ///
    /// # SAFETY
    ///
    /// TODO
    unsafe fn consume_bytes<const N: usize>(&mut self) {
        debug_assert!(self.bytes.as_slice().len() >= N);
        // SAFETY: TODO
        unsafe {
            for _i in 0..N {
                self.bytes.next().unwrap_unchecked();
            }
        }
    }

    fn start_chunk(&mut self) {
        self.chunk_start = self.bytes.as_slice().as_ptr();
    }
}

impl Codegen<'_> {
    /// Print a [`StringLiteral`].
    pub(crate) fn print_string_literal(&mut self, s: &StringLiteral<'_>, allow_backtick: bool) {
        self.add_source_mapping(s.span);

        // If `minify` option enabled, quote will be chosen depending on what produces shortest output.
        // What is the best quote to use will be determined when first character needing escape is found.
        // This avoids iterating through the string twice if it contains no quotes (common case).
        // Don't print opening quote now, because we don't know what it is yet.
        //
        // If not in `minify` mode, print the quote requested in options.
        let quote = self.quote;
        let quote_is_unknown = self.options.minify;
        if !quote_is_unknown {
            quote.print(self);
        };

        // Loop through bytes, looking for any which need to be escaped.
        // String is written to buffer in chunks.
        let bytes = s.value.as_bytes().iter();
        let mut state = PrintStringState {
            chunk_start: bytes.as_slice().as_ptr(),
            bytes,
            quote,
            quote_is_unknown,
            lone_surrogates: s.lone_surrogates,
            allow_backtick,
        };

        // Loop through bytes.
        while let Some(b) = state.peek() {
            // Look up whether byte needs escaping
            let escape = ESCAPES.0[b as usize];
            if escape == Escape::__ {
                // No escape required.
                // SAFETY: We just checked there's a byte to consume.
                unsafe { state.consume_byte() };
            } else {
                // Escape may be required. Execute byte handler.
                let byte_handler = BYTE_HANDLERS.0[escape as usize - 1];
                byte_handler(self, &mut state);
            }
        }

        // Flush any remaining bytes.
        // SAFETY: `bytes` iterator is at end, which by definition is on a UTF-8 char boundary
        unsafe { self.flush(&mut state) };

        // Print closing quote
        state.quote.print(self);
    }

    /// TODO
    ///
    /// # SAFETY
    ///
    /// TODO
    unsafe fn flush_and_consume_byte(&mut self, state: &mut PrintStringState) {
        // SAFETY: TODO
        unsafe { self.flush_and_consume_bytes::<1>(state) };
    }

    /// TODO
    ///
    /// # SAFETY
    ///
    /// TODO
    unsafe fn flush_and_consume_bytes<const N: usize>(&mut self, state: &mut PrintStringState) {
        // SAFETY: TODO
        unsafe { self.flush(state) };

        debug_assert!(state.bytes.as_slice().len() >= N);
        // SAFETY: TODO
        unsafe {
            for _i in 0..N {
                state.bytes.next().unwrap_unchecked();
            }
        }

        state.start_chunk();
    }

    /// Flush all bytes from `chunk_start` up to current position of `bytes` iterator into buffer.
    ///
    /// If what quote character to use has not been decided yet, calculate the best quote character to use,
    /// and print it before flushing.
    ///
    /// # SAFETY
    ///
    /// `bytes` iterator must be positioned on a UTF-8 character boundary.
    unsafe fn flush(&mut self, state: &mut PrintStringState) {
        // If which quote character to use is not already known, calculate it and print opening quote
        self.calculate_quote(state);

        // SAFETY: `chunk_start` is pointer to current position of `bytes` iterator at some point,
        // and the iterator only advances, so current position of `bytes` must be on or after `chunk_start`
        let len = unsafe {
            let bytes_ptr = state.bytes.as_slice().as_ptr();
            let offset = bytes_ptr.offset_from(state.chunk_start);
            usize::try_from(offset).unwrap_unchecked()
        };

        // SAFETY: `chunk_start` is within bounds of original `&str`.
        // `bytes` iter cannot go past end of `&str` either.
        // So a slice of `len` bytes starting at `chunk_start` must be within bounds of the `&str`.
        // Caller guarantees `bytes` iterator is positioned on a UTF-8 character boundary.
        // `chunk_start` is too. Therefore the slice between these two must be a valid UTF-8 string.
        unsafe {
            let slice = slice::from_raw_parts(state.chunk_start, len);
            self.code.print_bytes_unchecked(slice);
        }
    }

    /// Calculate what quote character to use, and print that quote.
    fn calculate_quote(&mut self, state: &mut PrintStringState) -> Quote {
        if !state.quote_is_unknown {
            return state.quote;
        }

        let bytes = state.bytes.clone();
        let quote = if state.allow_backtick {
            calculate_quote_maybe_backtick(bytes)
        } else {
            calculate_quote_no_backtick(bytes)
        };

        quote.print(self);

        state.quote = quote;
        state.quote_is_unknown = false;

        quote
    }
}

/// Calculate optimum quote character to use, when backtick (`) is an option.
fn calculate_quote_maybe_backtick(mut bytes: slice::Iter<'_, u8>) -> Quote {
    // String length is max `u32::MAX`, so use `i64` to make overflow impossible
    let mut single_cost: i64 = 0;
    let mut double_cost: i64 = 0;
    let mut backtick_cost: i64 = 0;
    while let Some(b) = bytes.next() {
        match b {
            b'\n' => backtick_cost -= 1,
            b'\'' => single_cost += 1,
            b'"' => double_cost += 1,
            b'`' => backtick_cost += 1,
            b'$' => {
                if bytes.clone().next() == Some(&b'{') {
                    backtick_cost += 1;
                }
            }
            _ => {}
        }
    }

    #[rustfmt::skip]
    let quote = if double_cost >= backtick_cost {
        if backtick_cost > single_cost {
            Quote::Single
        } else {
            Quote::Backtick
        }
    } else if double_cost > single_cost {
        Quote::Single
    } else {
        Quote::Double
    };
    quote
}

/// Calculate optimum quote character to use, when backtick (`) is not an option.
fn calculate_quote_no_backtick(bytes: slice::Iter<'_, u8>) -> Quote {
    // String length is max `u32::MAX`, so `i64` cannot overflow
    let mut single_cost: i64 = 0;
    for &b in bytes {
        match b {
            b'\'' => single_cost += 1,
            b'"' => single_cost -= 1,
            _ => {}
        }
    }

    if single_cost < 0 { Quote::Single } else { Quote::Double }
}

/// Convert `char` to UTF-8 bytes array.
const fn to_bytes<const N: usize>(ch: char) -> [u8; N] {
    let mut bytes = [0u8; N];
    ch.encode_utf8(&mut bytes);
    bytes
}

/// `LS` character as UTF-8 bytes.
const LS_BYTES: [u8; 3] = to_bytes(LS);
/// `PS` character as UTF-8 bytes.
const PS_BYTES: [u8; 3] = to_bytes(PS);

const _: () = assert!(LS_BYTES[0] == 0xE2);
const _: () = assert!(PS_BYTES[0] == 0xE2);
const LS_LAST_2_BYTES: [u8; 2] = [LS_BYTES[1], LS_BYTES[2]];
const PS_LAST_2_BYTES: [u8; 2] = [PS_BYTES[1], PS_BYTES[2]];

/// `NBSP` character as UTF-8 bytes.
const NBSP_BYTES: [u8; 2] = to_bytes(NBSP);
const _: () = assert!(NBSP_BYTES[0] == 0xC2);
const NBSP_LAST_BYTE: u8 = NBSP_BYTES[1];

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = to_bytes('\u{FFFD}');
const _: () = assert!(LOSSY_REPLACEMENT_CHAR_BYTES[0] == 0xEF);
const LOSSY_REPLACEMENT_CHAR_LAST_2_BYTES: [u8; 2] =
    [LOSSY_REPLACEMENT_CHAR_BYTES[1], LOSSY_REPLACEMENT_CHAR_BYTES[2]];

/// Escape codes.
/// Used as index into `BYTE_HANDLERS`.
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
    LS = 14, // LS/PS - U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR (first byte)
    NB = 15, // NBSP  - Non-breaking space (first byte)
    LO = 16, // �     - U+FFFD lossy replacement character (first byte)
}

#[repr(C, align(128))]
struct Aligned128<T>(T);

/// Table mapping bytes to `Escape`s.
static ESCAPES: Aligned128<[Escape; 256]> = {
    #[allow(clippy::enum_glob_use, clippy::allow_attributes)]
    use Escape::*;
    Aligned128([
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        NU, __, __, __, __, __, __, BE, BK, __, NL, VT, FF, CR, __, __, // 0
        __, __, __, __, __, __, __, __, __, __, __, ES, __, __, __, __, // 1
        __, __, DQ, __, DO, __, __, SQ, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
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
        __, __, LS, __, __, __, __, __, __, __, __, __, __, __, __, LO, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ])
};

type ByteHandler = fn(&mut Codegen, &mut PrintStringState);

/// Byte handlers.
/// Indexed by `Escape as usize - 1`.
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
    print_ls_or_ps,
    print_non_breaking_space,
    print_lossy_replacement,
]);

// Byte handlers for bytes which need escaping

// \x00
fn print_null(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\x00` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    if state.peek().is_some_and(|b| b.is_ascii_digit()) {
        codegen.print_str("\\x00");
    } else {
        codegen.print_str("\\0");
    }
}

// \x07
fn print_bell(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\x07` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\x07");
}

// \b
fn print_backspace(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\b` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\b");
}

// \v
fn print_vertical_tab(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\v` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\v");
}

// \f
fn print_form_field(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\f` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\f");
}

// \n
fn print_new_line(codegen: &mut Codegen, state: &mut PrintStringState) {
    if codegen.calculate_quote(state) == Quote::Backtick {
        // No need to escape.
        // SAFETY: `\n` is ASCII.
        unsafe { state.consume_byte() };
    } else {
        // SAFETY: `\n` is ASCII
        unsafe { codegen.flush_and_consume_byte(state) };
        codegen.print_str("\\n");
    }
}

// \r
fn print_carriage_return(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\r` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\r");
}

// \x1B
fn print_escape(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\x1B` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\x1B");
}

// \
fn print_backslash(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: `\` is ASCII
    unsafe { codegen.flush_and_consume_byte(state) };
    codegen.print_str("\\\\");
}

// '
fn print_single_quote(codegen: &mut Codegen, state: &mut PrintStringState) {
    if codegen.calculate_quote(state) == Quote::Single {
        // SAFETY: `'` is ASCII
        unsafe { codegen.flush_and_consume_byte(state) };
        codegen.print_str("\\'");
    } else {
        // No need to escape.
        // SAFETY: `'` is ASCII.
        unsafe { state.consume_byte() };
    }
}

// "
fn print_double_quote(codegen: &mut Codegen, state: &mut PrintStringState) {
    if codegen.calculate_quote(state) == Quote::Double {
        // SAFETY: `"` is ASCII
        unsafe { codegen.flush_and_consume_byte(state) };
        codegen.print_str("\\\"");
    } else {
        // No need to escape.
        // SAFETY: `"` is ASCII.
        unsafe { state.consume_byte() };
    }
}

// `
fn print_backtick(codegen: &mut Codegen, state: &mut PrintStringState) {
    if codegen.calculate_quote(state) == Quote::Backtick {
        // SAFETY: ` is ASCII
        unsafe { codegen.flush_and_consume_byte(state) };
        codegen.print_str("\\`");
    } else {
        // No need to escape.
        // SAFETY: ` is ASCII.
        unsafe { state.consume_byte() };
    }
}

// $
fn print_dollar(codegen: &mut Codegen, state: &mut PrintStringState) {
    // Note: Check next byte is `{` first, to avoid calculating quote unless have to
    let next = state.bytes.as_slice().get(1);
    if next == Some(&b'{') && codegen.calculate_quote(state) == Quote::Backtick {
        // SAFETY: `$` is ASCII
        unsafe { codegen.flush_and_consume_byte(state) };
        codegen.print_str("\\$");
    } else {
        // No need to escape.
        // SAFETY: `$` is ASCII.
        unsafe { state.consume_byte() };
    }
}

// 0xE2 - first byte of <LS> or <PS>
fn print_ls_or_ps(codegen: &mut Codegen, state: &mut PrintStringState) {
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
            unsafe { state.consume_bytes::<3>() };
            return;
        }
    };

    // SAFETY: 0xE2 is always the start of a 3-byte Unicode character.
    // We haven't advanced `bytes` since start of this function, so `bytes` must be positioned on
    // a UTF-8 character boundary.
    unsafe { codegen.flush_and_consume_bytes::<3>(state) };

    codegen.print_str(replacement);
}

// 0xC2 - first byte of <NBSP>
fn print_non_breaking_space(codegen: &mut Codegen, state: &mut PrintStringState) {
    // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
    // so there must be 1 more byte available to consume
    let next = unsafe { state.peek_unchecked() };
    if next == NBSP_LAST_BYTE {
        // Character is NBSP.
        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character.
        // We haven't advanced `bytes` since start of this function, so `bytes` must be positioned on
        // a UTF-8 character boundary.
        unsafe { codegen.flush_and_consume_bytes::<2>(state) };
        codegen.print_str("\\xA0");
    } else {
        // Some other character starting with 0xC2. Advance past it.
        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
        unsafe { state.consume_bytes::<2>() };
    }
}

// 0xEF - first byte of lossy replacement character (U+FFFD)
fn print_lossy_replacement(codegen: &mut Codegen, state: &mut PrintStringState) {
    let bytes = &mut state.bytes;

    if state.lone_surrogates {
        let next2: [u8; 2] = {
            // SAFETY: 0xEF is always the start of a 3-byte Unicode character,
            // so there must be 2 more bytes available to consume
            let next2 = unsafe { bytes.as_slice().get_unchecked(1..3) };
            next2.try_into().unwrap()
        };

        if next2 == LOSSY_REPLACEMENT_CHAR_LAST_2_BYTES {
            // String contains lone surrogates which use the lossy replacement character (U+FFFD)
            // as an escape marker.
            // The lone surrogate is encoded as `\u{FFFD}XXXX` where `XXXX` is the code point as hex.
            // Get the 4 hex bytes.
            let bytes = &mut state.bytes;
            let hex: [u8; 4] = bytes.as_slice()[3..7].try_into().unwrap();

            if hex == *b"fffd" {
                // Actual lossy replacement character.
                // Flush up to and including the lossy replacement character, then skip the 4 hex bytes.
                // SAFETY: 0xEF is always the start of a 3-byte Unicode character.
                // `bytes.as_slice()[3..7]` would have panicked if there weren't 4 more bytes after it.
                // All those bytes are ASCII, so this leaves `bytes` on a UTF-8 char boundary.
                unsafe {
                    state.consume_bytes::<3>();
                    codegen.flush(state);
                    state.consume_bytes::<4>();
                    state.start_chunk();
                }
                return;
            }

            // Flush text before the lossy replacement character.
            // SAFETY: We haven't advanced `bytes` yet, so it must be positioned on a UTF-8 char boundary
            unsafe { codegen.flush(state) };

            // Check all 4 hex bytes are ASCII
            assert_eq!(u32::from_ne_bytes(hex) & 0x8080_8080, 0);

            // SAFETY: `bytes.as_slice()[3..7]` would have panicked if aren't at least 7 bytes remaining.
            // First 3 bytes are lossy replacement character, we just checked next 4 bytes are ASCII,
            // so this leaves `bytes` on a UTF-8 char boundary.
            unsafe { state.consume_bytes::<7>() };
            state.start_chunk();

            // Lossy replacement character representing a lone surrogate.
            codegen.print_str("\\u");
            // SAFETY: Just checked all 4 bytes are ASCII
            unsafe { codegen.code.print_bytes_unchecked(&hex) };

            return;
        }
    }

    // `lone_surrogates` is `false` or character is some other character starting with 0xEF.
    // Advance past the character.
    // SAFETY: 0xEF is always the start of a 3-byte Unicode character
    unsafe { state.consume_bytes::<3>() };
}
