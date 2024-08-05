//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

use std::{cell::Cell, fmt, hash::Hash};

use oxc_allocator::Vec;
use oxc_span::{Atom, Span};

use crate::ast::*;

impl<'a> TSEnumDeclaration<'a> {
    pub fn new(
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> Self {
        Self { span, id, members, r#const, declare, scope_id: Cell::default() }
    }
}

impl<'a> Hash for TSEnumDeclaration<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.members.hash(state);
        self.r#const.hash(state);
        self.declare.hash(state);
    }
}

impl<'a> TSType<'a> {
    pub fn get_identifier_reference(&self) -> Option<IdentifierReference<'a>> {
        match self {
            TSType::TSTypeReference(reference) => {
                Some(TSTypeName::get_first_name(&reference.type_name))
            }
            TSType::TSQualifiedName(qualified) => Some(TSTypeName::get_first_name(&qualified.left)),
            TSType::TSTypeQuery(query) => match &query.expr_name {
                TSTypeQueryExprName::IdentifierReference(ident) => Some((*ident).clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_const_type_reference(&self) -> bool {
        matches!(self, TSType::TSTypeReference(reference) if reference.type_name.is_const())
    }

    /// Check if type maybe `undefined`
    pub fn is_maybe_undefined(&self) -> bool {
        match self {
            TSType::TSUndefinedKeyword(_) => true,
            TSType::TSUnionType(un) => un.types.iter().any(Self::is_maybe_undefined),
            _ => false,
        }
    }

    #[rustfmt::skip]
    pub fn is_keyword(&self) -> bool {
        matches!(self, TSType::TSAnyKeyword(_) | TSType::TSBigIntKeyword(_) | TSType::TSBooleanKeyword(_)
                | TSType::TSNeverKeyword(_) | TSType::TSNullKeyword(_) | TSType::TSNumberKeyword(_)
                | TSType::TSObjectKeyword(_) | TSType::TSStringKeyword(_)| TSType::TSVoidKeyword(_)
                | TSType::TSIntrinsicKeyword(_) | TSType::TSSymbolKeyword(_) | TSType::TSThisType(_)
                | TSType::TSUndefinedKeyword(_) | TSType::TSUnknownKeyword(_)
        )
    }

    pub fn is_keyword_or_literal(&self) -> bool {
        self.is_keyword() || matches!(self, TSType::TSLiteralType(_))
    }
}

impl<'a> TSTypeName<'a> {
    pub fn get_first_name(name: &TSTypeName<'a>) -> IdentifierReference<'a> {
        match name {
            TSTypeName::IdentifierReference(name) => (*name).clone(),
            TSTypeName::QualifiedName(name) => TSTypeName::get_first_name(&name.left),
        }
    }

    pub fn is_const(&self) -> bool {
        if let TSTypeName::IdentifierReference(ident) = self {
            if ident.name == "const" {
                return true;
            }
        }
        false
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::IdentifierReference(_))
    }

    pub fn is_qualified_name(&self) -> bool {
        matches!(self, Self::QualifiedName(_))
    }
}

impl<'a> fmt::Display for TSTypeName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TSTypeName::IdentifierReference(ident) => ident.fmt(f),
            TSTypeName::QualifiedName(qualified) => qualified.fmt(f),
        }
    }
}

impl<'a> fmt::Display for TSQualifiedName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.left, self.right)
    }
}

impl<'a> TSType<'a> {
    /// Remove nested parentheses from this type.
    pub fn without_parenthesized(&self) -> &Self {
        match self {
            Self::TSParenthesizedType(expr) => expr.type_annotation.without_parenthesized(),
            _ => self,
        }
    }
}

impl TSAccessibility {
    pub fn is_private(&self) -> bool {
        matches!(self, TSAccessibility::Private)
    }
}

impl<'a> TSModuleDeclaration<'a> {
    pub fn new(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> Self {
        Self { span, id, body, kind, declare, scope_id: Cell::default() }
    }

    pub fn is_strict(&self) -> bool {
        self.body.as_ref().is_some_and(TSModuleDeclarationBody::is_strict)
    }
}

impl<'a> Hash for TSModuleDeclaration<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.body.hash(state);
        self.kind.hash(state);
        self.declare.hash(state);
    }
}

impl<'a> TSModuleDeclarationName<'a> {
    pub fn is_string_literal(&self) -> bool {
        matches!(self, Self::StringLiteral(_))
    }

    pub fn name(&self) -> Atom<'a> {
        match self {
            Self::Identifier(ident) => ident.name.clone(),
            Self::StringLiteral(lit) => lit.value.clone(),
        }
    }
}

impl<'a> fmt::Display for TSModuleDeclarationName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier(id) => id.fmt(f),
            Self::StringLiteral(lit) => lit.fmt(f),
        }
    }
}

impl<'a> TSModuleDeclarationBody<'a> {
    pub fn is_strict(&self) -> bool {
        matches!(self, Self::TSModuleBlock(block) if block.is_strict())
    }
}

impl<'a> TSModuleBlock<'a> {
    pub fn is_strict(&self) -> bool {
        self.directives.iter().any(Directive::is_use_strict)
    }
}

impl<'a> TSModuleReference<'a> {
    /// Returns `true` if this is an [`TSModuleReference::ExternalModuleReference`].
    pub fn is_external(&self) -> bool {
        matches!(self, Self::ExternalModuleReference(_))
    }
}

impl<'a> Decorator<'a> {
    /// Get the name of the decorator
    /// ```ts
    /// @decorator
    /// @decorator.a.b
    /// @decorator(xx)
    /// @decorator.a.b(xx)
    /// The name of the decorator is `decorator`
    /// ```
    pub fn name(&self) -> Option<&str> {
        match &self.expression {
            Expression::Identifier(ident) => Some(&ident.name),
            expr @ match_member_expression!(Expression) => {
                expr.to_member_expression().static_property_name()
            }
            Expression::CallExpression(call) => {
                call.callee.get_member_expr().map(|member| member.static_property_name())?
            }
            _ => None,
        }
    }
}

impl ImportOrExportKind {
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value)
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }
}

impl<'a> Hash for TSMappedType<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.type_parameter.hash(state);
        self.name_type.hash(state);
        self.type_annotation.hash(state);
        self.optional.hash(state);
        self.readonly.hash(state);
    }
}

impl<'a> Hash for TSConditionalType<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.check_type.hash(state);
        self.extends_type.hash(state);
        self.true_type.hash(state);
        self.false_type.hash(state);
    }
}

impl<'a> Hash for TSInterfaceDeclaration<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.id.hash(state);
        self.type_parameters.hash(state);
        self.extends.hash(state);
        self.body.hash(state);
        self.declare.hash(state);
    }
}

impl<'a> Hash for TSTypeAliasDeclaration<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.id.hash(state);
        self.type_parameters.hash(state);
        self.type_annotation.hash(state);
        self.declare.hash(state);
    }
}

impl<'a> Hash for TSMethodSignature<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.key.hash(state);
        self.computed.hash(state);
        self.optional.hash(state);
        self.kind.hash(state);
        self.this_param.hash(state);
        self.params.hash(state);
        self.return_type.hash(state);
        self.type_parameters.hash(state);
    }
}

impl<'a> Hash for TSConstructSignatureDeclaration<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.params.hash(state);
        self.return_type.hash(state);
        self.type_parameters.hash(state);
    }
}
