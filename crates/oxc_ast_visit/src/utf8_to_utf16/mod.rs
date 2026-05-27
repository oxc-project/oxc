//! Convert UTF-8 span offsets to UTF-16.

use std::cell::Cell;

use oxc_ast::ast::{Comment, Program};
use oxc_span::Span;
use oxc_syntax::module_record::{ModuleRecord, VisitMutModuleRecord};

mod converter;
mod translation;
mod visit;
pub use converter::Utf8ToUtf16Converter;
use translation::{LineTranslation, Translation, build_translations, build_translations_and_lines};

/// Conversion table of UTF-8 span offsets to UTF-16.
pub struct Utf8ToUtf16 {
    translations: Vec<Translation>,
    /// Line-start table. `None` unless built with [`Utf8ToUtf16::new_with_lines`].
    ///
    /// When present, entry 0 always represents the start of line 0 (`utf8_offset: 0`).
    lines: Option<Vec<LineTranslation>>,
}

impl Utf8ToUtf16 {
    /// Create new [`Utf8ToUtf16`] conversion table from source text.
    pub fn new(source_text: &str) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });

        build_translations(source_text, &mut translations, 0);

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

        Self { translations, lines: None }
    }

    /// Create new [`Utf8ToUtf16`] conversion table from source text, optionally including a
    /// line-start table for `loc` lookups.
    ///
    /// When `include_lines` is `true`, the table also records the start of every line, supporting
    /// [`Self::offset_to_line_column`] and [`Self::offset_to_line_column_from_utf16`].
    pub fn new_with_lines(source_text: &str, include_lines: bool) -> Self {
        if !include_lines {
            return Self::new(source_text);
        }

        let mut translations = Vec::with_capacity(16);
        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });

        let mut lines = Vec::with_capacity(16);
        // Line 0 always starts at offset 0.
        lines.push(LineTranslation { utf8_offset: 0, utf16_difference: 0 });

        build_translations_and_lines(source_text, &mut translations, &mut lines);

        if translations.len() == 1 {
            if cfg!(feature = "conformance") {
                translations.push(Translation { utf8_offset: u32::MAX, utf16_difference: 0 });
            } else {
                translations.clear();
            }
        }

        Self { translations, lines: Some(lines) }
    }

    /// Create new [`Utf8ToUtf16`] conversion table from source text with an offset.
    ///
    /// `offset` is the number of bytes to subtract from UTF-8 offsets before converting to UTF-16.
    /// These bytes should not be part of `source_text` string.
    ///
    /// If file starts with a BOM and UTF-16 offsets should be for the source text without the BOM,
    /// pass `source_text` with the BOM trimmed from the start, and `offset` as 3 (length of BOM in UTF-8 bytes).
    pub fn new_with_offset(source_text: &str, offset: u32) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });
        translations.push(Translation { utf8_offset: offset, utf16_difference: offset });

        build_translations(source_text, &mut translations, offset);

        Self { translations, lines: None }
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

    /// Convert a UTF-8 byte offset to a 0-based `(line, column)` pair.
    ///
    /// `column` is measured in UTF-8 bytes from the start of the line.
    ///
    /// Returns `None` if line tracking was not enabled (i.e. table was not built with
    /// [`Self::new_with_lines`] with `include_lines = true`).
    pub fn offset_to_line_column(&self, utf8_offset: u32) -> Option<(u32, u32)> {
        let lines = self.lines.as_ref()?;
        Some(line_column_from_table(lines, utf8_offset, |line| line.utf8_offset))
    }

    /// Convert a UTF-16 offset to a 0-based `(line, column)` pair in UTF-16 code units.
    ///
    /// Returns `None` if line tracking was not enabled.
    ///
    /// This is more efficient than calling [`Self::convert_offset_back`] followed by
    /// [`Self::offset_to_line_column`] because it avoids the back-conversion step entirely:
    /// each line's UTF-16 start is derived from `utf8_offset - utf16_difference`.
    pub fn offset_to_line_column_from_utf16(&self, utf16_offset: u32) -> Option<(u32, u32)> {
        let lines = self.lines.as_ref()?;
        Some(line_column_from_table(lines, utf16_offset, line_start_utf16))
    }

    /// Returns `true` if this table was built with line tracking enabled.
    #[inline]
    pub fn has_lines(&self) -> bool {
        self.lines.is_some()
    }

    /// Create a cursor-based loc converter optimized for sequential UTF-16 offset lookups.
    ///
    /// The cursor advances linearly through the line table, giving O(1) amortized cost for the
    /// forward-ordered lookups typical during AST serialization. Backward jumps fall back to a
    /// binary search.
    ///
    /// Returns `None` if line tracking was not enabled.
    pub fn loc_cursor(&self) -> Option<Utf8ToUtf16LocCursor<'_>> {
        self.lines.as_deref().map(|lines| Utf8ToUtf16LocCursor { lines, cursor: Cell::new(0) })
    }
}

/// Cursor-based loc converter. Returned by [`Utf8ToUtf16::loc_cursor`].
///
/// Holds a borrow of the line table plus a `Cell<usize>` cursor that tracks the line of the
/// last lookup. Sequential forward lookups are O(1) amortized; out-of-order lookups fall back
/// to binary search.
pub struct Utf8ToUtf16LocCursor<'a> {
    lines: &'a [LineTranslation],
    cursor: Cell<usize>,
}

impl Utf8ToUtf16LocCursor<'_> {
    /// Convert a UTF-16 offset to a 0-based `(line, column)` pair in UTF-16 code units.
    pub fn offset_to_line_column_from_utf16(&self, utf16_offset: u32) -> (u32, u32) {
        if self.lines.is_empty() {
            return (0, utf16_offset);
        }

        let cursor = self.cursor.get();
        let current_start = line_start_utf16(self.lines[cursor]);

        let new_cursor = if utf16_offset >= current_start {
            self.advance_forward(utf16_offset, cursor)
        } else {
            self.binary_search(utf16_offset)
        };

        self.cursor.set(new_cursor);

        let line_start = line_start_utf16(self.lines[new_cursor]);
        #[expect(clippy::cast_possible_truncation)]
        let line = new_cursor as u32;
        (line, utf16_offset.wrapping_sub(line_start))
    }

    /// Linear scan forward up to [`LINEAR_ITERATIONS`] steps before falling back to binary search.
    #[inline]
    fn advance_forward(&self, utf16_offset: u32, mut cursor: usize) -> usize {
        const LINEAR_ITERATIONS: usize = 8;

        let linear_end = (cursor + LINEAR_ITERATIONS + 1).min(self.lines.len());
        while cursor + 1 < linear_end {
            if utf16_offset >= line_start_utf16(self.lines[cursor + 1]) {
                cursor += 1;
            } else {
                return cursor;
            }
        }

        if cursor + 1 >= self.lines.len() {
            return cursor;
        }

        // Binary search over the rest, including `cursor`'s neighbour onwards.
        let search = &self.lines[cursor + 1..];
        let rel = search.partition_point(|line| line_start_utf16(*line) <= utf16_offset);
        cursor + rel
    }

    #[cold]
    fn binary_search(&self, utf16_offset: u32) -> usize {
        self.lines.partition_point(|line| line_start_utf16(*line) <= utf16_offset).saturating_sub(1)
    }
}

/// UTF-16 offset of the start of `line`.
#[inline]
fn line_start_utf16(line: LineTranslation) -> u32 {
    line.utf8_offset.wrapping_sub(line.utf16_difference)
}

/// Locate the line containing `offset` in `table`, then return `(line_index, column)`.
///
/// `line_start` extracts each line's start offset in the same units as `offset` (UTF-8 bytes or
/// UTF-16 code units), so this helper backs both
/// [`Utf8ToUtf16::offset_to_line_column`] and [`Utf8ToUtf16::offset_to_line_column_from_utf16`].
#[inline]
fn line_column_from_table(
    table: &[LineTranslation],
    offset: u32,
    line_start: impl Fn(LineTranslation) -> u32,
) -> (u32, u32) {
    if table.is_empty() {
        return (0, offset);
    }

    // `partition_point` returns the first index where the predicate is false. Lines are
    // monotonically increasing by `line_start`, so this gives one past the line we want.
    let line_index = table.partition_point(|line| line_start(*line) <= offset).saturating_sub(1);

    let start = line_start(table[line_index]);
    #[expect(clippy::cast_possible_truncation)]
    let line = line_index as u32;
    (line, offset.wrapping_sub(start))
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

    #[test]
    fn line_tracking_disabled_by_default() {
        let table = Utf8ToUtf16::new("a\nb");
        assert_eq!(table.offset_to_line_column(0), None);
        assert_eq!(table.offset_to_line_column_from_utf16(0), None);
        assert!(table.loc_cursor().is_none());
    }

    #[test]
    fn line_tracking_lf() {
        let table = Utf8ToUtf16::new_with_lines("hello\nworld", true);
        assert_eq!(table.offset_to_line_column(0), Some((0, 0)));
        assert_eq!(table.offset_to_line_column(5), Some((0, 5))); // \n itself belongs to line 0
        assert_eq!(table.offset_to_line_column(6), Some((1, 0))); // 'w'
        assert_eq!(table.offset_to_line_column(11), Some((1, 5))); // EOF
    }

    #[test]
    fn line_tracking_crlf_treated_as_one_break() {
        let table = Utf8ToUtf16::new_with_lines("a\r\nb\r\nc", true);
        // After \r\n: 'b' is at line 1 col 0, not line 2.
        assert_eq!(table.offset_to_line_column(3), Some((1, 0)));
        assert_eq!(table.offset_to_line_column(6), Some((2, 0)));
    }

    #[test]
    fn line_tracking_cr_only() {
        let table = Utf8ToUtf16::new_with_lines("a\rb\rc", true);
        assert_eq!(table.offset_to_line_column(2), Some((1, 0)));
        assert_eq!(table.offset_to_line_column(4), Some((2, 0)));
    }

    #[test]
    fn line_tracking_unicode_separators() {
        // U+2028 (LS) and U+2029 (PS) are 3 UTF-8 bytes each, 1 UTF-16 code unit each.
        let table = Utf8ToUtf16::new_with_lines("a\u{2028}b\u{2029}c", true);
        // UTF-8 byte layout: a(0) E2 80 A8 (1-3) b(4) E2 80 A9 (5-7) c(8)
        assert_eq!(table.offset_to_line_column(4), Some((1, 0)));
        assert_eq!(table.offset_to_line_column(8), Some((2, 0)));
    }

    #[test]
    fn line_tracking_utf16_column_with_emoji() {
        // 🤨 is 4 UTF-8 bytes, 2 UTF-16 code units.
        let table = Utf8ToUtf16::new_with_lines("let \u{1F928}\n= 1", true);
        // UTF-16 layout: l(0) e(1) t(2) _(3) hi(4) lo(5) \n(6) =(7) _(8) 1(9)
        assert_eq!(table.offset_to_line_column_from_utf16(0), Some((0, 0)));
        assert_eq!(table.offset_to_line_column_from_utf16(5), Some((0, 5)));
        assert_eq!(table.offset_to_line_column_from_utf16(7), Some((1, 0)));
    }

    #[test]
    fn line_tracking_pure_ascii_long_text() {
        use std::fmt::Write;

        // Exercise the SIMD aligned-chunk path with line breaks in multiple chunks.
        let mut source = String::new();
        for i in 0..200 {
            writeln!(source, "line{i}").unwrap();
        }
        let table = Utf8ToUtf16::new_with_lines(&source, true);

        assert_eq!(table.offset_to_line_column(0), Some((0, 0)));

        // Find offset just past line 100's terminator.
        let mut prefix = String::new();
        for i in 0..100 {
            writeln!(prefix, "line{i}").unwrap();
        }
        #[expect(clippy::cast_possible_truncation)]
        let offset = prefix.len() as u32;
        assert_eq!(table.offset_to_line_column(offset), Some((100, 0)));
    }

    #[test]
    fn loc_cursor_matches_binary_search() {
        let source = "alpha\nbeta\ngamma\ndelta";
        let table = Utf8ToUtf16::new_with_lines(source, true);
        let cursor = table.loc_cursor().unwrap();

        // Forward sequential access (the optimized path).
        for offset in 0..=u32::try_from(source.len()).unwrap() {
            let (line, col) = cursor.offset_to_line_column_from_utf16(offset);
            let expected = table.offset_to_line_column_from_utf16(offset).unwrap();
            assert_eq!((line, col), expected, "forward offset {offset}");
        }

        // Mixed/backward access (falls back to binary search).
        for &offset in &[20u32, 0, 11, 5, 17] {
            let (line, col) = cursor.offset_to_line_column_from_utf16(offset);
            let expected = table.offset_to_line_column_from_utf16(offset).unwrap();
            assert_eq!((line, col), expected, "mixed offset {offset}");
        }
    }
}
