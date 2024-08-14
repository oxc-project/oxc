// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_builder.rs`

#![allow(
    clippy::default_trait_access,
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools
)]

use oxc_allocator::{Allocator, Box, IntoIn, Vec};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// AST builder for creating AST nodes
#[derive(Clone, Copy)]
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    /// Builds a [`BooleanLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_boolean_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }

    /// Builds a [`BooleanLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::boolean_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn alloc_boolean_literal(self, span: Span, value: bool) -> Box<'a, BooleanLiteral> {
        Box::new_in(self.boolean_literal(span, value), self.allocator)
    }

    /// Builds a [`NullLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_null_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn null_literal(self, span: Span) -> NullLiteral {
        NullLiteral { span }
    }

    /// Builds a [`NullLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::null_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_null_literal(self, span: Span) -> Box<'a, NullLiteral> {
        Box::new_in(self.null_literal(span), self.allocator)
    }

    /// Builds a [`NumericLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_numeric_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn numeric_literal<S>(
        self,
        span: Span,
        value: f64,
        raw: S,
        base: NumberBase,
    ) -> NumericLiteral<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        NumericLiteral { span, value, raw: raw.into_in(self.allocator), base }
    }

    /// Builds a [`NumericLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::numeric_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn alloc_numeric_literal<S>(
        self,
        span: Span,
        value: f64,
        raw: S,
        base: NumberBase,
    ) -> Box<'a, NumericLiteral<'a>>
    where
        S: IntoIn<'a, &'a str>,
    {
        Box::new_in(self.numeric_literal(span, value, raw, base), self.allocator)
    }

    /// Builds a [`BigIntLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_big_int_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - raw: The bigint as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn big_int_literal<A>(self, span: Span, raw: A, base: BigintBase) -> BigIntLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BigIntLiteral { span, raw: raw.into_in(self.allocator), base }
    }

    /// Builds a [`BigIntLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::big_int_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - raw: The bigint as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn alloc_big_int_literal<A>(
        self,
        span: Span,
        raw: A,
        base: BigintBase,
    ) -> Box<'a, BigIntLiteral<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.big_int_literal(span, raw, base), self.allocator)
    }

    /// Builds a [`RegExpLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    /// - regex
    #[inline]
    pub fn reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral { span, value, regex }
    }

    /// Builds a [`RegExpLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    /// - regex
    #[inline]
    pub fn alloc_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> Box<'a, RegExpLiteral<'a>> {
        Box::new_in(self.reg_exp_literal(span, value, regex), self.allocator)
    }

    /// Builds a [`StringLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_string_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn string_literal<A>(self, span: Span, value: A) -> StringLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        StringLiteral { span, value: value.into_in(self.allocator) }
    }

    /// Builds a [`StringLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::string_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn alloc_string_literal<A>(self, span: Span, value: A) -> Box<'a, StringLiteral<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.string_literal(span, value), self.allocator)
    }

    /// Builds a [`Program`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_program`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - hashbang
    /// - directives
    /// - body
    #[inline]
    pub fn program(
        self,
        span: Span,
        source_type: SourceType,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program { span, source_type, hashbang, directives, body, scope_id: Default::default() }
    }

    /// Builds a [`Program`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::program`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - hashbang
    /// - directives
    /// - body
    #[inline]
    pub fn alloc_program(
        self,
        span: Span,
        source_type: SourceType,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, Program<'a>> {
        Box::new_in(self.program(span, source_type, hashbang, directives, body), self.allocator)
    }

    /// Build a [`Expression::BooleanLiteral`]
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn expression_boolean_literal(self, span: Span, value: bool) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    /// Convert a [`BooleanLiteral`] into a [`Expression::BooleanLiteral`]
    #[inline]
    pub fn expression_from_boolean_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BooleanLiteral>>,
    {
        Expression::BooleanLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::NullLiteral`]
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn expression_null_literal(self, span: Span) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(self.null_literal(span)))
    }

    /// Convert a [`NullLiteral`] into a [`Expression::NullLiteral`]
    #[inline]
    pub fn expression_from_null_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NullLiteral>>,
    {
        Expression::NullLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::NumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn expression_numeric_literal<S>(
        self,
        span: Span,
        value: f64,
        raw: S,
        base: NumberBase,
    ) -> Expression<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        Expression::NumericLiteral(self.alloc(self.numeric_literal(span, value, raw, base)))
    }

    /// Convert a [`NumericLiteral`] into a [`Expression::NumericLiteral`]
    #[inline]
    pub fn expression_from_numeric_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NumericLiteral<'a>>>,
    {
        Expression::NumericLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::BigIntLiteral`]
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - raw: The bigint as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn expression_big_int_literal<A>(
        self,
        span: Span,
        raw: A,
        base: BigintBase,
    ) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::BigIntLiteral(self.alloc(self.big_int_literal(span, raw, base)))
    }

    /// Convert a [`BigIntLiteral`] into a [`Expression::BigIntLiteral`]
    #[inline]
    pub fn expression_from_big_int_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BigIntLiteral<'a>>>,
    {
        Expression::BigIntLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::RegExpLiteral`]
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    /// - regex
    #[inline]
    pub fn expression_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc(self.reg_exp_literal(span, value, regex)))
    }

    /// Convert a [`RegExpLiteral`] into a [`Expression::RegExpLiteral`]
    #[inline]
    pub fn expression_from_reg_exp_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, RegExpLiteral<'a>>>,
    {
        Expression::RegExpLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn expression_string_literal<A>(self, span: Span, value: A) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    /// Convert a [`StringLiteral`] into a [`Expression::StringLiteral`]
    #[inline]
    pub fn expression_from_string_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        Expression::StringLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TemplateLiteral`]
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - expressions
    #[inline]
    pub fn expression_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(self.template_literal(span, quasis, expressions)))
    }

    /// Convert a [`TemplateLiteral`] into a [`Expression::TemplateLiteral`]
    #[inline]
    pub fn expression_from_template_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TemplateLiteral<'a>>>,
    {
        Expression::TemplateLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::Identifier`]
    ///
    /// This node contains a [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn expression_identifier_reference<A>(self, span: Span, name: A) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::Identifier(self.alloc(self.identifier_reference(span, name)))
    }

    /// Convert a [`IdentifierReference`] into a [`Expression::Identifier`]
    #[inline]
    pub fn expression_from_identifier_reference<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierReference<'a>>>,
    {
        Expression::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::MetaProperty`]
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - meta
    /// - property
    #[inline]
    pub fn expression_meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Expression<'a> {
        Expression::MetaProperty(self.alloc(self.meta_property(span, meta, property)))
    }

    /// Convert a [`MetaProperty`] into a [`Expression::MetaProperty`]
    #[inline]
    pub fn expression_from_meta_property<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, MetaProperty<'a>>>,
    {
        Expression::MetaProperty(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::Super`]
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn expression_super(self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc(self.super_(span)))
    }

    /// Convert a [`Super`] into a [`Expression::Super`]
    #[inline]
    pub fn expression_from_super<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Super>>,
    {
        Expression::Super(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ArrayExpression`]
    ///
    /// This node contains a [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - trailing_comma: Array trailing comma
    #[inline]
    pub fn expression_array(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ArrayExpression(self.alloc(self.array_expression(
            span,
            elements,
            trailing_comma,
        )))
    }

    /// Convert a [`ArrayExpression`] into a [`Expression::ArrayExpression`]
    #[inline]
    pub fn expression_from_array<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ArrayExpression<'a>>>,
    {
        Expression::ArrayExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ArrowFunctionExpression`]
    ///
    /// This node contains a [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// - r#async
    /// - type_parameters
    /// - params
    /// - return_type
    /// - body: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn expression_arrow_function<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Expression::ArrowFunctionExpression(self.alloc(self.arrow_function_expression(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
        )))
    }

    /// Convert a [`ArrowFunctionExpression`] into a [`Expression::ArrowFunctionExpression`]
    #[inline]
    pub fn expression_from_arrow_function<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ArrowFunctionExpression<'a>>>,
    {
        Expression::ArrowFunctionExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::AssignmentExpression`]
    ///
    /// This node contains a [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - left
    /// - right
    #[inline]
    pub fn expression_assignment(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::AssignmentExpression(
            self.alloc(self.assignment_expression(span, operator, left, right)),
        )
    }

    /// Convert a [`AssignmentExpression`] into a [`Expression::AssignmentExpression`]
    #[inline]
    pub fn expression_from_assignment<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentExpression<'a>>>,
    {
        Expression::AssignmentExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::AwaitExpression`]
    ///
    /// This node contains a [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn expression_await(self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(self.await_expression(span, argument)))
    }

    /// Convert a [`AwaitExpression`] into a [`Expression::AwaitExpression`]
    #[inline]
    pub fn expression_from_await<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, AwaitExpression<'a>>>,
    {
        Expression::AwaitExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::BinaryExpression`]
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn expression_binary(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::BinaryExpression(
            self.alloc(self.binary_expression(span, left, operator, right)),
        )
    }

    /// Convert a [`BinaryExpression`] into a [`Expression::BinaryExpression`]
    #[inline]
    pub fn expression_from_binary<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BinaryExpression<'a>>>,
    {
        Expression::BinaryExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::CallExpression`]
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - arguments
    /// - callee
    /// - type_parameters
    /// - optional
    #[inline]
    pub fn expression_call<T1>(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: T1,
        optional: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::CallExpression(self.alloc(self.call_expression(
            span,
            arguments,
            callee,
            type_parameters,
            optional,
        )))
    }

    /// Convert a [`CallExpression`] into a [`Expression::CallExpression`]
    #[inline]
    pub fn expression_from_call<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, CallExpression<'a>>>,
    {
        Expression::CallExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ChainExpression`]
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn expression_chain(self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc(self.chain_expression(span, expression)))
    }

    /// Convert a [`ChainExpression`] into a [`Expression::ChainExpression`]
    #[inline]
    pub fn expression_from_chain<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ChainExpression<'a>>>,
    {
        Expression::ChainExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ClassExpression`]
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the class.
    /// - id: Class identifier, AKA the name
    /// - type_parameters
    /// - super_class: Super class. When present, this will usually be an [`IdentifierReference`].
    /// - super_type_parameters: Type parameters passed to super class.
    /// - implements: Interface implementation clause for TypeScript classes.
    /// - body
    /// - r#abstract: Whether the class is abstract
    /// - declare: Whether the class was `declare`ed
    #[inline]
    pub fn expression_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_parameters: T2,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Expression::ClassExpression(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_parameters,
            implements,
            body,
            r#abstract,
            declare,
        )))
    }

    /// Convert a [`Class`] into a [`Expression::ClassExpression`]
    #[inline]
    pub fn expression_from_class<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Class<'a>>>,
    {
        Expression::ClassExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ConditionalExpression`]
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn expression_conditional(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ConditionalExpression(
            self.alloc(self.conditional_expression(span, test, consequent, alternate)),
        )
    }

    /// Convert a [`ConditionalExpression`] into a [`Expression::ConditionalExpression`]
    #[inline]
    pub fn expression_from_conditional<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ConditionalExpression<'a>>>,
    {
        Expression::ConditionalExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::FunctionExpression`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id
    /// - generator
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params
    /// - return_type
    /// - body
    #[inline]
    pub fn expression_function<T1, T2, T3, T4>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Expression::FunctionExpression(self.alloc(self.function(
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
        )))
    }

    /// Convert a [`Function`] into a [`Expression::FunctionExpression`]
    #[inline]
    pub fn expression_from_function<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Expression::FunctionExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ImportExpression`]
    ///
    /// This node contains a [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source
    /// - arguments
    #[inline]
    pub fn expression_import(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc(self.import_expression(span, source, arguments)))
    }

    /// Convert a [`ImportExpression`] into a [`Expression::ImportExpression`]
    #[inline]
    pub fn expression_from_import<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ImportExpression<'a>>>,
    {
        Expression::ImportExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::LogicalExpression`]
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn expression_logical(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::LogicalExpression(
            self.alloc(self.logical_expression(span, left, operator, right)),
        )
    }

    /// Convert a [`LogicalExpression`] into a [`Expression::LogicalExpression`]
    #[inline]
    pub fn expression_from_logical<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, LogicalExpression<'a>>>,
    {
        Expression::LogicalExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::NewExpression`]
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - arguments
    /// - type_parameters
    #[inline]
    pub fn expression_new<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: T1,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::NewExpression(self.alloc(self.new_expression(
            span,
            callee,
            arguments,
            type_parameters,
        )))
    }

    /// Convert a [`NewExpression`] into a [`Expression::NewExpression`]
    #[inline]
    pub fn expression_from_new<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NewExpression<'a>>>,
    {
        Expression::NewExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ObjectExpression`]
    ///
    /// This node contains a [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties: Properties declared in the object
    /// - trailing_comma
    #[inline]
    pub fn expression_object(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> Expression<'a> {
        Expression::ObjectExpression(self.alloc(self.object_expression(
            span,
            properties,
            trailing_comma,
        )))
    }

    /// Convert a [`ObjectExpression`] into a [`Expression::ObjectExpression`]
    #[inline]
    pub fn expression_from_object<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectExpression<'a>>>,
    {
        Expression::ObjectExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ParenthesizedExpression`]
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn expression_parenthesized(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ParenthesizedExpression(
            self.alloc(self.parenthesized_expression(span, expression)),
        )
    }

    /// Convert a [`ParenthesizedExpression`] into a [`Expression::ParenthesizedExpression`]
    #[inline]
    pub fn expression_from_parenthesized<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ParenthesizedExpression<'a>>>,
    {
        Expression::ParenthesizedExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::SequenceExpression`]
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expressions
    #[inline]
    pub fn expression_sequence(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc(self.sequence_expression(span, expressions)))
    }

    /// Convert a [`SequenceExpression`] into a [`Expression::SequenceExpression`]
    #[inline]
    pub fn expression_from_sequence<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, SequenceExpression<'a>>>,
    {
        Expression::SequenceExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TaggedTemplateExpression`]
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - tag
    /// - quasi
    /// - type_parameters
    #[inline]
    pub fn expression_tagged_template<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: T1,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::TaggedTemplateExpression(self.alloc(self.tagged_template_expression(
            span,
            tag,
            quasi,
            type_parameters,
        )))
    }

    /// Convert a [`TaggedTemplateExpression`] into a [`Expression::TaggedTemplateExpression`]
    #[inline]
    pub fn expression_from_tagged_template<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TaggedTemplateExpression<'a>>>,
    {
        Expression::TaggedTemplateExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::ThisExpression`]
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn expression_this(self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(self.this_expression(span)))
    }

    /// Convert a [`ThisExpression`] into a [`Expression::ThisExpression`]
    #[inline]
    pub fn expression_from_this<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ThisExpression>>,
    {
        Expression::ThisExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::UnaryExpression`]
    ///
    /// This node contains a [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - argument
    #[inline]
    pub fn expression_unary(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc(self.unary_expression(span, operator, argument)))
    }

    /// Convert a [`UnaryExpression`] into a [`Expression::UnaryExpression`]
    #[inline]
    pub fn expression_from_unary<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, UnaryExpression<'a>>>,
    {
        Expression::UnaryExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::UpdateExpression`]
    ///
    /// This node contains a [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - prefix
    /// - argument
    #[inline]
    pub fn expression_update(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a> {
        Expression::UpdateExpression(
            self.alloc(self.update_expression(span, operator, prefix, argument)),
        )
    }

    /// Convert a [`UpdateExpression`] into a [`Expression::UpdateExpression`]
    #[inline]
    pub fn expression_from_update<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, UpdateExpression<'a>>>,
    {
        Expression::UpdateExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::YieldExpression`]
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - delegate
    /// - argument
    #[inline]
    pub fn expression_yield(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc(self.yield_expression(span, delegate, argument)))
    }

    /// Convert a [`YieldExpression`] into a [`Expression::YieldExpression`]
    #[inline]
    pub fn expression_from_yield<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, YieldExpression<'a>>>,
    {
        Expression::YieldExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::PrivateInExpression`]
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn expression_private_in(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::PrivateInExpression(
            self.alloc(self.private_in_expression(span, left, operator, right)),
        )
    }

    /// Convert a [`PrivateInExpression`] into a [`Expression::PrivateInExpression`]
    #[inline]
    pub fn expression_from_private_in<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, PrivateInExpression<'a>>>,
    {
        Expression::PrivateInExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::JSXElement`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_element: Opening tag of the element.
    /// - closing_element: Closing tag of the element. Will be [`None`] for self-closing tags.
    /// - children: Children of the element. This can be text, other elements, or expressions.
    #[inline]
    pub fn expression_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        closing_element: T2,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Expression::JSXElement(self.alloc(self.jsx_element(
            span,
            opening_element,
            closing_element,
            children,
        )))
    }

    /// Convert a [`JSXElement`] into a [`Expression::JSXElement`]
    #[inline]
    pub fn expression_from_jsx_element<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        Expression::JSXElement(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::JSXFragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_fragment: `<>`
    /// - closing_fragment: `</>`
    /// - children: Elements inside the fragment.
    #[inline]
    pub fn expression_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Expression<'a> {
        Expression::JSXFragment(self.alloc(self.jsx_fragment(
            span,
            opening_fragment,
            closing_fragment,
            children,
        )))
    }

    /// Convert a [`JSXFragment`] into a [`Expression::JSXFragment`]
    #[inline]
    pub fn expression_from_jsx_fragment<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        Expression::JSXFragment(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TSAsExpression`]
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn expression_ts_as(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSAsExpression(self.alloc(self.ts_as_expression(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSAsExpression`] into a [`Expression::TSAsExpression`]
    #[inline]
    pub fn expression_from_ts_as<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSAsExpression<'a>>>,
    {
        Expression::TSAsExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TSSatisfiesExpression`]
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn expression_ts_satisfies(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSSatisfiesExpression(self.alloc(self.ts_satisfies_expression(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSSatisfiesExpression`] into a [`Expression::TSSatisfiesExpression`]
    #[inline]
    pub fn expression_from_ts_satisfies<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSSatisfiesExpression<'a>>>,
    {
        Expression::TSSatisfiesExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TSTypeAssertion`]
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn expression_ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSTypeAssertion(self.alloc(self.ts_type_assertion(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSTypeAssertion`] into a [`Expression::TSTypeAssertion`]
    #[inline]
    pub fn expression_from_ts_type_assertion<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeAssertion<'a>>>,
    {
        Expression::TSTypeAssertion(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TSNonNullExpression`]
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn expression_ts_non_null(self, span: Span, expression: Expression<'a>) -> Expression<'a> {
        Expression::TSNonNullExpression(self.alloc(self.ts_non_null_expression(span, expression)))
    }

    /// Convert a [`TSNonNullExpression`] into a [`Expression::TSNonNullExpression`]
    #[inline]
    pub fn expression_from_ts_non_null<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSNonNullExpression<'a>>>,
    {
        Expression::TSNonNullExpression(inner.into_in(self.allocator))
    }

    /// Build a [`Expression::TSInstantiationExpression`]
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn expression_ts_instantiation<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Expression::TSInstantiationExpression(self.alloc(self.ts_instantiation_expression(
            span,
            expression,
            type_parameters,
        )))
    }

    /// Convert a [`TSInstantiationExpression`] into a [`Expression::TSInstantiationExpression`]
    #[inline]
    pub fn expression_from_ts_instantiation<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSInstantiationExpression<'a>>>,
    {
        Expression::TSInstantiationExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_member(self, inner: MemberExpression<'a>) -> Expression<'a> {
        Expression::from(inner)
    }

    /// Builds a [`IdentifierName`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_identifier_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn identifier_name<A>(self, span: Span, name: A) -> IdentifierName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        IdentifierName { span, name: name.into_in(self.allocator) }
    }

    /// Builds a [`IdentifierName`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::identifier_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn alloc_identifier_name<A>(self, span: Span, name: A) -> Box<'a, IdentifierName<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.identifier_name(span, name), self.allocator)
    }

    /// Builds a [`IdentifierReference`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_identifier_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn identifier_reference<A>(self, span: Span, name: A) -> IdentifierReference<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        IdentifierReference {
            span,
            name: name.into_in(self.allocator),
            reference_id: Default::default(),
            reference_flag: Default::default(),
        }
    }

    /// Builds a [`IdentifierReference`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::identifier_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn alloc_identifier_reference<A>(
        self,
        span: Span,
        name: A,
    ) -> Box<'a, IdentifierReference<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.identifier_reference(span, name), self.allocator)
    }

    /// Builds a [`BindingIdentifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binding_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    #[inline]
    pub fn binding_identifier<A>(self, span: Span, name: A) -> BindingIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BindingIdentifier {
            span,
            name: name.into_in(self.allocator),
            symbol_id: Default::default(),
        }
    }

    /// Builds a [`BindingIdentifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binding_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    #[inline]
    pub fn alloc_binding_identifier<A>(self, span: Span, name: A) -> Box<'a, BindingIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.binding_identifier(span, name), self.allocator)
    }

    /// Builds a [`LabelIdentifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_label_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn label_identifier<A>(self, span: Span, name: A) -> LabelIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        LabelIdentifier { span, name: name.into_in(self.allocator) }
    }

    /// Builds a [`LabelIdentifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::label_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn alloc_label_identifier<A>(self, span: Span, name: A) -> Box<'a, LabelIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.label_identifier(span, name), self.allocator)
    }

    /// Builds a [`ThisExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_this_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn this_expression(self, span: Span) -> ThisExpression {
        ThisExpression { span }
    }

    /// Builds a [`ThisExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::this_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_this_expression(self, span: Span) -> Box<'a, ThisExpression> {
        Box::new_in(self.this_expression(span), self.allocator)
    }

    /// Builds a [`ArrayExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_array_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - trailing_comma: Array trailing comma
    #[inline]
    pub fn array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> ArrayExpression<'a> {
        ArrayExpression { span, elements, trailing_comma }
    }

    /// Builds a [`ArrayExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::array_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - trailing_comma: Array trailing comma
    #[inline]
    pub fn alloc_array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayExpression<'a>> {
        Box::new_in(self.array_expression(span, elements, trailing_comma), self.allocator)
    }

    /// Build a [`ArrayExpressionElement::SpreadElement`]
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being spread.
    #[inline]
    pub fn array_expression_element_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::SpreadElement(self.alloc(self.spread_element(span, argument)))
    }

    /// Convert a [`SpreadElement`] into a [`ArrayExpressionElement::SpreadElement`]
    #[inline]
    pub fn array_expression_element_from_spread_element<T>(
        self,
        inner: T,
    ) -> ArrayExpressionElement<'a>
    where
        T: IntoIn<'a, Box<'a, SpreadElement<'a>>>,
    {
        ArrayExpressionElement::SpreadElement(inner.into_in(self.allocator))
    }

    /// Build a [`ArrayExpressionElement::Elision`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn array_expression_element_elision(self, span: Span) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::Elision(self.elision(span))
    }

    /// Convert a [`Elision`] into a [`ArrayExpressionElement::Elision`]
    #[inline]
    pub fn array_expression_element_from_elision<T>(self, inner: T) -> ArrayExpressionElement<'a>
    where
        T: IntoIn<'a, Elision>,
    {
        ArrayExpressionElement::Elision(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn array_expression_element_expression(
        self,
        inner: Expression<'a>,
    ) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::from(inner)
    }

    /// Builds a [`Elision`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_elision`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn elision(self, span: Span) -> Elision {
        Elision { span }
    }

    /// Builds a [`Elision`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::elision`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_elision(self, span: Span) -> Box<'a, Elision> {
        Box::new_in(self.elision(span), self.allocator)
    }

    /// Builds a [`ObjectExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_object_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties: Properties declared in the object
    /// - trailing_comma
    #[inline]
    pub fn object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> ObjectExpression<'a> {
        ObjectExpression { span, properties, trailing_comma }
    }

    /// Builds a [`ObjectExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::object_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties: Properties declared in the object
    /// - trailing_comma
    #[inline]
    pub fn alloc_object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ObjectExpression<'a>> {
        Box::new_in(self.object_expression(span, properties, trailing_comma), self.allocator)
    }

    /// Build a [`ObjectPropertyKind::ObjectProperty`]
    ///
    /// This node contains a [`ObjectProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - key
    /// - value
    /// - init
    /// - method
    /// - shorthand
    /// - computed
    #[inline]
    pub fn object_property_kind_object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        init: Option<Expression<'a>>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectPropertyKind<'a> {
        ObjectPropertyKind::ObjectProperty(
            self.alloc(
                self.object_property(span, kind, key, value, init, method, shorthand, computed),
            ),
        )
    }

    /// Convert a [`ObjectProperty`] into a [`ObjectPropertyKind::ObjectProperty`]
    #[inline]
    pub fn object_property_kind_from_object_property<T>(self, inner: T) -> ObjectPropertyKind<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectProperty<'a>>>,
    {
        ObjectPropertyKind::ObjectProperty(inner.into_in(self.allocator))
    }

    /// Build a [`ObjectPropertyKind::SpreadProperty`]
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being spread.
    #[inline]
    pub fn object_property_kind_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        ObjectPropertyKind::SpreadProperty(self.alloc(self.spread_element(span, argument)))
    }

    /// Convert a [`SpreadElement`] into a [`ObjectPropertyKind::SpreadProperty`]
    #[inline]
    pub fn object_property_kind_from_spread_element<T>(self, inner: T) -> ObjectPropertyKind<'a>
    where
        T: IntoIn<'a, Box<'a, SpreadElement<'a>>>,
    {
        ObjectPropertyKind::SpreadProperty(inner.into_in(self.allocator))
    }

    /// Builds a [`ObjectProperty`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_object_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - key
    /// - value
    /// - init
    /// - method
    /// - shorthand
    /// - computed
    #[inline]
    pub fn object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        init: Option<Expression<'a>>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'a> {
        ObjectProperty { span, kind, key, value, init, method, shorthand, computed }
    }

    /// Builds a [`ObjectProperty`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::object_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - key
    /// - value
    /// - init
    /// - method
    /// - shorthand
    /// - computed
    #[inline]
    pub fn alloc_object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        init: Option<Expression<'a>>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, ObjectProperty<'a>> {
        Box::new_in(
            self.object_property(span, kind, key, value, init, method, shorthand, computed),
            self.allocator,
        )
    }

    /// Build a [`PropertyKey::StaticIdentifier`]
    ///
    /// This node contains a [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn property_key_identifier_name<A>(self, span: Span, name: A) -> PropertyKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PropertyKey::StaticIdentifier(self.alloc(self.identifier_name(span, name)))
    }

    /// Convert a [`IdentifierName`] into a [`PropertyKey::StaticIdentifier`]
    #[inline]
    pub fn property_key_from_identifier_name<T>(self, inner: T) -> PropertyKey<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        PropertyKey::StaticIdentifier(inner.into_in(self.allocator))
    }

    /// Build a [`PropertyKey::PrivateIdentifier`]
    ///
    /// This node contains a [`PrivateIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn property_key_private_identifier<A>(self, span: Span, name: A) -> PropertyKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PropertyKey::PrivateIdentifier(self.alloc(self.private_identifier(span, name)))
    }

    /// Convert a [`PrivateIdentifier`] into a [`PropertyKey::PrivateIdentifier`]
    #[inline]
    pub fn property_key_from_private_identifier<T>(self, inner: T) -> PropertyKey<'a>
    where
        T: IntoIn<'a, Box<'a, PrivateIdentifier<'a>>>,
    {
        PropertyKey::PrivateIdentifier(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn property_key_expression(self, inner: Expression<'a>) -> PropertyKey<'a> {
        PropertyKey::from(inner)
    }

    /// Builds a [`TemplateLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_template_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - expressions
    #[inline]
    pub fn template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral { span, quasis, expressions }
    }

    /// Builds a [`TemplateLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::template_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - expressions
    #[inline]
    pub fn alloc_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, TemplateLiteral<'a>> {
        Box::new_in(self.template_literal(span, quasis, expressions), self.allocator)
    }

    /// Builds a [`TaggedTemplateExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_tagged_template_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - tag
    /// - quasi
    /// - type_parameters
    #[inline]
    pub fn tagged_template_expression<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: T1,
    ) -> TaggedTemplateExpression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TaggedTemplateExpression {
            span,
            tag,
            quasi,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TaggedTemplateExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::tagged_template_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - tag
    /// - quasi
    /// - type_parameters
    #[inline]
    pub fn alloc_tagged_template_expression<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        quasi: TemplateLiteral<'a>,
        type_parameters: T1,
    ) -> Box<'a, TaggedTemplateExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            self.tagged_template_expression(span, tag, quasi, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`TemplateElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_template_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - tail
    /// - value
    #[inline]
    pub fn template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> TemplateElement<'a> {
        TemplateElement { span, tail, value }
    }

    /// Builds a [`TemplateElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::template_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - tail
    /// - value
    #[inline]
    pub fn alloc_template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> Box<'a, TemplateElement<'a>> {
        Box::new_in(self.template_element(span, tail, value), self.allocator)
    }

    /// Build a [`MemberExpression::ComputedMemberExpression`]
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - expression
    /// - optional
    #[inline]
    pub fn member_expression_computed(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::ComputedMemberExpression(
            self.alloc(self.computed_member_expression(span, object, expression, optional)),
        )
    }

    /// Convert a [`ComputedMemberExpression`] into a [`MemberExpression::ComputedMemberExpression`]
    #[inline]
    pub fn member_expression_from_computed<T>(self, inner: T) -> MemberExpression<'a>
    where
        T: IntoIn<'a, Box<'a, ComputedMemberExpression<'a>>>,
    {
        MemberExpression::ComputedMemberExpression(inner.into_in(self.allocator))
    }

    /// Build a [`MemberExpression::StaticMemberExpression`]
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - property
    /// - optional
    #[inline]
    pub fn member_expression_static(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::StaticMemberExpression(
            self.alloc(self.static_member_expression(span, object, property, optional)),
        )
    }

    /// Convert a [`StaticMemberExpression`] into a [`MemberExpression::StaticMemberExpression`]
    #[inline]
    pub fn member_expression_from_static<T>(self, inner: T) -> MemberExpression<'a>
    where
        T: IntoIn<'a, Box<'a, StaticMemberExpression<'a>>>,
    {
        MemberExpression::StaticMemberExpression(inner.into_in(self.allocator))
    }

    /// Build a [`MemberExpression::PrivateFieldExpression`]
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - field
    /// - optional
    #[inline]
    pub fn member_expression_private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::PrivateFieldExpression(
            self.alloc(self.private_field_expression(span, object, field, optional)),
        )
    }

    /// Convert a [`PrivateFieldExpression`] into a [`MemberExpression::PrivateFieldExpression`]
    #[inline]
    pub fn member_expression_from_private_field_expression<T>(
        self,
        inner: T,
    ) -> MemberExpression<'a>
    where
        T: IntoIn<'a, Box<'a, PrivateFieldExpression<'a>>>,
    {
        MemberExpression::PrivateFieldExpression(inner.into_in(self.allocator))
    }

    /// Builds a [`ComputedMemberExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_computed_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - expression
    /// - optional
    #[inline]
    pub fn computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ComputedMemberExpression<'a> {
        ComputedMemberExpression { span, object, expression, optional }
    }

    /// Builds a [`ComputedMemberExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::computed_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - expression
    /// - optional
    #[inline]
    pub fn alloc_computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> Box<'a, ComputedMemberExpression<'a>> {
        Box::new_in(
            self.computed_member_expression(span, object, expression, optional),
            self.allocator,
        )
    }

    /// Builds a [`StaticMemberExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_static_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - property
    /// - optional
    #[inline]
    pub fn static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> StaticMemberExpression<'a> {
        StaticMemberExpression { span, object, property, optional }
    }

    /// Builds a [`StaticMemberExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::static_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - property
    /// - optional
    #[inline]
    pub fn alloc_static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, StaticMemberExpression<'a>> {
        Box::new_in(self.static_member_expression(span, object, property, optional), self.allocator)
    }

    /// Builds a [`PrivateFieldExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_private_field_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - field
    /// - optional
    #[inline]
    pub fn private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> PrivateFieldExpression<'a> {
        PrivateFieldExpression { span, object, field, optional }
    }

    /// Builds a [`PrivateFieldExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::private_field_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - field
    /// - optional
    #[inline]
    pub fn alloc_private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> Box<'a, PrivateFieldExpression<'a>> {
        Box::new_in(self.private_field_expression(span, object, field, optional), self.allocator)
    }

    /// Builds a [`CallExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_call_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - arguments
    /// - callee
    /// - type_parameters
    /// - optional
    #[inline]
    pub fn call_expression<T1>(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: T1,
        optional: bool,
    ) -> CallExpression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        CallExpression {
            span,
            arguments,
            callee,
            type_parameters: type_parameters.into_in(self.allocator),
            optional,
        }
    }

    /// Builds a [`CallExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::call_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - arguments
    /// - callee
    /// - type_parameters
    /// - optional
    #[inline]
    pub fn alloc_call_expression<T1>(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: T1,
        optional: bool,
    ) -> Box<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            self.call_expression(span, arguments, callee, type_parameters, optional),
            self.allocator,
        )
    }

    /// Builds a [`NewExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_new_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - arguments
    /// - type_parameters
    #[inline]
    pub fn new_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: T1,
    ) -> NewExpression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        NewExpression {
            span,
            callee,
            arguments,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`NewExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::new_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - arguments
    /// - type_parameters
    #[inline]
    pub fn alloc_new_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        arguments: Vec<'a, Argument<'a>>,
        type_parameters: T1,
    ) -> Box<'a, NewExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(self.new_expression(span, callee, arguments, type_parameters), self.allocator)
    }

    /// Builds a [`MetaProperty`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_meta_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - meta
    /// - property
    #[inline]
    pub fn meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> MetaProperty<'a> {
        MetaProperty { span, meta, property }
    }

    /// Builds a [`MetaProperty`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::meta_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - meta
    /// - property
    #[inline]
    pub fn alloc_meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Box<'a, MetaProperty<'a>> {
        Box::new_in(self.meta_property(span, meta, property), self.allocator)
    }

    /// Builds a [`SpreadElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_spread_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being spread.
    #[inline]
    pub fn spread_element(self, span: Span, argument: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement { span, argument }
    }

    /// Builds a [`SpreadElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::spread_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being spread.
    #[inline]
    pub fn alloc_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        Box::new_in(self.spread_element(span, argument), self.allocator)
    }

    /// Build a [`Argument::SpreadElement`]
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being spread.
    #[inline]
    pub fn argument_spread_element(self, span: Span, argument: Expression<'a>) -> Argument<'a> {
        Argument::SpreadElement(self.alloc(self.spread_element(span, argument)))
    }

    /// Convert a [`SpreadElement`] into a [`Argument::SpreadElement`]
    #[inline]
    pub fn argument_from_spread_element<T>(self, inner: T) -> Argument<'a>
    where
        T: IntoIn<'a, Box<'a, SpreadElement<'a>>>,
    {
        Argument::SpreadElement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn argument_expression(self, inner: Expression<'a>) -> Argument<'a> {
        Argument::from(inner)
    }

    /// Builds a [`UpdateExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_update_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - prefix
    /// - argument
    #[inline]
    pub fn update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> UpdateExpression<'a> {
        UpdateExpression { span, operator, prefix, argument }
    }

    /// Builds a [`UpdateExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::update_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - prefix
    /// - argument
    #[inline]
    pub fn alloc_update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Box<'a, UpdateExpression<'a>> {
        Box::new_in(self.update_expression(span, operator, prefix, argument), self.allocator)
    }

    /// Builds a [`UnaryExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_unary_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - argument
    #[inline]
    pub fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> UnaryExpression<'a> {
        UnaryExpression { span, operator, argument }
    }

    /// Builds a [`UnaryExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::unary_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - argument
    #[inline]
    pub fn alloc_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Box<'a, UnaryExpression<'a>> {
        Box::new_in(self.unary_expression(span, operator, argument), self.allocator)
    }

    /// Builds a [`BinaryExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binary_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> BinaryExpression<'a> {
        BinaryExpression { span, left, operator, right }
    }

    /// Builds a [`BinaryExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binary_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn alloc_binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, BinaryExpression<'a>> {
        Box::new_in(self.binary_expression(span, left, operator, right), self.allocator)
    }

    /// Builds a [`PrivateInExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_private_in_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> PrivateInExpression<'a> {
        PrivateInExpression { span, left, operator, right }
    }

    /// Builds a [`PrivateInExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::private_in_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn alloc_private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, PrivateInExpression<'a>> {
        Box::new_in(self.private_in_expression(span, left, operator, right), self.allocator)
    }

    /// Builds a [`LogicalExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_logical_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> LogicalExpression<'a> {
        LogicalExpression { span, left, operator, right }
    }

    /// Builds a [`LogicalExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::logical_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - operator
    /// - right
    #[inline]
    pub fn alloc_logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Box<'a, LogicalExpression<'a>> {
        Box::new_in(self.logical_expression(span, left, operator, right), self.allocator)
    }

    /// Builds a [`ConditionalExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_conditional_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> ConditionalExpression<'a> {
        ConditionalExpression { span, test, consequent, alternate }
    }

    /// Builds a [`ConditionalExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::conditional_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn alloc_conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Box<'a, ConditionalExpression<'a>> {
        Box::new_in(self.conditional_expression(span, test, consequent, alternate), self.allocator)
    }

    /// Builds a [`AssignmentExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - left
    /// - right
    #[inline]
    pub fn assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> AssignmentExpression<'a> {
        AssignmentExpression { span, operator, left, right }
    }

    /// Builds a [`AssignmentExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - left
    /// - right
    #[inline]
    pub fn alloc_assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentExpression<'a>> {
        Box::new_in(self.assignment_expression(span, operator, left, right), self.allocator)
    }

    #[inline]
    pub fn assignment_target_simple(
        self,
        inner: SimpleAssignmentTarget<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::from(inner)
    }

    #[inline]
    pub fn assignment_target_assignment_target_pattern(
        self,
        inner: AssignmentTargetPattern<'a>,
    ) -> AssignmentTarget<'a> {
        AssignmentTarget::from(inner)
    }

    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`]
    ///
    /// This node contains a [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn simple_assignment_target_identifier_reference<A>(
        self,
        span: Span,
        name: A,
    ) -> SimpleAssignmentTarget<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(
            self.alloc(self.identifier_reference(span, name)),
        )
    }

    /// Convert a [`IdentifierReference`] into a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`]
    #[inline]
    pub fn simple_assignment_target_from_identifier_reference<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierReference<'a>>>,
    {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(inner.into_in(self.allocator))
    }

    /// Build a [`SimpleAssignmentTarget::TSAsExpression`]
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn simple_assignment_target_ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSAsExpression(self.alloc(self.ts_as_expression(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSAsExpression`] into a [`SimpleAssignmentTarget::TSAsExpression`]
    #[inline]
    pub fn simple_assignment_target_from_ts_as_expression<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, TSAsExpression<'a>>>,
    {
        SimpleAssignmentTarget::TSAsExpression(inner.into_in(self.allocator))
    }

    /// Build a [`SimpleAssignmentTarget::TSSatisfiesExpression`]
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn simple_assignment_target_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSSatisfiesExpression(self.alloc(self.ts_satisfies_expression(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSSatisfiesExpression`] into a [`SimpleAssignmentTarget::TSSatisfiesExpression`]
    #[inline]
    pub fn simple_assignment_target_from_ts_satisfies_expression<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, TSSatisfiesExpression<'a>>>,
    {
        SimpleAssignmentTarget::TSSatisfiesExpression(inner.into_in(self.allocator))
    }

    /// Build a [`SimpleAssignmentTarget::TSNonNullExpression`]
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn simple_assignment_target_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSNonNullExpression(
            self.alloc(self.ts_non_null_expression(span, expression)),
        )
    }

    /// Convert a [`TSNonNullExpression`] into a [`SimpleAssignmentTarget::TSNonNullExpression`]
    #[inline]
    pub fn simple_assignment_target_from_ts_non_null_expression<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, TSNonNullExpression<'a>>>,
    {
        SimpleAssignmentTarget::TSNonNullExpression(inner.into_in(self.allocator))
    }

    /// Build a [`SimpleAssignmentTarget::TSTypeAssertion`]
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn simple_assignment_target_ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSTypeAssertion(self.alloc(self.ts_type_assertion(
            span,
            expression,
            type_annotation,
        )))
    }

    /// Convert a [`TSTypeAssertion`] into a [`SimpleAssignmentTarget::TSTypeAssertion`]
    #[inline]
    pub fn simple_assignment_target_from_ts_type_assertion<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeAssertion<'a>>>,
    {
        SimpleAssignmentTarget::TSTypeAssertion(inner.into_in(self.allocator))
    }

    /// Build a [`SimpleAssignmentTarget::TSInstantiationExpression`]
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn simple_assignment_target_ts_instantiation_expression<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> SimpleAssignmentTarget<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        SimpleAssignmentTarget::TSInstantiationExpression(
            self.alloc(self.ts_instantiation_expression(span, expression, type_parameters)),
        )
    }

    /// Convert a [`TSInstantiationExpression`] into a [`SimpleAssignmentTarget::TSInstantiationExpression`]
    #[inline]
    pub fn simple_assignment_target_from_ts_instantiation_expression<T>(
        self,
        inner: T,
    ) -> SimpleAssignmentTarget<'a>
    where
        T: IntoIn<'a, Box<'a, TSInstantiationExpression<'a>>>,
    {
        SimpleAssignmentTarget::TSInstantiationExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn simple_assignment_target_member_expression(
        self,
        inner: MemberExpression<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::from(inner)
    }

    /// Build a [`AssignmentTargetPattern::ArrayAssignmentTarget`]
    ///
    /// This node contains a [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    /// - trailing_comma
    #[inline]
    pub fn assignment_target_pattern_array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::ArrayAssignmentTarget(self.alloc(self.array_assignment_target(
            span,
            elements,
            rest,
            trailing_comma,
        )))
    }

    /// Convert a [`ArrayAssignmentTarget`] into a [`AssignmentTargetPattern::ArrayAssignmentTarget`]
    #[inline]
    pub fn assignment_target_pattern_from_array_assignment_target<T>(
        self,
        inner: T,
    ) -> AssignmentTargetPattern<'a>
    where
        T: IntoIn<'a, Box<'a, ArrayAssignmentTarget<'a>>>,
    {
        AssignmentTargetPattern::ArrayAssignmentTarget(inner.into_in(self.allocator))
    }

    /// Build a [`AssignmentTargetPattern::ObjectAssignmentTarget`]
    ///
    /// This node contains a [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn assignment_target_pattern_object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> AssignmentTargetPattern<'a> {
        AssignmentTargetPattern::ObjectAssignmentTarget(
            self.alloc(self.object_assignment_target(span, properties, rest)),
        )
    }

    /// Convert a [`ObjectAssignmentTarget`] into a [`AssignmentTargetPattern::ObjectAssignmentTarget`]
    #[inline]
    pub fn assignment_target_pattern_from_object_assignment_target<T>(
        self,
        inner: T,
    ) -> AssignmentTargetPattern<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectAssignmentTarget<'a>>>,
    {
        AssignmentTargetPattern::ObjectAssignmentTarget(inner.into_in(self.allocator))
    }

    /// Builds a [`ArrayAssignmentTarget`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_array_assignment_target`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    /// - trailing_comma
    #[inline]
    pub fn array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> ArrayAssignmentTarget<'a> {
        ArrayAssignmentTarget { span, elements, rest, trailing_comma }
    }

    /// Builds a [`ArrayAssignmentTarget`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::array_assignment_target`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    /// - trailing_comma
    #[inline]
    pub fn alloc_array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayAssignmentTarget<'a>> {
        Box::new_in(
            self.array_assignment_target(span, elements, rest, trailing_comma),
            self.allocator,
        )
    }

    /// Builds a [`ObjectAssignmentTarget`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_object_assignment_target`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> ObjectAssignmentTarget<'a> {
        ObjectAssignmentTarget { span, properties, rest }
    }

    /// Builds a [`ObjectAssignmentTarget`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::object_assignment_target`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn alloc_object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> Box<'a, ObjectAssignmentTarget<'a>> {
        Box::new_in(self.object_assignment_target(span, properties, rest), self.allocator)
    }

    /// Builds a [`AssignmentTargetRest`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_target_rest`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - target
    #[inline]
    pub fn assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> AssignmentTargetRest<'a> {
        AssignmentTargetRest { span, target }
    }

    /// Builds a [`AssignmentTargetRest`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_target_rest`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - target
    #[inline]
    pub fn alloc_assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> Box<'a, AssignmentTargetRest<'a>> {
        Box::new_in(self.assignment_target_rest(span, target), self.allocator)
    }

    /// Build a [`AssignmentTargetMaybeDefault::AssignmentTargetWithDefault`]
    ///
    /// This node contains a [`AssignmentTargetWithDefault`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn assignment_target_maybe_default_assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetMaybeDefault<'a> {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
            self.alloc(self.assignment_target_with_default(span, binding, init)),
        )
    }

    /// Convert a [`AssignmentTargetWithDefault`] into a [`AssignmentTargetMaybeDefault::AssignmentTargetWithDefault`]
    #[inline]
    pub fn assignment_target_maybe_default_from_assignment_target_with_default<T>(
        self,
        inner: T,
    ) -> AssignmentTargetMaybeDefault<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentTargetWithDefault<'a>>>,
    {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn assignment_target_maybe_default_assignment_target(
        self,
        inner: AssignmentTarget<'a>,
    ) -> AssignmentTargetMaybeDefault<'a> {
        AssignmentTargetMaybeDefault::from(inner)
    }

    /// Builds a [`AssignmentTargetWithDefault`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_target_with_default`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetWithDefault<'a> {
        AssignmentTargetWithDefault { span, binding, init }
    }

    /// Builds a [`AssignmentTargetWithDefault`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_target_with_default`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn alloc_assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> Box<'a, AssignmentTargetWithDefault<'a>> {
        Box::new_in(self.assignment_target_with_default(span, binding, init), self.allocator)
    }

    /// Build a [`AssignmentTargetProperty::AssignmentTargetPropertyIdentifier`]
    ///
    /// This node contains a [`AssignmentTargetPropertyIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn assignment_target_property_assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetProperty<'a> {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
            self.alloc(self.assignment_target_property_identifier(span, binding, init)),
        )
    }

    /// Convert a [`AssignmentTargetPropertyIdentifier`] into a [`AssignmentTargetProperty::AssignmentTargetPropertyIdentifier`]
    #[inline]
    pub fn assignment_target_property_from_assignment_target_property_identifier<T>(
        self,
        inner: T,
    ) -> AssignmentTargetProperty<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentTargetPropertyIdentifier<'a>>>,
    {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(inner.into_in(self.allocator))
    }

    /// Build a [`AssignmentTargetProperty::AssignmentTargetPropertyProperty`]
    ///
    /// This node contains a [`AssignmentTargetPropertyProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - binding
    #[inline]
    pub fn assignment_target_property_assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> AssignmentTargetProperty<'a> {
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(
            self.alloc(self.assignment_target_property_property(span, name, binding)),
        )
    }

    /// Convert a [`AssignmentTargetPropertyProperty`] into a [`AssignmentTargetProperty::AssignmentTargetPropertyProperty`]
    #[inline]
    pub fn assignment_target_property_from_assignment_target_property_property<T>(
        self,
        inner: T,
    ) -> AssignmentTargetProperty<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentTargetPropertyProperty<'a>>>,
    {
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(inner.into_in(self.allocator))
    }

    /// Builds a [`AssignmentTargetPropertyIdentifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_target_property_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetPropertyIdentifier<'a> {
        AssignmentTargetPropertyIdentifier { span, binding, init }
    }

    /// Builds a [`AssignmentTargetPropertyIdentifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_target_property_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - binding
    /// - init
    #[inline]
    pub fn alloc_assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> Box<'a, AssignmentTargetPropertyIdentifier<'a>> {
        Box::new_in(self.assignment_target_property_identifier(span, binding, init), self.allocator)
    }

    /// Builds a [`AssignmentTargetPropertyProperty`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_target_property_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - binding
    #[inline]
    pub fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> AssignmentTargetPropertyProperty<'a> {
        AssignmentTargetPropertyProperty { span, name, binding }
    }

    /// Builds a [`AssignmentTargetPropertyProperty`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_target_property_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - binding
    #[inline]
    pub fn alloc_assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> Box<'a, AssignmentTargetPropertyProperty<'a>> {
        Box::new_in(self.assignment_target_property_property(span, name, binding), self.allocator)
    }

    /// Builds a [`SequenceExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_sequence_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expressions
    #[inline]
    pub fn sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> SequenceExpression<'a> {
        SequenceExpression { span, expressions }
    }

    /// Builds a [`SequenceExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::sequence_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expressions
    #[inline]
    pub fn alloc_sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, SequenceExpression<'a>> {
        Box::new_in(self.sequence_expression(span, expressions), self.allocator)
    }

    /// Builds a [`Super`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_super_`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn super_(self, span: Span) -> Super {
        Super { span }
    }

    /// Builds a [`Super`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::super_`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_super_(self, span: Span) -> Box<'a, Super> {
        Box::new_in(self.super_(span), self.allocator)
    }

    /// Builds a [`AwaitExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_await_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn await_expression(self, span: Span, argument: Expression<'a>) -> AwaitExpression<'a> {
        AwaitExpression { span, argument }
    }

    /// Builds a [`AwaitExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::await_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn alloc_await_expression(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, AwaitExpression<'a>> {
        Box::new_in(self.await_expression(span, argument), self.allocator)
    }

    /// Builds a [`ChainExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_chain_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn chain_expression(self, span: Span, expression: ChainElement<'a>) -> ChainExpression<'a> {
        ChainExpression { span, expression }
    }

    /// Builds a [`ChainExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::chain_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_chain_expression(
        self,
        span: Span,
        expression: ChainElement<'a>,
    ) -> Box<'a, ChainExpression<'a>> {
        Box::new_in(self.chain_expression(span, expression), self.allocator)
    }

    /// Build a [`ChainElement::CallExpression`]
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - arguments
    /// - callee
    /// - type_parameters
    /// - optional
    #[inline]
    pub fn chain_element_call_expression<T1>(
        self,
        span: Span,
        arguments: Vec<'a, Argument<'a>>,
        callee: Expression<'a>,
        type_parameters: T1,
        optional: bool,
    ) -> ChainElement<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ChainElement::CallExpression(self.alloc(self.call_expression(
            span,
            arguments,
            callee,
            type_parameters,
            optional,
        )))
    }

    /// Convert a [`CallExpression`] into a [`ChainElement::CallExpression`]
    #[inline]
    pub fn chain_element_from_call_expression<T>(self, inner: T) -> ChainElement<'a>
    where
        T: IntoIn<'a, Box<'a, CallExpression<'a>>>,
    {
        ChainElement::CallExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn chain_element_member_expression(self, inner: MemberExpression<'a>) -> ChainElement<'a> {
        ChainElement::from(inner)
    }

    /// Builds a [`ParenthesizedExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_parenthesized_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ParenthesizedExpression<'a> {
        ParenthesizedExpression { span, expression }
    }

    /// Builds a [`ParenthesizedExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::parenthesized_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ParenthesizedExpression<'a>> {
        Box::new_in(self.parenthesized_expression(span, expression), self.allocator)
    }

    /// Build a [`Statement::BlockStatement`]
    ///
    /// This node contains a [`BlockStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn statement_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(self.alloc(self.block_statement(span, body)))
    }

    /// Convert a [`BlockStatement`] into a [`Statement::BlockStatement`]
    #[inline]
    pub fn statement_from_block<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Statement::BlockStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::BreakStatement`]
    ///
    /// This node contains a [`BreakStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn statement_break(self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc(self.break_statement(span, label)))
    }

    /// Convert a [`BreakStatement`] into a [`Statement::BreakStatement`]
    #[inline]
    pub fn statement_from_break<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, BreakStatement<'a>>>,
    {
        Statement::BreakStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ContinueStatement`]
    ///
    /// This node contains a [`ContinueStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn statement_continue(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc(self.continue_statement(span, label)))
    }

    /// Convert a [`ContinueStatement`] into a [`Statement::ContinueStatement`]
    #[inline]
    pub fn statement_from_continue<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ContinueStatement<'a>>>,
    {
        Statement::ContinueStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::DebuggerStatement`]
    ///
    /// This node contains a [`DebuggerStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn statement_debugger(self, span: Span) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc(self.debugger_statement(span)))
    }

    /// Convert a [`DebuggerStatement`] into a [`Statement::DebuggerStatement`]
    #[inline]
    pub fn statement_from_debugger<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, DebuggerStatement>>,
    {
        Statement::DebuggerStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::DoWhileStatement`]
    ///
    /// This node contains a [`DoWhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - test
    #[inline]
    pub fn statement_do_while(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc(self.do_while_statement(span, body, test)))
    }

    /// Convert a [`DoWhileStatement`] into a [`Statement::DoWhileStatement`]
    #[inline]
    pub fn statement_from_do_while<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, DoWhileStatement<'a>>>,
    {
        Statement::DoWhileStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::EmptyStatement`]
    ///
    /// This node contains a [`EmptyStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn statement_empty(self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(self.empty_statement(span)))
    }

    /// Convert a [`EmptyStatement`] into a [`Statement::EmptyStatement`]
    #[inline]
    pub fn statement_from_empty<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, EmptyStatement>>,
    {
        Statement::EmptyStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ExpressionStatement`]
    ///
    /// This node contains a [`ExpressionStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn statement_expression(self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(self.expression_statement(span, expression)))
    }

    /// Convert a [`ExpressionStatement`] into a [`Statement::ExpressionStatement`]
    #[inline]
    pub fn statement_from_expression<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ExpressionStatement<'a>>>,
    {
        Statement::ExpressionStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ForInStatement`]
    ///
    /// This node contains a [`ForInStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn statement_for_in(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForInStatement(self.alloc(self.for_in_statement(span, left, right, body)))
    }

    /// Convert a [`ForInStatement`] into a [`Statement::ForInStatement`]
    #[inline]
    pub fn statement_from_for_in<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForInStatement<'a>>>,
    {
        Statement::ForInStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ForOfStatement`]
    ///
    /// This node contains a [`ForOfStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#await
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn statement_for_of(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForOfStatement(
            self.alloc(self.for_of_statement(span, r#await, left, right, body)),
        )
    }

    /// Convert a [`ForOfStatement`] into a [`Statement::ForOfStatement`]
    #[inline]
    pub fn statement_from_for_of<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForOfStatement<'a>>>,
    {
        Statement::ForOfStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ForStatement`]
    ///
    /// This node contains a [`ForStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - init
    /// - test
    /// - update
    /// - body
    #[inline]
    pub fn statement_for(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForStatement(self.alloc(self.for_statement(span, init, test, update, body)))
    }

    /// Convert a [`ForStatement`] into a [`Statement::ForStatement`]
    #[inline]
    pub fn statement_from_for<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForStatement<'a>>>,
    {
        Statement::ForStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::IfStatement`]
    ///
    /// This node contains a [`IfStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn statement_if(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::IfStatement(self.alloc(self.if_statement(span, test, consequent, alternate)))
    }

    /// Convert a [`IfStatement`] into a [`Statement::IfStatement`]
    #[inline]
    pub fn statement_from_if<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, IfStatement<'a>>>,
    {
        Statement::IfStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::LabeledStatement`]
    ///
    /// This node contains a [`LabeledStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    /// - body
    #[inline]
    pub fn statement_labeled(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc(self.labeled_statement(span, label, body)))
    }

    /// Convert a [`LabeledStatement`] into a [`Statement::LabeledStatement`]
    #[inline]
    pub fn statement_from_labeled<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, LabeledStatement<'a>>>,
    {
        Statement::LabeledStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ReturnStatement`]
    ///
    /// This node contains a [`ReturnStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn statement_return(self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc(self.return_statement(span, argument)))
    }

    /// Convert a [`ReturnStatement`] into a [`Statement::ReturnStatement`]
    #[inline]
    pub fn statement_from_return<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ReturnStatement<'a>>>,
    {
        Statement::ReturnStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::SwitchStatement`]
    ///
    /// This node contains a [`SwitchStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - discriminant
    /// - cases
    #[inline]
    pub fn statement_switch(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc(self.switch_statement(span, discriminant, cases)))
    }

    /// Convert a [`SwitchStatement`] into a [`Statement::SwitchStatement`]
    #[inline]
    pub fn statement_from_switch<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, SwitchStatement<'a>>>,
    {
        Statement::SwitchStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::ThrowStatement`]
    ///
    /// This node contains a [`ThrowStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn statement_throw(self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(self.throw_statement(span, argument)))
    }

    /// Convert a [`ThrowStatement`] into a [`Statement::ThrowStatement`]
    #[inline]
    pub fn statement_from_throw<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ThrowStatement<'a>>>,
    {
        Statement::ThrowStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::TryStatement`]
    ///
    /// This node contains a [`TryStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block
    /// - handler
    /// - finalizer
    #[inline]
    pub fn statement_try<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> Statement<'a>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        Statement::TryStatement(self.alloc(self.try_statement(span, block, handler, finalizer)))
    }

    /// Convert a [`TryStatement`] into a [`Statement::TryStatement`]
    #[inline]
    pub fn statement_from_try<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, TryStatement<'a>>>,
    {
        Statement::TryStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::WhileStatement`]
    ///
    /// This node contains a [`WhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - body
    #[inline]
    pub fn statement_while(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc(self.while_statement(span, test, body)))
    }

    /// Convert a [`WhileStatement`] into a [`Statement::WhileStatement`]
    #[inline]
    pub fn statement_from_while<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, WhileStatement<'a>>>,
    {
        Statement::WhileStatement(inner.into_in(self.allocator))
    }

    /// Build a [`Statement::WithStatement`]
    ///
    /// This node contains a [`WithStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - body
    #[inline]
    pub fn statement_with(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc(self.with_statement(span, object, body)))
    }

    /// Convert a [`WithStatement`] into a [`Statement::WithStatement`]
    #[inline]
    pub fn statement_from_with<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, WithStatement<'a>>>,
    {
        Statement::WithStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_declaration(self, inner: Declaration<'a>) -> Statement<'a> {
        Statement::from(inner)
    }

    #[inline]
    pub fn statement_module_declaration(self, inner: ModuleDeclaration<'a>) -> Statement<'a> {
        Statement::from(inner)
    }

    /// Builds a [`Directive`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_directive`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Directive with any escapes unescaped
    /// - directive: Raw content of directive as it appears in source, any escapes left as is
    #[inline]
    pub fn directive<A>(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: A,
    ) -> Directive<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Directive { span, expression, directive: directive.into_in(self.allocator) }
    }

    /// Builds a [`Directive`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::directive`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Directive with any escapes unescaped
    /// - directive: Raw content of directive as it appears in source, any escapes left as is
    #[inline]
    pub fn alloc_directive<A>(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: A,
    ) -> Box<'a, Directive<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.directive(span, expression, directive), self.allocator)
    }

    /// Builds a [`Hashbang`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_hashbang`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn hashbang<A>(self, span: Span, value: A) -> Hashbang<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Hashbang { span, value: value.into_in(self.allocator) }
    }

    /// Builds a [`Hashbang`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::hashbang`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn alloc_hashbang<A>(self, span: Span, value: A) -> Box<'a, Hashbang<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.hashbang(span, value), self.allocator)
    }

    /// Builds a [`BlockStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_block_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn block_statement(self, span: Span, body: Vec<'a, Statement<'a>>) -> BlockStatement<'a> {
        BlockStatement { span, body, scope_id: Default::default() }
    }

    /// Builds a [`BlockStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::block_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn alloc_block_statement(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, BlockStatement<'a>> {
        Box::new_in(self.block_statement(span, body), self.allocator)
    }

    /// Build a [`Declaration::VariableDeclaration`]
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - declarations
    /// - declare
    #[inline]
    pub fn declaration_variable(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::VariableDeclaration(self.alloc(self.variable_declaration(
            span,
            kind,
            declarations,
            declare,
        )))
    }

    /// Convert a [`VariableDeclaration`] into a [`Declaration::VariableDeclaration`]
    #[inline]
    pub fn declaration_from_variable<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        Declaration::VariableDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::FunctionDeclaration`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id
    /// - generator
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params
    /// - return_type
    /// - body
    #[inline]
    pub fn declaration_function<T1, T2, T3, T4>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Declaration::FunctionDeclaration(self.alloc(self.function(
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
        )))
    }

    /// Convert a [`Function`] into a [`Declaration::FunctionDeclaration`]
    #[inline]
    pub fn declaration_from_function<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Declaration::FunctionDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::ClassDeclaration`]
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the class.
    /// - id: Class identifier, AKA the name
    /// - type_parameters
    /// - super_class: Super class. When present, this will usually be an [`IdentifierReference`].
    /// - super_type_parameters: Type parameters passed to super class.
    /// - implements: Interface implementation clause for TypeScript classes.
    /// - body
    /// - r#abstract: Whether the class is abstract
    /// - declare: Whether the class was `declare`ed
    #[inline]
    pub fn declaration_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_parameters: T2,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Declaration::ClassDeclaration(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_parameters,
            implements,
            body,
            r#abstract,
            declare,
        )))
    }

    /// Convert a [`Class`] into a [`Declaration::ClassDeclaration`]
    #[inline]
    pub fn declaration_from_class<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, Class<'a>>>,
    {
        Declaration::ClassDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::UsingDeclaration`]
    ///
    /// This node contains a [`UsingDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_await
    /// - declarations
    #[inline]
    pub fn declaration_using(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Declaration<'a> {
        Declaration::UsingDeclaration(self.alloc(self.using_declaration(
            span,
            is_await,
            declarations,
        )))
    }

    /// Convert a [`UsingDeclaration`] into a [`Declaration::UsingDeclaration`]
    #[inline]
    pub fn declaration_from_using<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, UsingDeclaration<'a>>>,
    {
        Declaration::UsingDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::TSTypeAliasDeclaration`]
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - type_parameters
    /// - type_annotation
    /// - declare
    #[inline]
    pub fn declaration_ts_type_alias<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Declaration::TSTypeAliasDeclaration(self.alloc(self.ts_type_alias_declaration(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
        )))
    }

    /// Convert a [`TSTypeAliasDeclaration`] into a [`Declaration::TSTypeAliasDeclaration`]
    #[inline]
    pub fn declaration_from_ts_type_alias<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeAliasDeclaration<'a>>>,
    {
        Declaration::TSTypeAliasDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::TSInterfaceDeclaration`]
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends
    /// - type_parameters
    /// - body
    /// - declare
    #[inline]
    pub fn declaration_ts_interface<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Declaration::TSInterfaceDeclaration(self.alloc(self.ts_interface_declaration(
            span,
            id,
            extends,
            type_parameters,
            body,
            declare,
        )))
    }

    /// Convert a [`TSInterfaceDeclaration`] into a [`Declaration::TSInterfaceDeclaration`]
    #[inline]
    pub fn declaration_from_ts_interface<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSInterfaceDeclaration<'a>>>,
    {
        Declaration::TSInterfaceDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::TSEnumDeclaration`]
    ///
    /// This node contains a [`TSEnumDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const
    /// - declare
    #[inline]
    pub fn declaration_ts_enum(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSEnumDeclaration(
            self.alloc(self.ts_enum_declaration(span, id, members, r#const, declare)),
        )
    }

    /// Convert a [`TSEnumDeclaration`] into a [`Declaration::TSEnumDeclaration`]
    #[inline]
    pub fn declaration_from_ts_enum<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSEnumDeclaration<'a>>>,
    {
        Declaration::TSEnumDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::TSModuleDeclaration`]
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - body
    /// - kind: The keyword used to define this module declaration
    /// - declare
    #[inline]
    pub fn declaration_ts_module(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSModuleDeclaration(
            self.alloc(self.ts_module_declaration(span, id, body, kind, declare)),
        )
    }

    /// Convert a [`TSModuleDeclaration`] into a [`Declaration::TSModuleDeclaration`]
    #[inline]
    pub fn declaration_from_ts_module<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSModuleDeclaration<'a>>>,
    {
        Declaration::TSModuleDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`Declaration::TSImportEqualsDeclaration`]
    ///
    /// This node contains a [`TSImportEqualsDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - module_reference
    /// - import_kind
    #[inline]
    pub fn declaration_ts_import_equals(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Declaration<'a> {
        Declaration::TSImportEqualsDeclaration(self.alloc(self.ts_import_equals_declaration(
            span,
            id,
            module_reference,
            import_kind,
        )))
    }

    /// Convert a [`TSImportEqualsDeclaration`] into a [`Declaration::TSImportEqualsDeclaration`]
    #[inline]
    pub fn declaration_from_ts_import_equals<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSImportEqualsDeclaration<'a>>>,
    {
        Declaration::TSImportEqualsDeclaration(inner.into_in(self.allocator))
    }

    /// Builds a [`VariableDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_variable_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - declarations
    /// - declare
    #[inline]
    pub fn variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> VariableDeclaration<'a> {
        VariableDeclaration { span, kind, declarations, declare }
    }

    /// Builds a [`VariableDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::variable_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - declarations
    /// - declare
    #[inline]
    pub fn alloc_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> Box<'a, VariableDeclaration<'a>> {
        Box::new_in(self.variable_declaration(span, kind, declarations, declare), self.allocator)
    }

    /// Builds a [`VariableDeclarator`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_variable_declarator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - id
    /// - init
    /// - definite
    #[inline]
    pub fn variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a> {
        VariableDeclarator { span, kind, id, init, definite }
    }

    /// Builds a [`VariableDeclarator`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::variable_declarator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - id
    /// - init
    /// - definite
    #[inline]
    pub fn alloc_variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> Box<'a, VariableDeclarator<'a>> {
        Box::new_in(self.variable_declarator(span, kind, id, init, definite), self.allocator)
    }

    /// Builds a [`UsingDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_using_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_await
    /// - declarations
    #[inline]
    pub fn using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> UsingDeclaration<'a> {
        UsingDeclaration { span, is_await, declarations }
    }

    /// Builds a [`UsingDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::using_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_await
    /// - declarations
    #[inline]
    pub fn alloc_using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, UsingDeclaration<'a>> {
        Box::new_in(self.using_declaration(span, is_await, declarations), self.allocator)
    }

    /// Builds a [`EmptyStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_empty_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn empty_statement(self, span: Span) -> EmptyStatement {
        EmptyStatement { span }
    }

    /// Builds a [`EmptyStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::empty_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_empty_statement(self, span: Span) -> Box<'a, EmptyStatement> {
        Box::new_in(self.empty_statement(span), self.allocator)
    }

    /// Builds a [`ExpressionStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_expression_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ExpressionStatement<'a> {
        ExpressionStatement { span, expression }
    }

    /// Builds a [`ExpressionStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::expression_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ExpressionStatement<'a>> {
        Box::new_in(self.expression_statement(span, expression), self.allocator)
    }

    /// Builds a [`IfStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_if_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> IfStatement<'a> {
        IfStatement { span, test, consequent, alternate }
    }

    /// Builds a [`IfStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::if_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    /// - alternate
    #[inline]
    pub fn alloc_if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Box<'a, IfStatement<'a>> {
        Box::new_in(self.if_statement(span, test, consequent, alternate), self.allocator)
    }

    /// Builds a [`DoWhileStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_do_while_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - test
    #[inline]
    pub fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> DoWhileStatement<'a> {
        DoWhileStatement { span, body, test }
    }

    /// Builds a [`DoWhileStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::do_while_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - test
    #[inline]
    pub fn alloc_do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Box<'a, DoWhileStatement<'a>> {
        Box::new_in(self.do_while_statement(span, body, test), self.allocator)
    }

    /// Builds a [`WhileStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_while_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - body
    #[inline]
    pub fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> WhileStatement<'a> {
        WhileStatement { span, test, body }
    }

    /// Builds a [`WhileStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::while_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - body
    #[inline]
    pub fn alloc_while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WhileStatement<'a>> {
        Box::new_in(self.while_statement(span, test, body), self.allocator)
    }

    /// Builds a [`ForStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - init
    /// - test
    /// - update
    /// - body
    #[inline]
    pub fn for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> ForStatement<'a> {
        ForStatement { span, init, test, update, body, scope_id: Default::default() }
    }

    /// Builds a [`ForStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - init
    /// - test
    /// - update
    /// - body
    #[inline]
    pub fn alloc_for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Box<'a, ForStatement<'a>> {
        Box::new_in(self.for_statement(span, init, test, update, body), self.allocator)
    }

    /// Build a [`ForStatementInit::VariableDeclaration`]
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - declarations
    /// - declare
    #[inline]
    pub fn for_statement_init_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ForStatementInit<'a> {
        ForStatementInit::VariableDeclaration(self.alloc(self.variable_declaration(
            span,
            kind,
            declarations,
            declare,
        )))
    }

    /// Convert a [`VariableDeclaration`] into a [`ForStatementInit::VariableDeclaration`]
    #[inline]
    pub fn for_statement_init_from_variable_declaration<T>(self, inner: T) -> ForStatementInit<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        ForStatementInit::VariableDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ForStatementInit::UsingDeclaration`]
    ///
    /// This node contains a [`UsingDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_await
    /// - declarations
    #[inline]
    pub fn for_statement_init_using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> ForStatementInit<'a> {
        ForStatementInit::UsingDeclaration(self.alloc(self.using_declaration(
            span,
            is_await,
            declarations,
        )))
    }

    /// Convert a [`UsingDeclaration`] into a [`ForStatementInit::UsingDeclaration`]
    #[inline]
    pub fn for_statement_init_from_using_declaration<T>(self, inner: T) -> ForStatementInit<'a>
    where
        T: IntoIn<'a, Box<'a, UsingDeclaration<'a>>>,
    {
        ForStatementInit::UsingDeclaration(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn for_statement_init_expression(self, inner: Expression<'a>) -> ForStatementInit<'a> {
        ForStatementInit::from(inner)
    }

    /// Builds a [`ForInStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_in_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForInStatement<'a> {
        ForInStatement { span, left, right, body, scope_id: Default::default() }
    }

    /// Builds a [`ForInStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_in_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn alloc_for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForInStatement<'a>> {
        Box::new_in(self.for_in_statement(span, left, right, body), self.allocator)
    }

    /// Build a [`ForStatementLeft::VariableDeclaration`]
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - declarations
    /// - declare
    #[inline]
    pub fn for_statement_left_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ForStatementLeft<'a> {
        ForStatementLeft::VariableDeclaration(self.alloc(self.variable_declaration(
            span,
            kind,
            declarations,
            declare,
        )))
    }

    /// Convert a [`VariableDeclaration`] into a [`ForStatementLeft::VariableDeclaration`]
    #[inline]
    pub fn for_statement_left_from_variable_declaration<T>(self, inner: T) -> ForStatementLeft<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        ForStatementLeft::VariableDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ForStatementLeft::UsingDeclaration`]
    ///
    /// This node contains a [`UsingDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_await
    /// - declarations
    #[inline]
    pub fn for_statement_left_using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> ForStatementLeft<'a> {
        ForStatementLeft::UsingDeclaration(self.alloc(self.using_declaration(
            span,
            is_await,
            declarations,
        )))
    }

    /// Convert a [`UsingDeclaration`] into a [`ForStatementLeft::UsingDeclaration`]
    #[inline]
    pub fn for_statement_left_from_using_declaration<T>(self, inner: T) -> ForStatementLeft<'a>
    where
        T: IntoIn<'a, Box<'a, UsingDeclaration<'a>>>,
    {
        ForStatementLeft::UsingDeclaration(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn for_statement_left_assignment_target(
        self,
        inner: AssignmentTarget<'a>,
    ) -> ForStatementLeft<'a> {
        ForStatementLeft::from(inner)
    }

    /// Builds a [`ForOfStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_of_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#await
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForOfStatement<'a> {
        ForOfStatement { span, r#await, left, right, body, scope_id: Default::default() }
    }

    /// Builds a [`ForOfStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_of_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#await
    /// - left
    /// - right
    /// - body
    #[inline]
    pub fn alloc_for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForOfStatement<'a>> {
        Box::new_in(self.for_of_statement(span, r#await, left, right, body), self.allocator)
    }

    /// Builds a [`ContinueStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_continue_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ContinueStatement<'a> {
        ContinueStatement { span, label }
    }

    /// Builds a [`ContinueStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::continue_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn alloc_continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, ContinueStatement<'a>> {
        Box::new_in(self.continue_statement(span, label), self.allocator)
    }

    /// Builds a [`BreakStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_break_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> BreakStatement<'a> {
        BreakStatement { span, label }
    }

    /// Builds a [`BreakStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::break_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    #[inline]
    pub fn alloc_break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, BreakStatement<'a>> {
        Box::new_in(self.break_statement(span, label), self.allocator)
    }

    /// Builds a [`ReturnStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_return_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> ReturnStatement<'a> {
        ReturnStatement { span, argument }
    }

    /// Builds a [`ReturnStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::return_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn alloc_return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, ReturnStatement<'a>> {
        Box::new_in(self.return_statement(span, argument), self.allocator)
    }

    /// Builds a [`WithStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_with_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - body
    #[inline]
    pub fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> WithStatement<'a> {
        WithStatement { span, object, body }
    }

    /// Builds a [`WithStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::with_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object
    /// - body
    #[inline]
    pub fn alloc_with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WithStatement<'a>> {
        Box::new_in(self.with_statement(span, object, body), self.allocator)
    }

    /// Builds a [`SwitchStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_switch_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - discriminant
    /// - cases
    #[inline]
    pub fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> SwitchStatement<'a> {
        SwitchStatement { span, discriminant, cases, scope_id: Default::default() }
    }

    /// Builds a [`SwitchStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::switch_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - discriminant
    /// - cases
    #[inline]
    pub fn alloc_switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Box<'a, SwitchStatement<'a>> {
        Box::new_in(self.switch_statement(span, discriminant, cases), self.allocator)
    }

    /// Builds a [`SwitchCase`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_switch_case`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    #[inline]
    pub fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { span, test, consequent }
    }

    /// Builds a [`SwitchCase`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::switch_case`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - test
    /// - consequent
    #[inline]
    pub fn alloc_switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> Box<'a, SwitchCase<'a>> {
        Box::new_in(self.switch_case(span, test, consequent), self.allocator)
    }

    /// Builds a [`LabeledStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_labeled_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    /// - body
    #[inline]
    pub fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> LabeledStatement<'a> {
        LabeledStatement { span, label, body }
    }

    /// Builds a [`LabeledStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::labeled_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - label
    /// - body
    #[inline]
    pub fn alloc_labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Box<'a, LabeledStatement<'a>> {
        Box::new_in(self.labeled_statement(span, label, body), self.allocator)
    }

    /// Builds a [`ThrowStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_throw_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn throw_statement(self, span: Span, argument: Expression<'a>) -> ThrowStatement<'a> {
        ThrowStatement { span, argument }
    }

    /// Builds a [`ThrowStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::throw_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn alloc_throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, ThrowStatement<'a>> {
        Box::new_in(self.throw_statement(span, argument), self.allocator)
    }

    /// Builds a [`TryStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_try_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block
    /// - handler
    /// - finalizer
    #[inline]
    pub fn try_statement<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> TryStatement<'a>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        TryStatement {
            span,
            block: block.into_in(self.allocator),
            handler: handler.into_in(self.allocator),
            finalizer: finalizer.into_in(self.allocator),
        }
    }

    /// Builds a [`TryStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::try_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block
    /// - handler
    /// - finalizer
    #[inline]
    pub fn alloc_try_statement<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> Box<'a, TryStatement<'a>>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        Box::new_in(self.try_statement(span, block, handler, finalizer), self.allocator)
    }

    /// Builds a [`CatchClause`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_catch_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param
    /// - body
    #[inline]
    pub fn catch_clause<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        CatchClause {
            span,
            param,
            body: body.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Builds a [`CatchClause`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::catch_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param
    /// - body
    #[inline]
    pub fn alloc_catch_clause<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> Box<'a, CatchClause<'a>>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Box::new_in(self.catch_clause(span, param, body), self.allocator)
    }

    /// Builds a [`CatchParameter`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_catch_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - pattern
    #[inline]
    pub fn catch_parameter(self, span: Span, pattern: BindingPattern<'a>) -> CatchParameter<'a> {
        CatchParameter { span, pattern }
    }

    /// Builds a [`CatchParameter`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::catch_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - pattern
    #[inline]
    pub fn alloc_catch_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> Box<'a, CatchParameter<'a>> {
        Box::new_in(self.catch_parameter(span, pattern), self.allocator)
    }

    /// Builds a [`DebuggerStatement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_debugger_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn debugger_statement(self, span: Span) -> DebuggerStatement {
        DebuggerStatement { span }
    }

    /// Builds a [`DebuggerStatement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::debugger_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_debugger_statement(self, span: Span) -> Box<'a, DebuggerStatement> {
        Box::new_in(self.debugger_statement(span), self.allocator)
    }

    /// Builds a [`BindingPattern`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binding_pattern`] instead.
    ///
    /// ## Parameters
    /// - kind
    /// - type_annotation
    /// - optional
    #[inline]
    pub fn binding_pattern<T1>(
        self,
        kind: BindingPatternKind<'a>,
        type_annotation: T1,
        optional: bool,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        BindingPattern { kind, type_annotation: type_annotation.into_in(self.allocator), optional }
    }

    /// Builds a [`BindingPattern`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binding_pattern`] instead.
    ///
    /// ## Parameters
    /// - kind
    /// - type_annotation
    /// - optional
    #[inline]
    pub fn alloc_binding_pattern<T1>(
        self,
        kind: BindingPatternKind<'a>,
        type_annotation: T1,
        optional: bool,
    ) -> Box<'a, BindingPattern<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(self.binding_pattern(kind, type_annotation, optional), self.allocator)
    }

    /// Build a [`BindingPatternKind::BindingIdentifier`]
    ///
    /// This node contains a [`BindingIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    #[inline]
    pub fn binding_pattern_kind_binding_identifier<A>(
        self,
        span: Span,
        name: A,
    ) -> BindingPatternKind<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BindingPatternKind::BindingIdentifier(self.alloc(self.binding_identifier(span, name)))
    }

    /// Convert a [`BindingIdentifier`] into a [`BindingPatternKind::BindingIdentifier`]
    #[inline]
    pub fn binding_pattern_kind_from_binding_identifier<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, BindingIdentifier<'a>>>,
    {
        BindingPatternKind::BindingIdentifier(inner.into_in(self.allocator))
    }

    /// Build a [`BindingPatternKind::ObjectPattern`]
    ///
    /// This node contains a [`ObjectPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn binding_pattern_kind_object_pattern<T1>(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> BindingPatternKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        BindingPatternKind::ObjectPattern(self.alloc(self.object_pattern(span, properties, rest)))
    }

    /// Convert a [`ObjectPattern`] into a [`BindingPatternKind::ObjectPattern`]
    #[inline]
    pub fn binding_pattern_kind_from_object_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectPattern<'a>>>,
    {
        BindingPatternKind::ObjectPattern(inner.into_in(self.allocator))
    }

    /// Build a [`BindingPatternKind::ArrayPattern`]
    ///
    /// This node contains a [`ArrayPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    #[inline]
    pub fn binding_pattern_kind_array_pattern<T1>(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> BindingPatternKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        BindingPatternKind::ArrayPattern(self.alloc(self.array_pattern(span, elements, rest)))
    }

    /// Convert a [`ArrayPattern`] into a [`BindingPatternKind::ArrayPattern`]
    #[inline]
    pub fn binding_pattern_kind_from_array_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, ArrayPattern<'a>>>,
    {
        BindingPatternKind::ArrayPattern(inner.into_in(self.allocator))
    }

    /// Build a [`BindingPatternKind::AssignmentPattern`]
    ///
    /// This node contains a [`AssignmentPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn binding_pattern_kind_assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> BindingPatternKind<'a> {
        BindingPatternKind::AssignmentPattern(
            self.alloc(self.assignment_pattern(span, left, right)),
        )
    }

    /// Convert a [`AssignmentPattern`] into a [`BindingPatternKind::AssignmentPattern`]
    #[inline]
    pub fn binding_pattern_kind_from_assignment_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentPattern<'a>>>,
    {
        BindingPatternKind::AssignmentPattern(inner.into_in(self.allocator))
    }

    /// Builds a [`AssignmentPattern`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_assignment_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> AssignmentPattern<'a> {
        AssignmentPattern { span, left, right }
    }

    /// Builds a [`AssignmentPattern`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::assignment_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn alloc_assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentPattern<'a>> {
        Box::new_in(self.assignment_pattern(span, left, right), self.allocator)
    }

    /// Builds a [`ObjectPattern`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_object_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn object_pattern<T1>(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> ObjectPattern<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        ObjectPattern { span, properties, rest: rest.into_in(self.allocator) }
    }

    /// Builds a [`ObjectPattern`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::object_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - properties
    /// - rest
    #[inline]
    pub fn alloc_object_pattern<T1>(
        self,
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> Box<'a, ObjectPattern<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Box::new_in(self.object_pattern(span, properties, rest), self.allocator)
    }

    /// Builds a [`BindingProperty`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binding_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - value
    /// - shorthand
    /// - computed
    #[inline]
    pub fn binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a> {
        BindingProperty { span, key, value, shorthand, computed }
    }

    /// Builds a [`BindingProperty`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binding_property`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - value
    /// - shorthand
    /// - computed
    #[inline]
    pub fn alloc_binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, BindingProperty<'a>> {
        Box::new_in(self.binding_property(span, key, value, shorthand, computed), self.allocator)
    }

    /// Builds a [`ArrayPattern`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_array_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    #[inline]
    pub fn array_pattern<T1>(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> ArrayPattern<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        ArrayPattern { span, elements, rest: rest.into_in(self.allocator) }
    }

    /// Builds a [`ArrayPattern`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::array_pattern`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - elements
    /// - rest
    #[inline]
    pub fn alloc_array_pattern<T1>(
        self,
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> Box<'a, ArrayPattern<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Box::new_in(self.array_pattern(span, elements, rest), self.allocator)
    }

    /// Builds a [`BindingRestElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binding_rest_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a> {
        BindingRestElement { span, argument }
    }

    /// Builds a [`BindingRestElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binding_rest_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn alloc_binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> Box<'a, BindingRestElement<'a>> {
        Box::new_in(self.binding_rest_element(span, argument), self.allocator)
    }

    /// Builds a [`Function`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_function`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id
    /// - generator
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params
    /// - return_type
    /// - body
    #[inline]
    pub fn function<T1, T2, T3, T4>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Function {
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            body: body.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Builds a [`Function`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::function`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id
    /// - generator
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params
    /// - return_type
    /// - body
    #[inline]
    pub fn alloc_function<T1, T2, T3, T4>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Box<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Box::new_in(
            self.function(
                r#type,
                span,
                id,
                generator,
                r#async,
                declare,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
            ),
            self.allocator,
        )
    }

    /// Builds a [`FormalParameters`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_formal_parameters`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - items
    /// - rest
    #[inline]
    pub fn formal_parameters<T1>(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> FormalParameters<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        FormalParameters { span, kind, items, rest: rest.into_in(self.allocator) }
    }

    /// Builds a [`FormalParameters`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::formal_parameters`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - kind
    /// - items
    /// - rest
    #[inline]
    pub fn alloc_formal_parameters<T1>(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> Box<'a, FormalParameters<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Box::new_in(self.formal_parameters(span, kind, items, rest), self.allocator)
    }

    /// Builds a [`FormalParameter`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_formal_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - decorators
    /// - pattern
    /// - accessibility
    /// - readonly
    /// - r#override
    #[inline]
    pub fn formal_parameter(
        self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
    ) -> FormalParameter<'a> {
        FormalParameter { span, decorators, pattern, accessibility, readonly, r#override }
    }

    /// Builds a [`FormalParameter`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::formal_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - decorators
    /// - pattern
    /// - accessibility
    /// - readonly
    /// - r#override
    #[inline]
    pub fn alloc_formal_parameter(
        self,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
    ) -> Box<'a, FormalParameter<'a>> {
        Box::new_in(
            self.formal_parameter(span, decorators, pattern, accessibility, readonly, r#override),
            self.allocator,
        )
    }

    /// Builds a [`FunctionBody`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_function_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - directives
    /// - statements
    #[inline]
    pub fn function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> FunctionBody<'a> {
        FunctionBody { span, directives, statements }
    }

    /// Builds a [`FunctionBody`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::function_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - directives
    /// - statements
    #[inline]
    pub fn alloc_function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        Box::new_in(self.function_body(span, directives, statements), self.allocator)
    }

    /// Builds a [`ArrowFunctionExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_arrow_function_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// - r#async
    /// - type_parameters
    /// - params
    /// - return_type
    /// - body: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn arrow_function_expression<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> ArrowFunctionExpression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        ArrowFunctionExpression {
            span,
            expression,
            r#async,
            type_parameters: type_parameters.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            body: body.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Builds a [`ArrowFunctionExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::arrow_function_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// - r#async
    /// - type_parameters
    /// - params
    /// - return_type
    /// - body: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn alloc_arrow_function_expression<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> Box<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Box::new_in(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            ),
            self.allocator,
        )
    }

    /// Builds a [`YieldExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_yield_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - delegate
    /// - argument
    #[inline]
    pub fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> YieldExpression<'a> {
        YieldExpression { span, delegate, argument }
    }

    /// Builds a [`YieldExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::yield_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - delegate
    /// - argument
    #[inline]
    pub fn alloc_yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, YieldExpression<'a>> {
        Box::new_in(self.yield_expression(span, delegate, argument), self.allocator)
    }

    /// Builds a [`Class`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_class`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the class.
    /// - id: Class identifier, AKA the name
    /// - type_parameters
    /// - super_class: Super class. When present, this will usually be an [`IdentifierReference`].
    /// - super_type_parameters: Type parameters passed to super class.
    /// - implements: Interface implementation clause for TypeScript classes.
    /// - body
    /// - r#abstract: Whether the class is abstract
    /// - declare: Whether the class was `declare`ed
    #[inline]
    pub fn class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_parameters: T2,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Class<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Class {
            r#type,
            span,
            decorators,
            id,
            type_parameters: type_parameters.into_in(self.allocator),
            super_class,
            super_type_parameters: super_type_parameters.into_in(self.allocator),
            implements,
            body: body.into_in(self.allocator),
            r#abstract,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Builds a [`Class`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::class`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the class.
    /// - id: Class identifier, AKA the name
    /// - type_parameters
    /// - super_class: Super class. When present, this will usually be an [`IdentifierReference`].
    /// - super_type_parameters: Type parameters passed to super class.
    /// - implements: Interface implementation clause for TypeScript classes.
    /// - body
    /// - r#abstract: Whether the class is abstract
    /// - declare: Whether the class was `declare`ed
    #[inline]
    pub fn alloc_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_parameters: T2,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Box<'a, Class<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Box::new_in(
            self.class(
                r#type,
                span,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_parameters,
                implements,
                body,
                r#abstract,
                declare,
            ),
            self.allocator,
        )
    }

    /// Builds a [`ClassBody`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_class_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn class_body(self, span: Span, body: Vec<'a, ClassElement<'a>>) -> ClassBody<'a> {
        ClassBody { span, body }
    }

    /// Builds a [`ClassBody`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::class_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn alloc_class_body(
        self,
        span: Span,
        body: Vec<'a, ClassElement<'a>>,
    ) -> Box<'a, ClassBody<'a>> {
        Box::new_in(self.class_body(span, body), self.allocator)
    }

    /// Build a [`ClassElement::StaticBlock`]
    ///
    /// This node contains a [`StaticBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn class_element_static_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc(self.static_block(span, body)))
    }

    /// Convert a [`StaticBlock`] into a [`ClassElement::StaticBlock`]
    #[inline]
    pub fn class_element_from_static_block<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, StaticBlock<'a>>>,
    {
        ClassElement::StaticBlock(inner.into_in(self.allocator))
    }

    /// Build a [`ClassElement::MethodDefinition`]
    ///
    /// This node contains a [`MethodDefinition`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type: Method definition type
    /// - span: The [`Span`] covering this node
    /// - decorators
    /// - key
    /// - value
    /// - kind
    /// - computed
    /// - r#static
    /// - r#override
    /// - optional
    /// - accessibility
    #[inline]
    pub fn class_element_method_definition<T1>(
        self,
        r#type: MethodDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        ClassElement::MethodDefinition(self.alloc(self.method_definition(
            r#type,
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
        )))
    }

    /// Convert a [`MethodDefinition`] into a [`ClassElement::MethodDefinition`]
    #[inline]
    pub fn class_element_from_method_definition<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, MethodDefinition<'a>>>,
    {
        ClassElement::MethodDefinition(inner.into_in(self.allocator))
    }

    /// Build a [`ClassElement::PropertyDefinition`]
    ///
    /// This node contains a [`PropertyDefinition`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - declare: Property is declared with a `declare` modifier.
    /// - r#override
    /// - optional: `true` when created with an optional modifier (`?`)
    /// - definite
    /// - readonly: `true` when declared with a `readonly` modifier
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn class_element_property_definition<T1>(
        self,
        r#type: PropertyDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        ClassElement::PropertyDefinition(self.alloc(self.property_definition(
            r#type,
            span,
            decorators,
            key,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            type_annotation,
            accessibility,
        )))
    }

    /// Convert a [`PropertyDefinition`] into a [`ClassElement::PropertyDefinition`]
    #[inline]
    pub fn class_element_from_property_definition<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, PropertyDefinition<'a>>>,
    {
        ClassElement::PropertyDefinition(inner.into_in(self.allocator))
    }

    /// Build a [`ClassElement::AccessorProperty`]
    ///
    /// This node contains a [`AccessorProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed
    /// - r#static
    #[inline]
    pub fn class_element_accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
    ) -> ClassElement<'a> {
        ClassElement::AccessorProperty(self.alloc(
            self.accessor_property(r#type, span, decorators, key, value, computed, r#static),
        ))
    }

    /// Convert a [`AccessorProperty`] into a [`ClassElement::AccessorProperty`]
    #[inline]
    pub fn class_element_from_accessor_property<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, AccessorProperty<'a>>>,
    {
        ClassElement::AccessorProperty(inner.into_in(self.allocator))
    }

    /// Build a [`ClassElement::TSIndexSignature`]
    ///
    /// This node contains a [`TSIndexSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    #[inline]
    pub fn class_element_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        ClassElement::TSIndexSignature(self.alloc(self.ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
        )))
    }

    /// Convert a [`TSIndexSignature`] into a [`ClassElement::TSIndexSignature`]
    #[inline]
    pub fn class_element_from_ts_index_signature<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexSignature<'a>>>,
    {
        ClassElement::TSIndexSignature(inner.into_in(self.allocator))
    }

    /// Builds a [`MethodDefinition`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_method_definition`] instead.
    ///
    /// ## Parameters
    /// - r#type: Method definition type
    /// - span: The [`Span`] covering this node
    /// - decorators
    /// - key
    /// - value
    /// - kind
    /// - computed
    /// - r#static
    /// - r#override
    /// - optional
    /// - accessibility
    #[inline]
    pub fn method_definition<T1>(
        self,
        r#type: MethodDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
    ) -> MethodDefinition<'a>
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        MethodDefinition {
            r#type,
            span,
            decorators,
            key,
            value: value.into_in(self.allocator),
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
        }
    }

    /// Builds a [`MethodDefinition`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::method_definition`] instead.
    ///
    /// ## Parameters
    /// - r#type: Method definition type
    /// - span: The [`Span`] covering this node
    /// - decorators
    /// - key
    /// - value
    /// - kind
    /// - computed
    /// - r#static
    /// - r#override
    /// - optional
    /// - accessibility
    #[inline]
    pub fn alloc_method_definition<T1>(
        self,
        r#type: MethodDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
    ) -> Box<'a, MethodDefinition<'a>>
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Box::new_in(
            self.method_definition(
                r#type,
                span,
                decorators,
                key,
                value,
                kind,
                computed,
                r#static,
                r#override,
                optional,
                accessibility,
            ),
            self.allocator,
        )
    }

    /// Builds a [`PropertyDefinition`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_property_definition`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - declare: Property is declared with a `declare` modifier.
    /// - r#override
    /// - optional: `true` when created with an optional modifier (`?`)
    /// - definite
    /// - readonly: `true` when declared with a `readonly` modifier
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn property_definition<T1>(
        self,
        r#type: PropertyDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> PropertyDefinition<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        PropertyDefinition {
            r#type,
            span,
            decorators,
            key,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            type_annotation: type_annotation.into_in(self.allocator),
            accessibility,
        }
    }

    /// Builds a [`PropertyDefinition`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::property_definition`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - declare: Property is declared with a `declare` modifier.
    /// - r#override
    /// - optional: `true` when created with an optional modifier (`?`)
    /// - definite
    /// - readonly: `true` when declared with a `readonly` modifier
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn alloc_property_definition<T1>(
        self,
        r#type: PropertyDefinitionType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> Box<'a, PropertyDefinition<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.property_definition(
                r#type,
                span,
                decorators,
                key,
                value,
                computed,
                r#static,
                declare,
                r#override,
                optional,
                definite,
                readonly,
                type_annotation,
                accessibility,
            ),
            self.allocator,
        )
    }

    /// Builds a [`PrivateIdentifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_private_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn private_identifier<A>(self, span: Span, name: A) -> PrivateIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PrivateIdentifier { span, name: name.into_in(self.allocator) }
    }

    /// Builds a [`PrivateIdentifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::private_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn alloc_private_identifier<A>(self, span: Span, name: A) -> Box<'a, PrivateIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.private_identifier(span, name), self.allocator)
    }

    /// Builds a [`StaticBlock`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_static_block`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn static_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> StaticBlock<'a> {
        StaticBlock { span, body, scope_id: Default::default() }
    }

    /// Builds a [`StaticBlock`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::static_block`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn alloc_static_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, StaticBlock<'a>> {
        Box::new_in(self.static_block(span, body), self.allocator)
    }

    /// Build a [`ModuleDeclaration::ImportDeclaration`]
    ///
    /// This node contains a [`ImportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - specifiers: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// - source
    /// - with_clause: Some(vec![]) for empty assertion
    /// - import_kind: `import type { foo } from 'bar'`
    #[inline]
    pub fn module_declaration_import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::ImportDeclaration(self.alloc(self.import_declaration(
            span,
            specifiers,
            source,
            with_clause,
            import_kind,
        )))
    }

    /// Convert a [`ImportDeclaration`] into a [`ModuleDeclaration::ImportDeclaration`]
    #[inline]
    pub fn module_declaration_from_import_declaration<T>(self, inner: T) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, ImportDeclaration<'a>>>,
    {
        ModuleDeclaration::ImportDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleDeclaration::ExportAllDeclaration`]
    ///
    /// This node contains a [`ExportAllDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - exported: If this declaration is re-named
    /// - source
    /// - with_clause: Will be `Some(vec![])` for empty assertion
    /// - export_kind
    #[inline]
    pub fn module_declaration_export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::ExportAllDeclaration(self.alloc(self.export_all_declaration(
            span,
            exported,
            source,
            with_clause,
            export_kind,
        )))
    }

    /// Convert a [`ExportAllDeclaration`] into a [`ModuleDeclaration::ExportAllDeclaration`]
    #[inline]
    pub fn module_declaration_from_export_all_declaration<T>(
        self,
        inner: T,
    ) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, ExportAllDeclaration<'a>>>,
    {
        ModuleDeclaration::ExportAllDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleDeclaration::ExportDefaultDeclaration`]
    ///
    /// This node contains a [`ExportDefaultDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - exported
    #[inline]
    pub fn module_declaration_export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::ExportDefaultDeclaration(self.alloc(self.export_default_declaration(
            span,
            declaration,
            exported,
        )))
    }

    /// Convert a [`ExportDefaultDeclaration`] into a [`ModuleDeclaration::ExportDefaultDeclaration`]
    #[inline]
    pub fn module_declaration_from_export_default_declaration<T>(
        self,
        inner: T,
    ) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, ExportDefaultDeclaration<'a>>>,
    {
        ModuleDeclaration::ExportDefaultDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleDeclaration::ExportNamedDeclaration`]
    ///
    /// This node contains a [`ExportNamedDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - specifiers
    /// - source
    /// - export_kind: `export type { foo }`
    /// - with_clause: Some(vec![]) for empty assertion
    #[inline]
    pub fn module_declaration_export_named_declaration(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: Option<WithClause<'a>>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::ExportNamedDeclaration(self.alloc(self.export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        )))
    }

    /// Convert a [`ExportNamedDeclaration`] into a [`ModuleDeclaration::ExportNamedDeclaration`]
    #[inline]
    pub fn module_declaration_from_export_named_declaration<T>(
        self,
        inner: T,
    ) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, ExportNamedDeclaration<'a>>>,
    {
        ModuleDeclaration::ExportNamedDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleDeclaration::TSExportAssignment`]
    ///
    /// This node contains a [`TSExportAssignment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn module_declaration_ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::TSExportAssignment(
            self.alloc(self.ts_export_assignment(span, expression)),
        )
    }

    /// Convert a [`TSExportAssignment`] into a [`ModuleDeclaration::TSExportAssignment`]
    #[inline]
    pub fn module_declaration_from_ts_export_assignment<T>(self, inner: T) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSExportAssignment<'a>>>,
    {
        ModuleDeclaration::TSExportAssignment(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleDeclaration::TSNamespaceExportDeclaration`]
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    #[inline]
    pub fn module_declaration_ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::TSNamespaceExportDeclaration(
            self.alloc(self.ts_namespace_export_declaration(span, id)),
        )
    }

    /// Convert a [`TSNamespaceExportDeclaration`] into a [`ModuleDeclaration::TSNamespaceExportDeclaration`]
    #[inline]
    pub fn module_declaration_from_ts_namespace_export_declaration<T>(
        self,
        inner: T,
    ) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSNamespaceExportDeclaration<'a>>>,
    {
        ModuleDeclaration::TSNamespaceExportDeclaration(inner.into_in(self.allocator))
    }

    /// Builds a [`AccessorProperty`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_accessor_property`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed
    /// - r#static
    #[inline]
    pub fn accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
    ) -> AccessorProperty<'a> {
        AccessorProperty { r#type, span, decorators, key, value, computed, r#static }
    }

    /// Builds a [`AccessorProperty`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::accessor_property`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed
    /// - r#static
    #[inline]
    pub fn alloc_accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
    ) -> Box<'a, AccessorProperty<'a>> {
        Box::new_in(
            self.accessor_property(r#type, span, decorators, key, value, computed, r#static),
            self.allocator,
        )
    }

    /// Builds a [`ImportExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source
    /// - arguments
    #[inline]
    pub fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> ImportExpression<'a> {
        ImportExpression { span, source, arguments }
    }

    /// Builds a [`ImportExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source
    /// - arguments
    #[inline]
    pub fn alloc_import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Box<'a, ImportExpression<'a>> {
        Box::new_in(self.import_expression(span, source, arguments), self.allocator)
    }

    /// Builds a [`ImportDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - specifiers: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// - source
    /// - with_clause: Some(vec![]) for empty assertion
    /// - import_kind: `import type { foo } from 'bar'`
    #[inline]
    pub fn import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration<'a> {
        ImportDeclaration { span, specifiers, source, with_clause, import_kind }
    }

    /// Builds a [`ImportDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - specifiers: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// - source
    /// - with_clause: Some(vec![]) for empty assertion
    /// - import_kind: `import type { foo } from 'bar'`
    #[inline]
    pub fn alloc_import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportDeclaration<'a>> {
        Box::new_in(
            self.import_declaration(span, specifiers, source, with_clause, import_kind),
            self.allocator,
        )
    }

    /// Build a [`ImportDeclarationSpecifier::ImportSpecifier`]
    ///
    /// This node contains a [`ImportSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - imported
    /// - local: The name of the imported symbol.
    /// - import_kind
    #[inline]
    pub fn import_declaration_specifier_import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportSpecifier(self.alloc(self.import_specifier(
            span,
            imported,
            local,
            import_kind,
        )))
    }

    /// Convert a [`ImportSpecifier`] into a [`ImportDeclarationSpecifier::ImportSpecifier`]
    #[inline]
    pub fn import_declaration_specifier_from_import_specifier<T>(
        self,
        inner: T,
    ) -> ImportDeclarationSpecifier<'a>
    where
        T: IntoIn<'a, Box<'a, ImportSpecifier<'a>>>,
    {
        ImportDeclarationSpecifier::ImportSpecifier(inner.into_in(self.allocator))
    }

    /// Build a [`ImportDeclarationSpecifier::ImportDefaultSpecifier`]
    ///
    /// This node contains a [`ImportDefaultSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local: The name of the imported symbol.
    #[inline]
    pub fn import_declaration_specifier_import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(
            self.alloc(self.import_default_specifier(span, local)),
        )
    }

    /// Convert a [`ImportDefaultSpecifier`] into a [`ImportDeclarationSpecifier::ImportDefaultSpecifier`]
    #[inline]
    pub fn import_declaration_specifier_from_import_default_specifier<T>(
        self,
        inner: T,
    ) -> ImportDeclarationSpecifier<'a>
    where
        T: IntoIn<'a, Box<'a, ImportDefaultSpecifier<'a>>>,
    {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(inner.into_in(self.allocator))
    }

    /// Build a [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`]
    ///
    /// This node contains a [`ImportNamespaceSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local
    #[inline]
    pub fn import_declaration_specifier_import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            self.alloc(self.import_namespace_specifier(span, local)),
        )
    }

    /// Convert a [`ImportNamespaceSpecifier`] into a [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`]
    #[inline]
    pub fn import_declaration_specifier_from_import_namespace_specifier<T>(
        self,
        inner: T,
    ) -> ImportDeclarationSpecifier<'a>
    where
        T: IntoIn<'a, Box<'a, ImportNamespaceSpecifier<'a>>>,
    {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(inner.into_in(self.allocator))
    }

    /// Builds a [`ImportSpecifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - imported
    /// - local: The name of the imported symbol.
    /// - import_kind
    #[inline]
    pub fn import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier { span, imported, local, import_kind }
    }

    /// Builds a [`ImportSpecifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - imported
    /// - local: The name of the imported symbol.
    /// - import_kind
    #[inline]
    pub fn alloc_import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportSpecifier<'a>> {
        Box::new_in(self.import_specifier(span, imported, local, import_kind), self.allocator)
    }

    /// Builds a [`ImportDefaultSpecifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_default_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local: The name of the imported symbol.
    #[inline]
    pub fn import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDefaultSpecifier<'a> {
        ImportDefaultSpecifier { span, local }
    }

    /// Builds a [`ImportDefaultSpecifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_default_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local: The name of the imported symbol.
    #[inline]
    pub fn alloc_import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportDefaultSpecifier<'a>> {
        Box::new_in(self.import_default_specifier(span, local), self.allocator)
    }

    /// Builds a [`ImportNamespaceSpecifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_namespace_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local
    #[inline]
    pub fn import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportNamespaceSpecifier<'a> {
        ImportNamespaceSpecifier { span, local }
    }

    /// Builds a [`ImportNamespaceSpecifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_namespace_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local
    #[inline]
    pub fn alloc_import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportNamespaceSpecifier<'a>> {
        Box::new_in(self.import_namespace_specifier(span, local), self.allocator)
    }

    /// Builds a [`WithClause`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_with_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - attributes_keyword
    /// - with_entries
    #[inline]
    pub fn with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> WithClause<'a> {
        WithClause { span, attributes_keyword, with_entries }
    }

    /// Builds a [`WithClause`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::with_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - attributes_keyword
    /// - with_entries
    #[inline]
    pub fn alloc_with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> Box<'a, WithClause<'a>> {
        Box::new_in(self.with_clause(span, attributes_keyword, with_entries), self.allocator)
    }

    /// Builds a [`ImportAttribute`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_import_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - value
    #[inline]
    pub fn import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> ImportAttribute<'a> {
        ImportAttribute { span, key, value }
    }

    /// Builds a [`ImportAttribute`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::import_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - value
    #[inline]
    pub fn alloc_import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> Box<'a, ImportAttribute<'a>> {
        Box::new_in(self.import_attribute(span, key, value), self.allocator)
    }

    /// Build a [`ImportAttributeKey::Identifier`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn import_attribute_key_identifier_name<A>(
        self,
        span: Span,
        name: A,
    ) -> ImportAttributeKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ImportAttributeKey::Identifier(self.identifier_name(span, name))
    }

    /// Convert a [`IdentifierName`] into a [`ImportAttributeKey::Identifier`]
    #[inline]
    pub fn import_attribute_key_from_identifier_name<T>(self, inner: T) -> ImportAttributeKey<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        ImportAttributeKey::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`ImportAttributeKey::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn import_attribute_key_string_literal<A>(
        self,
        span: Span,
        value: A,
    ) -> ImportAttributeKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ImportAttributeKey::StringLiteral(self.string_literal(span, value))
    }

    /// Convert a [`StringLiteral`] into a [`ImportAttributeKey::StringLiteral`]
    #[inline]
    pub fn import_attribute_key_from_string_literal<T>(self, inner: T) -> ImportAttributeKey<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        ImportAttributeKey::StringLiteral(inner.into_in(self.allocator))
    }

    /// Builds a [`ExportNamedDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_export_named_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - specifiers
    /// - source
    /// - export_kind: `export type { foo }`
    /// - with_clause: Some(vec![]) for empty assertion
    #[inline]
    pub fn export_named_declaration(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: Option<WithClause<'a>>,
    ) -> ExportNamedDeclaration<'a> {
        ExportNamedDeclaration { span, declaration, specifiers, source, export_kind, with_clause }
    }

    /// Builds a [`ExportNamedDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::export_named_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - specifiers
    /// - source
    /// - export_kind: `export type { foo }`
    /// - with_clause: Some(vec![]) for empty assertion
    #[inline]
    pub fn alloc_export_named_declaration(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: Option<WithClause<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        Box::new_in(
            self.export_named_declaration(
                span,
                declaration,
                specifiers,
                source,
                export_kind,
                with_clause,
            ),
            self.allocator,
        )
    }

    /// Builds a [`ExportDefaultDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_export_default_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - exported
    #[inline]
    pub fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> ExportDefaultDeclaration<'a> {
        ExportDefaultDeclaration { span, declaration, exported }
    }

    /// Builds a [`ExportDefaultDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::export_default_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - exported
    #[inline]
    pub fn alloc_export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        Box::new_in(self.export_default_declaration(span, declaration, exported), self.allocator)
    }

    /// Builds a [`ExportAllDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_export_all_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - exported: If this declaration is re-named
    /// - source
    /// - with_clause: Will be `Some(vec![])` for empty assertion
    /// - export_kind
    #[inline]
    pub fn export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration<'a> {
        ExportAllDeclaration { span, exported, source, with_clause, export_kind }
    }

    /// Builds a [`ExportAllDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::export_all_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - exported: If this declaration is re-named
    /// - source
    /// - with_clause: Will be `Some(vec![])` for empty assertion
    /// - export_kind
    #[inline]
    pub fn alloc_export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportAllDeclaration<'a>> {
        Box::new_in(
            self.export_all_declaration(span, exported, source, with_clause, export_kind),
            self.allocator,
        )
    }

    /// Builds a [`ExportSpecifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_export_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local
    /// - exported
    /// - export_kind
    #[inline]
    pub fn export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier { span, local, exported, export_kind }
    }

    /// Builds a [`ExportSpecifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::export_specifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - local
    /// - exported
    /// - export_kind
    #[inline]
    pub fn alloc_export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportSpecifier<'a>> {
        Box::new_in(self.export_specifier(span, local, exported, export_kind), self.allocator)
    }

    /// Build a [`ExportDefaultDeclarationKind::FunctionDeclaration`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id
    /// - generator
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params
    /// - return_type
    /// - body
    #[inline]
    pub fn export_default_declaration_kind_function<T1, T2, T3, T4>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
        body: T4,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        ExportDefaultDeclarationKind::FunctionDeclaration(self.alloc(self.function(
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
        )))
    }

    /// Convert a [`Function`] into a [`ExportDefaultDeclarationKind::FunctionDeclaration`]
    #[inline]
    pub fn export_default_declaration_kind_from_function<T>(
        self,
        inner: T,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        ExportDefaultDeclarationKind::FunctionDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ExportDefaultDeclarationKind::ClassDeclaration`]
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the class.
    /// - id: Class identifier, AKA the name
    /// - type_parameters
    /// - super_class: Super class. When present, this will usually be an [`IdentifierReference`].
    /// - super_type_parameters: Type parameters passed to super class.
    /// - implements: Interface implementation clause for TypeScript classes.
    /// - body
    /// - r#abstract: Whether the class is abstract
    /// - declare: Whether the class was `declare`ed
    #[inline]
    pub fn export_default_declaration_kind_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_parameters: T2,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        ExportDefaultDeclarationKind::ClassDeclaration(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_parameters,
            implements,
            body,
            r#abstract,
            declare,
        )))
    }

    /// Convert a [`Class`] into a [`ExportDefaultDeclarationKind::ClassDeclaration`]
    #[inline]
    pub fn export_default_declaration_kind_from_class<T>(
        self,
        inner: T,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T: IntoIn<'a, Box<'a, Class<'a>>>,
    {
        ExportDefaultDeclarationKind::ClassDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`ExportDefaultDeclarationKind::TSInterfaceDeclaration`]
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends
    /// - type_parameters
    /// - body
    /// - declare
    #[inline]
    pub fn export_default_declaration_kind_ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(self.alloc(
            self.ts_interface_declaration(span, id, extends, type_parameters, body, declare),
        ))
    }

    /// Convert a [`TSInterfaceDeclaration`] into a [`ExportDefaultDeclarationKind::TSInterfaceDeclaration`]
    #[inline]
    pub fn export_default_declaration_kind_from_ts_interface_declaration<T>(
        self,
        inner: T,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T: IntoIn<'a, Box<'a, TSInterfaceDeclaration<'a>>>,
    {
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn export_default_declaration_kind_expression(
        self,
        inner: Expression<'a>,
    ) -> ExportDefaultDeclarationKind<'a> {
        ExportDefaultDeclarationKind::from(inner)
    }

    /// Build a [`ModuleExportName::IdentifierName`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn module_export_name_identifier_name<A>(self, span: Span, name: A) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::IdentifierName(self.identifier_name(span, name))
    }

    /// Convert a [`IdentifierName`] into a [`ModuleExportName::IdentifierName`]
    #[inline]
    pub fn module_export_name_from_identifier_name<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        ModuleExportName::IdentifierName(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleExportName::IdentifierReference`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn module_export_name_identifier_reference<A>(
        self,
        span: Span,
        name: A,
    ) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::IdentifierReference(self.identifier_reference(span, name))
    }

    /// Convert a [`IdentifierReference`] into a [`ModuleExportName::IdentifierReference`]
    #[inline]
    pub fn module_export_name_from_identifier_reference<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, IdentifierReference<'a>>,
    {
        ModuleExportName::IdentifierReference(inner.into_in(self.allocator))
    }

    /// Build a [`ModuleExportName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn module_export_name_string_literal<A>(self, span: Span, value: A) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::StringLiteral(self.string_literal(span, value))
    }

    /// Convert a [`StringLiteral`] into a [`ModuleExportName::StringLiteral`]
    #[inline]
    pub fn module_export_name_from_string_literal<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        ModuleExportName::StringLiteral(inner.into_in(self.allocator))
    }

    /// Builds a [`TSThisParameter`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this
    /// - type_annotation
    #[inline]
    pub fn ts_this_parameter<T1>(
        self,
        span: Span,
        this: IdentifierName<'a>,
        type_annotation: T1,
    ) -> TSThisParameter<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSThisParameter { span, this, type_annotation: type_annotation.into_in(self.allocator) }
    }

    /// Builds a [`TSThisParameter`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_this_parameter<T1>(
        self,
        span: Span,
        this: IdentifierName<'a>,
        type_annotation: T1,
    ) -> Box<'a, TSThisParameter<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(self.ts_this_parameter(span, this, type_annotation), self.allocator)
    }

    /// Builds a [`TSEnumDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const
    /// - declare
    #[inline]
    pub fn ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> TSEnumDeclaration<'a> {
        TSEnumDeclaration { span, id, members, r#const, declare, scope_id: Default::default() }
    }

    /// Builds a [`TSEnumDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const
    /// - declare
    #[inline]
    pub fn alloc_ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> Box<'a, TSEnumDeclaration<'a>> {
        Box::new_in(self.ts_enum_declaration(span, id, members, r#const, declare), self.allocator)
    }

    /// Builds a [`TSEnumMember`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_enum_member`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - initializer
    #[inline]
    pub fn ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> TSEnumMember<'a> {
        TSEnumMember { span, id, initializer }
    }

    /// Builds a [`TSEnumMember`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_enum_member`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - initializer
    #[inline]
    pub fn alloc_ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> Box<'a, TSEnumMember<'a>> {
        Box::new_in(self.ts_enum_member(span, id, initializer), self.allocator)
    }

    /// Build a [`TSEnumMemberName::StaticIdentifier`]
    ///
    /// This node contains a [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn ts_enum_member_name_identifier_name<A>(self, span: Span, name: A) -> TSEnumMemberName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSEnumMemberName::StaticIdentifier(self.alloc(self.identifier_name(span, name)))
    }

    /// Convert a [`IdentifierName`] into a [`TSEnumMemberName::StaticIdentifier`]
    #[inline]
    pub fn ts_enum_member_name_from_identifier_name<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        TSEnumMemberName::StaticIdentifier(inner.into_in(self.allocator))
    }

    /// Build a [`TSEnumMemberName::StaticStringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn ts_enum_member_name_string_literal<A>(self, span: Span, value: A) -> TSEnumMemberName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSEnumMemberName::StaticStringLiteral(self.alloc(self.string_literal(span, value)))
    }

    /// Convert a [`StringLiteral`] into a [`TSEnumMemberName::StaticStringLiteral`]
    #[inline]
    pub fn ts_enum_member_name_from_string_literal<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        TSEnumMemberName::StaticStringLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSEnumMemberName::StaticTemplateLiteral`]
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - expressions
    #[inline]
    pub fn ts_enum_member_name_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TSEnumMemberName<'a> {
        TSEnumMemberName::StaticTemplateLiteral(self.alloc(self.template_literal(
            span,
            quasis,
            expressions,
        )))
    }

    /// Convert a [`TemplateLiteral`] into a [`TSEnumMemberName::StaticTemplateLiteral`]
    #[inline]
    pub fn ts_enum_member_name_from_template_literal<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, TemplateLiteral<'a>>>,
    {
        TSEnumMemberName::StaticTemplateLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSEnumMemberName::StaticNumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn ts_enum_member_name_numeric_literal<S>(
        self,
        span: Span,
        value: f64,
        raw: S,
        base: NumberBase,
    ) -> TSEnumMemberName<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        TSEnumMemberName::StaticNumericLiteral(
            self.alloc(self.numeric_literal(span, value, raw, base)),
        )
    }

    /// Convert a [`NumericLiteral`] into a [`TSEnumMemberName::StaticNumericLiteral`]
    #[inline]
    pub fn ts_enum_member_name_from_numeric_literal<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, NumericLiteral<'a>>>,
    {
        TSEnumMemberName::StaticNumericLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_enum_member_name_expression(self, inner: Expression<'a>) -> TSEnumMemberName<'a> {
        TSEnumMemberName::from(inner)
    }

    /// Builds a [`TSTypeAnnotation`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// - span: starts at the `:` token and ends at the end of the type annotation
    /// - type_annotation
    #[inline]
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation { span, type_annotation }
    }

    /// Builds a [`TSTypeAnnotation`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// - span: starts at the `:` token and ends at the end of the type annotation
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAnnotation<'a>> {
        Box::new_in(self.ts_type_annotation(span, type_annotation), self.allocator)
    }

    /// Builds a [`TSLiteralType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - literal
    #[inline]
    pub fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSLiteralType<'a> {
        TSLiteralType { span, literal }
    }

    /// Builds a [`TSLiteralType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - literal
    #[inline]
    pub fn alloc_ts_literal_type(
        self,
        span: Span,
        literal: TSLiteral<'a>,
    ) -> Box<'a, TSLiteralType<'a>> {
        Box::new_in(self.ts_literal_type(span, literal), self.allocator)
    }

    /// Build a [`TSLiteral::BooleanLiteral`]
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn ts_literal_boolean_literal(self, span: Span, value: bool) -> TSLiteral<'a> {
        TSLiteral::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    /// Convert a [`BooleanLiteral`] into a [`TSLiteral::BooleanLiteral`]
    #[inline]
    pub fn ts_literal_from_boolean_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, BooleanLiteral>>,
    {
        TSLiteral::BooleanLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::NullLiteral`]
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_literal_null_literal(self, span: Span) -> TSLiteral<'a> {
        TSLiteral::NullLiteral(self.alloc(self.null_literal(span)))
    }

    /// Convert a [`NullLiteral`] into a [`TSLiteral::NullLiteral`]
    #[inline]
    pub fn ts_literal_from_null_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, NullLiteral>>,
    {
        TSLiteral::NullLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::NumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn ts_literal_numeric_literal<S>(
        self,
        span: Span,
        value: f64,
        raw: S,
        base: NumberBase,
    ) -> TSLiteral<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        TSLiteral::NumericLiteral(self.alloc(self.numeric_literal(span, value, raw, base)))
    }

    /// Convert a [`NumericLiteral`] into a [`TSLiteral::NumericLiteral`]
    #[inline]
    pub fn ts_literal_from_numeric_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, NumericLiteral<'a>>>,
    {
        TSLiteral::NumericLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::BigIntLiteral`]
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - raw: The bigint as it appears in the source code
    /// - base: The base representation used by the literal in the source code
    #[inline]
    pub fn ts_literal_big_int_literal<A>(
        self,
        span: Span,
        raw: A,
        base: BigintBase,
    ) -> TSLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSLiteral::BigIntLiteral(self.alloc(self.big_int_literal(span, raw, base)))
    }

    /// Convert a [`BigIntLiteral`] into a [`TSLiteral::BigIntLiteral`]
    #[inline]
    pub fn ts_literal_from_big_int_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, BigIntLiteral<'a>>>,
    {
        TSLiteral::BigIntLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::RegExpLiteral`]
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    /// - regex
    #[inline]
    pub fn ts_literal_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> TSLiteral<'a> {
        TSLiteral::RegExpLiteral(self.alloc(self.reg_exp_literal(span, value, regex)))
    }

    /// Convert a [`RegExpLiteral`] into a [`TSLiteral::RegExpLiteral`]
    #[inline]
    pub fn ts_literal_from_reg_exp_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, RegExpLiteral<'a>>>,
    {
        TSLiteral::RegExpLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn ts_literal_string_literal<A>(self, span: Span, value: A) -> TSLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSLiteral::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    /// Convert a [`StringLiteral`] into a [`TSLiteral::StringLiteral`]
    #[inline]
    pub fn ts_literal_from_string_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        TSLiteral::StringLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::TemplateLiteral`]
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - expressions
    #[inline]
    pub fn ts_literal_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TSLiteral<'a> {
        TSLiteral::TemplateLiteral(self.alloc(self.template_literal(span, quasis, expressions)))
    }

    /// Convert a [`TemplateLiteral`] into a [`TSLiteral::TemplateLiteral`]
    #[inline]
    pub fn ts_literal_from_template_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, TemplateLiteral<'a>>>,
    {
        TSLiteral::TemplateLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSLiteral::UnaryExpression`]
    ///
    /// This node contains a [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - argument
    #[inline]
    pub fn ts_literal_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> TSLiteral<'a> {
        TSLiteral::UnaryExpression(self.alloc(self.unary_expression(span, operator, argument)))
    }

    /// Convert a [`UnaryExpression`] into a [`TSLiteral::UnaryExpression`]
    #[inline]
    pub fn ts_literal_from_unary_expression<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, UnaryExpression<'a>>>,
    {
        TSLiteral::UnaryExpression(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSAnyKeyword`]
    ///
    /// This node contains a [`TSAnyKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_any_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSAnyKeyword(self.alloc(self.ts_any_keyword(span)))
    }

    /// Convert a [`TSAnyKeyword`] into a [`TSType::TSAnyKeyword`]
    #[inline]
    pub fn ts_type_from_ts_any_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSAnyKeyword>>,
    {
        TSType::TSAnyKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSBigIntKeyword`]
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_big_int_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc(self.ts_big_int_keyword(span)))
    }

    /// Convert a [`TSBigIntKeyword`] into a [`TSType::TSBigIntKeyword`]
    #[inline]
    pub fn ts_type_from_ts_big_int_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSBigIntKeyword>>,
    {
        TSType::TSBigIntKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSBooleanKeyword`]
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_boolean_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBooleanKeyword(self.alloc(self.ts_boolean_keyword(span)))
    }

    /// Convert a [`TSBooleanKeyword`] into a [`TSType::TSBooleanKeyword`]
    #[inline]
    pub fn ts_type_from_ts_boolean_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSBooleanKeyword>>,
    {
        TSType::TSBooleanKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSIntrinsicKeyword`]
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_intrinsic_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSIntrinsicKeyword(self.alloc(self.ts_intrinsic_keyword(span)))
    }

    /// Convert a [`TSIntrinsicKeyword`] into a [`TSType::TSIntrinsicKeyword`]
    #[inline]
    pub fn ts_type_from_ts_intrinsic_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIntrinsicKeyword>>,
    {
        TSType::TSIntrinsicKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSNeverKeyword`]
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_never_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNeverKeyword(self.alloc(self.ts_never_keyword(span)))
    }

    /// Convert a [`TSNeverKeyword`] into a [`TSType::TSNeverKeyword`]
    #[inline]
    pub fn ts_type_from_ts_never_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNeverKeyword>>,
    {
        TSType::TSNeverKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSNullKeyword`]
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_null_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNullKeyword(self.alloc(self.ts_null_keyword(span)))
    }

    /// Convert a [`TSNullKeyword`] into a [`TSType::TSNullKeyword`]
    #[inline]
    pub fn ts_type_from_ts_null_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNullKeyword>>,
    {
        TSType::TSNullKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSNumberKeyword`]
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_number_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNumberKeyword(self.alloc(self.ts_number_keyword(span)))
    }

    /// Convert a [`TSNumberKeyword`] into a [`TSType::TSNumberKeyword`]
    #[inline]
    pub fn ts_type_from_ts_number_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNumberKeyword>>,
    {
        TSType::TSNumberKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSObjectKeyword`]
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_object_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSObjectKeyword(self.alloc(self.ts_object_keyword(span)))
    }

    /// Convert a [`TSObjectKeyword`] into a [`TSType::TSObjectKeyword`]
    #[inline]
    pub fn ts_type_from_ts_object_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSObjectKeyword>>,
    {
        TSType::TSObjectKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSStringKeyword`]
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_string_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSStringKeyword(self.alloc(self.ts_string_keyword(span)))
    }

    /// Convert a [`TSStringKeyword`] into a [`TSType::TSStringKeyword`]
    #[inline]
    pub fn ts_type_from_ts_string_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSStringKeyword>>,
    {
        TSType::TSStringKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSSymbolKeyword`]
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_symbol_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSSymbolKeyword(self.alloc(self.ts_symbol_keyword(span)))
    }

    /// Convert a [`TSSymbolKeyword`] into a [`TSType::TSSymbolKeyword`]
    #[inline]
    pub fn ts_type_from_ts_symbol_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSSymbolKeyword>>,
    {
        TSType::TSSymbolKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSUndefinedKeyword`]
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_undefined_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUndefinedKeyword(self.alloc(self.ts_undefined_keyword(span)))
    }

    /// Convert a [`TSUndefinedKeyword`] into a [`TSType::TSUndefinedKeyword`]
    #[inline]
    pub fn ts_type_from_ts_undefined_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUndefinedKeyword>>,
    {
        TSType::TSUndefinedKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSUnknownKeyword`]
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_unknown_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUnknownKeyword(self.alloc(self.ts_unknown_keyword(span)))
    }

    /// Convert a [`TSUnknownKeyword`] into a [`TSType::TSUnknownKeyword`]
    #[inline]
    pub fn ts_type_from_ts_unknown_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUnknownKeyword>>,
    {
        TSType::TSUnknownKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSVoidKeyword`]
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_void_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSVoidKeyword(self.alloc(self.ts_void_keyword(span)))
    }

    /// Convert a [`TSVoidKeyword`] into a [`TSType::TSVoidKeyword`]
    #[inline]
    pub fn ts_type_from_ts_void_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSVoidKeyword>>,
    {
        TSType::TSVoidKeyword(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSArrayType`]
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    #[inline]
    pub fn ts_type_array_type(self, span: Span, element_type: TSType<'a>) -> TSType<'a> {
        TSType::TSArrayType(self.alloc(self.ts_array_type(span, element_type)))
    }

    /// Convert a [`TSArrayType`] into a [`TSType::TSArrayType`]
    #[inline]
    pub fn ts_type_from_ts_array_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSArrayType<'a>>>,
    {
        TSType::TSArrayType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSConditionalType`]
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type
    /// - extends_type
    /// - true_type
    /// - false_type
    #[inline]
    pub fn ts_type_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSConditionalType(self.alloc(self.ts_conditional_type(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
        )))
    }

    /// Convert a [`TSConditionalType`] into a [`TSType::TSConditionalType`]
    #[inline]
    pub fn ts_type_from_ts_conditional_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSConditionalType<'a>>>,
    {
        TSType::TSConditionalType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSConstructorType`]
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_type_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSType::TSConstructorType(self.alloc(self.ts_constructor_type(
            span,
            r#abstract,
            params,
            return_type,
            type_parameters,
        )))
    }

    /// Convert a [`TSConstructorType`] into a [`TSType::TSConstructorType`]
    #[inline]
    pub fn ts_type_from_ts_constructor_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSConstructorType<'a>>>,
    {
        TSType::TSConstructorType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSFunctionType`]
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_type_function_type<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSType::TSFunctionType(self.alloc(self.ts_function_type(
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        )))
    }

    /// Convert a [`TSFunctionType`] into a [`TSType::TSFunctionType`]
    #[inline]
    pub fn ts_type_from_ts_function_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSFunctionType<'a>>>,
    {
        TSType::TSFunctionType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSImportType`]
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_type_import_type<T1>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSImportType(self.alloc(self.ts_import_type(
            span,
            is_type_of,
            parameter,
            qualifier,
            attributes,
            type_parameters,
        )))
    }

    /// Convert a [`TSImportType`] into a [`TSType::TSImportType`]
    #[inline]
    pub fn ts_type_from_ts_import_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSImportType<'a>>>,
    {
        TSType::TSImportType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSIndexedAccessType`]
    ///
    /// This node contains a [`TSIndexedAccessType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object_type
    /// - index_type
    #[inline]
    pub fn ts_type_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSIndexedAccessType(self.alloc(self.ts_indexed_access_type(
            span,
            object_type,
            index_type,
        )))
    }

    /// Convert a [`TSIndexedAccessType`] into a [`TSType::TSIndexedAccessType`]
    #[inline]
    pub fn ts_type_from_ts_indexed_access_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexedAccessType<'a>>>,
    {
        TSType::TSIndexedAccessType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSInferType`]
    ///
    /// This node contains a [`TSInferType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    #[inline]
    pub fn ts_type_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSType::TSInferType(self.alloc(self.ts_infer_type(span, type_parameter)))
    }

    /// Convert a [`TSInferType`] into a [`TSType::TSInferType`]
    #[inline]
    pub fn ts_type_from_ts_infer_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSInferType<'a>>>,
    {
        TSType::TSInferType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSIntersectionType`]
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn ts_type_intersection_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSIntersectionType(self.alloc(self.ts_intersection_type(span, types)))
    }

    /// Convert a [`TSIntersectionType`] into a [`TSType::TSIntersectionType`]
    #[inline]
    pub fn ts_type_from_ts_intersection_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIntersectionType<'a>>>,
    {
        TSType::TSIntersectionType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSLiteralType`]
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - literal
    #[inline]
    pub fn ts_type_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSType<'a> {
        TSType::TSLiteralType(self.alloc(self.ts_literal_type(span, literal)))
    }

    /// Convert a [`TSLiteralType`] into a [`TSType::TSLiteralType`]
    #[inline]
    pub fn ts_type_from_ts_literal_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSLiteralType<'a>>>,
    {
        TSType::TSLiteralType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSMappedType`]
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    /// - name_type
    /// - type_annotation
    /// - optional
    /// - readonly
    #[inline]
    pub fn ts_type_mapped_type<T1>(
        self,
        span: Span,
        type_parameter: T1,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSType::TSMappedType(self.alloc(self.ts_mapped_type(
            span,
            type_parameter,
            name_type,
            type_annotation,
            optional,
            readonly,
        )))
    }

    /// Convert a [`TSMappedType`] into a [`TSType::TSMappedType`]
    #[inline]
    pub fn ts_type_from_ts_mapped_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSMappedType<'a>>>,
    {
        TSType::TSMappedType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSNamedTupleMember`]
    ///
    /// This node contains a [`TSNamedTupleMember`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    /// - label
    /// - optional
    #[inline]
    pub fn ts_type_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> TSType<'a> {
        TSType::TSNamedTupleMember(self.alloc(self.ts_named_tuple_member(
            span,
            element_type,
            label,
            optional,
        )))
    }

    /// Convert a [`TSNamedTupleMember`] into a [`TSType::TSNamedTupleMember`]
    #[inline]
    pub fn ts_type_from_ts_named_tuple_member<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNamedTupleMember<'a>>>,
    {
        TSType::TSNamedTupleMember(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSQualifiedName`]
    ///
    /// This node contains a [`TSQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn ts_type_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSType<'a> {
        TSType::TSQualifiedName(self.alloc(self.ts_qualified_name(span, left, right)))
    }

    /// Convert a [`TSQualifiedName`] into a [`TSType::TSQualifiedName`]
    #[inline]
    pub fn ts_type_from_ts_qualified_name<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSQualifiedName<'a>>>,
    {
        TSType::TSQualifiedName(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTemplateLiteralType`]
    ///
    /// This node contains a [`TSTemplateLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - types
    #[inline]
    pub fn ts_type_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSType<'a> {
        TSType::TSTemplateLiteralType(
            self.alloc(self.ts_template_literal_type(span, quasis, types)),
        )
    }

    /// Convert a [`TSTemplateLiteralType`] into a [`TSType::TSTemplateLiteralType`]
    #[inline]
    pub fn ts_type_from_ts_template_literal_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTemplateLiteralType<'a>>>,
    {
        TSType::TSTemplateLiteralType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSThisType`]
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_this_type(self, span: Span) -> TSType<'a> {
        TSType::TSThisType(self.alloc(self.ts_this_type(span)))
    }

    /// Convert a [`TSThisType`] into a [`TSType::TSThisType`]
    #[inline]
    pub fn ts_type_from_ts_this_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSThisType>>,
    {
        TSType::TSThisType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTupleType`]
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_types
    #[inline]
    pub fn ts_type_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSType<'a> {
        TSType::TSTupleType(self.alloc(self.ts_tuple_type(span, element_types)))
    }

    /// Convert a [`TSTupleType`] into a [`TSType::TSTupleType`]
    #[inline]
    pub fn ts_type_from_ts_tuple_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTupleType<'a>>>,
    {
        TSType::TSTupleType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTypeLiteral`]
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - members
    #[inline]
    pub fn ts_type_type_literal(self, span: Span, members: Vec<'a, TSSignature<'a>>) -> TSType<'a> {
        TSType::TSTypeLiteral(self.alloc(self.ts_type_literal(span, members)))
    }

    /// Convert a [`TSTypeLiteral`] into a [`TSType::TSTypeLiteral`]
    #[inline]
    pub fn ts_type_from_ts_type_literal<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeLiteral<'a>>>,
    {
        TSType::TSTypeLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTypeOperatorType`]
    ///
    /// This node contains a [`TSTypeOperator`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation
    #[inline]
    pub fn ts_type_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSTypeOperatorType(self.alloc(self.ts_type_operator(
            span,
            operator,
            type_annotation,
        )))
    }

    /// Convert a [`TSTypeOperator`] into a [`TSType::TSTypeOperatorType`]
    #[inline]
    pub fn ts_type_from_ts_type_operator<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeOperator<'a>>>,
    {
        TSType::TSTypeOperatorType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTypePredicate`]
    ///
    /// This node contains a [`TSTypePredicate`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name
    /// - asserts
    /// - type_annotation
    #[inline]
    pub fn ts_type_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSType::TSTypePredicate(self.alloc(self.ts_type_predicate(
            span,
            parameter_name,
            asserts,
            type_annotation,
        )))
    }

    /// Convert a [`TSTypePredicate`] into a [`TSType::TSTypePredicate`]
    #[inline]
    pub fn ts_type_from_ts_type_predicate<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypePredicate<'a>>>,
    {
        TSType::TSTypePredicate(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTypeQuery`]
    ///
    /// This node contains a [`TSTypeQuery`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expr_name
    /// - type_parameters
    #[inline]
    pub fn ts_type_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSTypeQuery(self.alloc(self.ts_type_query(span, expr_name, type_parameters)))
    }

    /// Convert a [`TSTypeQuery`] into a [`TSType::TSTypeQuery`]
    #[inline]
    pub fn ts_type_from_ts_type_query<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeQuery<'a>>>,
    {
        TSType::TSTypeQuery(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSTypeReference`]
    ///
    /// This node contains a [`TSTypeReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_name
    /// - type_parameters
    #[inline]
    pub fn ts_type_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSTypeReference(self.alloc(self.ts_type_reference(
            span,
            type_name,
            type_parameters,
        )))
    }

    /// Convert a [`TSTypeReference`] into a [`TSType::TSTypeReference`]
    #[inline]
    pub fn ts_type_from_ts_type_reference<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeReference<'a>>>,
    {
        TSType::TSTypeReference(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSUnionType`]
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn ts_type_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc(self.ts_union_type(span, types)))
    }

    /// Convert a [`TSUnionType`] into a [`TSType::TSUnionType`]
    #[inline]
    pub fn ts_type_from_ts_union_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUnionType<'a>>>,
    {
        TSType::TSUnionType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::TSParenthesizedType`]
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_type_parenthesized_type(self, span: Span, type_annotation: TSType<'a>) -> TSType<'a> {
        TSType::TSParenthesizedType(self.alloc(self.ts_parenthesized_type(span, type_annotation)))
    }

    /// Convert a [`TSParenthesizedType`] into a [`TSType::TSParenthesizedType`]
    #[inline]
    pub fn ts_type_from_ts_parenthesized_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSParenthesizedType<'a>>>,
    {
        TSType::TSParenthesizedType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::JSDocNullableType`]
    ///
    /// This node contains a [`JSDocNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix: Was `?` after the type annotation?
    #[inline]
    pub fn ts_type_js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNullableType(self.alloc(self.js_doc_nullable_type(
            span,
            type_annotation,
            postfix,
        )))
    }

    /// Convert a [`JSDocNullableType`] into a [`TSType::JSDocNullableType`]
    #[inline]
    pub fn ts_type_from_js_doc_nullable_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocNullableType<'a>>>,
    {
        TSType::JSDocNullableType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::JSDocNonNullableType`]
    ///
    /// This node contains a [`JSDocNonNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix
    #[inline]
    pub fn ts_type_js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNonNullableType(self.alloc(self.js_doc_non_nullable_type(
            span,
            type_annotation,
            postfix,
        )))
    }

    /// Convert a [`JSDocNonNullableType`] into a [`TSType::JSDocNonNullableType`]
    #[inline]
    pub fn ts_type_from_js_doc_non_nullable_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocNonNullableType<'a>>>,
    {
        TSType::JSDocNonNullableType(inner.into_in(self.allocator))
    }

    /// Build a [`TSType::JSDocUnknownType`]
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_js_doc_unknown_type(self, span: Span) -> TSType<'a> {
        TSType::JSDocUnknownType(self.alloc(self.js_doc_unknown_type(span)))
    }

    /// Convert a [`JSDocUnknownType`] into a [`TSType::JSDocUnknownType`]
    #[inline]
    pub fn ts_type_from_js_doc_unknown_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocUnknownType>>,
    {
        TSType::JSDocUnknownType(inner.into_in(self.allocator))
    }

    /// Builds a [`TSConditionalType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type
    /// - extends_type
    /// - true_type
    /// - false_type
    #[inline]
    pub fn ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSConditionalType<'a> {
        TSConditionalType {
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSConditionalType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type
    /// - extends_type
    /// - true_type
    /// - false_type
    #[inline]
    pub fn alloc_ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> Box<'a, TSConditionalType<'a>> {
        Box::new_in(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type),
            self.allocator,
        )
    }

    /// Builds a [`TSUnionType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_union_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSUnionType<'a> {
        TSUnionType { span, types }
    }

    /// Builds a [`TSUnionType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_union_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn alloc_ts_union_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSUnionType<'a>> {
        Box::new_in(self.ts_union_type(span, types), self.allocator)
    }

    /// Builds a [`TSIntersectionType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_intersection_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> TSIntersectionType<'a> {
        TSIntersectionType { span, types }
    }

    /// Builds a [`TSIntersectionType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_intersection_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types
    #[inline]
    pub fn alloc_ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSIntersectionType<'a>> {
        Box::new_in(self.ts_intersection_type(span, types), self.allocator)
    }

    /// Builds a [`TSParenthesizedType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_parenthesized_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSParenthesizedType<'a> {
        TSParenthesizedType { span, type_annotation }
    }

    /// Builds a [`TSParenthesizedType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_parenthesized_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSParenthesizedType<'a>> {
        Box::new_in(self.ts_parenthesized_type(span, type_annotation), self.allocator)
    }

    /// Builds a [`TSTypeOperator`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation
    #[inline]
    pub fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSTypeOperator<'a> {
        TSTypeOperator { span, operator, type_annotation }
    }

    /// Builds a [`TSTypeOperator`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeOperator<'a>> {
        Box::new_in(self.ts_type_operator(span, operator, type_annotation), self.allocator)
    }

    /// Builds a [`TSArrayType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_array_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    #[inline]
    pub fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> TSArrayType<'a> {
        TSArrayType { span, element_type }
    }

    /// Builds a [`TSArrayType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_array_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    #[inline]
    pub fn alloc_ts_array_type(
        self,
        span: Span,
        element_type: TSType<'a>,
    ) -> Box<'a, TSArrayType<'a>> {
        Box::new_in(self.ts_array_type(span, element_type), self.allocator)
    }

    /// Builds a [`TSIndexedAccessType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_indexed_access_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object_type
    /// - index_type
    #[inline]
    pub fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSIndexedAccessType<'a> {
        TSIndexedAccessType { span, object_type, index_type }
    }

    /// Builds a [`TSIndexedAccessType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_indexed_access_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object_type
    /// - index_type
    #[inline]
    pub fn alloc_ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> Box<'a, TSIndexedAccessType<'a>> {
        Box::new_in(self.ts_indexed_access_type(span, object_type, index_type), self.allocator)
    }

    /// Builds a [`TSTupleType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_tuple_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_types
    #[inline]
    pub fn ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSTupleType<'a> {
        TSTupleType { span, element_types }
    }

    /// Builds a [`TSTupleType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_tuple_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_types
    #[inline]
    pub fn alloc_ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> Box<'a, TSTupleType<'a>> {
        Box::new_in(self.ts_tuple_type(span, element_types), self.allocator)
    }

    /// Builds a [`TSNamedTupleMember`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_named_tuple_member`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    /// - label
    /// - optional
    #[inline]
    pub fn ts_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> TSNamedTupleMember<'a> {
        TSNamedTupleMember { span, element_type, label, optional }
    }

    /// Builds a [`TSNamedTupleMember`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_named_tuple_member`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - element_type
    /// - label
    /// - optional
    #[inline]
    pub fn alloc_ts_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, TSNamedTupleMember<'a>> {
        Box::new_in(self.ts_named_tuple_member(span, element_type, label, optional), self.allocator)
    }

    /// Builds a [`TSOptionalType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_optional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_optional_type(self, span: Span, type_annotation: TSType<'a>) -> TSOptionalType<'a> {
        TSOptionalType { span, type_annotation }
    }

    /// Builds a [`TSOptionalType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_optional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSOptionalType<'a>> {
        Box::new_in(self.ts_optional_type(span, type_annotation), self.allocator)
    }

    /// Builds a [`TSRestType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_rest_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_rest_type(self, span: Span, type_annotation: TSType<'a>) -> TSRestType<'a> {
        TSRestType { span, type_annotation }
    }

    /// Builds a [`TSRestType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_rest_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSRestType<'a>> {
        Box::new_in(self.ts_rest_type(span, type_annotation), self.allocator)
    }

    /// Build a [`TSTupleElement::TSOptionalType`]
    ///
    /// This node contains a [`TSOptionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_tuple_element_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSOptionalType(self.alloc(self.ts_optional_type(span, type_annotation)))
    }

    /// Convert a [`TSOptionalType`] into a [`TSTupleElement::TSOptionalType`]
    #[inline]
    pub fn ts_tuple_element_from_ts_optional_type<T>(self, inner: T) -> TSTupleElement<'a>
    where
        T: IntoIn<'a, Box<'a, TSOptionalType<'a>>>,
    {
        TSTupleElement::TSOptionalType(inner.into_in(self.allocator))
    }

    /// Build a [`TSTupleElement::TSRestType`]
    ///
    /// This node contains a [`TSRestType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    #[inline]
    pub fn ts_tuple_element_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSRestType(self.alloc(self.ts_rest_type(span, type_annotation)))
    }

    /// Convert a [`TSRestType`] into a [`TSTupleElement::TSRestType`]
    #[inline]
    pub fn ts_tuple_element_from_ts_rest_type<T>(self, inner: T) -> TSTupleElement<'a>
    where
        T: IntoIn<'a, Box<'a, TSRestType<'a>>>,
    {
        TSTupleElement::TSRestType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_tuple_element_type(self, inner: TSType<'a>) -> TSTupleElement<'a> {
        TSTupleElement::from(inner)
    }

    /// Builds a [`TSAnyKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_any_keyword(self, span: Span) -> TSAnyKeyword {
        TSAnyKeyword { span }
    }

    /// Builds a [`TSAnyKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_any_keyword(self, span: Span) -> Box<'a, TSAnyKeyword> {
        Box::new_in(self.ts_any_keyword(span), self.allocator)
    }

    /// Builds a [`TSStringKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_string_keyword(self, span: Span) -> TSStringKeyword {
        TSStringKeyword { span }
    }

    /// Builds a [`TSStringKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_string_keyword(self, span: Span) -> Box<'a, TSStringKeyword> {
        Box::new_in(self.ts_string_keyword(span), self.allocator)
    }

    /// Builds a [`TSBooleanKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_boolean_keyword(self, span: Span) -> TSBooleanKeyword {
        TSBooleanKeyword { span }
    }

    /// Builds a [`TSBooleanKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_boolean_keyword(self, span: Span) -> Box<'a, TSBooleanKeyword> {
        Box::new_in(self.ts_boolean_keyword(span), self.allocator)
    }

    /// Builds a [`TSNumberKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_number_keyword(self, span: Span) -> TSNumberKeyword {
        TSNumberKeyword { span }
    }

    /// Builds a [`TSNumberKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_number_keyword(self, span: Span) -> Box<'a, TSNumberKeyword> {
        Box::new_in(self.ts_number_keyword(span), self.allocator)
    }

    /// Builds a [`TSNeverKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_never_keyword(self, span: Span) -> TSNeverKeyword {
        TSNeverKeyword { span }
    }

    /// Builds a [`TSNeverKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_never_keyword(self, span: Span) -> Box<'a, TSNeverKeyword> {
        Box::new_in(self.ts_never_keyword(span), self.allocator)
    }

    /// Builds a [`TSIntrinsicKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_intrinsic_keyword(self, span: Span) -> TSIntrinsicKeyword {
        TSIntrinsicKeyword { span }
    }

    /// Builds a [`TSIntrinsicKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_intrinsic_keyword(self, span: Span) -> Box<'a, TSIntrinsicKeyword> {
        Box::new_in(self.ts_intrinsic_keyword(span), self.allocator)
    }

    /// Builds a [`TSUnknownKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_unknown_keyword(self, span: Span) -> TSUnknownKeyword {
        TSUnknownKeyword { span }
    }

    /// Builds a [`TSUnknownKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_unknown_keyword(self, span: Span) -> Box<'a, TSUnknownKeyword> {
        Box::new_in(self.ts_unknown_keyword(span), self.allocator)
    }

    /// Builds a [`TSNullKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_null_keyword(self, span: Span) -> TSNullKeyword {
        TSNullKeyword { span }
    }

    /// Builds a [`TSNullKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_null_keyword(self, span: Span) -> Box<'a, TSNullKeyword> {
        Box::new_in(self.ts_null_keyword(span), self.allocator)
    }

    /// Builds a [`TSUndefinedKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_undefined_keyword(self, span: Span) -> TSUndefinedKeyword {
        TSUndefinedKeyword { span }
    }

    /// Builds a [`TSUndefinedKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_undefined_keyword(self, span: Span) -> Box<'a, TSUndefinedKeyword> {
        Box::new_in(self.ts_undefined_keyword(span), self.allocator)
    }

    /// Builds a [`TSVoidKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_void_keyword(self, span: Span) -> TSVoidKeyword {
        TSVoidKeyword { span }
    }

    /// Builds a [`TSVoidKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_void_keyword(self, span: Span) -> Box<'a, TSVoidKeyword> {
        Box::new_in(self.ts_void_keyword(span), self.allocator)
    }

    /// Builds a [`TSSymbolKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_symbol_keyword(self, span: Span) -> TSSymbolKeyword {
        TSSymbolKeyword { span }
    }

    /// Builds a [`TSSymbolKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_symbol_keyword(self, span: Span) -> Box<'a, TSSymbolKeyword> {
        Box::new_in(self.ts_symbol_keyword(span), self.allocator)
    }

    /// Builds a [`TSThisType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_this_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_this_type(self, span: Span) -> TSThisType {
        TSThisType { span }
    }

    /// Builds a [`TSThisType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_this_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_this_type(self, span: Span) -> Box<'a, TSThisType> {
        Box::new_in(self.ts_this_type(span), self.allocator)
    }

    /// Builds a [`TSObjectKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_object_keyword(self, span: Span) -> TSObjectKeyword {
        TSObjectKeyword { span }
    }

    /// Builds a [`TSObjectKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_object_keyword(self, span: Span) -> Box<'a, TSObjectKeyword> {
        Box::new_in(self.ts_object_keyword(span), self.allocator)
    }

    /// Builds a [`TSBigIntKeyword`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_big_int_keyword(self, span: Span) -> TSBigIntKeyword {
        TSBigIntKeyword { span }
    }

    /// Builds a [`TSBigIntKeyword`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_big_int_keyword(self, span: Span) -> Box<'a, TSBigIntKeyword> {
        Box::new_in(self.ts_big_int_keyword(span), self.allocator)
    }

    /// Builds a [`TSTypeReference`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_name
    /// - type_parameters
    #[inline]
    pub fn ts_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: T1,
    ) -> TSTypeReference<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeReference {
            span,
            type_name,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSTypeReference`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_name
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_parameters: T1,
    ) -> Box<'a, TSTypeReference<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(self.ts_type_reference(span, type_name, type_parameters), self.allocator)
    }

    /// Build a [`TSTypeName::IdentifierReference`]
    ///
    /// This node contains a [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn ts_type_name_identifier_reference<A>(self, span: Span, name: A) -> TSTypeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSTypeName::IdentifierReference(self.alloc(self.identifier_reference(span, name)))
    }

    /// Convert a [`IdentifierReference`] into a [`TSTypeName::IdentifierReference`]
    #[inline]
    pub fn ts_type_name_from_identifier_reference<T>(self, inner: T) -> TSTypeName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierReference<'a>>>,
    {
        TSTypeName::IdentifierReference(inner.into_in(self.allocator))
    }

    /// Build a [`TSTypeName::QualifiedName`]
    ///
    /// This node contains a [`TSQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn ts_type_name_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSTypeName<'a> {
        TSTypeName::QualifiedName(self.alloc(self.ts_qualified_name(span, left, right)))
    }

    /// Convert a [`TSQualifiedName`] into a [`TSTypeName::QualifiedName`]
    #[inline]
    pub fn ts_type_name_from_ts_qualified_name<T>(self, inner: T) -> TSTypeName<'a>
    where
        T: IntoIn<'a, Box<'a, TSQualifiedName<'a>>>,
    {
        TSTypeName::QualifiedName(inner.into_in(self.allocator))
    }

    /// Builds a [`TSQualifiedName`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_qualified_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSQualifiedName<'a> {
        TSQualifiedName { span, left, right }
    }

    /// Builds a [`TSQualifiedName`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_qualified_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    #[inline]
    pub fn alloc_ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> Box<'a, TSQualifiedName<'a>> {
        Box::new_in(self.ts_qualified_name(span, left, right), self.allocator)
    }

    /// Builds a [`TSTypeParameterInstantiation`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_parameter_instantiation`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    #[inline]
    pub fn ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> TSTypeParameterInstantiation<'a> {
        TSTypeParameterInstantiation { span, params }
    }

    /// Builds a [`TSTypeParameterInstantiation`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter_instantiation`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    #[inline]
    pub fn alloc_ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTypeParameterInstantiation<'a>> {
        Box::new_in(self.ts_type_parameter_instantiation(span, params), self.allocator)
    }

    /// Builds a [`TSTypeParameter`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - constraint
    /// - default
    /// - r#in
    /// - out
    /// - r#const
    #[inline]
    pub fn ts_type_parameter(
        self,
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> TSTypeParameter<'a> {
        TSTypeParameter { span, name, constraint, default, r#in, out, r#const }
    }

    /// Builds a [`TSTypeParameter`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - constraint
    /// - default
    /// - r#in
    /// - out
    /// - r#const
    #[inline]
    pub fn alloc_ts_type_parameter(
        self,
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> Box<'a, TSTypeParameter<'a>> {
        Box::new_in(
            self.ts_type_parameter(span, name, constraint, default, r#in, out, r#const),
            self.allocator,
        )
    }

    /// Builds a [`TSTypeParameterDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_parameter_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    #[inline]
    pub fn ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> TSTypeParameterDeclaration<'a> {
        TSTypeParameterDeclaration { span, params }
    }

    /// Builds a [`TSTypeParameterDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    #[inline]
    pub fn alloc_ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> Box<'a, TSTypeParameterDeclaration<'a>> {
        Box::new_in(self.ts_type_parameter_declaration(span, params), self.allocator)
    }

    /// Builds a [`TSTypeAliasDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - type_parameters
    /// - type_annotation
    /// - declare
    #[inline]
    pub fn ts_type_alias_declaration<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> TSTypeAliasDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSTypeAliasDeclaration {
            span,
            id,
            type_parameters: type_parameters.into_in(self.allocator),
            type_annotation,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSTypeAliasDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - type_parameters
    /// - type_annotation
    /// - declare
    #[inline]
    pub fn alloc_ts_type_alias_declaration<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> Box<'a, TSTypeAliasDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare),
            self.allocator,
        )
    }

    /// Builds a [`TSClassImplements`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_class_implements`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn ts_class_implements<T1>(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_parameters: T1,
    ) -> TSClassImplements<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSClassImplements {
            span,
            expression,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSClassImplements`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_class_implements`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_class_implements<T1>(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_parameters: T1,
    ) -> Box<'a, TSClassImplements<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(self.ts_class_implements(span, expression, type_parameters), self.allocator)
    }

    /// Builds a [`TSInterfaceDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends
    /// - type_parameters
    /// - body
    /// - declare
    #[inline]
    pub fn ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
    ) -> TSInterfaceDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        TSInterfaceDeclaration {
            span,
            id,
            extends,
            type_parameters: type_parameters.into_in(self.allocator),
            body: body.into_in(self.allocator),
            declare,
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSInterfaceDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends
    /// - type_parameters
    /// - body
    /// - declare
    #[inline]
    pub fn alloc_ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
    ) -> Box<'a, TSInterfaceDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Box::new_in(
            self.ts_interface_declaration(span, id, extends, type_parameters, body, declare),
            self.allocator,
        )
    }

    /// Builds a [`TSInterfaceBody`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_interface_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> TSInterfaceBody<'a> {
        TSInterfaceBody { span, body }
    }

    /// Builds a [`TSInterfaceBody`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_interface_body`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    #[inline]
    pub fn alloc_ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSInterfaceBody<'a>> {
        Box::new_in(self.ts_interface_body(span, body), self.allocator)
    }

    /// Builds a [`TSPropertySignature`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_property_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - computed
    /// - optional
    /// - readonly
    /// - key
    /// - type_annotation
    #[inline]
    pub fn ts_property_signature<T1>(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
    ) -> TSPropertySignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSPropertySignature {
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation: type_annotation.into_in(self.allocator),
        }
    }

    /// Builds a [`TSPropertySignature`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_property_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - computed
    /// - optional
    /// - readonly
    /// - key
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_property_signature<T1>(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
    ) -> Box<'a, TSPropertySignature<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_property_signature(span, computed, optional, readonly, key, type_annotation),
            self.allocator,
        )
    }

    /// Build a [`TSSignature::TSIndexSignature`]
    ///
    /// This node contains a [`TSIndexSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    #[inline]
    pub fn ts_signature_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSSignature::TSIndexSignature(self.alloc(self.ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
        )))
    }

    /// Convert a [`TSIndexSignature`] into a [`TSSignature::TSIndexSignature`]
    #[inline]
    pub fn ts_signature_from_ts_index_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexSignature<'a>>>,
    {
        TSSignature::TSIndexSignature(inner.into_in(self.allocator))
    }

    /// Build a [`TSSignature::TSPropertySignature`]
    ///
    /// This node contains a [`TSPropertySignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - computed
    /// - optional
    /// - readonly
    /// - key
    /// - type_annotation
    #[inline]
    pub fn ts_signature_property_signature<T1>(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSPropertySignature(self.alloc(self.ts_property_signature(
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
        )))
    }

    /// Convert a [`TSPropertySignature`] into a [`TSSignature::TSPropertySignature`]
    #[inline]
    pub fn ts_signature_from_ts_property_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSPropertySignature<'a>>>,
    {
        TSSignature::TSPropertySignature(inner.into_in(self.allocator))
    }

    /// Build a [`TSSignature::TSCallSignatureDeclaration`]
    ///
    /// This node contains a [`TSCallSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_signature_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSSignature::TSCallSignatureDeclaration(self.alloc(self.ts_call_signature_declaration(
            span,
            this_param,
            params,
            return_type,
            type_parameters,
        )))
    }

    /// Convert a [`TSCallSignatureDeclaration`] into a [`TSSignature::TSCallSignatureDeclaration`]
    #[inline]
    pub fn ts_signature_from_ts_call_signature_declaration<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSCallSignatureDeclaration<'a>>>,
    {
        TSSignature::TSCallSignatureDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`TSSignature::TSConstructSignatureDeclaration`]
    ///
    /// This node contains a [`TSConstructSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_signature_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSSignature::TSConstructSignatureDeclaration(self.alloc(
            self.ts_construct_signature_declaration(span, params, return_type, type_parameters),
        ))
    }

    /// Convert a [`TSConstructSignatureDeclaration`] into a [`TSSignature::TSConstructSignatureDeclaration`]
    #[inline]
    pub fn ts_signature_from_ts_construct_signature_declaration<T>(
        self,
        inner: T,
    ) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSConstructSignatureDeclaration<'a>>>,
    {
        TSSignature::TSConstructSignatureDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`TSSignature::TSMethodSignature`]
    ///
    /// This node contains a [`TSMethodSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_signature_method_signature<T1, T2, T3>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSSignature::TSMethodSignature(self.alloc(self.ts_method_signature(
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params,
            return_type,
            type_parameters,
        )))
    }

    /// Convert a [`TSMethodSignature`] into a [`TSSignature::TSMethodSignature`]
    #[inline]
    pub fn ts_signature_from_ts_method_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSMethodSignature<'a>>>,
    {
        TSSignature::TSMethodSignature(inner.into_in(self.allocator))
    }

    /// Builds a [`TSIndexSignature`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    #[inline]
    pub fn ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
    ) -> TSIndexSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSIndexSignature {
            span,
            parameters,
            type_annotation: type_annotation.into_in(self.allocator),
            readonly,
        }
    }

    /// Builds a [`TSIndexSignature`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    #[inline]
    pub fn alloc_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
    ) -> Box<'a, TSIndexSignature<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            self.ts_index_signature(span, parameters, type_annotation, readonly),
            self.allocator,
        )
    }

    /// Builds a [`TSCallSignatureDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSCallSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSCallSignatureDeclaration {
            span,
            this_param,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSCallSignatureDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> Box<'a, TSCallSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_call_signature_declaration(
                span,
                this_param,
                params,
                return_type,
                type_parameters,
            ),
            self.allocator,
        )
    }

    /// Builds a [`TSMethodSignature`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_method_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_method_signature<T1, T2, T3>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSMethodSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            this_param,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSMethodSignature`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_method_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_method_signature<T1, T2, T3>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> Box<'a, TSMethodSignature<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_method_signature(
                span,
                key,
                computed,
                optional,
                kind,
                this_param,
                params,
                return_type,
                type_parameters,
            ),
            self.allocator,
        )
    }

    /// Builds a [`TSConstructSignatureDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSConstructSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSConstructSignatureDeclaration {
            span,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSConstructSignatureDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> Box<'a, TSConstructSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_construct_signature_declaration(span, params, return_type, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`TSIndexSignatureName`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_index_signature_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - type_annotation
    #[inline]
    pub fn ts_index_signature_name<A, T1>(
        self,
        span: Span,
        name: A,
        type_annotation: T1,
    ) -> TSIndexSignatureName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSIndexSignatureName {
            span,
            name: name.into_in(self.allocator),
            type_annotation: type_annotation.into_in(self.allocator),
        }
    }

    /// Builds a [`TSIndexSignatureName`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_index_signature_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_index_signature_name<A, T1>(
        self,
        span: Span,
        name: A,
        type_annotation: T1,
    ) -> Box<'a, TSIndexSignatureName<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(self.ts_index_signature_name(span, name, type_annotation), self.allocator)
    }

    /// Builds a [`TSInterfaceHeritage`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_interface_heritage`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn ts_interface_heritage<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> TSInterfaceHeritage<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSInterfaceHeritage {
            span,
            expression,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSInterfaceHeritage`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_interface_heritage`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_interface_heritage<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> Box<'a, TSInterfaceHeritage<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(self.ts_interface_heritage(span, expression, type_parameters), self.allocator)
    }

    /// Builds a [`TSTypePredicate`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name
    /// - asserts
    /// - type_annotation
    #[inline]
    pub fn ts_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> TSTypePredicate<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSTypePredicate {
            span,
            parameter_name,
            asserts,
            type_annotation: type_annotation.into_in(self.allocator),
        }
    }

    /// Builds a [`TSTypePredicate`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name
    /// - asserts
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> Box<'a, TSTypePredicate<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_type_predicate(span, parameter_name, asserts, type_annotation),
            self.allocator,
        )
    }

    /// Build a [`TSTypePredicateName::Identifier`]
    ///
    /// This node contains a [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn ts_type_predicate_name_identifier_name<A>(
        self,
        span: Span,
        name: A,
    ) -> TSTypePredicateName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSTypePredicateName::Identifier(self.alloc(self.identifier_name(span, name)))
    }

    /// Convert a [`IdentifierName`] into a [`TSTypePredicateName::Identifier`]
    #[inline]
    pub fn ts_type_predicate_name_from_identifier_name<T>(self, inner: T) -> TSTypePredicateName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        TSTypePredicateName::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`TSTypePredicateName::This`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_predicate_name_this_type(self, span: Span) -> TSTypePredicateName<'a> {
        TSTypePredicateName::This(self.ts_this_type(span))
    }

    /// Convert a [`TSThisType`] into a [`TSTypePredicateName::This`]
    #[inline]
    pub fn ts_type_predicate_name_from_ts_this_type<T>(self, inner: T) -> TSTypePredicateName<'a>
    where
        T: IntoIn<'a, TSThisType>,
    {
        TSTypePredicateName::This(inner.into_in(self.allocator))
    }

    /// Builds a [`TSModuleDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - body
    /// - kind: The keyword used to define this module declaration
    /// - declare
    #[inline]
    pub fn ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> TSModuleDeclaration<'a> {
        TSModuleDeclaration { span, id, body, kind, declare, scope_id: Default::default() }
    }

    /// Builds a [`TSModuleDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - body
    /// - kind: The keyword used to define this module declaration
    /// - declare
    #[inline]
    pub fn alloc_ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        Box::new_in(self.ts_module_declaration(span, id, body, kind, declare), self.allocator)
    }

    /// Build a [`TSModuleDeclarationName::Identifier`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn ts_module_declaration_name_identifier_name<A>(
        self,
        span: Span,
        name: A,
    ) -> TSModuleDeclarationName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSModuleDeclarationName::Identifier(self.identifier_name(span, name))
    }

    /// Convert a [`IdentifierName`] into a [`TSModuleDeclarationName::Identifier`]
    #[inline]
    pub fn ts_module_declaration_name_from_identifier_name<T>(
        self,
        inner: T,
    ) -> TSModuleDeclarationName<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        TSModuleDeclarationName::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn ts_module_declaration_name_string_literal<A>(
        self,
        span: Span,
        value: A,
    ) -> TSModuleDeclarationName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSModuleDeclarationName::StringLiteral(self.string_literal(span, value))
    }

    /// Convert a [`StringLiteral`] into a [`TSModuleDeclarationName::StringLiteral`]
    #[inline]
    pub fn ts_module_declaration_name_from_string_literal<T>(
        self,
        inner: T,
    ) -> TSModuleDeclarationName<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        TSModuleDeclarationName::StringLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`TSModuleDeclarationBody::TSModuleDeclaration`]
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - body
    /// - kind: The keyword used to define this module declaration
    /// - declare
    #[inline]
    pub fn ts_module_declaration_body_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> TSModuleDeclarationBody<'a> {
        TSModuleDeclarationBody::TSModuleDeclaration(
            self.alloc(self.ts_module_declaration(span, id, body, kind, declare)),
        )
    }

    /// Convert a [`TSModuleDeclaration`] into a [`TSModuleDeclarationBody::TSModuleDeclaration`]
    #[inline]
    pub fn ts_module_declaration_body_from_ts_module_declaration<T>(
        self,
        inner: T,
    ) -> TSModuleDeclarationBody<'a>
    where
        T: IntoIn<'a, Box<'a, TSModuleDeclaration<'a>>>,
    {
        TSModuleDeclarationBody::TSModuleDeclaration(inner.into_in(self.allocator))
    }

    /// Build a [`TSModuleDeclarationBody::TSModuleBlock`]
    ///
    /// This node contains a [`TSModuleBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - directives
    /// - body
    #[inline]
    pub fn ts_module_declaration_body_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> TSModuleDeclarationBody<'a> {
        TSModuleDeclarationBody::TSModuleBlock(
            self.alloc(self.ts_module_block(span, directives, body)),
        )
    }

    /// Convert a [`TSModuleBlock`] into a [`TSModuleDeclarationBody::TSModuleBlock`]
    #[inline]
    pub fn ts_module_declaration_body_from_ts_module_block<T>(
        self,
        inner: T,
    ) -> TSModuleDeclarationBody<'a>
    where
        T: IntoIn<'a, Box<'a, TSModuleBlock<'a>>>,
    {
        TSModuleDeclarationBody::TSModuleBlock(inner.into_in(self.allocator))
    }

    /// Builds a [`TSModuleBlock`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_module_block`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - directives
    /// - body
    #[inline]
    pub fn ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> TSModuleBlock<'a> {
        TSModuleBlock { span, directives, body }
    }

    /// Builds a [`TSModuleBlock`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_module_block`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - directives
    /// - body
    #[inline]
    pub fn alloc_ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        Box::new_in(self.ts_module_block(span, directives, body), self.allocator)
    }

    /// Builds a [`TSTypeLiteral`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - members
    #[inline]
    pub fn ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> TSTypeLiteral<'a> {
        TSTypeLiteral { span, members }
    }

    /// Builds a [`TSTypeLiteral`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_literal`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - members
    #[inline]
    pub fn alloc_ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSTypeLiteral<'a>> {
        Box::new_in(self.ts_type_literal(span, members), self.allocator)
    }

    /// Builds a [`TSInferType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    #[inline]
    pub fn ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSInferType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSInferType { span, type_parameter: type_parameter.into_in(self.allocator) }
    }

    /// Builds a [`TSInferType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    #[inline]
    pub fn alloc_ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> Box<'a, TSInferType<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Box::new_in(self.ts_infer_type(span, type_parameter), self.allocator)
    }

    /// Builds a [`TSTypeQuery`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_query`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expr_name
    /// - type_parameters
    #[inline]
    pub fn ts_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: T1,
    ) -> TSTypeQuery<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeQuery { span, expr_name, type_parameters: type_parameters.into_in(self.allocator) }
    }

    /// Builds a [`TSTypeQuery`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_query`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expr_name
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_parameters: T1,
    ) -> Box<'a, TSTypeQuery<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(self.ts_type_query(span, expr_name, type_parameters), self.allocator)
    }

    /// Build a [`TSTypeQueryExprName::TSImportType`]
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_type_query_expr_name_import_type<T1>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: T1,
    ) -> TSTypeQueryExprName<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeQueryExprName::TSImportType(self.alloc(self.ts_import_type(
            span,
            is_type_of,
            parameter,
            qualifier,
            attributes,
            type_parameters,
        )))
    }

    /// Convert a [`TSImportType`] into a [`TSTypeQueryExprName::TSImportType`]
    #[inline]
    pub fn ts_type_query_expr_name_from_ts_import_type<T>(self, inner: T) -> TSTypeQueryExprName<'a>
    where
        T: IntoIn<'a, Box<'a, TSImportType<'a>>>,
    {
        TSTypeQueryExprName::TSImportType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_query_expr_name_type_name(
        self,
        inner: TSTypeName<'a>,
    ) -> TSTypeQueryExprName<'a> {
        TSTypeQueryExprName::from(inner)
    }

    /// Builds a [`TSImportType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_import_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_import_type<T1>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: T1,
    ) -> TSImportType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSImportType {
            span,
            is_type_of,
            parameter,
            qualifier,
            attributes,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSImportType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_import_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_import_type<T1>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: Option<TSImportAttributes<'a>>,
        type_parameters: T1,
    ) -> Box<'a, TSImportType<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            self.ts_import_type(
                span,
                is_type_of,
                parameter,
                qualifier,
                attributes,
                type_parameters,
            ),
            self.allocator,
        )
    }

    /// Builds a [`TSImportAttributes`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_import_attributes`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - attributes_keyword
    /// - elements
    #[inline]
    pub fn ts_import_attributes(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> TSImportAttributes<'a> {
        TSImportAttributes { span, attributes_keyword, elements }
    }

    /// Builds a [`TSImportAttributes`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_import_attributes`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - attributes_keyword
    /// - elements
    #[inline]
    pub fn alloc_ts_import_attributes(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> Box<'a, TSImportAttributes<'a>> {
        Box::new_in(self.ts_import_attributes(span, attributes_keyword, elements), self.allocator)
    }

    /// Builds a [`TSImportAttribute`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_import_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - value
    #[inline]
    pub fn ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> TSImportAttribute<'a> {
        TSImportAttribute { span, name, value }
    }

    /// Builds a [`TSImportAttribute`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_import_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    /// - value
    #[inline]
    pub fn alloc_ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> Box<'a, TSImportAttribute<'a>> {
        Box::new_in(self.ts_import_attribute(span, name, value), self.allocator)
    }

    /// Build a [`TSImportAttributeName::Identifier`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn ts_import_attribute_name_identifier_name<A>(
        self,
        span: Span,
        name: A,
    ) -> TSImportAttributeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSImportAttributeName::Identifier(self.identifier_name(span, name))
    }

    /// Convert a [`IdentifierName`] into a [`TSImportAttributeName::Identifier`]
    #[inline]
    pub fn ts_import_attribute_name_from_identifier_name<T>(
        self,
        inner: T,
    ) -> TSImportAttributeName<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        TSImportAttributeName::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`TSImportAttributeName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn ts_import_attribute_name_string_literal<A>(
        self,
        span: Span,
        value: A,
    ) -> TSImportAttributeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSImportAttributeName::StringLiteral(self.string_literal(span, value))
    }

    /// Convert a [`StringLiteral`] into a [`TSImportAttributeName::StringLiteral`]
    #[inline]
    pub fn ts_import_attribute_name_from_string_literal<T>(
        self,
        inner: T,
    ) -> TSImportAttributeName<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        TSImportAttributeName::StringLiteral(inner.into_in(self.allocator))
    }

    /// Builds a [`TSFunctionType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_function_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_function_type<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSFunctionType<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSFunctionType {
            span,
            this_param,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSFunctionType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_function_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_param
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_function_type<T1, T2, T3>(
        self,
        span: Span,
        this_param: Option<TSThisParameter<'a>>,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> Box<'a, TSFunctionType<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_function_type(span, this_param, params, return_type, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`TSConstructorType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn ts_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> TSConstructorType<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSConstructorType {
            span,
            r#abstract,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSConstructorType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - params
    /// - return_type
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        params: T1,
        return_type: T2,
        type_parameters: T3,
    ) -> Box<'a, TSConstructorType<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_constructor_type(span, r#abstract, params, return_type, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`TSMappedType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    /// - name_type
    /// - type_annotation
    /// - optional
    /// - readonly
    #[inline]
    pub fn ts_mapped_type<T1>(
        self,
        span: Span,
        type_parameter: T1,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> TSMappedType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSMappedType {
            span,
            type_parameter: type_parameter.into_in(self.allocator),
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id: Default::default(),
        }
    }

    /// Builds a [`TSMappedType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter
    /// - name_type
    /// - type_annotation
    /// - optional
    /// - readonly
    #[inline]
    pub fn alloc_ts_mapped_type<T1>(
        self,
        span: Span,
        type_parameter: T1,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
    ) -> Box<'a, TSMappedType<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Box::new_in(
            self.ts_mapped_type(
                span,
                type_parameter,
                name_type,
                type_annotation,
                optional,
                readonly,
            ),
            self.allocator,
        )
    }

    /// Builds a [`TSTemplateLiteralType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - types
    #[inline]
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSTemplateLiteralType<'a> {
        TSTemplateLiteralType { span, quasis, types }
    }

    /// Builds a [`TSTemplateLiteralType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis
    /// - types
    #[inline]
    pub fn alloc_ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTemplateLiteralType<'a>> {
        Box::new_in(self.ts_template_literal_type(span, quasis, types), self.allocator)
    }

    /// Builds a [`TSAsExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_as_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSAsExpression<'a> {
        TSAsExpression { span, expression, type_annotation }
    }

    /// Builds a [`TSAsExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_as_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSAsExpression<'a>> {
        Box::new_in(self.ts_as_expression(span, expression, type_annotation), self.allocator)
    }

    /// Builds a [`TSSatisfiesExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSSatisfiesExpression<'a> {
        TSSatisfiesExpression { span, expression, type_annotation }
    }

    /// Builds a [`TSSatisfiesExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSSatisfiesExpression<'a>> {
        Box::new_in(self.ts_satisfies_expression(span, expression, type_annotation), self.allocator)
    }

    /// Builds a [`TSTypeAssertion`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_assertion`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSTypeAssertion<'a> {
        TSTypeAssertion { span, expression, type_annotation }
    }

    /// Builds a [`TSTypeAssertion`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_assertion`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_annotation
    #[inline]
    pub fn alloc_ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAssertion<'a>> {
        Box::new_in(self.ts_type_assertion(span, expression, type_annotation), self.allocator)
    }

    /// Builds a [`TSImportEqualsDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_import_equals_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - module_reference
    /// - import_kind
    #[inline]
    pub fn ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> TSImportEqualsDeclaration<'a> {
        TSImportEqualsDeclaration { span, id, module_reference, import_kind }
    }

    /// Builds a [`TSImportEqualsDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_import_equals_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - module_reference
    /// - import_kind
    #[inline]
    pub fn alloc_ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, TSImportEqualsDeclaration<'a>> {
        Box::new_in(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind),
            self.allocator,
        )
    }

    /// Build a [`TSModuleReference::ExternalModuleReference`]
    ///
    /// This node contains a [`TSExternalModuleReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn ts_module_reference_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSModuleReference<'a> {
        TSModuleReference::ExternalModuleReference(
            self.alloc(self.ts_external_module_reference(span, expression)),
        )
    }

    /// Convert a [`TSExternalModuleReference`] into a [`TSModuleReference::ExternalModuleReference`]
    #[inline]
    pub fn ts_module_reference_from_ts_external_module_reference<T>(
        self,
        inner: T,
    ) -> TSModuleReference<'a>
    where
        T: IntoIn<'a, Box<'a, TSExternalModuleReference<'a>>>,
    {
        TSModuleReference::ExternalModuleReference(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_module_reference_type_name(self, inner: TSTypeName<'a>) -> TSModuleReference<'a> {
        TSModuleReference::from(inner)
    }

    /// Builds a [`TSExternalModuleReference`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_external_module_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSExternalModuleReference<'a> {
        TSExternalModuleReference { span, expression }
    }

    /// Builds a [`TSExternalModuleReference`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_external_module_reference`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> Box<'a, TSExternalModuleReference<'a>> {
        Box::new_in(self.ts_external_module_reference(span, expression), self.allocator)
    }

    /// Builds a [`TSNonNullExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_non_null_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSNonNullExpression<'a> {
        TSNonNullExpression { span, expression }
    }

    /// Builds a [`TSNonNullExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_non_null_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSNonNullExpression<'a>> {
        Box::new_in(self.ts_non_null_expression(span, expression), self.allocator)
    }

    /// Builds a [`Decorator`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_decorator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn decorator(self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { span, expression }
    }

    /// Builds a [`Decorator`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::decorator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_decorator(self, span: Span, expression: Expression<'a>) -> Box<'a, Decorator<'a>> {
        Box::new_in(self.decorator(span, expression), self.allocator)
    }

    /// Builds a [`TSExportAssignment`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_export_assignment`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSExportAssignment<'a> {
        TSExportAssignment { span, expression }
    }

    /// Builds a [`TSExportAssignment`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_export_assignment`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn alloc_ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSExportAssignment<'a>> {
        Box::new_in(self.ts_export_assignment(span, expression), self.allocator)
    }

    /// Builds a [`TSNamespaceExportDeclaration`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_namespace_export_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    #[inline]
    pub fn ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> TSNamespaceExportDeclaration<'a> {
        TSNamespaceExportDeclaration { span, id }
    }

    /// Builds a [`TSNamespaceExportDeclaration`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_namespace_export_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    #[inline]
    pub fn alloc_ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> Box<'a, TSNamespaceExportDeclaration<'a>> {
        Box::new_in(self.ts_namespace_export_declaration(span, id), self.allocator)
    }

    /// Builds a [`TSInstantiationExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_instantiation_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn ts_instantiation_expression<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> TSInstantiationExpression<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        TSInstantiationExpression {
            span,
            expression,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`TSInstantiationExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_instantiation_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_instantiation_expression<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_parameters: T1,
    ) -> Box<'a, TSInstantiationExpression<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Box::new_in(
            self.ts_instantiation_expression(span, expression, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`JSDocNullableType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_js_doc_nullable_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix: Was `?` after the type annotation?
    #[inline]
    pub fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNullableType<'a> {
        JSDocNullableType { span, type_annotation, postfix }
    }

    /// Builds a [`JSDocNullableType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::js_doc_nullable_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix: Was `?` after the type annotation?
    #[inline]
    pub fn alloc_js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNullableType<'a>> {
        Box::new_in(self.js_doc_nullable_type(span, type_annotation, postfix), self.allocator)
    }

    /// Builds a [`JSDocNonNullableType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_js_doc_non_nullable_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix
    #[inline]
    pub fn js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNonNullableType<'a> {
        JSDocNonNullableType { span, type_annotation, postfix }
    }

    /// Builds a [`JSDocNonNullableType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::js_doc_non_nullable_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_annotation
    /// - postfix
    #[inline]
    pub fn alloc_js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNonNullableType<'a>> {
        Box::new_in(self.js_doc_non_nullable_type(span, type_annotation, postfix), self.allocator)
    }

    /// Builds a [`JSDocUnknownType`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn js_doc_unknown_type(self, span: Span) -> JSDocUnknownType {
        JSDocUnknownType { span }
    }

    /// Builds a [`JSDocUnknownType`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_js_doc_unknown_type(self, span: Span) -> Box<'a, JSDocUnknownType> {
        Box::new_in(self.js_doc_unknown_type(span), self.allocator)
    }

    /// Builds a [`JSXElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_element: Opening tag of the element.
    /// - closing_element: Closing tag of the element. Will be [`None`] for self-closing tags.
    /// - children: Children of the element. This can be text, other elements, or expressions.
    #[inline]
    pub fn jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        closing_element: T2,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXElement<'a>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        JSXElement {
            span,
            opening_element: opening_element.into_in(self.allocator),
            closing_element: closing_element.into_in(self.allocator),
            children,
        }
    }

    /// Builds a [`JSXElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_element: Opening tag of the element.
    /// - closing_element: Closing tag of the element. Will be [`None`] for self-closing tags.
    /// - children: Children of the element. This can be text, other elements, or expressions.
    #[inline]
    pub fn alloc_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        closing_element: T2,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXElement<'a>>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Box::new_in(
            self.jsx_element(span, opening_element, closing_element, children),
            self.allocator,
        )
    }

    /// Builds a [`JSXOpeningElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - self_closing: Is this tag self-closing?
    /// - name
    /// - attributes: List of JSX attributes. In React-like applications, these become props.
    /// - type_parameters: Type parameters for generic JSX elements.
    #[inline]
    pub fn jsx_opening_element<T1>(
        self,
        span: Span,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        type_parameters: T1,
    ) -> JSXOpeningElement<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        JSXOpeningElement {
            span,
            self_closing,
            name,
            attributes,
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Builds a [`JSXOpeningElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - self_closing: Is this tag self-closing?
    /// - name
    /// - attributes: List of JSX attributes. In React-like applications, these become props.
    /// - type_parameters: Type parameters for generic JSX elements.
    #[inline]
    pub fn alloc_jsx_opening_element<T1>(
        self,
        span: Span,
        self_closing: bool,
        name: JSXElementName<'a>,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        type_parameters: T1,
    ) -> Box<'a, JSXOpeningElement<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            self.jsx_opening_element(span, self_closing, name, attributes, type_parameters),
            self.allocator,
        )
    }

    /// Builds a [`JSXClosingElement`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { span, name }
    }

    /// Builds a [`JSXClosingElement`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name
    #[inline]
    pub fn alloc_jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        Box::new_in(self.jsx_closing_element(span, name), self.allocator)
    }

    /// Builds a [`JSXFragment`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_fragment: `<>`
    /// - closing_fragment: `</>`
    /// - children: Elements inside the fragment.
    #[inline]
    pub fn jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXFragment<'a> {
        JSXFragment { span, opening_fragment, closing_fragment, children }
    }

    /// Builds a [`JSXFragment`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_fragment: `<>`
    /// - closing_fragment: `</>`
    /// - children: Elements inside the fragment.
    #[inline]
    pub fn alloc_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        Box::new_in(
            self.jsx_fragment(span, opening_fragment, closing_fragment, children),
            self.allocator,
        )
    }

    /// Build a [`JSXElementName::Identifier`]
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_element_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXElementName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXElementName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    /// Convert a [`JSXIdentifier`] into a [`JSXElementName::Identifier`]
    #[inline]
    pub fn jsx_element_name_from_jsx_identifier<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXIdentifier<'a>>>,
    {
        JSXElementName::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`JSXElementName::NamespacedName`]
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - namespace: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// - property: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn jsx_element_name_jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::NamespacedName(
            self.alloc(self.jsx_namespaced_name(span, namespace, property)),
        )
    }

    /// Convert a [`JSXNamespacedName`] into a [`JSXElementName::NamespacedName`]
    #[inline]
    pub fn jsx_element_name_from_jsx_namespaced_name<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXNamespacedName<'a>>>,
    {
        JSXElementName::NamespacedName(inner.into_in(self.allocator))
    }

    /// Build a [`JSXElementName::MemberExpression`]
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object: The object being accessed. This is everything before the last `.`.
    /// - property: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn jsx_element_name_jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::MemberExpression(
            self.alloc(self.jsx_member_expression(span, object, property)),
        )
    }

    /// Convert a [`JSXMemberExpression`] into a [`JSXElementName::MemberExpression`]
    #[inline]
    pub fn jsx_element_name_from_jsx_member_expression<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXMemberExpression<'a>>>,
    {
        JSXElementName::MemberExpression(inner.into_in(self.allocator))
    }

    /// Builds a [`JSXNamespacedName`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - namespace: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// - property: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXNamespacedName<'a> {
        JSXNamespacedName { span, namespace, property }
    }

    /// Builds a [`JSXNamespacedName`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - namespace: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// - property: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn alloc_jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXNamespacedName<'a>> {
        Box::new_in(self.jsx_namespaced_name(span, namespace, property), self.allocator)
    }

    /// Builds a [`JSXMemberExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object: The object being accessed. This is everything before the last `.`.
    /// - property: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpression<'a> {
        JSXMemberExpression { span, object, property }
    }

    /// Builds a [`JSXMemberExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object: The object being accessed. This is everything before the last `.`.
    /// - property: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn alloc_jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        Box::new_in(self.jsx_member_expression(span, object, property), self.allocator)
    }

    /// Build a [`JSXMemberExpressionObject::Identifier`]
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_member_expression_object_jsx_identifier<A>(
        self,
        span: Span,
        name: A,
    ) -> JSXMemberExpressionObject<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXMemberExpressionObject::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    /// Convert a [`JSXIdentifier`] into a [`JSXMemberExpressionObject::Identifier`]
    #[inline]
    pub fn jsx_member_expression_object_from_jsx_identifier<T>(
        self,
        inner: T,
    ) -> JSXMemberExpressionObject<'a>
    where
        T: IntoIn<'a, Box<'a, JSXIdentifier<'a>>>,
    {
        JSXMemberExpressionObject::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`JSXMemberExpressionObject::MemberExpression`]
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - object: The object being accessed. This is everything before the last `.`.
    /// - property: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn jsx_member_expression_object_jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpressionObject<'a> {
        JSXMemberExpressionObject::MemberExpression(
            self.alloc(self.jsx_member_expression(span, object, property)),
        )
    }

    /// Convert a [`JSXMemberExpression`] into a [`JSXMemberExpressionObject::MemberExpression`]
    #[inline]
    pub fn jsx_member_expression_object_from_jsx_member_expression<T>(
        self,
        inner: T,
    ) -> JSXMemberExpressionObject<'a>
    where
        T: IntoIn<'a, Box<'a, JSXMemberExpression<'a>>>,
    {
        JSXMemberExpressionObject::MemberExpression(inner.into_in(self.allocator))
    }

    /// Builds a [`JSXExpressionContainer`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression inside the container.
    #[inline]
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { span, expression }
    }

    /// Builds a [`JSXExpressionContainer`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression inside the container.
    #[inline]
    pub fn alloc_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        Box::new_in(self.jsx_expression_container(span, expression), self.allocator)
    }

    /// Build a [`JSXExpression::EmptyExpression`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn jsx_expression_jsx_empty_expression(self, span: Span) -> JSXExpression<'a> {
        JSXExpression::EmptyExpression(self.jsx_empty_expression(span))
    }

    /// Convert a [`JSXEmptyExpression`] into a [`JSXExpression::EmptyExpression`]
    #[inline]
    pub fn jsx_expression_from_jsx_empty_expression<T>(self, inner: T) -> JSXExpression<'a>
    where
        T: IntoIn<'a, JSXEmptyExpression>,
    {
        JSXExpression::EmptyExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_expression_expression(self, inner: Expression<'a>) -> JSXExpression<'a> {
        JSXExpression::from(inner)
    }

    /// Builds a [`JSXEmptyExpression`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }

    /// Builds a [`JSXEmptyExpression`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_jsx_empty_expression(self, span: Span) -> Box<'a, JSXEmptyExpression> {
        Box::new_in(self.jsx_empty_expression(span), self.allocator)
    }

    /// Build a [`JSXAttributeItem::Attribute`]
    ///
    /// This node contains a [`JSXAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the attribute. This is a prop in React-like applications.
    /// - value: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn jsx_attribute_item_jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::Attribute(self.alloc(self.jsx_attribute(span, name, value)))
    }

    /// Convert a [`JSXAttribute`] into a [`JSXAttributeItem::Attribute`]
    #[inline]
    pub fn jsx_attribute_item_from_jsx_attribute<T>(self, inner: T) -> JSXAttributeItem<'a>
    where
        T: IntoIn<'a, Box<'a, JSXAttribute<'a>>>,
    {
        JSXAttributeItem::Attribute(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeItem::SpreadAttribute`]
    ///
    /// This node contains a [`JSXSpreadAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn jsx_attribute_item_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::SpreadAttribute(self.alloc(self.jsx_spread_attribute(span, argument)))
    }

    /// Convert a [`JSXSpreadAttribute`] into a [`JSXAttributeItem::SpreadAttribute`]
    #[inline]
    pub fn jsx_attribute_item_from_jsx_spread_attribute<T>(self, inner: T) -> JSXAttributeItem<'a>
    where
        T: IntoIn<'a, Box<'a, JSXSpreadAttribute<'a>>>,
    {
        JSXAttributeItem::SpreadAttribute(inner.into_in(self.allocator))
    }

    /// Builds a [`JSXAttribute`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the attribute. This is a prop in React-like applications.
    /// - value: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttribute<'a> {
        JSXAttribute { span, name, value }
    }

    /// Builds a [`JSXAttribute`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the attribute. This is a prop in React-like applications.
    /// - value: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn alloc_jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        Box::new_in(self.jsx_attribute(span, name, value), self.allocator)
    }

    /// Builds a [`JSXSpreadAttribute`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXSpreadAttribute<'a> {
        JSXSpreadAttribute { span, argument }
    }

    /// Builds a [`JSXSpreadAttribute`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn alloc_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        Box::new_in(self.jsx_spread_attribute(span, argument), self.allocator)
    }

    /// Build a [`JSXAttributeName::Identifier`]
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_attribute_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXAttributeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXAttributeName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    /// Convert a [`JSXIdentifier`] into a [`JSXAttributeName::Identifier`]
    #[inline]
    pub fn jsx_attribute_name_from_jsx_identifier<T>(self, inner: T) -> JSXAttributeName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXIdentifier<'a>>>,
    {
        JSXAttributeName::Identifier(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeName::NamespacedName`]
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - namespace: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// - property: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn jsx_attribute_name_jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXAttributeName<'a> {
        JSXAttributeName::NamespacedName(
            self.alloc(self.jsx_namespaced_name(span, namespace, property)),
        )
    }

    /// Convert a [`JSXNamespacedName`] into a [`JSXAttributeName::NamespacedName`]
    #[inline]
    pub fn jsx_attribute_name_from_jsx_namespaced_name<T>(self, inner: T) -> JSXAttributeName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXNamespacedName<'a>>>,
    {
        JSXAttributeName::NamespacedName(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeValue::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value
    #[inline]
    pub fn jsx_attribute_value_string_literal<A>(
        self,
        span: Span,
        value: A,
    ) -> JSXAttributeValue<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXAttributeValue::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    /// Convert a [`StringLiteral`] into a [`JSXAttributeValue::StringLiteral`]
    #[inline]
    pub fn jsx_attribute_value_from_string_literal<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        JSXAttributeValue::StringLiteral(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeValue::ExpressionContainer`]
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression inside the container.
    #[inline]
    pub fn jsx_attribute_value_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXAttributeValue<'a> {
        JSXAttributeValue::ExpressionContainer(
            self.alloc(self.jsx_expression_container(span, expression)),
        )
    }

    /// Convert a [`JSXExpressionContainer`] into a [`JSXAttributeValue::ExpressionContainer`]
    #[inline]
    pub fn jsx_attribute_value_from_jsx_expression_container<T>(
        self,
        inner: T,
    ) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, JSXExpressionContainer<'a>>>,
    {
        JSXAttributeValue::ExpressionContainer(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeValue::Element`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_element: Opening tag of the element.
    /// - closing_element: Closing tag of the element. Will be [`None`] for self-closing tags.
    /// - children: Children of the element. This can be text, other elements, or expressions.
    #[inline]
    pub fn jsx_attribute_value_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        closing_element: T2,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXAttributeValue<'a>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        JSXAttributeValue::Element(self.alloc(self.jsx_element(
            span,
            opening_element,
            closing_element,
            children,
        )))
    }

    /// Convert a [`JSXElement`] into a [`JSXAttributeValue::Element`]
    #[inline]
    pub fn jsx_attribute_value_from_jsx_element<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        JSXAttributeValue::Element(inner.into_in(self.allocator))
    }

    /// Build a [`JSXAttributeValue::Fragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_fragment: `<>`
    /// - closing_fragment: `</>`
    /// - children: Elements inside the fragment.
    #[inline]
    pub fn jsx_attribute_value_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXAttributeValue<'a> {
        JSXAttributeValue::Fragment(self.alloc(self.jsx_fragment(
            span,
            opening_fragment,
            closing_fragment,
            children,
        )))
    }

    /// Convert a [`JSXFragment`] into a [`JSXAttributeValue::Fragment`]
    #[inline]
    pub fn jsx_attribute_value_from_jsx_fragment<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        JSXAttributeValue::Fragment(inner.into_in(self.allocator))
    }

    /// Builds a [`JSXIdentifier`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_identifier<A>(self, span: Span, name: A) -> JSXIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXIdentifier { span, name: name.into_in(self.allocator) }
    }

    /// Builds a [`JSXIdentifier`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier.
    #[inline]
    pub fn alloc_jsx_identifier<A>(self, span: Span, name: A) -> Box<'a, JSXIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.jsx_identifier(span, name), self.allocator)
    }

    /// Build a [`JSXChild::Text`]
    ///
    /// This node contains a [`JSXText`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The text content.
    #[inline]
    pub fn jsx_child_jsx_text<A>(self, span: Span, value: A) -> JSXChild<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXChild::Text(self.alloc(self.jsx_text(span, value)))
    }

    /// Convert a [`JSXText`] into a [`JSXChild::Text`]
    #[inline]
    pub fn jsx_child_from_jsx_text<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXText<'a>>>,
    {
        JSXChild::Text(inner.into_in(self.allocator))
    }

    /// Build a [`JSXChild::Element`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_element: Opening tag of the element.
    /// - closing_element: Closing tag of the element. Will be [`None`] for self-closing tags.
    /// - children: Children of the element. This can be text, other elements, or expressions.
    #[inline]
    pub fn jsx_child_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        closing_element: T2,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXChild<'a>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        JSXChild::Element(self.alloc(self.jsx_element(
            span,
            opening_element,
            closing_element,
            children,
        )))
    }

    /// Convert a [`JSXElement`] into a [`JSXChild::Element`]
    #[inline]
    pub fn jsx_child_from_jsx_element<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        JSXChild::Element(inner.into_in(self.allocator))
    }

    /// Build a [`JSXChild::Fragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - opening_fragment: `<>`
    /// - closing_fragment: `</>`
    /// - children: Elements inside the fragment.
    #[inline]
    pub fn jsx_child_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> JSXChild<'a> {
        JSXChild::Fragment(self.alloc(self.jsx_fragment(
            span,
            opening_fragment,
            closing_fragment,
            children,
        )))
    }

    /// Convert a [`JSXFragment`] into a [`JSXChild::Fragment`]
    #[inline]
    pub fn jsx_child_from_jsx_fragment<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        JSXChild::Fragment(inner.into_in(self.allocator))
    }

    /// Build a [`JSXChild::ExpressionContainer`]
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression inside the container.
    #[inline]
    pub fn jsx_child_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::ExpressionContainer(self.alloc(self.jsx_expression_container(span, expression)))
    }

    /// Convert a [`JSXExpressionContainer`] into a [`JSXChild::ExpressionContainer`]
    #[inline]
    pub fn jsx_child_from_jsx_expression_container<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXExpressionContainer<'a>>>,
    {
        JSXChild::ExpressionContainer(inner.into_in(self.allocator))
    }

    /// Build a [`JSXChild::Spread`]
    ///
    /// This node contains a [`JSXSpreadChild`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression being spread.
    #[inline]
    pub fn jsx_child_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::Spread(self.alloc(self.jsx_spread_child(span, expression)))
    }

    /// Convert a [`JSXSpreadChild`] into a [`JSXChild::Spread`]
    #[inline]
    pub fn jsx_child_from_jsx_spread_child<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXSpreadChild<'a>>>,
    {
        JSXChild::Spread(inner.into_in(self.allocator))
    }

    /// Builds a [`JSXSpreadChild`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression being spread.
    #[inline]
    pub fn jsx_spread_child(self, span: Span, expression: Expression<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expression }
    }

    /// Builds a [`JSXSpreadChild`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The expression being spread.
    #[inline]
    pub fn alloc_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        Box::new_in(self.jsx_spread_child(span, expression), self.allocator)
    }

    /// Builds a [`JSXText`]
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_text`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The text content.
    #[inline]
    pub fn jsx_text<A>(self, span: Span, value: A) -> JSXText<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXText { span, value: value.into_in(self.allocator) }
    }

    /// Builds a [`JSXText`] and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_text`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - value: The text content.
    #[inline]
    pub fn alloc_jsx_text<A>(self, span: Span, value: A) -> Box<'a, JSXText<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.jsx_text(span, value), self.allocator)
    }
}
