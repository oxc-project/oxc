use oxc_ast_macros::ast_meta;
use oxc_estree::{
    Concat2, ConcatElement, ESTree, JsonSafeString, SequenceSerializer, Serializer,
    StructSerializer,
};
use oxc_span::{GetSpan, Span};

use crate::ast::*;

use super::{EmptyArray, Null};

#[ast_meta]
pub struct VariableDeclaratorConverter<'a, 'b>(pub &'b VariableDeclarator<'a>);

impl ESTree for VariableDeclaratorConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "VariableDeclarator");
        state.serialize_field(
            "id",
            &BindingPatternKindAndTsFields {
                kind: &self.0.id,
                decorators: Some(&[]),
                optional: false,
                type_annotation: self.0.type_annotation.as_deref(),
                override_span: None,
            },
        );
        state.serialize_field("init", &self.0.init);
        state.serialize_ts_field("definite", &self.0.definite);
        state.serialize_span(self.0.span);
        state.end();
    }
}
// ----------------------------------------
// Binding patterns and function params
// ----------------------------------------

struct BindingPatternKindAndTsFields<'a, 'b> {
    kind: &'b BindingPattern<'a>,
    decorators: Option<&'b [Decorator<'a>]>,
    optional: bool,
    type_annotation: Option<&'b TSTypeAnnotation<'a>>,
    /// Override span to use instead of computing from pattern.
    /// Used for optional parameters where span needs to include `?` token.
    override_span: Option<Span>,
}

impl ESTree for BindingPatternKindAndTsFields<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();

        let mut span = match &self.kind {
            BindingPattern::BindingIdentifier(ident) => {
                state.serialize_field("type", &JsonSafeString("Identifier"));
                if let Some(d) = &self.decorators {
                    state.serialize_ts_field("decorators", d);
                }
                state.serialize_field("name", &JsonSafeString(ident.name.as_str()));
                ident.span
            }
            BindingPattern::ObjectPattern(object) => {
                state.serialize_field("type", &JsonSafeString("ObjectPattern"));
                if let Some(d) = &self.decorators {
                    state.serialize_ts_field("decorators", d);
                }
                state.serialize_field("properties", &Concat2(&object.properties, &object.rest));
                object.span
            }
            BindingPattern::ArrayPattern(array) => {
                state.serialize_field("type", &JsonSafeString("ArrayPattern"));
                if let Some(d) = &self.decorators {
                    state.serialize_ts_field("decorators", d);
                }
                state.serialize_field("elements", &Concat2(&array.elements, &array.rest));
                array.span
            }
            BindingPattern::AssignmentPattern(assignment) => {
                state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
                if let Some(d) = &self.decorators {
                    state.serialize_ts_field("decorators", d);
                }
                // Serialize left with decorators in TS mode
                state.serialize_field(
                    "left",
                    &BindingPatternKindAndTsFields {
                        kind: &assignment.left,
                        decorators: Some(&[]),
                        optional: false,
                        type_annotation: None,
                        override_span: None,
                    },
                );
                state.serialize_field("right", &assignment.right);
                assignment.span
            }
        };

        state.serialize_ts_field("optional", &self.optional);
        state.serialize_ts_field("typeAnnotation", &self.type_annotation);

        // Use override span if provided (which already includes type annotation),
        // otherwise extend pattern span to include type annotation
        if let Some(override_span) = self.override_span {
            span = override_span;
        } else if let Some(type_annotation) = self.type_annotation {
            span = span.merge(type_annotation.span);
        }

        state.serialize_span(span);

        state.end();
    }
}

/// Converter for [`CatchParameter`].
///
/// Serializes as the pattern with type annotation if in TS mode.
#[ast_meta]
#[estree(
    ts_type = "BindingPattern",
    raw_deser = "
        const pattern = DESER[BindingPattern](POS_OFFSET.pattern);
        if (IS_TS) {
            pattern.typeAnnotation = DESER[Option<Box<TSTypeAnnotation>>](POS_OFFSET.type_annotation);
        }
        pattern
    "
)]
pub struct CatchParameterConverter<'a, 'b>(pub &'b CatchParameter<'a>);

impl ESTree for CatchParameterConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        BindingPatternKindAndTsFields {
            kind: &self.0.pattern,
            decorators: Some(&[]),
            optional: false,
            type_annotation: self.0.type_annotation.as_deref(),
            override_span: None,
        }
        .serialize(serializer);
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

            let start, end;
            const previousParent = parent;
            const rest = parent = {
                type: 'RestElement',
                ...(IS_TS && { decorators: [] }),
                argument: null,
                ...(IS_TS && {
                    optional: false,
                    typeAnnotation: null,
                    value: null,
                }),
                start: start = DESER[u32]( POS_OFFSET<FormalParameterRest>.span.start ),
                end: end = DESER[u32]( POS_OFFSET<FormalParameterRest>.span.end ),
                ...(RANGE && { range: [start, end] }),
                ...(PARENT && { parent }),
            };
            rest.argument = DESER[BindingPattern]( POS_OFFSET<FormalParameterRest>.rest.argument );
            if (IS_TS) {
                rest.typeAnnotation = DESER[Option<Box<TSTypeAnnotation>>](
                    POS_OFFSET<FormalParameterRest>.type_annotation
                );
            }
            params.push(rest);
            if (PARENT) parent = previousParent;
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
            seq.serialize_element(rest.as_ref());
        }
    }
}

impl ESTree for FormalParameterRest<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let rest = self;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("RestElement"));
        state.serialize_ts_field("decorators", &EmptyArray(()));
        state.serialize_field("argument", &rest.rest.argument);
        state.serialize_ts_field("optional", &false);
        state.serialize_ts_field("typeAnnotation", &rest.type_annotation);
        state.serialize_ts_field("value", &Null(()));
        // Use the BindingRestElement span which includes the `...` token
        state.serialize_span(
            rest.type_annotation.as_ref().map_or(rest.rest.span, |ta| rest.span.merge(ta.span)),
        );
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
        let param;
        if (IS_TS) {
            const accessibility = DESER[Option<TSAccessibility>](POS_OFFSET.accessibility),
                readonly = DESER[bool](POS_OFFSET.readonly),
                override = DESER[bool](POS_OFFSET.override),
                previousParent = parent;
            if (accessibility === null && !readonly && !override) {
                param = parent = DESER[BindingPattern](POS_OFFSET.pattern);
                param.decorators = DESER[Vec<Decorator>](POS_OFFSET.decorators);
                param.optional = DESER[bool](POS_OFFSET.optional);
                param.typeAnnotation = DESER[Option<Box<TSTypeAnnotation>>](POS_OFFSET.type_annotation);
            } else {
                let start, end;
                param = parent = {
                    type: 'TSParameterProperty',
                    accessibility,
                    decorators: null,
                    override,
                    parameter: null,
                    readonly,
                    static: false,
                    start: start = DESER[u32]( POS_OFFSET.span.start ),
                    end: end = DESER[u32]( POS_OFFSET.span.end ),
                    ...(RANGE && { range: [start, end] }),
                    ...(PARENT && { parent }),
                };
                param.decorators = DESER[Vec<Decorator>](POS_OFFSET.decorators);
                param.parameter = DESER[BindingPattern](POS_OFFSET.pattern);
            }
            if (PARENT) parent = previousParent;
        } else {
            param = DESER[BindingPattern](POS_OFFSET.pattern);
        }
        param
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
                state.serialize_field("accessibility", &param.accessibility);
                state.serialize_field("decorators", &param.decorators);
                state.serialize_field("override", &param.r#override);

                // If there's an initializer, wrap pattern in AssignmentPattern for the parameter field
                if let Some(init) = &param.initializer {
                    // For TSParameterProperty with initializer, we need to compute spans carefully
                    // to exclude modifiers from the nested parameter spans
                    let pattern_span = param.pattern.span();
                    let left_span_end =
                        param.type_annotation.as_ref().map_or(pattern_span.end, |ta| ta.span.end);
                    let assignment_span = Span::new(pattern_span.start, init.span().end);

                    state.serialize_field(
                        "parameter",
                        &TSParameterPropertyAssignmentPattern {
                            param,
                            init,
                            left_span: Span::new(pattern_span.start, left_span_end),
                            assignment_span,
                        },
                    );
                } else {
                    // For TSParameterProperty, the parameter's span should start from the pattern,
                    // not from the modifiers
                    let override_span = if param.optional {
                        let pattern_span = param.pattern.span();
                        // Compute span from pattern start to type annotation end (or param end if no type annotation)
                        let end =
                            param.type_annotation.as_ref().map_or(param.span.end, |ta| ta.span.end);
                        Some(Span::new(pattern_span.start, end))
                    } else {
                        None
                    };
                    state.serialize_field(
                        "parameter",
                        &BindingPatternKindAndTsFields {
                            kind: &param.pattern,
                            decorators: Some(&[]),
                            optional: param.optional,
                            type_annotation: param.type_annotation.as_deref(),
                            override_span,
                        },
                    );
                }

                state.serialize_field("readonly", &param.readonly);
                state.serialize_field("static", &false);
                state.serialize_span(param.span);
                state.end();
            } else {
                // If there's an initializer, serialize as AssignmentPattern
                if let Some(init) = &param.initializer {
                    let mut state = serializer.serialize_struct();
                    state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
                    state.serialize_field("decorators", &param.decorators);
                    state.serialize_field(
                        "left",
                        &BindingPatternKindAndTsFields {
                            kind: &param.pattern,
                            decorators: Some(&[]),
                            optional: false,
                            type_annotation: param.type_annotation.as_deref(),
                            override_span: None,
                        },
                    );
                    state.serialize_field("right", init);
                    state.serialize_field("optional", &param.optional);
                    state.serialize_field("typeAnnotation", &Null(()));
                    state.serialize_span(param.span);
                    state.end();
                } else {
                    BindingPatternKindAndTsFields {
                        kind: &param.pattern,
                        decorators: Some(&param.decorators),
                        optional: param.optional,
                        type_annotation: param.type_annotation.as_deref(),
                        // Use param.span to include the `?` token when optional.
                        // When not optional, let the normal span extension logic handle type annotation.
                        override_span: if param.optional { Some(param.span) } else { None },
                    }
                    .serialize(serializer);
                }
            }
        } else {
            // Non-TS mode: If there's an initializer, serialize as AssignmentPattern
            if let Some(init) = &param.initializer {
                let mut state = serializer.serialize_struct();
                state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
                state.serialize_field("left", &param.pattern);
                state.serialize_field("right", init);
                state.serialize_span(param.span);
                state.end();
            } else {
                param.pattern.serialize(serializer);
            }
        }
    }
}

/// Helper for serializing TSParameterProperty's parameter field when it has an initializer
struct TSParameterPropertyAssignmentPattern<'a, 'b> {
    param: &'b FormalParameter<'a>,
    init: &'b Expression<'a>,
    left_span: Span,
    assignment_span: Span,
}

impl ESTree for TSParameterPropertyAssignmentPattern<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("AssignmentPattern"));
        state.serialize_field("decorators", &EmptyArray(()));
        state.serialize_field(
            "left",
            &BindingPatternKindAndTsFields {
                kind: &self.param.pattern,
                decorators: Some(&[]),
                optional: false,
                type_annotation: self.param.type_annotation.as_deref(),
                override_span: Some(self.left_span),
            },
        );
        state.serialize_field("right", self.init);
        state.serialize_field("optional", &false);
        state.serialize_field("typeAnnotation", &Null(()));
        state.serialize_span(self.assignment_span);
        state.end();
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
        if (IS_TS) {
            const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param);
            if (thisParam !== null) params.unshift(thisParam);
        }
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

/// Serializer for `body` field of `ArrowFunctionExpression`.
///
/// Serialize as either an expression (if `expression` property is set),
/// or a `BlockStatement` (if it's not).
#[ast_meta]
#[estree(
    ts_type = "FunctionBody | Expression",
    raw_deser = "
        let body = DESER[Box<FunctionBody>](POS_OFFSET.body);
        if (THIS.expression === true) {
            body = body.body[0].expression;
            if (PARENT) body.parent = parent;
        }
        body
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
        // Clone `key`
        let keyStart, keyEnd;
        let value = {
            type: 'Identifier',
            ...(IS_TS && { decorators: [] }),
            name: THIS.key.name,
            ...(IS_TS && {
                optional: false,
                typeAnnotation: null,
            }),
            start: keyStart = THIS.key.start,
            end: keyEnd = THIS.key.end,
            ...(RANGE && { range: [keyStart, keyEnd] }),
            ...(PARENT && { parent }),
        };
        const init = DESER[Option<Expression>](POS_OFFSET.init);
        if (init !== null) {
            const left = value;
            value = {
                type: 'AssignmentPattern',
                ...(IS_TS && { decorators: [] }),
                left,
                right: init,
                ...(IS_TS && {
                    optional: false,
                    typeAnnotation: null,
                }),
                start: THIS.start,
                end: THIS.end,
                ...(RANGE && { range: [THIS.start, THIS.end] }),
                ...(PARENT && { parent }),
            };
            if (PARENT) {
                left.parent = value;
                init.parent = value;
            }
        }
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
            state.serialize_ts_field("decorators", &EmptyArray(()));
            state.serialize_field("left", &self.0.binding);
            state.serialize_field("right", init);
            state.serialize_ts_field("optional", &false);
            state.serialize_ts_field("typeAnnotation", &Null(()));
            state.serialize_span(self.0.span);
            state.end();
        } else {
            self.0.binding.serialize(serializer);
        }
    }
}

/// Converter for [`ParenthesizedExpression`].
///
/// In raw transfer, do not produce a `ParenthesizedExpression` node in AST if `PRESERVE_PARENS` is false.
///
/// Not useful in `oxc-parser`, as can use parser option `preserve_parens`.
/// Required for `oxlint` plugins where we run parser with `preserve_parens` set to `true`,
/// to preserve them on Rust side, but need to remove them on JS side.
///
/// ESTree implementation is unchanged from the auto-generated version.
#[ast_meta]
#[estree(raw_deser = "
    let node;
    if (PRESERVE_PARENS) {
        let start, end;
        const previousParent = parent;
        node = parent = {
            type: 'ParenthesizedExpression',
            expression: null,
            start: start = DESER[u32]( POS_OFFSET.span.start ),
            end: end = DESER[u32]( POS_OFFSET.span.end ),
            ...(RANGE && { range: [start, end] }),
            ...(PARENT && { parent }),
        };
        node.expression = DESER[Expression](POS_OFFSET.expression);
        if (PARENT) parent = previousParent;
    } else {
        node = DESER[Expression](POS_OFFSET.expression);
    }
    node
")]
pub struct ParenthesizedExpressionConverter<'a, 'b>(pub &'b ParenthesizedExpression<'a>);

impl ESTree for ParenthesizedExpressionConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let paren_expr = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("ParenthesizedExpression"));
        state.serialize_field("expression", &paren_expr.expression);
        state.serialize_span(paren_expr.span);
        state.end();
    }
}
