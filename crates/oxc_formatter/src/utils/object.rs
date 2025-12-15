use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::Formatter,
    utils::string::{
        FormatLiteralStringToken, StringLiteralParentKind, is_identifier_name_patched,
    },
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
        .clean_text(f);

        string.format_leading_comments(f);
        write!(f, format);
        string.format_trailing_comments(f);

        format.width()
    } else {
        write!(f, key);

        f.source_text().span_width(key.span())
    }
}

/// Determine if the property key string literal should preserve its quotes
pub fn should_preserve_quote(key: &PropertyKey<'_>, f: &Formatter<'_, '_>) -> bool {
    matches!(&key, PropertyKey::StringLiteral(string) if {
        let quote_less_content = f.source_text().text_for(&string.span.shrink(1));
        !is_identifier_name_patched(quote_less_content)
    })
}
