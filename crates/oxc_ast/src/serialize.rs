use cow_utils::CowUtils;

use oxc_ast_macros::ast_meta;
use oxc_estree::{
    CompactJSSerializer, CompactTSSerializer, ESTree, JsonSafeString, PrettyJSSerializer,
    PrettyTSSerializer, SequenceSerializer, Serializer, StructSerializer,
};

use crate::ast::*;

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
// Basic types
// --------------------

/// Serialized as `null`.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
pub struct Null<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for Null<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
#[ts]
pub struct TsNull<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for TsNull<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

/// Serialized as `true`.
#[ast_meta]
#[estree(ts_type = "true", raw_deser = "true")]
pub struct True<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for True<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        true.serialize(serializer);
    }
}

/// Serialized as `false`.
#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false")]
pub struct False<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for False<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "false", raw_deser = "false")]
#[ts]
pub struct TsFalse<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for TsFalse<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `"in"`.
#[ast_meta]
#[estree(ts_type = "'in'", raw_deser = "'in'")]
pub struct In<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for In<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("in").serialize(serializer);
    }
}

/// Serialized as `"init"`.
#[ast_meta]
#[estree(ts_type = "'init'", raw_deser = "'init'")]
pub struct Init<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for Init<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("init").serialize(serializer);
    }
}

/// Serialized as `"this"`.
#[ast_meta]
#[estree(ts_type = "'this'", raw_deser = "'this'")]
pub struct This<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for This<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        JsonSafeString("this").serialize(serializer);
    }
}

/// Serialized as `[]`.
#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]")]
pub struct EmptyArray<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for EmptyArray<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        [(); 0].serialize(serializer);
    }
}

#[ast_meta]
#[estree(ts_type = "[]", raw_deser = "[]")]
#[ts]
pub struct TsEmptyArray<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for TsEmptyArray<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        [(); 0].serialize(serializer);
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
        ().serialize(serializer);
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
        ().serialize(serializer);
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

// --------------------
// Various
// --------------------

/// Serialize `ArrayExpressionElement::Elision` variant as `null`.
#[ast_meta]
#[estree(ts_type = "null", raw_deser = "null")]
pub struct ElisionConverter<'b>(#[expect(dead_code)] pub &'b Elision);

impl ESTree for ElisionConverter<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
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
        state.end();
    }
}

/// Serializer for `params` field of `Function`.
///
/// In TS AST, this adds `this_param` to start of the array.
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
            [(); 0].serialize(serializer);
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
            state.end();
        } else {
            self.0.binding.serialize(serializer);
        }
    }
}

/// Serializer for `options` field of `ImportExpression`.
///
/// Serialize only the first expression in `options`, or `null` if `options` is empty.
#[ast_meta]
#[estree(
    ts_type = "Expression | null",
    raw_deser = "
        const options = DESER[Vec<Expression>](POS_OFFSET.options);
        options.length === 0 ? null : options[0]
    "
)]
pub struct ImportExpressionOptions<'a>(pub &'a ImportExpression<'a>);

impl ESTree for ImportExpressionOptions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(expression) = self.0.options.first() {
            expression.serialize(serializer);
        } else {
            ().serialize(serializer);
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
            [(); 0].serialize(serializer);
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
            [(); 0].serialize(serializer);
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
            [(); 0].serialize(serializer);
        }
    }
}

// --------------------
// JSX
// --------------------

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

#[ast_meta]
#[estree(ts_type = "Array<JSXAttributeItem>", raw_deser = "[]")]
pub struct JSXOpeningFragmentAttributes<'b>(#[expect(dead_code)] pub &'b JSXOpeningFragment);

impl ESTree for JSXOpeningFragmentAttributes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        [(); 0].serialize(serializer);
    }
}

// --------------------
// TS
// --------------------

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
        ().serialize(serializer);
    }
}

/// Serializer for `implements` field of `Class`.
///
/// This field is only used in TS AST.
/// `None` is serialized as empty array (`[]`).
#[ast_meta]
#[estree(
    ts_type = "Array<TSClassImplements>",
    raw_deser = "
        const classImplements = DESER[Option<Vec<TSClassImplements>>](POS_OFFSET.implements);
        classImplements === null ? [] : classImplements
    "
)]
pub struct ClassImplements<'a, 'b>(pub &'b Class<'a>);

impl ESTree for ClassImplements<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(implements) = &self.0.implements {
            implements.serialize(serializer);
        } else {
            [(); 0].serialize(serializer);
        }
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
