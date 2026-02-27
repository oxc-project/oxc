use std::slice::Iter;

use itoa::Buffer as ItoaBuffer;

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
    pub span: Span,
}

impl ESTree for EstreeToken<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString(self.token_type.as_str()));
        state.serialize_field("value", &self.value);
        state.serialize_span(self.span);
        state.end();
    }
}

/// Token type for RegExps.
///
/// This is a separate type from `EstreeToken` because RegExp tokens have a nested `regex` object
/// containing `flags` and `pattern`, and the token type is always `"RegularExpression"`.
/// Pattern is taken from the AST node (`RegExpLiteral.regex.pattern.text`), and flags are sliced
/// from source text to preserve the original order (the AST stores flags as a bitfield which
/// would alphabetize them).
struct EstreeRegExpToken<'a> {
    value: &'a str,
    regex: RegExpData<'a>,
    span: Span,
}

/// The `regex` sub-object inside a `RegularExpression` token.
struct RegExpData<'a> {
    pattern: &'a str,
    flags: &'a str,
}

impl ESTree for EstreeRegExpToken<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("RegularExpression"));
        state.serialize_field("value", &self.value);
        state.serialize_field("regex", &self.regex);
        state.serialize_span(self.span);
        state.end();
    }
}

impl ESTree for RegExpData<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.pattern);
        // Flags are single ASCII letters (d, g, i, m, s, u, v, y) — always JSON-safe
        state.serialize_field("flags", &JsonSafeString(self.flags));
        state.end();
    }
}

/// Token type for identifiers (and keywords which are used as identifiers).
///
/// Despite the name, this type is also used for tokens with `"Keyword"` or `"PrivateIdentifier"`
/// type — it covers any token whose value is guaranteed JSON-safe.
///
/// This is a separate type from `EstreeToken` for two reasons:
/// 1. It has no `regex` field (identifiers never have regex data).
/// 2. Value is wrapped in `JsonSafeString` during serialization, skipping escape-checking.
///    Identifier names are guaranteed JSON-safe (no quotes, backslashes, or control characters).
struct EstreeIdentToken<'a> {
    token_type: TokenType,
    value: &'a str,
    span: Span,
}

impl ESTree for EstreeIdentToken<'_> {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        // Assemble the JSON string manually using `print_strs_array`.
        // This reduces bounds checks to only 1 per token.
        static TYPE_PREFIX: &str = "{\"type\":\"";
        static VALUE_PREFIX: &str = "\",\"value\":\"";
        static START_PREFIX: &str = "\",\"start\":";
        static END_PREFIX: &str = ",\"end\":";
        static POSTFIX: &str = "}";

        static TYPE_PREFIX_PRETTY: &str = "{\n    \"type\": \"";
        static VALUE_PREFIX_PRETTY: &str = "\",\n    \"value\": \"";
        static START_PREFIX_PRETTY: &str = "\",\n    \"start\": ";
        static END_PREFIX_PRETTY: &str = ",\n    \"end\": ";
        static POSTFIX_PRETTY: &str = "\n  }";

        let mut start_buffer = U32String::new();
        let start = start_buffer.format(self.span.start);

        let mut end_buffer = U32String::new();
        let end = end_buffer.format(self.span.end);

        serializer.buffer_mut().print_strs_array([
            if S::IS_COMPACT { TYPE_PREFIX } else { TYPE_PREFIX_PRETTY },
            self.token_type.as_str(),
            if S::IS_COMPACT { VALUE_PREFIX } else { VALUE_PREFIX_PRETTY },
            self.value,
            if S::IS_COMPACT { START_PREFIX } else { START_PREFIX_PRETTY },
            start,
            if S::IS_COMPACT { END_PREFIX } else { END_PREFIX_PRETTY },
            end,
            if S::IS_COMPACT { POSTFIX } else { POSTFIX_PRETTY },
        ]);
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

mod u32_string {
    use super::*;

    /// Maximum length of a decimal string representation of a `u32`.
    const MAX_U32_LEN: usize = "4294967295".len(); // `u32::MAX` (10 bytes string)

    /// Wrapper around [`ItoaBuffer`] which asserts that the formatted string is not longer than 10 bytes.
    ///
    /// Purpose of this type is to inform the compiler that the `&str` has a short length,
    /// which allows it to remove bounds checks when concatenating multiple strings.
    #[repr(transparent)]
    pub struct U32String(ItoaBuffer);

    #[expect(clippy::inline_always)]
    impl U32String {
        #[inline(always)]
        pub fn new() -> Self {
            Self(ItoaBuffer::new())
        }

        #[inline(always)]
        pub fn format(&mut self, n: u32) -> &str {
            let s = self.0.format(n);
            // SAFETY: A `u32` converted to decimal string cannot have more than 10 digits
            unsafe { assert_unchecked!(s.len() <= MAX_U32_LEN) };
            s
        }
    }
}
use u32_string::U32String;

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
    ///
    /// `name` is the decoded identifier name from the AST node.
    /// When the token has no escapes, `name` points into the source text, same as slicing it.
    /// When the token has escapes and `decode_identifier_escapes` is enabled, `name` provides
    /// the decoded value. Only when escapes are present but decoding is disabled do we need to
    /// fall back to slicing the raw source text (preserving the escape sequences in the output).
    fn emit_identifier(&mut self, start: u32, name: &str) {
        let token = self.advance_to(start);
        let token_type = if self.options.exclude_legacy_keyword_identifiers
            && matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static)
        {
            TokenType::new("Keyword")
        } else {
            TokenType::new("Identifier")
        };

        // Use `name` from AST node in most cases — it's JSON-safe so can skip escape checking.
        // Only fall back to raw source text when the token contains escapes and decoding is disabled,
        // since raw escape sequences contain `\` which needs JSON escaping.
        if self.options.decode_identifier_escapes || !token.escaped() {
            self.serialize_ident_token(token, token_type, name);
        } else {
            #[cold]
            #[inline(never)]
            fn emit<S: SequenceSerializer>(
                ctx: &mut EstreeTokenContext<'_, S>,
                token: &Token,
                token_type: TokenType,
            ) {
                ctx.emit_token(token, token_type);
            }
            emit(self, token, token_type);
        }
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

    /// Serialize a single token using its raw source text as the value.
    fn emit_token(&mut self, token: &Token, token_type: TokenType) {
        let value = &self.source_text[token.start() as usize..token.end() as usize];
        self.serialize_token(token, token_type, value);
    }

    /// Convert span to UTF-16 and serialize token.
    fn serialize_token(&mut self, token: &Token, token_type: TokenType, value: &str) {
        // Convert offsets to UTF-16
        let mut span = Span::new(token.start(), token.end());
        if let Some(converter) = self.span_converter.as_mut() {
            converter.convert_span(&mut span);
        }

        self.seq.serialize_element(&EstreeToken { token_type, value, span });
    }

    /// Serialize a token whose value is guaranteed JSON-safe, skipping escape-checking.
    fn serialize_ident_token(&mut self, token: &Token, token_type: TokenType, value: &str) {
        // Convert offsets to UTF-16
        let mut span = Span::new(token.start(), token.end());
        if let Some(converter) = self.span_converter.as_mut() {
            converter.convert_span(&mut span);
        }

        self.seq.serialize_element(&EstreeIdentToken { token_type, value, span });
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
            self.emit_identifier(identifier.span.start, &identifier.name);
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
            self.emit_identifier(identifier.span.start, &identifier.name);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.emit_identifier(identifier.span.start, &identifier.name);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.emit_identifier(identifier.span.start, &identifier.name);
    }

    fn visit_private_identifier(&mut self, identifier: &PrivateIdentifier<'a>) {
        let token = self.advance_to(identifier.span.start);

        // `identifier.name` has `#` stripped and escapes decoded by the parser, and is JSON-safe.
        // Only fall back to slicing raw source text when the token contains escapes and decoding
        // is disabled, since raw escape sequences contain `\` which needs JSON escaping.
        if self.options.decode_identifier_escapes || !token.escaped() {
            self.serialize_ident_token(
                token,
                TokenType::new("PrivateIdentifier"),
                &identifier.name,
            );
        } else {
            #[cold]
            #[inline(never)]
            fn emit<S: SequenceSerializer>(ctx: &mut EstreeTokenContext<'_, S>, token: &Token) {
                // Strip leading `#`
                let value = &ctx.source_text[token.start() as usize + 1..token.end() as usize];
                ctx.serialize_token(token, TokenType::new("PrivateIdentifier"), value);
            }
            emit(self, token);
        }
    }

    fn visit_reg_exp_literal(&mut self, regexp: &RegExpLiteral<'a>) {
        let token = self.advance_to(regexp.span.start);

        let value = regexp.raw.as_deref().unwrap();
        let pattern = regexp.regex.pattern.text.as_str();

        // Flags start after opening `/`, pattern, and closing `/`
        let flags = &value[pattern.len() + 2..];
        let regex = RegExpData { pattern, flags };

        // Convert offsets to UTF-16
        let mut span = Span::new(token.start(), token.end());
        if let Some(converter) = self.span_converter.as_mut() {
            converter.convert_span(&mut span);
        }

        self.seq.serialize_element(&EstreeRegExpToken { value, regex, span });
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
