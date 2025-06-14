use memchr::memchr;

use oxc_span::Span;
use oxc_syntax::identifier::is_identifier_part;

use crate::diagnostics;

use super::{
    Kind, Lexer, Token, cold_branch,
    search::{SafeByteMatchTable, byte_search, safe_byte_match_table},
};

static NOT_ASCII_JSX_ID_CONTINUE_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'-')));

static JSX_CHILD_END_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| b == b'{' || b == b'}' || b == b'>' || b == b'<');

/// `JSXDoubleStringCharacters` ::
///   `JSXDoubleStringCharacter` `JSXDoubleStringCharactersopt`
/// `JSXDoubleStringCharacter` ::
///   `JSXStringCharacter` but not "
/// `JSXSingleStringCharacters` ::
///   `JSXSingleStringCharacter` `JSXSingleStringCharactersopt`
/// `JSXSingleStringCharacter` ::
///   `JSXStringCharacter` but not '
/// `JSXStringCharacter` ::
///   `SourceCharacter` but not one of `HTMLCharacterReference`
impl Lexer<'_> {
    /// Read JSX string literal.
    /// # SAFETY
    /// * `delimiter` must be an ASCII character.
    /// * Next char in `lexer.source` must be ASCII.
    pub(super) unsafe fn read_jsx_string_literal(&mut self, delimiter: u8) -> Kind {
        debug_assert!(delimiter.is_ascii());

        // Skip opening quote
        // SAFETY: Caller guarantees next byte is ASCII, so `.add(1)` is a UTF-8 char boundary
        let after_opening_quote = unsafe { self.source.position().add(1) };
        let remaining = self.source.str_from_pos_to_end(after_opening_quote);

        let len = memchr(delimiter, remaining.as_bytes());
        if let Some(len) = len {
            // SAFETY: `after_opening_quote` + `len` is position of delimiter.
            // Caller guarantees delimiter is ASCII, so 1 byte after it is a UTF-8 char boundary.
            let after_closing_quote = unsafe { after_opening_quote.add(len + 1) };
            self.source.set_position(after_closing_quote);
            Kind::Str
        } else {
            self.source.advance_to_end();
            self.error(diagnostics::unterminated_string(self.unterminated_range()));
            Kind::Eof
        }
    }

    pub(crate) fn next_jsx_child(&mut self) -> Token {
        self.token.set_start(self.offset());
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    /// [`JSXChild`](https://facebook.github.io/jsx/#prod-JSXChild)
    /// `JSXChild` :
    /// `JSXText`
    /// `JSXElement`
    /// `JSXFragment`
    /// { `JSXChildExpressionopt` }
    fn read_jsx_child(&mut self) -> Kind {
        match self.peek_byte() {
            Some(b'<') => {
                self.consume_char();
                Kind::LAngle
            }
            Some(b'{') => {
                self.consume_char();
                Kind::LCurly
            }
            Some(_) => {
                let next_byte = byte_search! {
                    lexer: self,
                    table: JSX_CHILD_END_TABLE,
                    handle_eof: {
                        return Kind::Eof;
                    },
                };

                if matches!(next_byte, b'<' | b'{') {
                    Kind::JSXText
                } else {
                    cold_branch(|| {
                        let start = self.offset();
                        self.error(diagnostics::unexpected_jsx_end(
                            Span::empty(start),
                            next_byte as char,
                            if next_byte == b'}' { "rbrace" } else { "gt" },
                        ));
                        Kind::Eof
                    })
                }
            }
            None => Kind::Eof,
        }
    }

    /// Expand the current `Ident` token for `JSXIdentifier`
    ///
    /// The current character is at `Ident`, continue reading for `JSXIdentifier` if it has a `-`
    ///
    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    pub(crate) fn continue_lex_jsx_identifier(&mut self) -> Option<Token> {
        if self.peek_byte() != Some(b'-') {
            return None;
        }
        self.consume_char();

        // Consume bytes which are part of identifier tail
        let next_byte = byte_search! {
            lexer: self,
            table: NOT_ASCII_JSX_ID_CONTINUE_TABLE,
            handle_eof: {
                return Some(self.finish_next(Kind::Ident));
            },
        };

        // Found a matching byte.
        // Either end of identifier found, or a Unicode char.
        if !next_byte.is_ascii() {
            // Unicode chars are rare in identifiers, so cold branch to keep common path for ASCII
            // as fast as possible
            cold_branch(|| {
                while let Some(c) = self.peek_char() {
                    if c == '-' || is_identifier_part(c) {
                        self.consume_char();
                    } else {
                        break;
                    }
                }
            });
        }

        Some(self.finish_next(Kind::Ident))
    }
}
