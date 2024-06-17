use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;
use oxc_ast::{Comment, CommentKind};

use crate::Codegen;
static MATCHER: Lazy<DoubleArrayAhoCorasick<usize>> = Lazy::new(|| {
    let patterns = vec!["#__NO_SIDE_EFFECTS__", "@__NO_SIDE_EFFECTS__", "@__PURE__", "#__PURE__"];

    DoubleArrayAhoCorasick::new(patterns).unwrap()
});

pub fn get_leading_annotate_comment<const MINIFY: bool>(
    node_start: u32,
    codegen: &mut Codegen<{ MINIFY }>,
) -> Option<(u32, Comment)> {
    let maybe_leading_comment = codegen.try_get_leading_comment(node_start);
    let (comment_start, comment) = maybe_leading_comment?;
    let real_end = match comment.kind {
        CommentKind::SingleLine => comment.end,
        CommentKind::MultiLine => comment.end + 2,
    };
    let source_code = codegen.source_text;
    let content_between = &source_code[real_end as usize..node_start as usize];
    // Used for VariableDeclaration (Rollup only respects "const" and only for the first one)
    if content_between.chars().all(|ch| ch.is_ascii_whitespace()) {
        let comment_content = &source_code[*comment_start as usize..comment.end as usize];
        if MATCHER.find_iter(&comment_content).next().is_some() {
            return Some((*comment_start, *comment));
        }
        None
    } else {
        None
    }
}

pub fn print_comment<const MINIFY: bool>(
    comment_start: u32,
    comment: Comment,
    p: &mut Codegen<{ MINIFY }>,
) {
    match comment.kind {
        CommentKind::SingleLine => {
            p.print_str("//");
            p.print_range_of_source_code(comment_start as usize..comment.end as usize);
            p.print_soft_newline();
            p.print_indent();
        }
        CommentKind::MultiLine => {
            p.print_str("/*");
            p.print_range_of_source_code(comment_start as usize..comment.end as usize);
            p.print_str("*/");
            p.print_soft_space();
        }
    }
}

pub fn gen_comment<const MINIFY: bool>(node_start: u32, codegen: &mut Codegen<{ MINIFY }>) {
    if !codegen.options.preserve_annotate_comments {
        return;
    }
    if let Some((comment_start, comment)) = codegen.try_take_moved_comment(node_start) {
        print_comment::<MINIFY>(comment_start, comment, codegen);
    }
    let maybe_leading_annotate_comment = get_leading_annotate_comment(node_start, codegen);
    if let Some((comment_start, comment)) = maybe_leading_annotate_comment {
        print_comment::<MINIFY>(comment_start, comment, codegen);
    }
}
