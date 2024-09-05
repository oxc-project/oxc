use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;
use oxc_ast::{Comment, CommentKind};
use oxc_span::Span;

use crate::Codegen;
static MATCHER: Lazy<DoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    let patterns = vec!["#__NO_SIDE_EFFECTS__", "@__NO_SIDE_EFFECTS__", "@__PURE__", "#__PURE__"];

    DoubleArrayAhoCorasick::new(patterns).unwrap()
});

bitflags::bitflags! {
    /// In theory this should be a enum,but using bitflags is easy to merge many flags into one
    /// bitset, which is used to unique annotation comment in codegen
    #[derive(Debug, Default, Clone, Copy)]
    pub(crate) struct AnnotationKind: u8 {
        const NO_SIDE_EFFECTS = 1 << 0;
        const PURE = 1 << 1;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnnotationComment {
    pub(crate) annotation_kind: AnnotationKind,
    pub(crate) comment: Comment,
}

impl AnnotationComment {
    pub fn annotation_kind(&self) -> AnnotationKind {
        self.annotation_kind
    }

    pub fn span(&self) -> Span {
        self.comment.span
    }
    pub fn kind(&self) -> CommentKind {
        self.comment.kind
    }
}

impl From<(Comment, AnnotationKind)> for AnnotationComment {
    fn from(value: (Comment, AnnotationKind)) -> Self {
        Self { annotation_kind: value.1, comment: value.0 }
    }
}

impl<'a> Codegen<'a> {
    pub(crate) fn get_leading_annotate_comments(
        &mut self,
        node_start: u32,
    ) -> Vec<AnnotationComment> {
        if !self.preserve_annotate_comments() {
            return vec![];
        }
        let mut latest_comment_start = node_start;
        let source_text = self.source_text.unwrap_or_default();
        let mut ret = self
            .get_leading_comments(self.latest_consumed_comment_end, node_start)
            .rev()
            // each comment should be separated by whitespaces
            .take_while(|comment| {
                let comment_end = comment.real_span_end();
                let range_content =
                    &source_text[comment_end as usize..latest_comment_start as usize];
                let all_whitespace = range_content.chars().all(char::is_whitespace);
                latest_comment_start = comment.real_span_start();
                all_whitespace
            })
            .filter_map(|comment| {
                let comment_content =
                    &source_text[comment.span.start as usize..comment.span.end as usize];
                if let Some(m) = MATCHER.find_iter(&comment_content).next() {
                    let annotation_kind = match m.value() {
                        0 | 1 => AnnotationKind::NO_SIDE_EFFECTS,
                        2 | 3 => AnnotationKind::PURE,
                        _ => unreachable!(),
                    };
                    return Some((*comment, annotation_kind).into());
                }
                None
            })
            .collect::<Vec<_>>();
        ret.reverse();
        ret
    }

    pub(crate) fn print_comment(&mut self, comment: AnnotationComment) {
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
        let comment_span = comment.span();
        let real_span_end = comment.comment.real_span_end();
        if self.latest_consumed_comment_end >= real_span_end {
            return;
        }
        self.update_last_consumed_comment_end(real_span_end);
        match comment.kind() {
            CommentKind::SingleLine => {
                self.print_str("//");
                self.print_range_of_source_code(
                    comment_span.start as usize..comment_span.end as usize,
                );
                self.print_soft_newline();
                self.print_indent();
            }
            CommentKind::MultiLine => {
                self.print_str("/*");
                self.print_range_of_source_code(
                    comment_span.start as usize..comment_span.end as usize,
                );
                self.print_str("*/");
                self.print_soft_space();
            }
        }
        // FIXME: esbuild function `restoreExprStartFlags`
        self.start_of_default_export = self.code_len();
    }

    pub(crate) fn gen_comments(&mut self, node_start: u32) {
        if !self.preserve_annotate_comments() {
            return;
        }
        let mut annotation_kind_set = AnnotationKind::empty();
        if let Some(comments) = self.try_take_moved_comment(node_start) {
            self.print_comments(&comments, &mut annotation_kind_set);
        }
        let leading_annotate_comments = self.get_leading_annotate_comments(node_start);
        self.print_comments(&leading_annotate_comments, &mut annotation_kind_set);
    }

    #[inline]
    pub(crate) fn print_comments(
        &mut self,
        leading_annotate_comment: &Vec<AnnotationComment>,
        annotation_kind_set: &mut AnnotationKind,
    ) {
        for &comment in leading_annotate_comment {
            let kind = comment.annotation_kind();
            if !annotation_kind_set.contains(kind) {
                annotation_kind_set.insert(kind);
                self.print_comment(comment);
            }
        }
    }

    #[inline]
    pub fn update_last_consumed_comment_end(&mut self, end: u32) {
        self.latest_consumed_comment_end = self.latest_consumed_comment_end.max(end);
    }
}
