use std::cmp;

use cow_utils::CowUtils;

use crate::ast::*;
use oxc_ast_macros::ast_meta;
use oxc_estree::{
    CompactJSSerializer, CompactTSSerializer, ESTree, FlatStructSerializer, JsonSafeString,
    LoneSurrogatesString, PrettyJSSerializer, PrettyTSSerializer, SequenceSerializer, Serializer,
    StructSerializer, ser::AppendToConcat,
};
use oxc_span::GetSpan;

/// Main serialization methods for `Program`.
///
/// Note: 4 separate methods for the different serialization options, rather than 1 method
/// with behavior controlled by flags (e.g. `fn to_estree_json(&self, with_ts: bool, pretty: bool`)
/// to avoid bloating binary size.
///
/// Most consumers (and Oxc crates) will use only 1 of these methods, so we don't want to needlessly
/// compile all 4 serializers when only 1 is used.
///
/// Initial capacity for serializer's buffer is an estimate based on our benchmark fixtures
/// of ratio of source text size to JSON size.
///
/// | File                       | Compact TS | Compact JS | Pretty TS | Pretty JS |
/// |----------------------------|------------|------------|-----------|-----------|
/// | antd.js                    |         10 |          9 |        76 |        72 |
/// | cal.com.tsx                |         10 |          9 |        40 |        37 |
/// | checker.ts                 |          7 |          6 |        27 |        24 |
/// | pdf.mjs                    |         13 |         12 |        71 |        67 |
/// | RadixUIAdoptionSection.jsx |         10 |          9 |        45 |        44 |
/// |----------------------------|------------|------------|-----------|-----------|
/// | Maximum                    |         13 |         12 |        76 |        72 |
///
/// It's better to over-estimate than under-estimate, as having to grow the buffer is expensive,
/// so have gone on the generous side.
const JSON_CAPACITY_RATIO_COMPACT: usize = 16;
const JSON_CAPACITY_RATIO_PRETTY: usize = 80;

impl Program<'_> {
    /// Serialize AST to ESTree JSON, including TypeScript fields.
    pub fn to_estree_ts_json(&self) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let mut serializer = CompactTSSerializer::with_capacity(capacity);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to ESTree JSON, without TypeScript fields.
    pub fn to_estree_js_json(&self) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_COMPACT;
        let mut serializer = CompactJSSerializer::with_capacity(capacity);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to pretty-printed ESTree JSON, including TypeScript fields.
    pub fn to_pretty_estree_ts_json(&self) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let mut serializer = PrettyTSSerializer::with_capacity(capacity);
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to pretty-printed ESTree JSON, without TypeScript fields.
    pub fn to_pretty_estree_js_json(&self) -> String {
        let capacity = self.source_text.len() * JSON_CAPACITY_RATIO_PRETTY;
        let mut serializer = PrettyJSSerializer::with_capacity(capacity);
        self.serialize(&mut serializer);
        serializer.into_string()
    }
}

// --------------------
// Program
// --------------------

/// Serializer for `Program`.
///
/// In TS AST, set start span to start of first directive or statement.
/// This is required because unlike Acorn, TS-ESLint excludes whitespace and comments
/// from the `Program` start span.
/// See <https://github.com/oxc-project/oxc/pull/10134> for more info.
///
/// Special case where first statement is an `ExportNamedDeclaration` or `ExportDefaultDeclaration`
/// exporting a class with decorators, where one of the decorators is before `export`.
/// In these cases, the span of the statement starts after the span of the decorators.
/// e.g. `@dec export class C {}` - `ExportNamedDeclaration` span start is 5, `Decorator` span start is 0.
/// `Program` span start is 0 (not 5).
#[ast_meta]
#[estree(raw_deser = "
    const body = DESER[Vec<Directive>](POS_OFFSET.directives);
    body.push(...DESER[Vec<Statement>](POS_OFFSET.body));

    /* IF_JS */
    const start = DESER[u32](POS_OFFSET.span.start);
    /* END_IF_JS */

    const end = DESER[u32](POS_OFFSET.span.end);

    /* IF_TS */
    let start;
    if (body.length > 0) {
        const first = body[0];
        start = first.start;
        if (first.type === 'ExportNamedDeclaration' || first.type === 'ExportDefaultDeclaration') {
            const {declaration} = first;
            if (
                declaration !== null && declaration.type === 'ClassDeclaration'
                && declaration.decorators.length > 0
            ) {
                const decoratorStart = declaration.decorators[0].start;
                if (decoratorStart < start) start = decoratorStart;
            }
        }
    } else {
        start = end;
    }
    /* END_IF_TS */

    const program = {
        type: 'Program',
        start,
        end,
        body,
        sourceType: DESER[ModuleKind](POS_OFFSET.source_type.module_kind),
        hashbang: DESER[Option<Hashbang>](POS_OFFSET.hashbang),
    };
    program
")]
pub struct ProgramConverter<'a, 'b>(pub &'b Program<'a>);

impl ESTree for ProgramConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let program = self.0;
        let span_start =
            if S::INCLUDE_TS_FIELDS { get_ts_start_span(program) } else { program.span.start };

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("Program"));
        state.serialize_field("start", &span_start);
        state.serialize_field("end", &program.span.end);
        state.serialize_field(
            "body",
            &AppendToConcat { array: &program.directives, after: &program.body },
        );
        state.serialize_field("sourceType", &program.source_type.module_kind());
        state.serialize_field("hashbang", &program.hashbang);
        state.end();
    }
}

fn get_ts_start_span(program: &Program<'_>) -> u32 {
    if let Some(first_directive) = program.directives.first() {
        return first_directive.span.start;
    }

    let Some(first_stmt) = program.body.first() else {
        // Program contains no statements or directives. Span start = span end.
        return program.span.end;
    };

    match first_stmt {
        Statement::ExportNamedDeclaration(decl) => {
            let start = decl.span.start;
            if let Some(Declaration::ClassDeclaration(class)) = &decl.declaration {
                if let Some(decorator) = class.decorators.first() {
                    return cmp::min(start, decorator.span.start);
                }
            }
            start
        }
        Statement::ExportDefaultDeclaration(decl) => {
            let start = decl.span.start;
            if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &decl.declaration {
                if let Some(decorator) = class.decorators.first() {
                    return cmp::min(start, decorator.span.start);
                }
            }
            start
        }
        _ => first_stmt.span().start,
    }
}

// --------------------
// Basic types
// --------------------

/// Serialized as `null`.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
pub struct Null<T>(pub T);

impl<T> ESTree for Null<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
#[ts]
pub struct TsNull<T>(pub T);

impl<T> ESTree for TsNull<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

/// Serialized as `true`.
#[ast_meta]
#[estree(ts_type = "true", raw_deser = "true")]
pub struct True<T>(pub T);

impl<T> ESTree for True<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        true.serialize(serializer);
    }
}

/// Serialized as `false`.
#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false")]
pub struct False<T>(pub T);

impl<T> ESTree for False<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false")]
#[ts]
pub struct TsFalse<T>(pub T);

impl<T> ESTree for TsFalse<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `"value"`.
#[ast_meta]
#[estree(ts_type = "'value'", raw_deser = "'value'")]
#[ts]
pub struct TsValue<T>(pub T);

impl<T> ESTree for TsValue<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("value").serialize(serializer);
    }
}

/// Serialized as `"in"`.
#[ast_meta]
#[estree(ts_type = "'in'", raw_deser = "'in'")]
pub struct In<T>(pub T);

impl<T> ESTree for In<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("in").serialize(serializer);
    }
}

/// Serialized as `"init"`.
#[ast_meta]
#[estree(ts_type = "'init'", raw_deser = "'init'")]
pub struct Init<T>(pub T);

impl<T> ESTree for Init<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("init").serialize(serializer);
    }
}

/// Serialized as `"this"`.
#[ast_meta]
#[estree(ts_type = "'this'", raw_deser = "'this'")]
pub struct This<T>(pub T);

impl<T> ESTree for This<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("this").serialize(serializer);
    }
}

/// Serialized as `[]`.
#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]")]
pub struct EmptyArray<T>(pub T);

impl<T> ESTree for EmptyArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        [(); 0].serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]")]
#[ts]
pub struct TsEmptyArray<T>(pub T);

impl<T> ESTree for TsEmptyArray<T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        EmptyArray(()).serialize(serializer);
    }
}

// --------------------
// Literals
// --------------------

/// Serializer for `raw` field of `BooleanLiteral`.
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
#[ast_meta]
#[estree(
    ts_type = "'null' | null",
    raw_deser = "(THIS.start === 0 && THIS.end === 0) ? null : 'null'"
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
        let value = DESER[Atom](POS_OFFSET.value);
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
            // String contains lone surrogates
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

/// Serializer for `bigint` field of `BigIntLiteral`.
#[ast_meta]
#[estree(ts_type = "string", raw_deser = "THIS.raw.slice(0, -1).replace(/_/g, '')")]
pub struct BigIntLiteralBigint<'a, 'b>(pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralBigint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let bigint = self.0.raw[..self.0.raw.len() - 1].cow_replace('_', "");
        JsonSafeString(bigint.as_ref()).serialize(serializer);
    }
}

/// Serializer for `value` field of `BigIntLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `BigInt`.
#[ast_meta]
#[estree(ts_type = "BigInt", raw_deser = "BigInt(THIS.bigint)")]
pub struct BigIntLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

/// Serializer for `value` field of `RegExpLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `RegExp` if the regexp is valid.
#[ast_meta]
#[estree(
    ts_type = "RegExp | null",
    raw_deser = "
        let value = null;
        try {
            value = new RegExp(THIS.regex.pattern, THIS.regex.flags);
        } catch (e) {}
        value
    "
)]
pub struct RegExpLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b RegExpLiteral<'a>);

impl ESTree for RegExpLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "string")]
pub struct RegExpPatternConverter<'a, 'b>(pub &'b RegExpPattern<'a>);

impl ESTree for RegExpPatternConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.to_string().serialize(serializer);
    }
}

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
        start = DESER[u32](POS_OFFSET.span.start) /* IF_TS */ - 1 /* END_IF_TS */,
        end = DESER[u32](POS_OFFSET.span.end) /* IF_TS */ + 2 - tail /* END_IF_TS */,
        value = DESER[TemplateElementValue](POS_OFFSET.value);
    if (value.cooked !== null && DESER[bool](POS_OFFSET.lone_surrogates)) {
        value.cooked = value.cooked
            .replace(/\uFFFD(.{4})/g, (_, hex) => String.fromCodePoint(parseInt(hex, 16)));
    }
    { type: 'TemplateElement', start, end, value, tail }
"#)]
pub struct TemplateElementConverter<'a, 'b>(pub &'b TemplateElement<'a>);

impl ESTree for TemplateElementConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let element = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TemplateElement"));

        let mut span = element.span;
        if S::INCLUDE_TS_FIELDS {
            span.start -= 1;
            span.end += if element.tail { 1 } else { 2 };
        }
        state.serialize_field("start", &span.start);
        state.serialize_field("end", &span.end);

        state.serialize_field("value", &TemplateElementValue(element));
        state.serialize_field("tail", &element.tail);
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
            // String contains lone surrogates
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

// --------------------
// Various
// --------------------

/// Serialize `ArrayExpressionElement::Elision` variant as `null`.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
pub struct ElisionConverter<'b>(#[expect(dead_code)] pub &'b Elision);

impl ESTree for ElisionConverter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

/// Serialize `FormalParameters`, to be estree compatible, with `items` and `rest` fields combined
/// and `argument` field flattened.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Vec<FormalParameter>](POS_OFFSET.items);
        if (uint32[(POS_OFFSET.rest) >> 2] !== 0 && uint32[(POS_OFFSET.rest + 4) >> 2] !== 0) {
            pos = uint32[(POS_OFFSET.rest) >> 2];
            params.push({
                type: 'RestElement',
                start: DESER[u32]( POS_OFFSET<BindingRestElement>.span.start ),
                end: DESER[u32]( POS_OFFSET<BindingRestElement>.span.end ),
                argument: DESER[BindingPatternKind]( POS_OFFSET<BindingRestElement>.argument.kind ),
                /* IF_TS */
                typeAnnotation: DESER[Option<Box<TSTypeAnnotation>>](
                    POS_OFFSET<BindingRestElement>.argument.type_annotation
                ),
                optional: DESER[bool]( POS_OFFSET<BindingRestElement>.argument.optional ),
                /* END_IF_TS */
            });
        }
        params
    "
)]
pub struct FormalParametersConverter<'a, 'b>(pub &'b FormalParameters<'a>);

impl ESTree for FormalParametersConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        for item in &self.0.items {
            seq.serialize_element(item);
        }

        if let Some(rest) = &self.0.rest {
            seq.serialize_element(&FormalParametersRest(rest));
        }

        seq.end();
    }
}

struct FormalParametersRest<'a, 'b>(&'b BindingRestElement<'a>);

impl ESTree for FormalParametersRest<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let rest = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("RestElement"));
        state.serialize_field("start", &rest.span.start);
        state.serialize_field("end", &rest.span.end);
        state.serialize_field("argument", &rest.argument.kind);
        state.serialize_ts_field("typeAnnotation", &rest.argument.type_annotation);
        state.serialize_ts_field("optional", &rest.argument.optional);
        state.serialize_ts_field("decorators", &EmptyArray(()));
        state.serialize_ts_field("value", &Null(()));
        state.end();
    }
}

/// Converter for `FormalParameter`.
///
/// In TS-ESTree AST, if `accessibility` is `Some`, or `readonly` or `override` is `true`,
/// is serialized as `TSParameterProperty` instead, which has a different object shape.
#[ast_meta]
#[estree(
    ts_type = "FormalParameter | TSParameterProperty",
    raw_deser = "
        /* IF_JS */
        DESER[BindingPatternKind](POS_OFFSET.pattern.kind)
        /* END_IF_JS */

        /* IF_TS */
        const accessibility = DESER[Option<TSAccessibility>](POS_OFFSET.accessibility),
            readonly = DESER[bool](POS_OFFSET.readonly),
            override = DESER[bool](POS_OFFSET.override);
        let param;
        if (accessibility === null && !readonly && !override) {
            param = {
                ...DESER[BindingPatternKind](POS_OFFSET.pattern.kind),
                typeAnnotation: DESER[Option<Box<TSTypeAnnotation>>](POS_OFFSET.pattern.type_annotation),
                optional: DESER[bool](POS_OFFSET.pattern.optional),
                decorators: DESER[Vec<Decorator>](POS_OFFSET.decorators),
            };
        } else {
            param = {
                type: 'TSParameterProperty',
                start: DESER[u32](POS_OFFSET.span.start),
                end: DESER[u32](POS_OFFSET.span.end),
                accessibility,
                decorators: DESER[Vec<Decorator>](POS_OFFSET.decorators),
                override,
                parameter: DESER[BindingPattern](POS_OFFSET.pattern),
                readonly,
                static: false,
            };
        }
        param
        /* END_IF_TS */
    "
)]
pub struct FormalParameterConverter<'a, 'b>(pub &'b FormalParameter<'a>);

impl ESTree for FormalParameterConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let param = self.0;

        if S::INCLUDE_TS_FIELDS {
            if param.has_modifier() {
                let mut state = serializer.serialize_struct();
                state.serialize_field("type", &JsonSafeString("TSParameterProperty"));
                state.serialize_field("start", &param.span.start);
                state.serialize_field("end", &param.span.end);
                state.serialize_field("accessibility", &param.accessibility);
                state.serialize_field("decorators", &param.decorators);
                state.serialize_field("override", &param.r#override);
                state.serialize_field("parameter", &param.pattern);
                state.serialize_field("readonly", &param.readonly);
                state.serialize_field("static", &False(()));
                state.end();
            } else {
                let mut state = serializer.serialize_struct();
                param.pattern.kind.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("typeAnnotation", &param.pattern.type_annotation);
                state.serialize_field("optional", &param.pattern.optional);
                state.serialize_field("decorators", &param.decorators);
                state.end();
            }
        } else {
            param.pattern.kind.serialize(serializer);
        }
    }
}

/// Serializer for `params` field of `Function`.
///
/// In TS-ESTree, this adds `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        /* IF_TS */
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param)
        if (thisParam !== null) params.unshift(thisParam);
        /* END_IF_TS */
        params
    "
)]
pub struct FunctionFormalParameters<'a, 'b>(pub &'b Function<'a>);

impl ESTree for FunctionFormalParameters<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();

        if S::INCLUDE_TS_FIELDS {
            if let Some(this_param) = &self.0.this_param {
                seq.serialize_element(this_param);
            }
        }

        for item in &self.0.params.items {
            seq.serialize_element(item);
        }

        if let Some(rest) = &self.0.params.rest {
            seq.serialize_element(&FormalParametersRest(rest));
        }

        seq.end();
    }
}

/// Serializer for `key` field of `MethodDefinition`.
///
/// In TS-ESTree `"constructor"` in `class C { "constructor"() {} }`
/// is represented as an `Identifier`.
/// In Acorn and Espree, it's a `Literal`.
/// <https://github.com/typescript-eslint/typescript-eslint/issues/11084>
#[ast_meta]
#[estree(
    ts_type = "PropertyKey",
    raw_deser = "
        /* IF_JS */
        DESER[PropertyKey](POS_OFFSET.key)
        /* END_IF_JS */

        /* IF_TS */
        let key = DESER[PropertyKey](POS_OFFSET.key);
        if (THIS.kind === 'constructor') {
            key = {
                type: 'Identifier',
                start: key.start,
                end: key.end,
                name: 'constructor',
                decorators: [],
                optional: false,
                typeAnnotation: null,
            };
        }
        key
        /* END_IF_TS */
    "
)]
pub struct MethodDefinitionKey<'a, 'b>(pub &'b MethodDefinition<'a>);

impl ESTree for MethodDefinitionKey<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let method = self.0;
        if S::INCLUDE_TS_FIELDS && method.kind == MethodDefinitionKind::Constructor {
            // `key` can only be either an identifier `constructor`, or string `"constructor"`
            let span = method.key.span();
            IdentifierName { span, name: Atom::from("constructor") }.serialize(serializer);
        } else {
            method.key.serialize(serializer);
        }
    }
}

/// Serializer for `specifiers` field of `ImportDeclaration`.
///
/// Serialize `specifiers` as an empty array if it's `None`.
#[ast_meta]
#[estree(
    ts_type = "Array<ImportDeclarationSpecifier>",
    raw_deser = "
        let specifiers = DESER[Option<Vec<ImportDeclarationSpecifier>>](POS_OFFSET.specifiers);
        if (specifiers === null) specifiers = [];
        specifiers
    "
)]
pub struct ImportDeclarationSpecifiers<'a, 'b>(pub &'b ImportDeclaration<'a>);

impl ESTree for ImportDeclarationSpecifiers<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(specifiers) = &self.0.specifiers {
            specifiers.serialize(serializer);
        } else {
            EmptyArray(()).serialize(serializer);
        }
    }
}

/// Serializer for `ArrowFunctionExpression`'s `body` field.
///
/// Serializes as either an expression (if `expression` property is set),
/// or a `BlockStatement` (if it's not).
#[ast_meta]
#[estree(
    ts_type = "FunctionBody | Expression",
    raw_deser = "
        let body = DESER[Box<FunctionBody>](POS_OFFSET.body);
        THIS.expression ? body.body[0].expression : body
    "
)]
pub struct ArrowFunctionExpressionBody<'a>(pub &'a ArrowFunctionExpression<'a>);

impl ESTree for ArrowFunctionExpressionBody<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(expression) = self.0.get_expression() {
            expression.serialize(serializer);
        } else {
            self.0.body.serialize(serializer);
        }
    }
}

/// Serializer for `AssignmentTargetPropertyIdentifier`'s `init` field
/// (which is renamed to `value` in ESTree AST).
#[ast_meta]
#[estree(
    ts_type = "IdentifierReference | AssignmentTargetWithDefault",
    raw_deser = "
        const init = DESER[Option<Expression>](POS_OFFSET.init),
            keyCopy = {...THIS.key},
            value = init === null
                ? keyCopy
                : {
                    type: 'AssignmentPattern',
                    start: THIS.start,
                    end: THIS.end,
                    left: keyCopy,
                    right: init,
                    /* IF_TS */
                    typeAnnotation: null,
                    optional: false,
                    decorators: [],
                    /* END_IF_TS */
                };
        value
    "
)]
pub struct AssignmentTargetPropertyIdentifierValue<'a>(
    pub &'a AssignmentTargetPropertyIdentifier<'a>,
);

impl ESTree for AssignmentTargetPropertyIdentifierValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(init) = &self.0.init {
            let mut state = serializer.serialize_struct();
            state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
            state.serialize_field("start", &self.0.span.start);
            state.serialize_field("end", &self.0.span.end);
            state.serialize_field("left", &self.0.binding);
            state.serialize_field("right", init);
            state.serialize_ts_field("typeAnnotation", &Null(()));
            state.serialize_ts_field("optional", &False(()));
            state.serialize_ts_field("decorators", &EmptyArray(()));
            state.end();
        } else {
            self.0.binding.serialize(serializer);
        }
    }
}

// Serializers for `with_clause` field of `ImportDeclaration`, `ExportNamedDeclaration`,
// and `ExportAllDeclaration` (which are renamed to `attributes` in ESTree AST).
//
// Serialize only the `with_entries` field of `WithClause`, and serialize `None` as empty array (`[]`).
//
// https://github.com/estree/estree/blob/master/es2025.md#importdeclaration
// https://github.com/estree/estree/blob/master/es2025.md#exportnameddeclaration

#[ast_meta]
#[estree(
    ts_type = "Array<ImportAttribute>",
    raw_deser = "
        const withClause = DESER[Option<Box<WithClause>>](POS_OFFSET.with_clause);
        withClause === null ? [] : withClause.withEntries
    "
)]
pub struct ImportDeclarationWithClause<'a, 'b>(pub &'b ImportDeclaration<'a>);

impl ESTree for ImportDeclarationWithClause<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(with_clause) = &self.0.with_clause {
            with_clause.with_entries.serialize(serializer);
        } else {
            EmptyArray(()).serialize(serializer);
        }
    }
}

#[ast_meta]
#[estree(
    ts_type = "Array<ImportAttribute>",
    raw_deser = "
        const withClause = DESER[Option<Box<WithClause>>](POS_OFFSET.with_clause);
        withClause === null ? [] : withClause.withEntries
    "
)]
pub struct ExportNamedDeclarationWithClause<'a, 'b>(pub &'b ExportNamedDeclaration<'a>);

impl ESTree for ExportNamedDeclarationWithClause<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(with_clause) = &self.0.with_clause {
            with_clause.with_entries.serialize(serializer);
        } else {
            EmptyArray(()).serialize(serializer);
        }
    }
}

#[ast_meta]
#[estree(
    ts_type = "Array<ImportAttribute>",
    raw_deser = "
        const withClause = DESER[Option<Box<WithClause>>](POS_OFFSET.with_clause);
        withClause === null ? [] : withClause.withEntries
    "
)]
pub struct ExportAllDeclarationWithClause<'a, 'b>(pub &'b ExportAllDeclaration<'a>);

impl ESTree for ExportAllDeclarationWithClause<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(with_clause) = &self.0.with_clause {
            with_clause.with_entries.serialize(serializer);
        } else {
            EmptyArray(()).serialize(serializer);
        }
    }
}

// --------------------
// JSX
// --------------------

/// Serializer for `opening_element` field of `JSXElement`.
///
/// `selfClosing` field of `JSXOpeningElement` depends on whether `JSXElement` has a `closing_element`.
#[ast_meta]
#[estree(
    ts_type = "JSXOpeningElement",
    raw_deser = "
        const openingElement = DESER[Box<JSXOpeningElement>](POS_OFFSET.opening_element);
        if (THIS.closingElement === null) openingElement.selfClosing = true;
        openingElement
    "
)]
pub struct JSXElementOpening<'a, 'b>(pub &'b JSXElement<'a>);

impl ESTree for JSXElementOpening<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let element = self.0;
        let opening_element = element.opening_element.as_ref();

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXOpeningElement"));
        state.serialize_field("start", &opening_element.span.start);
        state.serialize_field("end", &opening_element.span.end);
        state.serialize_field("attributes", &opening_element.attributes);
        state.serialize_field("name", &opening_element.name);
        state.serialize_field("selfClosing", &element.closing_element.is_none());
        state.serialize_ts_field("typeArguments", &opening_element.type_arguments);
        state.end();
    }
}

/// Converter for `selfClosing` field of `JSXOpeningElement`.
///
/// This converter is not used for serialization - `JSXElementOpening` above handles serialization.
/// This type is only required to add `selfClosing: boolean` to TS type def,
/// and provide default value of `false` for raw transfer deserializer.
#[ast_meta]
#[estree(ts_type = "boolean", raw_deser = "false")]
pub struct JSXOpeningElementSelfClosing<'a, 'b>(#[expect(dead_code)] pub &'b JSXOpeningElement<'a>);

impl ESTree for JSXOpeningElementSelfClosing<'_, '_> {
    fn serialize<S: Serializer>(&self, _serializer: S) {
        unreachable!()
    }
}

/// Serializer for `IdentifierReference` variant of `JSXElementName` and `JSXMemberExpressionObject`.
///
/// Convert to `JSXIdentifier`.
#[ast_meta]
#[estree(
    ts_type = "JSXIdentifier",
    raw_deser = "
        const ident = DESER[Box<IdentifierReference>](POS);
        {type: 'JSXIdentifier', start: ident.start, end: ident.end, name: ident.name}
    "
)]
pub struct JSXElementIdentifierReference<'a, 'b>(pub &'b IdentifierReference<'a>);

impl ESTree for JSXElementIdentifierReference<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JSXIdentifier { span: self.0.span, name: self.0.name }.serialize(serializer);
    }
}

/// Serializer for `ThisExpression` variant of `JSXElementName` and `JSXMemberExpressionObject`.
///
/// Convert to `JSXIdentifier`.
#[ast_meta]
#[estree(
    ts_type = "JSXIdentifier",
    raw_deser = "
        const thisExpr = DESER[Box<ThisExpression>](POS);
        {type: 'JSXIdentifier', start: thisExpr.start, end: thisExpr.end, name: 'this'}
    "
)]
pub struct JSXElementThisExpression<'b>(pub &'b ThisExpression);

impl ESTree for JSXElementThisExpression<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JSXIdentifier { span: self.0.span, name: Atom::from("this") }.serialize(serializer);
    }
}

/// Converter for `JSXOpeningFragment`.
///
/// Add `attributes` and `selfClosing` fields in JS AST, but not in TS AST.
/// Acorn-JSX has these fields, but TS-ESLint parser does not.
///
/// The extra fields are added to the type as `TsEmptyArray` and `TsFalse`,
/// which are incorrect, as these fields appear only in the *JS* AST, not the TS one.
/// But that results in the fields being optional in TS type definition.
//
// TODO: Find a better way to do this.
#[ast_meta]
#[estree(raw_deser = "
    const node = {
        type: 'JSXOpeningFragment',
        start: DESER[u32](POS_OFFSET.span.start),
        end: DESER[u32](POS_OFFSET.span.end),
        /* IF_JS */
        attributes: [],
        selfClosing: false,
        /* END_IF_JS */
    };
    node
")]
pub struct JSXOpeningFragmentConverter<'b>(pub &'b JSXOpeningFragment);

impl ESTree for JSXOpeningFragmentConverter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("JSXOpeningFragment"));
        state.serialize_field("start", &self.0.span.start);
        state.serialize_field("end", &self.0.span.end);
        if !S::INCLUDE_TS_FIELDS {
            state.serialize_field("attributes", &EmptyArray(()));
            state.serialize_field("selfClosing", &False(()));
        }
        state.end();
    }
}

// --------------------
// TS
// --------------------

/// Serializer for `computed` field of `TSEnumMember`.
///
/// `true` if `id` field is one of the computed variants of `TSEnumMemberName`.
//
// TODO: Not ideal to have to include the enum discriminant's value here explicitly.
// Need a "macro" e.g. `ENUM_MATCHES(id, ComputedString | ComputedTemplateString)`.
#[ast_meta]
#[estree(ts_type = "boolean", raw_deser = "DESER[u8](POS_OFFSET.id) > 1")]
pub struct TSEnumMemberComputed<'a, 'b>(pub &'b TSEnumMember<'a>);

impl ESTree for TSEnumMemberComputed<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        matches!(
            self.0.id,
            TSEnumMemberName::ComputedString(_) | TSEnumMemberName::ComputedTemplateString(_)
        )
        .serialize(serializer);
    }
}

/// Serializer for `directive` field of `ExpressionStatement`.
/// This field is always `null`, and only appears in the TS AST, not JS ESTree.
#[ast_meta]
#[estree(ts_type = "string | null", raw_deser = "null")]
#[ts]
pub struct ExpressionStatementDirective<'a, 'b>(
    #[expect(dead_code)] pub &'b ExpressionStatement<'a>,
);

impl ESTree for ExpressionStatementDirective<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        Null(()).serialize(serializer);
    }
}

/// Serializer for `global` field of `TSModuleDeclaration`.
#[ast_meta]
#[estree(ts_type = "boolean", raw_deser = "THIS.kind === 'global'")]
pub struct TSModuleDeclarationGlobal<'a, 'b>(pub &'b TSModuleDeclaration<'a>);

impl ESTree for TSModuleDeclarationGlobal<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.kind.is_global().serialize(serializer);
    }
}

/// Serializer for `key` and `constraint` field of `TSMappedType`.
#[ast_meta]
#[estree(
    ts_type = "TSTypeParameter['name']",
    raw_deser = "
        const typeParameter = DESER[Box<TSTypeParameter>](POS_OFFSET.type_parameter);
        typeParameter.name
    "
)]
pub struct TSMappedTypeKey<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeKey<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.type_parameter.name.serialize(serializer);
    }
}

// NOTE: Variable `typeParameter` in `raw_deser` is shared between `key` and `constraint` serializers.
// They will be concatenated in the generated code.
#[ast_meta]
#[estree(ts_type = "TSTypeParameter['constraint']", raw_deser = "typeParameter.constraint")]
pub struct TSMappedTypeConstraint<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeConstraint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.type_parameter.constraint.serialize(serializer);
    }
}

#[ast_meta]
#[estree(
    ts_type = "true | '+' | '-' | null",
    raw_deser = "
        const operator = DESER[u8](POS);
        [true, '+', '-', null][operator]
    "
)]
pub struct TSMappedTypeModifierOperatorConverter<'a>(pub &'a TSMappedTypeModifierOperator);

impl ESTree for TSMappedTypeModifierOperatorConverter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self.0 {
            TSMappedTypeModifierOperator::True => true.serialize(serializer),
            TSMappedTypeModifierOperator::Plus => JsonSafeString("+").serialize(serializer),
            TSMappedTypeModifierOperator::Minus => JsonSafeString("-").serialize(serializer),
            // This is typed as `undefined` (= key is not present) in TS-ESTree.
            // But we serialize it as `null` to align result in snapshot tests.
            TSMappedTypeModifierOperator::None => Null(()).serialize(serializer),
        }
    }
}

/// Serializer for `params` field of `TSCallSignatureDeclaration`.
///
/// These add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param)
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSCallSignatureDeclarationFormalParameters<'a, 'b>(
    pub &'b TSCallSignatureDeclaration<'a>,
);

impl ESTree for TSCallSignatureDeclarationFormalParameters<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let v = self.0;
        serialize_formal_params_with_this_param(v.this_param.as_ref(), &v.params, serializer);
    }
}

/// Serializer for `params` field of `TSMethodSignature`.
///
/// These add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param)
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSMethodSignatureFormalParameters<'a, 'b>(pub &'b TSMethodSignature<'a>);

impl ESTree for TSMethodSignatureFormalParameters<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let v = self.0;
        serialize_formal_params_with_this_param(v.this_param.as_deref(), &v.params, serializer);
    }
}

/// Serializer for `params` field of `TSFunctionType`.
///
/// These add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param)
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSFunctionTypeFormalParameters<'a, 'b>(pub &'b TSFunctionType<'a>);

impl ESTree for TSFunctionTypeFormalParameters<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let v = self.0;
        serialize_formal_params_with_this_param(v.this_param.as_deref(), &v.params, serializer);
    }
}

/// Shared serialization logic used by:
/// - `TSCallSignatureDeclarationFormalParameters`
/// - `TSMethodSignatureFormalParameters`
/// - `TSFunctionTypeFormalParameters`
fn serialize_formal_params_with_this_param<'a, S: Serializer>(
    this_param: Option<&TSThisParameter<'a>>,
    params: &FormalParameters<'a>,
    serializer: S,
) {
    let mut seq = serializer.serialize_sequence();

    if let Some(this_param) = this_param {
        seq.serialize_element(this_param);
    }

    for item in &params.items {
        seq.serialize_element(item);
    }

    if let Some(rest) = &params.rest {
        seq.serialize_element(&FormalParametersRest(rest));
    }

    seq.end();
}

// --------------------
// Comments
// --------------------

/// Serialize `value` field of `Comment`.
///
/// This serializer does not work for JSON serializer, because there's no access to source text
/// in `fn serialize`. But in any case, comments often contain characters which need escaping in JSON,
/// which is slow, so it's probably faster to transfer comments as NAPI types (which we do).
///
/// This meta type is only present for raw transfer, which can transfer faster.
#[ast_meta]
#[estree(
    ts_type = "string",
    raw_deser = "
        const endCut = THIS.type === 'Line' ? 0 : 2;
        SOURCE_TEXT.slice(THIS.start + 2, THIS.end - endCut)
    "
)]
pub struct CommentValue<'b>(#[expect(dead_code)] pub &'b Comment);

impl ESTree for CommentValue<'_> {
    #[expect(clippy::unimplemented)]
    fn serialize<S: Serializer>(&self, _serializer: S) {
        unimplemented!();
    }
}
