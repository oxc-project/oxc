//! TypeScript Definitions
//!
//! [AST Spec](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
//! [Archived TypeScript spec](https://github.com/microsoft/TypeScript/blob/3c99d50da5a579d9fa92d02664b1b66d4ff55944/doc/spec-ARCHIVED.md)
// NB: `#[visited_node]` attribute on AST nodes does not do anything to the code in this file.
// It is purely a marker for codegen used in `oxc_traverse`. See docs in that crate.
use std::{cell::Cell, hash::Hash};

use oxc_allocator::{Box, Vec};
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

impl<'a> TSTypeParameter<'a> {
    pub fn new(
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> Self {
        Self { span, name, constraint, default, r#in, out, r#const }
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

impl<'a> TSMappedType<'a> {
    pub fn new(
        span: Span,
        type_parameter: Box<'a, TSTypeParameter<'a>>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> Self {
        Self {
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id: Cell::new(None),
        }
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

impl<'a> TSConditionalType<'a> {
    pub fn new(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> Self {
        Self { span, check_type, extends_type, true_type, false_type, scope_id: Cell::new(None) }
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

impl<'a> TSInterfaceDeclaration<'a> {
    pub fn new(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        body: Box<'a, TSInterfaceBody<'a>>,
        declare: bool,
    ) -> Self {
        Self { span, id, type_parameters, extends, body, declare, scope_id: Cell::new(None) }
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

impl<'a> TSTypeAliasDeclaration<'a> {
    pub fn new(
        span: Span,
        id: BindingIdentifier<'a>,
        type_annotation: TSType<'a>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
        declare: bool,
    ) -> Self {
        Self { span, id, type_parameters, type_annotation, declare, scope_id: Cell::new(None) }
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

#[allow(clippy::too_many_arguments)]
impl<'a> TSMethodSignature<'a> {
    pub fn new(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        this_param: Option<TSThisParameter<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Self {
        Self {
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params,
            return_type,
            type_parameters,
            scope_id: Cell::new(None),
        }
    }
}

impl<'a> TSConstructSignatureDeclaration<'a> {
    pub fn new(
        span: Span,
        params: Box<'a, FormalParameters<'a>>,
        return_type: Option<Box<'a, TSTypeAnnotation<'a>>>,
        type_parameters: Option<Box<'a, TSTypeParameterDeclaration<'a>>>,
    ) -> Self {
        Self { span, params, return_type, type_parameters, scope_id: Cell::new(None) }
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
