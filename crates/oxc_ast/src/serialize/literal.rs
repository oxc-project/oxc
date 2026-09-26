use oxc_ast_macros::ast_meta;
use oxc_estree::{ESTree, JsonSafeString, LoneSurrogatesString, Serializer, StructSerializer};

use crate::ast::*;

use super::Null;

/// Serializer for `raw` field of `BooleanLiteral`.
///
/// If `BooleanLiteral` has a span, set `raw` field to `"true"` or `"false"`.
/// If no span, `null`.
#[ast_meta]
#[estree(
    ts_type = "string | null",
    raw_deser = "(THIS.start === 0 && THIS.end === 0) ? null : THIS.value + ''"
)]
pub struct BooleanLiteralRaw<'b>(pub &'b BooleanLiteral);

impl ESTree for BooleanLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        #[expect(clippy::collection_is_never_read)] // Clippy is wrong!
        let raw = if self.0.span.is_unspanned() {
            None
        } else if self.0.value {
            Some(JsonSafeString("true"))
        } else {
            Some(JsonSafeString("false"))
        };
        raw.serialize(serializer);
    }
}

/// Serializer for `raw` field of `NullLiteral`.
///
/// If `NullLiteral` has a span, set `raw` field to `"null"`. If no span, `null`.
#[ast_meta]
#[estree(
    ts_type = "'null' | null",
    raw_deser = "(THIS.start === 0 && THIS.end === 0) ? null : 'null'",
    raw_deser_inline
)]
pub struct NullLiteralRaw<'b>(pub &'b NullLiteral);

impl ESTree for NullLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        #[expect(clippy::collection_is_never_read)] // Clippy is wrong!
        let raw = if self.0.span.is_unspanned() { None } else { Some(JsonSafeString("null")) };
        raw.serialize(serializer);
    }
}

/// Serializer for `value` field of `StringLiteral`.
///
/// Handle when `lone_surrogates` flag is set, indicating the string contains lone surrogates.
#[ast_meta]
#[estree(
    ts_type = "string",
    raw_deser = r#"
        let value = DESER[Str](POS_OFFSET.value);
        if (DESER[bool](POS_OFFSET.lone_surrogates)) {
            value = value.replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16)));
        }
        value
    "#
)]
pub struct StringLiteralValue<'a, 'b>(pub &'b StringLiteral<'a>);

impl ESTree for StringLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let lit = self.0;
        #[expect(clippy::if_not_else)]
        if !lit.lone_surrogates {
            lit.value.serialize(serializer);
        } else {
            // String contains lone surrogates. Very uncommon, so cold path.
            self.serialize_lone_surrogates(serializer);
        }
    }
}

impl StringLiteralValue<'_, '_> {
    #[cold]
    #[inline(never)]
    fn serialize_lone_surrogates<S: Serializer>(&self, serializer: S) {
        LoneSurrogatesString(self.0.value.as_str()).serialize(serializer);
    }
}

/// Serializer for `value` field of `BigIntLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `BigInt`.
///
/// In the Rust type, `value` field is the BigInt as a string.
/// This ends up in ESTree AST as `bigint` property.
///
/// We have to use 2 serializers here to swap the fields around,
/// because otherwise there's a clash on the field name `value`.
#[ast_meta]
#[estree(
    ts_type = "bigint",
    raw_deser = "
        const bigint = DESER[Str](POS_OFFSET.value);
        BigInt(bigint)
    ",
    raw_deser_inline
)]
pub struct BigIntLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        // Record that this node needs fixing on JS side
        serializer.record_fix_path();
        Null(()).serialize(serializer);
    }
}

/// Serializer for `bigint` field of `BigIntLiteral`.
///
/// Comes from `value` field in Rust type.
///
/// `bigint` var in `raw_deser` comes from `BigIntLiteralValue` serializer.
#[ast_meta]
#[estree(ts_type = "string", raw_deser = "bigint", raw_deser_inline)]
pub struct BigIntLiteralBigint<'a, 'b>(pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralBigint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString(self.0.value.as_str()).serialize(serializer);
    }
}

/// Serializer for `value` field of `RegExpLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `RegExp`, if the regexp is valid.
/// `regex` in `raw_deser` is initialized by `RegExpLiteralRegex`.
#[ast_meta]
#[estree(
    ts_type = "RegExp | null",
    raw_deser = "
        let value = null;
        try {
            value = new RegExp(regex.pattern, regex.flags);
        } catch (e) {}
        value
    "
)]
pub struct RegExpLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b RegExpLiteral<'a>);

impl ESTree for RegExpLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        // Record that this node needs fixing on JS side
        serializer.record_fix_path();
        Null(()).serialize(serializer);
    }
}

/// Serializer for the `regex` field, preserving source flag order when it matches the AST.
#[ast_meta]
#[estree(
    ts_type = "{ pattern: string; flags: string; }",
    raw_deser = "
        const regexStart = DESER[u32](POS_OFFSET.span.start);
        const regexEnd = DESER[u32](POS_OFFSET.span.end);
        const flagBits = DESER[u8](POS_OFFSET.regex.flags);
        let rawBits = 0, flagStart = regexEnd;
        // Raw transfer only handles parsed ASTs. Read flags backwards from the source span.
        // Reject unknown or repeated flags, including those from parser error recovery.
        while (flagStart > regexStart && SOURCE_TEXT.charCodeAt(flagStart - 1) !== 47) {
            const index = 'gimsuydv'.indexOf(SOURCE_TEXT[--flagStart]);
            const bit = 1 << index;
            if (index === -1 || (rawBits & bit) !== 0) {
                rawBits = -1;
                break;
            }
            rawBits |= bit;
        }
        const flags = rawBits === flagBits && flagStart > regexStart
            ? SOURCE_TEXT.slice(flagStart, regexEnd)
            : DESER[RegExpFlags](POS_OFFSET.regex.flags);
        const regex = { pattern: DESER[Str](POS_OFFSET.regex.pattern.text), flags };
        regex
    ",
    raw_deser_inline
)]
pub struct RegExpLiteralRegex<'a, 'b>(pub &'b RegExpLiteral<'a>);

impl ESTree for RegExpLiteralRegex<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let literal = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &literal.regex.pattern.text);
        if let Some(flags) = source_regexp_flags(literal) {
            state.serialize_field("flags", &JsonSafeString(flags));
        } else {
            state.serialize_field("flags", &literal.regex.flags);
        }
        state.end();
    }
}

/// Borrow source flags only when they still represent the literal's flag set.
fn source_regexp_flags<'a>(literal: &RegExpLiteral<'a>) -> Option<&'a str> {
    let raw = literal.raw?.as_str().strip_prefix('/')?;
    let (_, flags) = raw.rsplit_once('/')?;
    if flags.len() != literal.regex.flags.bits().count_ones() as usize {
        return None;
    }
    let mut parsed = RegExpFlags::empty();
    for byte in flags.bytes() {
        parsed |= RegExpFlags::try_from(byte).ok()?;
    }
    // Equal length and flag sets also exclude duplicate flags.
    (parsed == literal.regex.flags).then_some(flags)
}

/// Converter for `RegExpFlags`.
///
/// Serialize as a string, with flags in alphabetical order.
#[ast_meta]
#[estree(
    ts_type = "string",
    raw_deser = "
        const flagBits = DESER[u8](POS);
        let flags = '';
        // Alphabetical order
        if (flagBits & 64) flags += 'd';
        if (flagBits & 1) flags += 'g';
        if (flagBits & 2) flags += 'i';
        if (flagBits & 4) flags += 'm';
        if (flagBits & 8) flags += 's';
        if (flagBits & 16) flags += 'u';
        if (flagBits & 128) flags += 'v';
        if (flagBits & 32) flags += 'y';
        flags
    "
)]
pub struct RegExpFlagsConverter<'b>(pub &'b RegExpFlags);

impl ESTree for RegExpFlagsConverter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString(self.0.to_inline_string().as_str()).serialize(serializer);
    }
}

/// Converter for `TemplateElement`.
///
/// Decode `cooked` if it contains lone surrogates.
///
/// Also adjust span in TS AST.
/// TS-ESLint produces a different span from Acorn:
/// ```js
/// const template = `abc${x}def${x}ghi`;
/// // Acorn:         ^^^    ^^^    ^^^
/// // TS-ESLint:    ^^^^^^ ^^^^^^ ^^^^^
/// ```
// TODO: Raise an issue on TS-ESLint and see if they'll change span to match Acorn.
#[ast_meta]
#[estree(raw_deser = r#"
    const tail = DESER[bool](POS_OFFSET.tail),
        start = IS_TS ? DESER[i32](POS_OFFSET.span.start) - 1 : DESER[i32](POS_OFFSET.span.start),
        end = IS_TS ? DESER[i32](POS_OFFSET.span.end) + 2 - tail : DESER[i32](POS_OFFSET.span.end),
        value = DESER[TemplateElementValue](POS_OFFSET.value);
    if (value.cooked !== null && DESER[bool](POS_OFFSET.lone_surrogates)) {
        value.cooked = value.cooked
            .replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16)));
    }
    { type: 'TemplateElement', value, tail, start, end, ...(RANGE && { range: [start, end] }), ...(PARENT && { parent }) }
"#)]
pub struct TemplateElementConverter<'a, 'b>(pub &'b TemplateElement<'a>);

impl ESTree for TemplateElementConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let element = self.0;

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TemplateElement"));

        state.serialize_field("value", &TemplateElementValue(element));
        state.serialize_field("tail", &element.tail);

        let mut span = element.span;
        if state.include_ts_fields() {
            span.start -= 1;
            span.end += if element.tail { 1 } else { 2 };
        }
        state.serialize_span(span);

        state.end();
    }
}

/// Serializer for `value` field of `TemplateElement`.
///
/// Handle when `lone_surrogates` flag is set, indicating the cooked string contains lone surrogates.
///
/// Implementation for `raw_deser` is included in `TemplateElementConverter` above.
#[ast_meta]
#[estree(
    ts_type = "TemplateElementValue",
    raw_deser = "(() => { throw new Error('Should not appear in deserializer code'); })()"
)]
pub struct TemplateElementValue<'a, 'b>(pub &'b TemplateElement<'a>);

impl ESTree for TemplateElementValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let element = self.0;
        #[expect(clippy::if_not_else)]
        if !element.lone_surrogates {
            element.value.serialize(serializer);
        } else {
            // String contains lone surrogates. Very uncommon, so cold path.
            self.serialize_lone_surrogates(serializer);
        }
    }
}

impl TemplateElementValue<'_, '_> {
    #[cold]
    #[inline(never)]
    fn serialize_lone_surrogates<S: Serializer>(&self, serializer: S) {
        let value = &self.0.value;

        let mut state = serializer.serialize_struct();
        state.serialize_field("raw", &value.raw);

        let cooked = value.cooked.as_ref().map(|cooked| LoneSurrogatesString(cooked.as_str()));
        state.serialize_field("cooked", &cooked);

        state.end();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use oxc_estree::{CompactSerializer, ESTree};
    use oxc_span::Span;

    use crate::ast::{RegExp, RegExpFlags, RegExpLiteral, RegExpPattern};

    fn serialize_regex(raw: Option<&str>, flags: RegExpFlags) -> String {
        let literal = RegExpLiteral {
            node_id: Cell::default(),
            span: Span::default(),
            regex: RegExp { pattern: RegExpPattern { text: "a".into(), pattern: None }, flags },
            raw: raw.map(Into::into),
        };
        let mut serializer = CompactSerializer::with_capacity(128, false, false);
        literal.serialize(&mut serializer);
        serializer.into_string()
    }

    #[test]
    fn regexp_preserves_source_flag_order() {
        let flags = RegExpFlags::G | RegExpFlags::I;
        for raw in ["/a/ig", r"/a\/é/ig"] {
            assert!(
                serialize_regex(Some(raw), flags)
                    .contains(r#""regex":{"pattern":"a","flags":"ig"}"#)
            );
        }
        assert!(
            serialize_regex(Some("/a/"), RegExpFlags::empty())
                .contains(r#""regex":{"pattern":"a","flags":""}"#)
        );
    }

    #[test]
    fn regexp_uses_canonical_flags_without_matching_raw() {
        let flags = RegExpFlags::G | RegExpFlags::I;
        for raw in [None, Some("/a/m"), Some("/a/im"), Some("/a/gg"), Some("/a/gz"), Some("ig")] {
            assert!(
                serialize_regex(raw, flags).contains(r#""regex":{"pattern":"a","flags":"gi"}"#),
                "raw: {raw:?}"
            );
        }
    }
}
