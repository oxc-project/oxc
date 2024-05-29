use std::usize;

use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;
use oxc_ast::{Comment, CommentKind};
use oxc_span::GetSpan;

use crate::Codegen;
static MATCHER: Lazy<DoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    let patterns = vec!["#__NO_SIDE_EFFECTS__", "@__NO_SIDE_EFFECTS__", "@__PURE__", "#__PURE__"];
    let pma = DoubleArrayAhoCorasick::new(patterns).unwrap();
    pma
});

pub fn get_leading_annotate_comment<'a>(
    node_start: u32,
    leading_comment: Option<(&u32, &Comment)>,
    source_code: &'a str,
) -> Option<(&'a str, CommentKind)> {
    let (comment_start, comment) = leading_comment?;
    let real_end = match comment.kind() {
        CommentKind::SingleLine => comment.end(),
        CommentKind::MultiLine => comment.end() + 2,
    };
    let content_between = &source_code[real_end as usize..node_start as usize];
    if content_between.chars().all(|ch| ch.is_ascii_whitespace()) {
        let comment_content = &source_code[*comment_start as usize..comment.end() as usize];
        if MATCHER.find_iter(&comment_content).next().is_some() {
            return Some((comment_content, comment.kind()));
        }
        None
    } else {
        None
    }
}

pub fn print_comment<const MINIFY: bool>(
    content: &str,
    kind: CommentKind,
    p: &mut Codegen<{ MINIFY }>,
) {
    match kind {
        CommentKind::SingleLine => {
            p.print_str("//");
            p.print_str(content);
            p.print_soft_newline();
            p.print_indent();
        }
        CommentKind::MultiLine => {
            p.print_str("/*");
            p.print_str(content);
            p.print_str("*/");
            p.print_space_before_identifier();
        }
    }
}

pub fn gen_comment<const MINIFY: bool>(node_start: u32, codegen: &mut Codegen<{ MINIFY }>) {
    if !codegen.options.preserve_annotate_comments {
        return;
    }
    let maybe_leading_comment = codegen.trivials.comments().range(0..node_start).rev().next();
    let maybe_leading_annotate_comment =
        get_leading_annotate_comment(node_start, maybe_leading_comment, codegen.source_code);
    if let Some((content, kind)) = maybe_leading_annotate_comment {
        print_comment::<MINIFY>(content, kind, codegen);
    }
}
