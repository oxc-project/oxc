use oxc_ast::ast::*;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_estree::{CompactFormatter, Config, ESTreeSerializer, PrettyFormatter};
use oxc_parser::Token;

mod jsx_state;
mod options;
mod serialize;
mod token_type;
mod u32_string;

pub use jsx_state::{JSXState, JSXStateJS, JSXStateTS};
pub use options::{
    ESTreeTokenConfig, ESTreeTokenOptions, ESTreeTokenOptionsJS, ESTreeTokenOptionsTS,
};
use serialize::{estimate_json_len, serialize_tokens};

/// Serializer config for tokens.
/// We never include ranges, so use this custom config which returns `false` for `ranges()`.
/// This allows compiler to remove the branch which checks whether to print ranges in `serialize_span`.
struct TokenSerializerConfig;

impl Config for TokenSerializerConfig {
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
