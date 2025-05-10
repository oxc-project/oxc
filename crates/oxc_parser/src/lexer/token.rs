//! Token

use oxc_span::Span;

use super::kind::Kind;

// Bit layout for u64:
// - Bits 0-31 (32 bits): `start`
// - Bits 32-39 (8 bits): `kind` (as u8)
// - Bit 40 (1 bit): `is_on_new_line`
// - Bits 41-60 (20 bits): `len`
// - Bit 61 (1 bit): `escaped`
// - Bit 62 (1 bit): `lone_surrogates`
// - Bit 63 (1 bit): `has_separator`

const START_SHIFT: u32 = 0;
const KIND_SHIFT: u32 = 32;
const IS_ON_NEW_LINE_SHIFT: u32 = 40;
const LEN_SHIFT: u32 = 41; // Adjusted Shift
const ESCAPED_SHIFT: u32 = 61; // Adjusted Shift
const LONE_SURROGATES_SHIFT: u32 = 62; // Adjusted Shift
const HAS_SEPARATOR_SHIFT: u32 = 63; // Adjusted Shift

const START_MASK: u64 = 0xFFFF_FFFF; // 32 bits
const KIND_MASK: u64 = 0xFF; // 8 bits
const LEN_MASK: u64 = 0xF_FFFF; // 20 bits (0xFFFFF)

const IS_ON_NEW_LINE_FLAG: u64 = 1 << IS_ON_NEW_LINE_SHIFT;
const ESCAPED_FLAG: u64 = 1 << ESCAPED_SHIFT; // Will use new ESCAPED_SHIFT
const LONE_SURROGATES_FLAG: u64 = 1 << LONE_SURROGATES_SHIFT; // Will use new LONE_SURROGATES_SHIFT
const HAS_SEPARATOR_FLAG: u64 = 1 << HAS_SEPARATOR_SHIFT; // Will use new HAS_SEPARATOR_SHIFT

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Token(u64);

impl Default for Token {
    fn default() -> Self {
        let mut token = Self(0);
        // Kind::default() is Kind::Eof. Kind::Eof as u8 needs to be set.
        // Assuming Kind::Eof will be 1 after #[repr(u8)] (Undetermined = 0, Eof = 1)
        token.set_kind(Kind::default());
        token
    }
}

impl Token {
    pub(super) fn new_on_new_line() -> Self {
        // Start with a default token, then set the flag.
        let mut token = Self::default();
        token.0 |= IS_ON_NEW_LINE_FLAG;
        token
    }

    #[inline]
    pub fn kind(&self) -> Kind {
        // SAFETY: This conversion is safe because Kind is #[repr(u8)] and we ensure the value stored is a valid Kind variant.
        unsafe { std::mem::transmute(((self.0 >> KIND_SHIFT) & KIND_MASK) as u8) }
    }

    #[inline]
    pub fn start(&self) -> u32 {
        ((self.0 >> START_SHIFT) & START_MASK) as u32
    }

    #[inline]
    fn len(self) -> u32 {
        ((self.0 >> LEN_SHIFT) & LEN_MASK) as u32
    }

    #[inline]
    pub fn end(&self) -> u32 {
        self.start() + self.len()
    }

    #[inline]
    pub fn is_on_new_line(&self) -> bool {
        (self.0 & IS_ON_NEW_LINE_FLAG) != 0
    }

    #[inline]
    pub fn escaped(&self) -> bool {
        (self.0 & ESCAPED_FLAG) != 0
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        (self.0 & LONE_SURROGATES_FLAG) != 0
    }

    pub fn span(&self) -> Span {
        Span::new(self.start(), self.end())
    }

    #[inline]
    pub fn has_separator(&self) -> bool {
        (self.0 & HAS_SEPARATOR_FLAG) != 0
    }

    pub(crate) fn set_has_separator(&mut self) {
        self.0 |= HAS_SEPARATOR_FLAG;
    }

    // Helper to set kind, used in Default and potentially elsewhere
    pub(crate) fn set_kind(&mut self, kind: Kind) {
        self.0 &= !(KIND_MASK << KIND_SHIFT); // Clear current kind bits
        self.0 |= u64::from(kind as u8) << KIND_SHIFT;
    }

    pub(crate) fn set_start(&mut self, start: u32) {
        self.0 &= !(START_MASK << START_SHIFT); // Clear current start bits
        self.0 |= u64::from(start) << START_SHIFT;
    }

    pub(crate) fn set_is_on_new_line(&mut self, value: bool) {
        if value {
            self.0 |= IS_ON_NEW_LINE_FLAG;
        } else {
            self.0 &= !IS_ON_NEW_LINE_FLAG;
        }
    }

    pub(crate) fn set_escaped(&mut self, escaped: bool) {
        if escaped {
            self.0 |= ESCAPED_FLAG;
        } else {
            self.0 &= !ESCAPED_FLAG;
        }
    }

    pub(crate) fn set_lone_surrogates(&mut self, value: bool) {
        if value {
            self.0 |= LONE_SURROGATES_FLAG;
        } else {
            self.0 &= !LONE_SURROGATES_FLAG;
        }
    }

    pub(crate) fn set_end(&mut self, end: u32) {
        let start = self.start();
        debug_assert!(end >= start, "end should not be before start");
        self.set_len(end - start);
    }

    fn set_len(&mut self, len: u32) {
        self.0 &= !(LEN_MASK << LEN_SHIFT); // Clear current len bits
        self.0 |= u64::from(len) << LEN_SHIFT; // Ensure len is cast to u64 before shifting
    }
}

#[cfg(test)]
mod size_asserts {
    use super::Kind;
    use super::Token;
    // Test new size
    const _: () = assert!(std::mem::size_of::<Token>() == 8);

    // Test default token values
    #[test]
    fn default_token_values() {
        let token = Token::default();
        assert_eq!(token.start(), 0);
        assert_eq!(token.len(), 0u32);
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
        assert_eq!(token.len(), 0u32);
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
        let len = 5u32;
        let is_on_new_line = true;
        let escaped = false;
        let lone_surrogates = true;
        let has_separator = false;

        let mut token = Token::default();
        token.set_kind(kind);
        token.set_start(start);
        token.set_len(len);
        token.set_is_on_new_line(is_on_new_line);
        token.set_escaped(escaped);
        token.set_lone_surrogates(lone_surrogates);
        if has_separator {
            // Assuming set_has_separator is not always called if false
            token.set_has_separator();
        }

        assert_eq!(token.kind(), kind);
        assert_eq!(token.start(), start);
        assert_eq!(token.len(), len);
        assert_eq!(token.end(), start + len);
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
        token.set_len(5);
        // is_on_new_line, escaped, lone_surrogates, has_separator are false by default from Token::default()

        assert_eq!(token.start(), 10);
        assert!(!token.escaped());
        assert!(!token.is_on_new_line());
        assert!(!token.lone_surrogates());

        // Test set_end
        let mut token_for_set_end = Token::default();
        token_for_set_end.set_kind(Kind::Ident);
        token_for_set_end.set_start(10);
        token_for_set_end.set_len(5);

        assert_eq!(token_for_set_end.end(), 15);
        token_for_set_end.set_end(30);
        assert_eq!(token_for_set_end.start(), 10);
        assert_eq!(token_for_set_end.len(), 20u32);
        assert_eq!(token_for_set_end.end(), 30);

        // Test that other flags are not affected by set_start
        let mut token_with_flags = Token::default();
        token_with_flags.set_kind(Kind::Str);
        token_with_flags.set_start(30);
        token_with_flags.set_len(3);
        token_with_flags.set_is_on_new_line(true);
        token_with_flags.set_escaped(true);
        token_with_flags.set_lone_surrogates(true);
        token_with_flags.set_has_separator();

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
        token_with_flags2.set_len(2);
        token_with_flags2.set_is_on_new_line(true);
        // escaped is false by default
        token_with_flags2.set_lone_surrogates(true);
        token_with_flags2.set_has_separator();

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
        token_flags_test_newline.set_len(2);
        // is_on_new_line is false by default
        token_flags_test_newline.set_escaped(true);
        token_flags_test_newline.set_lone_surrogates(true);
        token_flags_test_newline.set_has_separator();

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
        token_flags_test_lone_surrogates.set_len(2);
        token_flags_test_lone_surrogates.set_is_on_new_line(true);
        token_flags_test_lone_surrogates.set_escaped(true);
        // lone_surrogates is false by default
        token_flags_test_lone_surrogates.set_has_separator();

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
