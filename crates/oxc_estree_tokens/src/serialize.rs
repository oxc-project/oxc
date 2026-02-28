use std::slice::{Iter, IterMut};

use oxc_ast::ast::*;
use oxc_ast_visit::{
    Visit,
    utf8_to_utf16::{Utf8ToUtf16, Utf8ToUtf16Converter},
    walk,
};
use oxc_estree::{ESTree, JsonSafeString, SequenceSerializer, Serializer, StructSerializer};
use oxc_parser::{Kind, Token};
use oxc_span::{GetSpan, Span};

use crate::{ESTreeTokenConfig, JSXState, token_type::TokenType, u32_string::U32String};

// ==============================================================================================
// Entry points
// ==============================================================================================

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
pub fn estimate_json_len(tokens_len: usize, source_text_len: usize, is_compact: bool) -> usize {
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
pub fn serialize_tokens<O: ESTreeTokenConfig>(
    serializer: impl Serializer,
    tokens: &[Token],
    program: &Program<'_>,
    source_text: &str,
    span_converter: &Utf8ToUtf16,
    options: O,
) {
    let mut visitor = Visitor {
        ctx: JsonContext {
            seq: serializer.serialize_sequence(),
            tokens: tokens.iter(),
            source_text,
            span_converter: span_converter.converter(),
            options,
            jsx_state: O::JSXState::default(),
        },
    };
    visitor.visit_program(program);
    visitor.ctx.finish();
}

/// Walk AST and update token kinds to match ESTree token types.
/// Convert token spans from UTF-8 byte offsets to UTF-16 offsets.
///
/// After this pass, `get_token_type(token.kind())` returns the correct ESTree token type
/// for every token, without needing AST context.
pub fn update_tokens<O: ESTreeTokenConfig>(
    tokens: &mut [Token],
    program: &Program<'_>,
    span_converter: &Utf8ToUtf16,
    options: O,
) {
    let mut visitor = Visitor {
        ctx: UpdateContext {
            tokens: tokens.iter_mut(),
            span_converter: span_converter.converter(),
            options,
            jsx_state: O::JSXState::default(),
        },
    };
    visitor.visit_program(program);
    visitor.ctx.finish();
}

// ==============================================================================================
// `Context` trait
// ==============================================================================================

/// Trait abstracting over the two token processing modes:
/// JSON serialization ([`JsonContext`]) and in-place kind update ([`UpdateContext`]).
///
/// Each implementation holds its own `options` and `jsx_state`, so `is_ts` / `is_js`
/// resolve statically when the generic `O: ESTreeTokenConfig` is monomorphized.
trait Context: Sized {
    /// JSX state type for tracking when to emit JSX identifiers.
    type JSXState: JSXState;

    /// Returns `true` if serializing in TS style.
    fn is_ts(&self) -> bool;

    /// Returns `true` if serializing in JS style.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_js(&self) -> bool {
        !self.is_ts()
    }

    /// Get reference to [`JSXState`] for the serializer/updater.
    fn jsx_state(&self) -> &Self::JSXState;

    /// Get mutable reference to [`JSXState`] for the serializer/updater.
    fn jsx_state_mut(&mut self) -> &mut Self::JSXState;

    /// Emit the token at `start` as an identifier.
    ///
    /// * JSON mode: Serialize with type `Identifier` or `Keyword`.
    /// * Update mode: Set kind to `Kind::Ident`, unless in JS style and the token is `yield` / `let` / `static`
    ///   (which should remain as `Keyword`).
    fn emit_identifier_at(&mut self, start: u32, name: &str);

    /// Emit the `this` keyword at `start` as `Identifier`.
    ///
    /// * JSON mode: Serialize as `Identifier` / `"this"`.
    /// * Update mode: Set kind to `Kind::Ident`.
    fn emit_this_identifier_at(&mut self, start: u32);

    /// Emit the token at `start` as `JSXIdentifier`.
    ///
    /// * JSON mode: Serialize as `JSXIdentifier`.
    /// * Update mode: Set kind to `Kind::JSXIdentifier`.
    fn emit_jsx_identifier_at(&mut self, start: u32, name: &str);

    /// Emit the token at `start` as `PrivateIdentifier`.
    ///
    /// * JSON mode: Serialize with appropriate encoding.
    /// * Update mode: No-op (token already has `Kind::PrivateIdentifier`).
    fn emit_private_identifier_at(&mut self, start: u32, name: &str);

    /// Emit a `StringLiteral` in a JSX attribute as `JSXText`.
    ///
    /// Unlike [`emit_unsafe_token_at`], this changes the token's kind in update mode,
    /// because the token has `Kind::Str` but needs to become `Kind::JSXText`.
    /// Use [`emit_unsafe_token_at`] for actual `JSXText` tokens which already have the correct kind.
    ///
    /// * JSON mode: Serialize as `JSXText` with JSON encoding.
    /// * Update mode: Set kind to `Kind::JSXText`.
    ///
    /// [`emit_unsafe_token_at`]: Context::emit_unsafe_token_at
    fn emit_jsx_text_at(&mut self, start: u32);

    /// Emit a token whose value may not be JSON-safe (strings, templates, JSXText).
    ///
    /// * JSON mode: Serialize with JSON encoding.
    /// * Update mode: No-op (token already has the correct kind).
    fn emit_unsafe_token_at(&mut self, start: u32, token_type: TokenType);

    /// Emit a `RegularExpression` token.
    ///
    /// * JSON mode: Serialize using `ESTreeRegExpToken`.
    /// * Update mode: No-op (token already has the correct kind).
    fn emit_regexp(&mut self, regexp: &RegExpLiteral<'_>);

    /// Walk template quasis interleaved with their interpolated parts (expressions or TS types).
    ///
    /// * JSON mode: Emit quasi tokens interleaved with interpolation visits.
    /// * Update mode: Only visit interpolations (quasis don't need `Kind` changes).
    fn walk_template_quasis_interleaved<I>(
        visitor: &mut Visitor<Self>,
        quasis: &[TemplateElement<'_>],
        visit_interpolation: impl FnMut(&mut Visitor<Self>, &I),
        interpolations: &[I],
    );
}

// ==============================================================================================
// `JsonContext` — JSON serialization
// ==============================================================================================

/// JSON serialization context.
///
/// Serializes each token to JSON with its correct ESTree token type.
struct JsonContext<'b, O: ESTreeTokenConfig, S: SequenceSerializer> {
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

    fn emit_safe_token_at(&mut self, start: u32, token_type: TokenType, value: &str) {
        let token = self.advance_to(start);
        self.serialize_safe_token(token, token_type, value);
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

    /// Get reference to [`JSXState`] for the serializer/updater.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state(&self) -> &Self::JSXState {
        &self.jsx_state
    }

    /// Get mutable reference to [`JSXState`] for the serializer/updater.
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

    /// Emit the token at `start` as `PrivateIdentifier`.
    fn emit_private_identifier_at(&mut self, start: u32, name: &str) {
        let token = self.advance_to(start);

        // `identifier.name` has `#` stripped and escapes decoded by the parser, and is JSON-safe.
        // Use it in most cases — if token is not marked as escaped, it's JSON-safe, so can skip JSON encoding.
        // When `self.is_js()` is `true`, token `value` should *always* be the unescaped version,
        // so can also use `name` from AST node and skip JSON encoding.
        // Only fall back to raw source text when the token contains escapes *and* decoding is disabled,
        // since escape sequences contain `\` which needs JSON escaping.
        // Escaped identifiers are extremely rare, so handle them in `#[cold]` branch.
        if self.is_js() || !token.escaped() {
            self.serialize_safe_token(token, TokenType::new("PrivateIdentifier"), name);
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

    /// Emit the token at `start` as `JSXText`.
    fn emit_jsx_text_at(&mut self, start: u32) {
        let token = self.advance_to(start);
        self.emit_unsafe_token(token, TokenType::new("JSXText"));
    }

    /// Emit the token at `start` as the specified token type,
    /// where the token's `value` may not be JSON-safe.
    fn emit_unsafe_token_at(&mut self, start: u32, token_type: TokenType) {
        let token = self.advance_to(start);
        self.emit_unsafe_token(token, token_type);
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
            visitor.ctx.emit_unsafe_token_at(quasi.span.start - 1, TokenType::new("Template"));
        }

        // Remaining quasis interleaved with interpolations
        for (interpolation, quasi) in interpolations.iter().zip(quasis) {
            visit_interpolation(visitor, interpolation);
            visitor.ctx.emit_unsafe_token_at(quasi.span.start - 1, TokenType::new("Template"));
        }
    }
}

// ==============================================================================================
// Token serialization structs (used only by `JsonContext`)
// ==============================================================================================

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

// ==============================================================================================
// `UpdateContext` — in-place token `Kind` mutation
// ==============================================================================================

/// In-place kind update context.
///
/// Updates token kinds so that `get_token_type(token.kind())` returns
/// the correct ESTree token type without needing AST context.
/// Also converts token spans from UTF-8 byte offsets to UTF-16 offsets.
struct UpdateContext<'b, O: ESTreeTokenConfig> {
    /// Mutable tokens iterator
    tokens: IterMut<'b, Token>,
    /// Span converter for UTF-8 to UTF-16 conversion.
    /// `None` if source is ASCII-only.
    span_converter: Option<Utf8ToUtf16Converter<'b>>,
    /// Options controlling JS/TS style differences
    options: O,
    /// JSX state tracking
    jsx_state: O::JSXState,
}

impl<O: ESTreeTokenConfig> UpdateContext<'_, O> {
    /// Advance iterator to the token at `start`, converting spans along the way.
    /// Skipped tokens are not modified (they already have the correct kind),
    /// but their spans are converted from UTF-8 to UTF-16.
    fn advance_to(&mut self, start: u32) -> &mut Token {
        let Self { tokens, span_converter, .. } = self;

        if let Some(span_converter) = span_converter {
            for token in tokens {
                debug_assert!(
                    token.start() <= start,
                    "Expected token at position {start}, found token at position {}",
                    token.start(),
                );

                let is_target = token.start() == start;

                // Convert span from UTF-8 byte offsets to UTF-16 offsets
                let mut span = token.span();
                span_converter.convert_span(&mut span);
                token.set_span(span);

                if is_target {
                    return token;
                }
            }
        } else {
            for token in tokens {
                debug_assert!(
                    token.start() <= start,
                    "Expected token at position {start}, found token at position {}",
                    token.start(),
                );

                if token.start() == start {
                    return token;
                }
            }
        }

        unreachable!("Expected token at position {start}");
    }

    /// Convert remaining token spans from UTF-8 byte offsets to UTF-16 offsets.
    fn finish(self) {
        if let Some(mut converter) = self.span_converter {
            for token in self.tokens {
                let mut span = token.span();
                converter.convert_span(&mut span);
                token.set_span(span);
            }
        }
    }
}

impl<O: ESTreeTokenConfig> Context for UpdateContext<'_, O> {
    /// JSX state type for tracking when to emit JSX identifiers.
    /// Inherited from config.
    type JSXState = O::JSXState;

    /// Returns `true` if serializing in TS style.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn is_ts(&self) -> bool {
        self.options.is_ts()
    }

    /// Get reference to [`JSXState`] for the serializer/updater.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state(&self) -> &Self::JSXState {
        &self.jsx_state
    }

    /// Get mutable reference to [`JSXState`] for the serializer/updater.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn jsx_state_mut(&mut self) -> &mut Self::JSXState {
        &mut self.jsx_state
    }

    /// Set `Kind` of the token at `start` to `Identifier`.
    /// In JS mode, if it's a `yield`, `let`, or `static` keyword, leave it as a `Keyword` token.
    fn emit_identifier_at(&mut self, start: u32, _name: &str) {
        let is_ts = self.is_ts();

        let token = self.advance_to(start);

        // In JS style, `yield` / `let` / `static` used as identifiers should remain as keywords
        if is_ts || !matches!(token.kind(), Kind::Yield | Kind::Let | Kind::Static) {
            token.set_kind(Kind::Ident);
        }
    }

    /// Set `Kind` of the token at `start` to `Identifier`.
    fn emit_this_identifier_at(&mut self, start: u32) {
        let token = self.advance_to(start);
        token.set_kind(Kind::Ident);
    }

    /// Set `Kind` of the token at `start` to `JSXIdentifier`.
    fn emit_jsx_identifier_at(&mut self, start: u32, _name: &str) {
        let token = self.advance_to(start);
        token.set_kind(Kind::JSXIdentifier);
    }

    /// Handle `PrivateIdentifier` token at `start` (no-op).
    #[inline(always)]
    fn emit_private_identifier_at(&mut self, _start: u32, _name: &str) {
        // No-op: token already has `Kind::PrivateIdentifier`.
        // The iterator will skip past this token on the next `advance_to` call.
    }

    /// Set `Kind` of the token at `start` to `JSXText`.
    fn emit_jsx_text_at(&mut self, start: u32) {
        let token = self.advance_to(start);
        token.set_kind(Kind::JSXText);
    }

    /// Handle token at `start` (no-op).
    #[inline(always)]
    fn emit_unsafe_token_at(&mut self, _start: u32, _token_type: TokenType) {}

    /// Handle `RegExpLiteral` (no-op).
    #[inline(always)]
    fn emit_regexp(&mut self, _regexp: &RegExpLiteral<'_>) {}

    /// Walk template interpolations (expressions or TS types).
    fn walk_template_quasis_interleaved<I>(
        visitor: &mut Visitor<Self>,
        _quasis: &[TemplateElement<'_>],
        mut visit_interpolation: impl FnMut(&mut Visitor<Self>, &I),
        interpolations: &[I],
    ) {
        // Quasis don't need kind changes, so skip them and only visit interpolations
        for interpolation in interpolations {
            visit_interpolation(visitor, interpolation);
        }
    }
}

// ==============================================================================================
// `Visitor` — the visitor
// ==============================================================================================

/// Visitor that walks the AST and delegates token processing to a [`Context`].
///
/// AST visitation is in source order, matching the order of tokens in the iterator.
///
/// Tokens are consumed from `tokens` iterator in source order. When visitor method encounters
/// an AST node that requires a token type override, all preceding tokens are emitted
/// with their default types, then the overridden token is emitted with its corrected type.
/// After the AST walk, any remaining tokens are emitted with default types.
///
/// This wrapper is needed because Rust's orphan rules prevent implementing the foreign [`Visit`] trait
/// directly on [`Context`] implementors (which are generic over `O: ESTreeTokenConfig`).
/// `Visitor` is a local type, so it can implement [`Visit`].
#[repr(transparent)]
struct Visitor<C: Context> {
    ctx: C,
}

impl<'a, C: Context> Visit<'a> for Visitor<C> {
    fn visit_ts_type_name(&mut self, type_name: &TSTypeName<'a>) {
        // `this` is emitted as `Identifier` token instead of `Keyword`
        match type_name {
            TSTypeName::ThisExpression(this_expr) => {
                self.ctx.emit_this_identifier_at(this_expr.span.start);
            }
            TSTypeName::IdentifierReference(ident) => {
                self.visit_identifier_reference(ident);
            }
            TSTypeName::QualifiedName(qualified_name) => {
                self.visit_ts_qualified_name(qualified_name);
            }
        }
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        // Manual walk.
        // * `source` is a `StringLiteral` — visit to ensure it's emitted with JSON encoding
        //   (string values are not JSON-safe). No-op in update mode.
        // * `options` is an `ObjectExpression`. Manually walk each property, but don't visit the key if it's `with`,
        //   as it needs to remain a `Keyword` token, not get converted to `Identifier`.
        // * `qualifier` and `type_arguments` are visited as usual.
        self.visit_string_literal(&import_type.source);

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
        if self.ctx.is_ts() && self.ctx.jsx_state().should_emit_jsx_identifier() {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.ctx.is_ts() && self.ctx.jsx_state().should_emit_jsx_identifier() {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_private_identifier(&mut self, identifier: &PrivateIdentifier<'a>) {
        self.ctx.emit_private_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_reg_exp_literal(&mut self, regexp: &RegExpLiteral<'a>) {
        self.ctx.emit_regexp(regexp);
    }

    fn visit_ts_this_parameter(&mut self, parameter: &TSThisParameter<'a>) {
        self.ctx.emit_this_identifier_at(parameter.this_span.start);
        walk::walk_ts_this_parameter(self, parameter);
    }

    fn visit_meta_property(&mut self, _meta_property: &MetaProperty<'a>) {
        // Don't walk.
        // * `meta` (either `import` or `new`) has a `Keyword` token already, which is correct.
        // * `property` (either `meta` or `target`) has an `Identifier` token, which is correct.
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid emitting the same token twice.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_expression(&property.value);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid emitting the same token twice.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_binding_pattern(&property.value);
    }

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        // For `import { x }`, `imported` and `local` share the same span.
        // Only visit `imported` when it differs from `local`, to avoid emitting the same token twice.
        if specifier.imported.span() != specifier.local.span {
            self.visit_module_export_name(&specifier.imported);
        }
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_export_specifier(&mut self, specifier: &ExportSpecifier<'a>) {
        // For `export { x }`, `local` and `exported` share the same span.
        // Only visit `exported` when it differs from `local`, to avoid emitting the same token twice.
        self.visit_module_export_name(&specifier.local);
        if specifier.exported.span() != specifier.local.span() {
            self.visit_module_export_name(&specifier.exported);
        }
    }

    fn visit_jsx_identifier(&mut self, identifier: &JSXIdentifier<'a>) {
        self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        if let JSXElementName::IdentifierReference(identifier) = name {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            walk::walk_jsx_element_name(self, name);
        }
    }

    fn visit_jsx_member_expression_object(&mut self, object: &JSXMemberExpressionObject<'a>) {
        if let JSXMemberExpressionObject::IdentifierReference(identifier) = object {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            walk::walk_jsx_member_expression_object(self, object);
        }
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        if self.ctx.is_js() {
            self.ctx.emit_jsx_identifier_at(name.namespace.span.start, &name.namespace.name);
            self.ctx.emit_jsx_identifier_at(name.name.span.start, &name.name.name);
        } else {
            // In TS mode, these tokens retain their default type (`Identifier`)
        }
    }

    fn visit_jsx_expression_container(&mut self, container: &JSXExpressionContainer<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_expression_container(self, container);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_member_expression(&mut self, member_expr: &MemberExpression<'a>) {
        self.ctx.jsx_state_mut().enter_member_expression(member_expr);
        walk::walk_member_expression(self, member_expr);
        self.ctx.jsx_state_mut().exit_member_expression(member_expr);
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_spread_attribute(self, attribute);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_jsx_spread_child(&mut self, spread_child: &JSXSpreadChild<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_spread_child(self, spread_child);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        // No-op in update mode - token's `Kind` is already `String`
        self.ctx.emit_unsafe_token_at(literal.span.start, TokenType::new("String"));
    }

    fn visit_jsx_text(&mut self, text: &JSXText<'a>) {
        // Use `emit_unsafe_token_at` not `emit_jsx_text_at`, as the token's `Kind` is already `JSXText`,
        // so no-op in update mode
        self.ctx.emit_unsafe_token_at(text.span.start, TokenType::new("JSXText"));
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        // Manual walk.
        // * `name`: Visit normally.
        // * `value`: Set `JSXText` token type if it's a `StringLiteral`.
        self.visit_jsx_attribute_name(&attribute.name);
        match &attribute.value {
            Some(JSXAttributeValue::StringLiteral(string_literal)) => {
                // Use `emit_jsx_text_at` not `emit_unsafe_token_at`, as the token `Kind`
                // needs to be updated to `JSXText` in update mode
                self.ctx.emit_jsx_text_at(string_literal.span.start);
            }
            Some(value) => self.visit_jsx_attribute_value(value),
            None => {}
        }
    }

    fn visit_template_literal(&mut self, literal: &TemplateLiteral<'a>) {
        C::walk_template_quasis_interleaved(
            self,
            &literal.quasis,
            Visit::visit_expression,
            &literal.expressions,
        );
    }

    fn visit_ts_template_literal_type(&mut self, literal: &TSTemplateLiteralType<'a>) {
        C::walk_template_quasis_interleaved(
            self,
            &literal.quasis,
            Visit::visit_ts_type,
            &literal.types,
        );
    }
}
