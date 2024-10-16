#![allow(missing_docs)] // fixme
use assert_unchecked::assert_unchecked;
use unicode_id_start::{is_id_continue_unicode, is_id_start_unicode};

pub const EOF: char = '\0';

// 11.1 Unicode Format-Control Characters

/// U+200C ZERO WIDTH NON-JOINER, abbreviated in the spec as `<ZWNJ>`.
/// Specially permitted in identifiers.
pub const ZWNJ: char = '\u{200c}';

/// U+200D ZERO WIDTH JOINER, abbreviated as `<ZWJ>`.
/// Specially permitted in identifiers.
pub const ZWJ: char = '\u{200d}';

/// U+FEFF ZERO WIDTH NO-BREAK SPACE, abbreviated `<ZWNBSP>`.
/// Considered a whitespace character in JS.
pub const ZWNBSP: char = '\u{feff}';

// 11.2 White Space
/// U+0009 CHARACTER TABULATION, abbreviated `<TAB>`.
pub const TAB: char = '\u{9}';

/// U+000B VERTICAL TAB, abbreviated `<VT>`.
pub const VT: char = '\u{b}';

/// U+000C FORM FEED, abbreviated `<FF>`.
pub const FF: char = '\u{c}';

/// U+00A0 NON-BREAKING SPACE, abbreviated `<NBSP>`.
pub const NBSP: char = '\u{a0}';

pub fn is_irregular_whitespace(c: char) -> bool {
    matches!(
        c,
        VT | FF | NBSP | ZWNBSP | '\u{85}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

// 11.3 Line Terminators

///  U+000A LINE FEED, abbreviated in the spec as `<LF>`.
pub const LF: char = '\u{a}';

/// U+000D CARRIAGE RETURN, abbreviated in the spec as `<CR>`.
pub const CR: char = '\u{d}';

/// U+2028 LINE SEPARATOR, abbreviated `<LS>`.
pub const LS: char = '\u{2028}';

/// U+2029 PARAGRAPH SEPARATOR, abbreviated `<PS>`.
pub const PS: char = '\u{2029}';

pub fn is_regular_line_terminator(c: char) -> bool {
    matches!(c, LF | CR)
}

pub fn is_irregular_line_terminator(c: char) -> bool {
    matches!(c, LS | PS)
}

pub fn is_line_terminator(c: char) -> bool {
    is_regular_line_terminator(c) || is_irregular_line_terminator(c)
}

const XX: bool = true;
const __: bool = false;

#[repr(C, align(64))]
pub struct Align64<T>(pub(crate) T);

// `a`-`z`, `A`-`Z`, `$` (0x24), `_` (0x5F)
#[rustfmt::skip]
pub static ASCII_START: Align64<[bool; 128]> = Align64([
//  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F   //
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
    __, __, __, __, XX, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 4
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, __, __, __, __, XX, // 5
    __, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 6
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, __, __, __, __, __, // 7
]);

// `ASCII_START` + `0`-`9`
#[rustfmt::skip]
pub static ASCII_CONTINUE: Align64<[bool; 128]> = Align64([
//  0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F   //
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
    __, __, __, __, XX, __, __, __, __, __, __, __, __, __, __, __, // 2
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, __, __, __, __, __, __, // 3
    __, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 4
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, __, __, __, __, XX, // 5
    __, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 6
    XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, __, __, __, __, __, // 7
]);

/// Section 12.7 Detect `IdentifierStartChar`
#[inline]
pub fn is_identifier_start(c: char) -> bool {
    if c.is_ascii() {
        return is_identifier_start_ascii(c);
    }
    is_identifier_start_unicode(c)
}

#[inline]
pub fn is_identifier_start_ascii(c: char) -> bool {
    ASCII_START.0[c as usize]
}

#[inline]
pub fn is_identifier_start_unicode(c: char) -> bool {
    is_id_start_unicode(c)
}

/// Section 12.7 Detect `IdentifierPartChar`
/// NOTE 2: The nonterminal `IdentifierPart` derives _ via `UnicodeIDContinue`.
#[inline]
pub fn is_identifier_part(c: char) -> bool {
    if c.is_ascii() {
        return is_identifier_part_ascii(c);
    }
    is_identifier_part_unicode(c)
}

#[inline]
pub fn is_identifier_part_ascii(c: char) -> bool {
    ASCII_CONTINUE.0[c as usize]
}

#[inline]
pub fn is_identifier_part_unicode(c: char) -> bool {
    is_id_continue_unicode(c) || c == ZWNJ || c == ZWJ
}

/// Determine if a string is a valid JS identifier.
#[allow(clippy::missing_panics_doc)]
pub fn is_identifier_name(name: &str) -> bool {
    // This function contains a fast path for ASCII (common case), iterating over bytes and using
    // the cheap `is_identifier_start_ascii` and `is_identifier_part_ascii` to test bytes.
    // Only if a Unicode char is found, fall back to iterating over `char`s, and using the more
    // expensive `is_identifier_start_unicode` and `is_identifier_part`.
    // As a further optimization, we test if bytes are ASCII in blocks of 8 or 4 bytes, rather than 1 by 1.

    // Get first byte. Exit if empty string.
    let bytes = name.as_bytes();
    let Some(&first_byte) = bytes.first() else { return false };

    let mut chars = if first_byte.is_ascii() {
        // First byte is ASCII
        if !is_identifier_start_ascii(first_byte as char) {
            return false;
        }

        let mut index = 1;
        'outer: loop {
            // Check blocks of 8 bytes, then 4 bytes, then single bytes
            let bytes_remaining = bytes.len() - index;
            if bytes_remaining >= 8 {
                // Process block of 8 bytes.
                // Check that next 8 bytes are all ASCII.
                // SAFETY: We checked above that there are at least 8 bytes to read starting at `index`
                #[allow(clippy::cast_ptr_alignment)]
                let next8_as_u64 = unsafe {
                    let ptr = bytes.as_ptr().add(index).cast::<u64>();
                    ptr.read_unaligned()
                };
                let high_bits = next8_as_u64 & 0x8080_8080_8080_8080;
                if high_bits != 0 {
                    // Some chars in this block are non-ASCII
                    break;
                }

                let next8 = next8_as_u64.to_ne_bytes();
                for b in next8 {
                    // SAFETY: We just checked all these bytes are ASCII
                    unsafe { assert_unchecked!(b.is_ascii()) };
                    if !is_identifier_part_ascii(b as char) {
                        return false;
                    }
                }

                index += 8;
            } else if bytes_remaining >= 4 {
                // Process block of 4 bytes.
                // Check that next 4 bytes are all ASCII.
                // SAFETY: We checked above that there are at least 4 bytes to read starting at `index`
                #[allow(clippy::cast_ptr_alignment)]
                let next4_as_u32 = unsafe {
                    let ptr = bytes.as_ptr().add(index).cast::<u32>();
                    ptr.read_unaligned()
                };
                let high_bits = next4_as_u32 & 0x8080_8080;
                if high_bits != 0 {
                    // Some chars in this block are non-ASCII
                    break;
                }

                let next4 = next4_as_u32.to_ne_bytes();
                for b in next4 {
                    // SAFETY: We just checked all these bytes are ASCII
                    unsafe { assert_unchecked!(b.is_ascii()) };
                    if !is_identifier_part_ascii(b as char) {
                        return false;
                    }
                }

                index += 4;
            } else {
                loop {
                    let Some(&b) = bytes.get(index) else {
                        // We got to the end with no non-identifier chars found
                        return true;
                    };

                    if b.is_ascii() {
                        if !is_identifier_part_ascii(b as char) {
                            return false;
                        }
                    } else {
                        // Unicode byte found
                        break 'outer;
                    }

                    index += 1;
                }
            }
        }

        // Unicode byte found - search rest of string (from this byte onwards) as Unicode
        name[index..].chars()
    } else {
        // First char is Unicode.
        // NB: `unwrap()` cannot fail because we already checked the string is not empty.
        let mut chars = name.chars();
        let first_char = chars.next().unwrap();
        if !is_identifier_start_unicode(first_char) {
            return false;
        }
        // Search rest of string as Unicode
        chars
    };

    // A Unicode char was found - search rest of string as Unicode
    chars.all(is_identifier_part)
}

#[test]
fn is_identifier_name_true() {
    let cases = [
        // 1 char ASCII
        "a",
        "z",
        "A",
        "Z",
        "_",
        "$",
        // 1 char Unicode
        "µ", // 2 bytes
        "ख", // 3 bytes
        "𐀀", // 4 bytes
        // Multiple chars ASCII
        "az",
        "AZ",
        "_a",
        "$Z",
        "a0",
        "A9",
        "_0",
        "$9",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_$",
        "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$",
        "$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
        // Multiple chars Unicode
        "µख𐀀",
        // ASCII + Unicode, starting with ASCII
        "AµBखC𐀀D",
        // ASCII + Unicode, starting with Unicode
        "µAखB𐀀",
    ];

    for str in cases {
        assert!(is_identifier_name(str));
    }
}

#[test]
fn is_identifier_name_false() {
    let cases = [
        // Empty string
        "",
        // 1 char ASCII
        "0",
        "9",
        "-",
        "~",
        "+",
        // 1 char Unicode
        "£", // 2 bytes
        "৸", // 3 bytes
        "𐄬", // 4 bytes
        // Multiple chars ASCII
        "0a",
        "9a",
        "-a",
        "+a",
        "a-Z",
        "A+z",
        "a-",
        "a+",
        // Multiple chars Unicode
        "£৸𐄬",
        // ASCII + Unicode, starting with ASCII
        "A£",
        "A৸",
        "A𐄬",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc£",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc৸",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc𐄬",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc£abcdefghijklmnopqrstuvwxyz",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc৸abcdefghijklmnopqrstuvwxyz",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc𐄬abcdefghijklmnopqrstuvwxyz",
        // ASCII + Unicode, starting with Unicode
        "£A",
        "৸A",
        "𐄬A",
    ];

    for str in cases {
        assert!(!is_identifier_name(str));
    }
}
