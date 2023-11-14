use oxc_span::Span;

pub fn is_next_line_empty(source_text: &str, span: Span) -> bool {
    source_text.chars().nth((span.end as usize) + 1).is_some_and(|c| c == '\n')
}
