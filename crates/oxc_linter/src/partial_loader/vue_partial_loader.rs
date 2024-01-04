use std::str::Chars;

use oxc_span::Span;

use super::PartialLoaderValue;

pub struct VuePartialLoader<'a> {
    source_text: &'a str,
    chars: Chars<'a>,
    /// JS code start position
    start: u32,
    /// JS code end position
    end: u32,
    is_ts: bool,
    is_jsx: bool,
    is_reading_js: bool,
    /// Record current <template> 's depth
    template_depth: u32,
}

impl<'a> VuePartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self {
            source_text,
            chars: source_text.chars(),
            start: 0,
            end: 0,
            is_ts: false,
            is_jsx: false,
            is_reading_js: false,
            template_depth: 0,
        }
    }

    pub fn build(mut self) -> Option<PartialLoaderValue<'a>> {
        self.parse();
        if self.end <= self.start {
            return None;
        }
        let js_code = Span::new(self.start, self.end).source_text(self.source_text);
        Some(PartialLoaderValue::new(js_code, self.is_ts, self.is_jsx))
    }

    fn parse(&mut self) {
        while let Some(ch) = self.advance() {
            if self.is_reading_js {
                match ch {
                    '<' => {
                        if self.can_eat("/script>") {
                            self.is_reading_js = false;
                            // minus 1 for '<'
                            self.end = self.offset() - 1;
                            break;
                        }
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
                self.start = self.offset();
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

        while let Some(c) = self.advance() {
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
                        self.skip_to_end_dollar_brace();
                    }
                }
                _ => last_is_escape = false,
            }
        }
    }
    fn skip_to_end_dollar_brace(&mut self) {
        self.advance();
        let mut brace_depth = 0;
        let mut last_is_escape = false;

        while let Some(c) = self.advance() {
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
    }
    fn skip_until_next_delimiter(&mut self, delimiter: char) {
        let mut last_is_escape = false;
        for c in self.chars.by_ref() {
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
    }
    // O(1) time complexity
    // More info: https://oxc-project.github.io/docs/learn/parser_in_rust/lexer.html#token
    #[allow(clippy::cast_possible_truncation)]
    fn offset(&self) -> u32 {
        (self.source_text.len() - self.chars.as_str().len()) as u32
    }
    fn eat(&mut self, target: &str) -> bool {
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
        for ch in self.chars.by_ref() {
            if target == ch {
                return true;
            }
        }

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
}

#[cfg(test)]
mod test {
    use super::VuePartialLoader;

    #[test]
    fn test_parse_vue_one_line() {
        let source_text = r#"
        <template>
          <h1>hello world</h1>
        </template>
        <script> console.log("hi") </script>
        "#;

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
        assert_eq!(loader_value.source_text, r#" console.log("hi") "#);
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(
            loader_value.source_text,
            r#"
            console.log("hi")
            console.log("I am multi line")
            console.log("<script></script>")
            console.log(`<script></script>`)
            console.log('<script></script>')
        "#
        );
    }

    #[test]
    fn test_build_vue_with_ts_flag_1() {
        let source_text = r#"
        <script lang="ts" setup>
            1/1
        </script>
        "#;

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
        assert!(!loader_value.source_type.is_typescript());
        assert_eq!(loader_value.source_text.trim(), "console.log('success')");
    }

    #[test]
    fn test_no_script() {
        let source_text = r"
            <template></template>
        ";

        let loader_value = VuePartialLoader::new(source_text).build();
        assert!(loader_value.is_none());
    }

    #[test]
    fn test_syntax_error() {
        let source_text = r"
        <script>
            console.log('error')
        ";
        let loader_value = VuePartialLoader::new(source_text).build();
        assert!(loader_value.is_none());
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

        let loader_value = VuePartialLoader::new(source_text).build().unwrap();
        assert_eq!(
            loader_value.source_text.trim(),
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
}
