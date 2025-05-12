//! Token

use oxc_span::Span;

use super::kind::Kind;

#[derive(Debug, Clone, Copy, Default)]
pub struct Token {
    /// Token Kind
    kind: Kind,

    /// Start offset in source
    start: u32,

    /// End offset in source
    end: u32,

    /// Indicates the token is on a newline
    is_on_new_line: bool,

    /// True if the identifier / string / template kinds has escaped strings.
    /// The escaped strings are saved in [Lexer::escaped_strings] and [Lexer::escaped_templates] by
    /// [Token::start].
    ///
    /// [Lexer::escaped_strings]: [super::Lexer::escaped_strings]
    /// [Lexer::escaped_templates]: [super::Lexer::escaped_templates]
    escaped: bool,

    /// True if a string contains lone surrogates.
    lone_surrogates: bool,

    /// True if for numeric literal tokens that contain separator characters (`_`).
    ///
    /// Numeric literals are defined in Section 12.9.3 of the ECMAScript
    /// standard and include [`Kind::Decimal`], [`Kind::Binary`],
    /// [`Kind::Octal`], [`Kind::Hex`], etc.
    has_separator: bool,

    // Padding to fill to 16 bytes.
    // This makes copying a `Token` 1 x xmmword load & store, rather than 1 x dword + 1 x qword
    // and `Token::default()` is 1 x xmmword store, rather than 1 x dword + 1 x qword.
    _padding2: u16,
}

impl Token {
    #[inline]
    pub(super) fn new_on_new_line() -> Self {
        Self { is_on_new_line: true, ..Self::default() }
    }

    #[inline]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    #[inline]
    pub fn start(&self) -> u32 {
        self.start
    }

    #[cfg(test)]
    #[inline]
    fn len(self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub fn end(&self) -> u32 {
        self.end
    }

    #[inline]
    pub fn is_on_new_line(&self) -> bool {
        self.is_on_new_line
    }

    #[inline]
    pub fn escaped(&self) -> bool {
        self.escaped
    }

    #[inline]
    pub fn lone_surrogates(&self) -> bool {
        self.lone_surrogates
    }

    #[inline]
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }

    #[inline]
    pub(crate) fn has_separator(&self) -> bool {
        self.has_separator
    }

    #[inline]
    pub(crate) fn set_has_separator(&mut self) {
        self.has_separator = true;
    }

    // Helper to set kind, used in Default and potentially elsewhere
    #[inline]
    pub(crate) fn set_kind(&mut self, kind: Kind) {
        self.kind = kind;
    }

    #[inline]
    pub(crate) fn set_start(&mut self, start: u32) {
        self.start = start;
    }

    #[inline]
    pub(crate) fn set_is_on_new_line(&mut self, value: bool) {
        self.is_on_new_line = value;
    }

    #[inline]
    pub(crate) fn set_escaped(&mut self, escaped: bool) {
        self.escaped = escaped;
    }

    #[inline]
    pub(crate) fn set_lone_surrogates(&mut self, value: bool) {
        self.lone_surrogates = value;
    }

    #[inline]
    pub(crate) fn set_end(&mut self, end: u32) {
        debug_assert!(end >= self.start, "end should not be before start");
        self.end = end;
    }

    #[cfg(test)]
    #[inline]
    fn set_len(&mut self, len: u32) {
        self.end = self.start + len;
    }
}

#[cfg(test)]
mod test {
    use super::Kind;
    use super::Token;

    // Test token size
    const _: () = assert!(std::mem::size_of::<Token>() == 16);

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
