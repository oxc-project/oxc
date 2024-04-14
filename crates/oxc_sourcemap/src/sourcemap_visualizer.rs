use crate::SourceMap;
use rustc_hash::FxHashMap;

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
        let mut source_log_map = FxHashMap::default();
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

        self.sourcemap.get_tokens().reduce(|pre_token, token| {
            if let Some(source) =
                pre_token.get_source_id().and_then(|id| self.sourcemap.get_source(id))
            {
                if let Some(Some(source_contents_lines)) = source_contents_lines_map.get(source) {
                    // Print source
                    source_log_map.entry(source).or_insert_with(|| {
                        s.push('-');
                        s.push(' ');
                        s.push_str(source);
                        s.push('\n');
                        true
                    });

                    // Print token
                    if pre_token.get_source_id() == token.get_source_id() {
                        s.push_str(&format!(
                            "({}:{}-{}:{}) {:?}",
                            pre_token.get_src_line(),
                            pre_token.get_src_col(),
                            token.get_src_line(),
                            token.get_src_col(),
                            Self::str_slice_by_token(
                                source_contents_lines,
                                (pre_token.get_src_line(), pre_token.get_src_col()),
                                (token.get_src_line(), token.get_src_col())
                            )
                        ));
                    } else if token.get_source_id().is_some() {
                        Self::print_source_last_mapping(
                            &mut s,
                            source_contents_lines,
                            (pre_token.get_src_line(), pre_token.get_src_col()),
                        );
                    }

                    s.push_str(" --> ");

                    s.push_str(&format!(
                        "({}:{}-{}:{}) {:?}",
                        pre_token.get_dst_line(),
                        pre_token.get_dst_col(),
                        token.get_dst_line(),
                        token.get_dst_col(),
                        Self::str_slice_by_token(
                            &output_lines,
                            (pre_token.get_dst_line(), pre_token.get_dst_col(),),
                            (token.get_dst_line(), token.get_dst_col(),)
                        )
                    ));
                    s.push('\n');
                }
            }

            token
        });

        if let Some(last_token) =
            self.sourcemap.get_token(self.sourcemap.get_tokens().count() as u32 - 1)
        {
            if let Some(Some(source_contents_lines)) = last_token
                .get_source_id()
                .and_then(|id| self.sourcemap.get_source(id))
                .and_then(|source| source_contents_lines_map.get(source))
            {
                Self::print_source_last_mapping(
                    &mut s,
                    source_contents_lines,
                    (last_token.get_src_line(), last_token.get_src_col()),
                );
            }
            s.push_str(" --> ");
            Self::print_source_last_mapping(
                &mut s,
                &output_lines,
                (last_token.get_dst_line(), last_token.get_dst_col()),
            );
            s.push('\n');
        }

        s
    }

    #[allow(clippy::cast_possible_truncation)]
    fn print_source_last_mapping(s: &mut String, buff: &[Vec<u16>], start: (u32, u32)) {
        let line = if buff.is_empty() { 0 } else { buff.len() as u32 - 1 };
        let column = buff.last().map(|v| v.len() as u32).unwrap_or_default();
        s.push_str(&format!(
            "({}:{}-{}:{}) {:?}",
            start.0,
            start.1,
            line,
            column,
            Self::str_slice_by_token(buff, start, (line, column))
        ));
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
                    tables.push(content[line_byte_offset..i].encode_utf16().collect::<Vec<_>>());
                    line_byte_offset = i;
                }
                _ => {}
            }
        }
        tables.push(content[line_byte_offset..].encode_utf16().collect::<Vec<_>>());
        tables
    }

    fn str_slice_by_token(buff: &[Vec<u16>], start: (u32, u32), end: (u32, u32)) -> String {
        if start.0 == end.0 {
            return String::from_utf16(&buff[start.0 as usize][start.1 as usize..end.1 as usize])
                .unwrap();
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

        // Windows: Replace "\r\n" and replace with "\n"
        s.replace('\r', "")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_work() {
        let sourcemap = SourceMap::from_json_string(r#"{
            "version":3,
            "sources":["shared.js","index.js"],
            "sourcesContent":["const a = 'shared.js'\n\nexport { a }","import { a as a2 } from './shared'\nconst a = 'index.js'\nconsole.log(a, a2)\n"],
            "names":["a","a$1"],
            "mappings":";;AAAA,MAAMA,IAAI;;;ACCV,MAAMC,MAAI;AACV,QAAQ,IAAIA,KAAGD,EAAG"
        }"#).unwrap();
        let output = "\n// shared.js\nconst a = 'shared.js';\n\n// index.js\nconst a$1 = 'index.js';\nconsole.log(a$1, a);\n";
        let visualizer = SourcemapVisualizer::new(output, &sourcemap);
        let visualizer_text = visualizer.into_visualizer_text();
        println!("{visualizer_text}");
        assert_eq!(
            visualizer_text,
            r#"- shared.js
(0:0-0:6) "const " --> (2:0-2:6) "\nconst"
(0:6-0:10) "a = " --> (2:6-2:10) " a ="
(0:10-2:13) "'shared.js'\n\nexport { a }" --> (2:10-5:0) " 'shared.js';\n\n// index.js"
- index.js
(1:0-1:6) "\nconst" --> (5:0-5:6) "\nconst"
(1:6-1:10) " a =" --> (5:6-5:12) " a$1 ="
(1:10-2:0) " 'index.js'" --> (5:12-6:0) " 'index.js';"
(2:0-2:8) "\nconsole" --> (6:0-6:8) "\nconsole"
(2:8-2:12) ".log" --> (6:8-6:12) ".log"
(2:12-2:15) "(a," --> (6:12-6:17) "(a$1,"
(2:15-2:18) " a2" --> (6:17-6:19) " a"
(2:18-3:1) ")\n" --> (6:19-7:1) ");\n"
"#
        );
    }
}
