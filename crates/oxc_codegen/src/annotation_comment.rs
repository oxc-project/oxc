use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;
use oxc_ast::{Comment, CommentKind};

use crate::Codegen;
static MATCHER: Lazy<DoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    let patterns = vec!["#__NO_SIDE_EFFECTS__", "@__NO_SIDE_EFFECTS__", "@__PURE__", "#__PURE__"];

    DoubleArrayAhoCorasick::new(patterns).unwrap()
});

impl<'a, const MINIFY: bool> Codegen<'a, MINIFY> {
    pub(crate) fn get_leading_annotate_comment(&mut self, node_start: u32) -> Option<Comment> {
        let maybe_leading_comment = self.try_get_leading_comment(node_start);
        let comment = maybe_leading_comment?;
        let real_end = match comment.kind {
            CommentKind::SingleLine => comment.span.end,
            CommentKind::MultiLine => comment.span.end + 2,
        };
        let source_code = self.source_text;
        let content_between = &source_code[real_end as usize..node_start as usize];
        // Used for VariableDeclaration (Rollup only respects "const" and only for the first one)
        if content_between.chars().all(|ch| ch.is_ascii_whitespace()) {
            let comment_content =
                &source_code[comment.span.start as usize..comment.span.end as usize];
            if MATCHER.find_iter(&comment_content).next().is_some() {
                return Some(*comment);
            }
            None
        } else {
            None
        }
    }

    pub(crate) fn print_comment(&mut self, comment: Comment) {
        // ```js
        // /*#__PURE__*/
        // Object.getOwnPropertyNames(Symbol)
        // // ios10.x Object.getOwnPropertyNames(Symbol) can enumerate 'arguments' and 'caller'
        // // but accessing them on Symbol leads to TypeError because Symbol is a strict mode
        // // function
        //   .filter(key => key !== 'arguments' && key !== 'caller')
        //   .map(key => (Symbol)[key])
        //   .filter(isSymbol),
        // ```
        // in this example, `Object.getOwnPropertyNames(Symbol)` and `Object.getOwnPropertyNames(Symbol).filter()`, `Object.getOwnPropertyNames(Symbol).filter().map()`
        // share the same leading comment. since they both are call expr and has same span start, we need to avoid print the same comment multiple times.
        if self.latest_consumed_comment_end >= comment.span.end {
            return;
        }
        self.latest_consumed_comment_end = comment.span.end;
        match comment.kind {
            CommentKind::SingleLine => {
                self.print_str("//");
                self.print_range_of_source_code(
                    comment.span.start as usize..comment.span.end as usize,
                );
                self.print_soft_newline();
                self.print_indent();
            }
            CommentKind::MultiLine => {
                self.print_str("/*");
                self.print_range_of_source_code(
                    comment.span.start as usize..comment.span.end as usize,
                );
                self.print_str("*/");
                self.print_soft_space();
            }
        }
    }

    pub(crate) fn gen_comment(&mut self, node_start: u32) {
        if !self.comment_options.preserve_annotate_comments {
            return;
        }
        if let Some(comment) = self.try_take_moved_comment(node_start) {
            self.print_comment(comment);
        }
        let maybe_leading_annotate_comment = self.get_leading_annotate_comment(node_start);
        if let Some(comment) = maybe_leading_annotate_comment {
            self.print_comment(comment);
        }
    }
}
