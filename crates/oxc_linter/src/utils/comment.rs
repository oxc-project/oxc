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
        end_line - start_line
    }
}
