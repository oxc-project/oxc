use memchr::memmem;
use oxc_span::Span;

use super::PartialLoaderValue;

pub struct AstroPartialLoader<'a> {
    source_text: &'a str,
    /// JS code start position
    start: u32,
    /// JS code end position
    end: u32,
}

impl<'a> AstroPartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text, start: 0, end: 0 }
    }

    pub fn build(mut self) -> Option<PartialLoaderValue<'a>> {
        self.parse();
        if self.end <= self.start {
            return None;
        }
        let js_code = Span::new(self.start, self.end).source_text(self.source_text);
        Some(PartialLoaderValue::new(js_code, /* is_ts */ true, /* self.is_jsx */ false))
    }

    fn parse(&mut self) {
        let indexes = memmem::find_iter(self.source_text.as_bytes(), "---").collect::<Vec<_>>();
        if indexes.len() <= 1 {
            return;
        }
        let Some(start) = indexes.first() else { return };
        let Some(end) = indexes.last() else { return };
        let Ok(start) = u32::try_from(*start) else { return };
        let Ok(end) = u32::try_from(*end) else { return };
        self.start = start + 3;
        self.end = end;
    }
}
