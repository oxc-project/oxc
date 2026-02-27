use std::slice::Iter;

use oxc_ast::ast::*;
use oxc_ast_visit::{
    Visit,
    utf8_to_utf16::{Utf8ToUtf16, Utf8ToUtf16Converter},
    walk,
};
use oxc_data_structures::assert_unchecked;
use oxc_estree::{
    CompactFormatter, Config, ESTree, ESTreeSerializer, JsonSafeString, PrettyFormatter,
    SequenceSerializer, Serializer, StructSerializer,
};
use oxc_parser::{Kind, Token};
use oxc_span::{GetSpan, Span};

/// Serializer config for tokens.
/// We never include ranges, so use this custom config which returns `false` for `ranges()`.
/// This allows compiler to remove the branch which checks if should print ranges from `serialize_span`.
struct TokenConfig;

impl Config for TokenConfig {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[expect(clippy::inline_always)] // It's a no-op
    #[inline(always)]
    fn new(_ranges: bool) -> Self {
        Self
    }

    // Never include ranges, so always return `false`.
    // `#[inline(always)]` to ensure compiler removes dead code resulting from the static value
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn ranges(&self) -> bool {
        false
    }
}

type CompactTokenSerializer = ESTreeSerializer<TokenConfig, CompactFormatter>;
type PrettyTokenSerializer = ESTreeSerializer<TokenConfig, PrettyFormatter>;

pub struct EstreeToken<'a> {
    pub token_type: TokenType,
    pub value: &'a str,
    pub regex: Option<EstreeRegExpToken<'a>>,
    pub span: Span,
}

pub struct EstreeRegExpToken<'a> {
    pub pattern: &'a str,
    pub flags: &'a str,
}

impl ESTree for EstreeToken<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString(self.token_type.as_str()));
        state.serialize_field("value", &self.value);
        if let Some(regex) = &self.regex {
            state.serialize_field("regex", regex);
        }
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for EstreeRegExpToken<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.pattern);
        state.serialize_field("flags", &self.flags);
        state.end();
    }
}

/// Token type.
/// Wrapped in a module to ensure `TokenType` can only be constructed via `TokenType::new`.
mod token_type {
    use super::*;

    const TOKEN_TYPE_MAX_LEN: usize = 17; // PrivateIdentifier, RegularExpression

    /// Token type.
    ///
    /// Just a wrapper around a `&'static str`.
    /// Purpose of this type is to inform the compiler that the `&str` has a short length,
    /// which allows it to remove bounds checks when concatenating multiple strings.
    #[derive(Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct TokenType(&'static str);

    #[expect(clippy::inline_always)]
    impl TokenType {
        #[inline(always)]
        pub const fn new(name: &'static str) -> Self {
            assert!(name.len() <= TOKEN_TYPE_MAX_LEN);
            Self(name)
        }

        #[inline(always)]
        pub const fn as_str(&self) -> &'static str {
            let s = self.0;
            // SAFETY: `TokenType` can only be constructed via `TokenType::new`,
            // which ensures `s.len() <= TOKEN_TYPE_MAX_LEN`
            unsafe { assert_unchecked!(s.len() <= TOKEN_TYPE_MAX_LEN) };
            s
        }
    }
}
use token_type::TokenType;

#[derive(Debug, Clone, Copy)]
pub struct EstreeTokenOptions {
    pub exclude_legacy_keyword_identifiers: bool,
    pub decode_identifier_escapes: bool,
    pub jsx_namespace_jsx_identifiers: bool,
    pub member_expr_in_jsx_expression_jsx_identifiers: bool,
}

impl EstreeTokenOptions {
    pub const fn test262() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: true,
            decode_identifier_escapes: true,
            jsx_namespace_jsx_identifiers: true,
            member_expr_in_jsx_expression_jsx_identifiers: false,
        }
    }

    pub const fn typescript() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: false,
            decode_identifier_escapes: false,
            jsx_namespace_jsx_identifiers: false,
            member_expr_in_jsx_expression_jsx_identifiers: true,
        }
    }

    pub const fn linter() -> Self {
        Self {
            exclude_legacy_keyword_identifiers: true,
            decode_identifier_escapes: false,
            jsx_namespace_jsx_identifiers: true,
            member_expr_in_jsx_expression_jsx_identifiers: false,
        }
    }
}

/// Serialize tokens to JSON.
///
/// `program` must have unconverted UTF-8 byte offset spans (as returned by the parser).
/// Token span conversion to UTF-16 is handled internally.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. BOM must be present on start of `source_text`, if the file has a BOM.
pub fn to_estree_tokens_json(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
) -> String {
    // Estimated size of a single token serialized to JSON, in bytes.
    // TODO: Estimate this better based on real-world usage.
    const BYTES_PER_TOKEN: usize = 64;

    let mut serializer =
        CompactTokenSerializer::with_capacity(tokens.len() * BYTES_PER_TOKEN, false);
    serialize_tokens(&mut serializer, tokens, program, source_text, span_converter, options);
    serializer.into_string()
}

/// Serialize tokens to pretty-printed JSON.
///
/// `program` must have unconverted UTF-8 byte offset spans (as returned by the parser).
/// Token span conversion to UTF-16 is handled internally.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. BOM must be present on start of `source_text`, if the file has a BOM.
pub fn to_estree_tokens_pretty_json(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
) -> String {
    // Estimated size of a single token serialized to JSON, in bytes.
    // TODO: Estimate this better based on real-world usage.
    const BYTES_PER_TOKEN: usize = 64;

    let mut serializer =
        PrettyTokenSerializer::with_capacity(tokens.len() * BYTES_PER_TOKEN, false);
    serialize_tokens(&mut serializer, tokens, program, source_text, span_converter, options);
    serializer.into_string()
}

/// Walk AST and serialize each token to the serializer as it's encountered.
///
/// Tokens are consumed from the `tokens` slice in source order. When a visitor method
/// encounters an AST node that requires a token type override (e.g. a keyword used as an
/// identifier), it serializes all preceding tokens with their default types, then serializes
/// the overridden token with its corrected type.
fn serialize_tokens(
    serializer: impl Serializer,
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: EstreeTokenOptions,
) {
    let mut context = EstreeTokenContext {
        seq: serializer.serialize_sequence(),
        tokens: tokens.iter(),
        source_text,
        span_converter: span_converter.converter(),
        options,
        jsx_expression_depth: 0,
        jsx_member_expression_depth: 0,
        jsx_computed_member_depth: 0,
    };
    context.visit_program(program);
    context.finish();
}

/// Visitor that walks the AST and serializes tokens as it encounters them.
///
/// Tokens are consumed from a slice in source order. When a visitor method encounters
/// an AST node that requires a token type override, all preceding tokens are serialized
/// with their default types, then the overridden token is serialized with its corrected type.
/// After the AST walk, any remaining tokens are serialized with default types.
struct EstreeTokenContext<'b, S: SequenceSerializer> {
    /// JSON sequence serializer.
    /// Tokens are serialized into this serializer.
    seq: S,
    /// Token iterator
    tokens: Iter<'b, Token>,
    /// Source text (for extracting token values)
    source_text: &'b str,
    /// Span converter for UTF-8 to UTF-16 conversion.
    /// `None` if source is ASCII-only.
    span_converter: Option<Utf8ToUtf16Converter<'b>>,
    /// Options
    options: EstreeTokenOptions,
    // State
    jsx_expression_depth: usize,
    jsx_member_expression_depth: usize,
    jsx_computed_member_depth: usize,
}

impl<'b, S: SequenceSerializer> EstreeTokenContext<'b, S> {
    /// Serialize all tokens before `start` with default types,
    /// then serialize the token at `start` with the given `token_type`.
    fn emit_token_at(&mut self, start: u32, token_type: TokenType) {
        let token = self.advance_to(start);
        self.emit_token(token, token_type);
    }

    /// Emit the token at `start` as `"Identifier"`, unless it's a legacy keyword
    /// and `exclude_legacy_keyword_identifiers` is set (in which case it gets its default type).
    fn emit_identifier(&mut self, start: u32) {
        let token = self.advance_to(start);
        let token_type = if self.options.exclude_legacy_keyword_identifiers
            && matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static)
        {
            TokenType::new("Keyword")
        } else {
            TokenType::new("Identifier")
        };
        self.emit_token(token, token_type);
    }

    /// Consume all tokens before `start` (emitting them with default types),
    /// and return the token at `start`.
    fn advance_to(&mut self, start: u32) -> &'b Token {
        while let Some(token) = self.tokens.next() {
            if token.start() < start {
                self.emit_token(token, get_token_type(token.kind()));
            } else {
                debug_assert_eq!(
                    token.start(),
                    start,
                    "Expected token at position {start}, found token at position {}",
                    token.start(),
                );
                return token;
            }
        }
        unreachable!("Expected token at position {start}");
    }

    /// Serialize a single token.
    fn emit_token(&mut self, token: &Token, token_type: TokenType) {
        let kind = token.kind();
        let start = token.start();
        let end = token.end();
        let value = &self.source_text[start as usize..end as usize];
        let value = if kind == Kind::PrivateIdentifier { &value[1..] } else { value };

        let decoded;
        let value = if self.options.decode_identifier_escapes
            && token.escaped()
            && (kind.is_identifier_name() || kind == Kind::PrivateIdentifier)
            && value.contains("\\u")
        {
            decoded = decode_js_unicode_escapes(value);
            decoded.as_str()
        } else {
            value
        };

        let regex = if kind == Kind::RegExp {
            regex_parts(value).map(|(pattern, flags)| EstreeRegExpToken { pattern, flags })
        } else {
            None
        };

        // Convert offsets to UTF-16
        let mut span = Span::new(start, end);
        if let Some(converter) = self.span_converter.as_mut() {
            converter.convert_span(&mut span);
        }

        let estree_token = EstreeToken { token_type, value, regex, span };
        self.seq.serialize_element(&estree_token);
    }

    /// Serialize all remaining tokens and close the sequence.
    fn finish(mut self) {
        while let Some(token) = self.tokens.next() {
            self.emit_token(token, get_token_type(token.kind()));
        }
        self.seq.end();
    }
}

impl<'a, S: SequenceSerializer> Visit<'a> for EstreeTokenContext<'_, S> {
    fn visit_ts_type_query(&mut self, type_query: &TSTypeQuery<'a>) {
        fn collect_type_query_this<S: SequenceSerializer>(
            ctx: &mut EstreeTokenContext<'_, S>,
            type_name: &TSTypeName<'_>,
        ) {
            match type_name {
                TSTypeName::ThisExpression(this_expression) => {
                    ctx.emit_token_at(this_expression.span.start, TokenType::new("Identifier"));
                }
                TSTypeName::QualifiedName(qualified_name) => {
                    collect_type_query_this(ctx, &qualified_name.left);
                }
                TSTypeName::IdentifierReference(_) => {}
            }
        }

        match &type_query.expr_name {
            TSTypeQueryExprName::ThisExpression(this_expression) => {
                self.emit_token_at(this_expression.span.start, TokenType::new("Identifier"));
            }
            TSTypeQueryExprName::QualifiedName(qualified_name) => {
                collect_type_query_this(self, &qualified_name.left);
            }
            TSTypeQueryExprName::IdentifierReference(_) | TSTypeQueryExprName::TSImportType(_) => {}
        }

        walk::walk_ts_type_query(self, type_query);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        // Manual walk.
        // * `source` is a `StringLiteral` — can't contain identifiers, so we skip visiting it.
        // * `options` is an `ObjectExpression`. Manually walk each property, but don't visit the key if it's `with`,
        //   as it needs to remain "Keyword" token, not get converted to "Identifier".
        // * `qualifier` and `type_arguments` are visited as usual.
        if let Some(options) = &import_type.options {
            for property in &options.properties {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        let is_with_key = matches!(
                            &property.key,
                            PropertyKey::StaticIdentifier(id) if id.name == "with"
                        );
                        if !is_with_key {
                            self.visit_property_key(&property.key);
                        }
                        self.visit_expression(&property.value);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        self.visit_spread_element(spread);
                    }
                }
            }
        }
        if let Some(qualifier) = &import_type.qualifier {
            self.visit_ts_import_type_qualifier(qualifier);
        }
        if let Some(type_arguments) = &import_type.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    fn visit_identifier_name(&mut self, identifier: &IdentifierName<'a>) {
        // `JSXIdentifier` takes priority over `Identifier` in `token_type`,
        // so `"Identifier"` is redundant when `"JSXIdentifier"` is set.
        if self.options.member_expr_in_jsx_expression_jsx_identifiers
            && self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.emit_token_at(identifier.span.start, TokenType::new("JSXIdentifier"));
        } else {
            self.emit_identifier(identifier.span.start);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.options.member_expr_in_jsx_expression_jsx_identifiers
            && self.jsx_expression_depth > 0
            && self.jsx_member_expression_depth > 0
            && self.jsx_computed_member_depth == 0
        {
            self.emit_token_at(identifier.span.start, TokenType::new("JSXIdentifier"));
        } else {
            self.emit_identifier(identifier.span.start);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.emit_identifier(identifier.span.start);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.emit_identifier(identifier.span.start);
    }

    fn visit_ts_this_parameter(&mut self, parameter: &TSThisParameter<'a>) {
        self.emit_token_at(parameter.this_span.start, TokenType::new("Identifier"));
        walk::walk_ts_this_parameter(self, parameter);
    }

    fn visit_meta_property(&mut self, _meta_property: &MetaProperty<'a>) {
        // Don't walk.
        // * `meta` (either `import` or `new`) has a "Keyword" token already, which is correct.
        // * `property` (either `meta` or `target`) has an "Identifier" token, which is correct.
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid double-flagging.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_expression(&property.value);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_binding_pattern(&property.value);
    }

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        // For `import { x }`, `imported` and `local` share the same span.
        // Only visit `imported` when it differs from `local`.
        if specifier.imported.span() != specifier.local.span {
            self.visit_module_export_name(&specifier.imported);
        }
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_export_specifier(&mut self, specifier: &ExportSpecifier<'a>) {
        // For `export { x }`, `local` and `exported` share the same span.
        // Only visit `exported` when it differs from `local`.
        self.visit_module_export_name(&specifier.local);
        if specifier.exported.span() != specifier.local.span() {
            self.visit_module_export_name(&specifier.exported);
        }
    }

    fn visit_jsx_identifier(&mut self, identifier: &JSXIdentifier<'a>) {
        self.emit_token_at(identifier.span.start, TokenType::new("JSXIdentifier"));
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        if let JSXElementName::IdentifierReference(identifier) = name {
            self.emit_token_at(identifier.span.start, TokenType::new("JSXIdentifier"));
        } else {
            walk::walk_jsx_element_name(self, name);
        }
    }

    fn visit_jsx_member_expression_object(&mut self, object: &JSXMemberExpressionObject<'a>) {
        if let JSXMemberExpressionObject::IdentifierReference(identifier) = object {
            self.emit_token_at(identifier.span.start, TokenType::new("JSXIdentifier"));
        } else {
            walk::walk_jsx_member_expression_object(self, object);
        }
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        if self.options.jsx_namespace_jsx_identifiers {
            self.emit_token_at(name.namespace.span.start, TokenType::new("JSXIdentifier"));
            self.emit_token_at(name.name.span.start, TokenType::new("JSXIdentifier"));
        }
        // When `!jsx_namespace_jsx_identifiers`, these tokens retain their default type
    }

    fn visit_jsx_expression_container(&mut self, container: &JSXExpressionContainer<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_expression_container(self, container);
        self.jsx_expression_depth -= 1;
    }

    fn visit_member_expression(&mut self, expression: &MemberExpression<'a>) {
        if self.jsx_expression_depth > 0 {
            self.jsx_member_expression_depth += 1;
            if matches!(expression, MemberExpression::ComputedMemberExpression(_)) {
                self.jsx_computed_member_depth += 1;
            }
            walk::walk_member_expression(self, expression);
            if matches!(expression, MemberExpression::ComputedMemberExpression(_)) {
                self.jsx_computed_member_depth -= 1;
            }
            self.jsx_member_expression_depth -= 1;
        } else {
            walk::walk_member_expression(self, expression);
        }
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_spread_attribute(self, attribute);
        self.jsx_expression_depth -= 1;
    }

    fn visit_jsx_spread_child(&mut self, spread_child: &JSXSpreadChild<'a>) {
        self.jsx_expression_depth += 1;
        walk::walk_jsx_spread_child(self, spread_child);
        self.jsx_expression_depth -= 1;
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        // Manual walk.
        // * `name`: Visit normally.
        // * `value`: Set `JSXText` token type if is a `StringLiteral`.
        self.visit_jsx_attribute_name(&attribute.name);
        match &attribute.value {
            Some(JSXAttributeValue::StringLiteral(string_literal)) => {
                self.emit_token_at(string_literal.span.start, TokenType::new("JSXText"));
            }
            Some(value) => self.visit_jsx_attribute_value(value),
            None => {}
        }
    }
}

fn get_token_type(kind: Kind) -> TokenType {
    match kind {
        Kind::Ident | Kind::Await => TokenType::new("Identifier"),
        Kind::PrivateIdentifier => TokenType::new("PrivateIdentifier"),
        Kind::JSXText => TokenType::new("JSXText"),
        Kind::Str => TokenType::new("String"),
        Kind::RegExp => TokenType::new("RegularExpression"),
        Kind::NoSubstitutionTemplate
        | Kind::TemplateHead
        | Kind::TemplateMiddle
        | Kind::TemplateTail => TokenType::new("Template"),
        Kind::True | Kind::False => TokenType::new("Boolean"),
        Kind::Null => TokenType::new("Null"),
        _ if kind.is_number() => TokenType::new("Numeric"),
        _ if kind.is_contextual_keyword() => TokenType::new("Identifier"),
        _ if kind.is_any_keyword() => TokenType::new("Keyword"),
        _ => TokenType::new("Punctuator"),
    }
}

fn regex_parts(raw: &str) -> Option<(&str, &str)> {
    let bytes = raw.as_bytes();
    if bytes.first() != Some(&b'/') {
        return None;
    }

    let mut escaped = false;
    let mut in_character_class = false;
    for index in 1..bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'[' if !in_character_class => in_character_class = true,
            b']' if in_character_class => in_character_class = false,
            b'/' if !in_character_class => return Some((&raw[1..index], &raw[index + 1..])),
            _ => {}
        }
    }

    None
}

fn decode_js_unicode_escapes(raw: &str) -> String {
    debug_assert!(raw.contains("\\u"));

    let bytes = raw.as_bytes();
    let mut output = String::with_capacity(raw.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'u' {
            // `\u{...}`
            if i + 2 < bytes.len() && bytes[i + 2] == b'{' {
                let mut j = i + 3;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                if j < bytes.len()
                    && j > i + 3
                    && let Ok(codepoint) = u32::from_str_radix(&raw[i + 3..j], 16)
                    && let Some(ch) = char::from_u32(codepoint)
                {
                    output.push(ch);
                    i = j + 1;
                    continue;
                }
            // `\uXXXX`
            } else if i + 6 <= bytes.len()
                && let Ok(codepoint) = u32::from_str_radix(&raw[i + 2..i + 6], 16)
                && let Some(ch) = char::from_u32(codepoint)
            {
                output.push(ch);
                i += 6;
                continue;
            }
        }

        let ch = raw[i..].chars().next().unwrap();
        output.push(ch);
        i += ch.len_utf8();
    }

    output
}
