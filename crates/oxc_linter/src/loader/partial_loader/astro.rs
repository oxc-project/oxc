use memchr::memmem::{Finder, FinderRev};

use oxc_span::{SourceType, Span};

use crate::loader::JavaScriptSource;

use super::{
    AttributeValue, COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_attribute,
    find_script_closing_angle, find_script_start,
};

const ASTRO_SPLIT: &str = "---";

pub struct AstroPartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> AstroPartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        let mut results = vec![];
        let mut start = 0;
        if let Some((frontmatter, frontmatter_end)) = self.parse_frontmatter() {
            start = frontmatter_end;
            results.push(frontmatter);
        }
        results.extend(self.parse_scripts(start));
        results
    }

    /// Parse `---` frontmatter block
    #[expect(clippy::cast_possible_truncation)]
    fn parse_frontmatter(&self) -> Option<(JavaScriptSource<'a>, usize)> {
        let split_finder = Finder::new(ASTRO_SPLIT);
        let mut offsets = split_finder
            .find_iter(self.source_text.as_bytes())
            .filter(|offset| self.is_fence(*offset));

        let start = offsets.next()?;
        if !self.source_text[..start].chars().all(char::is_whitespace) {
            return None;
        }
        let end = offsets.next()?;

        let Ok(start) = u32::try_from(start) else {
            return None;
        };
        let Ok(end) = u32::try_from(end) else {
            return None;
        };

        // move start to the end of the ASTRO_SPLIT
        let start = start + ASTRO_SPLIT.len() as u32;
        let js_code = Span::new(start, end).source_text(self.source_text);
        let frontmatter = JavaScriptSource::partial(js_code, SourceType::ts(), start);

        Some((frontmatter, end as usize + ASTRO_SPLIT.len()))
    }

    fn is_fence(&self, offset: usize) -> bool {
        let line_start = self.source_text[..offset].rfind('\n').map_or(0, |index| index + 1);
        if !self.source_text[line_start..offset].chars().all(char::is_whitespace) {
            return false;
        }

        let fence_end = offset + ASTRO_SPLIT.len();
        let line_end = self.source_text[fence_end..]
            .find('\n')
            .map_or(self.source_text.len(), |index| fence_end + index);

        self.source_text[fence_end..line_end].chars().all(char::is_whitespace)
    }

    /// In .astro files, you can add client-side JavaScript by adding one (or more) `<script>` tags.
    /// <https://docs.astro.build/en/guides/client-side-scripts/#using-script-in-astro>
    fn parse_scripts(&self, start: usize) -> Vec<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);
        let script_end_finder = Finder::new(SCRIPT_END);
        let comment_start_finder = FinderRev::new(COMMENT_START);
        let comment_end_finder = Finder::new(COMMENT_END);

        let mut results = vec![];
        let mut pointer = start;

        loop {
            let js_end;
            // find opening "<script"
            if let Some(offset) = find_script_start(
                self.source_text,
                pointer,
                &script_start_finder,
                &comment_start_finder,
                &comment_end_finder,
            ) {
                pointer += offset;
            } else {
                break;
            }
            // find closing ">"
            let Some(offset) = find_script_closing_angle(self.source_text, pointer) else {
                break;
            };
            let script_opening_tag = &self.source_text[pointer..pointer + offset];
            let is_javascript_script = Self::is_javascript_script(script_opening_tag);

            pointer += offset + 1;
            let js_start = pointer;

            // check for the / of a self closing script tag
            if js_start.checked_sub(2).is_some_and(|slash_offset| {
                self.source_text.as_bytes().get(slash_offset) == Some(&b'/')
            }) {
                js_end = pointer;
            // find "</script>" if no self closing tag was found
            } else if let Some(offset) =
                script_end_finder.find(&self.source_text.as_bytes()[pointer..])
            {
                js_end = pointer + offset;
                pointer += offset + SCRIPT_END.len();
            } else {
                break;
            }

            if !is_javascript_script {
                continue;
            }

            // NOTE: loader checked that source_text.len() is less than u32::MAX
            #[expect(clippy::cast_possible_truncation)]
            results.push(JavaScriptSource::partial(
                &self.source_text[js_start..js_end],
                SourceType::ts(),
                js_start as u32,
            ));
        }
        results
    }

    fn is_javascript_script(content: &str) -> bool {
        match find_attribute(content, "type") {
            Some(AttributeValue::Empty) | None => true,
            Some(AttributeValue::Value(value)) => {
                let script_type = value.trim().split(';').next().unwrap_or("").trim();
                script_type.is_empty()
                    || [
                        "module",
                        "text/javascript",
                        "application/javascript",
                        "text/ecmascript",
                        "application/ecmascript",
                    ]
                    .iter()
                    .any(|allowed| script_type.eq_ignore_ascii_case(allowed))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AstroPartialLoader, JavaScriptSource};

    fn parse_astro(source_text: &str) -> Vec<JavaScriptSource<'_>> {
        AstroPartialLoader::new(source_text).parse()
    }

    #[test]
    fn test_parse_astro() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[0].start, 51);
    }

    #[test]
    fn test_parse_astro_with_fontmatter() {
        let source_text = r#"
        ---
            const { message = 'Welcome, world!' } = Astro.props;
        ---

        <h1>Welcome, world!</h1>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources[0].source_text.trim(),
            "const { message = 'Welcome, world!' } = Astro.props;"
        );
        assert_eq!(sources[0].start, 12);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 141);
    }

    #[test]
    fn test_parse_astro_with_inline_script() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script is:inline src="https://my-analytics.com/script.js"></script>

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert!(sources[0].source_text.is_empty());
        assert_eq!(sources[0].start, 102);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 129);
    }

    #[test]
    fn test_script_inside_code_comment() {
        let source_text = r"
        <!-- <script>a</script> -->
        <!-- <script> -->
        <script>b</script>
        ";

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "b");
        assert_eq!(sources[0].start, 79);
    }

    #[test]
    fn test_parse_astro_with_inline_script_self_closing() {
        let source_text = r#"
        <h1>Welcome, world!</h1>

        <script is:inline src="https://my-analytics.com/script.js" />

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert!(sources[0].source_text.is_empty());
        assert_eq!(sources[0].start, 104);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
        assert_eq!(sources[1].start, 122);
    }

    #[test]
    fn test_parse_astro_with_inline_script_self_closing_after_unicode() {
        let source_text = r#"
        <h1>日历</h1>

        <script is:inline src="https://my-analytics.com/script.js" />

        <script>
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert!(sources[0].source_text.is_empty());
        assert_eq!(sources[1].source_text.trim(), r#"console.log("Hi");"#);
    }

    #[test]
    fn test_parse_astro_skips_non_javascript_script_type() {
        let source_text = r#"
        <script TYPE="importmap">
            {
                "imports": {
                    "swiper": "https://cdn.jsdelivr.net/npm/swiper@12/swiper-bundle.min.mjs"
                }
            }
        </script>

        <script type="application/ld+json">
            { "@context": "https://schema.org" }
        </script>

        <script type="module">
            console.log("Hi");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("Hi");"#);
    }

    #[test]
    fn test_parse_astro_keeps_javascript_script_type() {
        let source_text = r#"
        <script type="text/javascript">
            console.log("text/javascript");
        </script>

        <script type="application/ecmascript; charset=utf-8">
            console.log("application/ecmascript");
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), r#"console.log("text/javascript");"#);
        assert_eq!(sources[1].source_text.trim(), r#"console.log("application/ecmascript");"#);
    }

    #[test]
    fn test_parse_astro_frontmatter_with_later_js_separator() {
        let source_text = r#"
        ---
        const title = "hello";
        ---

        <div>{"---"}</div>

        <script>
            const marker = "---";
            console.log(marker);
        </script>
        "#;

        let sources = parse_astro(source_text);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "const title = \"hello\";");
        assert_eq!(
            sources[1].source_text.trim(),
            "const marker = \"---\";\n            console.log(marker);"
        );
    }
}
