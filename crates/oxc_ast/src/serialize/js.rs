use oxc_ast_macros::ast_meta;
use oxc_estree::{
    Concat2, ConcatElement, ESTree, JsonSafeString, SequenceSerializer, Serializer,
    StructSerializer,
};
use oxc_span::GetSpan;

use crate::ast::*;

use super::{EmptyArray, Null};

// ----------------------------------------
// Binding patterns and function params
// ----------------------------------------

/// Converter for [`BindingPattern`].
///
/// Take `typeAnnotation` and `optional` fields from `BindingPattern`,
/// remaining fields from flattening `BindingPatternKind`.
#[ast_meta]
#[estree(raw_deser = "
    const pattern = DESER[BindingPatternKind](POS_OFFSET.kind);
    /* IF_TS */
    pattern.optional = DESER[bool](POS_OFFSET.optional);
    pattern.typeAnnotation = DESER[Option<Box<TSTypeAnnotation>>](POS_OFFSET.type_annotation);
    /* END_IF_TS */
    pattern
")]
pub struct BindingPatternConverter<'a, 'b>(pub &'b BindingPattern<'a>);

impl ESTree for BindingPatternConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let pattern = self.0;

        if S::INCLUDE_TS_FIELDS {
            BindingPatternKindAndTsFields {
                kind: &pattern.kind,
                decorators: &[],
                optional: pattern.optional,
                type_annotation: pattern.type_annotation.as_deref(),
            }
            .serialize(serializer);
        } else {
            pattern.kind.serialize(serializer);
        }
    }
}

struct BindingPatternKindAndTsFields<'a, 'b> {
    kind: &'b BindingPatternKind<'a>,
    decorators: &'b [Decorator<'a>],
    optional: bool,
    type_annotation: Option<&'b TSTypeAnnotation<'a>>,
}

impl ESTree for BindingPatternKindAndTsFields<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();

        match &self.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                state.serialize_field("type", &JsonSafeString("Identifier"));
                state.serialize_field("start", &ident.span.start);
                state.serialize_field("end", &ident.span.end);
                state.serialize_field("decorators", &self.decorators);
                state.serialize_field("name", &JsonSafeString(ident.name.as_str()));
            }
            BindingPatternKind::ObjectPattern(object) => {
                state.serialize_field("type", &JsonSafeString("ObjectPattern"));
                state.serialize_field("start", &object.span.start);
                state.serialize_field("end", &object.span.end);
                state.serialize_field("decorators", &self.decorators);
                state.serialize_field("properties", &Concat2(&object.properties, &object.rest));
            }
            BindingPatternKind::ArrayPattern(array) => {
                state.serialize_field("type", &JsonSafeString("ArrayPattern"));
                state.serialize_field("start", &array.span.start);
                state.serialize_field("end", &array.span.end);
                state.serialize_field("decorators", &self.decorators);
                state.serialize_field("elements", &Concat2(&array.elements, &array.rest));
            }
            BindingPatternKind::AssignmentPattern(assignment) => {
                state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
                state.serialize_field("start", &assignment.span.start);
                state.serialize_field("end", &assignment.span.end);
                state.serialize_field("decorators", &self.decorators);
                state.serialize_field("left", &assignment.left);
                state.serialize_field("right", &assignment.right);
            }
        }

        state.serialize_field("optional", &self.optional);
        state.serialize_field("typeAnnotation", &self.type_annotation);

        state.end();
    }
}

/// Converter for [`CatchParameter`].
///
/// Just delegate to [`BindingPattern`] serializer, ignoring `span` field.
///
/// We could do this just with `#[estree(skip)]` and `#[estree(flatten)]` on the Rust type def.
/// This converter only exists to generate more efficient raw deser code.
#[ast_meta]
#[estree(ts_type = "BindingPattern", raw_deser = "DESER[BindingPattern](POS_OFFSET.pattern)")]
pub struct CatchParameterConverter<'a, 'b>(pub &'b CatchParameter<'a>);

impl ESTree for CatchParameterConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.pattern.serialize(serializer);
    }
}

/// Converter for `FormalParameters`.
///
/// Combine `items` and `rest` fields. Convert `rest` field.
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
                /* IF_TS */
                decorators: [],
                /* END_IF_TS */
                argument: DESER[BindingPatternKind]( POS_OFFSET<BindingRestElement>.argument.kind ),
                /* IF_TS */
                optional: DESER[bool]( POS_OFFSET<BindingRestElement>.argument.optional ),
                typeAnnotation: DESER[Option<Box<TSTypeAnnotation>>](
                    POS_OFFSET<BindingRestElement>.argument.type_annotation
                ),
                value: null,
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
        self.0.push_to_sequence(&mut seq);
        seq.end();
    }
}

impl ConcatElement for FormalParameters<'_> {
    fn push_to_sequence<S: SequenceSerializer>(&self, seq: &mut S) {
        self.items.push_to_sequence(seq);
        if let Some(rest) = &self.rest {
            seq.serialize_element(&FormalParametersRest(rest));
        }
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
        state.serialize_ts_field("decorators", &EmptyArray(()));
        state.serialize_field("argument", &rest.argument.kind);
        state.serialize_ts_field("optional", &rest.argument.optional);
        state.serialize_ts_field("typeAnnotation", &rest.argument.type_annotation);
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
            param = DESER[BindingPatternKind](POS_OFFSET.pattern.kind);
            param.decorators = DESER[Vec<Decorator>](POS_OFFSET.decorators);
            param.optional = DESER[bool](POS_OFFSET.pattern.optional);
            param.typeAnnotation = DESER[Option<Box<TSTypeAnnotation>>](POS_OFFSET.pattern.type_annotation);
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
                state.serialize_field("static", &false);
                state.end();
            } else {
                BindingPatternKindAndTsFields {
                    kind: &param.pattern.kind,
                    decorators: &param.decorators,
                    optional: param.pattern.optional,
                    type_annotation: param.pattern.type_annotation.as_deref(),
                }
                .serialize(serializer);
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
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param);
        if (thisParam !== null) params.unshift(thisParam);
        /* END_IF_TS */
        params
    "
)]
pub struct FunctionParams<'a, 'b>(pub &'b Function<'a>);

impl ESTree for FunctionParams<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let func = self.0;
        if S::INCLUDE_TS_FIELDS {
            Concat2(&func.this_param, func.params.as_ref()).serialize(serializer);
        } else {
            func.params.serialize(serializer);
        }
    }
}

// ----------------------------------------
// Import / export
// ----------------------------------------

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
        withClause === null ? [] : withClause.attributes
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
        withClause === null ? [] : withClause.attributes
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
        withClause === null ? [] : withClause.attributes
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

// ----------------------------------------
// Misc
// ----------------------------------------

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
                decorators: [],
                name: 'constructor',
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

/// Serializer for `body` field of `ArrowFunctionExpression`.
///
/// Serialize as either an expression (if `expression` property is set),
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

/// Serializer for `init` field of `AssignmentTargetPropertyIdentifier`
/// (which is renamed to `value` in ESTree AST).
#[ast_meta]
#[estree(
    ts_type = "IdentifierReference | AssignmentTargetWithDefault",
    raw_deser = "
        const init = DESER[Option<Expression>](POS_OFFSET.init),
            keyCopy = { ...THIS.key },
            value = init === null
                ? keyCopy
                : {
                    type: 'AssignmentPattern',
                    start: THIS.start,
                    end: THIS.end,
                    /* IF_TS */
                    decorators: [],
                    /* END_IF_TS */
                    left: keyCopy,
                    right: init,
                    /* IF_TS */
                    optional: false,
                    typeAnnotation: null,
                    /* END_IF_TS */
                };
        value
    "
)]
pub struct AssignmentTargetPropertyIdentifierInit<'a>(
    pub &'a AssignmentTargetPropertyIdentifier<'a>,
);

impl ESTree for AssignmentTargetPropertyIdentifierInit<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(init) = &self.0.init {
            let mut state = serializer.serialize_struct();
            state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
            state.serialize_field("start", &self.0.span.start);
            state.serialize_field("end", &self.0.span.end);
            state.serialize_ts_field("decorators", &EmptyArray(()));
            state.serialize_field("left", &self.0.binding);
            state.serialize_field("right", init);
            state.serialize_ts_field("optional", &false);
            state.serialize_ts_field("typeAnnotation", &Null(()));
            state.end();
        } else {
            self.0.binding.serialize(serializer);
        }
    }
}
