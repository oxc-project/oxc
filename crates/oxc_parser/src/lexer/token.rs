//! Token

use std::mem;

use oxc_span::Span;

use super::kind::Kind;

// `Token` is stored as a `[u64; 2]`.
//
// Packing the data into a pair of `u64`s provides a significant speed-up to the parser
// vs a struct with separate fields.
// https://github.com/oxc-project/oxc/pull/10933
//
// Stored as 2 x `u64`s rather than 1 x `u128`, so `Token` is aligned on 8, not 16.
// This reduces the size of `Lookahead` to 24 bytes (it's 32 bytes when `Token` is aligned on 16).
//
// Bit layout:
//
// 1st `u64`:
// - Bits 0-31 (32 bits): `start`
// - Bits 32-63 (32 bits): `end`
//
// 2nd `u64`:
// - Bits 0-7 (8 bits): `kind` (as u8)
// - Bit 8 (1 bit): `is_on_new_line`
// - Bit 9 (1 bit): `escaped`
// - Bit 10 (1 bit): `lone_surrogates`
// - Bit 11 (1 bit): `has_separator`

// 1st `u64`
const START_SHIFT: u32 = 0;
const END_SHIFT: u32 = 32;
const START_MASK: u64 = 0xFFFF_FFFF; // 32 bits
const END_MASK: u64 = 0xFFFF_FFFF; // 32 bits

// 2nd `u64`
const KIND_SHIFT: u32 = 0;
const IS_ON_NEW_LINE_SHIFT: u32 = 8;
const ESCAPED_SHIFT: u32 = 9;
const LONE_SURROGATES_SHIFT: u32 = 10;
const HAS_SEPARATOR_SHIFT: u32 = 11;

const KIND_MASK: u64 = 0xFF; // 8 bits

const IS_ON_NEW_LINE_FLAG: u64 = 1 << IS_ON_NEW_LINE_SHIFT;
const ESCAPED_FLAG: u64 = 1 << ESCAPED_SHIFT;
const LONE_SURROGATES_FLAG: u64 = 1 << LONE_SURROGATES_SHIFT;
const HAS_SEPARATOR_FLAG: u64 = 1 << HAS_SEPARATOR_SHIFT;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Token([u64; 2]);

impl Default for Token {
    #[inline]
    fn default() -> Self {
        // `Kind::default()` is `Kind::Eof`. So `[0; 2]` is equivalent to:
        // start: 0,
        // end: 0,
        // kind: Kind::default(),
        // is_on_new_line: false,
        // escaped: false,
        // lone_surrogates: false,
        // has_separator: false,
        const _: () = assert!(Kind::Eof as u8 == 0);
        Self([0; 2])
    }
}

impl Token {
    #[inline]
    pub(super) fn new_on_new_line() -> Self {
        // Start with a default token, then set the flag
        let mut token = Self::default();
        token.0[1] |= IS_ON_NEW_LINE_FLAG;
        token
    }
}

// Getters and setters
impl Token {
    #[inline]
    pub fn span(&self) -> Span {
        Span::new(self.start(), self.end())
    }

    #[inline]
    pub fn start(&self) -> u32 {
        ((self.0[0] >> START_SHIFT) & START_MASK) as u32
    }

    #[inline]
    pub(crate) fn set_start(&mut self, start: u32) {
        self.0[0] &= !(START_MASK << START_SHIFT); // Clear current `start` bits
        self.0[0] |= u64::from(start) << START_SHIFT;
    }

    #[inline]
    pub fn end(&self) -> u32 {
        ((self.0[0] >> END_SHIFT) & END_MASK) as u32
    }

    #[inline]
    pub(crate) fn set_end(&mut self, end: u32) {
        let start = self.start();
        debug_assert!(end >= start, "Token end ({end}) cannot be less than start ({start})");
        self.0[0] &= !(END_MASK << END_SHIFT); // Clear current `end` bits
        self.0[0] |= u64::from(end) << END_SHIFT;
    }

    #[inline]
    pub fn kind(&self) -> Kind {
        // SAFETY: `Kind` is `#[repr(u8)]`. Only `Token::default` and `Token::set_kind` set these bits,
        // and they set them to the `u8` value of an existing `Kind`.
        // So transmuting these bits back to `Kind` must produce a valid `Kind`.
        unsafe { mem::transmute::<u8, Kind>(((self.0[1] >> KIND_SHIFT) & KIND_MASK) as u8) }
    }

    #[inline]
    pub(crate) fn set_kind(&mut self, kind: Kind) {
        self.0[1] &= !(KIND_MASK << KIND_SHIFT); // Clear current `kind` bits
        self.0[1] |= u64::from(kind as u8) << KIND_SHIFT;
    }

    #[inline]
    pub fn is_on_new_line(&self) -> bool {
        (self.0[1] & IS_ON_NEW_LINE_FLAG) != 0
    }

    #[inline]
    pub(crate) fn set_is_on_new_line(&mut self, value: bool) {
        self.0[1] = (self.0[1] & !IS_ON_NEW_LINE_FLAG) | (u64::from(value) * IS_ON_NEW_LINE_FLAG);
    }

    #[inline]
    pub fn escaped(&self) -> bool {
        (self.0[1] & ESCAPED_FLAG) != 0
    }

    #[inline]
    pub(crate) fn set_escaped(&mut self, escaped: bool) {
        self.0[1] = (self.0[1] & !ESCAPED_FLAG) | (u64::from(escaped) * ESCAPED_FLAG);
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        (self.0[1] & LONE_SURROGATES_FLAG) != 0
    }

    #[inline]
    pub(crate) fn set_lone_surrogates(&mut self, value: bool) {
        self.0[1] = (self.0[1] & !LONE_SURROGATES_FLAG) | (u64::from(value) * LONE_SURROGATES_FLAG);
    }

    #[inline]
    pub fn has_separator(&self) -> bool {
        (self.0[1] & HAS_SEPARATOR_FLAG) != 0
    }

    #[inline]
    pub(crate) fn set_has_separator(&mut self, value: bool) {
        self.0[1] = (self.0[1] & !HAS_SEPARATOR_FLAG) | (u64::from(value) * HAS_SEPARATOR_FLAG);
    }
}

#[cfg(test)]
mod test {
    use super::Kind;
    use super::Token;

    // Test size of `Token`
    const _: () = assert!(size_of::<Token>() == 16);

    // Test default token values
    #[test]
    fn default_token_values() {
        let token = Token::default();
        assert_eq!(token.start(), 0);
        assert_eq!(token.end(), 0);
        assert_eq!(token.kind(), Kind::Eof); // Kind::default() is Eof
        assert!(!token.is_on_new_line());
        assert!(!token.escaped());
        assert!(!token.lone_surrogates());
        assert!(!token.has_separator());
    }

    #[test]
    fn new_on_new_line_token_values() {
        let token = Token::new_on_new_line();
        assert_eq!(token.start(), 0);
        assert_eq!(token.end(), 0);
        assert_eq!(token.kind(), Kind::Eof);
        assert!(token.is_on_new_line());
        assert!(!token.escaped());
        assert!(!token.lone_surrogates());
        assert!(!token.has_separator());
    }

    #[test]
    fn token_creation_and_retrieval() {
        let kind = Kind::Ident;
        let start = 100u32;
        let end = start + 5u32;
        let is_on_new_line = true;
        let escaped = false;
        let lone_surrogates = true;
        let has_separator = false;

        let mut token = Token::default();
        token.set_kind(kind);
        token.set_start(start);
        token.set_end(end);
        token.set_is_on_new_line(is_on_new_line);
        token.set_escaped(escaped);
        token.set_lone_surrogates(lone_surrogates);
        if has_separator {
            // Assuming set_has_separator is not always called if false
            token.set_has_separator(true);
        }

        assert_eq!(token.kind(), kind);
        assert_eq!(token.start(), start);
        assert_eq!(token.end(), end);
        assert_eq!(token.is_on_new_line(), is_on_new_line);
        assert_eq!(token.escaped(), escaped);
        assert_eq!(token.lone_surrogates(), lone_surrogates);
        assert_eq!(token.has_separator(), has_separator);
    }

    #[test]
    fn token_setters() {
        let mut token = Token::default();
        token.set_kind(Kind::Ident);
        token.set_start(10);
        token.set_end(15);
        // is_on_new_line, escaped, lone_surrogates, has_separator are false by default from Token::default()

        assert_eq!(token.start(), 10);
        assert!(!token.escaped());
        assert!(!token.is_on_new_line());
        assert!(!token.lone_surrogates());

        // Test set_end
        let mut token_for_set_end = Token::default();
        token_for_set_end.set_kind(Kind::Ident);
        token_for_set_end.set_start(10);
        token_for_set_end.set_end(15);

        assert_eq!(token_for_set_end.end(), 15);
        token_for_set_end.set_end(30);
        assert_eq!(token_for_set_end.start(), 10);
        assert_eq!(token_for_set_end.end(), 30);

        // Test that other flags are not affected by set_start
        let mut token_with_flags = Token::default();
        token_with_flags.set_kind(Kind::Str);
        token_with_flags.set_start(30);
        token_with_flags.set_end(33);
        token_with_flags.set_is_on_new_line(true);
        token_with_flags.set_escaped(true);
        token_with_flags.set_lone_surrogates(true);
        token_with_flags.set_has_separator(true);

        token_with_flags.set_start(40);
        assert_eq!(token_with_flags.start(), 40);
        assert!(token_with_flags.is_on_new_line());
        assert!(token_with_flags.escaped());
        assert!(token_with_flags.lone_surrogates());
        assert!(token_with_flags.has_separator());

        // Test that other flags are not affected by set_escaped
        let mut token_with_flags2 = Token::default();
        token_with_flags2.set_kind(Kind::Str);
        token_with_flags2.set_start(50);
        token_with_flags2.set_end(52);
        token_with_flags2.set_is_on_new_line(true);
        // escaped is false by default
        token_with_flags2.set_lone_surrogates(true);
        token_with_flags2.set_has_separator(true);

        token_with_flags2.set_escaped(true);
        assert_eq!(token_with_flags2.start(), 50);
        assert!(token_with_flags2.is_on_new_line());
        assert!(token_with_flags2.escaped());
        assert!(token_with_flags2.lone_surrogates());
        assert!(token_with_flags2.has_separator());
        token_with_flags2.set_escaped(false);
        assert!(!token_with_flags2.escaped());
        assert!(token_with_flags2.is_on_new_line()); // Check again
        assert!(token_with_flags2.lone_surrogates()); // Check again
        assert!(token_with_flags2.has_separator()); // Check again

        // Test set_is_on_new_line does not affect other flags
        let mut token_flags_test_newline = Token::default();
        token_flags_test_newline.set_kind(Kind::Str);
        token_flags_test_newline.set_start(60);
        token_flags_test_newline.set_end(62);
        // is_on_new_line is false by default
        token_flags_test_newline.set_escaped(true);
        token_flags_test_newline.set_lone_surrogates(true);
        token_flags_test_newline.set_has_separator(true);

        token_flags_test_newline.set_is_on_new_line(true);
        assert!(token_flags_test_newline.is_on_new_line());
        assert_eq!(token_flags_test_newline.start(), 60);
        assert!(token_flags_test_newline.escaped());
        assert!(token_flags_test_newline.lone_surrogates());
        assert!(token_flags_test_newline.has_separator());
        token_flags_test_newline.set_is_on_new_line(false);
        assert!(!token_flags_test_newline.is_on_new_line());
        assert!(token_flags_test_newline.escaped());
        assert!(token_flags_test_newline.lone_surrogates());
        assert!(token_flags_test_newline.has_separator());

        // Test set_lone_surrogates does not affect other flags
        let mut token_flags_test_lone_surrogates = Token::default();
        token_flags_test_lone_surrogates.set_kind(Kind::Str);
        token_flags_test_lone_surrogates.set_start(70);
        token_flags_test_lone_surrogates.set_end(72);
        token_flags_test_lone_surrogates.set_is_on_new_line(true);
        token_flags_test_lone_surrogates.set_escaped(true);
        // lone_surrogates is false by default
        token_flags_test_lone_surrogates.set_has_separator(true);

        token_flags_test_lone_surrogates.set_lone_surrogates(true);
        assert!(token_flags_test_lone_surrogates.lone_surrogates());
        assert_eq!(token_flags_test_lone_surrogates.start(), 70);
        assert!(token_flags_test_lone_surrogates.is_on_new_line());
        assert!(token_flags_test_lone_surrogates.escaped());
        assert!(token_flags_test_lone_surrogates.has_separator());
        token_flags_test_lone_surrogates.set_lone_surrogates(false);
        assert!(!token_flags_test_lone_surrogates.lone_surrogates());
        assert!(token_flags_test_lone_surrogates.is_on_new_line());
        assert!(token_flags_test_lone_surrogates.escaped());
        assert!(token_flags_test_lone_surrogates.has_separator());
    }
}
