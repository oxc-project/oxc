// FIXME: lots of methods are missing docs. If you have time, it would be a huge help to add some :)
#![warn(missing_docs)]
use std::{borrow::Cow, fmt};

use oxc_allocator::{Box, FromIn, Vec};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{operator::UnaryOperator, scope::ScopeFlags, symbol::SymbolId};

use crate::ast::*;

impl Program<'_> {
    /// Returns `true` if this program has no statements or directives.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty() && self.directives.is_empty()
    }

    /// Returns `true` if this program has a `"use strict"` directive.
    pub fn has_use_strict_directive(&self) -> bool {
        self.directives.iter().any(Directive::is_use_strict)
    }
}

impl<'a> Expression<'a> {
    /// Returns `true` if this expression is TypeScript-specific syntax.
    pub fn is_typescript_syntax(&self) -> bool {
        matches!(
            self,
            Self::TSAsExpression(_)
                | Self::TSSatisfiesExpression(_)
                | Self::TSTypeAssertion(_)
                | Self::TSNonNullExpression(_)
                | Self::TSInstantiationExpression(_)
        )
    }

    /// Returns `true` if this is a [primary expression](https://tc39.es/ecma262/#sec-primary-expression).
    pub fn is_primary_expression(&self) -> bool {
        self.is_literal()
            || matches!(
                self,
                Self::Identifier(_)
                    | Self::ThisExpression(_)
                    | Self::FunctionExpression(_)
                    | Self::ClassExpression(_)
                    | Self::ParenthesizedExpression(_)
                    | Self::ArrayExpression(_)
                    | Self::ObjectExpression(_)
            )
    }

    /// `true` if this [`Expression`] is a literal expression for a primitive value.
    ///
    /// Does not include [`TemplateLiteral`]s, [`object literals`], or [`array literals`].
    ///
    /// [`object literals`]: ObjectExpression
    /// [`array literals`]: ArrayExpression
    pub fn is_literal(&self) -> bool {
        // Note: TemplateLiteral is not `Literal`
        matches!(
            self,
            Self::BooleanLiteral(_)
                | Self::NullLiteral(_)
                | Self::NumericLiteral(_)
                | Self::BigIntLiteral(_)
                | Self::RegExpLiteral(_)
                | Self::StringLiteral(_)
        )
    }

    /// Returns `true` for [string](StringLiteral) and [template](TemplateLiteral) literals.
    pub fn is_string_literal(&self) -> bool {
        matches!(self, Self::StringLiteral(_) | Self::TemplateLiteral(_))
    }

    /// Returns `true` for [numeric](NumericLiteral) and [big int](BigIntLiteral) literals.
    pub fn is_number_literal(&self) -> bool {
        matches!(self, Self::NumericLiteral(_) | Self::BigIntLiteral(_))
    }

    /// Returns `true` for [bigint literals](BigIntLiteral).
    pub fn is_big_int_literal(&self) -> bool {
        matches!(self, Self::BigIntLiteral(_))
    }

    /// Returns `true` for [string literals](StringLiteral) matching the
    /// expected value. Note that [non-substitution template
    /// literals](TemplateLiteral) are not considered.
    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        match self {
            Self::StringLiteral(s) => s.value == string,
            _ => false,
        }
    }

    /// Determines whether the given expr is a `null` literal
    pub fn is_null(&self) -> bool {
        matches!(self, Expression::NullLiteral(_))
    }

    /// Determines whether the given expr is a `undefined` literal
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "undefined")
    }

    /// Determines whether the given expr is a `void expr`
    pub fn is_void(&self) -> bool {
        matches!(self, Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void)
    }

    /// Determines whether the given expr is a `void 0`
    pub fn is_void_0(&self) -> bool {
        match self {
            Self::UnaryExpression(expr) if expr.operator == UnaryOperator::Void => {
                matches!(&expr.argument, Self::NumericLiteral(lit) if lit.value == 0.0)
            }
            _ => false,
        }
    }

    /// Returns `true` for [numeric literals](NumericLiteral)
    pub fn is_number(&self) -> bool {
        matches!(self, Self::NumericLiteral(_))
    }

    /// Determines whether the given expr is a `0`
    pub fn is_number_0(&self) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.value == 0.0)
    }

    /// Determines whether the given expr is a specific [number](NumericLiteral) literal.
    pub fn is_number_value(&self, val: f64) -> bool {
        matches!(self, Self::NumericLiteral(lit) if (lit.value - val).abs() < f64::EPSILON)
    }

    /// Determines whether the given numeral literal's raw value is exactly val
    pub fn is_specific_raw_number_literal(&self, val: &str) -> bool {
        matches!(self, Self::NumericLiteral(lit) if lit.raw.as_ref().is_some_and(|raw| raw == val))
    }

    /// Determines whether the given expr evaluate to `undefined`
    pub fn evaluate_to_undefined(&self) -> bool {
        self.is_undefined() || self.is_void()
    }

    /// Determines whether the given expr is a `null` or `undefined` or `void 0`
    ///
    /// Corresponds to a [nullish value check](https://developer.mozilla.org/en-US/docs/Glossary/Nullish).
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.evaluate_to_undefined()
    }

    /// Determines whether the given expr is a `NaN` literal
    pub fn is_nan(&self) -> bool {
        matches!(self, Self::Identifier(ident) if ident.name == "NaN")
    }

    /// Remove nested parentheses from this expression.
    pub fn without_parentheses(&self) -> &Self {
        let mut expr = self;
        while let Expression::ParenthesizedExpression(paran_expr) = expr {
            expr = &paran_expr.expression;
        }
        expr
    }

    /// Returns `true` if this [`Expression`] is an [`IdentifierReference`] with specified `name`.
    pub fn is_specific_id(&self, name: &str) -> bool {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => ident.name == name,
            _ => false,
        }
    }

    #[allow(missing_docs)]
    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        match self.get_inner_expression() {
            expr if expr.is_member_expression() => {
                expr.to_member_expression().is_specific_member_access(object, property)
            }
            Expression::ChainExpression(chain) => {
                let Some(expr) = chain.expression.as_member_expression() else {
                    return false;
                };
                expr.is_specific_member_access(object, property)
            }
            _ => false,
        }
    }

    #[allow(missing_docs)]
    #[must_use]
    pub fn into_inner_expression(self) -> Expression<'a> {
        let mut expr = self;
        loop {
            expr = match expr {
                Expression::ParenthesizedExpression(e) => e.unbox().expression,
                Expression::TSAsExpression(e) => e.unbox().expression,
                Expression::TSSatisfiesExpression(e) => e.unbox().expression,
                Expression::TSInstantiationExpression(e) => e.unbox().expression,
                Expression::TSNonNullExpression(e) => e.unbox().expression,
                Expression::TSTypeAssertion(e) => e.unbox().expression,
                _ => break,
            };
        }
        expr
    }

    #[allow(missing_docs)]
    pub fn get_inner_expression(&self) -> &Expression<'a> {
        let mut expr = self;
        loop {
            expr = match expr {
                Expression::ParenthesizedExpression(e) => &e.expression,
                Expression::TSAsExpression(e) => &e.expression,
                Expression::TSSatisfiesExpression(e) => &e.expression,
                Expression::TSInstantiationExpression(e) => &e.expression,
                Expression::TSNonNullExpression(e) => &e.expression,
                Expression::TSTypeAssertion(e) => &e.expression,
                _ => break,
            };
        }
        expr
    }

    #[allow(missing_docs)]
    pub fn get_inner_expression_mut(&mut self) -> &mut Expression<'a> {
        let mut expr = self;
        loop {
            expr = match expr {
                Expression::ParenthesizedExpression(e) => &mut e.expression,
                Expression::TSAsExpression(e) => &mut e.expression,
                Expression::TSSatisfiesExpression(e) => &mut e.expression,
                Expression::TSInstantiationExpression(e) => &mut e.expression,
                Expression::TSNonNullExpression(e) => &mut e.expression,
                Expression::TSTypeAssertion(e) => &mut e.expression,
                _ => break,
            };
        }
        expr
    }

    /// Returns `true` if this [`Expression`] is an [`IdentifierReference`].
    pub fn is_identifier_reference(&self) -> bool {
        matches!(self, Expression::Identifier(_))
    }

    #[allow(missing_docs)]
    pub fn get_identifier_reference(&self) -> Option<&IdentifierReference<'a>> {
        match self.get_inner_expression() {
            Expression::Identifier(ident) => Some(ident),
            _ => None,
        }
    }

    /// Returns `true` if this [`Expression`] is a function
    /// (either [`Function`] or [`ArrowFunctionExpression`]).
    pub fn is_function(&self) -> bool {
        matches!(self, Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
    }

    /// Returns `true` if this [`Expression`] is a [`CallExpression`].
    pub fn is_call_expression(&self) -> bool {
        matches!(self, Expression::CallExpression(_))
    }

    /// Returns `true` if this [`Expression`] is a [`Super`].
    pub fn is_super(&self) -> bool {
        matches!(self, Expression::Super(_))
    }

    /// Returns `true` if this [`Expression`] is a [`CallExpression`] with [`Super`] as callee.
    pub fn is_super_call_expression(&self) -> bool {
        matches!(self, Expression::CallExpression(expr) if matches!(&expr.callee, Expression::Super(_)))
    }

    /// Returns `true` if this [`Expression`] is a [`CallExpression`], [`NewExpression`],
    /// or [`ImportExpression`].
    pub fn is_call_like_expression(&self) -> bool {
        self.is_call_expression()
            && matches!(self, Expression::NewExpression(_) | Expression::ImportExpression(_))
    }

    /// Returns `true` if this [`Expression`] is a [`BinaryExpression`] or [`LogicalExpression`].
    pub fn is_binaryish(&self) -> bool {
        matches!(self, Expression::BinaryExpression(_) | Expression::LogicalExpression(_))
    }

    #[allow(missing_docs)]
    pub fn get_member_expr(&self) -> Option<&MemberExpression<'a>> {
        match self.get_inner_expression() {
            Expression::ChainExpression(chain_expr) => chain_expr.expression.as_member_expression(),
            expr => expr.as_member_expression(),
        }
    }

    /// Returns `true` if this [`Expression`] is a `require` call.
    ///
    /// See [`CallExpression::is_require_call`] for details of the exact patterns that match.
    pub fn is_require_call(&self) -> bool {
        if let Self::CallExpression(call_expr) = self {
            call_expr.is_require_call()
        } else {
            false
        }
    }

    /// Returns `true` if this is an [assignment expression](AssignmentExpression).
    pub fn is_assignment(&self) -> bool {
        matches!(self, Expression::AssignmentExpression(_))
    }
}

impl fmt::Display for IdentifierName<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Display for IdentifierReference<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Display for BindingIdentifier<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl ArrayExpressionElement<'_> {
    #[allow(missing_docs)]
    pub fn is_elision(&self) -> bool {
        matches!(self, Self::Elision(_))
    }
}

impl<'a> From<Argument<'a>> for ArrayExpressionElement<'a> {
    fn from(argument: Argument<'a>) -> Self {
        match argument {
            Argument::SpreadElement(spread) => Self::SpreadElement(spread),
            match_expression!(Argument) => Self::from(argument.into_expression()),
        }
    }
}

impl ObjectPropertyKind<'_> {
    /// Returns `true` if this object property is a [spread](SpreadElement).
    #[inline]
    pub fn is_spread(&self) -> bool {
        matches!(self, Self::SpreadProperty(_))
    }
}

impl<'a> PropertyKey<'a> {
    #[allow(missing_docs)]
    pub fn static_name(&self) -> Option<Cow<'a, str>> {
        match self {
            Self::StaticIdentifier(ident) => Some(Cow::Borrowed(ident.name.as_str())),
            Self::StringLiteral(lit) => Some(Cow::Borrowed(lit.value.as_str())),
            Self::RegExpLiteral(lit) => Some(Cow::Owned(lit.regex.to_string())),
            Self::NumericLiteral(lit) => Some(Cow::Owned(lit.value.to_string())),
            Self::BigIntLiteral(lit) => Some(Cow::Borrowed(lit.raw.as_str())),
            Self::NullLiteral(_) => Some(Cow::Borrowed("null")),
            Self::TemplateLiteral(lit) => {
                lit.expressions.is_empty().then(|| lit.quasi()).flatten().map(Into::into)
            }
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn is_specific_static_name(&self, name: &str) -> bool {
        self.static_name().is_some_and(|n| n == name)
    }

    #[allow(missing_docs)]
    pub fn is_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_) | Self::StaticIdentifier(_))
    }

    #[allow(missing_docs)]
    pub fn is_private_identifier(&self) -> bool {
        matches!(self, Self::PrivateIdentifier(_))
    }

    #[allow(missing_docs)]
    pub fn private_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::PrivateIdentifier(ident) => Some(ident.name.clone()),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn name(&self) -> Option<Cow<'a, str>> {
        if self.is_private_identifier() {
            self.private_name().map(|name| Cow::Borrowed(name.as_str()))
        } else {
            self.static_name()
        }
    }

    #[allow(missing_docs)]
    pub fn is_specific_id(&self, name: &str) -> bool {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.name == name,
            _ => false,
        }
    }

    #[allow(missing_docs)]
    pub fn is_specific_string_literal(&self, string: &str) -> bool {
        matches!(self, Self::StringLiteral(s) if s.value == string)
    }
}

impl PropertyKind {
    /// Returns `true` if this property is a getter or setter.
    ///
    /// Analogous to [`MethodDefinitionKind::is_accessor`].
    pub fn is_accessor(self) -> bool {
        matches!(self, Self::Get | Self::Set)
    }
}

impl<'a> TemplateLiteral<'a> {
    #[allow(missing_docs)]
    pub fn is_no_substitution_template(&self) -> bool {
        self.expressions.is_empty() && self.quasis.len() == 1
    }

    /// Get single quasi from `template`
    pub fn quasi(&self) -> Option<Atom<'a>> {
        self.quasis.first().and_then(|quasi| quasi.value.cooked.clone())
    }
}

impl<'a> MemberExpression<'a> {
    #[allow(missing_docs)]
    pub fn is_computed(&self) -> bool {
        matches!(self, MemberExpression::ComputedMemberExpression(_))
    }

    #[allow(missing_docs)]
    pub fn optional(&self) -> bool {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => expr.optional,
            MemberExpression::StaticMemberExpression(expr) => expr.optional,
            MemberExpression::PrivateFieldExpression(expr) => expr.optional,
        }
    }

    #[allow(missing_docs)]
    pub fn object(&self) -> &Expression<'a> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => &expr.object,
            MemberExpression::StaticMemberExpression(expr) => &expr.object,
            MemberExpression::PrivateFieldExpression(expr) => &expr.object,
        }
    }

    #[allow(missing_docs)]
    pub fn object_mut(&mut self) -> &mut Expression<'a> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => &mut expr.object,
            MemberExpression::StaticMemberExpression(expr) => &mut expr.object,
            MemberExpression::PrivateFieldExpression(expr) => &mut expr.object,
        }
    }

    #[allow(missing_docs)]
    pub fn static_property_name(&self) -> Option<&'a str> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.static_property_name().map(|name| name.as_str())
            }
            MemberExpression::StaticMemberExpression(expr) => Some(expr.property.name.as_str()),
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    #[allow(missing_docs)]
    pub fn static_property_info(&self) -> Option<(Span, &'a str)> {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => match &expr.expression {
                Expression::StringLiteral(lit) => Some((lit.span, lit.value.as_str())),
                Expression::TemplateLiteral(lit) => {
                    if lit.expressions.is_empty() && lit.quasis.len() == 1 {
                        Some((lit.span, lit.quasis[0].value.raw.as_str()))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            MemberExpression::StaticMemberExpression(expr) => {
                Some((expr.property.span, expr.property.name.as_str()))
            }
            MemberExpression::PrivateFieldExpression(_) => None,
        }
    }

    #[allow(missing_docs)]
    pub fn through_optional_is_specific_member_access(&self, object: &str, property: &str) -> bool {
        let object_matches = match self.object().without_parentheses() {
            Expression::ChainExpression(x) => match x.expression.member_expression() {
                None => false,
                Some(member_expr) => {
                    member_expr.object().without_parentheses().is_specific_id(object)
                }
            },
            x => x.is_specific_id(object),
        };

        let property_matches = self.static_property_name().is_some_and(|p| p == property);

        object_matches && property_matches
    }

    /// Whether it is a static member access `object.property`
    pub fn is_specific_member_access(&self, object: &str, property: &str) -> bool {
        self.object().is_specific_id(object)
            && self.static_property_name().is_some_and(|p| p == property)
    }
}

impl<'a> ComputedMemberExpression<'a> {
    #[allow(missing_docs)]
    pub fn static_property_name(&self) -> Option<Atom<'a>> {
        match &self.expression {
            Expression::StringLiteral(lit) => Some(lit.value.clone()),
            Expression::TemplateLiteral(lit)
                if lit.expressions.is_empty() && lit.quasis.len() == 1 =>
            {
                Some(lit.quasis[0].value.raw.clone())
            }
            Expression::RegExpLiteral(lit) => lit.raw.clone(),
            _ => None,
        }
    }
}

impl<'a> StaticMemberExpression<'a> {
    #[allow(missing_docs)]
    pub fn get_first_object(&self) -> &Expression<'a> {
        let mut object = &self.object;
        loop {
            match object {
                Expression::StaticMemberExpression(member) => {
                    object = &member.object;
                    continue;
                }
                Expression::ChainExpression(chain) => {
                    if let ChainElement::StaticMemberExpression(member) = &chain.expression {
                        object = &member.object;
                        continue;
                    }
                }
                _ => {}
            }

            return object;
        }
    }
}

impl<'a> ChainElement<'a> {
    /// Returns the member expression.
    pub fn member_expression(&self) -> Option<&MemberExpression<'a>> {
        match self {
            ChainElement::TSNonNullExpression(e) => match &e.expression {
                match_member_expression!(Expression) => e.expression.as_member_expression(),
                _ => None,
            },
            _ => self.as_member_expression(),
        }
    }
}

impl CallExpression<'_> {
    #[allow(missing_docs)]
    pub fn callee_name(&self) -> Option<&str> {
        match &self.callee {
            Expression::Identifier(ident) => Some(ident.name.as_str()),
            expr => expr.as_member_expression().and_then(MemberExpression::static_property_name),
        }
    }

    /// Returns `true` if this [`CallExpression`] matches one of these patterns:
    /// ```js
    /// require('string')
    /// require(`string`)
    /// require(`foo${bar}qux`) // Any number of expressions and quasis
    /// ```
    pub fn is_require_call(&self) -> bool {
        if self.arguments.len() != 1 {
            return false;
        }
        if let Expression::Identifier(id) = &self.callee {
            id.name == "require"
                && matches!(
                    self.arguments.first(),
                    Some(Argument::StringLiteral(_) | Argument::TemplateLiteral(_)),
                )
        } else {
            false
        }
    }

    #[allow(missing_docs)]
    pub fn is_symbol_or_symbol_for_call(&self) -> bool {
        // TODO: is 'Symbol' reference to global object
        match &self.callee {
            Expression::Identifier(id) => id.name == "Symbol",
            expr => match expr.as_member_expression() {
                Some(member) => {
                    matches!(member.object(), Expression::Identifier(id) if id.name == "Symbol")
                        && member.static_property_name() == Some("for")
                }
                None => false,
            },
        }
    }

    #[allow(missing_docs)]
    pub fn common_js_require(&self) -> Option<&StringLiteral> {
        if !(self.callee.is_specific_id("require") && self.arguments.len() == 1) {
            return None;
        }
        match &self.arguments[0] {
            Argument::StringLiteral(str_literal) => Some(str_literal),
            _ => None,
        }
    }
}

impl Argument<'_> {
    #[allow(missing_docs)]
    pub fn is_spread(&self) -> bool {
        matches!(self, Self::SpreadElement(_))
    }
}

impl<'a> AssignmentTarget<'a> {
    #[allow(missing_docs)]
    pub fn get_identifier(&self) -> Option<&'a str> {
        self.as_simple_assignment_target().and_then(SimpleAssignmentTarget::get_identifier)
    }

    #[allow(missing_docs)]
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        self.as_simple_assignment_target().and_then(SimpleAssignmentTarget::get_expression)
    }

    #[allow(missing_docs)]
    pub fn get_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        self.as_simple_assignment_target_mut().and_then(SimpleAssignmentTarget::get_expression_mut)
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    #[allow(missing_docs)]
    pub fn get_identifier(&self) -> Option<&'a str> {
        match self {
            Self::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
            match_member_expression!(Self) => self.to_member_expression().static_property_name(),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        match self {
            Self::TSAsExpression(expr) => Some(&expr.expression),
            Self::TSSatisfiesExpression(expr) => Some(&expr.expression),
            Self::TSNonNullExpression(expr) => Some(&expr.expression),
            Self::TSTypeAssertion(expr) => Some(&expr.expression),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn get_expression_mut(&mut self) -> Option<&mut Expression<'a>> {
        match self {
            Self::TSAsExpression(expr) => Some(&mut expr.expression),
            Self::TSSatisfiesExpression(expr) => Some(&mut expr.expression),
            Self::TSNonNullExpression(expr) => Some(&mut expr.expression),
            Self::TSTypeAssertion(expr) => Some(&mut expr.expression),
            Self::TSInstantiationExpression(expr) => Some(&mut expr.expression),
            _ => None,
        }
    }
}

impl<'a> ArrayAssignmentTarget<'a> {
    #[allow(missing_docs)]
    pub fn new_with_elements(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
    ) -> Self {
        Self { span, elements, rest: None, trailing_comma: None }
    }
}

impl<'a> ObjectAssignmentTarget<'a> {
    #[allow(missing_docs)]
    pub fn new_with_properties(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
    ) -> Self {
        Self { span, properties, rest: None }
    }

    #[allow(missing_docs)]
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    #[allow(missing_docs)]
    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

impl AssignmentTargetMaybeDefault<'_> {
    #[allow(missing_docs)]
    pub fn name(&self) -> Option<Atom> {
        match self {
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => Some(id.name.clone()),
            Self::AssignmentTargetWithDefault(target) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(id) = &target.binding {
                    Some(id.name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Statement<'_> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            match_declaration!(Self) => {
                self.as_declaration().is_some_and(Declaration::is_typescript_syntax)
            }
            match_module_declaration!(Self) => {
                self.as_module_declaration().is_some_and(ModuleDeclaration::is_typescript_syntax)
            }
            _ => false,
        }
    }

    #[allow(missing_docs)]
    pub fn is_iteration_statement(&self) -> bool {
        matches!(
            self,
            Statement::DoWhileStatement(_)
                | Statement::ForInStatement(_)
                | Statement::ForOfStatement(_)
                | Statement::ForStatement(_)
                | Statement::WhileStatement(_)
        )
    }
}

impl<'a> FromIn<'a, Expression<'a>> for Statement<'a> {
    fn from_in(expression: Expression<'a>, allocator: &'a oxc_allocator::Allocator) -> Self {
        Statement::ExpressionStatement(Box::from_in(
            ExpressionStatement { span: expression.span(), expression },
            allocator,
        ))
    }
}

impl Directive<'_> {
    /// A Use Strict Directive is an ExpressionStatement in a Directive Prologue whose StringLiteral is either of the exact code point sequences "use strict" or 'use strict'.
    /// A Use Strict Directive may not contain an EscapeSequence or LineContinuation.
    /// <https://tc39.es/ecma262/#sec-directive-prologues-and-the-use-strict-directive>
    pub fn is_use_strict(&self) -> bool {
        self.directive == "use strict"
    }
}

impl<'a> Declaration<'a> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::VariableDeclaration(decl) => decl.is_typescript_syntax(),
            Self::FunctionDeclaration(func) => func.is_typescript_syntax(),
            Self::ClassDeclaration(class) => class.is_typescript_syntax(),
            _ => true,
        }
    }

    /// Get the identifier bound by this declaration.
    ///
    /// ## Example
    /// ```ts
    /// const x = 1; // None. may change in the future.
    /// class Foo {} // Some(IdentifierReference { name: "Foo", .. })
    /// enum Bar {} // Some(IdentifierReference { name: "Bar", .. })
    /// ```
    pub fn id(&self) -> Option<&BindingIdentifier<'a>> {
        match self {
            Declaration::FunctionDeclaration(decl) => decl.id.as_ref(),
            Declaration::ClassDeclaration(decl) => decl.id.as_ref(),
            Declaration::TSTypeAliasDeclaration(decl) => Some(&decl.id),
            Declaration::TSInterfaceDeclaration(decl) => Some(&decl.id),
            Declaration::TSEnumDeclaration(decl) => Some(&decl.id),
            Declaration::TSImportEqualsDeclaration(decl) => Some(&decl.id),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn declare(&self) -> bool {
        match self {
            Declaration::VariableDeclaration(decl) => decl.declare,
            Declaration::FunctionDeclaration(decl) => decl.declare,
            Declaration::ClassDeclaration(decl) => decl.declare,
            Declaration::TSEnumDeclaration(decl) => decl.declare,
            Declaration::TSTypeAliasDeclaration(decl) => decl.declare,
            Declaration::TSModuleDeclaration(decl) => decl.declare,
            Declaration::TSInterfaceDeclaration(decl) => decl.declare,
            Declaration::TSImportEqualsDeclaration(_) => false,
        }
    }
}

impl VariableDeclaration<'_> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        self.declare
    }

    /// Returns `true` if any of this declaration's variables have an initializer.
    pub fn has_init(&self) -> bool {
        self.declarations.iter().any(|decl| decl.init.is_some())
    }
}

impl VariableDeclarationKind {
    /// `var x`
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var)
    }

    /// `const x`
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }

    /// `let x` or `const x`
    pub fn is_lexical(&self) -> bool {
        matches!(self, Self::Const | Self::Let)
    }

    /// `await using x`
    pub fn is_await(&self) -> bool {
        matches!(self, Self::AwaitUsing)
    }

    #[allow(missing_docs)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Const => "const",
            Self::Let => "let",
            Self::Using => "using",
            Self::AwaitUsing => "await using",
        }
    }
}

impl fmt::Display for VariableDeclarationKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self.as_str();
        write!(f, "{s}")
    }
}

impl ForStatementInit<'_> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

impl ForStatementLeft<'_> {
    /// LexicalDeclaration[In, Yield, Await] :
    ///   LetOrConst BindingList[?In, ?Yield, ?Await] ;
    pub fn is_lexical_declaration(&self) -> bool {
        matches!(self, Self::VariableDeclaration(decl) if decl.kind.is_lexical())
    }
}

impl SwitchCase<'_> {
    /// `true` for `default:` cases.
    pub fn is_default_case(&self) -> bool {
        self.test.is_none()
    }
}

impl<'a> BindingPattern<'a> {
    #[allow(missing_docs)]
    pub fn get_identifier(&self) -> Option<Atom<'a>> {
        self.kind.get_identifier()
    }

    #[allow(missing_docs)]
    pub fn get_binding_identifier(&self) -> Option<&BindingIdentifier<'a>> {
        self.kind.get_binding_identifier()
    }
}

impl<'a> BindingPatternKind<'a> {
    #[allow(missing_docs)]
    pub fn get_identifier(&self) -> Option<Atom<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident.name.clone()),
            Self::AssignmentPattern(assign) => assign.left.get_identifier(),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn get_binding_identifier(&self) -> Option<&BindingIdentifier<'a>> {
        match self {
            Self::BindingIdentifier(ident) => Some(ident),
            Self::AssignmentPattern(assign) => assign.left.get_binding_identifier(),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    pub fn is_destructuring_pattern(&self) -> bool {
        match self {
            Self::ObjectPattern(_) | Self::ArrayPattern(_) => true,
            Self::AssignmentPattern(pattern) => pattern.left.kind.is_destructuring_pattern(),
            Self::BindingIdentifier(_) => false,
        }
    }

    #[allow(missing_docs)]
    pub fn is_binding_identifier(&self) -> bool {
        matches!(self, Self::BindingIdentifier(_))
    }

    #[allow(missing_docs)]
    pub fn is_object_pattern(&self) -> bool {
        matches!(self, Self::ObjectPattern(_))
    }

    #[allow(missing_docs)]
    pub fn is_array_pattern(&self) -> bool {
        matches!(self, Self::ArrayPattern(_))
    }

    #[allow(missing_docs)]
    pub fn is_assignment_pattern(&self) -> bool {
        matches!(self, Self::AssignmentPattern(_))
    }
}

impl ObjectPattern<'_> {
    /// `true` for empty object patterns (`{}`).
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty() && self.rest.is_none()
    }

    /// The number of properties, including rest properties, in this object pattern.
    pub fn len(&self) -> usize {
        self.properties.len() + usize::from(self.rest.is_some())
    }
}

impl ArrayPattern<'_> {
    /// `true` for empty array patterns (`[]`).
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty() && self.rest.is_none()
    }

    /// The number of elements, including rest elements, in this array pattern.
    pub fn len(&self) -> usize {
        self.elements.len() + usize::from(self.rest.is_some())
    }
}

impl<'a> Function<'a> {
    /// Returns this [`Function`]'s name, if it has one.
    #[inline]
    pub fn name(&self) -> Option<Atom<'a>> {
        self.id.as_ref().map(|id| id.name.clone())
    }

    /// Get the [`SymbolId`] this [`Function`] is bound to.
    ///
    /// Returns [`None`] for anonymous functions.
    #[inline]
    pub fn symbol_id(&self) -> Option<SymbolId> {
        self.id.as_ref().map(BindingIdentifier::symbol_id)
    }

    /// `true` for overload signatures and `declare function` statements.
    pub fn is_typescript_syntax(&self) -> bool {
        matches!(
            self.r#type,
            FunctionType::TSDeclareFunction | FunctionType::TSEmptyBodyFunctionExpression
        ) || self.body.is_none()
            || self.declare
    }

    /// `true` for function expressions
    pub fn is_expression(&self) -> bool {
        self.r#type == FunctionType::FunctionExpression
    }

    /// `true` for function declarations
    pub fn is_function_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration)
    }

    /// `true` for `declare function` statements
    pub fn is_ts_declare_function(&self) -> bool {
        matches!(self.r#type, FunctionType::TSDeclareFunction)
    }

    /// `true` for non-expression functions
    pub fn is_declaration(&self) -> bool {
        matches!(self.r#type, FunctionType::FunctionDeclaration | FunctionType::TSDeclareFunction)
    }

    /// Returns `true` if this function's body has a `"use strict"` directive.
    pub fn has_use_strict_directive(&self) -> bool {
        self.body.as_ref().is_some_and(|body| body.has_use_strict_directive())
    }
}

impl<'a> FormalParameters<'a> {
    /// Number of parameters bound in this parameter list.
    pub fn parameters_count(&self) -> usize {
        self.items.len() + self.rest.as_ref().map_or(0, |_| 1)
    }

    /// Iterates over all bound parameters, including rest parameters.
    pub fn iter_bindings(&self) -> impl Iterator<Item = &BindingPattern<'a>> + '_ {
        self.items
            .iter()
            .map(|param| &param.pattern)
            .chain(self.rest.iter().map(|rest| &rest.argument))
    }
}

impl FormalParameter<'_> {
    /// `true` if a `public` accessibility modifier is present. Use
    /// [`has_modifier`](FormalParameter::has_modifier) if you want to check for
    /// _any_ modifier, including `readonly` and `override`.
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///     constructor(
    ///         public x: number,  // <- true
    ///         private y: string, // <- false
    ///         z: string          // <- false
    ///     ) {}
    /// }
    pub fn is_public(&self) -> bool {
        matches!(self.accessibility, Some(TSAccessibility::Public))
    }

    /// `true` if any modifier, accessibility or otherwise, is present.
    ///
    /// ## Example
    /// ```ts
    /// class Foo {
    ///     constructor(
    ///         public a: number,   // <- true
    ///         readonly b: string, // <- true
    ///         override c: string, // <- true
    ///         d: string           // <- false
    ///    ) {}
    /// }
    /// ```
    #[inline]
    pub fn has_modifier(&self) -> bool {
        self.accessibility.is_some() || self.readonly || self.r#override
    }
}

impl FormalParameterKind {
    /// `true` when part of a TypeScript method or function signature.
    pub fn is_signature(&self) -> bool {
        matches!(self, Self::Signature)
    }
}

impl FormalParameters<'_> {
    /// `true` if no parameters are bound.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `true` if at least one parameter is bound, including [rest bindings](BindingRestElement).
    pub fn has_parameter(&self) -> bool {
        !self.is_empty() || self.rest.is_some()
    }
}

impl FunctionBody<'_> {
    /// `true` if this function body contains no statements or directives.
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty() && self.statements.is_empty()
    }

    /// `true` if this function body contains a `"use strict"` directive.
    pub fn has_use_strict_directive(&self) -> bool {
        self.directives.iter().any(Directive::is_use_strict)
    }
}

impl<'a> ArrowFunctionExpression<'a> {
    /// Get expression part of `ArrowFunctionExpression`: `() => expression_part`.
    pub fn get_expression(&self) -> Option<&Expression<'a>> {
        if self.expression {
            if let Statement::ExpressionStatement(expr_stmt) = &self.body.statements[0] {
                return Some(&expr_stmt.expression);
            }
        }
        None
    }

    /// Returns `true` if this arrow function's body has a `"use strict"` directive.
    pub fn has_use_strict_directive(&self) -> bool {
        self.body.has_use_strict_directive()
    }
}

impl Class<'_> {
    /// `true` if this [`Class`] is an expression.
    ///
    /// For example,
    /// ```ts
    /// var Foo = class { /* ... */ }
    /// ```
    pub fn is_expression(&self) -> bool {
        self.r#type == ClassType::ClassExpression
    }

    /// `true` if this [`Class`] is a declaration statement.
    ///
    /// For example,
    /// ```ts
    /// class Foo {
    ///   // ...
    /// }
    /// ```
    pub fn is_declaration(&self) -> bool {
        self.r#type == ClassType::ClassDeclaration
    }

    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        self.declare || self.r#abstract
    }
}

impl<'a> ClassElement<'a> {
    /// Returns `true` if this is a [`ClassElement::StaticBlock`].
    pub fn is_static_block(&self) -> bool {
        matches!(self, Self::StaticBlock(_))
    }

    /// Returns `true` if this [`ClassElement`] has a static modifier.
    ///
    /// Note: Class static blocks do not have a "modifier", as there is no non-static equivalent.
    /// Therefore, returns `false` for static blocks.
    ///
    /// The following all return `true`:
    /// ```ts
    /// class {
    ///   static prop = 1;
    ///   static method() {}
    ///   static #private = 2;
    ///   static #privateMethod() {}
    ///   static accessor accessorProp = 3;
    ///   static accessor #privateAccessorProp = 4;
    /// }
    /// ```
    pub fn r#static(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.r#static,
            Self::PropertyDefinition(def) => def.r#static,
            Self::AccessorProperty(def) => def.r#static,
        }
    }

    #[allow(missing_docs)]
    pub fn computed(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => false,
            Self::MethodDefinition(def) => def.computed,
            Self::PropertyDefinition(def) => def.computed,
            Self::AccessorProperty(def) => def.computed,
        }
    }

    #[allow(missing_docs)]
    pub fn accessibility(&self) -> Option<TSAccessibility> {
        match self {
            Self::StaticBlock(_) | Self::TSIndexSignature(_) | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => def.accessibility,
            Self::PropertyDefinition(def) => def.accessibility,
        }
    }

    #[allow(missing_docs)]
    pub fn method_definition_kind(&self) -> Option<MethodDefinitionKind> {
        match self {
            Self::TSIndexSignature(_)
            | Self::StaticBlock(_)
            | Self::PropertyDefinition(_)
            | Self::AccessorProperty(_) => None,
            Self::MethodDefinition(def) => Some(def.kind),
        }
    }

    #[allow(missing_docs)]
    pub fn property_key(&self) -> Option<&PropertyKey<'a>> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => Some(&def.key),
            Self::PropertyDefinition(def) => Some(&def.key),
            Self::AccessorProperty(def) => Some(&def.key),
        }
    }

    /// Try to get the statically known name of this [`ClassElement`]. Handles
    /// computed members that use literals.
    pub fn static_name(&self) -> Option<Cow<'a, str>> {
        match self {
            Self::TSIndexSignature(_) | Self::StaticBlock(_) => None,
            Self::MethodDefinition(def) => def.key.static_name(),
            Self::PropertyDefinition(def) => def.key.static_name(),
            Self::AccessorProperty(def) => def.key.static_name(),
        }
    }

    /// Returns `true` if this [`ClassElement`] is a property or accessor
    pub fn is_property(&self) -> bool {
        matches!(self, Self::PropertyDefinition(_) | Self::AccessorProperty(_))
    }

    /// `true` for overloads, declarations, index signatures, and abstract
    /// methods, etc. That is, any non-concrete implementation.
    pub fn is_ts_empty_body_function(&self) -> bool {
        match self {
            Self::PropertyDefinition(_)
            | Self::StaticBlock(_)
            | Self::AccessorProperty(_)
            | Self::TSIndexSignature(_) => false,
            Self::MethodDefinition(method) => method.value.body.is_none(),
        }
    }

    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::TSIndexSignature(_) => true,
            Self::MethodDefinition(method) => method.value.is_typescript_syntax(),
            Self::PropertyDefinition(property) => {
                property.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
            }
            Self::AccessorProperty(property) => property.r#type.is_abstract(),
            Self::StaticBlock(_) => false,
        }
    }

    /// `true` for [decorated](Decorator) class elements.
    pub fn has_decorator(&self) -> bool {
        match self {
            Self::MethodDefinition(method) => !method.decorators.is_empty(),
            Self::PropertyDefinition(property) => !property.decorators.is_empty(),
            Self::AccessorProperty(property) => !property.decorators.is_empty(),
            Self::StaticBlock(_) | Self::TSIndexSignature(_) => false,
        }
    }

    /// Has this property been marked as abstract?
    ///
    /// ```ts
    /// abstract class Foo {    // <-- not considered
    ///   foo: string;          // <-- false
    ///   abstract bar: string; // <-- true
    /// }
    /// ```
    pub fn is_abstract(&self) -> bool {
        match self {
            Self::MethodDefinition(method) => method.r#type.is_abstract(),
            Self::AccessorProperty(accessor) => accessor.r#type.is_abstract(),
            Self::PropertyDefinition(property) => property.r#type.is_abstract(),
            Self::StaticBlock(_) | Self::TSIndexSignature(_) => false,
        }
    }
}

impl PropertyDefinitionType {
    /// `true` for abstract properties and methods.
    pub fn is_abstract(&self) -> bool {
        matches!(self, Self::TSAbstractPropertyDefinition)
    }
}

impl MethodDefinitionKind {
    /// `true` for constructors.
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Constructor)
    }

    /// `true` for regular methods.
    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method)
    }

    /// `true` for setter methods.
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set)
    }

    /// `true` for getter methods.
    pub fn is_get(&self) -> bool {
        matches!(self, Self::Get)
    }

    /// Returns `true` if this method is a getter or a setter.
    ///
    /// Analogous to [`PropertyKind::is_accessor`].
    pub fn is_accessor(&self) -> bool {
        matches!(self, Self::Get | Self::Set)
    }

    #[allow(missing_docs)]
    pub fn scope_flags(self) -> ScopeFlags {
        match self {
            Self::Constructor => ScopeFlags::Constructor | ScopeFlags::Function,
            Self::Method => ScopeFlags::Function,
            Self::Get => ScopeFlags::GetAccessor | ScopeFlags::Function,
            Self::Set => ScopeFlags::SetAccessor | ScopeFlags::Function,
        }
    }
}

impl MethodDefinitionType {
    #[allow(missing_docs)]
    pub fn is_abstract(&self) -> bool {
        matches!(self, Self::TSAbstractMethodDefinition)
    }
}

impl<'a> ModuleDeclaration<'a> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            ModuleDeclaration::ImportDeclaration(_) => false,
            ModuleDeclaration::ExportDefaultDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::ExportNamedDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::ExportAllDeclaration(decl) => decl.is_typescript_syntax(),
            ModuleDeclaration::TSNamespaceExportDeclaration(_)
            | ModuleDeclaration::TSExportAssignment(_) => true,
        }
    }

    #[allow(missing_docs)]
    pub fn is_import(&self) -> bool {
        matches!(self, Self::ImportDeclaration(_))
    }

    #[allow(missing_docs)]
    pub fn is_export(&self) -> bool {
        matches!(
            self,
            Self::ExportAllDeclaration(_)
                | Self::ExportDefaultDeclaration(_)
                | Self::ExportNamedDeclaration(_)
                | Self::TSExportAssignment(_)
                | Self::TSNamespaceExportDeclaration(_)
        )
    }

    #[allow(missing_docs)]
    pub fn is_default_export(&self) -> bool {
        matches!(self, Self::ExportDefaultDeclaration(_))
    }

    #[allow(missing_docs)]
    pub fn source(&self) -> Option<&StringLiteral<'a>> {
        match self {
            Self::ImportDeclaration(decl) => Some(&decl.source),
            Self::ExportAllDeclaration(decl) => Some(&decl.source),
            Self::ExportNamedDeclaration(decl) => decl.source.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }

    #[allow(missing_docs)]
    pub fn with_clause(&self) -> Option<&Box<'a, WithClause<'a>>> {
        match self {
            Self::ImportDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportAllDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportNamedDeclaration(decl) => decl.with_clause.as_ref(),
            Self::ExportDefaultDeclaration(_)
            | Self::TSExportAssignment(_)
            | Self::TSNamespaceExportDeclaration(_) => None,
        }
    }
}

impl AccessorPropertyType {
    #[allow(missing_docs)]
    pub fn is_abstract(&self) -> bool {
        matches!(self, Self::TSAbstractAccessorProperty)
    }
}

impl<'a> ImportDeclarationSpecifier<'a> {
    #[allow(missing_docs)]
    pub fn local(&self) -> &BindingIdentifier<'a> {
        match self {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
        }
    }

    #[allow(missing_docs)]
    pub fn name(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.local().name.as_str())
    }
}

impl<'a> ImportAttributeKey<'a> {
    #[allow(missing_docs)]
    pub fn as_atom(&self) -> Atom<'a> {
        match self {
            Self::Identifier(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }
}

impl ExportNamedDeclaration<'_> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind == ImportOrExportKind::Type
            || self.declaration.as_ref().map_or(false, Declaration::is_typescript_syntax)
    }
}

impl ExportDefaultDeclaration<'_> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        self.declaration.is_typescript_syntax()
    }
}

impl ExportAllDeclaration<'_> {
    #[allow(missing_docs)]
    pub fn is_typescript_syntax(&self) -> bool {
        self.export_kind.is_type()
    }
}

impl ExportDefaultDeclarationKind<'_> {
    #[allow(missing_docs)]
    #[inline]
    pub fn is_typescript_syntax(&self) -> bool {
        match self {
            Self::FunctionDeclaration(func) => func.is_typescript_syntax(),
            Self::ClassDeclaration(class) => class.is_typescript_syntax(),
            Self::TSInterfaceDeclaration(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ModuleExportName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::IdentifierName(identifier) => identifier.name.to_string(),
            Self::IdentifierReference(identifier) => identifier.name.to_string(),
            Self::StringLiteral(literal) => format!(r#""{}""#, literal.value),
        };
        write!(f, "{s}")
    }
}

impl<'a> ModuleExportName<'a> {
    #[allow(missing_docs)]
    pub fn name(&self) -> Atom<'a> {
        match self {
            Self::IdentifierName(identifier) => identifier.name.clone(),
            Self::IdentifierReference(identifier) => identifier.name.clone(),
            Self::StringLiteral(literal) => literal.value.clone(),
        }
    }

    #[allow(missing_docs)]
    pub fn identifier_name(&self) -> Option<Atom<'a>> {
        match self {
            Self::IdentifierName(identifier) => Some(identifier.name.clone()),
            Self::IdentifierReference(identifier) => Some(identifier.name.clone()),
            Self::StringLiteral(_) => None,
        }
    }
}

impl ImportPhase {
    #[allow(missing_docs)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Defer => "defer",
        }
    }
}
