use std::str::Chars;

use oxc_span::Span;

use super::PartialLoaderValue;

pub struct VuePartialLoader<'a> {
    source_text: &'a str,
    chars: Chars<'a>,
    code: Vec<u8>,
    is_ts: bool,
    is_jsx: bool,
    is_reading_js: bool,
    /// Record current <template> 's depth
    template_depth: u32,
}

impl<'a> VuePartialLoader<'a> {
    pub fn from(source_text: &'a str) -> Self {
        Self {
            source_text,
            chars: source_text.chars(),
            code: vec![],
            is_ts: false,
            is_jsx: false,
            is_reading_js: false,
            template_depth: 0,
        }
    }
    pub fn build(mut self) -> PartialLoaderValue {
        self.parse();
        // SAFETY: criteria of `from_utf8_unchecked`.are met.
        let js_content = unsafe { String::from_utf8_unchecked(self.code) };
        PartialLoaderValue::from(js_content, self.is_ts, self.is_jsx)
    }
    fn parse(&mut self) {
        while let Some(ch) = self.advance() {
            // if ch equals '<', we should check if it is the started char of </script>
            let may_end_script_tag = self.is_reading_js && ch == '<';
            if !may_end_script_tag {
                self.push_ch_or_space(ch);
            }

            if self.is_reading_js {
                match ch {
                    '<' => {
                        if self.can_eat("/script>") {
                            self.is_reading_js = false;
                            break;
                        }
                        self.push_ch_or_space(ch);
                    }
                    '\'' | '"' => {
                        self.skip_until_next_delimiter(ch);
                    }
                    '`' => {
                        self.skip_to_end_of_template_literal();
                    }
                    '/' => {
                        self.try_read_comment();
                    }
                    _ => {}
                }
            } else if ch == '<' {
                self.try_read_tag_name();
            }
        }
    }

    fn try_read_tag_name(&mut self) {
        if self.eat("template") {
            self.template_depth += 1;
        } else if self.eat("/template>") {
            self.template_depth -= 1;
        } else if self.template_depth == 0 && self.eat("script") {
            let open_tag_start = self.offset();
            if self.eat_to('>') {
                let open_tag_end = self.offset();

                let attributes_text =
                    Span::new(open_tag_start, open_tag_end).source_text(self.source_text);
                self.is_ts = VuePartialLoader::contains_ts_flag(attributes_text);
                self.is_jsx = VuePartialLoader::contains_jsx_flag(attributes_text);
                self.is_reading_js = true;
            }
        }
    }

    fn try_read_comment(&mut self) {
        match self.peek() {
            // single line comment
            Some('/') => {
                self.eat_to('\n');
            }
            // multi line comment
            Some('*') => {
                while let Some(c) = self.advance() {
                    if c == '*' && self.peek() == Some('/') {
                        self.eat("/");
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
    fn skip_to_end_of_template_literal(&mut self) {
        let mut last_is_escape = false;
        let mut chars = vec![];

        while let Some(c) = self.advance() {
            chars.push(c as u8);
            if last_is_escape {
                last_is_escape = false;
                continue;
            }
            match c {
                '\\' => last_is_escape = true,
                '`' => {
                    break;
                }
                '$' => {
                    if self.peek() == Some('{') {
                        let sub_chars = self.skip_to_end_dollar_brace();
                        chars.extend(sub_chars);
                    }
                }
                _ => last_is_escape = false,
            }
        }

        self.push_str_or_multi_space(chars);
    }
    fn skip_to_end_dollar_brace(&mut self) -> Vec<u8> {
        self.advance();
        let mut chars = vec![b'{'];
        let mut brace_depth = 0;
        let mut last_is_escape = false;

        while let Some(c) = self.advance() {
            chars.push(c as u8);
            if last_is_escape {
                last_is_escape = false;
                continue;
            }

            match c {
                '{' => {
                    if !last_is_escape {
                        brace_depth += 1;
                    }
                }
                '}' => {
                    if brace_depth == 0 {
                        break;
                    }
                    brace_depth -= 1;
                }
                _ => {}
            }
        }
        chars
    }
    fn skip_until_next_delimiter(&mut self, delimiter: char) {
        let mut last_is_escape = false;
        let mut chars = vec![];

        for c in self.chars.by_ref() {
            chars.push(c as u8);
            if last_is_escape {
                last_is_escape = false;
                continue;
            }
            match c {
                '\\' => last_is_escape = true,
                '\'' | '"' => {
                    if c == delimiter {
                        break;
                    }
                }
                _ => last_is_escape = false,
            }
        }

        self.push_str_or_multi_space(chars);
    }
    #[allow(clippy::cast_possible_truncation)]
    fn offset(&self) -> u32 {
        (self.source_text.len() - self.chars.as_str().len()) as u32
    }
    fn eat(&mut self, target: &str) -> bool {
        let mut chars = self.chars.clone();
        let mut code = vec![];
        for ch in target.chars() {
            code.push(ch as u8);
            if let Some(c) = chars.next() {
                if c != ch {
                    return false;
                }
            } else {
                return false;
            }
        }

        self.push_str_or_multi_space(code);
        self.chars = chars;
        true
    }
    fn can_eat(&self, target: &str) -> bool {
        let mut chars = self.chars.clone();
        for ch in target.chars() {
            if let Some(c) = chars.next() {
                if c != ch {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
    fn eat_to(&mut self, target: char) -> bool {
        let mut chars = vec![];
        for ch in self.chars.by_ref() {
            chars.push(ch as u8);
            if target == ch {
                self.push_str_or_multi_space(chars);
                return true;
            }
        }

        self.push_str_or_multi_space(chars);
        false
    }

    fn contains_ts_flag(s: &str) -> bool {
        ["lang=ts", "lang='ts'", r#"lang="ts""#, "lang=tsx", "lang='tsx'", r#"lang="tsx""#]
            .iter()
            .any(|flag| s.contains(flag))
    }

    fn contains_jsx_flag(s: &str) -> bool {
        ["lang=jsx", "lang='jsx'", r#"lang="jsx""#, "lang=tsx", "lang='tsx'", r#"lang="tsx""#]
            .iter()
            .any(|flag| s.contains(flag))
    }

    fn push_str_or_multi_space(&mut self, s: Vec<u8>) {
        for byte in s {
            self.push_ch_or_space(byte as char);
        }
    }

    fn push_ch_or_space(&mut self, ch: char) {
        if self.is_reading_js {
            self.code.push(ch as u8);
        } else {
            self.code.push(if ch == '\n' { b'\n' } else { b' ' });
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::VuePartialLoader;

    fn visualize_empty_line(source: &str) -> String {
        source
            .lines()
            .map(|line| {
                if matches!(line.trim(), "" | "\n") {
                    "-".repeat(line.len())
                } else {
                    line.to_string()
                }
            })
            .join("\n")
    }

    #[test]
    fn test_parse_vue_one_line() {
        let source_text = r#"
        <template>
          <h1>hello world</h1>
        </template>
        <script> console.log("hi") </script>
        "#;

        let loader_value = VuePartialLoader::from(source_text).build();
        assert_eq!(
            visualize_empty_line(&loader_value.source_text),
            r#"
------------------
------------------------------
-------------------
                 console.log("hi") "#
        );
    }

    #[test]
    fn test_build_vue_multi_line() {
        let source_text = r#"
        <template>
          <h1>hello world</h1>
        </template>
        <script>
            console.log("hi")
            console.log("I am multi line")
            console.log("<script></script>")
            console.log(`<script></script>`)
            console.log('<script></script>')
        </script>
        "#;

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(
            visualize_empty_line(&loader_value.source_text),
            r#"
------------------
------------------------------
-------------------
----------------
            console.log("hi")
            console.log("I am multi line")
            console.log("<script></script>")
            console.log(`<script></script>`)
            console.log('<script></script>')
--------"#
        );
    }

    #[test]
    fn test_build_vue_with_ts_flag_1() {
        let source_text = r#"
        <script lang="ts" setup>
            1/1
        </script>
        "#;

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_ts_flag_2() {
        let source_text = r"
        <script lang=ts setup>
            1/1
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_ts_flag_3() {
        let source_text = r"
        <script lang='ts' setup>
            1/1
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_tsx_flag() {
        let source_text = r"
        <script lang=tsx setup>
            1/1
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(loader_value.source_type.is_jsx());
        assert!(loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "1/1");
    }

    #[test]
    fn test_build_vue_with_escape_string() {
        let source_text = r"
        <script setup>
            a.replace(/&#39;/g, '\''))
        </script>
        <template> </template>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), r"a.replace(/&#39;/g, '\''))");
    }

    #[test]
    fn test_multi_level_template_literal() {
        let source_text = r"
        <script setup>
            `a${b( `c \`${d}\``)}`
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert_eq!(loader_value.source_text.trim(), r"`a${b( `c \`${d}\``)}`");
    }

    // TODO: fix this, current did know whether / is the start of regex or not
    #[ignore]
    #[test]
    fn test_brace_with_regex_in_template_literal() {
        let source_text = r"
        <script setup>
            `${/{/}`
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert_eq!(loader_value.source_text.trim(), r"`${/{/}`");
    }

    #[test]
    fn test_ignore_script_tag_in_template() {
        let source_text = r"
        <template>
            <script>console.log('error')</script>
        </template>
        <script>
            console.log('success')
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "console.log('success')");
    }

    #[test]
    fn test_ignore_script_tag_in_template_2() {
        let source_text = r"
        <template>
            <div>
                <template>
                    <script>console.log('error')</script>
                </template>
                <script>console.log('error')</script>
            </div>
        </template>
        <script>
            console.log('success')
        </script>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "console.log('success')");
    }

    #[test]
    fn test_no_script() {
        let source_text = r"
            <template></template>
        ";

        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "");
    }

    #[test]
    fn test_syntax_error() {
        let source_text = r"
        <script>
            console.log('error')
        ";
        let loader_value = VuePartialLoader::from(source_text).build();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "console.log('error')");
    }
}
