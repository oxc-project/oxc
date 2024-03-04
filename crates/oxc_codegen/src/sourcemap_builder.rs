use oxc_syntax::identifier::{LS, PS};

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
    enable_sourcemap: bool,
    source_id: u32,
    last_generated_update: usize,
    last_position: Option<u32>,
    line_offset_tables: Vec<LineOffsetTable>,
    sourcemap_builder: sourcemap::SourceMapBuilder,
    generated_line: u32,
    generated_column: u32,
}

impl Default for SourcemapBuilder {
    fn default() -> Self {
        Self {
            enable_sourcemap: false,
            source_id: 0,
            last_generated_update: 0,
            last_position: None,
            line_offset_tables: vec![],
            sourcemap_builder: sourcemap::SourceMapBuilder::new(None),
            generated_line: 0,
            generated_column: 0,
        }
    }
}

impl SourcemapBuilder {
    pub fn with_enable_sourcemap(&mut self, enable_sourcemap: bool) -> &mut Self {
        self.enable_sourcemap = enable_sourcemap;
        self
    }

    pub fn with_source_and_name(&mut self, source: &str, name: &str) -> &mut Self {
        self.line_offset_tables = Self::generate_line_offset_tables(source);
        self.source_id = self.sourcemap_builder.add_source(name);
        self.sourcemap_builder.set_source_contents(self.source_id, Some(source));
        self
    }

    pub fn into_sourcemap(self) -> sourcemap::SourceMap {
        self.sourcemap_builder.into_sourcemap()
    }

    pub fn add_source_mapping(&mut self, output: &Vec<u8>, position: u32, name: Option<&str>) {
        if self.enable_sourcemap {
            if matches!(self.last_position, Some(last_position) if last_position >= position) {
                return;
            }
            let (original_line, original_column) = self.search_original_line_and_column(position);
            self.update_generated_line_and_column(output);
            let name_id = name.map(|s| self.sourcemap_builder.add_name(s));
            self.sourcemap_builder.add_raw(
                self.generated_line,
                self.generated_column,
                original_line,
                original_column,
                Some(self.source_id),
                name_id,
            );
            self.last_position = Some(position);
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn search_original_line_and_column(&self, position: u32) -> (u32, u32) {
        let result = self
            .line_offset_tables
            .partition_point(|table| table.byte_offset_to_start_of_line <= position as usize);
        let original_line = if result > 0 { result - 1 } else { 0 };
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
    fn update_generated_line_and_column(&mut self, output: &Vec<u8>) {
        // SAFETY: criteria of `from_utf8_unchecked` are met
        let s = unsafe { std::str::from_utf8_unchecked(&output[self.last_generated_update..]) };
        for (i, ch) in s.char_indices() {
            match ch {
                '\r' | '\n' | LS | PS => {
                    // Handle Windows-specific "\r\n" newlines
                    if ch == '\r' && output.get(self.last_generated_update + i + 1) == Some(&b'\n')
                    {
                        continue;
                    }
                    self.generated_line += 1;
                    self.generated_column = 0;
                }
                _ => {
                    // Mozilla's "source-map" library counts columns using UTF-16 code units
                    self.generated_column += ch.len_utf16() as u32;
                }
            }
        }
        self.last_generated_update = output.len();
    }

    fn generate_line_offset_tables(content: &str) -> Vec<LineOffsetTable> {
        let mut tables = vec![];
        let mut columns = None;
        let mut column = 0;
        let mut line_byte_offset = 0;
        let mut byte_offset_to_first = 0;
        for (i, ch) in content.char_indices() {
            // Mark the start of the next line
            if column == 0 {
                line_byte_offset = i;
            }

            // Start the mapping if this character is non-ASCII
            if !ch.is_ascii() && columns.is_none() {
                byte_offset_to_first = i - line_byte_offset;
                columns = Some(vec![]);
            }

            // Update the per-byte column offsets
            if let Some(columns) = &mut columns {
                for _ in 0..ch.len_utf8() {
                    columns.push(column);
                }
            }

            match ch {
                '\r' | '\n' | LS | PS => {
                    // Handle Windows-specific "\r\n" newlines
                    if ch == '\r' && content.as_bytes().get(i + 1) == Some(&b'\n') {
                        column += 1;
                        continue;
                    }

                    tables.push(LineOffsetTable {
                        columns,
                        byte_offset_to_first,
                        byte_offset_to_start_of_line: line_byte_offset,
                    });
                    column = 0;
                    columns = None;
                    byte_offset_to_first = 0;
                }
                _ => {
                    // Mozilla's "source-map" library counts columns using UTF-16 code units
                    column += ch.len_utf16();
                }
            }
        }
        // Mark the start of the next line
        if column == 0 {
            line_byte_offset = content.len();
        }

        // Do one last update for the column at the end of the file
        if let Some(columns) = &mut columns {
            columns.push(column);
        }

        tables.push(LineOffsetTable {
            columns,
            byte_offset_to_first,
            byte_offset_to_start_of_line: line_byte_offset,
        });

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
        builder.with_source_and_name(source, "x.js");
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
        fn create_mappings(source: &str) {
            let mut builder = SourcemapBuilder::default();
            builder.with_enable_sourcemap(true).with_source_and_name(source, "x.js");
            let output: Vec<u8> = source.as_bytes().into();
            for (i, _ch) in source.char_indices() {
                #[allow(clippy::cast_possible_truncation)]
                builder.add_source_mapping(&output, i as u32, None);
            }
        }

        create_mappings("");
        create_mappings("abc");
        create_mappings("\n");
        create_mappings("\r");
        create_mappings("\r\n");
        create_mappings("\nabc");
        create_mappings("abc\n");
        create_mappings("\rabc");
        create_mappings("abc\r");
        create_mappings("\r\nabc");
        create_mappings("abc\r\n");
    }
}
