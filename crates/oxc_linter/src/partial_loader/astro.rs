use memchr::memmem::Finder;

use oxc_span::{SourceType, Span};

use super::{JavaScriptSource, SCRIPT_END, SCRIPT_START};

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

    /// Parse `---` front matter block
    #[allow(clippy::cast_possible_truncation)]
    fn parse_frontmatter(&self) -> Option<JavaScriptSource<'a>> {
        let split_finder = Finder::new(ASTRO_SPLIT);
        let offsets = split_finder.find_iter(self.source_text.as_bytes()).collect::<Vec<_>>();
        if offsets.len() <= 1 {
            return None;
        }

        let start = offsets.first()?;
        let end = offsets.last()?;
        let Ok(start) = u32::try_from(*start) else { return None };
        let Ok(end) = u32::try_from(*end) else { return None };

        let js_code =
            Span::new(start + ASTRO_SPLIT.len() as u32, end).source_text(self.source_text);
        Some(JavaScriptSource::new(
            js_code,
            SourceType::default().with_typescript(true).with_module(true),
        ))
    }

    /// In .astro files, you can add client-side JavaScript by adding one (or more) <script> tags.
    /// https://docs.astro.build/en/guides/client-side-scripts/#using-script-in-astro
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
            // find "</script>"
            if let Some(offset) = script_end_finder.find(self.source_text[pointer..].as_bytes()) {
                js_end = pointer + offset;
                pointer += offset + SCRIPT_END.len();
            } else {
                break;
            };
            results.push(JavaScriptSource::new(
                &self.source_text[js_start..js_end],
                SourceType::default().with_typescript(true).with_module(true),
            ));
        }
        results
    }
}
