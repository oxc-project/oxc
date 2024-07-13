use std::borrow::Cow;

use oxc_span::{Span, SPAN};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Fix<'a> {
    pub content: Cow<'a, str>,
    pub span: Span,
}

impl Default for Fix<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a> Fix<'a> {
    pub const fn delete(span: Span) -> Self {
        Self { content: Cow::Borrowed(""), span }
    }

    pub fn new<T: Into<Cow<'a, str>>>(content: T, span: Span) -> Self {
        Self { content: content.into(), span }
    }

    /// Creates a [`Fix`] that doesn't change the source code.
    #[inline]
    pub const fn empty() -> Self {
        Self { content: Cow::Borrowed(""), span: SPAN }
    }
}

#[derive(Debug, Default)]
pub enum CompositeFix<'a> {
    /// No fixes
    #[default]
    None,
    Single(Fix<'a>),
    /// Several fixes that will be merged into one, in order.
    Multiple(Vec<Fix<'a>>),
}

impl<'a> From<Fix<'a>> for CompositeFix<'a> {
    fn from(fix: Fix<'a>) -> Self {
        CompositeFix::Single(fix)
    }
}

impl<'a> From<Option<Fix<'a>>> for CompositeFix<'a> {
    fn from(fix: Option<Fix<'a>>) -> Self {
        match fix {
            Some(fix) => CompositeFix::Single(fix),
            None => CompositeFix::None,
        }
    }
}

impl<'a> From<Vec<Fix<'a>>> for CompositeFix<'a> {
    fn from(fixes: Vec<Fix<'a>>) -> Self {
        if fixes.is_empty() {
            CompositeFix::None
        } else {
            CompositeFix::Multiple(fixes)
        }
    }
}

impl<'a> CompositeFix<'a> {
    /// Gets one fix from the fixes. If we retrieve multiple fixes, this merges those into one.
    /// <https://github.com/eslint/eslint/blob/main/lib/linter/report-translator.js#L181-L203>
    pub fn normalize_fixes(self, source_text: &str) -> Fix<'a> {
        match self {
            CompositeFix::Single(fix) => fix,
            CompositeFix::Multiple(fixes) => Self::merge_fixes(fixes, source_text),
            CompositeFix::None => Fix::empty(),
        }
    }
    /// Merges multiple fixes to one, returns an `Fix::default`(which will not fix anything) if:
    ///
    /// 1. `fixes` is empty
    /// 2. contains overlapped ranges
    /// 3. contains negative ranges (span.start > span.end)
    ///
    /// <https://github.com/eslint/eslint/blob/main/lib/linter/report-translator.js#L147-L179>
    fn merge_fixes(fixes: Vec<Fix<'a>>, source_text: &str) -> Fix<'a> {
        let mut fixes = fixes;
        if fixes.is_empty() {
            // Do nothing
            return Fix::empty();
        }
        if fixes.len() == 1 {
            return fixes.pop().unwrap();
        }

        fixes.sort_by(|a, b| a.span.cmp(&b.span));

        // safe, as fixes.len() > 1
        let start = fixes[0].span.start;
        let end = fixes[fixes.len() - 1].span.end;
        let mut last_pos = start;
        let mut output = String::new();

        for fix in fixes {
            let Fix { ref content, span } = fix;
            // negative range or overlapping ranges is invalid
            if span.start > span.end {
                debug_assert!(false, "Negative range is invalid: {span:?}");
                return Fix::empty();
            }
            if last_pos > span.start {
                debug_assert!(
                    false,
                    "Fix must not be overlapped, last_pos: {}, span.start: {}",
                    last_pos, span.start
                );
                return Fix::empty();
            }

            let Some(before) = source_text.get((last_pos) as usize..span.start as usize) else {
                debug_assert!(false, "Invalid range: {}, {}", last_pos, span.start);
                return Fix::empty();
            };

            output.reserve(before.len() + content.len());
            output.push_str(before);
            output.push_str(content);
            last_pos = span.end;
        }

        let Some(after) = source_text.get(last_pos as usize..end as usize) else {
            debug_assert!(false, "Invalid range: {:?}", last_pos as usize..end as usize);
            return Fix::empty();
        };

        output.push_str(after);
        output.shrink_to_fit();
        Fix::new(output, Span::new(start, end))
    }
}
