use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::Formatter,
    utils::string::{FormatLiteralStringToken, StringLiteralParentKind},
    write,
};

pub fn format_property_key<'a>(key: &AstNode<'a, PropertyKey<'a>>, f: &mut Formatter<'_, 'a>) {
    if let PropertyKey::StringLiteral(s) = key.as_ref() {
        // `"constructor"` property in the class should be kept quoted
        let kind = if matches!(key.parent, AstNodes::PropertyDefinition(_))
            && matches!(key.as_ref(), PropertyKey::StringLiteral(string) if string.value == "constructor")
        {
            StringLiteralParentKind::Expression
        } else {
            StringLiteralParentKind::Member
        };

        FormatLiteralStringToken::new(
            f.source_text().text_for(s.as_ref()),
            /* jsx */
            false,
            kind,
        )
        .fmt(f);
    } else {
        write!(f, key);
    }
}

pub fn write_member_name<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> usize {
    if let AstNodes::StringLiteral(string) = key.as_ast_nodes() {
        let format = FormatLiteralStringToken::new(
            f.source_text().text_for(string),
            false,
            StringLiteralParentKind::Member,
        )
        .clean_text(f.context().source_type(), f.options());

        string.format_leading_comments(f);
        write!(f, format);
        string.format_trailing_comments(f);

        format.width()
    } else {
        write!(f, key);

        f.source_text().span_width(key.span())
    }
}
