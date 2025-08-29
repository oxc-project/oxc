//! Convert UTF-8 span offsets to UTF-16.

use oxc_ast::ast::{Comment, Program};
use oxc_span::Span;
use oxc_syntax::module_record::{ModuleRecord, VisitMutModuleRecord};

mod converter;
mod translation;
mod visit;
pub use converter::Utf8ToUtf16Converter;
use translation::{Translation, build_translations};

/// Conversion table of UTF-8 span offsets to UTF-16.
pub struct Utf8ToUtf16 {
    translations: Vec<Translation>,
}

impl Utf8ToUtf16 {
    /// Create new [`Utf8ToUtf16`] conversion table from source text.
    pub fn new(source_text: &str) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });

        build_translations(source_text, &mut translations);

        // If no translations have been added after the first `0, 0` dummy, then source is entirely ASCII.
        // Remove the dummy entry.
        // Therefore, `translations` always has at least 2 entries, if it has any.
        if translations.len() == 1 {
            // In conformance tests, force offset conversion to happen on all inputs,
            // even if they are pure ASCII
            if cfg!(feature = "conformance") {
                translations.push(Translation { utf8_offset: u32::MAX, utf16_difference: 0 });
            } else {
                translations.clear();
            }
        }

        Self { translations }
    }

    /// Create a [`Utf8ToUtf16Converter`] converter, to convert offsets from UTF-8 to UTF-16.
    ///
    /// The converter is optimized for converting a sequence of offsets in ascending order.
    /// It will also correctly handle offsets in any order, but at a performance cost.
    ///
    /// Returns `None` if the source text is entirely ASCII, and so requires no conversion.
    pub fn converter(&self) -> Option<Utf8ToUtf16Converter<'_>> {
        if self.translations.is_empty() {
            None
        } else {
            // SAFETY: `translations` contains at least 2 entries if it's not empty.
            // We just checked it's not empty.
            Some(unsafe { Utf8ToUtf16Converter::new(&self.translations, false) })
        }
    }

    /// Convert all spans in AST to UTF-16.
    pub fn convert_program(&self, program: &mut Program<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.convert_program(program);
        }
    }

    /// Convert all spans in AST to UTF-16.
    ///
    /// Additionally, checks that conversion of offsets during traversal via [`Utf8ToUtf16Converter`]
    /// happens in ascending order of offset. Panics if it doesn't.
    ///
    /// This results in the fastest conversion, and [`Utf8ToUtf16Converter`] is designed to ensure that
    /// [`Utf8ToUtf16Converter::convert_offset`] is called with offsets strictly in ascending order.
    /// This should always be the case when the AST has come direct from parser.
    /// It might well not be the case in an AST which has been modified, e.g. by transformer or minifier.
    ///
    /// This method is for use only in conformance tests, and requires `conformance` Cargo feature.
    ///
    /// # Panics
    ///
    /// Panics if offsets are converted out of order.
    #[cfg(feature = "conformance")]
    pub fn convert_program_with_ascending_order_checks(&self, program: &mut Program<'_>) {
        assert!(self.translations.len() >= 2);

        // SAFETY: We just checked `translations` contains at least 2 entries
        let mut converter = unsafe { Utf8ToUtf16Converter::new(&self.translations, true) };
        converter.convert_program(program);
    }

    /// Convert all spans in comments to UTF-16.
    pub fn convert_comments(&self, comments: &mut [Comment]) {
        if let Some(mut converter) = self.converter() {
            for comment in comments {
                converter.convert_span(&mut comment.span);
            }
        }
    }

    /// Convert all spans in `ModuleRecord` to UTF-16.
    pub fn convert_module_record(&self, module_record: &mut ModuleRecord<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.visit_module_record(module_record);
        }
    }

    /// Convert a single UTF-16 offset back to UTF-8.
    pub fn convert_offset_back(&self, utf16_offset: &mut u32) {
        if let Some(converter) = self.converter() {
            converter.convert_offset_back(utf16_offset);
        }
    }

    /// Convert [`Span`] from UTF-16 offsets to UTF-8 offsets.
    pub fn convert_span_back(&self, span: &mut Span) {
        if let Some(converter) = self.converter() {
            converter.convert_span_back(span);
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{
        AstBuilder, Comment, CommentKind,
        ast::{Expression, Statement},
    };
    use oxc_span::{GetSpan, SourceType, Span};

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

        let span_converter = Utf8ToUtf16::new(program.source_text);
        span_converter.convert_program(&mut program);
        span_converter.convert_comments(&mut program.comments);

        assert_eq!(program.span, Span::new(0, 11));
        assert_eq!(program.body[1].span(), Span::new(1, 5));
        let Statement::ExpressionStatement(expr_stmt) = &program.body[1] else { unreachable!() };
        let Expression::StringLiteral(s) = &expr_stmt.expression else { unreachable!() };
        assert_eq!(s.span, Span::new(1, 5));
        assert_eq!(program.comments[0].span, Span::new(6, 11));

        // Check converting back from UTF-16 to UTF-8
        let convert_back = |utf16_offset: u32| {
            let mut utf8_offset = utf16_offset;
            span_converter.convert_offset_back(&mut utf8_offset);
            utf8_offset
        };

        assert_eq!(convert_back(0), 0);
        assert_eq!(convert_back(2), 2);
        assert_eq!(convert_back(4), 6);
        assert_eq!(convert_back(9), 11);
        assert_eq!(convert_back(11), 15);
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

        // Convert cases to `Vec`
        let mut cases_vec = cases
            .iter()
            .map(|&(text, translations)| (text, translations.to_vec()))
            .collect::<Vec<_>>();

        // Create 1 long string containing 99 repeats of each test case, concatenated
        let repeats = 99u32;
        let mut texts = String::new();
        for (text, _) in cases {
            for _i in 0..repeats {
                texts.push_str(text);
            }
        }

        // Generate more test cases for each of the defined cases repeated 99 times.
        // Each case references a slice of the large `texts` string.
        // Reason we do that is so that these string slices have uneven alignments, to exercise all parts
        // of `build_translations`, which handles unaligned header/tail differently from the main body
        // of the source text.
        // The number of repeats is 99, for the same reason - to ensure each string slice begins at
        // a memory address which is not evenly aligned.
        let mut offset = 0;
        for &(text, translations) in cases {
            let end_offset = offset + text.len() * (repeats as usize);
            let repeated_text = &texts[offset..end_offset];

            let (len_utf8, len_utf16) = *translations.last().unwrap();
            assert_eq!(text.len(), len_utf8 as usize);

            let mut repeated_translations = vec![];
            for i in 0..repeats {
                for &(offset_utf8, offset_utf16) in translations {
                    repeated_translations
                        .push((offset_utf8 + len_utf8 * i, offset_utf16 + len_utf16 * i));
                }
            }

            cases_vec.push((repeated_text, repeated_translations));

            offset = end_offset;
        }

        for (text, translations) in cases_vec {
            let table = Utf8ToUtf16::new(text);
            let converter = table.converter();
            if let Some(mut converter) = converter {
                // Iterate in forwards order
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }

                // Iterate in backwards order
                for &(utf8_offset, expected_utf16_offset) in translations.iter().rev() {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }

                // Convert back from UTF-16 to UTF-8
                for &(expected_utf8_offset, utf16_offset) in &translations {
                    let mut utf8_offset = utf16_offset;
                    converter.convert_offset_back(&mut utf8_offset);
                    assert_eq!(utf8_offset, expected_utf8_offset);
                }
            } else {
                // No Unicode chars. All offsets should be the same.
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    assert_eq!(utf8_offset, expected_utf16_offset);
                }
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
