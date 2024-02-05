use super::{AutoCow, Kind, Lexer, Token};
use crate::diagnostics;

use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

impl<'a> Lexer<'a> {
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
    pub(super) fn read_jsx_string_literal(&mut self, delimiter: char) -> Kind {
        let mut builder = AutoCow::new(self);
        loop {
            match self.next_char() {
                Some(c @ ('"' | '\'')) => {
                    if c == delimiter {
                        self.save_string(builder.has_escape(), builder.finish_without_push(self));
                        return Kind::Str;
                    }
                    builder.push_matching(c);
                }
                Some(other) => {
                    builder.push_matching(other);
                }
                None => {
                    self.error(diagnostics::UnterminatedString(self.unterminated_range()));
                    return Kind::Undetermined;
                }
            }
        }
    }

    pub(crate) fn next_jsx_child(&mut self) -> Token {
        self.token.start = self.offset();
        let kind = self.read_jsx_child();
        self.finish_next(kind)
    }

    /// Expand the current token for `JSXIdentifier`
    pub(crate) fn next_jsx_identifier(&mut self, start_offset: u32) -> Token {
        let kind = self.read_jsx_identifier(start_offset);
        self.lookahead.clear();
        self.finish_next(kind)
    }

    /// [`JSXChild`](https://facebook.github.io/jsx/#prod-JSXChild)
    /// `JSXChild` :
    /// `JSXText`
    /// `JSXElement`
    /// `JSXFragment`
    /// { `JSXChildExpressionopt` }
    fn read_jsx_child(&mut self) -> Kind {
        match self.peek() {
            Some('<') => {
                self.consume_char();
                Kind::LAngle
            }
            Some('{') => {
                self.consume_char();
                Kind::LCurly
            }
            Some(_) => {
                loop {
                    // The tokens `{`, `<`, `>` and `}` cannot appear in a jsx text.
                    // The TypeScript compiler raises the error "Unexpected token. Did you mean `{'>'}` or `&gt;`?".
                    // Where as the Babel compiler does not raise any errors.
                    // The following check omits `>` and `}` so that more Babel tests can be passed.
                    if self.peek().is_some_and(|c| c == '{' || c == '<') {
                        break;
                    }
                    if self.next_char().is_none() {
                        break;
                    }
                }
                Kind::JSXText
            }
            None => Kind::Eof,
        }
    }

    /// `JSXIdentifier` :
    ///   `IdentifierStart`
    ///   `JSXIdentifier` `IdentifierPart`
    ///   `JSXIdentifier` [no `WhiteSpace` or Comment here] -
    fn read_jsx_identifier(&mut self, _start_offset: u32) -> Kind {
        while let Some(c) = self.peek() {
            if c == '-' || is_identifier_start(c) {
                self.consume_char();
                while let Some(c) = self.peek() {
                    if is_identifier_part(c) {
                        self.consume_char();
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        Kind::Ident
    }
}
