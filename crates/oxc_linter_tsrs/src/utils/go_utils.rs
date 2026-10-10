//! Ports of internal/utils/utils.go helpers, added as rules need them.

use super::*;
use tsrs_ast::CommentRange;
use tsrs_core::TextRange;

/// utils.go TypeRecurser: the predicate returns true to stop.
pub fn type_recurser(t: P<Type>, predicate: &mut dyn FnMut(P<Type>) -> bool) -> bool {
    if is_type_flag_set(t, TypeFlags::UnionOrIntersection) {
        for &subtype in t.types() {
            if type_recurser(subtype, predicate) {
                return true;
            }
        }
        false
    } else {
        predicate(t)
    }
}

/// utils.go GetHeritageClauses.
pub fn get_heritage_clauses(node: P<Node>) -> Option<P<ast::NodeList>> {
    match node.kind() {
        Kind::ClassDeclaration => node.as_class_declaration().heritage_clauses(),
        Kind::ClassExpression => node.as_class_expression().heritage_clauses(),
        Kind::InterfaceDeclaration => node.as_interface_declaration().heritage_clauses(),
        _ => None,
    }
}

/// utils.go GetNumberIndexType.
pub fn get_number_index_type(c: &mut Checker, t: P<Type>) -> Option<P<Type>> {
    let number_type = c.number_type;
    c.get_index_type_of_type(t, number_type)
}

/// utils.go GetCommentsInRange: trailing then leading comment ranges starting at in_range.pos, stopping at
/// in_range.end.
pub fn get_comments_in_range(source_file: P<SourceFile>, in_range: TextRange) -> Vec<CommentRange> {
    let text = source_file.text();
    let mut out = Vec::new();
    for comment_range in tsrs_scanner::get_trailing_comment_ranges(text, in_range.pos()) {
        if comment_range.pos() >= in_range.end() {
            break;
        }
        out.push(comment_range);
    }
    for comment_range in tsrs_scanner::get_leading_comment_ranges(text, in_range.pos()) {
        if comment_range.pos() >= in_range.end() {
            break;
        }
        out.push(comment_range);
    }
    out
}

/// utils.go HasCommentsInRange.
pub fn has_comments_in_range(source_file: P<SourceFile>, in_range: TextRange) -> bool {
    !get_comments_in_range(source_file, in_range).is_empty()
}

/// utils.go IsStrWhiteSpace (jsnum's StrWhiteSpaceChar: LineTerminator, WhiteSpace, Unicode Zs).
pub fn is_str_white_space(r: char) -> bool {
    matches!(
        r,
        '\n' | '\r'
            | '\u{2028}'
            | '\u{2029}'
            | '\t'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{FEFF}'
            // Unicode category Zs
            | ' '
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// utils.go IsStringWhiteSpace.
pub fn is_string_white_space(s: &str) -> bool {
    s.chars().all(is_str_white_space)
}
