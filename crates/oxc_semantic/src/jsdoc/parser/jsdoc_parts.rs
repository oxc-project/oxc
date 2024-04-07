use oxc_span::Span;

#[derive(Debug, Clone, Copy)]
pub struct JSDocCommentPart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocCommentPart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        Self { raw: part_content, span }
    }

    // TODO: single line span?
    // TODO: trimmed span?
    // Use `Span` for `@kind` part instead of whole tag.
    //
    // For example, whole `tag.span` in the following JSDoc will be:
    // /**
    //  * @kind1 bar
    //  * baz...
    //  * @kind2
    //  */
    // for `@kind1`: `@kind1 bar\n * baz...\n * `
    // for `@kind2`: `@kind2\n `
    //
    // It's too verbose and may not fit for linter diagnostics span.

    pub fn parsed(&self) -> String {
        let lines = self.raw.lines();

        // If single line, there is no leading `*`
        if lines.clone().count() == 1 {
            return self.raw.trim().to_string();
        }

        self.raw
            .lines()
            // Trim leading the first `*` in each line
            .map(|line| line.trim().strip_prefix('*').unwrap_or(line).trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JSDocTagKindPart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagKindPart<'a> {
    pub fn new(part_content: &'a str, span: Span) -> Self {
        debug_assert!(part_content.starts_with('@'));

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        // +1 for `@`
        &self.raw[1..]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JSDocTagTypePart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagTypePart<'a> {
    pub fn new(part_content: &'a str, span_start: u32) -> Self {
        debug_assert!(part_content.starts_with('{'));
        debug_assert!(part_content.ends_with('}'));

        let span = Span::new(
            span_start,
            span_start + u32::try_from(part_content.len()).unwrap_or_default(),
        );

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        // +1 for `{`, -1 for `}`
        &self.raw[1..self.raw.len() - 1]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JSDocTagTypeNamePart<'a> {
    raw: &'a str,
    pub span: Span,
}
impl<'a> JSDocTagTypeNamePart<'a> {
    pub fn new(part_content: &'a str, span_start: u32) -> Self {
        let span = Span::new(
            span_start,
            span_start + u32::try_from(part_content.len()).unwrap_or_default(),
        );

        Self { raw: part_content, span }
    }

    pub fn parsed(&self) -> &str {
        self.raw
    }
}
