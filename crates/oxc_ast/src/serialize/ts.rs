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
        global = kind === 'global',
        start = DESER[u32](POS_OFFSET.span.start),
        end = DESER[u32](POS_OFFSET.span.end),
        declare = DESER[bool](POS_OFFSET.declare);
    let id = DESER[TSModuleDeclarationName](POS_OFFSET.id),
        body = DESER[Option<TSModuleDeclarationBody>](POS_OFFSET.body);

    // Flatten `body`, and nest `id`
    if (body !== null && body.type === 'TSModuleDeclaration') {
        let innerId = body.id;
        if (innerId.type === 'Identifier') {
            id = {
                type: 'TSQualifiedName',
                start: id.start,
                end: innerId.end,
                left: id,
                right: innerId,
            };
        } else {
            // Replace `left` of innermost `TSQualifiedName` with a nested `TSQualifiedName` with `id` of
            // this module on left, and previous `left` of innermost `TSQualifiedName` on right
            while (true) {
                innerId.start = id.start;
                if (innerId.left.type === 'Identifier') break;
                innerId = innerId.left;
            }
            innerId.left = {
                type: 'TSQualifiedName',
                start: id.start,
                end: innerId.left.end,
                left: id,
                right: innerId.left,
            };
            id = body.id;
        }
        body = Object.hasOwn(body, 'body') ? body.body : null;
    }

    // Skip `body` field if `null`
    const node = body === null
        ? { type: 'TSModuleDeclaration', start, end, id, kind, declare, global }
        : { type: 'TSModuleDeclaration', start, end, id, body, kind, declare, global };
    node
")]
pub struct TSModuleDeclarationConverter<'a, 'b>(pub &'b TSModuleDeclaration<'a>);

impl ESTree for TSModuleDeclarationConverter<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let module = self.0;

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSModuleDeclaration"));
        state.serialize_field("start", &module.span.start);
        state.serialize_field("end", &module.span.end);

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
        state.serialize_field("global", &TSModuleDeclarationGlobal(module));
        state.end();
    }
}

struct TSModuleDeclarationIdParts<'a, 'b>(&'b [&'b BindingIdentifier<'a>]);

impl ESTree for TSModuleDeclarationIdParts<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let parts = self.0;
        assert!(!parts.is_empty());

        let span_start = parts[0].span.start;
        let (&last, rest) = parts.split_last().unwrap();

        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TSQualifiedName"));
        state.serialize_field("start", &span_start);
        state.serialize_field("end", &last.span.end);

        if rest.len() == 1 {
            // Only one part remaining (e.g. `X`). Serialize as `Identifier`.
            state.serialize_field("left", &rest[0]);
        } else {
            // Multiple parts remaining (e.g. `X.Y`). Recurse to serialize as `TSQualifiedName`.
            state.serialize_field("left", &TSModuleDeclarationIdParts(rest));
        }

        state.serialize_field("right", last);
        state.end();
    }
}

/// Serializer for `global` field of `TSModuleDeclaration`.
///
/// `true` if `kind` is `TSModuleDeclarationKind::Global`.
#[ast_meta]
#[estree(ts_type = "boolean", raw_deser = "THIS.kind === 'global'")]
pub struct TSModuleDeclarationGlobal<'a, 'b>(pub &'b TSModuleDeclaration<'a>);

impl ESTree for TSModuleDeclarationGlobal<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.kind.is_global().serialize(serializer);
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

/// Serializer for `constraint` field of `TSMappedType`.
///
/// NOTE: Variable `typeParameter` in `raw_deser` is shared between `key` and `constraint` serializers.
/// They will be concatenated in the generated code.
#[ast_meta]
#[estree(ts_type = "TSTypeParameter['constraint']", raw_deser = "typeParameter.constraint")]
pub struct TSMappedTypeConstraint<'a, 'b>(pub &'b TSMappedType<'a>);

impl ESTree for TSMappedTypeConstraint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.0.type_parameter.constraint.serialize(serializer);
    }
}

/// Serializer for `IdentifierReference` variant of `TSTypeName`.
///
/// Where is an identifier called `this`, TS-ESTree presents it as a `ThisExpression`.
#[ast_meta]
#[estree(
    ts_type = "IdentifierReference | ThisExpression",
    raw_deser = "
        let id = DESER[Box<IdentifierReference>](POS);
        if (id.name === 'this') id = { type: 'ThisExpression', start: id.start, end: id.end };
        id
    "
)]
pub struct TSTypeNameIdentifierReference<'a, 'b>(pub &'b IdentifierReference<'a>);

impl ESTree for TSTypeNameIdentifierReference<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let ident = self.0;
        if ident.name == "this" {
            ThisExpression { span: ident.span }.serialize(serializer);
        } else {
            ident.serialize(serializer);
        }
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
            let parent = expression = {
                type: 'MemberExpression',
                start: expression.start,
                end: expression.end,
                object: expression.left,
                property: expression.right,
                optional: false,
                computed: false,
            };

            while (parent.object.type === 'TSQualifiedName') {
                const object = parent.object;
                parent = parent.object = {
                    type: 'MemberExpression',
                    start: object.start,
                    end: object.end,
                    object: object.left,
                    property: object.right,
                    optional: false,
                    computed: false,
                };
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
                TSTypeNameIdentifierReference(ident).serialize(serializer);
            }
            TSTypeName::QualifiedName(name) => {
                // Convert to `TSQualifiedName` to `MemberExpression`.
                // Recursively convert `left` to `MemberExpression` too if it's a `TSQualifiedName`.
                let mut state = serializer.serialize_struct();
                state.serialize_field("type", &JsonSafeString("MemberExpression"));
                state.serialize_field("start", &name.span.start);
                state.serialize_field("end", &name.span.end);
                state.serialize_field("object", &TSTypeNameAsMemberExpression(&name.left));
                state.serialize_field("property", &name.right);
                state.serialize_field("optional", &false);
                state.serialize_field("computed", &false);
                state.end();
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
