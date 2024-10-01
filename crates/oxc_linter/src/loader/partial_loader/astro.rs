use memchr::memmem::Finder;
use oxc_span::{SourceType, Span};

use super::{SCRIPT_END, SCRIPT_START};
use crate::loader::JavaScriptSource;

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
        let frontmatter = self.parse_frontmatter();
        let start = frontmatter.as_ref().map_or(0, |r| r.source_text.len() + ASTRO_SPLIT.len() * 2);
        results.extend(frontmatter);
        results.extend(self.parse_scripts(start));
        results
    }

    /// Parse `---` frontmatter block
    #[allow(clippy::cast_possible_truncation)]
    fn parse_frontmatter(&self) -> Option<JavaScriptSource<'a>> {
        let split_finder = Finder::new(ASTRO_SPLIT);
        let offsets = split_finder.find_iter(self.source_text.as_bytes()).collect::<Vec<_>>();
        if offsets.len() <= 1 {
            return None;
        }

        let start = offsets.first()?;
        let end = offsets.last()?;
        let Ok(start) = u32::try_from(*start) else {
            return None;
        };
        let Ok(end) = u32::try_from(*end) else {
            return None;
        };

        // move start to the end of the ASTRO_SPLIT
        let start = start + ASTRO_SPLIT.len() as u32;
        let js_code = Span::new(start, end).source_text(self.source_text);
        Some(JavaScriptSource::partial(js_code, SourceType::ts(), start))
    }

    /// In .astro files, you can add client-side JavaScript by adding one (or more) `<script>` tags.
    /// <https://docs.astro.build/en/guides/client-side-scripts/#using-script-in-astro>
    fn parse_scripts(&self, start: usize) -> Vec<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);
        let script_end_finder = Finder::new(SCRIPT_END);

        let mut results = vec![];
        let mut pointer = start;

        loop {
            let js_start;
            let js_end;
            // find opening "<script"
            if let Some(offset) = script_start_finder.find(self.source_text[pointer..].as_bytes()) {
                pointer += offset + SCRIPT_START.len();
            } else {
                break;
            };
            // find closing ">"
            if let Some(offset) = self.source_text[pointer..].find('>') {
                pointer += offset + 1;
                js_start = pointer;
            } else {
                break;
            };
            // check for the / of a self closing script tag
            if let Some('/') = self.source_text.chars().nth(js_start - 2) {
                js_end = pointer;
            // find "</script>" if no self closing tag was found
            } else if let Some(offset) =
                script_end_finder.find(self.source_text[pointer..].as_bytes())
            {
                js_end = pointer + offset;
                pointer += offset + SCRIPT_END.len();
            } else {
                break;
            };

            // NOTE: loader checked that source_text.len() is less than u32::MAX
            #[allow(clippy::cast_possible_truncation)]
            results.push(JavaScriptSource::partial(
                &self.source_text[js_start..js_end],
                SourceType::ts(),
                js_start as u32,
            ));
        }
        results
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
}
