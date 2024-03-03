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
                    if ch == '\r' && output[self.last_generated_update + i + 1] == b'\n' {
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
        let mut column_byte_offset = 0;
        let mut line_byte_offset = 0;
        let mut byte_offset_to_first = 0;
        for (i, ch) in content.char_indices() {
            // Mark the start of the next line
            if column == 0 {
                line_byte_offset = i;
            }

            // Start the mapping if this character is non-ASCII
            if !ch.is_ascii() && columns.is_none() {
                column_byte_offset = i - line_byte_offset;
                byte_offset_to_first = column_byte_offset;
                columns = Some(vec![]);
            }

            // Update the per-byte column offsets
            if let Some(columns) = &mut columns {
                for _ in column_byte_offset..=(i - line_byte_offset) {
                    columns.push(column);
                }
            }

            match ch {
                '\r' | '\n' | LS | PS => {
                    // Handle Windows-specific "\r\n" newlines
                    if ch == '\r' && content.chars().nth(i + 1) == Some('\n') {
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
                    column_byte_offset = 0;
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
            for _ in column_byte_offset..=(content.len() - line_byte_offset) {
                columns.push(column);
            }
        }

        tables.push(LineOffsetTable {
            columns,
            byte_offset_to_first,
            byte_offset_to_start_of_line: line_byte_offset,
        });

        tables
    }
}
