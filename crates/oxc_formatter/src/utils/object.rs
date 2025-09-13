use oxc_ast::ast::*;
use oxc_span::GetSpan;
use unicode_width::UnicodeWidthStr;

use crate::{
    Buffer, Format, FormatResult,
    formatter::Formatter,
    generated::ast_nodes::{AstNode, AstNodes},
    utils::string_utils::{FormatLiteralStringToken, StringLiteralParentKind},
    write,
};

pub fn format_property_key<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    if let PropertyKey::StringLiteral(s) = key.as_ref() {
        FormatLiteralStringToken::new(
            f.source_text().text_for(s.as_ref()),
            s.span,
            /* jsx */
            false,
            StringLiteralParentKind::Member,
        )
        .fmt(f)
    } else {
        write!(f, key)
    }
}

pub fn write_member_name<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<usize> {
    if let AstNodes::StringLiteral(string) = key.as_ast_nodes() {
        let format = FormatLiteralStringToken::new(
            f.source_text().text_for(string),
            string.span,
            false,
            StringLiteralParentKind::Member,
        )
        .clean_text(f.context().source_type(), f.options());

        string.format_leading_comments(f)?;
        write!(f, format)?;
        string.format_trailing_comments(f)?;

        Ok(format.width())
    } else {
        write!(f, key)?;

        Ok(f.source_text().span_width(key.span()))
    }
}
