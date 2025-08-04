use memchr::memmem::Finder;

use oxc_span::SourceType;

use super::{JavaScriptSource, SCRIPT_END, SCRIPT_START, find_script_closing_angle};

pub struct VuePartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> VuePartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        self.parse_scripts()
    }

    /// Each *.vue file can contain at most
    ///  * one `<script>` block (excluding `<script setup>`).
    ///  * one `<script setup>` block (excluding normal `<script>`).
    /// <https://vuejs.org/api/sfc-spec.html#script>
    fn parse_scripts(&self) -> Vec<JavaScriptSource<'a>> {
        let mut pointer = 0;
        let Some(result1) = self.parse_script(&mut pointer) else {
            return vec![];
        };
        let Some(result2) = self.parse_script(&mut pointer) else {
            return vec![result1];
        };
        vec![result1, result2]
    }

    fn parse_script(&self, pointer: &mut usize) -> Option<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);

        // find opening "<script"
        let offset = script_start_finder.find(&self.source_text.as_bytes()[*pointer..])?;
        *pointer += offset + SCRIPT_START.len();

        // skip `<script-`
        if !self.source_text[*pointer..].starts_with([' ', '>']) {
            return self.parse_script(pointer);
        }

        // find closing ">"
        let offset = find_script_closing_angle(self.source_text, *pointer)?;

        // get ts and jsx attribute
        let content = &self.source_text[*pointer..*pointer + offset];

        // parse `lang`
        let lang = Self::extract_lang_attribute(content);

        let Ok(mut source_type) = SourceType::from_extension(lang) else { return None };
        if !lang.contains('x') {
            source_type = source_type.with_standard(true);
        }

        *pointer += offset + 1;
        let js_start = *pointer;

        // find "</script>"
        let script_end_finder = Finder::new(SCRIPT_END);
        let offset = script_end_finder.find(&self.source_text.as_bytes()[*pointer..])?;
        let js_end = *pointer + offset;
        *pointer += offset + SCRIPT_END.len();

        let source_text = &self.source_text[js_start..js_end];
        // NOTE: loader checked that source_text.len() is less than u32::MAX
        #[expect(clippy::cast_possible_truncation)]
        Some(JavaScriptSource::partial(source_text, source_type, js_start as u32))
    }

    fn extract_lang_attribute(content: &str) -> &str {
        let content = content.trim();

        let Some(lang_index) = content.find("lang") else { return "mjs" };

        // Move past "lang"
        let mut rest = content[lang_index + 4..].trim_start();

        if !rest.starts_with('=') {
            return "mjs";
        }

        // Move past "="
        rest = rest[1..].trim_start();

        let first_char = rest.chars().next();

        match first_char {
            Some('"' | '\'') => {
                let quote = first_char.unwrap();
                rest = &rest[1..];
                match rest.find(quote) {
                    Some(end) => &rest[..end],
                    None => "mjs", // Unterminated quote
                }
            }
            Some(_) => {
                // Unquoted value: take until first whitespace or attribute separator
                match rest.find(|c: char| c.is_whitespace() || c == '>') {
                    Some(end) => &rest[..end],
                    None => rest, // whole rest is the lang value
                }
            }
            None => "mjs", // nothing after =
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_span::SourceType;

    use super::{JavaScriptSource, VuePartialLoader};

    fn parse_vue(source_text: &str) -> JavaScriptSource<'_> {
        let sources = VuePartialLoader::new(source_text).parse();
        *sources.first().unwrap()
    }

    #[test]
    fn test_parse_vue_one_line() {
        let source_text = r#"
        <template>
          <h1>hello world</h1>
        </template>
        <script> console.log("hi") </script>
        "#;

        let result = parse_vue(source_text);
        assert_eq!(result.source_text, r#" console.log("hi") "#);
    }

    #[test]
    fn test_build_vue_with_ts_flag_1() {
        let source_text = r#"
        <script lang="ts" setup generic="T extends Record<string, string>">
            1/1
        </script>
        "#;

        let result = parse_vue(source_text);
        assert_eq!(result.source_type, SourceType::ts());
        assert_eq!(result.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_ts_flag_3() {
        let source_text = r"
        <script lang='ts' setup>
            1/1
        </script>
        ";

        let result = parse_vue(source_text);
        assert_eq!(result.source_type, SourceType::ts());
        assert_eq!(result.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_escape_string() {
        let source_text = r"
        <script setup>
            a.replace(/&#39;/g, '\''))
        </script>
        <template> </template>
        ";

        let result = parse_vue(source_text);
        assert!(!result.source_type.is_typescript());
        assert_eq!(result.source_text.trim(), r"a.replace(/&#39;/g, '\''))");
    }

    #[test]
    fn test_multi_level_template_literal() {
        let source_text = r"
        <script setup>
            `a${b( `c \`${d}\``)}`
        </script>
        ";

        let result = parse_vue(source_text);
        assert_eq!(result.source_text.trim(), r"`a${b( `c \`${d}\``)}`");
    }

    #[test]
    fn test_brace_with_regex_in_template_literal() {
        let source_text = r"
        <script setup>
            `${/{/}`
        </script>
        ";

        let result = parse_vue(source_text);
        assert_eq!(result.source_text.trim(), r"`${/{/}`");
    }

    #[test]
    fn test_no_script() {
        let source_text = r"
            <template></template>
        ";

        let sources = VuePartialLoader::new(source_text).parse();
        assert!(sources.is_empty());
    }

    #[test]
    fn test_syntax_error() {
        let source_text = r"
        <script>
            console.log('error')
        ";
        let sources = VuePartialLoader::new(source_text).parse();
        assert!(sources.is_empty());
    }

    #[test]
    fn test_multiple_scripts() {
        let source_text = r"
        <template></template>
        <script>a</script>
        <script setup>b</script>
        ";
        let sources = VuePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text, "a");
        assert_eq!(sources[1].source_text, "b");
    }

    #[test]
    fn test_unicode() {
        let source_text = r"
        <script setup>
        let 日历 = '2000年';
        const t = useTranslate({
            'zh-CN': {
                calendar: '日历',
                tiledDisplay: '平铺展示',
            },
        });
        </script>
        ";

        let result = parse_vue(source_text);
        assert_eq!(
            result.source_text.trim(),
            "let 日历 = '2000年';
        const t = useTranslate({
            'zh-CN': {
                calendar: '日历',
                tiledDisplay: '平铺展示',
            },
        });"
            .trim()
        );
    }

    #[test]
    fn test_script_in_template() {
        let source_text = r"
        <template><script-view /></template>
        <script>a</script>
        ";
        let sources = VuePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "a");
    }

    #[test]
    fn test_closing_character_inside_attribute() {
        let source_text = r"
        <script description='PI > 5'>a</script>
        ";

        let result = parse_vue(source_text);
        assert_eq!(result.source_text, "a");
    }

    #[test]
    fn lang() {
        let cases = [
            ("<script>debugger</script>", Some(SourceType::mjs())),
            ("<script lang = 'tsx' >debugger</script>", Some(SourceType::tsx())),
            (r#"<script lang = "cjs" >debugger</script>"#, Some(SourceType::cjs())),
            ("<script lang=tsx>debugger</script>", Some(SourceType::tsx())),
            ("<script lang = 'xxx'>debugger</script>", None),
            (r#"<script lang = "xxx">debugger</script>"#, None),
            ("<script lang='xxx'>debugger</script>", None),
            (r#"<script lang="xxx">debugger</script>"#, None),
        ];

        for (source_text, source_type) in cases {
            let sources = VuePartialLoader::new(source_text).parse();
            if let Some(expected) = source_type {
                assert_eq!(sources.len(), 1);
                assert_eq!(sources[0].source_type, expected);
            } else {
                assert_eq!(sources.len(), 0);
            }
        }
    }
}
