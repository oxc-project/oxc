use std::sync::Arc;

use oxc_span::Span;
use oxc_syntax::identifier::{LS, PS};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_OR_PS_SECOND: u8 = 0x80;
const LS_THIRD: u8 = 0xA8;
const PS_THIRD: u8 = 0xA9;

/// Line offset table
///
/// Used for tracking lines and columns from byte offsets via binary search.
///
/// Code is adapted from [esbuild](https://github.com/evanw/esbuild/blob/cc74e6042a9f573bf58e1e3f165ebda70af4ad3b/internal/js_printer/js_printer.go#L4806-L4808)
#[derive(Debug)]
pub struct LineOffsetTable {
    columns: Option<Vec<usize>>,
    byte_offset_to_first: usize,
    byte_offset_to_start_of_line: usize,
}

#[allow(clippy::struct_field_names)]
pub struct SourcemapBuilder {
    source_id: u32,
    original_source: Arc<str>,
    last_generated_update: usize,
    last_position: Option<u32>,
    last_search_line: usize,
    line_offset_tables: Vec<LineOffsetTable>,
    sourcemap_builder: oxc_sourcemap::SourceMapBuilder,
    generated_line: u32,
    generated_column: u32,
}

impl Default for SourcemapBuilder {
    fn default() -> Self {
        Self {
            source_id: 0,
            original_source: "".into(),
            last_generated_update: 0,
            last_position: None,
            last_search_line: 0,
            line_offset_tables: vec![],
            sourcemap_builder: oxc_sourcemap::SourceMapBuilder::default(),
            generated_line: 0,
            generated_column: 0,
        }
    }
}

impl SourcemapBuilder {
    pub fn with_name_and_source(&mut self, name: &str, source: &str) {
        self.line_offset_tables = Self::generate_line_offset_tables(source);
        self.source_id = self.sourcemap_builder.set_source_and_content(name, source);
        self.original_source = source.into();
    }

    pub fn into_sourcemap(self) -> oxc_sourcemap::SourceMap {
        self.sourcemap_builder.into_sourcemap()
    }

    pub fn add_source_mapping_for_name(&mut self, output: &[u8], span: Span, name: &str) {
        debug_assert!(
            (span.end as usize) <= self.original_source.len(),
            "violated {}:{} <= {} for {name}",
            span.start,
            span.end,
            self.original_source.len()
        );
        let original_name = self.original_source.get(span.start as usize..span.end as usize);
        // The token name should be original name.
        // If it hasn't change, name should be `None` to reduce `SourceMap` size.
        let token_name =
            if original_name == Some(name) { None } else { original_name.map(Into::into) };
        self.add_source_mapping(output, span.start, token_name);
    }

    pub fn add_source_mapping(&mut self, output: &[u8], position: u32, name: Option<Arc<str>>) {
        if matches!(self.last_position, Some(last_position) if last_position >= position) {
            return;
        }
        let (original_line, original_column) = self.search_original_line_and_column(position);
        self.update_generated_line_and_column(output);
        let name_id = name.map(|s| self.sourcemap_builder.add_name(&s));
        self.sourcemap_builder.add_token(
            self.generated_line,
            self.generated_column,
            original_line,
            original_column,
            Some(self.source_id),
            name_id,
        );
        self.last_position = Some(position);
    }

    #[allow(clippy::cast_possible_truncation)]
    fn search_original_line_and_column(&mut self, position: u32) -> (u32, u32) {
        let mut original_line = 0;
        for i in self.last_search_line..self.line_offset_tables.len() {
            if self.line_offset_tables[i].byte_offset_to_start_of_line > position as usize {
                original_line = i - 1;
                break;
            }
            if i == self.line_offset_tables.len() - 1 {
                original_line = i;
            }
        }
        self.last_search_line = original_line;
        let line = &self.line_offset_tables[original_line];
        let mut original_column = (position as usize) - line.byte_offset_to_start_of_line;
        if original_column >= line.byte_offset_to_first {
            if let Some(cols) = &line.columns {
                original_column = cols[original_column - line.byte_offset_to_first];
            }
        }
        (original_line as u32, original_column as u32)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_generated_line_and_column(&mut self, output: &[u8]) {
        let remaining = &output[self.last_generated_update..];

        // Find last line break
        let mut line_start_ptr = remaining.as_ptr();
        let mut last_line_is_ascii = true;
        let mut iter = remaining.iter();
        while let Some(&b) = iter.next() {
            match b {
                b'\n' => {}
                b'\r' => {
                    // Handle Windows-specific "\r\n" newlines
                    if iter.clone().next() == Some(&b'\n') {
                        iter.next();
                    }
                }
                _ if b.is_ascii() => {
                    continue;
                }
                LS_OR_PS_FIRST => {
                    let next_byte = *iter.next().unwrap();
                    let next_next_byte = *iter.next().unwrap();
                    if next_byte != LS_OR_PS_SECOND
                        || !matches!(next_next_byte, LS_THIRD | PS_THIRD)
                    {
                        last_line_is_ascii = false;
                        continue;
                    }
                }
                _ => {
                    // Unicode char
                    last_line_is_ascii = false;
                    continue;
                }
            }

            // Line break found.
            // `iter` is now positioned after line break.
            line_start_ptr = iter.as_slice().as_ptr();
            self.generated_line += 1;
            self.generated_column = 0;
            last_line_is_ascii = true;
        }

        // Calculate column
        self.generated_column += if last_line_is_ascii {
            // `iter` is now exhausted, so `iter.as_slice().as_ptr()` is pointer to end of `output`
            (iter.as_slice().as_ptr() as usize - line_start_ptr as usize) as u32
        } else {
            let line_byte_offset = line_start_ptr as usize - remaining.as_ptr() as usize;
            // TODO: It'd be better if could use `from_utf8_unchecked` here, but we'd need to make this
            // function unsafe and caller guarantees `output` contains a valid UTF-8 string
            let last_line = std::str::from_utf8(&remaining[line_byte_offset..]).unwrap();
            // Mozilla's "source-map" library counts columns using UTF-16 code units
            last_line.encode_utf16().count() as u32
        };
        self.last_generated_update = output.len();
    }

    fn generate_line_offset_tables(content: &str) -> Vec<LineOffsetTable> {
        let mut tables = vec![];

        // Process content line-by-line.
        // For each line, start by assuming line will be entirely ASCII, and read byte-by-byte.
        // If line is all ASCII, UTF-8 columns and UTF-16 columns are the same,
        // so no need to create a `columns` Vec. This is the fast path for common case.
        // If a Unicode character found, read rest of line char-by-char, populating `columns` Vec.
        // At end of line, go back to top of outer loop, and again assume ASCII for next line.
        let mut line_byte_offset = 0;
        'lines: loop {
            tables.push(LineOffsetTable {
                columns: None,
                // `usize::MAX` so `original_column >= line.byte_offset_to_first` check in
                // `search_original_line_and_column` fails if line is all ASCII
                byte_offset_to_first: usize::MAX,
                byte_offset_to_start_of_line: line_byte_offset,
            });

            let remaining = &content.as_bytes()[line_byte_offset..];
            for (mut byte_offset_from_line_start, b) in remaining.iter().enumerate() {
                match b {
                    b'\n' => {
                        byte_offset_from_line_start += 1;
                    }
                    b'\r' => {
                        byte_offset_from_line_start += 1;
                        // Handle Windows-specific "\r\n" newlines
                        if remaining.get(byte_offset_from_line_start) == Some(&b'\n') {
                            byte_offset_from_line_start += 1;
                        }
                    }
                    _ if b.is_ascii() => {
                        continue;
                    }
                    _ => {
                        // Unicode char found.
                        // Create `columns` Vec, and set `byte_offset_to_first`.
                        let table = tables.iter_mut().last().unwrap();
                        table.byte_offset_to_first = byte_offset_from_line_start;
                        table.columns = Some(vec![]);
                        let columns = table.columns.as_mut().unwrap();

                        // Loop through rest of line char-by-char.
                        // `chunk_byte_offset` in this loop is byte offset from start of this 1st
                        // Unicode char.
                        let mut column = byte_offset_from_line_start;
                        line_byte_offset += byte_offset_from_line_start;
                        let remaining = &content[line_byte_offset..];
                        for (mut chunk_byte_offset, ch) in remaining.char_indices() {
                            for _ in 0..ch.len_utf8() {
                                columns.push(column);
                            }

                            match ch {
                                '\r' => {
                                    // Handle Windows-specific "\r\n" newlines
                                    chunk_byte_offset += 1;
                                    if remaining.as_bytes().get(chunk_byte_offset) == Some(&b'\n') {
                                        chunk_byte_offset += 1;
                                        columns.push(column + 1);
                                    }
                                }
                                '\n' => {
                                    chunk_byte_offset += 1;
                                }
                                LS | PS => {
                                    chunk_byte_offset += 3;
                                }
                                _ => {
                                    // Mozilla's "source-map" library counts columns using UTF-16 code units
                                    column += ch.len_utf16();
                                    continue;
                                }
                            }

                            // Line break found.
                            // `chunk_byte_offset` is now the offset of *end* of the line break.
                            line_byte_offset += chunk_byte_offset;
                            // Revert back to outer loop for next line
                            continue 'lines;
                        }

                        // EOF.
                        // One last column entry for EOF position.
                        columns.push(column);
                        break 'lines;
                    }
                };

                // Line break found.
                // `byte_offset_from_line_start` is now the length of line *including* line break.
                line_byte_offset += byte_offset_from_line_start;
                continue 'lines;
            }

            // EOF
            break;
        }

        tables
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn builder_ascii() {
        assert_mapping("", &[(0, 0, 0)]);
        assert_mapping("a", &[(0, 0, 0), (1, 0, 1)]);
        assert_mapping("\n", &[(0, 0, 0), (1, 1, 0)]);
        assert_mapping("a\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]);
        assert_mapping("\na", &[(0, 0, 0), (1, 1, 0), (2, 1, 1)]);
        assert_mapping(
            "ab\ncd\n\nef",
            &[
                (0, 0, 0),
                (1, 0, 1),
                (2, 0, 2),
                (3, 1, 0),
                (4, 1, 1),
                (5, 1, 2),
                (6, 2, 0),
                (7, 3, 0),
                (8, 3, 1),
                (9, 3, 2),
            ],
        );

        assert_mapping("\r", &[(0, 0, 0), (1, 1, 0)]);
        assert_mapping("\r\r", &[(0, 0, 0), (1, 1, 0), (2, 2, 0)]);
        assert_mapping("a\ra", &[(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1)]);

        assert_mapping("\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]);
        assert_mapping("\r\n\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1), (4, 2, 0)]);
        assert_mapping("a\r\na", &[(0, 0, 0), (1, 0, 1), (2, 0, 2), (3, 1, 0), (4, 1, 1)]);
    }

    #[test]
    fn builder_unicode() {
        assert_mapping("Ö", &[(0, 0, 0), (2, 0, 1)]);
        assert_mapping("ÖÖ", &[(0, 0, 0), (2, 0, 1), (4, 0, 2)]);
        assert_mapping("Ö\n", &[(0, 0, 0), (2, 0, 1), (3, 1, 0)]);
        assert_mapping("ÖÖ\n", &[(0, 0, 0), (2, 0, 1), (4, 0, 2), (5, 1, 0)]);
        assert_mapping("\nÖ", &[(0, 0, 0), (1, 1, 0), (3, 1, 1)]);
        assert_mapping("\nÖÖ", &[(0, 0, 0), (1, 1, 0), (3, 1, 1), (5, 1, 2)]);
        assert_mapping("Ö\nÖ", &[(0, 0, 0), (2, 0, 1), (3, 1, 0), (5, 1, 1)]);
        assert_mapping("\nÖÖ\n", &[(0, 0, 0), (1, 1, 0), (3, 1, 1), (5, 1, 2), (6, 2, 0)]);
        assert_mapping("Ö\ra", &[(0, 0, 0), (2, 0, 1), (3, 1, 0), (4, 1, 1)]);
        assert_mapping("Ö\r\na", &[(0, 0, 0), (2, 0, 1), (3, 0, 2), (4, 1, 0), (5, 1, 1)]);
    }

    fn assert_mapping(source: &str, mappings: &[(u32, u32, u32)]) {
        let mut builder = SourcemapBuilder::default();
        builder.with_name_and_source("x.js", source);
        for (position, expected_line, expected_col) in mappings.iter().copied() {
            let (line, col) = builder.search_original_line_and_column(position);
            assert_eq!(
                builder.search_original_line_and_column(position),
                (expected_line, expected_col),
                "Incorrect mapping for '{source}' - position {position} = line {line}, column {col}"
            );
        }
    }

    #[test]
    fn add_source_mapping() {
        fn create_mappings(source: &str, line: u32, column: u32) {
            let mut builder = SourcemapBuilder::default();
            builder.with_name_and_source("x.js", source);
            let output: Vec<u8> = source.as_bytes().into();
            for (i, _ch) in source.char_indices() {
                #[allow(clippy::cast_possible_truncation)]
                builder.add_source_mapping(&output, i as u32, None);
                assert!(
                    builder.generated_line == line && builder.generated_column == column,
                    "Incorrect generated mapping for '{source}' ({:?}) starting at {i} - line {}, column {}",
                    source.as_bytes(),
                    builder.generated_line,
                    builder.generated_column
                );
                assert_eq!(builder.last_generated_update, source.len());
            }
        }

        create_mappings("", 0, 0);
        create_mappings("abc", 0, 3);
        create_mappings("\n", 1, 0);
        create_mappings("\n\n\n", 3, 0);
        create_mappings("\r", 1, 0);
        create_mappings("\r\r\r", 3, 0);
        create_mappings("\r\n", 1, 0);
        create_mappings("\r\n\r\n\r\n", 3, 0);
        create_mappings("\nabc", 1, 3);
        create_mappings("abc\n", 1, 0);
        create_mappings("\rabc", 1, 3);
        create_mappings("abc\r", 1, 0);
        create_mappings("\r\nabc", 1, 3);
        create_mappings("abc\r\n", 1, 0);
        create_mappings("ÖÖ\nÖ\nÖÖÖ", 2, 3);
    }

    #[test]
    fn add_source_mapping_for_name() {
        let output = "ac".as_bytes();
        let mut builder = SourcemapBuilder::default();
        builder.with_name_and_source("x.js", "ab");
        builder.add_source_mapping_for_name(output, Span::new(0, 1), "a");
        builder.add_source_mapping_for_name(output, Span::new(1, 2), "c");
        let sm = builder.into_sourcemap();
        // The name `a` not change.
        assert_eq!(
            sm.get_source_view_token(0_u32).as_ref().and_then(|token| token.get_name()),
            None
        );
        // The name `b` -> `c`, save `b` to token.
        assert_eq!(
            sm.get_source_view_token(1_u32).as_ref().and_then(|token| token.get_name()),
            Some("b")
        );
    }
}
