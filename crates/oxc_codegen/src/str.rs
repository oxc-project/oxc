use oxc_ast::ast::StringLiteral;
use oxc_syntax::identifier::{LS, NBSP, PS};

use crate::Codegen;

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

const _: () = assert!(LS_BYTES[0] == PS_BYTES[0]);
const LS_OR_PS_FIRST_BYTE: u8 = LS_BYTES[0]; // 0xE2
const LS_LAST_2_BYTES: [u8; 2] = [LS_BYTES[1], LS_BYTES[2]];
const PS_LAST_2_BYTES: [u8; 2] = [PS_BYTES[1], PS_BYTES[2]];

/// `NBSP` character as UTF-8 bytes.
const NBSP_BYTES: [u8; 2] = to_bytes(NBSP);
const NBSP_FIRST_BYTE: u8 = NBSP_BYTES[0]; // 0xC2

/// Lossy replacement character (U+FFFD) as UTF-8 bytes.
const LOSSY_REPLACEMENT_CHAR_BYTES: [u8; 3] = to_bytes('\u{FFFD}');
const LOSSY_REPLACEMENT_CHAR_FIRST_BYTE: u8 = LOSSY_REPLACEMENT_CHAR_BYTES[0]; // 0xEF

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
        // Encoded string cannot be longer than 4 x length of `s.value`.
        // The longest escape expansion is `\x00` (1 byte), which is printed as "\x00" (4 bytes).
        // Reserve sufficient capacity for the longest possible output, so can skip bounds checks
        // in the loop below.
        self.code.reserve(s.value.len() * 4);

        let mut bytes = s.value.as_bytes().iter();

        // SAFETY: We have reserved the maximum possible capacity required for this string
        // even if every character required the longest escape.
        // So all `print_str_unchecked_cap`, `print_byte_unchecked_cap`, and `print_bytes_unchecked_cap`
        // calls in this loop are guaranteed not to exceed the buffer's capacity.
        unsafe {
            while let Some(&b) = bytes.next() {
                match b {
                    b'\x00' => {
                        if bytes.clone().next().is_some_and(u8::is_ascii_digit) {
                            self.code.print_str_unchecked_cap("\\x00");
                        } else {
                            self.code.print_str_unchecked_cap("\\0");
                        }
                    }
                    b'\x07' => self.code.print_str_unchecked_cap("\\x07"),
                    b'\x08' => self.code.print_str_unchecked_cap("\\b"), // \b
                    b'\x0B' => self.code.print_str_unchecked_cap("\\v"), // \v
                    b'\x0C' => self.code.print_str_unchecked_cap("\\f"), // \f
                    b'\n' => {
                        if quote == Quote::Backtick {
                            self.code.print_byte_unchecked_cap(b'\n');
                        } else {
                            self.code.print_str_unchecked_cap("\\n");
                        }
                    }
                    b'\r' => self.code.print_str_unchecked_cap("\\r"),
                    b'\x1B' => self.code.print_str_unchecked_cap("\\x1B"),
                    b'\\' => self.code.print_str_unchecked_cap("\\\\"),
                    // Allow `U+2028` and `U+2029` in string literals
                    // <https://tc39.es/proposal-json-superset>
                    // <https://github.com/tc39/proposal-json-superset>
                    LS_OR_PS_FIRST_BYTE => {
                        // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
                        // so there must be 2 more bytes available to consume
                        let next2: [u8; 2] =
                            bytes.as_slice().get_unchecked(..2).try_into().unwrap();
                        bytes.next().unwrap_unchecked();
                        bytes.next().unwrap_unchecked();

                        match next2 {
                            LS_LAST_2_BYTES => self.code.print_str_unchecked_cap("\\u2028"),
                            PS_LAST_2_BYTES => self.code.print_str_unchecked_cap("\\u2029"),
                            _ => {
                                // SAFETY: 0xE2 is always the start of a 3-byte Unicode character,
                                // so printing those 3 bytes leaves `CodeBuffer` containing a valid UTF-8 string
                                self.code.print_bytes_unchecked_cap(&[
                                    LS_OR_PS_FIRST_BYTE,
                                    next2[0],
                                    next2[1],
                                ]);
                            }
                        }
                    }
                    NBSP_FIRST_BYTE => {
                        // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
                        // so there must be 1 more byte available to consume
                        let next = *bytes.next().unwrap_unchecked();
                        if next == NBSP_BYTES[1] {
                            self.code.print_str_unchecked_cap("\\xA0");
                        } else {
                            // SAFETY: 0xC2 is always the start of a 2-byte Unicode character,
                            // so printing those 2 bytes leaves `CodeBuffer` containing a valid UTF-8 string
                            self.code.print_bytes_unchecked_cap(&[NBSP_FIRST_BYTE, next]);
                        }
                    }
                    b'\'' => {
                        if quote == Quote::Single {
                            self.code.print_byte_unchecked_cap(b'\\');
                        }
                        self.code.print_byte_unchecked_cap(b'\'');
                    }
                    b'\"' => {
                        if quote == Quote::Double {
                            self.code.print_byte_unchecked_cap(b'\\');
                        }
                        self.code.print_byte_unchecked_cap(b'"');
                    }
                    b'`' => {
                        if quote == Quote::Backtick {
                            self.code.print_byte_unchecked_cap(b'\\');
                        }
                        self.code.print_byte_unchecked_cap(b'`');
                    }
                    b'$' => {
                        if bytes.clone().next().copied() == Some(b'{') {
                            self.code.print_byte_unchecked_cap(b'\\');
                        }
                        self.code.print_byte_unchecked_cap(b'$');
                    }
                    LOSSY_REPLACEMENT_CHAR_FIRST_BYTE => {
                        // SAFETY: 0xEF is always the start of a 3-byte Unicode character,
                        // so there must be 2 more bytes available to consume
                        let next2: [u8; 2] =
                            bytes.as_slice().get_unchecked(..2).try_into().unwrap();
                        bytes.next().unwrap_unchecked();
                        bytes.next().unwrap_unchecked();

                        if next2
                            == [LOSSY_REPLACEMENT_CHAR_BYTES[1], LOSSY_REPLACEMENT_CHAR_BYTES[2]]
                            && s.lone_surrogates
                        {
                            // If `lone_surrogates` is set, string contains lone surrogates which are escaped
                            // using the lossy replacement character (U+FFFD) as an escape marker.
                            // The lone surrogate is encoded as `\u{FFFD}XXXX` where `XXXX` is the code point as hex.
                            let hex: [u8; 4] = bytes.as_slice()[..4].try_into().unwrap();
                            // SAFETY: `bytes.as_slice()[..4]` would have panicked if there were not
                            // at least 4 bytes remaining
                            for _i in 0..4 {
                                bytes.next().unwrap_unchecked();
                            }

                            if hex == *b"fffd" {
                                // Actual lossy replacement character
                                self.code.print_str_unchecked_cap("\u{FFFD}");
                            } else {
                                // Lossy replacement character representing a lone surrogate.
                                // Check all 4 hex chars are ASCII.
                                assert_eq!(u32::from_ne_bytes(hex) & 0x8080_8080, 0);
                                self.code.print_str_unchecked_cap("\\u");
                                // SAFETY: Just checked all 4 bytes are ASCII
                                self.code.print_bytes_unchecked_cap(&hex);
                            }
                        } else {
                            // Another Unicode char beginning with 0xEF or `lone_surrogates` flag is unset.
                            // SAFETY: 0xEF is always the start of a 3-byte Unicode character,
                            // so printing those 3 bytes leaves `CodeBuffer` containing a valid UTF-8 string.
                            self.code.print_bytes_unchecked_cap(&[
                                LOSSY_REPLACEMENT_CHAR_FIRST_BYTE,
                                next2[0],
                                next2[1],
                            ]);
                        }
                    }
                    _ => {
                        // SAFETY: If `b` is not ASCII, will temporarily leave `CodeBuffer` containing
                        // an invalid UTF-8 string. But none of the match arms above will match the remaining
                        // bytes of the Unicode character, so we'll also print those remaining bytes on
                        // next turns of the loop, restoring `CodeBuffer` to a valid UTF-8 string.
                        self.code.print_byte_unchecked_cap(b);
                    }
                }
            }
        }
    }
}
