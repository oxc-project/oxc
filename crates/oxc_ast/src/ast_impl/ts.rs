//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)
#![warn(missing_docs)]

use std::fmt;

use oxc_span::Atom;

use crate::ast::*;

impl<'a> TSEnumMemberName<'a> {
    /// Get the name of this enum member if it can be determined statically.
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
    /// Get the first identifier reference in this type.
    ///
    /// For qualified (i.e.  namespaced) types, the left-most identifier is
    /// returned.
    ///
    /// ```
    /// let ty = get_type_for("foo.bar.Baz"); // TSType::TSQualifiedName
    /// get_identifier_reference(&ty); // Some(IdentifierReference { name: "foo", .. })
    /// ```
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

    /// Returns `true` if this type is a type reference to `const`.
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

    /// Returns `true` if this is a keyword type (e.g. `number`, `any`, `string`).
    #[rustfmt::skip]
    pub fn is_keyword(&self) -> bool {
        matches!(self, TSType::TSAnyKeyword(_) | TSType::TSBigIntKeyword(_) | TSType::TSBooleanKeyword(_)
                | TSType::TSNeverKeyword(_) | TSType::TSNullKeyword(_) | TSType::TSNumberKeyword(_)
                | TSType::TSObjectKeyword(_) | TSType::TSStringKeyword(_)| TSType::TSVoidKeyword(_)
                | TSType::TSIntrinsicKeyword(_) | TSType::TSSymbolKeyword(_) | TSType::TSThisType(_)
                | TSType::TSUndefinedKeyword(_) | TSType::TSUnknownKeyword(_)
        )
    }

    /// Returns `true` if this is a [keyword] or literal type.
    ///
    /// [keyword]: Self::is_keyword
    pub fn is_keyword_or_literal(&self) -> bool {
        self.is_keyword() || matches!(self, TSType::TSLiteralType(_))
    }
}

impl<'a> TSTypeName<'a> {
    /// Get the "leftmost" identifier in a dot-separated type name.
    ///
    /// ## Example
    /// ```ts
    /// type Foo = Bar; // -> Bar
    /// type Foo = Bar.Baz; // -> Bar
    /// ```
    pub fn get_first_name(name: &TSTypeName<'a>) -> IdentifierReference<'a> {
        match name {
            TSTypeName::IdentifierReference(name) => (*name).clone(),
            TSTypeName::QualifiedName(name) => TSTypeName::get_first_name(&name.left),
        }
    }

    /// Returns `true` if this is a reference to `const`.
    pub fn is_const(&self) -> bool {
        if let TSTypeName::IdentifierReference(ident) = self {
            if ident.name == "const" {
                return true;
            }
        }
        false
    }

    /// Returns `true` if this is an [`TSTypeName::IdentifierReference`].
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::IdentifierReference(_))
    }

    /// Returns `true` if this is a [qualified name](TSTypeName::QualifiedName)
    /// (i.e. a dot-separated name).
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
    /// Returns `true` for `private` accessibility modifiers.
    #[inline]
    pub fn is_private(self) -> bool {
        matches!(self, Self::Private)
    }

    /// Converts this modifier into a string as it would appear in the source code.
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
    /// Returns `true` if this module's body exists and uses strict mode
    /// semantics (as determined by [`TSModuleDeclarationBody::is_strict`]).
    pub fn is_strict(&self) -> bool {
        self.body.as_ref().is_some_and(TSModuleDeclarationBody::is_strict)
    }
}

impl TSModuleDeclarationKind {
    /// Returns `true` for `declare global { ... }`
    pub fn is_global(self) -> bool {
        matches!(self, TSModuleDeclarationKind::Global)
    }

    /// Declaration keyword as a string, identical to how it would appear in the
    /// source code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Module => "module",
            Self::Namespace => "namespace",
        }
    }
}

impl<'a> TSModuleDeclarationName<'a> {
    /// Returns `true` if this name is a string literal.
    ///
    /// ## Example
    /// ```ts
    /// // true
    /// module "*.less" {
    ///     const styles: { [key: string]: string };
    ///     export default styles;
    /// }
    ///
    /// // false
    /// module bar {}
    /// namespace bang {}
    /// ```
    pub fn is_string_literal(&self) -> bool {
        matches!(self, Self::StringLiteral(_))
    }

    /// Get the static name of this module declaration name.
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
    /// Does the body of this module use strict mode semantics?
    pub fn is_strict(&self) -> bool {
        matches!(self, Self::TSModuleBlock(block) if block.is_strict())
    }

    /// Returns `true` if this module contains no statements.
    pub fn is_empty(&self) -> bool {
        match self {
            TSModuleDeclarationBody::TSModuleDeclaration(declaration) => declaration.body.is_none(),
            TSModuleDeclarationBody::TSModuleBlock(block) => block.body.len() == 0,
        }
    }

    /// Get a mutable reference to `self` as a [`TSModuleBlock`]. Returns
    /// [`None`] if the body is something other than a block.
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
    /// Returns `true` if this module contains a `"use strict"` directive.
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
    /// Returns `true` for "regular" imports and exports.
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value)
    }

    /// Returns `true` if this is an `import type` or `export type` statement.
    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }
}

impl TSTypeOperatorOperator {
    /// Get the operator string as it would appear in the source code.
    pub fn to_str(self) -> &'static str {
        match self {
            TSTypeOperatorOperator::Keyof => "keyof",
            TSTypeOperatorOperator::Readonly => "readonly",
            TSTypeOperatorOperator::Unique => "unique",
        }
    }
}
