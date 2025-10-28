use oxc_ast::Comment;

fn line_has_just_comment(line: &str, comment_chars: &str) -> bool {
    if let Some(line) = line.trim().strip_prefix(comment_chars) { line.is_empty() } else { false }
}

pub fn count_comment_lines(comment: &Comment, source_text: &str) -> usize {
    let comment_span = comment.content_span();
    if comment.is_line() {
        let comment_line =
            source_text[..comment_span.start as usize].lines().next_back().unwrap_or("");
        usize::from(line_has_just_comment(comment_line, "//"))
    } else {
        let mut start_line = source_text[..comment_span.start as usize].lines().count();
        let comment_start_line =
            source_text[..comment_span.start as usize].lines().next_back().unwrap_or("");
        if !line_has_just_comment(comment_start_line, "/*") {
            start_line += 1;
        }
        let mut end_line = source_text[..=comment_span.end as usize].lines().count();
        let comment_end_line =
            source_text[comment_span.end as usize..].lines().next().unwrap_or("");
        if line_has_just_comment(comment_end_line, "*/") {
            end_line += 1;
        }
        end_line.saturating_sub(start_line)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    #[test]
    fn test_jsx_comment_should_panic() {
        let source = r#"export function Component() {
  return (
    <div>
      {/* hello */}
      <span>content</span>
    </div>
  );
}"#;

        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let ret = Parser::new(&allocator, source, source_type).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        // This should panic with integer overflow
        for comment in semantic.comments() {
            let _lines = count_comment_lines(comment, source);
        }
    }
}
