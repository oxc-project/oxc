use rustc_hash::FxHashMap;

use oxc_ast::{Comment, Trivias};
use oxc_span::Span;

use crate::Codegen;

pub type CommentsMap = FxHashMap</* attached_to */ u32, Vec<Comment>>;

impl<'a> Codegen<'a> {
    pub(crate) fn build_leading_comments(&mut self, source_text: &str, trivias: &Trivias) {
        let mut leading_comments: CommentsMap = FxHashMap::default();
        for comment in trivias
            .comments()
            .copied()
            .filter(|comment| comment.attached_to != 0 && comment.is_leading())
            .filter(|comment| Self::is_pragma_comment(comment, source_text))
        {
            leading_comments.entry(comment.attached_to).or_default().push(comment);
        }
        self.leading_comments = leading_comments;
    }

    /// Only print jsdoc (block comments starting with `*`), comments starting with `@` and `#__`.
    /// Example `@` comments: `@license`, `@preserve` and `@internal`
    /// Example `#__` comments: `#__NO_SIDE_EFFECTS__`, `@__NO_SIDE_EFFECTS__`, `@__PURE__`, `#__PURE__`
    fn is_pragma_comment(comment: &Comment, source_text: &str) -> bool {
        let s = comment.span.source_text(source_text);
        // jsdoc
        if comment.is_block() && s.starts_with('*') {
            return true;
        }
        let s = s.trim_start();
        // special comments
        s.starts_with('@')
        // webpack <https://webpack.js.org/api/module-methods/#magic-comments>
        || s.starts_with("webpack")
        // `#__PURE__`
        // `#__NO_SIDE_EFFECTS__` <https://github.com/javascript-compiler-hints/compiler-notations-spec/blob/main/no-side-effects-notation-spec.md>
        || s.starts_with("#__")
        || s.starts_with("#__NO_SIDE_EFFECTS__")
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if self.options.minify {
            return;
        }
        let Some(source_text) = self.source_text else { return };
        let Some(comments) = self.leading_comments.remove(&start) else { return };

        let first = comments.first().copied().unwrap();
        let last = comments.last().copied().unwrap();

        let s = Span::new(first.real_span_start(), last.real_span_end()).source_text(source_text);
        if first.preceded_by_newline {
            // Skip if this comment is already on a newline.
            if self.peek_nth(0).is_some_and(|c| c != '\n' && c != '\t') {
                self.print_char(b'\n');
            }
            self.print_indent();
        }

        self.print_str(s);
        if last.is_line() || last.followed_by_newline {
            self.print_char(b'\n');
            self.print_indent();
        } else if last.is_block() && !last.followed_by_newline {
            // self.print_soft_space();
            self.print_next_indent_as_space = true;
        }
    }
}
