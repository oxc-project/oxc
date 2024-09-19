use oxc_syntax::identifier::is_line_terminator;
use rustc_hash::FxHashMap;

use oxc_ast::{Comment, CommentKind, Trivias};

use crate::Codegen;

pub type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

impl<'a> Codegen<'a> {
    pub(crate) fn build_leading_comments(&mut self, source_text: &str, trivias: &Trivias) {
        let mut leading_comments: CommentsMap = FxHashMap::default();
        for comment in trivias
            .comments()
            .copied()
            .filter(|comment| Self::should_keep_comment(comment, source_text))
        {
            leading_comments.entry(comment.attached_to).or_default().push(comment);
        }
        self.leading_comments = leading_comments;
    }

    fn should_keep_comment(comment: &Comment, source_text: &str) -> bool {
        comment.is_jsdoc(source_text)
            && comment.preceded_by_newline
            // webpack comment `/*****/`
            && !comment.span.source_text(source_text).chars().all(|c| c == '*')
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if self.options.minify {
            return;
        }
        let Some(source_text) = self.source_text else { return };
        let Some(comments) = self.leading_comments.remove(&start) else { return };

        let first = comments.first().unwrap();
        if first.preceded_by_newline {
            // Skip printing newline if this comment is already on a newline.
            if self.peek_nth(0).is_some_and(|c| c != '\n' && c != '\t') {
                self.print_char(b'\n');
                self.print_indent();
            }
        }

        for comment in &comments {
            let s = comment.real_span().source_text(source_text);
            match comment.kind {
                CommentKind::Line => {
                    self.print_str(s);
                }
                CommentKind::Block => {
                    // Print block comments with our own indentation.
                    let lines = s.split(is_line_terminator);
                    for line in lines {
                        if !line.starts_with("/*") {
                            self.print_indent();
                        }
                        self.print_str(line.trim_start());
                        if !line.ends_with("*/") {
                            self.print_hard_newline();
                        }
                    }
                }
            }
        }

        let last = comments.last().unwrap();
        if last.is_line() || last.followed_by_newline {
            self.print_hard_newline();
            self.print_indent();
        }
    }
}
