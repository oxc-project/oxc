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
    fn test_count_comment_lines() {
        let cases: Vec<(&str, SourceType, Vec<usize>)> = vec![
            // Line comments - standalone
            ("// This is a comment\nlet x = 1;", SourceType::default(), vec![1]),
            ("    // This is a comment\nlet x = 1;", SourceType::default(), vec![1]),
            // Line comments - with code on same line
            ("let x = 1; // comment", SourceType::default(), vec![0]),
            // Multiple line comments
            (
                "// Comment 1\n// Comment 2\n// Comment 3\nlet x = 1;",
                SourceType::default(),
                vec![1, 1, 1],
            ),
            // Block comments - single line standalone
            ("/* comment */\nlet x = 1;", SourceType::default(), vec![1]),
            // Block comments - single line with code
            ("let x = /* comment */ 1;", SourceType::default(), vec![0]),
            // Block comments - empty
            ("/**/\nlet x = 1;", SourceType::default(), vec![1]),
            // Block comments - multiline with delimiters on own lines
            (
                "/*\n * This is a\n * multi-line comment\n */\nlet x = 1;",
                SourceType::default(),
                vec![4],
            ),
            // Block comments - multiline with leading whitespace
            ("    /*\n     * Comment\n     */\nlet x = 1;", SourceType::default(), vec![3]),
            // Block comments - multiline with start delimiter sharing line with code
            ("let y = 2; /*\n * Comment\n */\nlet x = 1;", SourceType::default(), vec![2]),
            // Block comments - multiline with end delimiter sharing line with code
            ("/*\n * Comment\n */ let x = 1;", SourceType::default(), vec![2]),
            // Block comments - multiline with both delimiters sharing lines with code
            ("let y = 2; /*\n * Comment\n */ let x = 1;", SourceType::default(), vec![1]),
            // JSX comments
            (
                r"export function Component() {
        // hello
        /*
        cons
        */
  return (
    <div>
      {/* hello */}
      <span>content</span>
    </div>
  );
}",
                SourceType::tsx(),
                vec![1, 3, 0],
            ),
        ];

        for (source, source_type, expected_counts) in cases {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.errors.is_empty());
            let semantic = SemanticBuilder::new().build(&ret.program).semantic;
            let comments = semantic.comments();

            assert_eq!(
                comments.len(),
                expected_counts.len(),
                "Expected {} comments in source: {:?}",
                expected_counts.len(),
                source
            );

            for (i, expected_count) in expected_counts.iter().enumerate() {
                let actual_count = count_comment_lines(&comments[i], source);
                assert_eq!(
                    actual_count, *expected_count,
                    "Comment {i} in source {source:?} expected {expected_count} lines but got {actual_count}",
                );
            }
        }
    }
}
