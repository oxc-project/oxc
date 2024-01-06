use memchr::memmem::Finder;

use oxc_span::SourceType;

use super::{JavaScriptSource, SCRIPT_END, SCRIPT_START};

pub struct SveltePartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> SveltePartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        self.parse_script().map_or_else(Vec::new, |source| vec![source])
    }

    fn parse_script(&self) -> Option<JavaScriptSource<'a>> {
        let script_start_finder = Finder::new(SCRIPT_START);
        let script_end_finder = Finder::new(SCRIPT_END);

        let mut pointer = 0;

        // find opening "<script"
        let offset = script_start_finder.find(self.source_text[pointer..].as_bytes())?;
        pointer += offset + SCRIPT_START.len();

        // find closing ">"
        let offset = self.source_text[pointer..].find('>')?;

        // get lang="ts" attribute
        let content = &self.source_text[pointer..pointer + offset];
        let is_ts = content.contains("ts");

        pointer += offset + 1;
        let js_start = pointer;

        // find "</script>"
        let offset = script_end_finder.find(self.source_text[pointer..].as_bytes())?;
        let js_end = pointer + offset;

        let source_text = &self.source_text[js_start..js_end];
        let source_type = SourceType::default().with_module(true).with_typescript(is_ts);
        Some(JavaScriptSource::new(source_text, source_type))
    }
}
