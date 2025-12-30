use oxc_ast_macros::ast_meta;
use oxc_estree::{Concat2, ESTree, JsonSafeString, Serializer, StructSerializer};

use crate::ast::*;

use super::Null;

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
///
/// This field is always `null`, and only appears in the TS-ESTree AST, not JS ESTree.
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

/// Converter for `TSModuleDeclaration`.
///
/// Our AST represents `module X.Y.Z {}` as 3 x nested `TSModuleDeclaration`s.
/// TS-ESTree represents it as a single `TSModuleDeclaration`,
/// with a nested tree of `TSQualifiedName`s as `id`.
#[ast_meta]
#[estree(raw_deser = "
    const kind = DESER[TSModuleDeclarationKind](POS_OFFSET.kind),
        start = DESER[u32](POS_OFFSET.span.start),
        end = DESER[u32](POS_OFFSET.span.end),
        declare = DESER[bool](POS_OFFSET.declare);

    let node;
    const previousParent = parent;

    let body = DESER[Option<TSModuleDeclarationBody>](POS_OFFSET.body);
    if (body === null) {
        node = parent = {
            type: 'TSModuleDeclaration',
            id: null,
            // No `body` field
            kind,
            declare,
            global: false,
            start,
            end,
            ...(RANGE && { range: [start, end] }),
            ...(PARENT && { parent }),
        };
        node.id = DESER[TSModuleDeclarationName](POS_OFFSET.id);
    } else {
        node = parent = {
            type: 'TSModuleDeclaration',
            id: null,
            body,
            kind,
            declare,
            global: false,
            start,
            end,
            ...(RANGE && { range: [start, end] }),
            ...(PARENT && { parent }),
        };

        const id = DESER[TSModuleDeclarationName](POS_OFFSET.id);

        if (body.type === 'TSModuleBlock') {
            node.id = id;
            if (PARENT) body.parent = node;
        } else {
            let innerId = body.id;
            if (innerId.type === 'Identifier') {
                let start, end;
                const outerId = node.id = parent = {
                    type: 'TSQualifiedName',
                    left: id,
                    right: innerId,
                    start: start = id.start,
                    end: end = innerId.end,
                    ...(RANGE && { range: [start, end] }),
                    ...(PARENT && { parent: node }),
                };
                if (PARENT) id.parent = innerId.parent = outerId;
            } else {
                // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with `id` of
                // this module on left, and previous `left` of innermost `TSQualifiedName` on right
                node.id = innerId;
                if (PARENT) innerId.parent = node;

                const { start } = id;
                while (true) {
                    if (RANGE) {
                        innerId.start = innerId.range[0] = start;
                    } else {
                        innerId.start = start;
                    }
                    if (innerId.left.type === 'Identifier') break;
                    innerId = innerId.left;
                }

                let end;
                const right = innerId.left;
                const left = innerId.left = {
                    type: 'TSQualifiedName',
                    left: id,
                    right,
                    start,
                    end: end = right.end,
                    ...(RANGE && { range: [start, end] }),
                    ...(PARENT && { parent: innerId }),
                };
                if (PARENT) id.parent = right.parent = left;
            }

            if (Object.hasOwn(body, 'body')) {
                body = body.body;
                node.body = body;
                if (PARENT) body.parent = node;
            } else {
                body = null;
            }
        }
    }

    if (PARENT) parent = previousParent;

    node
")]
pub struct TSModuleDeclarationConverter<'a, 'b>(pub &'b TSModuleDeclaration<'a>);

impl ESTree for TSModuleDeclarationConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let module = self.0;

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSModuleDeclaration"));

        match &module.body {
            Some(TSModuleDeclarationBody::TSModuleDeclaration(inner_module)) => {
                // Nested modules e.g. `module X.Y.Z {}`.
                // Collect all IDs in a `Vec`, in order they appear (i.e. [`X`, `Y`, `Z`]).
                // Also get the inner `TSModuleBlock`.
                let mut parts = Vec::with_capacity(4);

                let TSModuleDeclarationName::Identifier(id) = &module.id else { unreachable!() };
                parts.push(id);

                let mut body = None;
                let mut inner_module = inner_module.as_ref();
                loop {
                    let TSModuleDeclarationName::Identifier(id) = &inner_module.id else {
                        unreachable!()
                    };
                    parts.push(id);

                    match &inner_module.body {
                        Some(TSModuleDeclarationBody::TSModuleDeclaration(inner_inner_module)) => {
                            inner_module = inner_inner_module.as_ref();
                        }
                        Some(TSModuleDeclarationBody::TSModuleBlock(block)) => {
                            body = Some(block.as_ref());
                            break;
                        }
                        None => break,
                    }
                }

                // Serialize `parts` as a nested tree of `TSQualifiedName`s
                state.serialize_field("id", &TSModuleDeclarationIdParts(&parts));

                // Skip `body` field if it's `None`
                if let Some(body) = body {
                    state.serialize_field("body", body);
                }
            }
            Some(TSModuleDeclarationBody::TSModuleBlock(block)) => {
                // No nested modules.
                // Serialize as usual, with `id` being either a `BindingIdentifier` or `StringLiteral`.
                state.serialize_field("id", &module.id);
                state.serialize_field("body", block);
            }
            None => {
                // No body. Skip `body` field.
                state.serialize_field("id", &module.id);
            }
        }

        state.serialize_field("kind", &module.kind);
        state.serialize_field("declare", &module.declare);
        state.serialize_field("global", &false);

        state.serialize_span(module.span);

        state.end();
    }
}

struct TSModuleDeclarationIdParts<'a, 'b>(&'b [&'b BindingIdentifier<'a>]);

impl ESTree for TSModuleDeclarationIdParts<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let parts = self.0;
        assert!(!parts.is_empty());

        let (&last, rest) = parts.split_last().unwrap();

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSQualifiedName"));

        if rest.len() == 1 {
            // Only one part remaining (e.g. `X`). Serialize as `Identifier`.
            state.serialize_field("left", &rest[0]);
        } else {
            // Multiple parts remaining (e.g. `X.Y`). Recurse to serialize as `TSQualifiedName`.
            state.serialize_field("left", &TSModuleDeclarationIdParts(rest));
        }

        state.serialize_field("right", last);

        let span = Span::new(parts[0].span.start, last.span.end);
        state.serialize_span(span);

        state.end();
    }
}

/// Serializer for `id` field of `TSGlobalDeclaration`.
///
/// Contains an identifier `global`, with the span from `global_span` field.
#[ast_meta]
#[estree(
    ts_type = "IdentifierName",
    raw_deser = "
        let keywordStart, keywordEnd;
        const ident = {
            type: 'Identifier',
            ...(IS_TS && { decorators: [] }),
            name: 'global',
            ...(IS_TS && {
                optional: false,
                typeAnnotation: null,
            }),
            start: keywordStart = DESER[u32](POS_OFFSET.global_span.start),
            end: keywordEnd = DESER[u32](POS_OFFSET.global_span.end),
            ...(RANGE && { range: [keywordStart, keywordEnd] }),
            ...(PARENT && { parent }),
        };
        ident
    "
)]
pub struct TSGlobalDeclarationId<'a, 'b>(pub &'b TSGlobalDeclaration<'a>);

impl ESTree for TSGlobalDeclarationId<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let ident = IdentifierName { span: self.0.global_span, name: Atom::from("global") };
        ident.serialize(serializer);
    }
}

/// Serializer for `optional` field of `TSMappedType`.
///
/// `None` is serialized as `false`.
#[ast_meta]
#[estree(
    ts_type = "TSMappedTypeModifierOperator | false",
    raw_deser = "
        let optional = DESER[Option<TSMappedTypeModifierOperator>](POS_OFFSET.optional);
        if (optional === null) optional = false;
        optional
    "
)]
pub struct TSMappedTypeOptional<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeOptional<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(optional) = self.0.optional {
            optional.serialize(serializer);
        } else {
            false.serialize(serializer);
        }
    }
}

/// Serializer for `key` field of `TSMappedType`.
#[ast_meta]
#[estree(
    ts_type = "TSTypeParameter['name']",
    raw_deser = "
        const typeParameter = DESER[Box<TSTypeParameter>](POS_OFFSET.type_parameter),
            key = typeParameter.name;
        if (PARENT) key.parent = parent;
        key
    "
)]
pub struct TSMappedTypeKey<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeKey<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.type_parameter.name.serialize(serializer);
    }
}

/// Serializer for `constraint` field of `TSMappedType`.
///
/// NOTE: Variable `typeParameter` in `raw_deser` is shared between `key` and `constraint` serializers.
/// They will be concatenated in the generated code.
#[ast_meta]
#[estree(
    ts_type = "TSTypeParameter['constraint']",
    raw_deser = "
        const { constraint } = typeParameter;
        if (PARENT && constraint !== null) constraint.parent = parent;
        constraint
    "
)]
pub struct TSMappedTypeConstraint<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeConstraint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.type_parameter.constraint.serialize(serializer);
    }
}

/// Serializer for `expression` field of `TSClassImplements`.
///
/// Our AST represents `X.Y` in `class C implements X.Y {}` as a `TSQualifiedName`.
/// TS-ESTree represents `X.Y` as a `MemberExpression`.
///
/// Where there are more parts e.g. `class C implements X.Y.Z {}`, the `TSQualifiedName`s (Oxc)
/// or `MemberExpression`s (TS-ESTree) are nested.
#[ast_meta]
#[estree(
    ts_type = "IdentifierReference | ThisExpression | MemberExpression",
    raw_deser = "
        let expression = DESER[TSTypeName](POS_OFFSET.expression);
        if (expression.type === 'TSQualifiedName') {
            let object = expression.left;
            const { right } = expression;
            let start, end;
            let previous = expression = {
                type: 'MemberExpression',
                object,
                property: right,
                optional: false,
                computed: false,
                start: start = expression.start,
                end: end = expression.end,
                ...(RANGE && { range: [start, end] }),
                ...(PARENT && { parent }),
            };

            if (PARENT) right.parent = previous;

            while (true) {
                if (object.type !== 'TSQualifiedName') {
                    if (PARENT) object.parent = previous;
                    break;
                }

                const { left, right } = object;
                previous = previous.object = {
                    type: 'MemberExpression',
                    object: left,
                    property: right,
                    optional: false,
                    computed: false,
                    start: start = object.start,
                    end: end = object.end,
                    ...(RANGE && { range: [start, end] }),
                    ...(PARENT && { parent: previous }),
                };

                if (PARENT) right.parent = previous;

                object = left;
            }
        }
        expression
    "
)]
pub struct TSClassImplementsExpression<'a, 'b>(pub &'b TSClassImplements<'a>);

impl ESTree for TSClassImplementsExpression<'_, '_> {
    #[inline] // Because it just delegates
    fn serialize<S: Serializer>(&self, serializer: S) {
        TSTypeNameAsMemberExpression(&self.0.expression).serialize(serializer);
    }
}

struct TSTypeNameAsMemberExpression<'a, 'b>(&'b TSTypeName<'a>);

impl ESTree for TSTypeNameAsMemberExpression<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self.0 {
            TSTypeName::IdentifierReference(ident) => {
                ident.serialize(serializer);
            }
            TSTypeName::QualifiedName(name) => {
                // Convert to `TSQualifiedName` to `MemberExpression`.
                // Recursively convert `left` to `MemberExpression` too if it's a `TSQualifiedName`.
                let mut state = serializer.serialize_struct();
                state.serialize_field("type", &JsonSafeString("MemberExpression"));
                state.serialize_field("object", &TSTypeNameAsMemberExpression(&name.left));
                state.serialize_field("property", &name.right);
                state.serialize_field("optional", &false);
                state.serialize_field("computed", &false);
                state.serialize_span(name.span);
                state.end();
            }
            TSTypeName::ThisExpression(e) => {
                e.serialize(serializer);
            }
        }
    }
}

/// Serializer for `params` field of `TSCallSignatureDeclaration`.
///
/// Add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param);
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSCallSignatureDeclarationParams<'a, 'b>(pub &'b TSCallSignatureDeclaration<'a>);

impl ESTree for TSCallSignatureDeclarationParams<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let decl = self.0;
        Concat2(&decl.this_param, decl.params.as_ref()).serialize(serializer);
    }
}

/// Serializer for `params` field of `TSMethodSignature`.
///
/// Add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param);
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSMethodSignatureParams<'a, 'b>(pub &'b TSMethodSignature<'a>);

impl ESTree for TSMethodSignatureParams<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let sig = self.0;
        Concat2(&sig.this_param, sig.params.as_ref()).serialize(serializer);
    }
}

/// Serializer for `params` field of `TSFunctionType`.
///
/// Add `this_param` to start of the `params` array.
#[ast_meta]
#[estree(
    ts_type = "ParamPattern[]",
    raw_deser = "
        const params = DESER[Box<FormalParameters>](POS_OFFSET.params);
        const thisParam = DESER[Option<Box<TSThisParameter>>](POS_OFFSET.this_param);
        if (thisParam !== null) params.unshift(thisParam);
        params
    "
)]
pub struct TSFunctionTypeParams<'a, 'b>(pub &'b TSFunctionType<'a>);

impl ESTree for TSFunctionTypeParams<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let fn_type = self.0;
        Concat2(&fn_type.this_param, fn_type.params.as_ref()).serialize(serializer);
    }
}

/// Converter for [`TSParenthesizedType`].
///
/// In raw transfer, do not produce a `TSParenthesizedType` node in AST if `PRESERVE_PARENS` is false.
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
            type: 'TSParenthesizedType',
            typeAnnotation: null,
            start: start = DESER[u32]( POS_OFFSET.span.start ),
            end: end = DESER[u32]( POS_OFFSET.span.end ),
            ...(RANGE && { range: [start, end] }),
            ...(PARENT && { parent }),
        };
        node.typeAnnotation = DESER[TSType](POS_OFFSET.type_annotation);
        if (PARENT) parent = previousParent;
    } else {
        node = DESER[TSType](POS_OFFSET.type_annotation);
    }
    node
")]
pub struct TSParenthesizedTypeConverter<'a, 'b>(pub &'b TSParenthesizedType<'a>);

impl ESTree for TSParenthesizedTypeConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let paren_type = self.0;
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSParenthesizedType"));
        state.serialize_field("typeAnnotation", &paren_type.type_annotation);
        state.serialize_span(paren_type.span);
        state.end();
    }
}

// ----------------------------------------
// FromESTreeConverter implementations
// ----------------------------------------
// These are gated by the `deserialize` feature since they depend on the deserialize module.

#[cfg(feature = "deserialize")]
mod from_estree_converters {
    use crate::ast::{js, ts};
    use crate::deserialize::{DeserError, DeserResult, FromESTree, FromESTreeConverter};
    use oxc_allocator::{Allocator, Box as ABox, Vec as AVec};

    use super::{
        TSCallSignatureDeclarationParams, TSClassImplementsExpression, TSFunctionTypeParams,
        TSMappedTypeOptional, TSMethodSignatureParams,
    };

    /// Deserialize `expression` field for `TSClassImplements`.
    ///
    /// ESTree serializes as a member expression (Identifier or MemberExpression),
    /// but oxc uses TSTypeName. We need to reverse the conversion.
    impl<'a> FromESTreeConverter<'a> for TSClassImplementsExpression<'a, '_> {
        type Output = ts::TSTypeName<'a>;

        fn from_estree_converter(
            value: &serde_json::Value,
            allocator: &'a Allocator,
        ) -> DeserResult<Self::Output> {
            // ESTree represents as Identifier or MemberExpression
            // We need to convert to TSTypeName
            let type_str = value.get("type").and_then(|t| t.as_str());
            match type_str {
                Some("Identifier") => {
                    let ident: js::IdentifierReference = FromESTree::from_estree(value, allocator)?;
                    Ok(ts::TSTypeName::IdentifierReference(ABox::new_in(ident, allocator)))
                }
                Some("MemberExpression") => {
                    // Convert MemberExpression back to TSQualifiedName
                    let qualified = convert_member_expr_to_qualified_name(value, allocator)?;
                    Ok(ts::TSTypeName::QualifiedName(ABox::new_in(qualified, allocator)))
                }
                _ => Err(DeserError::UnknownNodeType(type_str.unwrap_or("unknown").to_string())),
            }
        }
    }

    /// Helper to convert ESTree MemberExpression to TSQualifiedName
    fn convert_member_expr_to_qualified_name<'a>(
        value: &serde_json::Value,
        allocator: &'a Allocator,
    ) -> DeserResult<ts::TSQualifiedName<'a>> {
        let object = value.get("object").ok_or(DeserError::MissingField("object"))?;
        let property = value.get("property").ok_or(DeserError::MissingField("property"))?;

        // Property is always an Identifier -> IdentifierName
        let right: js::IdentifierName = FromESTree::from_estree(property, allocator)?;

        // Object can be Identifier or MemberExpression
        let object_type = object.get("type").and_then(|t| t.as_str());
        let left = match object_type {
            Some("Identifier") => {
                let ident: js::IdentifierReference = FromESTree::from_estree(object, allocator)?;
                ts::TSTypeName::IdentifierReference(ABox::new_in(ident, allocator))
            }
            Some("MemberExpression") => {
                let qualified = convert_member_expr_to_qualified_name(object, allocator)?;
                ts::TSTypeName::QualifiedName(ABox::new_in(qualified, allocator))
            }
            _ => {
                return Err(DeserError::UnknownNodeType(
                    object_type.unwrap_or("unknown").to_string(),
                ));
            }
        };

        let span = crate::deserialize::parse_span_or_empty(value);
        Ok(ts::TSQualifiedName { span, left, right })
    }

    /// Deserialize `params` field for `TSCallSignatureDeclaration`.
    ///
    /// Similar to FunctionParams - takes array and produces FormalParameters.
    impl<'a> FromESTreeConverter<'a> for TSCallSignatureDeclarationParams<'a, '_> {
        type Output = ABox<'a, js::FormalParameters<'a>>;

        fn from_estree_converter(
            value: &serde_json::Value,
            allocator: &'a Allocator,
        ) -> DeserResult<Self::Output> {
            let arr = value.as_array().ok_or(DeserError::ExpectedArray)?;

            let mut items = AVec::with_capacity_in(arr.len(), allocator);
            for item in arr {
                // Skip TSThisParameter if present
                let item_type = item.get("type").and_then(|t| t.as_str());
                if item_type == Some("TSThisParameter") {
                    continue;
                }
                let param: js::FormalParameter = FromESTree::from_estree(item, allocator)?;
                items.push(param);
            }

            Ok(ABox::new_in(
                js::FormalParameters {
                    span: oxc_span::SPAN,
                    kind: js::FormalParameterKind::FormalParameter,
                    items,
                    rest: None,
                },
                allocator,
            ))
        }
    }

    /// Deserialize `params` field for `TSMethodSignature`.
    impl<'a> FromESTreeConverter<'a> for TSMethodSignatureParams<'a, '_> {
        type Output = ABox<'a, js::FormalParameters<'a>>;

        fn from_estree_converter(
            value: &serde_json::Value,
            allocator: &'a Allocator,
        ) -> DeserResult<Self::Output> {
            // Same logic as TSCallSignatureDeclarationParams
            TSCallSignatureDeclarationParams::from_estree_converter(value, allocator)
        }
    }

    /// Deserialize `params` field for `TSFunctionType`.
    impl<'a> FromESTreeConverter<'a> for TSFunctionTypeParams<'a, '_> {
        type Output = ABox<'a, js::FormalParameters<'a>>;

        fn from_estree_converter(
            value: &serde_json::Value,
            allocator: &'a Allocator,
        ) -> DeserResult<Self::Output> {
            // Same logic as TSCallSignatureDeclarationParams
            TSCallSignatureDeclarationParams::from_estree_converter(value, allocator)
        }
    }

    /// Deserialize `optional` field for `TSMappedType`.
    ///
    /// ESTree represents as boolean or TSMappedTypeModifierOperator.
    /// oxc uses Option<TSMappedTypeModifierOperator>.
    impl<'a> FromESTreeConverter<'a> for TSMappedTypeOptional<'a, '_> {
        type Output = Option<ts::TSMappedTypeModifierOperator>;

        fn from_estree_converter(
            value: &serde_json::Value,
            _allocator: &'a Allocator,
        ) -> DeserResult<Self::Output> {
            if value.is_null() {
                return Ok(None);
            }
            // Can be boolean false (meaning no modifier) or a string like "+" or "-"
            if let Some(b) = value.as_bool() {
                if !b {
                    return Ok(None);
                }
                // True means just the modifier exists with no +/-
                return Ok(Some(ts::TSMappedTypeModifierOperator::True));
            }
            if let Some(s) = value.as_str() {
                let op = match s {
                    "+" => ts::TSMappedTypeModifierOperator::Plus,
                    "-" => ts::TSMappedTypeModifierOperator::Minus,
                    _ => ts::TSMappedTypeModifierOperator::True,
                };
                return Ok(Some(op));
            }
            Ok(None)
        }
    }
} // mod from_estree_converters
