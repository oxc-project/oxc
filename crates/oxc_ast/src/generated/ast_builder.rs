// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_builder.rs`

//! AST node factories

#![allow(
    clippy::default_trait_access,
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools
)]

use std::cell::Cell;

use oxc_allocator::{Allocator, Box, IntoIn, Vec};
use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::ast::*;

/// AST builder for creating AST nodes
#[derive(Clone, Copy)]
pub struct AstBuilder<'a> {
    /// The memory allocator used to allocate AST nodes in the arena.
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    /// Build a [`BooleanLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_boolean_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The boolean value itself
    #[inline]
    pub fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }

    /// Build a [`BooleanLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::boolean_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The boolean value itself
    #[inline]
    pub fn alloc_boolean_literal(self, span: Span, value: bool) -> Box<'a, BooleanLiteral> {
        Box::new_in(self.boolean_literal(span, value), self.allocator)
    }

    /// Build a [`NullLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_null_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn null_literal(self, span: Span) -> NullLiteral {
        NullLiteral { span }
    }

    /// Build a [`NullLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::null_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn alloc_null_literal(self, span: Span) -> Box<'a, NullLiteral> {
        Box::new_in(self.null_literal(span), self.allocator)
    }

    /// Build a [`NumericLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_numeric_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`NumericLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::numeric_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`BigIntLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_big_int_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - raw: The bigint as it appears in source code
    /// - base: The base representation used by the literal in source code
    #[inline]
    pub fn big_int_literal<A>(self, span: Span, raw: A, base: BigintBase) -> BigIntLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BigIntLiteral { span, raw: raw.into_in(self.allocator), base }
    }

    /// Build a [`BigIntLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::big_int_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - raw: The bigint as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`RegExpLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - regex: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// - raw: The regular expression as it appears in source code
    #[inline]
    pub fn reg_exp_literal<S>(self, span: Span, regex: RegExp<'a>, raw: S) -> RegExpLiteral<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        RegExpLiteral { span, regex, raw: raw.into_in(self.allocator) }
    }

    /// Build a [`RegExpLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - regex: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// - raw: The regular expression as it appears in source code
    #[inline]
    pub fn alloc_reg_exp_literal<S>(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: S,
    ) -> Box<'a, RegExpLiteral<'a>>
    where
        S: IntoIn<'a, &'a str>,
    {
        Box::new_in(self.reg_exp_literal(span, regex, raw), self.allocator)
    }

    /// Build a [`StringLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_string_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn string_literal<A>(self, span: Span, value: A) -> StringLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        StringLiteral { span, value: value.into_in(self.allocator) }
    }

    /// Build a [`StringLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::string_literal`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn alloc_string_literal<A>(self, span: Span, value: A) -> Box<'a, StringLiteral<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.string_literal(span, value), self.allocator)
    }

    /// Build a [`Program`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_program`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - source_text
    /// - comments: Sorted comments
    /// - hashbang
    /// - directives
    /// - body
    #[inline]
    pub fn program<S>(
        self,
        span: Span,
        source_type: SourceType,
        source_text: S,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        Program {
            span,
            source_type,
            source_text: source_text.into_in(self.allocator),
            comments,
            hashbang,
            directives,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`Program`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::program`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - source_text
    /// - comments: Sorted comments
    /// - hashbang
    /// - directives
    /// - body
    #[inline]
    pub fn alloc_program<S>(
        self,
        span: Span,
        source_type: SourceType,
        source_text: S,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, Program<'a>>
    where
        S: IntoIn<'a, &'a str>,
    {
        Box::new_in(
            self.program(span, source_type, source_text, comments, hashbang, directives, body),
            self.allocator,
        )
    }

    /// Build a [`Program`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_program_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - source_text
    /// - comments: Sorted comments
    /// - hashbang
    /// - directives
    /// - body
    /// - scope_id
    #[inline]
    pub fn program_with_scope_id<S>(
        self,
        span: Span,
        source_type: SourceType,
        source_text: S,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Program<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        Program {
            span,
            source_type,
            source_text: source_text.into_in(self.allocator),
            comments,
            hashbang,
            directives,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Program`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::program_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - source_type
    /// - source_text
    /// - comments: Sorted comments
    /// - hashbang
    /// - directives
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_program_with_scope_id<S>(
        self,
        span: Span,
        source_type: SourceType,
        source_text: S,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Box<'a, Program<'a>>
    where
        S: IntoIn<'a, &'a str>,
    {
        Box::new_in(
            self.program_with_scope_id(
                span,
                source_type,
                source_text,
                comments,
                hashbang,
                directives,
                body,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build an [`Expression::BooleanLiteral`]
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The boolean value itself
    #[inline]
    pub fn expression_boolean_literal(self, span: Span, value: bool) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    /// Build an [`Expression::NullLiteral`]
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn expression_null_literal(self, span: Span) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(self.null_literal(span)))
    }

    /// Build an [`Expression::NumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build an [`Expression::BigIntLiteral`]
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - raw: The bigint as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build an [`Expression::RegExpLiteral`]
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - regex: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// - raw: The regular expression as it appears in source code
    #[inline]
    pub fn expression_reg_exp_literal<S>(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: S,
    ) -> Expression<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        Expression::RegExpLiteral(self.alloc(self.reg_exp_literal(span, regex, raw)))
    }

    /// Build an [`Expression::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn expression_string_literal<A>(self, span: Span, value: A) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    /// Build an [`Expression::TemplateLiteral`]
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

    /// Build an [`Expression::Identifier`]
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
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

    /// Build an [`Expression::MetaProperty`]
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

    /// Build an [`Expression::Super`]
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn expression_super(self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc(self.super_(span)))
    }

    /// Build an [`Expression::ArrayExpression`]
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::ArrowFunctionExpression`]
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::AssignmentExpression`]
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::AwaitExpression`]
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument
    #[inline]
    pub fn expression_await(self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(self.await_expression(span, argument)))
    }

    /// Build an [`Expression::BinaryExpression`]
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

    /// Build an [`Expression::CallExpression`]
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - type_parameters
    /// - arguments
    /// - optional
    #[inline]
    pub fn expression_call<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_parameters: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::CallExpression(self.alloc(self.call_expression(
            span,
            callee,
            type_parameters,
            arguments,
            optional,
        )))
    }

    /// Build an [`Expression::ChainExpression`]
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

    /// Build an [`Expression::ClassExpression`]
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

    /// Build an [`Expression::ConditionalExpression`]
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

    /// Build an [`Expression::FunctionExpression`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    #[inline]
    pub fn expression_function<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
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

    /// Build an [`Expression::ImportExpression`]
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::LogicalExpression`]
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

    /// Build an [`Expression::NewExpression`]
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

    /// Build an [`Expression::ObjectExpression`]
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::ParenthesizedExpression`]
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

    /// Build an [`Expression::SequenceExpression`]
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

    /// Build an [`Expression::TaggedTemplateExpression`]
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

    /// Build an [`Expression::ThisExpression`]
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn expression_this(self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(self.this_expression(span)))
    }

    /// Build an [`Expression::UnaryExpression`]
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::UpdateExpression`]
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
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

    /// Build an [`Expression::YieldExpression`]
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

    /// Build an [`Expression::PrivateInExpression`]
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

    /// Build an [`Expression::JSXElement`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build an [`Expression::JSXFragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build an [`Expression::TSAsExpression`]
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

    /// Build an [`Expression::TSSatisfiesExpression`]
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The value expression being constrained.
    /// - type_annotation: The type `expression` must satisfy.
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

    /// Build an [`Expression::TSTypeAssertion`]
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

    /// Build an [`Expression::TSNonNullExpression`]
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

    /// Build an [`Expression::TSInstantiationExpression`]
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

    /// Build an [`IdentifierName`].
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

    /// Build an [`IdentifierName`], and store it in the memory arena.
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

    /// Build an [`IdentifierReference`].
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
        }
    }

    /// Build an [`IdentifierReference`], and store it in the memory arena.
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

    /// Build an [`IdentifierReference`] with `ReferenceId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_identifier_reference_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    /// - reference_id: Reference ID
    #[inline]
    pub fn identifier_reference_with_reference_id<A>(
        self,
        span: Span,
        name: A,
        reference_id: ReferenceId,
    ) -> IdentifierReference<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        IdentifierReference {
            span,
            name: name.into_in(self.allocator),
            reference_id: Cell::new(Some(reference_id)),
        }
    }

    /// Build an [`IdentifierReference`] with `ReferenceId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::identifier_reference_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    /// - reference_id: Reference ID
    #[inline]
    pub fn alloc_identifier_reference_with_reference_id<A>(
        self,
        span: Span,
        name: A,
        reference_id: ReferenceId,
    ) -> Box<'a, IdentifierReference<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(
            self.identifier_reference_with_reference_id(span, name, reference_id),
            self.allocator,
        )
    }

    /// Build a [`BindingIdentifier`].
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

    /// Build a [`BindingIdentifier`], and store it in the memory arena.
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

    /// Build a [`BindingIdentifier`] with `SymbolId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_binding_identifier_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    /// - symbol_id: Unique identifier for this binding.
    #[inline]
    pub fn binding_identifier_with_symbol_id<A>(
        self,
        span: Span,
        name: A,
        symbol_id: SymbolId,
    ) -> BindingIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BindingIdentifier {
            span,
            name: name.into_in(self.allocator),
            symbol_id: Cell::new(Some(symbol_id)),
        }
    }

    /// Build a [`BindingIdentifier`] with `SymbolId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::binding_identifier_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    /// - symbol_id: Unique identifier for this binding.
    #[inline]
    pub fn alloc_binding_identifier_with_symbol_id<A>(
        self,
        span: Span,
        name: A,
        symbol_id: SymbolId,
    ) -> Box<'a, BindingIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.binding_identifier_with_symbol_id(span, name, symbol_id), self.allocator)
    }

    /// Build a [`LabelIdentifier`].
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

    /// Build a [`LabelIdentifier`], and store it in the memory arena.
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

    /// Build a [`ThisExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_this_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn this_expression(self, span: Span) -> ThisExpression {
        ThisExpression { span }
    }

    /// Build a [`ThisExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::this_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_this_expression(self, span: Span) -> Box<'a, ThisExpression> {
        Box::new_in(self.this_expression(span), self.allocator)
    }

    /// Build an [`ArrayExpression`].
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

    /// Build an [`ArrayExpression`], and store it in the memory arena.
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

    /// Build an [`ArrayExpressionElement::SpreadElement`]
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

    /// Build an [`ArrayExpressionElement::Elision`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn array_expression_element_elision(self, span: Span) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::Elision(self.elision(span))
    }

    /// Build an [`Elision`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_elision`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn elision(self, span: Span) -> Elision {
        Elision { span }
    }

    /// Build an [`Elision`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::elision`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_elision(self, span: Span) -> Box<'a, Elision> {
        Box::new_in(self.elision(span), self.allocator)
    }

    /// Build an [`ObjectExpression`].
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

    /// Build an [`ObjectExpression`], and store it in the memory arena.
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

    /// Build an [`ObjectPropertyKind::ObjectProperty`]
    ///
    /// This node contains an [`ObjectProperty`] that will be stored in the memory arena.
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

    /// Build an [`ObjectPropertyKind::SpreadProperty`]
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

    /// Build an [`ObjectProperty`].
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

    /// Build an [`ObjectProperty`], and store it in the memory arena.
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
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
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

    /// Build a [`TemplateLiteral`].
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

    /// Build a [`TemplateLiteral`], and store it in the memory arena.
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

    /// Build a [`TaggedTemplateExpression`].
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

    /// Build a [`TaggedTemplateExpression`], and store it in the memory arena.
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

    /// Build a [`TemplateElement`].
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

    /// Build a [`TemplateElement`], and store it in the memory arena.
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

    /// Build a [`ComputedMemberExpression`].
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

    /// Build a [`ComputedMemberExpression`], and store it in the memory arena.
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

    /// Build a [`StaticMemberExpression`].
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

    /// Build a [`StaticMemberExpression`], and store it in the memory arena.
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

    /// Build a [`PrivateFieldExpression`].
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

    /// Build a [`PrivateFieldExpression`], and store it in the memory arena.
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

    /// Build a [`CallExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_call_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - type_parameters
    /// - arguments
    /// - optional
    #[inline]
    pub fn call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_parameters: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
    ) -> CallExpression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        CallExpression {
            span,
            callee,
            type_parameters: type_parameters.into_in(self.allocator),
            arguments,
            optional,
        }
    }

    /// Build a [`CallExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::call_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - callee
    /// - type_parameters
    /// - arguments
    /// - optional
    #[inline]
    pub fn alloc_call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_parameters: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
    ) -> Box<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            self.call_expression(span, callee, type_parameters, arguments, optional),
            self.allocator,
        )
    }

    /// Build a [`NewExpression`].
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

    /// Build a [`NewExpression`], and store it in the memory arena.
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

    /// Build a [`MetaProperty`].
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

    /// Build a [`MetaProperty`], and store it in the memory arena.
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

    /// Build a [`SpreadElement`].
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

    /// Build a [`SpreadElement`], and store it in the memory arena.
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

    /// Build an [`Argument::SpreadElement`]
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

    /// Build an [`UpdateExpression`].
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

    /// Build an [`UpdateExpression`], and store it in the memory arena.
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

    /// Build an [`UnaryExpression`].
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

    /// Build an [`UnaryExpression`], and store it in the memory arena.
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

    /// Build a [`BinaryExpression`].
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

    /// Build a [`BinaryExpression`], and store it in the memory arena.
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

    /// Build a [`PrivateInExpression`].
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

    /// Build a [`PrivateInExpression`], and store it in the memory arena.
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

    /// Build a [`LogicalExpression`].
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

    /// Build a [`LogicalExpression`], and store it in the memory arena.
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

    /// Build a [`ConditionalExpression`].
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

    /// Build a [`ConditionalExpression`], and store it in the memory arena.
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

    /// Build an [`AssignmentExpression`].
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

    /// Build an [`AssignmentExpression`], and store it in the memory arena.
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

    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`]
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
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

    /// Build a [`SimpleAssignmentTarget::TSSatisfiesExpression`]
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The value expression being constrained.
    /// - type_annotation: The type `expression` must satisfy.
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

    /// Build an [`AssignmentTargetPattern::ArrayAssignmentTarget`]
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
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

    /// Build an [`AssignmentTargetPattern::ObjectAssignmentTarget`]
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
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

    /// Build an [`ArrayAssignmentTarget`].
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

    /// Build an [`ArrayAssignmentTarget`], and store it in the memory arena.
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

    /// Build an [`ObjectAssignmentTarget`].
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

    /// Build an [`ObjectAssignmentTarget`], and store it in the memory arena.
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

    /// Build an [`AssignmentTargetRest`].
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

    /// Build an [`AssignmentTargetRest`], and store it in the memory arena.
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

    /// Build an [`AssignmentTargetMaybeDefault::AssignmentTargetWithDefault`]
    ///
    /// This node contains an [`AssignmentTargetWithDefault`] that will be stored in the memory arena.
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

    /// Build an [`AssignmentTargetWithDefault`].
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

    /// Build an [`AssignmentTargetWithDefault`], and store it in the memory arena.
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

    /// Build an [`AssignmentTargetProperty::AssignmentTargetPropertyIdentifier`]
    ///
    /// This node contains an [`AssignmentTargetPropertyIdentifier`] that will be stored in the memory arena.
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

    /// Build an [`AssignmentTargetProperty::AssignmentTargetPropertyProperty`]
    ///
    /// This node contains an [`AssignmentTargetPropertyProperty`] that will be stored in the memory arena.
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

    /// Build an [`AssignmentTargetPropertyIdentifier`].
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

    /// Build an [`AssignmentTargetPropertyIdentifier`], and store it in the memory arena.
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

    /// Build an [`AssignmentTargetPropertyProperty`].
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

    /// Build an [`AssignmentTargetPropertyProperty`], and store it in the memory arena.
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

    /// Build a [`SequenceExpression`].
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

    /// Build a [`SequenceExpression`], and store it in the memory arena.
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

    /// Build a [`Super`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_super_`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn super_(self, span: Span) -> Super {
        Super { span }
    }

    /// Build a [`Super`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::super_`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_super_(self, span: Span) -> Box<'a, Super> {
        Box::new_in(self.super_(span), self.allocator)
    }

    /// Build an [`AwaitExpression`].
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

    /// Build an [`AwaitExpression`], and store it in the memory arena.
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

    /// Build a [`ChainExpression`].
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

    /// Build a [`ChainExpression`], and store it in the memory arena.
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
    /// - callee
    /// - type_parameters
    /// - arguments
    /// - optional
    #[inline]
    pub fn chain_element_call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_parameters: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
    ) -> ChainElement<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ChainElement::CallExpression(self.alloc(self.call_expression(
            span,
            callee,
            type_parameters,
            arguments,
            optional,
        )))
    }

    /// Build a [`ParenthesizedExpression`].
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

    /// Build a [`ParenthesizedExpression`], and store it in the memory arena.
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

    /// Build a [`Statement::EmptyStatement`]
    ///
    /// This node contains an [`EmptyStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn statement_empty(self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(self.empty_statement(span)))
    }

    /// Build a [`Statement::ExpressionStatement`]
    ///
    /// This node contains an [`ExpressionStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression
    #[inline]
    pub fn statement_expression(self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(self.expression_statement(span, expression)))
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

    /// Build a [`Statement::IfStatement`]
    ///
    /// This node contains an [`IfStatement`] that will be stored in the memory arena.
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

    /// Build a [`Statement::ThrowStatement`]
    ///
    /// This node contains a [`ThrowStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn statement_throw(self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(self.throw_statement(span, argument)))
    }

    /// Build a [`Statement::TryStatement`]
    ///
    /// This node contains a [`TryStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block: Statements in the `try` block
    /// - handler: The `catch` clause, including the parameter and the block statement
    /// - finalizer: The `finally` clause
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

    /// Build a [`Directive`].
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

    /// Build a [`Directive`], and store it in the memory arena.
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

    /// Build a [`Hashbang`].
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

    /// Build a [`Hashbang`], and store it in the memory arena.
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

    /// Build a [`BlockStatement`].
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

    /// Build a [`BlockStatement`], and store it in the memory arena.
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

    /// Build a [`BlockStatement`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_block_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - scope_id
    #[inline]
    pub fn block_statement_with_scope_id(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> BlockStatement<'a> {
        BlockStatement { span, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`BlockStatement`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::block_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_block_statement_with_scope_id(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Box<'a, BlockStatement<'a>> {
        Box::new_in(self.block_statement_with_scope_id(span, body, scope_id), self.allocator)
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

    /// Build a [`Declaration::FunctionDeclaration`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    #[inline]
    pub fn declaration_function<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
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

    /// Build a [`Declaration::TSTypeAliasDeclaration`]
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
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

    /// Build a [`Declaration::TSInterfaceDeclaration`]
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
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

    /// Build a [`Declaration::TSEnumDeclaration`]
    ///
    /// This node contains a [`TSEnumDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const: `true` for const enums
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

    /// Build a [`Declaration::TSModuleDeclaration`]
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
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

    /// Build a [`VariableDeclaration`].
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

    /// Build a [`VariableDeclaration`], and store it in the memory arena.
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

    /// Build a [`VariableDeclarator`].
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

    /// Build a [`VariableDeclarator`], and store it in the memory arena.
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

    /// Build an [`EmptyStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_empty_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn empty_statement(self, span: Span) -> EmptyStatement {
        EmptyStatement { span }
    }

    /// Build an [`EmptyStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::empty_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_empty_statement(self, span: Span) -> Box<'a, EmptyStatement> {
        Box::new_in(self.empty_statement(span), self.allocator)
    }

    /// Build an [`ExpressionStatement`].
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

    /// Build an [`ExpressionStatement`], and store it in the memory arena.
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

    /// Build an [`IfStatement`].
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

    /// Build an [`IfStatement`], and store it in the memory arena.
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

    /// Build a [`DoWhileStatement`].
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

    /// Build a [`DoWhileStatement`], and store it in the memory arena.
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

    /// Build a [`WhileStatement`].
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

    /// Build a [`WhileStatement`], and store it in the memory arena.
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

    /// Build a [`ForStatement`].
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

    /// Build a [`ForStatement`], and store it in the memory arena.
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

    /// Build a [`ForStatement`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - init
    /// - test
    /// - update
    /// - body
    /// - scope_id
    #[inline]
    pub fn for_statement_with_scope_id(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ForStatement<'a> {
        ForStatement { span, init, test, update, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`ForStatement`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - init
    /// - test
    /// - update
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_for_statement_with_scope_id(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Box<'a, ForStatement<'a>> {
        Box::new_in(
            self.for_statement_with_scope_id(span, init, test, update, body, scope_id),
            self.allocator,
        )
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

    /// Build a [`ForInStatement`].
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

    /// Build a [`ForInStatement`], and store it in the memory arena.
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

    /// Build a [`ForInStatement`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_in_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    /// - body
    /// - scope_id
    #[inline]
    pub fn for_in_statement_with_scope_id(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ForInStatement<'a> {
        ForInStatement { span, left, right, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`ForInStatement`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_in_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - left
    /// - right
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_for_in_statement_with_scope_id(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Box<'a, ForInStatement<'a>> {
        Box::new_in(
            self.for_in_statement_with_scope_id(span, left, right, body, scope_id),
            self.allocator,
        )
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

    /// Build a [`ForOfStatement`].
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

    /// Build a [`ForOfStatement`], and store it in the memory arena.
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

    /// Build a [`ForOfStatement`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_for_of_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#await
    /// - left
    /// - right
    /// - body
    /// - scope_id
    #[inline]
    pub fn for_of_statement_with_scope_id(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ForOfStatement<'a> {
        ForOfStatement { span, r#await, left, right, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`ForOfStatement`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::for_of_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#await
    /// - left
    /// - right
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_for_of_statement_with_scope_id(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Box<'a, ForOfStatement<'a>> {
        Box::new_in(
            self.for_of_statement_with_scope_id(span, r#await, left, right, body, scope_id),
            self.allocator,
        )
    }

    /// Build a [`ContinueStatement`].
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

    /// Build a [`ContinueStatement`], and store it in the memory arena.
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

    /// Build a [`BreakStatement`].
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

    /// Build a [`BreakStatement`], and store it in the memory arena.
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

    /// Build a [`ReturnStatement`].
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

    /// Build a [`ReturnStatement`], and store it in the memory arena.
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

    /// Build a [`WithStatement`].
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

    /// Build a [`WithStatement`], and store it in the memory arena.
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

    /// Build a [`SwitchStatement`].
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

    /// Build a [`SwitchStatement`], and store it in the memory arena.
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

    /// Build a [`SwitchStatement`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_switch_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - discriminant
    /// - cases
    /// - scope_id
    #[inline]
    pub fn switch_statement_with_scope_id(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
    ) -> SwitchStatement<'a> {
        SwitchStatement { span, discriminant, cases, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`SwitchStatement`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::switch_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - discriminant
    /// - cases
    /// - scope_id
    #[inline]
    pub fn alloc_switch_statement_with_scope_id(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
    ) -> Box<'a, SwitchStatement<'a>> {
        Box::new_in(
            self.switch_statement_with_scope_id(span, discriminant, cases, scope_id),
            self.allocator,
        )
    }

    /// Build a [`SwitchCase`].
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

    /// Build a [`SwitchCase`], and store it in the memory arena.
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

    /// Build a [`LabeledStatement`].
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

    /// Build a [`LabeledStatement`], and store it in the memory arena.
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

    /// Build a [`ThrowStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_throw_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn throw_statement(self, span: Span, argument: Expression<'a>) -> ThrowStatement<'a> {
        ThrowStatement { span, argument }
    }

    /// Build a [`ThrowStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::throw_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - argument: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn alloc_throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, ThrowStatement<'a>> {
        Box::new_in(self.throw_statement(span, argument), self.allocator)
    }

    /// Build a [`TryStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_try_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block: Statements in the `try` block
    /// - handler: The `catch` clause, including the parameter and the block statement
    /// - finalizer: The `finally` clause
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

    /// Build a [`TryStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::try_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - block: Statements in the `try` block
    /// - handler: The `catch` clause, including the parameter and the block statement
    /// - finalizer: The `finally` clause
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

    /// Build a [`CatchClause`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_catch_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// - body: The statements run when an error is caught
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

    /// Build a [`CatchClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::catch_clause`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// - body: The statements run when an error is caught
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

    /// Build a [`CatchClause`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_catch_clause_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// - body: The statements run when an error is caught
    /// - scope_id
    #[inline]
    pub fn catch_clause_with_scope_id<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        CatchClause {
            span,
            param,
            body: body.into_in(self.allocator),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`CatchClause`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::catch_clause_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - param: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// - body: The statements run when an error is caught
    /// - scope_id
    #[inline]
    pub fn alloc_catch_clause_with_scope_id<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
    ) -> Box<'a, CatchClause<'a>>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Box::new_in(self.catch_clause_with_scope_id(span, param, body, scope_id), self.allocator)
    }

    /// Build a [`CatchParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_catch_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - pattern: The bound error
    #[inline]
    pub fn catch_parameter(self, span: Span, pattern: BindingPattern<'a>) -> CatchParameter<'a> {
        CatchParameter { span, pattern }
    }

    /// Build a [`CatchParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::catch_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - pattern: The bound error
    #[inline]
    pub fn alloc_catch_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> Box<'a, CatchParameter<'a>> {
        Box::new_in(self.catch_parameter(span, pattern), self.allocator)
    }

    /// Build a [`DebuggerStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_debugger_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn debugger_statement(self, span: Span) -> DebuggerStatement {
        DebuggerStatement { span }
    }

    /// Build a [`DebuggerStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::debugger_statement`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_debugger_statement(self, span: Span) -> Box<'a, DebuggerStatement> {
        Box::new_in(self.debugger_statement(span), self.allocator)
    }

    /// Build a [`BindingPattern`].
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

    /// Build a [`BindingPattern`], and store it in the memory arena.
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

    /// Build a [`BindingPatternKind::ObjectPattern`]
    ///
    /// This node contains an [`ObjectPattern`] that will be stored in the memory arena.
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

    /// Build a [`BindingPatternKind::ArrayPattern`]
    ///
    /// This node contains an [`ArrayPattern`] that will be stored in the memory arena.
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

    /// Build a [`BindingPatternKind::AssignmentPattern`]
    ///
    /// This node contains an [`AssignmentPattern`] that will be stored in the memory arena.
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

    /// Build an [`AssignmentPattern`].
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

    /// Build an [`AssignmentPattern`], and store it in the memory arena.
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

    /// Build an [`ObjectPattern`].
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

    /// Build an [`ObjectPattern`], and store it in the memory arena.
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

    /// Build a [`BindingProperty`].
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

    /// Build a [`BindingProperty`], and store it in the memory arena.
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

    /// Build an [`ArrayPattern`].
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

    /// Build an [`ArrayPattern`], and store it in the memory arena.
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

    /// Build a [`BindingRestElement`].
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

    /// Build a [`BindingRestElement`], and store it in the memory arena.
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

    /// Build a [`Function`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_function`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    #[inline]
    pub fn function<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Function {
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param: this_param.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            body: body.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Build a [`Function`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::function`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    #[inline]
    pub fn alloc_function<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> Box<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
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

    /// Build a [`Function`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_function_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    /// - scope_id
    #[inline]
    pub fn function_with_scope_id<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        scope_id: ScopeId,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Function {
            r#type,
            span,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param: this_param.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            body: body.into_in(self.allocator),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Function`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::function_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    /// - scope_id
    #[inline]
    pub fn alloc_function_with_scope_id<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        scope_id: ScopeId,
    ) -> Box<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Box::new_in(
            self.function_with_scope_id(
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
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`FormalParameters`].
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

    /// Build a [`FormalParameters`], and store it in the memory arena.
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

    /// Build a [`FormalParameter`].
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

    /// Build a [`FormalParameter`], and store it in the memory arena.
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

    /// Build a [`FunctionBody`].
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

    /// Build a [`FunctionBody`], and store it in the memory arena.
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

    /// Build an [`ArrowFunctionExpression`].
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

    /// Build an [`ArrowFunctionExpression`], and store it in the memory arena.
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

    /// Build an [`ArrowFunctionExpression`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_arrow_function_expression_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// - r#async
    /// - type_parameters
    /// - params
    /// - return_type
    /// - body: See `expression` for whether this arrow expression returns an expression.
    /// - scope_id
    #[inline]
    pub fn arrow_function_expression_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
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
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build an [`ArrowFunctionExpression`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::arrow_function_expression_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// - r#async
    /// - type_parameters
    /// - params
    /// - return_type
    /// - body: See `expression` for whether this arrow expression returns an expression.
    /// - scope_id
    #[inline]
    pub fn alloc_arrow_function_expression_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
    ) -> Box<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Box::new_in(
            self.arrow_function_expression_with_scope_id(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`YieldExpression`].
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

    /// Build a [`YieldExpression`], and store it in the memory arena.
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

    /// Build a [`Class`].
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

    /// Build a [`Class`], and store it in the memory arena.
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

    /// Build a [`Class`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_class_with_scope_id`] instead.
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
    /// - scope_id: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn class_with_scope_id<T1, T2, T3>(
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
        scope_id: ScopeId,
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
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Class`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::class_with_scope_id`] instead.
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
    /// - scope_id: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn alloc_class_with_scope_id<T1, T2, T3>(
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
        scope_id: ScopeId,
    ) -> Box<'a, Class<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Box::new_in(
            self.class_with_scope_id(
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
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`ClassBody`].
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

    /// Build a [`ClassBody`], and store it in the memory arena.
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

    /// Build a [`ClassElement::AccessorProperty`]
    ///
    /// This node contains an [`AccessorProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - definite: Property has a `!` after its key.
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn class_element_accessor_property<T1>(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        definite: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        ClassElement::AccessorProperty(self.alloc(self.accessor_property(
            r#type,
            span,
            decorators,
            key,
            value,
            computed,
            r#static,
            definite,
            type_annotation,
            accessibility,
        )))
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
    /// - r#static
    #[inline]
    pub fn class_element_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        ClassElement::TSIndexSignature(self.alloc(self.ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
        )))
    }

    /// Build a [`MethodDefinition`].
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

    /// Build a [`MethodDefinition`], and store it in the memory arena.
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

    /// Build a [`PropertyDefinition`].
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

    /// Build a [`PropertyDefinition`], and store it in the memory arena.
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

    /// Build a [`PrivateIdentifier`].
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

    /// Build a [`PrivateIdentifier`], and store it in the memory arena.
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

    /// Build a [`StaticBlock`].
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

    /// Build a [`StaticBlock`], and store it in the memory arena.
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

    /// Build a [`StaticBlock`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_static_block_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - scope_id
    #[inline]
    pub fn static_block_with_scope_id(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> StaticBlock<'a> {
        StaticBlock { span, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`StaticBlock`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::static_block_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - body
    /// - scope_id
    #[inline]
    pub fn alloc_static_block_with_scope_id(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Box<'a, StaticBlock<'a>> {
        Box::new_in(self.static_block_with_scope_id(span, body, scope_id), self.allocator)
    }

    /// Build a [`ModuleDeclaration::ImportDeclaration`]
    ///
    /// This node contains an [`ImportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - specifiers: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// - source
    /// - with_clause: Some(vec![]) for empty assertion
    /// - import_kind: `import type { foo } from 'bar'`
    #[inline]
    pub fn module_declaration_import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ModuleDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ImportDeclaration(self.alloc(self.import_declaration(
            span,
            specifiers,
            source,
            with_clause,
            import_kind,
        )))
    }

    /// Build a [`ModuleDeclaration::ExportAllDeclaration`]
    ///
    /// This node contains an [`ExportAllDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - exported: If this declaration is re-named
    /// - source
    /// - with_clause: Will be `Some(vec![])` for empty assertion
    /// - export_kind
    #[inline]
    pub fn module_declaration_export_all_declaration<T1>(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
    ) -> ModuleDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ExportAllDeclaration(self.alloc(self.export_all_declaration(
            span,
            exported,
            source,
            with_clause,
            export_kind,
        )))
    }

    /// Build a [`ModuleDeclaration::ExportDefaultDeclaration`]
    ///
    /// This node contains an [`ExportDefaultDeclaration`] that will be stored in the memory arena.
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

    /// Build a [`ModuleDeclaration::ExportNamedDeclaration`]
    ///
    /// This node contains an [`ExportNamedDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - declaration
    /// - specifiers
    /// - source
    /// - export_kind: `export type { foo }`
    /// - with_clause: Some(vec![]) for empty assertion
    #[inline]
    pub fn module_declaration_export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ModuleDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ExportNamedDeclaration(self.alloc(self.export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        )))
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

    /// Build an [`AccessorProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_accessor_property`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - definite: Property has a `!` after its key.
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn accessor_property<T1>(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        definite: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> AccessorProperty<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        AccessorProperty {
            r#type,
            span,
            decorators,
            key,
            value,
            computed,
            r#static,
            definite,
            type_annotation: type_annotation.into_in(self.allocator),
            accessibility,
        }
    }

    /// Build an [`AccessorProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::accessor_property`] instead.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - decorators: Decorators applied to the accessor property.
    /// - key: The expression used to declare the property.
    /// - value: Initialized value in the declaration, if present.
    /// - computed: Property was declared with a computed key
    /// - r#static: Property was declared with a `static` modifier
    /// - definite: Property has a `!` after its key.
    /// - type_annotation: Type annotation on the property.
    /// - accessibility: Accessibility modifier.
    #[inline]
    pub fn alloc_accessor_property<T1>(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        definite: bool,
        type_annotation: T1,
        accessibility: Option<TSAccessibility>,
    ) -> Box<'a, AccessorProperty<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.accessor_property(
                r#type,
                span,
                decorators,
                key,
                value,
                computed,
                r#static,
                definite,
                type_annotation,
                accessibility,
            ),
            self.allocator,
        )
    }

    /// Build an [`ImportExpression`].
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

    /// Build an [`ImportExpression`], and store it in the memory arena.
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

    /// Build an [`ImportDeclaration`].
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
    pub fn import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ImportDeclaration {
            span,
            specifiers,
            source,
            with_clause: with_clause.into_in(self.allocator),
            import_kind,
        }
    }

    /// Build an [`ImportDeclaration`], and store it in the memory arena.
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
    pub fn alloc_import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Box::new_in(
            self.import_declaration(span, specifiers, source, with_clause, import_kind),
            self.allocator,
        )
    }

    /// Build an [`ImportDeclarationSpecifier::ImportSpecifier`]
    ///
    /// This node contains an [`ImportSpecifier`] that will be stored in the memory arena.
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

    /// Build an [`ImportDeclarationSpecifier::ImportDefaultSpecifier`]
    ///
    /// This node contains an [`ImportDefaultSpecifier`] that will be stored in the memory arena.
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

    /// Build an [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`]
    ///
    /// This node contains an [`ImportNamespaceSpecifier`] that will be stored in the memory arena.
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

    /// Build an [`ImportSpecifier`].
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

    /// Build an [`ImportSpecifier`], and store it in the memory arena.
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

    /// Build an [`ImportDefaultSpecifier`].
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

    /// Build an [`ImportDefaultSpecifier`], and store it in the memory arena.
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

    /// Build an [`ImportNamespaceSpecifier`].
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

    /// Build an [`ImportNamespaceSpecifier`], and store it in the memory arena.
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

    /// Build a [`WithClause`].
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

    /// Build a [`WithClause`], and store it in the memory arena.
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

    /// Build an [`ImportAttribute`].
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

    /// Build an [`ImportAttribute`], and store it in the memory arena.
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

    /// Build an [`ImportAttributeKey::Identifier`]
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

    /// Build an [`ImportAttributeKey::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
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

    /// Build an [`ExportNamedDeclaration`].
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
    pub fn export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ExportNamedDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ExportNamedDeclaration {
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause: with_clause.into_in(self.allocator),
        }
    }

    /// Build an [`ExportNamedDeclaration`], and store it in the memory arena.
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
    pub fn alloc_export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> Box<'a, ExportNamedDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
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

    /// Build an [`ExportDefaultDeclaration`].
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

    /// Build an [`ExportDefaultDeclaration`], and store it in the memory arena.
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

    /// Build an [`ExportAllDeclaration`].
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
    pub fn export_all_declaration<T1>(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        ExportAllDeclaration {
            span,
            exported,
            source,
            with_clause: with_clause.into_in(self.allocator),
            export_kind,
        }
    }

    /// Build an [`ExportAllDeclaration`], and store it in the memory arena.
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
    pub fn alloc_export_all_declaration<T1>(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportAllDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Box::new_in(
            self.export_all_declaration(span, exported, source, with_clause, export_kind),
            self.allocator,
        )
    }

    /// Build an [`ExportSpecifier`].
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

    /// Build an [`ExportSpecifier`], and store it in the memory arena.
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

    /// Build an [`ExportDefaultDeclarationKind::FunctionDeclaration`]
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - r#type
    /// - span: The [`Span`] covering this node
    /// - id: The function identifier. [`None`] for anonymous function expressions.
    /// - generator: Is this a generator function?
    /// - r#async
    /// - declare
    /// - type_parameters
    /// - this_param: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// - params: Function parameters.
    /// - return_type: The TypeScript return type annotation.
    /// - body: The function body.
    #[inline]
    pub fn export_default_declaration_kind_function<T1, T2, T3, T4, T5>(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
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

    /// Build an [`ExportDefaultDeclarationKind::ClassDeclaration`]
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

    /// Build an [`ExportDefaultDeclarationKind::TSInterfaceDeclaration`]
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
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

    /// Build a [`ModuleExportName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn module_export_name_string_literal<A>(self, span: Span, value: A) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::StringLiteral(self.string_literal(span, value))
    }

    /// Build a [`TSThisParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_span
    /// - type_annotation: Type type the `this` keyword will have in the function
    #[inline]
    pub fn ts_this_parameter<T1>(
        self,
        span: Span,
        this_span: Span,
        type_annotation: T1,
    ) -> TSThisParameter<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSThisParameter {
            span,
            this_span,
            type_annotation: type_annotation.into_in(self.allocator),
        }
    }

    /// Build a [`TSThisParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - this_span
    /// - type_annotation: Type type the `this` keyword will have in the function
    #[inline]
    pub fn alloc_ts_this_parameter<T1>(
        self,
        span: Span,
        this_span: Span,
        type_annotation: T1,
    ) -> Box<'a, TSThisParameter<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(self.ts_this_parameter(span, this_span, type_annotation), self.allocator)
    }

    /// Build a [`TSEnumDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const: `true` for const enums
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

    /// Build a [`TSEnumDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const: `true` for const enums
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

    /// Build a [`TSEnumDeclaration`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_enum_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const: `true` for const enums
    /// - declare
    /// - scope_id
    #[inline]
    pub fn ts_enum_declaration_with_scope_id(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> TSEnumDeclaration<'a> {
        TSEnumDeclaration {
            span,
            id,
            members,
            r#const,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSEnumDeclaration`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_enum_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id
    /// - members
    /// - r#const: `true` for const enums
    /// - declare
    /// - scope_id
    #[inline]
    pub fn alloc_ts_enum_declaration_with_scope_id(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> Box<'a, TSEnumDeclaration<'a>> {
        Box::new_in(
            self.ts_enum_declaration_with_scope_id(span, id, members, r#const, declare, scope_id),
            self.allocator,
        )
    }

    /// Build a [`TSEnumMember`].
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

    /// Build a [`TSEnumMember`], and store it in the memory arena.
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
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
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

    /// Build a [`TSEnumMemberName::StaticStringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn ts_enum_member_name_string_literal<A>(self, span: Span, value: A) -> TSEnumMemberName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSEnumMemberName::StaticStringLiteral(self.alloc(self.string_literal(span, value)))
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

    /// Build a [`TSEnumMemberName::StaticNumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`TSTypeAnnotation`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// - span: starts at the `:` token and ends at the end of the type annotation
    /// - type_annotation: The actual type in the annotation
    #[inline]
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation { span, type_annotation }
    }

    /// Build a [`TSTypeAnnotation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// - span: starts at the `:` token and ends at the end of the type annotation
    /// - type_annotation: The actual type in the annotation
    #[inline]
    pub fn alloc_ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAnnotation<'a>> {
        Box::new_in(self.ts_type_annotation(span, type_annotation), self.allocator)
    }

    /// Build a [`TSLiteralType`].
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

    /// Build a [`TSLiteralType`], and store it in the memory arena.
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
    /// - span: Node location in source code
    /// - value: The boolean value itself
    #[inline]
    pub fn ts_literal_boolean_literal(self, span: Span, value: bool) -> TSLiteral<'a> {
        TSLiteral::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    /// Build a [`TSLiteral::NullLiteral`]
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn ts_literal_null_literal(self, span: Span) -> TSLiteral<'a> {
        TSLiteral::NullLiteral(self.alloc(self.null_literal(span)))
    }

    /// Build a [`TSLiteral::NumericLiteral`]
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The value of the number, converted into base 10
    /// - raw: The number as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`TSLiteral::BigIntLiteral`]
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - raw: The bigint as it appears in source code
    /// - base: The base representation used by the literal in source code
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

    /// Build a [`TSLiteral::RegExpLiteral`]
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - regex: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// - raw: The regular expression as it appears in source code
    #[inline]
    pub fn ts_literal_reg_exp_literal<S>(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: S,
    ) -> TSLiteral<'a>
    where
        S: IntoIn<'a, &'a str>,
    {
        TSLiteral::RegExpLiteral(self.alloc(self.reg_exp_literal(span, regex, raw)))
    }

    /// Build a [`TSLiteral::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
    #[inline]
    pub fn ts_literal_string_literal<A>(self, span: Span, value: A) -> TSLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSLiteral::StringLiteral(self.alloc(self.string_literal(span, value)))
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

    /// Build a [`TSLiteral::UnaryExpression`]
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
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

    /// Build a [`TSType::TSConditionalType`]
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type: The type before `extends` in the test expression.
    /// - extends_type: The type `check_type` is being tested against.
    /// - true_type: The type evaluated to if the test is true.
    /// - false_type: The type evaluated to if the test is false.
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

    /// Build a [`TSType::TSConstructorType`]
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_type_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSConstructorType(self.alloc(self.ts_constructor_type(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
        )))
    }

    /// Build a [`TSType::TSFunctionType`]
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters: Generic type parameters
    /// - this_param: `this` parameter
    /// - params: Function parameters. Akin to [`Function::params`].
    /// - return_type: Return type of the function.
    #[inline]
    pub fn ts_type_function_type<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSFunctionType(self.alloc(self.ts_function_type(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
        )))
    }

    /// Build a [`TSType::TSImportType`]
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of: `true` for `typeof import("foo")`
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_type_import_type<T1, T2>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: T1,
        type_parameters: T2,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSImportAttributes<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
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

    /// Build a [`TSType::TSInferType`]
    ///
    /// This node contains a [`TSInferType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: The type bound when the
    #[inline]
    pub fn ts_type_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSType::TSInferType(self.alloc(self.ts_infer_type(span, type_parameter)))
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

    /// Build a [`TSType::TSMappedType`]
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// - name_type
    /// - type_annotation
    /// - optional: Optional modifier on type annotation
    /// - readonly: Readonly modifier before keyed index signature
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

    /// Build a [`TSType::TSTemplateLiteralType`]
    ///
    /// This node contains a [`TSTemplateLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis: The string parts of the template literal.
    /// - types: The interpolated expressions in the template literal.
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

    /// Build a [`TSType::TSTypeOperatorType`]
    ///
    /// This node contains a [`TSTypeOperator`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation: The type being operated on
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

    /// Build a [`TSType::TSTypePredicate`]
    ///
    /// This node contains a [`TSTypePredicate`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name: The identifier the predicate operates on
    /// - asserts: Does this predicate include an `asserts` modifier?
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

    /// Build a [`TSType::TSUnionType`]
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types: The types in the union.
    #[inline]
    pub fn ts_type_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc(self.ts_union_type(span, types)))
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

    /// Build a [`TSConditionalType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type: The type before `extends` in the test expression.
    /// - extends_type: The type `check_type` is being tested against.
    /// - true_type: The type evaluated to if the test is true.
    /// - false_type: The type evaluated to if the test is false.
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

    /// Build a [`TSConditionalType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type: The type before `extends` in the test expression.
    /// - extends_type: The type `check_type` is being tested against.
    /// - true_type: The type evaluated to if the test is true.
    /// - false_type: The type evaluated to if the test is false.
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

    /// Build a [`TSConditionalType`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_conditional_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type: The type before `extends` in the test expression.
    /// - extends_type: The type `check_type` is being tested against.
    /// - true_type: The type evaluated to if the test is true.
    /// - false_type: The type evaluated to if the test is false.
    /// - scope_id
    #[inline]
    pub fn ts_conditional_type_with_scope_id(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
    ) -> TSConditionalType<'a> {
        TSConditionalType {
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConditionalType`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_conditional_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - check_type: The type before `extends` in the test expression.
    /// - extends_type: The type `check_type` is being tested against.
    /// - true_type: The type evaluated to if the test is true.
    /// - false_type: The type evaluated to if the test is false.
    /// - scope_id
    #[inline]
    pub fn alloc_ts_conditional_type_with_scope_id(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
    ) -> Box<'a, TSConditionalType<'a>> {
        Box::new_in(
            self.ts_conditional_type_with_scope_id(
                span,
                check_type,
                extends_type,
                true_type,
                false_type,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSUnionType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_union_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types: The types in the union.
    #[inline]
    pub fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSUnionType<'a> {
        TSUnionType { span, types }
    }

    /// Build a [`TSUnionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_union_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - types: The types in the union.
    #[inline]
    pub fn alloc_ts_union_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSUnionType<'a>> {
        Box::new_in(self.ts_union_type(span, types), self.allocator)
    }

    /// Build a [`TSIntersectionType`].
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

    /// Build a [`TSIntersectionType`], and store it in the memory arena.
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

    /// Build a [`TSParenthesizedType`].
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

    /// Build a [`TSParenthesizedType`], and store it in the memory arena.
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

    /// Build a [`TSTypeOperator`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation: The type being operated on
    #[inline]
    pub fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSTypeOperator<'a> {
        TSTypeOperator { span, operator, type_annotation }
    }

    /// Build a [`TSTypeOperator`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - operator
    /// - type_annotation: The type being operated on
    #[inline]
    pub fn alloc_ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeOperator<'a>> {
        Box::new_in(self.ts_type_operator(span, operator, type_annotation), self.allocator)
    }

    /// Build a [`TSArrayType`].
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

    /// Build a [`TSArrayType`], and store it in the memory arena.
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

    /// Build a [`TSIndexedAccessType`].
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

    /// Build a [`TSIndexedAccessType`], and store it in the memory arena.
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

    /// Build a [`TSTupleType`].
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

    /// Build a [`TSTupleType`], and store it in the memory arena.
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

    /// Build a [`TSNamedTupleMember`].
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

    /// Build a [`TSNamedTupleMember`], and store it in the memory arena.
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

    /// Build a [`TSOptionalType`].
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

    /// Build a [`TSOptionalType`], and store it in the memory arena.
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

    /// Build a [`TSRestType`].
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

    /// Build a [`TSRestType`], and store it in the memory arena.
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

    /// Build a [`TSAnyKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_any_keyword(self, span: Span) -> TSAnyKeyword {
        TSAnyKeyword { span }
    }

    /// Build a [`TSAnyKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_any_keyword(self, span: Span) -> Box<'a, TSAnyKeyword> {
        Box::new_in(self.ts_any_keyword(span), self.allocator)
    }

    /// Build a [`TSStringKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_string_keyword(self, span: Span) -> TSStringKeyword {
        TSStringKeyword { span }
    }

    /// Build a [`TSStringKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_string_keyword(self, span: Span) -> Box<'a, TSStringKeyword> {
        Box::new_in(self.ts_string_keyword(span), self.allocator)
    }

    /// Build a [`TSBooleanKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_boolean_keyword(self, span: Span) -> TSBooleanKeyword {
        TSBooleanKeyword { span }
    }

    /// Build a [`TSBooleanKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_boolean_keyword(self, span: Span) -> Box<'a, TSBooleanKeyword> {
        Box::new_in(self.ts_boolean_keyword(span), self.allocator)
    }

    /// Build a [`TSNumberKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_number_keyword(self, span: Span) -> TSNumberKeyword {
        TSNumberKeyword { span }
    }

    /// Build a [`TSNumberKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_number_keyword(self, span: Span) -> Box<'a, TSNumberKeyword> {
        Box::new_in(self.ts_number_keyword(span), self.allocator)
    }

    /// Build a [`TSNeverKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_never_keyword(self, span: Span) -> TSNeverKeyword {
        TSNeverKeyword { span }
    }

    /// Build a [`TSNeverKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_never_keyword(self, span: Span) -> Box<'a, TSNeverKeyword> {
        Box::new_in(self.ts_never_keyword(span), self.allocator)
    }

    /// Build a [`TSIntrinsicKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_intrinsic_keyword(self, span: Span) -> TSIntrinsicKeyword {
        TSIntrinsicKeyword { span }
    }

    /// Build a [`TSIntrinsicKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_intrinsic_keyword(self, span: Span) -> Box<'a, TSIntrinsicKeyword> {
        Box::new_in(self.ts_intrinsic_keyword(span), self.allocator)
    }

    /// Build a [`TSUnknownKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_unknown_keyword(self, span: Span) -> TSUnknownKeyword {
        TSUnknownKeyword { span }
    }

    /// Build a [`TSUnknownKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_unknown_keyword(self, span: Span) -> Box<'a, TSUnknownKeyword> {
        Box::new_in(self.ts_unknown_keyword(span), self.allocator)
    }

    /// Build a [`TSNullKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_null_keyword(self, span: Span) -> TSNullKeyword {
        TSNullKeyword { span }
    }

    /// Build a [`TSNullKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_null_keyword(self, span: Span) -> Box<'a, TSNullKeyword> {
        Box::new_in(self.ts_null_keyword(span), self.allocator)
    }

    /// Build a [`TSUndefinedKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_undefined_keyword(self, span: Span) -> TSUndefinedKeyword {
        TSUndefinedKeyword { span }
    }

    /// Build a [`TSUndefinedKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_undefined_keyword(self, span: Span) -> Box<'a, TSUndefinedKeyword> {
        Box::new_in(self.ts_undefined_keyword(span), self.allocator)
    }

    /// Build a [`TSVoidKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_void_keyword(self, span: Span) -> TSVoidKeyword {
        TSVoidKeyword { span }
    }

    /// Build a [`TSVoidKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_void_keyword(self, span: Span) -> Box<'a, TSVoidKeyword> {
        Box::new_in(self.ts_void_keyword(span), self.allocator)
    }

    /// Build a [`TSSymbolKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_symbol_keyword(self, span: Span) -> TSSymbolKeyword {
        TSSymbolKeyword { span }
    }

    /// Build a [`TSSymbolKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_symbol_keyword(self, span: Span) -> Box<'a, TSSymbolKeyword> {
        Box::new_in(self.ts_symbol_keyword(span), self.allocator)
    }

    /// Build a [`TSThisType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_this_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_this_type(self, span: Span) -> TSThisType {
        TSThisType { span }
    }

    /// Build a [`TSThisType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_this_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_this_type(self, span: Span) -> Box<'a, TSThisType> {
        Box::new_in(self.ts_this_type(span), self.allocator)
    }

    /// Build a [`TSObjectKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_object_keyword(self, span: Span) -> TSObjectKeyword {
        TSObjectKeyword { span }
    }

    /// Build a [`TSObjectKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_object_keyword(self, span: Span) -> Box<'a, TSObjectKeyword> {
        Box::new_in(self.ts_object_keyword(span), self.allocator)
    }

    /// Build a [`TSBigIntKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_big_int_keyword(self, span: Span) -> TSBigIntKeyword {
        TSBigIntKeyword { span }
    }

    /// Build a [`TSBigIntKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_ts_big_int_keyword(self, span: Span) -> Box<'a, TSBigIntKeyword> {
        Box::new_in(self.ts_big_int_keyword(span), self.allocator)
    }

    /// Build a [`TSTypeReference`].
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

    /// Build a [`TSTypeReference`], and store it in the memory arena.
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
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
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

    /// Build a [`TSQualifiedName`].
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

    /// Build a [`TSQualifiedName`], and store it in the memory arena.
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

    /// Build a [`TSTypeParameterInstantiation`].
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

    /// Build a [`TSTypeParameterInstantiation`], and store it in the memory arena.
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

    /// Build a [`TSTypeParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// - constraint: Constrains what types can be passed to the type parameter.
    /// - default: Default value of the type parameter if no type is provided when using the type.
    /// - r#in: Was an `in` modifier keyword present?
    /// - out: Was an `out` modifier keyword present?
    /// - r#const: Was a `const` modifier keyword present?
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

    /// Build a [`TSTypeParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// - constraint: Constrains what types can be passed to the type parameter.
    /// - default: Default value of the type parameter if no type is provided when using the type.
    /// - r#in: Was an `in` modifier keyword present?
    /// - out: Was an `out` modifier keyword present?
    /// - r#const: Was a `const` modifier keyword present?
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

    /// Build a [`TSTypeParameterDeclaration`].
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

    /// Build a [`TSTypeParameterDeclaration`], and store it in the memory arena.
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

    /// Build a [`TSTypeAliasDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
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

    /// Build a [`TSTypeAliasDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
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

    /// Build a [`TSTypeAliasDeclaration`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_alias_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// - type_parameters
    /// - type_annotation
    /// - declare
    /// - scope_id
    #[inline]
    pub fn ts_type_alias_declaration_with_scope_id<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
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
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSTypeAliasDeclaration`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_alias_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// - type_parameters
    /// - type_annotation
    /// - declare
    /// - scope_id
    #[inline]
    pub fn alloc_ts_type_alias_declaration_with_scope_id<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> Box<'a, TSTypeAliasDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            self.ts_type_alias_declaration_with_scope_id(
                span,
                id,
                type_parameters,
                type_annotation,
                declare,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSClassImplements`].
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

    /// Build a [`TSClassImplements`], and store it in the memory arena.
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

    /// Build a [`TSInterfaceDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
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

    /// Build a [`TSInterfaceDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
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

    /// Build a [`TSInterfaceDeclaration`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_interface_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
    /// - scope_id
    #[inline]
    pub fn ts_interface_declaration_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
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
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSInterfaceDeclaration`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_interface_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The identifier (name) of the interface.
    /// - extends: Other interfaces/types this interface extends.
    /// - type_parameters: Type parameters that get bound to the interface.
    /// - body
    /// - declare: `true` for `declare interface Foo {}`
    /// - scope_id
    #[inline]
    pub fn alloc_ts_interface_declaration_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        extends: Option<Vec<'a, TSInterfaceHeritage<'a>>>,
        type_parameters: T1,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
    ) -> Box<'a, TSInterfaceDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Box::new_in(
            self.ts_interface_declaration_with_scope_id(
                span,
                id,
                extends,
                type_parameters,
                body,
                declare,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSInterfaceBody`].
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

    /// Build a [`TSInterfaceBody`], and store it in the memory arena.
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

    /// Build a [`TSPropertySignature`].
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

    /// Build a [`TSPropertySignature`], and store it in the memory arena.
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
    /// - r#static
    #[inline]
    pub fn ts_signature_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSSignature::TSIndexSignature(self.alloc(self.ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
        )))
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

    /// Build a [`TSSignature::TSCallSignatureDeclaration`]
    ///
    /// This node contains a [`TSCallSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_signature_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSCallSignatureDeclaration(self.alloc(self.ts_call_signature_declaration(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
        )))
    }

    /// Build a [`TSSignature::TSConstructSignatureDeclaration`]
    ///
    /// This node contains a [`TSConstructSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_signature_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSConstructSignatureDeclaration(self.alloc(
            self.ts_construct_signature_declaration(span, type_parameters, params, return_type),
        ))
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
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_signature_method_signature<T1, T2, T3, T4>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSMethodSignature(self.alloc(self.ts_method_signature(
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters,
            this_param,
            params,
            return_type,
        )))
    }

    /// Build a [`TSIndexSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    /// - r#static
    #[inline]
    pub fn ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> TSIndexSignature<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSIndexSignature {
            span,
            parameters,
            type_annotation: type_annotation.into_in(self.allocator),
            readonly,
            r#static,
        }
    }

    /// Build a [`TSIndexSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameters
    /// - type_annotation
    /// - readonly
    /// - r#static
    #[inline]
    pub fn alloc_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> Box<'a, TSIndexSignature<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static),
            self.allocator,
        )
    }

    /// Build a [`TSCallSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
    ) -> TSCallSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSCallSignatureDeclaration {
            span,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param,
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
        }
    }

    /// Build a [`TSCallSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn alloc_ts_call_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: Option<TSThisParameter<'a>>,
        params: T2,
        return_type: T3,
    ) -> Box<'a, TSCallSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_call_signature_declaration(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSMethodSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_method_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_method_signature<T1, T2, T3, T4>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSMethodSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param: this_param.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSMethodSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_method_signature`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    #[inline]
    pub fn alloc_ts_method_signature<T1, T2, T3, T4>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> Box<'a, TSMethodSignature<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_method_signature(
                span,
                key,
                computed,
                optional,
                kind,
                type_parameters,
                this_param,
                params,
                return_type,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSMethodSignature`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_method_signature_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    /// - scope_id
    #[inline]
    pub fn ts_method_signature_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> TSMethodSignature<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSMethodSignature {
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param: this_param.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSMethodSignature`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_method_signature_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - key
    /// - computed
    /// - optional
    /// - kind
    /// - type_parameters
    /// - this_param
    /// - params
    /// - return_type
    /// - scope_id
    #[inline]
    pub fn alloc_ts_method_signature_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> Box<'a, TSMethodSignature<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_method_signature_with_scope_id(
                span,
                key,
                computed,
                optional,
                kind,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSConstructSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSConstructSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSConstructSignatureDeclaration {
            span,
            type_parameters: type_parameters.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn alloc_ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> Box<'a, TSConstructSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_construct_signature_declaration(span, type_parameters, params, return_type),
            self.allocator,
        )
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_construct_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - params
    /// - return_type
    /// - scope_id
    #[inline]
    pub fn ts_construct_signature_declaration_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> TSConstructSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        TSConstructSignatureDeclaration {
            span,
            type_parameters: type_parameters.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_construct_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters
    /// - params
    /// - return_type
    /// - scope_id
    #[inline]
    pub fn alloc_ts_construct_signature_declaration_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> Box<'a, TSConstructSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            self.ts_construct_signature_declaration_with_scope_id(
                span,
                type_parameters,
                params,
                return_type,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSIndexSignatureName`].
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

    /// Build a [`TSIndexSignatureName`], and store it in the memory arena.
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

    /// Build a [`TSInterfaceHeritage`].
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

    /// Build a [`TSInterfaceHeritage`], and store it in the memory arena.
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

    /// Build a [`TSTypePredicate`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name: The identifier the predicate operates on
    /// - asserts: Does this predicate include an `asserts` modifier?
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

    /// Build a [`TSTypePredicate`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - parameter_name: The identifier the predicate operates on
    /// - asserts: Does this predicate include an `asserts` modifier?
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
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
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

    /// Build a [`TSTypePredicateName::This`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn ts_type_predicate_name_this_type(self, span: Span) -> TSTypePredicateName<'a> {
        TSTypePredicateName::This(self.ts_this_type(span))
    }

    /// Build a [`TSModuleDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
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

    /// Build a [`TSModuleDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
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

    /// Build a [`TSModuleDeclaration`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_module_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
    /// - declare
    /// - scope_id
    #[inline]
    pub fn ts_module_declaration_with_scope_id(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
    ) -> TSModuleDeclaration<'a> {
        TSModuleDeclaration { span, id, body, kind, declare, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`TSModuleDeclaration`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_module_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
    /// - declare
    /// - scope_id
    #[inline]
    pub fn alloc_ts_module_declaration_with_scope_id(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        Box::new_in(
            self.ts_module_declaration_with_scope_id(span, id, body, kind, declare, scope_id),
            self.allocator,
        )
    }

    /// Build a [`TSModuleDeclarationName::Identifier`]
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The identifier name being bound.
    #[inline]
    pub fn ts_module_declaration_name_binding_identifier<A>(
        self,
        span: Span,
        name: A,
    ) -> TSModuleDeclarationName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSModuleDeclarationName::Identifier(self.binding_identifier(span, name))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
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

    /// Build a [`TSModuleDeclarationBody::TSModuleDeclaration`]
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - id: The name of the module/namespace being declared.
    /// - body
    /// - kind: The keyword used to define this module declaration.
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

    /// Build a [`TSModuleBlock`].
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

    /// Build a [`TSModuleBlock`], and store it in the memory arena.
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

    /// Build a [`TSTypeLiteral`].
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

    /// Build a [`TSTypeLiteral`], and store it in the memory arena.
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

    /// Build a [`TSInferType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: The type bound when the
    #[inline]
    pub fn ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSInferType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSInferType { span, type_parameter: type_parameter.into_in(self.allocator) }
    }

    /// Build a [`TSInferType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: The type bound when the
    #[inline]
    pub fn alloc_ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> Box<'a, TSInferType<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Box::new_in(self.ts_infer_type(span, type_parameter), self.allocator)
    }

    /// Build a [`TSTypeQuery`].
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

    /// Build a [`TSTypeQuery`], and store it in the memory arena.
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
    /// - is_type_of: `true` for `typeof import("foo")`
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_type_query_expr_name_import_type<T1, T2>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: T1,
        type_parameters: T2,
    ) -> TSTypeQueryExprName<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSImportAttributes<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
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

    /// Build a [`TSImportType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_import_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of: `true` for `typeof import("foo")`
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn ts_import_type<T1, T2>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: T1,
        type_parameters: T2,
    ) -> TSImportType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSImportAttributes<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSImportType {
            span,
            is_type_of,
            parameter,
            qualifier,
            attributes: attributes.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
        }
    }

    /// Build a [`TSImportType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_import_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - is_type_of: `true` for `typeof import("foo")`
    /// - parameter
    /// - qualifier
    /// - attributes
    /// - type_parameters
    #[inline]
    pub fn alloc_ts_import_type<T1, T2>(
        self,
        span: Span,
        is_type_of: bool,
        parameter: TSType<'a>,
        qualifier: Option<TSTypeName<'a>>,
        attributes: T1,
        type_parameters: T2,
    ) -> Box<'a, TSImportType<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSImportAttributes<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
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

    /// Build a [`TSImportAttributes`].
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

    /// Build a [`TSImportAttributes`], and store it in the memory arena.
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

    /// Build a [`TSImportAttribute`].
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

    /// Build a [`TSImportAttribute`], and store it in the memory arena.
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

    /// Build a [`TSImportAttributeName::StringLiteral`]
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
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

    /// Build a [`TSFunctionType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_function_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters: Generic type parameters
    /// - this_param: `this` parameter
    /// - params: Function parameters. Akin to [`Function::params`].
    /// - return_type: Return type of the function.
    #[inline]
    pub fn ts_function_type<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSFunctionType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSFunctionType {
            span,
            type_parameters: type_parameters.into_in(self.allocator),
            this_param: this_param.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
        }
    }

    /// Build a [`TSFunctionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_function_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameters: Generic type parameters
    /// - this_param: `this` parameter
    /// - params: Function parameters. Akin to [`Function::params`].
    /// - return_type: Return type of the function.
    #[inline]
    pub fn alloc_ts_function_type<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> Box<'a, TSFunctionType<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            self.ts_function_type(span, type_parameters, this_param, params, return_type),
            self.allocator,
        )
    }

    /// Build a [`TSConstructorType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn ts_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSConstructorType<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        TSConstructorType {
            span,
            r#abstract,
            type_parameters: type_parameters.into_in(self.allocator),
            params: params.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
        }
    }

    /// Build a [`TSConstructorType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - r#abstract
    /// - type_parameters
    /// - params
    /// - return_type
    #[inline]
    pub fn alloc_ts_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> Box<'a, TSConstructorType<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            self.ts_constructor_type(span, r#abstract, type_parameters, params, return_type),
            self.allocator,
        )
    }

    /// Build a [`TSMappedType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// - name_type
    /// - type_annotation
    /// - optional: Optional modifier on type annotation
    /// - readonly: Readonly modifier before keyed index signature
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

    /// Build a [`TSMappedType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// - name_type
    /// - type_annotation
    /// - optional: Optional modifier on type annotation
    /// - readonly: Readonly modifier before keyed index signature
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

    /// Build a [`TSMappedType`] with `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_mapped_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// - name_type
    /// - type_annotation
    /// - optional: Optional modifier on type annotation
    /// - readonly: Readonly modifier before keyed index signature
    /// - scope_id
    #[inline]
    pub fn ts_mapped_type_with_scope_id<T1>(
        self,
        span: Span,
        type_parameter: T1,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
        scope_id: ScopeId,
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
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSMappedType`] with `ScopeId`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_mapped_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - type_parameter: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// - name_type
    /// - type_annotation
    /// - optional: Optional modifier on type annotation
    /// - readonly: Readonly modifier before keyed index signature
    /// - scope_id
    #[inline]
    pub fn alloc_ts_mapped_type_with_scope_id<T1>(
        self,
        span: Span,
        type_parameter: T1,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: TSMappedTypeModifierOperator,
        readonly: TSMappedTypeModifierOperator,
        scope_id: ScopeId,
    ) -> Box<'a, TSMappedType<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Box::new_in(
            self.ts_mapped_type_with_scope_id(
                span,
                type_parameter,
                name_type,
                type_annotation,
                optional,
                readonly,
                scope_id,
            ),
            self.allocator,
        )
    }

    /// Build a [`TSTemplateLiteralType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis: The string parts of the template literal.
    /// - types: The interpolated expressions in the template literal.
    #[inline]
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSTemplateLiteralType<'a> {
        TSTemplateLiteralType { span, quasis, types }
    }

    /// Build a [`TSTemplateLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - quasis: The string parts of the template literal.
    /// - types: The interpolated expressions in the template literal.
    #[inline]
    pub fn alloc_ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTemplateLiteralType<'a>> {
        Box::new_in(self.ts_template_literal_type(span, quasis, types), self.allocator)
    }

    /// Build a [`TSAsExpression`].
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

    /// Build a [`TSAsExpression`], and store it in the memory arena.
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

    /// Build a [`TSSatisfiesExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The value expression being constrained.
    /// - type_annotation: The type `expression` must satisfy.
    #[inline]
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSSatisfiesExpression<'a> {
        TSSatisfiesExpression { span, expression, type_annotation }
    }

    /// Build a [`TSSatisfiesExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - expression: The value expression being constrained.
    /// - type_annotation: The type `expression` must satisfy.
    #[inline]
    pub fn alloc_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSSatisfiesExpression<'a>> {
        Box::new_in(self.ts_satisfies_expression(span, expression, type_annotation), self.allocator)
    }

    /// Build a [`TSTypeAssertion`].
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

    /// Build a [`TSTypeAssertion`], and store it in the memory arena.
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

    /// Build a [`TSImportEqualsDeclaration`].
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

    /// Build a [`TSImportEqualsDeclaration`], and store it in the memory arena.
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

    /// Build a [`TSExternalModuleReference`].
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

    /// Build a [`TSExternalModuleReference`], and store it in the memory arena.
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

    /// Build a [`TSNonNullExpression`].
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

    /// Build a [`TSNonNullExpression`], and store it in the memory arena.
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

    /// Build a [`Decorator`].
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

    /// Build a [`Decorator`], and store it in the memory arena.
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

    /// Build a [`TSExportAssignment`].
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

    /// Build a [`TSExportAssignment`], and store it in the memory arena.
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

    /// Build a [`TSNamespaceExportDeclaration`].
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

    /// Build a [`TSNamespaceExportDeclaration`], and store it in the memory arena.
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

    /// Build a [`TSInstantiationExpression`].
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

    /// Build a [`TSInstantiationExpression`], and store it in the memory arena.
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

    /// Build a [`JSDocNullableType`].
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

    /// Build a [`JSDocNullableType`], and store it in the memory arena.
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

    /// Build a [`JSDocNonNullableType`].
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

    /// Build a [`JSDocNonNullableType`], and store it in the memory arena.
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

    /// Build a [`JSDocUnknownType`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn js_doc_unknown_type(self, span: Span) -> JSDocUnknownType {
        JSDocUnknownType { span }
    }

    /// Build a [`JSDocUnknownType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn alloc_js_doc_unknown_type(self, span: Span) -> Box<'a, JSDocUnknownType> {
        Box::new_in(self.js_doc_unknown_type(span), self.allocator)
    }

    /// Build a [`JSXElement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXOpeningElement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - self_closing: Is this tag self-closing?
    /// - name: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
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

    /// Build a [`JSXOpeningElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - self_closing: Is this tag self-closing?
    /// - name: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
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

    /// Build a [`JSXClosingElement`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - name: The tag name, e.g. `Foo` in `</Foo>`.
    #[inline]
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { span, name }
    }

    /// Build a [`JSXClosingElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - name: The tag name, e.g. `Foo` in `</Foo>`.
    #[inline]
    pub fn alloc_jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        Box::new_in(self.jsx_closing_element(span, name), self.allocator)
    }

    /// Build a [`JSXFragment`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXFragment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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
    /// - span: Node location in source code
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_element_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXElementName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXElementName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    /// Build a [`JSXElementName::IdentifierReference`]
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn jsx_element_name_identifier_reference<A>(self, span: Span, name: A) -> JSXElementName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXElementName::IdentifierReference(self.alloc(self.identifier_reference(span, name)))
    }

    /// Build a [`JSXElementName::NamespacedName`]
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXElementName::MemberExpression`]
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXElementName::ThisExpression`]
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn jsx_element_name_this_expression(self, span: Span) -> JSXElementName<'a> {
        JSXElementName::ThisExpression(self.alloc(self.this_expression(span)))
    }

    /// Build a [`JSXNamespacedName`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXNamespacedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXMemberExpressionObject::IdentifierReference`]
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    /// - name: The name of the identifier being referenced.
    #[inline]
    pub fn jsx_member_expression_object_identifier_reference<A>(
        self,
        span: Span,
        name: A,
    ) -> JSXMemberExpressionObject<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXMemberExpressionObject::IdentifierReference(
            self.alloc(self.identifier_reference(span, name)),
        )
    }

    /// Build a [`JSXMemberExpressionObject::MemberExpression`]
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXMemberExpressionObject::ThisExpression`]
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: The [`Span`] covering this node
    #[inline]
    pub fn jsx_member_expression_object_this_expression(
        self,
        span: Span,
    ) -> JSXMemberExpressionObject<'a> {
        JSXMemberExpressionObject::ThisExpression(self.alloc(self.this_expression(span)))
    }

    /// Build a [`JSXExpressionContainer`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - expression: The expression inside the container.
    #[inline]
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { span, expression }
    }

    /// Build a [`JSXExpressionContainer`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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
    /// - span: Node location in source code
    #[inline]
    pub fn jsx_expression_jsx_empty_expression(self, span: Span) -> JSXExpression<'a> {
        JSXExpression::EmptyExpression(self.jsx_empty_expression(span))
    }

    /// Build a [`JSXEmptyExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }

    /// Build a [`JSXEmptyExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    #[inline]
    pub fn alloc_jsx_empty_expression(self, span: Span) -> Box<'a, JSXEmptyExpression> {
        Box::new_in(self.jsx_empty_expression(span), self.allocator)
    }

    /// Build a [`JSXAttributeItem::Attribute`]
    ///
    /// This node contains a [`JSXAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXAttributeItem::SpreadAttribute`]
    ///
    /// This node contains a [`JSXSpreadAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - argument: The expression being spread.
    #[inline]
    pub fn jsx_attribute_item_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::SpreadAttribute(self.alloc(self.jsx_spread_attribute(span, argument)))
    }

    /// Build a [`JSXAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXSpreadAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - argument: The expression being spread.
    #[inline]
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXSpreadAttribute<'a> {
        JSXSpreadAttribute { span, argument }
    }

    /// Build a [`JSXSpreadAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - argument: The expression being spread.
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
    /// - span: Node location in source code
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_attribute_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXAttributeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXAttributeName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    /// Build a [`JSXAttributeName::NamespacedName`]
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXAttributeValue::StringLiteral`]
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The string as it appears in source code
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

    /// Build a [`JSXAttributeValue::ExpressionContainer`]
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXAttributeValue::Element`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXAttributeValue::Fragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - name: The name of the identifier.
    #[inline]
    pub fn jsx_identifier<A>(self, span: Span, name: A) -> JSXIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXIdentifier { span, name: name.into_in(self.allocator) }
    }

    /// Build a [`JSXIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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
    /// - span: Node location in source code
    /// - value: The text content.
    #[inline]
    pub fn jsx_child_jsx_text<A>(self, span: Span, value: A) -> JSXChild<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXChild::Text(self.alloc(self.jsx_text(span, value)))
    }

    /// Build a [`JSXChild::Element`]
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXChild::Fragment`]
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
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

    /// Build a [`JSXChild::ExpressionContainer`]
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - expression: The expression inside the container.
    #[inline]
    pub fn jsx_child_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::ExpressionContainer(self.alloc(self.jsx_expression_container(span, expression)))
    }

    /// Build a [`JSXChild::Spread`]
    ///
    /// This node contains a [`JSXSpreadChild`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - expression: The expression being spread.
    #[inline]
    pub fn jsx_child_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::Spread(self.alloc(self.jsx_spread_child(span, expression)))
    }

    /// Build a [`JSXSpreadChild`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - expression: The expression being spread.
    #[inline]
    pub fn jsx_spread_child(self, span: Span, expression: Expression<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expression }
    }

    /// Build a [`JSXSpreadChild`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - expression: The expression being spread.
    #[inline]
    pub fn alloc_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        Box::new_in(self.jsx_spread_child(span, expression), self.allocator)
    }

    /// Build a [`JSXText`].
    ///
    /// If you want the built node to be allocated in the memory arena, use [`AstBuilder::alloc_jsx_text`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The text content.
    #[inline]
    pub fn jsx_text<A>(self, span: Span, value: A) -> JSXText<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXText { span, value: value.into_in(self.allocator) }
    }

    /// Build a [`JSXText`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::jsx_text`] instead.
    ///
    /// ## Parameters
    /// - span: Node location in source code
    /// - value: The text content.
    #[inline]
    pub fn alloc_jsx_text<A>(self, span: Span, value: A) -> Box<'a, JSXText<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Box::new_in(self.jsx_text(span, value), self.allocator)
    }
}
