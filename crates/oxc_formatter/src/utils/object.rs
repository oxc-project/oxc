use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Buffer, Format,
    ast_nodes::{AstNode, AstNodes},
    formatter::Formatter,
    utils::{
        string::{FormatLiteralStringToken, StringLiteralParentKind, is_identifier_name_patched},
        tailwindcss::{tailwind_context_for_string_literal, write_tailwind_string_literal},
    },
    write,
};

pub fn format_property_key<'a>(key: &AstNode<'a, PropertyKey<'a>>, f: &mut Formatter<'_, 'a>) {
    // Check if we're in a Tailwind context and the key is a string literal with multiple classes
    if let AstNodes::StringLiteral(string) = key.as_ast_nodes() {
        if let Some(ctx) = tailwind_context_for_string_literal(string, f) {
            // Reuse the existing Tailwind string literal writer
            write_tailwind_string_literal(string, ctx, f);
            return;
        }

        // For TypeScript class property declarations, quotes should always be preserved.
        // https://github.com/prettier/prettier/issues/4516
        let kind = if matches!(key.parent(), AstNodes::PropertyDefinition(_))
            && f.context().source_type().is_typescript()
        {
            StringLiteralParentKind::Expression
        } else {
            StringLiteralParentKind::Member
        };

        FormatLiteralStringToken::new(f.source_text().text_for(string), /* jsx */ false, kind)
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
        if let Some(ctx) = tailwind_context_for_string_literal(string, f) {
            // Reuse the existing Tailwind string literal writer
            string.format_leading_comments(f);
            write_tailwind_string_literal(string, ctx, f);
            string.format_trailing_comments(f);

            // Compute the normalized width based on the same cleaned string token
            FormatLiteralStringToken::new(
                f.source_text().text_for(string),
                false,
                StringLiteralParentKind::Member,
            )
            .clean_text(f)
            .width()
        } else {
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
        }
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
