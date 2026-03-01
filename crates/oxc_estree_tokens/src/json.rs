//! Functions and data structures to serialize tokens to JSON.

use std::slice::Iter;

use oxc_ast::ast::{
    JSXText, PrivateIdentifier, Program, RegExpLiteral, StringLiteral, TemplateElement,
};
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_ast_visit::{Visit, utf8_to_utf16::Utf8ToUtf16Converter};
use oxc_estree::{
    CompactFormatter, Config as ESTreeConfig, ESTree, ESTreeSerializer, JsonSafeString,
    PrettyFormatter, SequenceSerializer, Serializer, StructSerializer,
};
use oxc_parser::{Kind, Token};
use oxc_span::Span;

use crate::{
    context::Context, options::ESTreeTokenConfig, token_type::TokenType, u32_string::U32String,
    visitor::Visitor,
};

/// Serializer config for tokens.
/// We never include ranges, so use this custom config which returns `false` for `ranges()`.
/// This allows compiler to remove the branch which checks whether to print ranges in `serialize_span`.
struct TokenSerializerConfig;

impl ESTreeConfig for TokenSerializerConfig {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[expect(clippy::inline_always)] // It's a no-op
    #[inline(always)]
    fn new(_ranges: bool) -> Self {
        Self
    }

    // Never include ranges, so always return `false`.
    // `#[inline(always)]` to ensure compiler removes dead code resulting from the static value.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn ranges(&self) -> bool {
        false
    }
}

/// Serializer for tokens to compact JSON.
type CompactTokenSerializer = ESTreeSerializer<TokenSerializerConfig, CompactFormatter>;

/// Serializer for tokens to pretty-printed JSON.
type PrettyTokenSerializer = ESTreeSerializer<TokenSerializerConfig, PrettyFormatter>;

/// Serialize tokens to JSON.
///
/// `program` must have unconverted UTF-8 byte offset spans (as returned by the parser).
/// Token span conversion to UTF-16 is handled internally.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. if the file has a BOM, it must be present at the start of `source_text`.
pub fn to_estree_tokens_json<O: ESTreeTokenConfig>(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: O,
) -> String {
    let capacity = estimate_json_len(tokens.len(), source_text.len(), true);
    let mut serializer = CompactTokenSerializer::with_capacity(capacity, false);
    serialize_tokens(&mut serializer, tokens, program, source_text, span_converter, options);
    serializer.into_string()
}

/// Serialize tokens to pretty-printed JSON.
///
/// `program` must have unconverted UTF-8 byte offset spans (as returned by the parser).
/// Token span conversion to UTF-16 is handled internally.
///
/// `source_text` must be the original source text, prior to BOM removal.
/// i.e. if the file has a BOM, it must be present at the start of `source_text`.
pub fn to_estree_tokens_pretty_json<O: ESTreeTokenConfig>(
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: O,
) -> String {
    let capacity = estimate_json_len(tokens.len(), source_text.len(), false);
    let mut serializer = PrettyTokenSerializer::with_capacity(capacity, false);
    serialize_tokens(&mut serializer, tokens, program, source_text, span_converter, options);
    serializer.into_string()
}

/// Estimate size of tokens serialized to JSON, in bytes.
/// Aim is to allocate capacity which is a reasonable over-estimate for the size of all tokens serialized to JSON,
/// in order to ensure the serializer's buffer never has to grow during serialization.
///
/// Combine the following:
///
/// * Basic JSON structure for a token x number of tokens.
/// * Max length of token type x number of tokens.
///   This is an over-estimate because not all tokens have the longest type (`PrivateIdentifier`).
/// * Length of source text.
///   Tokens can at most include all of the source text in their `value` fields (tokens cannot overlap).
///   In a minified file, this is usually a slight under-estimate, as some `value` fields will need escaping in JSON.
///   In a non-minified file, there'll be whitespace and comments between tokens, so it's likely an over-estimate.
/// * Max offset length x number of tokens x 2.
///   Each token includes `start` and `end` fields, which cannot be larger than the length of the source text.
///   This is a bit of an over-estimate, as earlier tokens will have smaller offsets, but it's in right ballpark.
/// * 2 bytes for leading/trailing `[` and `]`. This is purely to get the right length for empty source text.
///
/// Regex tokens (which are longer) are ignored in this calculation, on assumption they're relatively rare.
///
/// There are 2 factors which are under-estimates in this calculation, but overall it's likely to be
/// a decent over-estimate, due to the over-estimate on length of token type.
fn estimate_json_len(tokens_len: usize, source_text_len: usize, is_compact: bool) -> usize {
    const TYPE_LEN: usize = "PrivateIdentifier".len();

    const COMPACT_JSON_STRUCTURE_LEN: usize =
        r#"{"type":"","value":"","start":,"end":},"#.len() + TYPE_LEN;
    const PRETTY_JSON_STRUCTURE_LEN: usize =
        "  {\n    \"type\": \"\",\n    \"value\": \"\",\n    \"start\": ,\n    \"end\": \n  },\n"
            .len()
            + TYPE_LEN;
    const COMPACT_JSON_HEADER_FOOTER_LEN: usize = "[]".len();
    const PRETTY_JSON_HEADER_FOOTER_LEN: usize = "[\n]".len();

    let (structure_len, header_footer_len) = if is_compact {
        (COMPACT_JSON_STRUCTURE_LEN, COMPACT_JSON_HEADER_FOOTER_LEN)
    } else {
        (PRETTY_JSON_STRUCTURE_LEN, PRETTY_JSON_HEADER_FOOTER_LEN)
    };
    let offset_len = source_text_len.checked_ilog10().unwrap_or(0) as usize + 1;
    let token_len = structure_len + offset_len * 2;
    token_len * tokens_len + source_text_len + header_footer_len
}

/// Serialize tokens to JSON using provided serializer.
///
/// Walk AST and serialize each token into the serializer as it's encountered.
/// Also convert token spans from UTF-8 byte offsets to UTF-16 offsets.
///
/// Tokens are consumed from the `tokens` slice in source order.
/// When a visitor method encounters an AST node that requires a token type override
/// (e.g. a keyword used as an identifier), it serializes all preceding tokens with their default types,
/// then serializes the overridden token with its corrected type.
fn serialize_tokens<O: ESTreeTokenConfig>(
    serializer: impl Serializer,
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: O,
) {
    let mut visitor = Visitor::new(JsonContext {
        seq: serializer.serialize_sequence(),
        tokens: tokens.iter(),
        source_text,
        span_converter: span_converter.converter(),
        options,
        jsx_state: O::JSXState::default(),
    });
    visitor.visit_program(program);
    visitor.into_ctx().finish();
}

/// JSON serialization context.
///
/// Serializes each token to JSON with its correct ESTree token type.
pub struct JsonContext<'b, O: ESTreeTokenConfig, S: SequenceSerializer> {
    /// JSON sequence serializer.
    /// Tokens are serialized into this serializer.
    seq: S,
    /// Tokens iterator (immutable - tokens are read, not modified)
    tokens: Iter<'b, Token>,
    /// Source text (for extracting token values)
    source_text: &'b str,
    /// Span converter for UTF-8 to UTF-16 conversion.
    /// `None` if source is ASCII-only.
    span_converter: Option<Utf8ToUtf16Converter<'b>>,
    /// Options controlling JS/TS style differences
    options: O,
    // JSX state. Used when outputting tokens in TS style.
    jsx_state: O::JSXState,
}

impl<'b, O: ESTreeTokenConfig, S: SequenceSerializer> JsonContext<'b, O, S> {
    /// Consume all tokens before `start` (emitting them with default types),
    /// and return the token at `start`.
    ///
    /// Tokens emitted here are guaranteed JSON-safe because all non-JSON-safe token types
    /// (strings, templates, regexes, JSXText) are dealt with by their own visitors.
    fn advance_to(&mut self, start: u32) -> &'b Token {
        while let Some(token) = self.tokens.next() {
            if token.start() < start {
                self.emit_default_token(token);
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

    /// Serialize a token with its default type (determined by its `Kind`).
    ///
    /// Token values serialized here are guaranteed JSON-safe
    /// (punctuators, keywords, numbers, booleans, `null`).
    fn emit_default_token(&mut self, token: &Token) {
        let kind = token.kind();

        // Tokens with these `Kind`s are always consumed by specific visitors and should never reach here
        debug_assert!(
            !matches!(
                kind,
                Kind::Str
                    | Kind::RegExp
                    | Kind::JSXText
                    | Kind::PrivateIdentifier
                    | Kind::NoSubstitutionTemplate
                    | Kind::TemplateHead
                    | Kind::TemplateMiddle
                    | Kind::TemplateTail
            ),
            "Token kind {kind:?} should be consumed by its visitor, and not reach `get_token_type`",
        );

        let token_type = match kind {
            Kind::Ident | Kind::Await => TokenType::new("Identifier"),
            Kind::True | Kind::False => TokenType::new("Boolean"),
            Kind::Null => TokenType::new("Null"),
            _ if kind.is_number() => TokenType::new("Numeric"),
            _ if kind.is_contextual_keyword() => TokenType::new("Identifier"),
            _ if kind.is_any_keyword() => TokenType::new("Keyword"),
            _ => TokenType::new("Punctuator"),
        };

        let value = &self.source_text[token.start() as usize..token.end() as usize];

        self.serialize_safe_token(token, token_type, value);
    }

    /// Advance to the token at `start` and serialize it using provided `value` without JSON encoding.
    fn emit_safe_token_at(&mut self, start: u32, token_type: TokenType, value: &str) {
        let token = self.advance_to(start);
        self.serialize_safe_token(token, token_type, value);
    }

    /// Advance to the token at `start` and serialize it using raw source text with JSON encoding.
    fn emit_unsafe_token_at(&mut self, start: u32, token_type: TokenType) {
        let token = self.advance_to(start);
        self.emit_unsafe_token(token, token_type);
    }

    /// Serialize a token using its raw source text, with JSON encoding.
    ///
    /// Used for tokens whose values may contain backslashes, quotes, or control characters
    /// (escaped identifiers, string literals, template literals, JSXText).
    fn emit_unsafe_token(&mut self, token: &Token, token_type: TokenType) {
        let value = &self.source_text[token.start() as usize..token.end() as usize];
        self.serialize_unsafe_token(token, token_type, value);
    }

    /// Serialize a token whose value is guaranteed JSON-safe, skipping JSON-encoding.
    fn serialize_safe_token(&mut self, token: &Token, token_type: TokenType, value: &str) {
        let span = self.get_utf16_span(token);
        self.seq.serialize_element(&ESTreeSafeToken { token_type, value, span });
    }

    /// Serialize a token whose value may not be JSON-safe.
    fn serialize_unsafe_token(&mut self, token: &Token, token_type: TokenType, value: &str) {
        let span = self.get_utf16_span(token);
        self.seq.serialize_element(&ESTreeUnsafeToken { token_type, value, span });
    }

    /// Get UTF-16 span for a token.
    fn get_utf16_span(&mut self, token: &Token) -> Span {
        let mut span = Span::new(token.start(), token.end());
        if let Some(converter) = self.span_converter.as_mut() {
            converter.convert_span(&mut span);
        }
        span
    }

    /// Serialize all remaining tokens and close the sequence.
    ///
    /// Tokens emitted here are guaranteed JSON-safe because all non-JSON-safe token types
    /// (escaped identifiers, strings, templates, regexes, JSXText) are consumed by their own visitors.
    fn finish(mut self) {
        while let Some(token) = self.tokens.next() {
            self.emit_default_token(token);
        }
        self.seq.end();
    }
}

impl<O: ESTreeTokenConfig, S: SequenceSerializer> Context for JsonContext<'_, O, S> {
    /// JSX state type for tracking when to emit JSX identifiers.
    /// Inherited from config.
    type JSXState = O::JSXState;

    /// Returns `true` if serializing in JS style.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_ts(&self) -> bool {
        self.options.is_ts()
    }

    /// Get reference to [`JSXState`] for the context.
    ///
    /// [`JSXState`]: crate::jsx_state::JSXState
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state(&self) -> &Self::JSXState {
        &self.jsx_state
    }

    /// Get mutable reference to [`JSXState`] for the context.
    ///
    /// [`JSXState`]: crate::jsx_state::JSXState
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state_mut(&mut self) -> &mut Self::JSXState {
        &mut self.jsx_state
    }

    /// Emit the token at `start` as `Identifier`, unless it's a legacy keyword and serializing in JS style
    /// (in which case it gets `Keyword` type).
    fn emit_identifier_at(&mut self, start: u32, name: &str) {
        let token = self.advance_to(start);

        let token_type =
            if self.is_js() && matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static) {
                TokenType::new("Keyword")
            } else {
                TokenType::new("Identifier")
            };

        // `name` is from AST, has escapes decoded by the parser, and is JSON-safe.
        // Use it in most cases — if token is not marked as escaped, it's JSON-safe, so can skip JSON encoding.
        // When `self.options.decode_identifier_escapes` is `true`, token `value` should *always* be
        // the unescaped version, so can also use `name` from AST node and skip JSON encoding.
        // Only fall back to raw source text when the token contains escapes *and* decoding is disabled,
        // since escape sequences contain `\` which needs JSON escaping.
        // Escaped identifiers are extremely rare, so handle them in `#[cold]` branch.
        if self.is_js() || !token.escaped() {
            self.serialize_safe_token(token, token_type, name);
        } else {
            #[cold]
            #[inline(never)]
            fn emit<O: ESTreeTokenConfig, S: SequenceSerializer>(
                ctx: &mut JsonContext<'_, O, S>,
                token: &Token,
                token_type: TokenType,
            ) {
                ctx.emit_unsafe_token(token, token_type);
            }
            emit(self, token, token_type);
        }
    }

    /// Emit the `this` keyword at `start` as `Identifier`.
    /// Used for `this` in TS type queries and TS `this` parameters.
    fn emit_this_identifier_at(&mut self, start: u32) {
        self.emit_safe_token_at(start, TokenType::new("Identifier"), "this");
    }

    /// Emit the token at `start` as `JSXIdentifier`.
    /// JSX identifier names are guaranteed JSON-safe (no unicode escapes, no special characters).
    fn emit_jsx_identifier_at(&mut self, start: u32, name: &str) {
        self.emit_safe_token_at(start, TokenType::new("JSXIdentifier"), name);
    }

    /// Emit a `PrivateIdentifier` token.
    fn emit_private_identifier(&mut self, ident: &PrivateIdentifier<'_>) {
        let token = self.advance_to(ident.span.start);

        // `ident.name` has `#` stripped and escapes decoded by the parser, and is JSON-safe.
        // Use it in most cases — if token is not marked as escaped, it's JSON-safe, so can skip JSON encoding.
        // When `self.is_js()` is `true`, token `value` should *always* be the unescaped version,
        // so can also use `name` from AST node and skip JSON encoding.
        // Only fall back to raw source text when the token contains escapes *and* decoding is disabled,
        // since escape sequences contain `\` which needs JSON escaping.
        // Escaped identifiers are extremely rare, so handle them in `#[cold]` branch.
        if self.is_js() || !token.escaped() {
            self.serialize_safe_token(token, TokenType::new("PrivateIdentifier"), &ident.name);
        } else {
            #[cold]
            #[inline(never)]
            fn emit<O: ESTreeTokenConfig, S: SequenceSerializer>(
                ctx: &mut JsonContext<'_, O, S>,
                token: &Token,
            ) {
                // Strip leading `#`
                let value = &ctx.source_text[token.start() as usize + 1..token.end() as usize];
                ctx.serialize_unsafe_token(token, TokenType::new("PrivateIdentifier"), value);
            }
            emit(self, token);
        }
    }

    /// Emit a `StringLiteral` token.
    fn emit_string_literal(&mut self, literal: &StringLiteral<'_>) {
        self.emit_unsafe_token_at(literal.span.start, TokenType::new("String"));
    }

    /// Emit a `StringLiteral` in a JSX attribute as `JSXText`.
    fn emit_string_literal_as_jsx_text(&mut self, literal: &StringLiteral<'_>) {
        self.emit_unsafe_token_at(literal.span.start, TokenType::new("JSXText"));
    }

    /// Emit a `JSXText` token.
    fn emit_jsx_text(&mut self, jsx_text: &JSXText<'_>) {
        self.emit_unsafe_token_at(jsx_text.span.start, TokenType::new("JSXText"));
    }

    /// Emit token for `RegExpLiteral`.
    fn emit_regexp(&mut self, regexp: &RegExpLiteral<'_>) {
        let token = self.advance_to(regexp.span.start);

        let value = regexp.raw.as_deref().unwrap();
        let pattern = regexp.regex.pattern.text.as_str();

        // Flags start after opening `/`, pattern, and closing `/`
        let flags = &value[pattern.len() + 2..];
        let regex = RegExpData { pattern, flags };

        let span = self.get_utf16_span(token);
        self.seq.serialize_element(&ESTreeRegExpToken { value, regex, span });
    }

    /// Emit template quasis interleaved with their interpolated parts (expressions or TS types).
    ///
    /// `TemplateElement.span` excludes delimiters (parser adjusts `start + 1`),
    /// so subtract 1 to get the token start position.
    fn walk_template_quasis_interleaved<I>(
        visitor: &mut Visitor<Self>,
        quasis: &[TemplateElement<'_>],
        mut visit_interpolation: impl FnMut(&mut Visitor<Self>, &I),
        interpolations: &[I],
    ) {
        // Quasis and interpolations must be walked in interleaved source order,
        // because `advance_to` consumes tokens sequentially.
        // The default `walk_template_literal` visits all quasis first, then all expressions,
        // which would break source-order token consumption.
        let mut quasis = quasis.iter();

        // First quasi (TemplateHead or NoSubstitutionTemplate).
        // `TemplateElement.span` excludes delimiters (parser adjusts `start + 1`),
        // so subtract 1 to get the token start position.
        if let Some(quasi) = quasis.next() {
            visitor
                .ctx_mut()
                .emit_unsafe_token_at(quasi.span.start - 1, TokenType::new("Template"));
        }

        // Remaining quasis interleaved with interpolations
        for (interpolation, quasi) in interpolations.iter().zip(quasis) {
            visit_interpolation(visitor, interpolation);
            visitor
                .ctx_mut()
                .emit_unsafe_token_at(quasi.span.start - 1, TokenType::new("Template"));
        }
    }
}

/// Token whose `value` is guaranteed JSON-safe.
///
/// Used for identifiers, keywords, punctuators, numbers, booleans, `null` —
/// any token whose `value` cannot contain quotes, backslashes, or control characters.
///
/// Construct JSON manually and write it all in one go with a single bounds check using
/// `CodeBuffer::print_strs_array`. This is far more efficient than a standard `serialize_struct` approach.
struct ESTreeSafeToken<'a> {
    token_type: TokenType,
    value: &'a str,
    span: Span,
}

impl ESTree for ESTreeSafeToken<'_> {
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

/// Token whose `value` may not be JSON-safe.
///
/// Used for strings, templates, escaped identifiers, and JSXText.
struct ESTreeUnsafeToken<'a> {
    token_type: TokenType,
    value: &'a str,
    span: Span,
}

impl ESTree for ESTreeUnsafeToken<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString(self.token_type.as_str()));
        state.serialize_field("value", &self.value);
        state.serialize_span(self.span);
        state.end();
    }
}

/// `RegularExpression` token.
///
/// This is a separate type from `ESTreeSafeToken` / `ESTreeUnsafeToken` because RegExp tokens have
/// a nested `regex` object containing `flags` and `pattern`.
///
/// Pattern is taken from the AST node (`RegExpLiteral.regex.pattern.text`).
/// Flags are sliced from source text to preserve the original order
/// (the AST stores flags as a bitfield which would alphabetize them).
struct ESTreeRegExpToken<'a> {
    value: &'a str,
    regex: RegExpData<'a>,
    span: Span,
}

/// The `regex` sub-object inside `ESTreeRegExpToken`.
struct RegExpData<'a> {
    pattern: &'a str,
    flags: &'a str,
}

impl ESTree for ESTreeRegExpToken<'_> {
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
