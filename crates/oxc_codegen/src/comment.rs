use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;

use oxc_ast::{Comment, CommentKind};
use oxc_syntax::identifier::is_line_terminator;

use crate::Codegen;

static ANNOTATION_MATCHER: Lazy<DoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    let patterns = vec!["#__NO_SIDE_EFFECTS__", "@__NO_SIDE_EFFECTS__", "@__PURE__", "#__PURE__"];

    DoubleArrayAhoCorasick::new(patterns).unwrap()
});

pub(crate) type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

impl<'a> Codegen<'a> {
    pub(crate) fn preserve_annotate_comments(&self) -> bool {
        self.comment_options.preserve_annotate_comments && !self.options.minify
    }

    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        for comment in comments {
            self.comments.entry(comment.attached_to).or_default().push(*comment);
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.contains_key(&start)
    }

    pub(crate) fn has_annotation_comment(&self, start: u32) -> bool {
        if !self.preserve_annotate_comments() {
            return false;
        }
        let Some(source_text) = self.source_text else { return false };
        self.comments.get(&start).is_some_and(|comments| {
            comments.iter().any(|comment| Self::is_annotation_comment(comment, source_text))
        })
    }

    pub(crate) fn has_non_annotation_comment(&self, start: u32) -> bool {
        if !self.preserve_annotate_comments() {
            return self.has_comment(start);
        }
        let Some(source_text) = self.source_text else { return false };
        self.comments.get(&start).is_some_and(|comments| {
            comments.iter().any(|comment| !Self::is_annotation_comment(comment, source_text))
        })
    }

    /// Weather to keep leading comments.
    fn is_leading_comments(comment: &Comment, source_text: &str) -> bool {
        (comment.is_jsdoc(source_text) || (comment.is_line() && Self::is_annotation_comment(comment, source_text)))
            && comment.preceded_by_newline
            // webpack comment `/*****/`
            && !comment.span.source_text(source_text).chars().all(|c| c == '*')
    }

    fn print_comment(&mut self, comment: &Comment, source_text: &str) {
        let comment_source = comment.real_span().source_text(source_text);
        match comment.kind {
            CommentKind::Line => {
                self.print_str(comment_source);
            }
            CommentKind::Block => {
                // Print block comments with our own indentation.
                let lines = comment_source.split(is_line_terminator);
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

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if self.options.minify {
            return;
        }
        let Some(source_text) = self.source_text else { return };
        let Some(comments) = self.comments.remove(&start) else {
            return;
        };

        let (comments, unused_comments): (Vec<_>, Vec<_>) = comments
            .into_iter()
            .partition(|comment| Self::is_leading_comments(comment, source_text));

        if comments.first().is_some_and(|c| c.preceded_by_newline) {
            // Skip printing newline if this comment is already on a newline.
            if self.peek_nth(0).is_some_and(|c| c != '\n' && c != '\t') {
                self.print_hard_newline();
                self.print_indent();
            }
        }

        for (i, comment) in comments.iter().enumerate() {
            if i >= 1 && comment.preceded_by_newline {
                self.print_hard_newline();
                self.print_indent();
            }

            self.print_comment(comment, source_text);
        }

        if comments.last().is_some_and(|c| c.is_line() || c.followed_by_newline) {
            self.print_hard_newline();
            self.print_indent();
        }

        if !unused_comments.is_empty() {
            self.comments.insert(start, unused_comments);
        }
    }

    fn is_annotation_comment(comment: &Comment, source_text: &str) -> bool {
        let comment_content = comment.span.source_text(source_text);
        ANNOTATION_MATCHER.find_iter(comment_content).count() != 0
    }

    pub(crate) fn print_annotation_comments(&mut self, node_start: u32) {
        if !self.preserve_annotate_comments() {
            return;
        }

        // If there is has annotation comments awaiting move to here, print them.
        let start = self.start_of_annotation_comment.take().unwrap_or(node_start);

        let Some(source_text) = self.source_text else { return };
        let Some(comments) = self.comments.remove(&start) else { return };

        for comment in comments {
            if !Self::is_annotation_comment(&comment, source_text) {
                continue;
            }
            if comment.is_line() {
                self.print_str("/*");
                self.print_str(comment.span.source_text(source_text));
                self.print_str("*/");
            } else {
                self.print_str(comment.real_span().source_text(source_text));
            }
            self.print_hard_space();
        }
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        if self.options.minify {
            return false;
        }
        let Some(source_text) = self.source_text else { return false };
        let Some(comments) = self.comments.remove(&start) else { return false };

        let (annotation_comments, comments): (Vec<_>, Vec<_>) = comments
            .into_iter()
            .partition(|comment| Self::is_annotation_comment(comment, source_text));

        if !annotation_comments.is_empty() {
            self.comments.insert(start, annotation_comments);
        }

        for comment in &comments {
            self.print_hard_newline();
            self.print_indent();
            self.print_comment(comment, source_text);
        }

        if comments.is_empty() {
            false
        } else {
            self.print_hard_newline();
            true
        }
    }
}
