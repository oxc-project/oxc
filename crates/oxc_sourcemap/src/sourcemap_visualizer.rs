use std::borrow::Cow;

use rustc_hash::FxHashMap;

use crate::SourceMap;
use cow_utils::CowUtils;

/// The `SourcemapVisualizer` is a helper for sourcemap testing.
/// It print the mapping of original content and final content tokens.
pub struct SourcemapVisualizer<'a> {
    output: &'a str,
    sourcemap: &'a SourceMap,
}

impl<'a> SourcemapVisualizer<'a> {
    pub fn new(output: &'a str, sourcemap: &'a SourceMap) -> Self {
        Self { output, sourcemap }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn into_visualizer_text(self) -> String {
        let source_contents_lines_map: FxHashMap<String, Option<Vec<Vec<u16>>>> = self
            .sourcemap
            .get_sources()
            .enumerate()
            .map(|(source_id, source)| {
                (
                    source.to_string(),
                    self.sourcemap
                        .get_source_content(source_id as u32)
                        .map(Self::generate_line_utf16_tables),
                )
            })
            .collect();
        let output_lines = Self::generate_line_utf16_tables(self.output);

        let mut s = String::new();

        let tokens = &self.sourcemap.tokens;

        let mut last_source: Option<&str> = None;
        for i in 0..tokens.len() {
            let t = &tokens[i];
            let Some(source_id) = t.source_id else { continue };
            let Some(source) = self.sourcemap.get_source(source_id) else { continue };
            let Some(source_contents_lines) = source_contents_lines_map[source].as_ref() else {
                continue;
            };

            // find next dst column or EOL
            let dst_end_col = {
                match tokens.get(i + 1) {
                    Some(t2) if t2.dst_line == t.dst_line => t2.dst_col,
                    _ => output_lines[t.dst_line as usize].len() as u32,
                }
            };

            // find next src column or EOL
            let src_end_col = 'result: {
                for t2 in &tokens[i + 1..] {
                    if t2.source_id == t.source_id && t2.src_line == t.src_line {
                        // skip duplicate or backward
                        if t2.src_col <= t.src_col {
                            continue;
                        }
                        break 'result t2.src_col;
                    }
                    break;
                }
                source_contents_lines[t.src_line as usize].len() as u32
            };

            // Print source
            if last_source != Some(source) {
                s.push('-');
                s.push(' ');
                s.push_str(source);
                s.push('\n');
                last_source = Some(source);
            }

            s.push_str(&format!(
                "({}:{}) {:?}",
                t.src_line,
                t.src_col,
                Self::str_slice_by_token(
                    source_contents_lines,
                    (t.src_line, t.src_col),
                    (t.src_line, src_end_col)
                )
            ));

            s.push_str(" --> ");

            s.push_str(&format!(
                "({}:{}) {:?}",
                t.dst_line,
                t.dst_col,
                Self::str_slice_by_token(
                    &output_lines,
                    (t.dst_line, t.dst_col),
                    (t.dst_line, dst_end_col)
                )
            ));
            s.push('\n');
        }

        s
    }

    fn generate_line_utf16_tables(content: &str) -> Vec<Vec<u16>> {
        let mut tables = vec![];
        let mut line_byte_offset = 0;
        for (i, ch) in content.char_indices() {
            match ch {
                '\r' | '\n' | '\u{2028}' | '\u{2029}' => {
                    // Handle Windows-specific "\r\n" newlines
                    if ch == '\r' && content.chars().nth(i + 1) == Some('\n') {
                        continue;
                    }
                    tables.push(content[line_byte_offset..=i].encode_utf16().collect::<Vec<_>>());
                    line_byte_offset = i + 1;
                }
                _ => {}
            }
        }
        tables.push(content[line_byte_offset..].encode_utf16().collect::<Vec<_>>());
        tables
    }

    fn str_slice_by_token(buff: &[Vec<u16>], start: (u32, u32), end: (u32, u32)) -> Cow<'_, str> {
        if start.0 == end.0 {
            if start.1 <= end.1 {
                return Cow::Owned(
                    String::from_utf16(&buff[start.0 as usize][start.1 as usize..end.1 as usize])
                        .unwrap(),
                );
            }
            return Cow::Owned(
                String::from_utf16(&buff[start.0 as usize][end.1 as usize..start.1 as usize])
                    .unwrap(),
            );
        }

        let mut s = String::new();
        for i in start.0..=end.0 {
            let slice = &buff[i as usize];
            if i == start.0 {
                s.push_str(&String::from_utf16(&slice[start.1 as usize..]).unwrap());
            } else if i == end.0 {
                s.push_str(&String::from_utf16(&slice[..end.1 as usize]).unwrap());
            } else {
                s.push_str(&String::from_utf16(slice).unwrap());
            }
        }

        let replaced: Cow<str> = s.cow_replace("\r", "");

        // Windows: Replace "\r\n" and replace with "\n"
        Cow::Owned(replaced.into_owned())
    }
}
