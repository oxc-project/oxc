use std::{borrow::Cow, collections::hash_map::Entry, path::Path};

use oxc_index::{IndexVec, define_nonmax_u32_index_type};
use oxc_span::Span;
use oxc_syntax::line_terminator::{LS, LS_LAST_2_BYTES, LS_OR_PS_FIRST_BYTE, PS, PS_LAST_2_BYTES};
use rustc_hash::FxHashMap;

/// Number of lines to check with linear search when translating byte position to line index
const LINE_SEARCH_LINEAR_ITERATIONS: usize = 16;

define_nonmax_u32_index_type! {
    /// Index into vec of `ColumnOffsets`
    struct ColumnOffsetsId;
}

/// Line offset tables.
///
/// Used for tracking lines and columns from byte offsets via binary search.
///
/// Code is adapted from [esbuild](https://github.com/evanw/esbuild/blob/cc74e6042a9f573bf58e1e3f165ebda70af4ad3b/internal/js_printer/js_printer.go#L4806-L4808)
///
/// Most lines of source code will not contain Unicode chars, so optimize storage for this common case.
///
/// Each line is represented by a `Line`.
/// Where a line is entirely ASCII, translating byte offset to UTF-16 column is simple,
/// given the byte offset of start of line. A column lookup table isn't needed for that line.
/// In this case, `Line::column_offsets_id` is `None`.
/// For rare lines which do contain Unicode chars, we store column offsets in a `ColumnOffsets` which
/// is stored in a separate `IndexVec`. `Line::column_offsets_id` contains index for that line's `ColumnOffsets`.
/// Storing column offset info which is rarely used in a separate structure keeps `Line` as small as possible.
#[derive(Debug, Default)]
pub struct LineOffsetTables {
    lines: Vec<Line>,
    column_offsets: IndexVec<ColumnOffsetsId, ColumnOffsets>,
}

#[derive(Debug)]
pub struct Line {
    byte_offset_to_start_of_line: u32,
    column_offsets_id: Option<ColumnOffsetsId>,
}

#[derive(Debug)]
pub struct ColumnOffsets {
    byte_offset_to_first: u32,
    columns: Box<[u32]>,
}

pub struct SourcemapBuilder<'a> {
    source_name: String,
    original_source: &'a str,
    names: Vec<&'a str>,
    names_map: FxHashMap<&'a str, u32>,
    tokens: Vec<oxc_sourcemap::Token>,
    last_generated_update: usize,
    last_position: Option<u32>,
    last_token_has_name: bool,
    line_offset_tables: LineOffsetTables,
    generated_line: u32,
    generated_column: u32,
    /// Tracks the last accessed line index to optimize sequential lookups in `search_original_line_and_column`.
    /// Most calls to this method access positions in increasing order (e.g., when mapping source tokens linearly),
    /// so we can avoid unnecessary binary searches by advancing linearly from this cached index.
    last_line_lookup: u32,
}

impl<'a> SourcemapBuilder<'a> {
    pub fn new(path: &Path, source_text: &'a str) -> Self {
        let line_offset_tables = Self::generate_line_offset_tables(source_text);
        // Estimate mapping and name counts from source length to reduce reallocations.
        let source_len = source_text.len();
        Self {
            source_name: path.to_string_lossy().into_owned(),
            original_source: source_text,
            names: Vec::with_capacity(source_len / 64),
            names_map: FxHashMap::default(),
            tokens: Vec::with_capacity(source_len / 4),
            last_generated_update: 0,
            last_position: None,
            last_token_has_name: false,
            line_offset_tables,
            generated_line: 0,
            generated_column: 0,
            last_line_lookup: 0,
        }
    }

    pub fn into_sourcemap(mut self) -> oxc_sourcemap::SourceMap<'a> {
        self.names.shrink_to_fit();
        self.tokens.shrink_to_fit();

        oxc_sourcemap::SourceMap::from_parts(oxc_sourcemap::SourceMapParts {
            file: None,
            names: self.names.into_iter().map(Cow::Borrowed).collect(),
            source_root: None,
            sources: vec![Cow::Owned(self.source_name)],
            source_contents: vec![Some(Cow::Borrowed(self.original_source))],
            tokens: self.tokens.into_boxed_slice(),
            token_chunks: None,
            ignore_list: None,
            debug_id: None,
        })
    }

    pub fn add_source_mapping_for_name(&mut self, output: &[u8], span: Span, name: &str) {
        let original_name = self.original_name(span, name);
        // The token name should be original name.
        // If it hasn't changed, name should be `None` to reduce `SourceMap` size.
        let token_name = if original_name == Some(name) { None } else { original_name };
        self.add_source_mapping(output, span.start, token_name);
    }

    /// Add a mapping for a private identifier, which prints as `#` followed by `name`.
    ///
    /// The token is `#name`, so that is what the mapping is compared against and what it records.
    pub fn add_source_mapping_for_private_name(&mut self, output: &[u8], span: Span, name: &str) {
        let original_name = self.original_name(span, name);
        let unchanged = original_name.is_some_and(|original| {
            original
                .as_bytes()
                .split_first()
                .is_some_and(|(&hash, rest)| hash == b'#' && rest == name.as_bytes())
        });
        let token_name = if unchanged { None } else { original_name };
        self.add_source_mapping(output, span.start, token_name);
    }

    /// Get the source text `span` covers, which is the name to record unless nothing was renamed.
    fn original_name(&self, span: Span, name: &str) -> Option<&'a str> {
        debug_assert!(
            (span.end as usize) <= self.original_source.len(),
            "violated {}:{} <= {} for {name}",
            span.start,
            span.end,
            self.original_source.len()
        );
        self.original_source.get(span.start as usize..span.end as usize)
    }

    pub fn add_source_mapping(&mut self, output: &[u8], position: u32, name: Option<&'a str>) {
        if self.last_position == Some(position) {
            if let Some(name) = name
                && !self.last_token_has_name
            {
                let name_id = self.add_name(name);
                let token = self.tokens.last_mut().unwrap();
                *token = oxc_sourcemap::Token::new(
                    token.get_dst_line(),
                    token.get_dst_col(),
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_source_id(),
                    Some(name_id),
                );
                self.last_token_has_name = true;
            }
            return;
        }
        let (original_line, original_column) = self.search_original_line_and_column(position);
        self.update_generated_line_and_column(output);
        let name_id = name.map(|s| self.add_name(s));
        let token = oxc_sourcemap::Token::new(
            self.generated_line,
            self.generated_column,
            original_line,
            original_column,
            Some(0),
            name_id,
        );
        // Nothing was printed since the last mapping, so the last mapping is for an outer node
        // (for example a class member that starts at a removed `public` modifier, or a removed `(`).
        // Source map consumers use the first mapping at a generated position,
        // so replace it with this mapping, which is for the token that is printed here.
        match self.tokens.last_mut() {
            Some(last)
                if last.get_dst_line() == self.generated_line
                    && last.get_dst_col() == self.generated_column =>
            {
                *last = token;
            }
            _ => self.tokens.push(token),
        }
        self.last_position = Some(position);
        self.last_token_has_name = name_id.is_some();
    }

    fn add_name(&mut self, name: &'a str) -> u32 {
        match self.names_map.entry(name) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = u32::try_from(self.names.len())
                    .expect("sourcemap names length should fit in u32");
                entry.insert(id);
                self.names.push(name);
                id
            }
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    fn search_original_line_and_column(&mut self, position: u32) -> (u32, u32) {
        let original_line = self.search_original_line(position);

        // Store line index as starting point for next search
        self.last_line_lookup = original_line as u32;

        let line = &self.line_offset_tables.lines[original_line];
        let mut original_column = position - line.byte_offset_to_start_of_line;
        if let Some(column_offsets_id) = line.column_offsets_id {
            let column_offsets = &self.line_offset_tables.column_offsets[column_offsets_id];
            if original_column >= column_offsets.byte_offset_to_first {
                original_column = column_offsets.columns
                    [(original_column - column_offsets.byte_offset_to_first) as usize];
            }
        }
        (original_line as u32, original_column)
    }

    /// Find line index for byte index `position`, using line offset table.
    ///
    /// Usually output code is roughly in same order as it was in original source,
    /// so line will be close to the line found in last call to this function.
    ///
    /// So do fast linear search first over a few lines, and fallback to slower binary search
    /// if this doesn't find the line.
    fn search_original_line(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let idx = self.last_line_lookup as usize;

        if position >= lines[idx].byte_offset_to_start_of_line {
            self.search_original_line_forwards(position)
        } else {
            self.search_original_line_backwards(position)
        }
    }

    /// Find line index for byte index `position`, starting search at `last_line_lookup`,
    /// and working forwards.
    ///
    /// Search forwards, looking for first line which starts *after* `position`.
    /// `position` then must be on the line before that one.
    fn search_original_line_forwards(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let last_idx = self.last_line_lookup as usize;

        let start_idx = last_idx + 1;
        let end_idx = start_idx + LINE_SEARCH_LINEAR_ITERATIONS;

        // We have a fast path for when there are more than `LINE_SEARCH_LINEAR_ITERATIONS` lines
        // to search (common case unless file is very short).
        // Fast path is do linear search on first `LINE_SEARCH_LINEAR_ITERATIONS` lines,
        // then fallback to binary search over remaining lines.
        // If less than `LINE_SEARCH_LINEAR_ITERATIONS` lines to search, just do linear search,
        // but that's slower as number of lines being searched is not constant, so loop cannot be unrolled.
        if end_idx > lines.len() {
            // Less than `LINE_SEARCH_LINEAR_ITERATIONS` lines to search. Take slow path.
            // Unless file is very short, this branch is rarely taken.
            return self.search_original_line_forwards_when_few_lines(position);
        }

        // Linear search for `LINE_SEARCH_LINEAR_ITERATIONS` lines.
        // Compiler should unroll this loop as it has constant number of iterations.
        // https://godbolt.org/z/heh1cnYa4
        for (line_idx, line) in lines[start_idx..end_idx].iter().enumerate() {
            if line.byte_offset_to_start_of_line > position {
                // This line starts after `position`. `position` must be on previous line.
                return start_idx + line_idx - 1;
            }
        }

        // Line not found yet. Binary search over remaining lines.
        lines[end_idx..].partition_point(|line| line.byte_offset_to_start_of_line <= position)
            + end_idx
            - 1
    }

    #[cold]
    fn search_original_line_forwards_when_few_lines(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let last_idx = self.last_line_lookup as usize;
        let start_idx = last_idx + 1;

        for (line_idx, line) in lines[start_idx..].iter().enumerate() {
            if line.byte_offset_to_start_of_line > position {
                // This line starts after `position`. `position` must be on previous line.
                return start_idx + line_idx - 1;
            }
        }

        // No line starts after `position`. `position` must be on last line.
        lines.len() - 1
    }

    fn search_original_line_backwards(&self, position: u32) -> usize {
        let lines = &self.line_offset_tables.lines;
        let mut idx = self.last_line_lookup as usize;

        while lines[idx].byte_offset_to_start_of_line > position {
            idx -= 1;
        }

        if lines[idx].byte_offset_to_start_of_line < position {
            idx = lines
                .partition_point(|table| table.byte_offset_to_start_of_line <= position)
                .saturating_sub(1);
        }

        idx
    }

    #[expect(clippy::cast_possible_truncation)]
    fn update_generated_line_and_column(&mut self, output: &[u8]) {
        // Bytes processed per iteration when fast-forwarding over "boring" bytes
        // (ASCII that is neither a line break nor part of a Unicode char).
        const STRIDE: usize = 8;
        const LO: u64 = 0x0101_0101_0101_0101;
        const HI: u64 = 0x8080_8080_8080_8080;

        let len = output.len();
        let start_index = self.last_generated_update;

        // Find last line break
        let mut line_start_index = start_index;
        let mut idx = start_index;
        let mut last_line_is_ascii = true;

        while idx < len {
            // Fast-forward over runs of "boring" bytes `STRIDE` at a time. The vast
            // majority of output is ASCII that is neither a line break nor part of a
            // Unicode char; such bytes don't change the line count or the last-line
            // ASCII flag, so a run of them can be skipped in bulk instead of matched
            // byte-by-byte. SWAR (SIMD-within-a-register) detects, branchlessly, whether
            // any byte of an 8-byte word is `\n` (0x0A), `\r` (0x0D), or non-ASCII
            // (>= 0x80) using the classic "zero byte in a word" bit trick; if none are,
            // the whole word is skipped.
            while idx + STRIDE <= len {
                let word = u64::from_ne_bytes(output[idx..idx + STRIDE].try_into().unwrap());
                let lf = word ^ (LO * u64::from(b'\n'));
                let cr = word ^ (LO * u64::from(b'\r'));
                let has_lf = lf.wrapping_sub(LO) & !lf & HI;
                let has_cr = cr.wrapping_sub(LO) & !cr & HI;
                let non_ascii = word & HI;
                if (has_lf | has_cr | non_ascii) != 0 {
                    break;
                }
                idx += STRIDE;
            }

            // The word scan stops at a special byte within the next word, or at
            // a tail shorter than STRIDE. Skip common ASCII bytes (space and above)
            // without repeating the word scan for every byte.
            while idx < len {
                if !(b' '..=0x7f).contains(&output[idx]) {
                    break;
                }
                idx += 1;
            }
            if idx >= len {
                break;
            }

            let byte = output[idx];
            match byte {
                b'\n' => {}
                b'\r' => {
                    // Handle Windows-specific "\r\n" newlines
                    if output.get(idx + 1) == Some(&b'\n') {
                        idx += 1;
                    }
                }
                _ if byte.is_ascii() => {
                    idx += 1;
                    continue;
                }
                LS_OR_PS_FIRST_BYTE => {
                    let next_two_bytes = output.get(idx + 1..idx + 3);
                    if next_two_bytes != Some(&LS_LAST_2_BYTES[..])
                        && next_two_bytes != Some(&PS_LAST_2_BYTES[..])
                    {
                        last_line_is_ascii = false;
                        // 3-byte Unicode char that isn't a line break.
                        // We know it's 3 bytes, so we could do `idx += 3`, but that's more instruction bytes
                        // than `idx += 1` (`inc` instruction). This branch is extremely rarely taken, and this
                        // is in a hot loop, so it's better to optimize for the common case.
                        // The remaining 2 bytes will hit the "Unicode char" branch below on next turns of the loop,
                        // and they'll be skipped too.
                        idx += 1;
                        continue;
                    }
                    // 3-byte line break. Add 2 to `idx` here, as 1 more is added below.
                    idx += 2;
                }
                _ => {
                    // Unicode char
                    last_line_is_ascii = false;
                    // Note: Only increment `idx` by 1. We don't know how many bytes the char is, and non-ASCII
                    // chars are rare, so it's not worthwhile adding more instructions to this hot loop to find out.
                    // The remaining bytes of the char will hit this branch again on next turns of the loop,
                    // and they'll be skipped too.
                    idx += 1;
                    continue;
                }
            }

            // Line break found.
            line_start_index = idx + 1;
            self.generated_line += 1;
            self.generated_column = 0;
            last_line_is_ascii = true;
            idx += 1;
        }

        // Calculate column
        self.generated_column += if last_line_is_ascii {
            (len - line_start_index) as u32
        } else {
            // TODO: It'd be better if could use `from_utf8_unchecked` here, but we'd need to make this
            // function unsafe and caller guarantees `output` contains a valid UTF-8 string
            let last_line = std::str::from_utf8(&output[line_start_index..]).unwrap();
            // Mozilla's "source-map" library counts columns using UTF-16 code units
            last_line.encode_utf16().count() as u32
        };
        self.last_generated_update = len;
    }

    /// Find the first CR, LF, or non-ASCII byte, or return the input length.
    fn find_ascii_line_end(bytes: &[u8]) -> usize {
        let mut offset = 0;
        // Bound each search so LS/PS cannot cause repeated scans of the rest of the file
        // when the Unicode fallback consumes only one logical line.
        for chunk in bytes.chunks(256) {
            let line_end = memchr::memchr2(b'\n', b'\r', chunk).unwrap_or(chunk.len());
            let prefix = &chunk[..line_end];
            if !prefix.is_ascii() {
                return offset + prefix.iter().position(|b| !b.is_ascii()).unwrap();
            }
            offset += line_end;
            if line_end < chunk.len() {
                break;
            }
        }
        offset
    }

    fn generate_line_offset_tables(content: &str) -> LineOffsetTables {
        // Select once per file so ASCII input keeps the direct memchr2 fast path
        // without per-line Unicode checks or chunk-loop overhead.
        if content.is_ascii() {
            Self::generate_line_offset_tables_impl::<true>(content)
        } else {
            Self::generate_line_offset_tables_impl::<false>(content)
        }
    }

    fn generate_line_offset_tables_impl<const ASCII: bool>(content: &str) -> LineOffsetTables {
        let mut lines = Vec::with_capacity(content.len() / 24 + 1);
        let mut column_offsets = IndexVec::new();

        // Used as a buffer to reduce memory reallocations.
        // Pre-allocate with reasonable capacity to avoid frequent reallocations.
        // 16 is a guess, probably adequate for most files which don't heavily use unicode chars.
        // 16 entries is 64 bytes = 1 CPU cache line on most CPUs.
        // If file does heavily use unicode chars, `columns` will grow adaptively as needed.
        let mut columns = Vec::with_capacity(16);

        // Process content line-by-line.
        // For each line, start by assuming line will be entirely ASCII, and scan in chunks.
        // If line is all ASCII, UTF-8 columns and UTF-16 columns are the same,
        // so no need to create a `columns` Vec. This is the fast path for common case.
        // If a Unicode character found, read rest of line char-by-char, populating `columns` Vec.
        // At end of line, go back to top of outer loop, and again assume ASCII for next line.
        let mut line_byte_offset = 0;
        'lines: loop {
            lines.push(Line {
                byte_offset_to_start_of_line: line_byte_offset,
                column_offsets_id: None,
            });

            let remaining = &content.as_bytes()[line_byte_offset as usize..];

            let ascii_end = if ASCII {
                memchr::memchr2(b'\n', b'\r', remaining).unwrap_or(remaining.len())
            } else {
                Self::find_ascii_line_end(remaining)
            };
            let Some(&byte) = remaining.get(ascii_end) else {
                // EOF reached, and remainder of file is plain ASCII.
                break;
            };

            // ASCII line: UTF-8 and UTF-16 columns are identical, so no column table is needed.
            if ASCII || byte.is_ascii() {
                #[expect(clippy::cast_possible_truncation)]
                let mut byte_offset_from_line_start = ascii_end as u32 + 1;
                // Handle Windows-specific "\r\n" newlines, including across chunk boundaries.
                if byte == b'\r'
                    && remaining.get(byte_offset_from_line_start as usize) == Some(&b'\n')
                {
                    byte_offset_from_line_start += 1;
                }

                // Line break found.
                // `byte_offset_from_line_start` is now the length of line *including* line break.
                line_byte_offset += byte_offset_from_line_start;
                continue 'lines;
            }

            // Slow path: process the rest of the line char-by-char, starting at the first
            // non-ASCII byte (which may itself start a Unicode line break such as LS/PS).
            #[expect(clippy::cast_possible_truncation)]
            let byte_offset_from_line_start = ascii_end as u32;

            // Set `column_offsets_id` for line and create `columns` Vec.
            let line = lines.iter_mut().last().unwrap();
            line.column_offsets_id = Some(ColumnOffsetsId::from_usize(column_offsets.len()));

            // Loop through rest of line char-by-char.
            // `chunk_byte_offset` in this loop is byte offset from start of this 1st
            // Unicode char.
            let mut column = byte_offset_from_line_start;
            line_byte_offset += byte_offset_from_line_start;
            let remaining = &content[line_byte_offset as usize..];
            for (chunk_byte_offset, ch) in remaining.char_indices() {
                #[expect(clippy::cast_possible_truncation)]
                let mut chunk_byte_offset = chunk_byte_offset as u32;
                columns.extend(std::iter::repeat_n(column, ch.len_utf8()));

                match ch {
                    '\r' => {
                        // Handle Windows-specific "\r\n" newlines
                        chunk_byte_offset += 1;
                        if remaining.as_bytes().get(chunk_byte_offset as usize) == Some(&b'\n') {
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
                    #[expect(clippy::cast_possible_truncation)]
                    _ => {
                        // Mozilla's "source-map" library counts columns using UTF-16 code units
                        column += ch.len_utf16() as u32;
                        continue;
                    }
                }

                // Line break found.
                // `chunk_byte_offset` is now the offset of *end* of the line break.
                line_byte_offset += chunk_byte_offset;

                // Record column offsets.
                // `columns.clone().into_boxed_slice()` does perform an allocation,
                // but only one - `Vec::clone` produces a `Vec` with `capacity == len`,
                // so `Vec::into_boxed_slice` just drops the `capacity` field,
                // and does not need to reallocate again.
                // `columns` is reused for next line, and will grow adaptively depending on
                // how heavily the file uses unicode chars.
                column_offsets.push(ColumnOffsets {
                    byte_offset_to_first: byte_offset_from_line_start,
                    columns: columns.clone().into_boxed_slice(),
                });
                columns.clear();

                // Revert back to outer loop for next line
                continue 'lines;
            }

            // EOF.
            // One last column entry for EOF position.
            columns.push(column);

            // Record column offsets
            column_offsets.push(ColumnOffsets {
                byte_offset_to_first: byte_offset_from_line_start,
                columns: columns.into_boxed_slice(),
            });

            break 'lines;
        }

        LineOffsetTables { lines, column_offsets }
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
    fn builder_ascii_line_break_boundaries() {
        for prefix_len in [0_u32, 7, 8, 15, 16, 255, 256, 257, 1023] {
            for terminator in ["\n", "\r", "\r\n"] {
                let source = format!("{}{terminator}x", "a".repeat(prefix_len as usize));
                let next_line = prefix_len + u32::try_from(terminator.len()).unwrap();
                assert_mapping(
                    &source,
                    &[(prefix_len, 0, prefix_len), (next_line, 1, 0), (next_line + 1, 1, 1)],
                );
                assert!(
                    SourcemapBuilder::generate_line_offset_tables(&source)
                        .column_offsets
                        .is_empty()
                );
            }
        }
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

    #[test]
    fn builder_mixed_unicode_line_terminators() {
        assert_mapping(
            "a\u{2028}😀\r\né\u{2029}中\rb\n",
            &[
                (0, 0, 0),
                (1, 0, 1),
                (4, 1, 0),
                (8, 1, 2),
                (9, 1, 3),
                (10, 2, 0),
                (12, 2, 1),
                (15, 3, 0),
                (18, 3, 1),
                (19, 4, 0),
                (20, 4, 1),
                (21, 5, 0),
            ],
        );
    }

    #[test]
    fn ascii_scan_stops_at_unicode() {
        for prefix_len in [0, 7, 8, 15, 16, 255, 256, 257, 1023] {
            let prefix = "x".repeat(prefix_len);
            assert_eq!(SourcemapBuilder::find_ascii_line_end(prefix.as_bytes()), prefix_len);
            for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}", "Ö", "𝄞"] {
                // A distant CR/LF must not make the scan skip over a Unicode separator.
                let source = format!("{prefix}{terminator}{}\n", "x".repeat(4096));
                assert_eq!(SourcemapBuilder::find_ascii_line_end(source.as_bytes()), prefix_len);
            }
        }
    }

    #[test]
    fn builder_many_unicode_line_separators() {
        // Without bounding the ASCII scan, each LS/PS rescans the remaining suffix: O(n²).
        const LINE_COUNT: usize = 256_000;
        for separator in [LS, PS] {
            let source = format!("x;{separator}").repeat(LINE_COUNT);
            let tables = SourcemapBuilder::generate_line_offset_tables(&source);
            assert_eq!(tables.lines.len(), LINE_COUNT + 1);
            assert_eq!(tables.column_offsets.len(), LINE_COUNT);
            for (index, line) in tables.lines.iter().enumerate() {
                assert_eq!(line.byte_offset_to_start_of_line as usize, index * 5);
                if index == LINE_COUNT {
                    assert!(line.column_offsets_id.is_none());
                } else {
                    let columns = &tables.column_offsets[line.column_offsets_id.unwrap()];
                    assert_eq!(columns.byte_offset_to_first, 2);
                    assert_eq!(&*columns.columns, &[2, 2, 2]);
                }
            }
        }
    }

    #[test]
    fn builder_with_unordered_position() {
        assert_mapping("\na\nb", &[(4, 2, 1), (0, 0, 0), (1, 1, 0), (2, 1, 1), (3, 2, 0)]);
    }

    fn assert_mapping(source: &str, mappings: &[(u32, u32, u32)]) {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), source);
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
            let mut builder = SourcemapBuilder::new(Path::new("x.js"), source);
            let output: Vec<u8> = source.as_bytes().into();
            for (i, _ch) in source.char_indices() {
                #[expect(clippy::cast_possible_truncation)]
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
        create_mappings("\u{2028}", 1, 0);
        create_mappings("\u{2029}", 1, 0);
        create_mappings("a\u{2028}b", 1, 1);
        create_mappings("a\u{2029}b", 1, 1);
        create_mappings("`\u{2028}`", 1, 1);
        create_mappings("`\u{2029}`", 1, 1);

        // Long lines (>= 8 bytes) to exercise the SWAR boring-byte fast-forward in
        // `update_generated_line_and_column`, with line breaks / unicode at various offsets.
        create_mappings("abcdefghijklmnop", 0, 16);
        create_mappings("abcdefgh\nijklmnop", 1, 8);
        create_mappings("abcdefghijklmnop\n", 1, 0);
        create_mappings("\nabcdefghijklmnop", 1, 16);
        create_mappings("aaaaaaaaaa\r\nbbbbbbbbbb", 1, 10);
        create_mappings("aaaaaaaaaa\rbbbbbbbbbb", 1, 10);
        create_mappings("aaaaaaaaÖbbbbbbbb", 0, 17);
        create_mappings("aaaaaaaaaaaaaaaa\u{2028}bbbbbbbbbbbbbbbb", 1, 16);
        create_mappings("abcdefghijklmnopqrstuvwxyz0123456789", 0, 36);
        // Line breaks straddling the 8-byte SWAR word boundary (`\r` at index 7).
        create_mappings("aaaaaaa\r\nbbbbbbbb", 1, 8);
        create_mappings("aaaaaaa\rbbbbbbbb", 1, 8);
    }

    #[test]
    fn generated_line_and_column_ascii_tails() {
        for prefix_len in 0..=24 {
            for (suffix, expected_line, expected_column) in [
                ("", 0, prefix_len),
                ("\txyz", 0, prefix_len + 4),
                ("é😀", 0, prefix_len + 3),
                ("\nxyz", 1, 3),
                ("\rxyz", 1, 3),
                ("\r\nxyz", 1, 3),
                ("\u{2028}xyz", 1, 3),
                ("\u{2029}xyz", 1, 3),
            ] {
                let output = format!("{}{suffix}", "a".repeat(prefix_len as usize));
                let mut builder = SourcemapBuilder::new(Path::new("x.js"), "");
                builder.update_generated_line_and_column(output.as_bytes());
                assert_eq!(
                    (builder.generated_line, builder.generated_column),
                    (expected_line, expected_column),
                    "prefix length {prefix_len}, suffix {suffix:?}"
                );
                assert_eq!(builder.last_generated_update, output.len());
            }
        }
    }

    #[test]
    fn add_source_mapping_for_name() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ab");
        builder.add_source_mapping_for_name(b"", Span::new(0, 1), "a");
        builder.add_source_mapping_for_name(b"a", Span::new(1, 2), "c");
        let sm = builder.into_sourcemap();
        // The name `a` not change.
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            None
        );
        // The name `b` -> `c`, save `b` to token.
        assert_eq!(
            sm.get_source_view_token(1_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            Some("b")
        );
    }

    #[test]
    fn duplicate_source_mappings_preserve_names_and_coordinates() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "first second");
        builder.add_source_mapping(b"", 0, None);
        builder.add_source_mapping(b"(a", 0, Some("first"));
        builder.add_source_mapping(b"(a", 0, Some("ignored"));
        builder.add_source_mapping(b"(a\n", 6, None);
        builder.add_source_mapping(b"(a\nb", 6, Some("second"));
        builder.add_source_mapping(b"(a\nb", 6, None);
        builder.add_source_mapping(b"(a\nb", 6, Some("ignored"));

        let sm = builder.into_sourcemap();
        let tokens = sm
            .get_source_view_tokens()
            .map(|token| {
                (
                    token.get_dst_line(),
                    token.get_dst_col(),
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_name(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(tokens, [(0, 0, 0, 0, Some("first")), (1, 0, 0, 6, Some("second"))]);
        assert_eq!(sm.get_names().collect::<Vec<_>>(), ["first", "second"]);
    }

    #[test]
    fn add_source_mapping_for_name_from_private_identifier() {
        // A transform can print a plain identifier where the source had a private one.
        // The `#` is part of the region the mapping describes, so it is part of the name recorded.
        let output = b"foo";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "#foo");
        builder.add_source_mapping_for_name(output, Span::new(0, 4), "foo");
        let sm = builder.into_sourcemap();
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            Some("#foo")
        );
    }

    #[test]
    fn add_source_mapping_for_private_name() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "#a#b");
        builder.add_source_mapping_for_private_name(b"", Span::new(0, 2), "a");
        builder.add_source_mapping_for_private_name(b"#a", Span::new(2, 4), "c");
        let sm = builder.into_sourcemap();
        // The name `#a` not change.
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            None
        );
        // The name `#b` -> `#c`, save `#b` to token, `#` included.
        assert_eq!(
            sm.get_source_view_token(1_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            Some("#b")
        );
    }

    #[test]
    fn add_source_mapping_for_private_name_from_plain_identifier() {
        // A transform can print a private identifier where the source had a plain one.
        // The token still differs from the source text, so the original name is recorded.
        let output = b"#foo";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ffoo");
        builder.add_source_mapping_for_private_name(output, Span::new(0, 4), "foo");
        let sm = builder.into_sourcemap();
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(oxc_sourcemap::SourceViewToken::get_name),
            Some("ffoo")
        );
    }

    #[test]
    fn same_generated_position_keeps_last_mapping() {
        // Nothing is printed between the first two mappings, or between the last two,
        // so the later mapping replaces the earlier one. The name of the later mapping is kept.
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "(c).d (e).f");
        builder.add_source_mapping(b"", 0, None);
        builder.add_source_mapping(b"", 1, None);
        builder.add_source_mapping(b"c.d ", 6, None);
        builder.add_source_mapping(b"c.d ", 7, Some("e"));
        let sm = builder.into_sourcemap();
        let tokens = sm
            .get_source_view_tokens()
            .map(|token| {
                (
                    token.get_dst_line(),
                    token.get_dst_col(),
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_name(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(tokens, [(0, 0, 0, 1, None), (0, 4, 0, 7, Some("e"))]);
    }

    #[test]
    fn add_source_mapping_for_unordered_position() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ab");
        builder.add_source_mapping(b"", 1, None);
        builder.add_source_mapping(b"b", 0, None);
        let sm = builder.into_sourcemap();
        assert_eq!(sm.get_tokens().count(), 2);
    }

    static SOURCE: &str = "a\nbc\ndef";
    static MAPPINGS: &[(u32, u32)] = &[
        (0, 0), // 'a'
        (0, 1), // '\n'
        (1, 0), // 'b'
        (1, 1), // 'c'
        (1, 2), // '\n'
        (2, 0), // 'd'
        (2, 1), // 'e'
        (2, 2), // 'f'
        (2, 3), // EOF
    ];

    #[test]
    fn test_search_original_line_and_column_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        #[expect(clippy::cast_possible_truncation)]
        for (pos, (expected_line, expected_col)) in MAPPINGS.iter().copied().enumerate() {
            let (line, col) = builder.search_original_line_and_column(pos as u32);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }

    #[test]
    fn test_search_original_line_and_column_reverse_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        #[expect(clippy::cast_possible_truncation)]
        for (pos, (expected_line, expected_col)) in MAPPINGS.iter().copied().enumerate().rev() {
            let (line, col) = builder.search_original_line_and_column(pos as u32);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }

    #[test]
    fn test_search_original_line_and_column_non_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        let indexes = [8, 0, 7, 1, 6, 2, 5, 3, 4];

        for pos in indexes {
            let (expected_line, expected_col) = MAPPINGS[pos as usize];
            let (line, col) = builder.search_original_line_and_column(pos);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }
}
