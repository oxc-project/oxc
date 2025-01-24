//! Convert UTF-8 span offsets to UTF-16.

use oxc_span::Span;

use crate::{ast::Program, visit::VisitMut};

/// Convert UTF-8 span offsets to UTF-16.
pub struct Utf8ToUtf16 {
    translations: Vec<Translation>,
}

#[derive(Clone, Copy)]
#[repr(align(8))]
struct Translation {
    // UTF-8 byte offset
    utf8_offset: u32,
    // Number to subtract from UTF-8 byte offset to get UTF-16 char offset
    // for offsets *after* `utf8_offset`
    utf16_difference: u32,
}

impl Utf8ToUtf16 {
    /// Create new `Utf8ToUtf16` converter.
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut translations = Vec::with_capacity(16);
        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });
        Self { translations }
    }

    /// Convert all spans in the AST to UTF-16.
    pub fn convert(mut self, program: &mut Program<'_>) {
        self.build_table(program.source_text);
        // Skip if source is entirely ASCII
        if self.translations.len() == 1 {
            return;
        }
        self.visit_program(program);
        for comment in &mut program.comments {
            self.convert_span(&mut comment.span);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn build_table(&mut self, source_text: &str) {
        // Translation from UTF-8 byte offset to UTF-16 char offset:
        //
        // * 1-byte UTF-8 sequence
        //   = 1st byte 0xxxxxxx (0 - 0x7F)
        //   -> 1 x UTF-16 char
        //   UTF-16 len = UTF-8 len
        // * 2-byte UTF-8 sequence
        //   = 1st byte 110xxxxx (0xC0 - 0xDF), remaining bytes 10xxxxxx (0x80 - 0xBF)
        //   -> 1 x UTF-16
        //   UTF-16 len = UTF-8 len - 1
        // * 3-byte UTF-8 sequence
        //   = 1st byte 1110xxxx (0xE0 - 0xEF), remaining bytes 10xxxxxx (0x80 - 0xBF)
        //   -> 1 x UTF-16
        //   UTF-16 len = UTF-8 len - 2
        // * 4-byte UTF-8 sequence
        //   = 1st byte 1111xxxx (0xF0 - 0xFF), remaining bytes 10xxxxxx (0x80 - 0xBF)
        //   -> 2 x UTF-16
        //   UTF-16 len = UTF-8 len - 2
        //
        // So UTF-16 offset = UTF-8 offset - count of bytes `>= 0xC0` - count of bytes `>= 0xE0`
        let mut utf16_difference = 0;
        for (utf8_offset, &byte) in source_text.as_bytes().iter().enumerate() {
            if byte >= 0xC0 {
                let difference_for_this_byte = u32::from(byte >= 0xE0) + 1;
                utf16_difference += difference_for_this_byte;
                // Record `utf8_offset + 1` not `utf8_offset`, because it's only offsets *after* this
                // Unicode character that need to be shifted
                self.translations
                    .push(Translation { utf8_offset: utf8_offset as u32 + 1, utf16_difference });
            }
        }
    }

    fn convert_span(&self, span: &mut Span) {
        span.start = self.convert_offset(span.start);
        span.end = self.convert_offset(span.end);
    }

    fn convert_offset(&self, utf8_offset: u32) -> u32 {
        // Find the first entry in table *after* the UTF-8 offset.
        // The difference we need to subtract is recorded in the entry prior to it.
        let index =
            self.translations.partition_point(|translation| translation.utf8_offset <= utf8_offset);
        // First entry in table is `0, 0`. `partition_point` finds the first entry where
        // `utf8_offset < translation.utf8_offset` (or `translations.len()` if none exists).
        // So guaranteed `index > 0`, and `index <= translations.len()`.
        // Therefore `index - 1` cannot wrap around, and cannot be out of bounds.
        let translation = self.translations[index - 1];
        utf8_offset - translation.utf16_difference
    }
}

impl VisitMut<'_> for Utf8ToUtf16 {
    fn visit_span(&mut self, span: &mut Span) {
        self.convert_span(span);
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_span::{GetSpan, SourceType, Span};

    use crate::{
        ast::{Expression, Statement},
        AstBuilder, Comment, CommentKind,
    };

    use super::Utf8ToUtf16;

    #[test]
    fn translate_ast() {
        let allocator = Allocator::new();
        let ast = AstBuilder::new(&allocator);

        let mut program = ast.program(
            Span::new(0, 15),
            SourceType::default(),
            ";'🤨' // 🤨",
            ast.vec1(Comment::new(8, 15, CommentKind::Line)),
            None,
            ast.vec(),
            ast.vec_from_array([
                ast.statement_empty(Span::new(0, 1)),
                ast.statement_expression(
                    Span::new(1, 7),
                    ast.expression_string_literal(Span::new(1, 7), "🤨", None),
                ),
            ]),
        );

        Utf8ToUtf16::new().convert(&mut program);
        assert_eq!(program.span, Span::new(0, 11));
        assert_eq!(program.body[1].span(), Span::new(1, 5));
        let Statement::ExpressionStatement(expr_stmt) = &program.body[1] else { unreachable!() };
        let Expression::StringLiteral(s) = &expr_stmt.expression else { unreachable!() };
        assert_eq!(s.span, Span::new(1, 5));
        assert_eq!(program.comments[0].span, Span::new(6, 11));
    }

    #[test]
    fn translate_offsets() {
        assert_eq!('_'.len_utf8(), 1);
        assert_eq!('_'.len_utf16(), 1);
        assert_eq!('£'.len_utf8(), 2);
        assert_eq!('£'.len_utf16(), 1);
        assert_eq!('ऊ'.len_utf8(), 3);
        assert_eq!('ऊ'.len_utf16(), 1);
        assert_eq!('🤨'.len_utf8(), 4);
        assert_eq!('🤨'.len_utf16(), 2);

        let cases: &[(&str, &[(u32, u32)])] = &[
            // 1-byte
            ("_", &[(0, 0), (1, 1)]),
            // 2-byte
            ("£", &[(0, 0), (2, 1)]),
            ("£_", &[(0, 0), (2, 1), (3, 2)]),
            ("_£", &[(0, 0), (1, 1), (3, 2)]),
            ("_£_", &[(0, 0), (1, 1), (3, 2), (4, 3)]),
            ("_££_", &[(0, 0), (1, 1), (3, 2), (5, 3), (6, 4)]),
            ("_£_£_", &[(0, 0), (1, 1), (3, 2), (4, 3), (6, 4), (7, 5)]),
            // 3-byte
            ("ऊ", &[(0, 0), (3, 1)]),
            ("ऊ_", &[(0, 0), (3, 1), (4, 2)]),
            ("_ऊ", &[(0, 0), (1, 1), (4, 2)]),
            ("_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3)]),
            ("_ऊऊ_", &[(0, 0), (1, 1), (4, 2), (7, 3), (8, 4)]),
            ("_ऊ_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3), (8, 4), (9, 5)]),
            // 4-byte
            ("🤨", &[(0, 0), (4, 2)]),
            ("🤨_", &[(0, 0), (4, 2), (5, 3)]),
            ("_🤨", &[(0, 0), (1, 1), (5, 3)]),
            ("_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4)]),
            ("_🤨🤨_", &[(0, 0), (1, 1), (5, 3), (9, 5), (10, 6)]),
            ("_🤨_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4), (10, 6), (11, 7)]),
        ];

        for (text, translations) in cases {
            let mut converter = Utf8ToUtf16::new();
            converter.build_table(text);
            for &(utf8_offset, expected_utf16_offset) in *translations {
                assert_eq!(converter.convert_offset(utf8_offset), expected_utf16_offset);
            }
        }
    }

    // Check assumptions about how many UTF-16 chars result from different UTF-8 character sequences,
    // which are relied on by `build_table`
    #[test]
    fn char_lengths() {
        macro_rules! assert_utf8_bytes_eq {
            ($c:expr, $bytes:expr) => {{
                let mut buffer = [0; 4];
                let bytes = $c.encode_utf8(&mut buffer).as_bytes();
                assert!($bytes == bytes);
            }};
        }

        // All 1-byte UTF-8 character sequences = 1 x UTF-16 character.
        // First byte is 0x00 - 0x7F.
        let min_1_byte_char = char::from_u32(0).unwrap();
        assert_eq!(min_1_byte_char.len_utf8(), 1);
        assert_eq!(min_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_1_byte_char, [0x00]);
        let max_1_byte_char = char::from_u32(0x7F).unwrap();
        assert_eq!(max_1_byte_char.len_utf8(), 1);
        assert_eq!(max_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_1_byte_char, [0x7F]);

        // All 2-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xC2 - 0xDF.
        let min_2_byte_char = char::from_u32(0x80).unwrap();
        assert_eq!(min_2_byte_char.len_utf8(), 2);
        assert_eq!(min_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_2_byte_char, [0xC2, 0x80]);
        let max_2_byte_char = char::from_u32(0x7FF).unwrap();
        assert_eq!(max_2_byte_char.len_utf8(), 2);
        assert_eq!(max_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_2_byte_char, [0xDF, 0xBF]);

        // All 3-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xE0 - 0xEF.
        let min_3_byte_char = char::from_u32(0x800).unwrap();
        assert_eq!(min_3_byte_char.len_utf8(), 3);
        assert_eq!(min_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_3_byte_char, [0xE0, 0xA0, 0x80]);
        let max_3_byte_char = char::from_u32(0xFFFF).unwrap();
        assert_eq!(max_3_byte_char.len_utf8(), 3);
        assert_eq!(max_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_3_byte_char, [0xEF, 0xBF, 0xBF]);

        // All 4-byte UTF-8 character sequences = 2 x UTF-16 characters
        // First byte is 0xF0 - 0xF4.
        let min_4_byte_char = char::from_u32(0x10000).unwrap();
        assert_eq!(min_4_byte_char.len_utf8(), 4);
        assert_eq!(min_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(min_4_byte_char, [0xF0, 0x90, 0x80, 0x80]);
        let max_4_byte_char = char::MAX;
        assert_eq!(max_4_byte_char.len_utf8(), 4);
        assert_eq!(max_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(max_4_byte_char, [0xF4, 0x8F, 0xBF, 0xBF]);
    }
}
