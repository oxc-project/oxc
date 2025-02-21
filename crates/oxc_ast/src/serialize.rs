use cow_utils::CowUtils;

use oxc_ast_macros::ast_meta;
use oxc_estree::{
    CompactTSSerializer, ESTree, PrettyTSSerializer, SequenceSerializer, Serializer,
    StructSerializer,
};

use crate::ast::*;

impl Program<'_> {
    /// Serialize AST to ESTree JSON, including TypeScript fields.
    pub fn to_estree_ts_json(&self) -> String {
        let mut serializer = CompactTSSerializer::new();
        self.serialize(&mut serializer);
        serializer.into_string()
    }

    /// Serialize AST to pretty-printed ESTree JSON, including TypeScript fields.
    pub fn to_pretty_estree_ts_json(&self) -> String {
        let mut serializer = PrettyTSSerializer::new();
        self.serialize(&mut serializer);
        serializer.into_string()
    }
}

// --------------------
// Basic types
// --------------------

/// Serialized as `null`.
#[ast_meta]
#[estree(ts_type = "null")]
pub struct Null<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for Null<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

/// Serialized as `true`.
#[ast_meta]
#[estree(ts_type = "true")]
pub struct True<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for True<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        true.serialize(serializer);
    }
}

/// Serialized as `false`.
#[ast_meta]
#[estree(ts_type = "false")]
pub struct False<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for False<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        false.serialize(serializer);
    }
}

/// Serialized as `"in"`.
#[ast_meta]
#[estree(ts_type = "'in'")]
pub struct In<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for In<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        "in".serialize(serializer);
    }
}

/// Serialized as `"init"`.
#[ast_meta]
#[estree(ts_type = "'init'")]
pub struct Init<'b, T>(#[expect(dead_code)] pub &'b T);

impl<T> ESTree for Init<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        "init".serialize(serializer);
    }
}

// --------------------
// Literals
// --------------------

/// Serializer for `raw` field of `BooleanLiteral`.
#[ast_meta]
#[estree(ts_type = "string | null")]
pub struct BooleanLiteralRaw<'b>(pub &'b BooleanLiteral);

impl ESTree for BooleanLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        #[expect(clippy::collection_is_never_read)] // Clippy is wrong!
        let raw = if self.0.span.is_unspanned() {
            None
        } else if self.0.value {
            Some("true")
        } else {
            Some("false")
        };
        raw.serialize(serializer);
    }
}

/// Serializer for `raw` field of `NullLiteral`.
#[ast_meta]
#[estree(ts_type = "'null' | null")]
pub struct NullLiteralRaw<'b>(pub &'b NullLiteral);

impl ESTree for NullLiteralRaw<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        #[expect(clippy::collection_is_never_read)] // Clippy is wrong!
        let raw = if self.0.span.is_unspanned() { None } else { Some("null") };
        raw.serialize(serializer);
    }
}

/// Serializer for `bigint` field of `BigIntLiteral`.
#[ast_meta]
#[estree(ts_type = "string")]
pub struct BigIntLiteralBigint<'a, 'b>(pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralBigint<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let bigint = self.0.raw.strip_suffix('n').unwrap().cow_replace('_', "");
        bigint.serialize(serializer);
    }
}

/// Serializer for `value` field of `BigIntLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `BigInt`.
#[ast_meta]
#[estree(ts_type = "BigInt")]
pub struct BigIntLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b BigIntLiteral<'a>);

impl ESTree for BigIntLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

/// Serializer for `regex` field of `RegExpLiteral`.
#[ast_meta]
#[estree(ts_type = "RegExp")]
pub struct RegExpLiteralRegex<'a, 'b>(pub &'b RegExpLiteral<'a>);

impl ESTree for RegExpLiteralRegex<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("pattern", &self.0.regex.pattern);

        // If `raw` field is present, flags must be in same order as in source to match Acorn.
        // Count number of set bits in `flags` to get number of flags
        // (cheaper than searching through `raw` for last `/`).
        let flags = self.0.regex.flags;
        if let Some(raw) = &self.0.raw {
            let flags_count = flags.bits().count_ones() as usize;
            let flags_index = raw.len() - flags_count;
            state.serialize_field("flags", &raw[flags_index..]);
        } else {
            state.serialize_field("flags", &flags);
        }
        state.end();
    }
}

/// Serializer for `value` field of `RegExpLiteral`.
///
/// Serialized as `null` in JSON, but updated on JS side to contain a `RegExp` if the regexp is valid.
#[ast_meta]
#[estree(ts_type = "RegExp | null")]
pub struct RegExpLiteralValue<'a, 'b>(#[expect(dead_code)] pub &'b RegExpLiteral<'a>);

impl ESTree for RegExpLiteralValue<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

impl ESTree for RegExpFlags {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.to_string().as_str().serialize(serializer);
    }
}

impl ESTree for RegExpPattern<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        self.to_string().as_str().serialize(serializer);
    }
}

// --------------------
// Various
// --------------------

/// Serialize `ArrayExpressionElement::Elision` variant as `null`.
impl ESTree for Elision {
    fn serialize<S: Serializer>(&self, serializer: S) {
        ().serialize(serializer);
    }
}

/// Serialize `FormalParameters`, to be estree compatible, with `items` and `rest` fields combined
/// and `argument` field flattened.
impl ESTree for FormalParameters<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        for item in &self.items {
            seq.serialize_element(item);
        }

        if let Some(rest) = &self.rest {
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
        state.serialize_field("type", "RestElement");
        state.serialize_field("start", &rest.span.start);
        state.serialize_field("end", &rest.span.end);
        state.serialize_field("argument", &rest.argument.kind);
        state.serialize_field("type_annotation", &rest.argument.type_annotation);
        state.serialize_field("optional", &rest.argument.optional);
        state.end();
    }
}

/// Serializer for `specifiers` field of `ImportDeclaration`.
///
/// Serialize `specifiers` as an empty array if it's `None`.
#[ast_meta]
#[estree(ts_type = "Array<ImportDeclarationSpecifier>")]
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

/// Serialize `ObjectProperty` with fields in same order as Acorn.
impl ESTree for ObjectProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Property");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("method", &self.method);
        state.serialize_field("shorthand", &self.shorthand);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("key", &self.key);
        // Acorn has `kind` field before `value` for methods and shorthand properties
        if self.method || self.kind != PropertyKind::Init || self.shorthand {
            state.serialize_field("kind", &self.kind);
            state.serialize_field("value", &self.value);
        } else {
            state.serialize_field("value", &self.value);
            state.serialize_field("kind", &self.kind);
        }
        state.end();
    }
}

/// Serialize `BindingProperty` with fields in same order as Acorn.
impl ESTree for BindingProperty<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "Property");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("method", &false);
        state.serialize_field("shorthand", &self.shorthand);
        state.serialize_field("computed", &self.computed);
        state.serialize_field("key", &self.key);
        // Acorn has `kind` field before `value` for shorthand properties
        if self.shorthand {
            state.serialize_field("kind", "init");
            state.serialize_field("value", &self.value);
        } else {
            state.serialize_field("value", &self.value);
            state.serialize_field("kind", "init");
        }
        state.end();
    }
}

/// Serializer for `ArrowFunctionExpression`'s `body` field.
///
/// Serializes as either an expression (if `expression` property is set),
/// or a `BlockStatement` (if it's not).
#[ast_meta]
#[estree(ts_type = "FunctionBody | Expression")]
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
#[estree(ts_type = "IdentifierReference | AssignmentTargetWithDefault")]
pub struct AssignmentTargetPropertyIdentifierValue<'a>(
    pub &'a AssignmentTargetPropertyIdentifier<'a>,
);

impl ESTree for AssignmentTargetPropertyIdentifierValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(init) = &self.0.init {
            let mut state = serializer.serialize_struct();
            state.serialize_field("type", "AssignmentPattern");
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

/// Serializer for `arguments` field of `ImportExpression`
/// (which is renamed to `options` in ESTree AST).
///
/// Serialize only the first expression in `arguments`, or `null` if `arguments` is empty.
#[ast_meta]
#[estree(ts_type = "Expression | null")]
pub struct ImportExpressionArguments<'a>(pub &'a ImportExpression<'a>);

impl ESTree for ImportExpressionArguments<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(expression) = self.0.arguments.first() {
            expression.serialize(serializer);
        } else {
            ().serialize(serializer);
        }
    }
}

/// Serialize `ExportNamedDeclaration`.
///
/// Omit `with_clause` field (which is renamed to `attributes` in ESTree)
/// unless `source` field is `Some`.
impl ESTree for ExportNamedDeclaration<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", "ExportNamedDeclaration");
        state.serialize_field("start", &self.span.start);
        state.serialize_field("end", &self.span.end);
        state.serialize_field("declaration", &self.declaration);
        state.serialize_field("specifiers", &self.specifiers);
        state.serialize_field("source", &self.source);
        state.serialize_field("exportKind", &self.export_kind);
        if self.source.is_some() {
            state.serialize_field(
                "attributes",
                &crate::serialize::ExportNamedDeclarationWithClause(self),
            );
        }
        state.end();
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
#[estree(ts_type = "Array<ImportAttribute>")]
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
#[estree(ts_type = "Array<ImportAttribute>")]
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
#[estree(ts_type = "Array<ImportAttribute>")]
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

impl ESTree for JSXElementName<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::Identifier(ident) => ident.serialize(serializer),
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer);
            }
            Self::NamespacedName(name) => name.serialize(serializer),
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer);
            }
        }
    }
}

impl ESTree for JSXMemberExpressionObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        match self {
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name }.serialize(serializer);
            }
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer);
            }
        }
    }
}
