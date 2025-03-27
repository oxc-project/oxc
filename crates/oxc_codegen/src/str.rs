use std::slice;

use oxc_ast::ast::StringLiteral;
use oxc_data_structures::code_buffer::CodeBuffer;
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

struct PrintStringState<'i> {
    bytes: slice::Iter<'i, u8>,
    quote: Quote,
    lone_surrogates: bool,
}

impl Codegen<'_> {
    pub(crate) fn print_string_literal(&mut self, s: &StringLiteral<'_>, allow_backtick: bool) {
        self.add_source_mapping(s.span);

        let quote = if self.options.minify {
            let mut single_cost: i32 = 0;
            let mut double_cost: i32 = 0;
            let mut backtick_cost: i32 = 0;
            let mut bytes = s.value.as_bytes().iter().peekable();
            while let Some(b) = bytes.next() {
                match b {
                    b'\n' if self.options.minify => backtick_cost = backtick_cost.saturating_sub(1),
                    b'\'' => single_cost += 1,
                    b'"' => double_cost += 1,
                    b'`' => backtick_cost += 1,
                    b'$' => {
                        if bytes.peek() == Some(&&b'{') {
                            backtick_cost += 1;
                        }
                    }
                    _ => {}
                }
            }
            let mut quote = Quote::Double;
            if allow_backtick && double_cost >= backtick_cost {
                quote = Quote::Backtick;
                if backtick_cost > single_cost {
                    quote = Quote::Single;
                }
            } else if double_cost > single_cost {
                quote = Quote::Single;
            }
            quote
        } else {
            self.quote
        };

        quote.print(self);
        self.print_unquoted_utf16(s, quote);
        quote.print(self);
    }

    fn print_unquoted_utf16(&mut self, s: &StringLiteral<'_>, quote: Quote) {
        let mut state = PrintStringState {
            bytes: s.value.as_bytes().iter(),
            quote,
            lone_surrogates: s.lone_surrogates,
        };

        while let Some(&b) = state.bytes.next() {
            // Lookup whether byte needs escaping
            let escape = ESCAPES[b as usize];
            if escape == Escape::__ {
                // No escape required.
                // SAFETY: If `b` is not ASCII, will temporarily leave `CodeBuffer` containing
                // an invalid UTF-8 string. But the escape table will contain `Escape::__` for
                // the remaining bytes of the Unicode character, so we'll also print those remaining
                // bytes on next turns of the loop, restoring `CodeBuffer` to a valid UTF-8 string.
                unsafe { self.code.print_byte_unchecked(b) };
            } else {
                // Escape required. Call handler for this byte value.
                let byte_handler = BYTE_HANDLERS[escape as usize - 1];
                byte_handler(&mut self.code, &mut state);
            }
        }
    }
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

const LS_OR_PS_FIRST_BYTE: u8 = LS_BYTES[0];
const _: () = {
    assert!(LS_OR_PS_FIRST_BYTE == 0xE2);
    assert!(PS_BYTES[0] == LS_OR_PS_FIRST_BYTE);
};
const LS_LAST_2_BYTES: [u8; 2] = [LS_BYTES[1], LS_BYTES[2]];
const PS_LAST_2_BYTES: [u8; 2] = [PS_BYTES[1], PS_BYTES[2]];

/// `NBSP` character as UTF-8 bytes.
const NBSP_BYTES: [u8; 2] = to_bytes(NBSP);
const NBSP_FIRST_BYTE: u8 = NBSP_BYTES[0];
const _: () = assert!(NBSP_FIRST_BYTE == 0xC2);

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = to_bytes('\u{FFFD}');
const LOSSY_REPLACEMENT_CHAR_FIRST_BYTE: u8 = LOSSY_REPLACEMENT_CHAR_BYTES[0];
const _: () = assert!(LOSSY_REPLACEMENT_CHAR_FIRST_BYTE == 0xEF);

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
    LO = 14, // �     - U+FFFD lossy replacement character (first byte)
    LS = 15, // LS/PS - U+2028 LINE SEPARATOR or U+2029 PARAGRAPH SEPARATOR (first byte)
    NB = 16, // NBSP  - Non-breaking space (first byte)
}

static ESCAPES: [Escape; 256] = {
    #[allow(clippy::enum_glob_use, clippy::allow_attributes)]
    use Escape::*;
    [
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
    ]
};

type ByteHandler = fn(&mut CodeBuffer, &mut PrintStringState);

static BYTE_HANDLERS: [ByteHandler; 16] = [
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
    print_lossy_replacement,
    print_ls_or_ps,
    print_not_breaking_space,
];

fn print_null(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.bytes.clone().next().is_some_and(u8::is_ascii_digit) {
        code.print_str("\\x00");
    } else {
        code.print_str("\\0");
    }
}

fn print_bell(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\x07");
}

fn print_backspace(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\b");
}

fn print_vertical_tab(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\v");
}

fn print_form_field(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\f");
}

fn print_new_line(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.quote == Quote::Backtick {
        code.print_ascii_byte(b'\n');
    } else {
        code.print_str("\\n");
    }
}

fn print_carriage_return(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\r");
}

fn print_escape(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\x1B");
}

fn print_backslash(code: &mut CodeBuffer, _state: &mut PrintStringState) {
    code.print_str("\\\\");
}

fn print_single_quote(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.quote == Quote::Single {
        code.print_str("\\'");
    } else {
        code.print_ascii_byte(b'\'');
    }
}

fn print_double_quote(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.quote == Quote::Double {
        code.print_str("\\\"");
    } else {
        code.print_ascii_byte(b'"');
    }
}

fn print_backtick(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.quote == Quote::Backtick {
        code.print_str("\\`");
    } else {
        code.print_ascii_byte(b'`');
    }
}

fn print_dollar(code: &mut CodeBuffer, state: &mut PrintStringState) {
    if state.bytes.clone().next().copied() == Some(b'{') {
        code.print_str("\\$");
    } else {
        code.print_ascii_byte(b'$');
    }
}

fn print_lossy_replacement(code: &mut CodeBuffer, state: &mut PrintStringState) {
    let bytes = &mut state.bytes;

    // SAFETY: 0xEF is always the start of a 3-byte Unicode character,
    // so there must be 2 more bytes available to consume
    let next2: [u8; 2] = unsafe {
        let next2 = bytes.as_slice().get_unchecked(..2).try_into().unwrap();
        bytes.next().unwrap_unchecked();
        bytes.next().unwrap_unchecked();
        next2
    };

    if next2 == [LOSSY_REPLACEMENT_CHAR_BYTES[1], LOSSY_REPLACEMENT_CHAR_BYTES[2]]
        && state.lone_surrogates
    {
        // If `lone_surrogates` is set, string contains lone surrogates which are escaped
        // using the lossy replacement character (U+FFFD) as an escape marker.
        // The lone surrogate is encoded as `\u{FFFD}XXXX` where `XXXX` is the code point as hex.
        let hex: [u8; 4] = bytes.as_slice()[..4].try_into().unwrap();
        // SAFETY: `bytes.as_slice()[..4]` would have panicked if there were not
        // at least 4 bytes remaining
        unsafe {
            for _i in 0..4 {
                bytes.next().unwrap_unchecked();
            }
        }

        if hex == *b"fffd" {
            // Actual lossy replacement character
            code.print_str("\u{FFFD}");
        } else {
            // Lossy replacement character representing a lone surrogate.
            // Check all 4 hex chars are ASCII.
            assert_eq!(u32::from_ne_bytes(hex) & 0x8080_8080, 0);
            code.print_str("\\u");
            // SAFETY: Just checked all 4 bytes are ASCII
            unsafe { code.print_bytes_unchecked(&hex) };
        }
    } else {
        // Another Unicode char beginning with 0xEF or `lone_surrogates` flag is unset.
        // SAFETY: 0xEF is always the start of a 3-byte Unicode character,
        // so printing those 3 bytes leaves `CodeBuffer` containing a valid UTF-8 string.
        unsafe {
            code.print_bytes_unchecked(&[LOSSY_REPLACEMENT_CHAR_FIRST_BYTE, next2[0], next2[1]]);
        }
    }
}

fn print_ls_or_ps(code: &mut CodeBuffer, state: &mut PrintStringState) {
    let bytes = &mut state.bytes;

    // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
    // so there must be 2 more bytes available to consume
    let next2: [u8; 2] = unsafe {
        let next2 = bytes.as_slice().get_unchecked(..2).try_into().unwrap();
        bytes.next().unwrap_unchecked();
        bytes.next().unwrap_unchecked();
        next2
    };

    match next2 {
        LS_LAST_2_BYTES => code.print_str("\\u2028"),
        PS_LAST_2_BYTES => code.print_str("\\u2029"),
        _ => {
            // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
            // so printing those 3 bytes leaves `CodeBuffer` containing a valid UTF-8 string
            unsafe { code.print_bytes_unchecked(&[LS_OR_PS_FIRST_BYTE, next2[0], next2[1]]) };
        }
    }
}

fn print_not_breaking_space(code: &mut CodeBuffer, state: &mut PrintStringState) {
    let bytes = &mut state.bytes;

    // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
    // so there must be 1 more byte available to consume
    let next = unsafe { *bytes.next().unwrap_unchecked() };
    if next == NBSP_BYTES[1] {
        code.print_str("\\xA0");
    } else {
        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
        // so printing those 2 bytes leaves `CodeBuffer` containing a valid UTF-8 string
        unsafe { code.print_bytes_unchecked(&[NBSP_FIRST_BYTE, next]) };
    }
}
