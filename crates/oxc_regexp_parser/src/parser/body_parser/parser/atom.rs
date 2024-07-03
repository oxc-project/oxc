use crate::{ast, parser::body_parser::unicode};

impl<'a> super::parse::PatternParser<'a> {
    // ```
    // PatternCharacter ::
    //   SourceCharacter but not SyntaxCharacter
    // ```
    // <https://tc39.es/ecma262/#prod-PatternCharacter>
    // ```
    // SyntaxCharacter :: one of
    //   ^ $ \ . * + ? ( ) [ ] { } |
    // ```
    // <https://tc39.es/ecma262/#prod-SyntaxCharacter>
    pub(super) fn consume_pattern_character(&mut self) -> Option<ast::Character> {
        let span_start = self.reader.span_position();

        let cp = self.reader.peek()?;
        if unicode::is_syntax_character(cp) {
            return None;
        }

        self.reader.advance();
        Some(ast::Character {
            span: self.span_factory.create(span_start, self.reader.span_position()),
            value: cp,
        })
    }

    pub(super) fn consume_dot(&mut self) -> Option<ast::AnyCharacterSet> {
        let span_start = self.reader.span_position();

        if !self.reader.eat('.') {
            return None;
        }

        Some(ast::AnyCharacterSet {
            span: self.span_factory.create(span_start, self.reader.span_position()),
        })
    }
}
