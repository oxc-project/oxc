use memchr::memmem;

use oxc_span::{SourceType, Span};

use super::JavaScriptSource;

pub struct AstroPartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> AstroPartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        let mut results = vec![];
        results.extend(self.parse_frontmatter());
        results
    }

    /// Parse `---` front matter block
    #[allow(clippy::cast_possible_truncation)]
    fn parse_frontmatter(&self) -> Option<JavaScriptSource<'a>> {
        let split = "---";
        let indexes = memmem::find_iter(self.source_text.as_bytes(), split).collect::<Vec<_>>();
        if indexes.len() <= 1 {
            return None;
        }

        let start = indexes.first()?;
        let end = indexes.last()?;
        let Ok(start) = u32::try_from(*start) else { return None };
        let Ok(end) = u32::try_from(*end) else { return None };

        let js_code = Span::new(start + split.len() as u32, end).source_text(self.source_text);
        Some(JavaScriptSource::new(
            js_code,
            SourceType::default().with_typescript(true).with_module(true),
        ))
    }
}
