//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)

use std::{cell::Cell, fmt};

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
impl<'a> TSEnumMemberName<'a> {
    pub fn static_name(&self) -> Option<&'a str> {
        match self {
            Self::StaticIdentifier(ident) => Some(ident.name.as_str()),
            Self::StaticStringLiteral(lit) => Some(lit.value.as_str()),
            Self::NumericLiteral(lit) => Some(lit.raw),
            Self::StaticTemplateLiteral(lit) => lit.quasi().map(Into::into),
            _ => None,
        }
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
            TSType::TSAnyKeyword(_)
            | TSType::TSUnknownKeyword(_)
            | TSType::TSUndefinedKeyword(_) => true,
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
    #[inline]
    pub fn is_private(self) -> bool {
        matches!(self, Self::Private)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
        }
    }
}

impl From<TSAccessibility> for &'static str {
    fn from(accessibility: TSAccessibility) -> Self {
        accessibility.as_str()
    }
}

impl fmt::Display for TSAccessibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

impl TSModuleDeclarationKind {
    pub fn is_global(self) -> bool {
        matches!(self, TSModuleDeclarationKind::Global)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Module => "module",
            Self::Namespace => "namespace",
        }
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

    pub fn is_empty(&self) -> bool {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(declaration) => declaration.body.is_none(),
            TSModuleDeclarationBody::TSModuleBlock(block) => block.body.len() == 0,
        }
    }

    pub fn as_module_block_mut(&mut self) -> Option<&mut TSModuleBlock<'a>> {
        match self {
            TSModuleDeclarationBody::TSModuleBlock(block) => Some(block.as_mut()),
            TSModuleDeclarationBody::TSModuleDeclaration(decl) => {
                decl.body.as_mut().and_then(|body| body.as_module_block_mut())
            }
        }
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
    /// // The name of the decorator is `decorator`
    /// @decorator
    /// @decorator.a.b
    /// @decorator(xx)
    /// @decorator.a.b(xx)
    /// ```
    pub fn name(&self) -> Option<&'a str> {
        match &self.expression {
            Expression::Identifier(ident) => Some(ident.name.as_str()),
            expr @ match_member_expression!(Expression) => {
                expr.to_member_expression().static_property_name()
            }
            Expression::CallExpression(call) => {
                call.callee.get_member_expr().and_then(MemberExpression::static_property_name)
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

impl TSTypeOperatorOperator {
    pub fn to_str(self) -> &'static str {
        match self {
            TSTypeOperatorOperator::Keyof => "keyof",
            TSTypeOperatorOperator::Readonly => "readonly",
            TSTypeOperatorOperator::Unique => "unique",
        }
    }
}
