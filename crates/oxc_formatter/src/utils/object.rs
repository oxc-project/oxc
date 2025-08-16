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
            s.span.source_text(f.source_text()),
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
            string.span.source_text(f.source_text()),
            string.span,
            false,
            StringLiteralParentKind::Member,
        )
        .clean_text(f);

        string.format_leading_comments(f)?;
        write!(f, format)?;
        string.format_trailing_comments(f)?;

        Ok(format.width())
    } else {
        write!(f, key)?;

        Ok(key.span().source_text(f.source_text()).width())
    }
}
