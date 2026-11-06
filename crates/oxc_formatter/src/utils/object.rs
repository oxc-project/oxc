use unicode_width::UnicodeWidthStr;

use oxc_ast::ast::*;
use oxc_syntax::identifier::is_identifier_name_patched;

use crate::{
    Format,
    ast_nodes::{AstNode, AstNodeKind, AstNodes},
    formatter::{JsFormatter, JsFormatterExt as _, TailwindContextEntry},
    utils::{
        string::{FormatLiteralStringToken, StringLiteralParentKind},
        tailwindcss::{
            tailwind_context_for_class_list, tailwind_single_class_key,
            write_tailwind_string_literal,
        },
    },
    write,
};

pub fn format_property_key<'a>(
    key: &AstNode<'_, 'a, PropertyKey<'a>>,
    computed: bool,
    f: &mut JsFormatter<'_, 'a>,
) {
    if computed {
        write!(f, ["[", key, "]"]);
        return;
    }

    if let AstNodeKind::StringLiteral(string) = &key.as_kind() {
        let text = match string_key_text(string, f) {
            Ok(text) => text,
            Err(ctx) => {
                write_tailwind_string_literal(string, ctx, f);
                return;
            }
        };

        let preserve_quotes = match key.parent() {
            // For TypeScript class property declarations, quotes should always be preserved.
            // https://github.com/prettier/prettier/issues/4516
            AstNodes::PropertyDefinition(_) => f.context().source_type().is_typescript(),
            AstNodes::TSMethodSignature(method) => is_quoted_new_method_signature(method),
            _ => false,
        };
        let kind = if preserve_quotes {
            StringLiteralParentKind::Expression
        } else {
            StringLiteralParentKind::Member
        };

        FormatLiteralStringToken::new(text, /* jsx */ false, kind).fmt(f);
    } else {
        write!(f, key);
    }
}

pub fn write_member_name<'a>(
    key: &AstNode<'_, 'a, PropertyKey<'a>>,
    f: &mut JsFormatter<'_, 'a>,
) -> usize {
    if let AstNodeKind::StringLiteral(string) = &key.as_kind() {
        string.format_leading_comments(f);
        let width = match string_key_text(string, f) {
            Ok(text) => {
                let format =
                    FormatLiteralStringToken::new(text, false, StringLiteralParentKind::Member)
                        .clean_text(f);
                write!(f, format);
                format.width()
            }
            Err(ctx) => write_tailwind_string_literal(string, ctx, f),
        };
        string.format_trailing_comments(f);
        width
    } else {
        write!(f, key);

        f.source_text().text_for(key).width()
    }
}

/// Returns the text a string key is printed from as a plain key, quotes included,
/// or the Tailwind context to sort it in.
/// - A key printed as a single class is a plain key with it (see [`tailwind_single_class_key`])
fn string_key_text<'a>(
    string: &StringLiteral<'_>,
    f: &JsFormatter<'_, 'a>,
) -> Result<&'a str, TailwindContextEntry> {
    let text = f.source_text().text_for(string);
    let Some(ctx) = tailwind_context_for_class_list(string, f) else {
        return Ok(text);
    };
    let class = tailwind_single_class_key(string, ctx, f).ok_or(ctx)?;
    let quote = &text[..1];
    Ok(f.allocator().alloc_concat_strs_array([quote, class, quote]))
}

/// Determine if this is a method signature named `"new"` via a quoted key, which must keep its quotes.
/// Unquoting `new(...)` in an interface or type literal would turn it into a construct signature.
/// Optional (`new?()`) and getter/setter forms would stay a member named `new` even unquoted,
/// but Prettier keeps the quotes for every method-signature form; only the property form is unquoted.
pub fn is_quoted_new_method_signature(method: &TSMethodSignature<'_>) -> bool {
    // Trimmed, since Tailwind sorting can print `" new "` as `"new"`
    !method.computed
        && matches!(&method.key, PropertyKey::StringLiteral(string)
            if string.value.as_str().is_some_and(|value| value.trim() == "new"))
}

/// Determine if the string literal key should preserve its quotes,
/// i.e. its content cannot be written as a plain identifier.
pub fn should_preserve_string_quote(string: &StringLiteral<'_>, f: &JsFormatter<'_, '_>) -> bool {
    let quote_less_content = f.source_text().text_for(&string.span.shrink(1));
    !is_identifier_name_patched(quote_less_content)
}

/// Determine if the property key string literal should preserve its quotes,
/// checked on the text it's printed from (see [`string_key_text`]).
/// Computed keys are ignored.
pub fn should_preserve_quote(
    key: &PropertyKey<'_>,
    computed: bool,
    f: &JsFormatter<'_, '_>,
) -> bool {
    // Sorted as Tailwind classes (`Err`) keeps the quotes
    !computed
        && matches!(&key, PropertyKey::StringLiteral(string)
            if !string_key_text(string, f)
                .is_ok_and(|text| is_identifier_name_patched(&text[1..text.len() - 1])))
}

/// Determine if the enum member name string literal should preserve its quotes.
/// Enum member names behave like object property keys for `quoteProps`.
pub fn should_preserve_quote_for_enum_member(
    id: &TSEnumMemberName<'_>,
    f: &JsFormatter<'_, '_>,
) -> bool {
    matches!(id, TSEnumMemberName::String(string) | TSEnumMemberName::ComputedString(string)
        if should_preserve_string_quote(string, f))
}
