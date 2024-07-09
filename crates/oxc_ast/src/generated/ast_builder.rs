// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_codegen/src/generators/ast_builder.rs`

#![allow(
    clippy::default_trait_access,
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools
)]

use oxc_allocator::{Allocator, Box, IntoIn, Vec};
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{
    number::{BigintBase, NumberBase},
    operator::{
        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
    },
};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// AST builder for creating AST nodes
#[derive(Clone, Copy)]
pub struct AstBuilder<'a> {
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    #[inline]
    pub fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { span, value }
    }

    #[inline]
    pub fn alloc_boolean_literal(self, span: Span, value: bool) -> Box<'a, BooleanLiteral> {
        self.boolean_literal(span, value).into_in(self.allocator)
    }

    #[inline]
    pub fn null_literal(self, span: Span) -> NullLiteral {
        NullLiteral { span }
    }

    #[inline]
    pub fn alloc_null_literal(self, span: Span) -> Box<'a, NullLiteral> {
        self.null_literal(span).into_in(self.allocator)
    }

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
        self.numeric_literal(span, value, raw, base).into_in(self.allocator)
    }

    #[inline]
    pub fn big_int_literal<A>(self, span: Span, raw: A, base: BigintBase) -> BigIntLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        BigIntLiteral { span, raw: raw.into_in(self.allocator), base }
    }

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
        self.big_int_literal(span, raw, base).into_in(self.allocator)
    }

    #[inline]
    pub fn reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral { span, value, regex }
    }

    #[inline]
    pub fn alloc_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> Box<'a, RegExpLiteral<'a>> {
        self.reg_exp_literal(span, value, regex).into_in(self.allocator)
    }

    #[inline]
    pub fn string_literal<A>(self, span: Span, value: A) -> StringLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        StringLiteral { span, value: value.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_string_literal<A>(self, span: Span, value: A) -> Box<'a, StringLiteral<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.string_literal(span, value).into_in(self.allocator)
    }

    #[inline]
    pub fn program(
        self,
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program { span, source_type, directives, hashbang, body, scope_id: Default::default() }
    }

    #[inline]
    pub fn alloc_program(
        self,
        span: Span,
        source_type: SourceType,
        directives: Vec<'a, Directive<'a>>,
        hashbang: Option<Hashbang<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, Program<'a>> {
        self.program(span, source_type, directives, hashbang, body).into_in(self.allocator)
    }

    #[inline]
    pub fn expression_boolean_literal(self, span: Span, value: bool) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    #[inline]
    pub fn expression_from_boolean_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BooleanLiteral>>,
    {
        Expression::BooleanLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_null_literal(self, span: Span) -> Expression<'a> {
        Expression::NullLiteral(self.alloc(self.null_literal(span)))
    }

    #[inline]
    pub fn expression_from_null_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NullLiteral>>,
    {
        Expression::NullLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_numeric_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NumericLiteral<'a>>>,
    {
        Expression::NumericLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_big_int_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BigIntLiteral<'a>>>,
    {
        Expression::BigIntLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc(self.reg_exp_literal(span, value, regex)))
    }

    #[inline]
    pub fn expression_from_reg_exp_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, RegExpLiteral<'a>>>,
    {
        Expression::RegExpLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_string_literal<A>(self, span: Span, value: A) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    #[inline]
    pub fn expression_from_string_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        Expression::StringLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc(self.template_literal(span, quasis, expressions)))
    }

    #[inline]
    pub fn expression_from_template_literal<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TemplateLiteral<'a>>>,
    {
        Expression::TemplateLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_identifier_reference<A>(self, span: Span, name: A) -> Expression<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Expression::Identifier(self.alloc(self.identifier_reference(span, name)))
    }

    #[inline]
    pub fn expression_from_identifier_reference<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierReference<'a>>>,
    {
        Expression::Identifier(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Expression<'a> {
        Expression::MetaProperty(self.alloc(self.meta_property(span, meta, property)))
    }

    #[inline]
    pub fn expression_from_meta_property<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, MetaProperty<'a>>>,
    {
        Expression::MetaProperty(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_super(self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc(self.super_(span)))
    }

    #[inline]
    pub fn expression_from_super<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Super>>,
    {
        Expression::Super(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_array<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ArrayExpression<'a>>>,
    {
        Expression::ArrayExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_arrow_function<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: T1,
        body: T2,
        type_parameters: T3,
        return_type: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Expression::ArrowFunctionExpression(self.alloc(self.arrow_function_expression(
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        )))
    }

    #[inline]
    pub fn expression_from_arrow_function<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ArrowFunctionExpression<'a>>>,
    {
        Expression::ArrowFunctionExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_assignment<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentExpression<'a>>>,
    {
        Expression::AssignmentExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_await(self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc(self.await_expression(span, argument)))
    }

    #[inline]
    pub fn expression_from_await<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, AwaitExpression<'a>>>,
    {
        Expression::AwaitExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_binary<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, BinaryExpression<'a>>>,
    {
        Expression::BinaryExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_call<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, CallExpression<'a>>>,
    {
        Expression::CallExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_chain(self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc(self.chain_expression(span, expression)))
    }

    #[inline]
    pub fn expression_from_chain<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ChainExpression<'a>>>,
    {
        Expression::ChainExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: T1,
        type_parameters: T2,
        super_type_parameters: T3,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Box<'a, ClassBody<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::ClassExpression(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            declare,
        )))
    }

    #[inline]
    pub fn expression_from_class<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Class<'a>>>,
    {
        Expression::ClassExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_conditional<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ConditionalExpression<'a>>>,
    {
        Expression::ConditionalExpression(inner.into_in(self.allocator))
    }

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
        body: T3,
        return_type: T4,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
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
            body,
            return_type,
        )))
    }

    #[inline]
    pub fn expression_from_function<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Expression::FunctionExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_import(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc(self.import_expression(span, source, arguments)))
    }

    #[inline]
    pub fn expression_from_import<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ImportExpression<'a>>>,
    {
        Expression::ImportExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_logical<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, LogicalExpression<'a>>>,
    {
        Expression::LogicalExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_new<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, NewExpression<'a>>>,
    {
        Expression::NewExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_object<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectExpression<'a>>>,
    {
        Expression::ObjectExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_parenthesized<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ParenthesizedExpression<'a>>>,
    {
        Expression::ParenthesizedExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_sequence(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc(self.sequence_expression(span, expressions)))
    }

    #[inline]
    pub fn expression_from_sequence<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, SequenceExpression<'a>>>,
    {
        Expression::SequenceExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_tagged_template<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TaggedTemplateExpression<'a>>>,
    {
        Expression::TaggedTemplateExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_this(self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc(self.this_expression(span)))
    }

    #[inline]
    pub fn expression_from_this<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, ThisExpression>>,
    {
        Expression::ThisExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_unary(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc(self.unary_expression(span, operator, argument)))
    }

    #[inline]
    pub fn expression_from_unary<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, UnaryExpression<'a>>>,
    {
        Expression::UnaryExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_update<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, UpdateExpression<'a>>>,
    {
        Expression::UpdateExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_yield(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc(self.yield_expression(span, delegate, argument)))
    }

    #[inline]
    pub fn expression_from_yield<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, YieldExpression<'a>>>,
    {
        Expression::YieldExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_private_in<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, PrivateInExpression<'a>>>,
    {
        Expression::PrivateInExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_jsx_element<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        Expression::JSXElement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_jsx_fragment<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        Expression::JSXFragment(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_ts_as<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSAsExpression<'a>>>,
    {
        Expression::TSAsExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_ts_satisfies<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSSatisfiesExpression<'a>>>,
    {
        Expression::TSSatisfiesExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn expression_from_ts_type_assertion<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeAssertion<'a>>>,
    {
        Expression::TSTypeAssertion(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn expression_ts_non_null(self, span: Span, expression: Expression<'a>) -> Expression<'a> {
        Expression::TSNonNullExpression(self.alloc(self.ts_non_null_expression(span, expression)))
    }

    #[inline]
    pub fn expression_from_ts_non_null<T>(self, inner: T) -> Expression<'a>
    where
        T: IntoIn<'a, Box<'a, TSNonNullExpression<'a>>>,
    {
        Expression::TSNonNullExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn identifier_name<A>(self, span: Span, name: A) -> IdentifierName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        IdentifierName { span, name: name.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_identifier_name<A>(self, span: Span, name: A) -> Box<'a, IdentifierName<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.identifier_name(span, name).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_identifier_reference<A>(
        self,
        span: Span,
        name: A,
    ) -> Box<'a, IdentifierReference<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.identifier_reference(span, name).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_binding_identifier<A>(self, span: Span, name: A) -> Box<'a, BindingIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.binding_identifier(span, name).into_in(self.allocator)
    }

    #[inline]
    pub fn label_identifier<A>(self, span: Span, name: A) -> LabelIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        LabelIdentifier { span, name: name.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_label_identifier<A>(self, span: Span, name: A) -> Box<'a, LabelIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.label_identifier(span, name).into_in(self.allocator)
    }

    #[inline]
    pub fn this_expression(self, span: Span) -> ThisExpression {
        ThisExpression { span }
    }

    #[inline]
    pub fn alloc_this_expression(self, span: Span) -> Box<'a, ThisExpression> {
        self.this_expression(span).into_in(self.allocator)
    }

    #[inline]
    pub fn array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> ArrayExpression<'a> {
        ArrayExpression { span, elements, trailing_comma }
    }

    #[inline]
    pub fn alloc_array_expression(
        self,
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayExpression<'a>> {
        self.array_expression(span, elements, trailing_comma).into_in(self.allocator)
    }

    #[inline]
    pub fn array_expression_element_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::SpreadElement(self.alloc(self.spread_element(span, argument)))
    }

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

    #[inline]
    pub fn array_expression_element_elision(self, span: Span) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::Elision(self.elision(span))
    }

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

    #[inline]
    pub fn elision(self, span: Span) -> Elision {
        Elision { span }
    }

    #[inline]
    pub fn alloc_elision(self, span: Span) -> Box<'a, Elision> {
        self.elision(span).into_in(self.allocator)
    }

    #[inline]
    pub fn object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> ObjectExpression<'a> {
        ObjectExpression { span, properties, trailing_comma }
    }

    #[inline]
    pub fn alloc_object_expression(
        self,
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ObjectExpression<'a>> {
        self.object_expression(span, properties, trailing_comma).into_in(self.allocator)
    }

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

    #[inline]
    pub fn object_property_kind_from_object_property<T>(self, inner: T) -> ObjectPropertyKind<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectProperty<'a>>>,
    {
        ObjectPropertyKind::ObjectProperty(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn object_property_kind_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        ObjectPropertyKind::SpreadProperty(self.alloc(self.spread_element(span, argument)))
    }

    #[inline]
    pub fn object_property_kind_from_spread_element<T>(self, inner: T) -> ObjectPropertyKind<'a>
    where
        T: IntoIn<'a, Box<'a, SpreadElement<'a>>>,
    {
        ObjectPropertyKind::SpreadProperty(inner.into_in(self.allocator))
    }

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
        self.object_property(span, kind, key, value, init, method, shorthand, computed)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn property_key_identifier_name<A>(self, span: Span, name: A) -> PropertyKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PropertyKey::StaticIdentifier(self.alloc(self.identifier_name(span, name)))
    }

    #[inline]
    pub fn property_key_from_identifier_name<T>(self, inner: T) -> PropertyKey<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        PropertyKey::StaticIdentifier(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn property_key_private_identifier<A>(self, span: Span, name: A) -> PropertyKey<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PropertyKey::PrivateIdentifier(self.alloc(self.private_identifier(span, name)))
    }

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

    #[inline]
    pub fn template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral { span, quasis, expressions }
    }

    #[inline]
    pub fn alloc_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, TemplateLiteral<'a>> {
        self.template_literal(span, quasis, expressions).into_in(self.allocator)
    }

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
        self.tagged_template_expression(span, tag, quasi, type_parameters).into_in(self.allocator)
    }

    #[inline]
    pub fn template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> TemplateElement<'a> {
        TemplateElement { span, tail, value }
    }

    #[inline]
    pub fn alloc_template_element(
        self,
        span: Span,
        tail: bool,
        value: TemplateElementValue<'a>,
    ) -> Box<'a, TemplateElement<'a>> {
        self.template_element(span, tail, value).into_in(self.allocator)
    }

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

    #[inline]
    pub fn member_expression_from_computed<T>(self, inner: T) -> MemberExpression<'a>
    where
        T: IntoIn<'a, Box<'a, ComputedMemberExpression<'a>>>,
    {
        MemberExpression::ComputedMemberExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn member_expression_from_static<T>(self, inner: T) -> MemberExpression<'a>
    where
        T: IntoIn<'a, Box<'a, StaticMemberExpression<'a>>>,
    {
        MemberExpression::StaticMemberExpression(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn alloc_computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> Box<'a, ComputedMemberExpression<'a>> {
        self.computed_member_expression(span, object, expression, optional).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, StaticMemberExpression<'a>> {
        self.static_member_expression(span, object, property, optional).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> Box<'a, PrivateFieldExpression<'a>> {
        self.private_field_expression(span, object, field, optional).into_in(self.allocator)
    }

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
        self.call_expression(span, arguments, callee, type_parameters, optional)
            .into_in(self.allocator)
    }

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
        self.new_expression(span, callee, arguments, type_parameters).into_in(self.allocator)
    }

    #[inline]
    pub fn meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> MetaProperty<'a> {
        MetaProperty { span, meta, property }
    }

    #[inline]
    pub fn alloc_meta_property(
        self,
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
    ) -> Box<'a, MetaProperty<'a>> {
        self.meta_property(span, meta, property).into_in(self.allocator)
    }

    #[inline]
    pub fn spread_element(self, span: Span, argument: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement { span, argument }
    }

    #[inline]
    pub fn alloc_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, SpreadElement<'a>> {
        self.spread_element(span, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn argument_spread_element(self, span: Span, argument: Expression<'a>) -> Argument<'a> {
        Argument::SpreadElement(self.alloc(self.spread_element(span, argument)))
    }

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

    #[inline]
    pub fn alloc_update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Box<'a, UpdateExpression<'a>> {
        self.update_expression(span, operator, prefix, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> UnaryExpression<'a> {
        UnaryExpression { span, operator, argument }
    }

    #[inline]
    pub fn alloc_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Box<'a, UnaryExpression<'a>> {
        self.unary_expression(span, operator, argument).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, BinaryExpression<'a>> {
        self.binary_expression(span, left, operator, right).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Box<'a, PrivateInExpression<'a>> {
        self.private_in_expression(span, left, operator, right).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Box<'a, LogicalExpression<'a>> {
        self.logical_expression(span, left, operator, right).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Box<'a, ConditionalExpression<'a>> {
        self.conditional_expression(span, test, consequent, alternate).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentExpression<'a>> {
        self.assignment_expression(span, operator, left, right).into_in(self.allocator)
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

    #[inline]
    pub fn alloc_array_assignment_target(
        self,
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: Option<AssignmentTargetRest<'a>>,
        trailing_comma: Option<Span>,
    ) -> Box<'a, ArrayAssignmentTarget<'a>> {
        self.array_assignment_target(span, elements, rest, trailing_comma).into_in(self.allocator)
    }

    #[inline]
    pub fn object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> ObjectAssignmentTarget<'a> {
        ObjectAssignmentTarget { span, properties, rest }
    }

    #[inline]
    pub fn alloc_object_assignment_target(
        self,
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: Option<AssignmentTargetRest<'a>>,
    ) -> Box<'a, ObjectAssignmentTarget<'a>> {
        self.object_assignment_target(span, properties, rest).into_in(self.allocator)
    }

    #[inline]
    pub fn assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> AssignmentTargetRest<'a> {
        AssignmentTargetRest { span, target }
    }

    #[inline]
    pub fn alloc_assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> Box<'a, AssignmentTargetRest<'a>> {
        self.assignment_target_rest(span, target).into_in(self.allocator)
    }

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

    #[inline]
    pub fn assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetWithDefault<'a> {
        AssignmentTargetWithDefault { span, binding, init }
    }

    #[inline]
    pub fn alloc_assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> Box<'a, AssignmentTargetWithDefault<'a>> {
        self.assignment_target_with_default(span, binding, init).into_in(self.allocator)
    }

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

    #[inline]
    pub fn assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetPropertyIdentifier<'a> {
        AssignmentTargetPropertyIdentifier { span, binding, init }
    }

    #[inline]
    pub fn alloc_assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> Box<'a, AssignmentTargetPropertyIdentifier<'a>> {
        self.assignment_target_property_identifier(span, binding, init).into_in(self.allocator)
    }

    #[inline]
    pub fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> AssignmentTargetPropertyProperty<'a> {
        AssignmentTargetPropertyProperty { span, name, binding }
    }

    #[inline]
    pub fn alloc_assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
    ) -> Box<'a, AssignmentTargetPropertyProperty<'a>> {
        self.assignment_target_property_property(span, name, binding).into_in(self.allocator)
    }

    #[inline]
    pub fn sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> SequenceExpression<'a> {
        SequenceExpression { span, expressions }
    }

    #[inline]
    pub fn alloc_sequence_expression(
        self,
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
    ) -> Box<'a, SequenceExpression<'a>> {
        self.sequence_expression(span, expressions).into_in(self.allocator)
    }

    #[inline]
    pub fn super_(self, span: Span) -> Super {
        Super { span }
    }

    #[inline]
    pub fn alloc_super_(self, span: Span) -> Box<'a, Super> {
        self.super_(span).into_in(self.allocator)
    }

    #[inline]
    pub fn await_expression(self, span: Span, argument: Expression<'a>) -> AwaitExpression<'a> {
        AwaitExpression { span, argument }
    }

    #[inline]
    pub fn alloc_await_expression(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, AwaitExpression<'a>> {
        self.await_expression(span, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn chain_expression(self, span: Span, expression: ChainElement<'a>) -> ChainExpression<'a> {
        ChainExpression { span, expression }
    }

    #[inline]
    pub fn alloc_chain_expression(
        self,
        span: Span,
        expression: ChainElement<'a>,
    ) -> Box<'a, ChainExpression<'a>> {
        self.chain_expression(span, expression).into_in(self.allocator)
    }

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

    #[inline]
    pub fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ParenthesizedExpression<'a> {
        ParenthesizedExpression { span, expression }
    }

    #[inline]
    pub fn alloc_parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ParenthesizedExpression<'a>> {
        self.parenthesized_expression(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn statement_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(self.alloc(self.block_statement(span, body)))
    }

    #[inline]
    pub fn statement_from_block<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Statement::BlockStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_break(self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc(self.break_statement(span, label)))
    }

    #[inline]
    pub fn statement_from_break<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, BreakStatement<'a>>>,
    {
        Statement::BreakStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_continue(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc(self.continue_statement(span, label)))
    }

    #[inline]
    pub fn statement_from_continue<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ContinueStatement<'a>>>,
    {
        Statement::ContinueStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_debugger(self, span: Span) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc(self.debugger_statement(span)))
    }

    #[inline]
    pub fn statement_from_debugger<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, DebuggerStatement>>,
    {
        Statement::DebuggerStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_do_while(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc(self.do_while_statement(span, body, test)))
    }

    #[inline]
    pub fn statement_from_do_while<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, DoWhileStatement<'a>>>,
    {
        Statement::DoWhileStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_empty(self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc(self.empty_statement(span)))
    }

    #[inline]
    pub fn statement_from_empty<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, EmptyStatement>>,
    {
        Statement::EmptyStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_expression(self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc(self.expression_statement(span, expression)))
    }

    #[inline]
    pub fn statement_from_expression<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ExpressionStatement<'a>>>,
    {
        Statement::ExpressionStatement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn statement_from_for_in<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForInStatement<'a>>>,
    {
        Statement::ForInStatement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn statement_from_for_of<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForOfStatement<'a>>>,
    {
        Statement::ForOfStatement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn statement_from_for<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ForStatement<'a>>>,
    {
        Statement::ForStatement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn statement_from_if<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, IfStatement<'a>>>,
    {
        Statement::IfStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_labeled(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc(self.labeled_statement(span, label, body)))
    }

    #[inline]
    pub fn statement_from_labeled<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, LabeledStatement<'a>>>,
    {
        Statement::LabeledStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_return(self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc(self.return_statement(span, argument)))
    }

    #[inline]
    pub fn statement_from_return<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ReturnStatement<'a>>>,
    {
        Statement::ReturnStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_switch(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc(self.switch_statement(span, discriminant, cases)))
    }

    #[inline]
    pub fn statement_from_switch<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, SwitchStatement<'a>>>,
    {
        Statement::SwitchStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_throw(self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc(self.throw_statement(span, argument)))
    }

    #[inline]
    pub fn statement_from_throw<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, ThrowStatement<'a>>>,
    {
        Statement::ThrowStatement(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn statement_from_try<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, TryStatement<'a>>>,
    {
        Statement::TryStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_while(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc(self.while_statement(span, test, body)))
    }

    #[inline]
    pub fn statement_from_while<T>(self, inner: T) -> Statement<'a>
    where
        T: IntoIn<'a, Box<'a, WhileStatement<'a>>>,
    {
        Statement::WhileStatement(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn statement_with(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc(self.with_statement(span, object, body)))
    }

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
        self.directive(span, expression, directive).into_in(self.allocator)
    }

    #[inline]
    pub fn hashbang<A>(self, span: Span, value: A) -> Hashbang<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        Hashbang { span, value: value.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_hashbang<A>(self, span: Span, value: A) -> Box<'a, Hashbang<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.hashbang(span, value).into_in(self.allocator)
    }

    #[inline]
    pub fn block_statement(self, span: Span, body: Vec<'a, Statement<'a>>) -> BlockStatement<'a> {
        BlockStatement { span, body, scope_id: Default::default() }
    }

    #[inline]
    pub fn alloc_block_statement(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, BlockStatement<'a>> {
        self.block_statement(span, body).into_in(self.allocator)
    }

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

    #[inline]
    pub fn declaration_from_variable<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        Declaration::VariableDeclaration(inner.into_in(self.allocator))
    }

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
        body: T3,
        return_type: T4,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
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
            body,
            return_type,
        )))
    }

    #[inline]
    pub fn declaration_from_function<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Declaration::FunctionDeclaration(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn declaration_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: T1,
        type_parameters: T2,
        super_type_parameters: T3,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Box<'a, ClassBody<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Declaration::ClassDeclaration(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            declare,
        )))
    }

    #[inline]
    pub fn declaration_from_class<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, Class<'a>>>,
    {
        Declaration::ClassDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_using<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, UsingDeclaration<'a>>>,
    {
        Declaration::UsingDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_ts_type_alias<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeAliasDeclaration<'a>>>,
    {
        Declaration::TSTypeAliasDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_ts_interface<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSInterfaceDeclaration<'a>>>,
    {
        Declaration::TSInterfaceDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_ts_enum<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSEnumDeclaration<'a>>>,
    {
        Declaration::TSEnumDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_ts_module<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSModuleDeclaration<'a>>>,
    {
        Declaration::TSModuleDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn declaration_from_ts_import_equals<T>(self, inner: T) -> Declaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSImportEqualsDeclaration<'a>>>,
    {
        Declaration::TSImportEqualsDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn alloc_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> Box<'a, VariableDeclaration<'a>> {
        self.variable_declaration(span, kind, declarations, declare).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_variable_declarator(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> Box<'a, VariableDeclarator<'a>> {
        self.variable_declarator(span, kind, id, init, definite).into_in(self.allocator)
    }

    #[inline]
    pub fn using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> UsingDeclaration<'a> {
        UsingDeclaration { span, is_await, declarations }
    }

    #[inline]
    pub fn alloc_using_declaration(
        self,
        span: Span,
        is_await: bool,
        declarations: Vec<'a, VariableDeclarator<'a>>,
    ) -> Box<'a, UsingDeclaration<'a>> {
        self.using_declaration(span, is_await, declarations).into_in(self.allocator)
    }

    #[inline]
    pub fn empty_statement(self, span: Span) -> EmptyStatement {
        EmptyStatement { span }
    }

    #[inline]
    pub fn alloc_empty_statement(self, span: Span) -> Box<'a, EmptyStatement> {
        self.empty_statement(span).into_in(self.allocator)
    }

    #[inline]
    pub fn expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ExpressionStatement<'a> {
        ExpressionStatement { span, expression }
    }

    #[inline]
    pub fn alloc_expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, ExpressionStatement<'a>> {
        self.expression_statement(span, expression).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Box<'a, IfStatement<'a>> {
        self.if_statement(span, test, consequent, alternate).into_in(self.allocator)
    }

    #[inline]
    pub fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> DoWhileStatement<'a> {
        DoWhileStatement { span, body, test }
    }

    #[inline]
    pub fn alloc_do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Box<'a, DoWhileStatement<'a>> {
        self.do_while_statement(span, body, test).into_in(self.allocator)
    }

    #[inline]
    pub fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> WhileStatement<'a> {
        WhileStatement { span, test, body }
    }

    #[inline]
    pub fn alloc_while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WhileStatement<'a>> {
        self.while_statement(span, test, body).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Box<'a, ForStatement<'a>> {
        self.for_statement(span, init, test, update, body).into_in(self.allocator)
    }

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

    #[inline]
    pub fn for_statement_init_from_variable_declaration<T>(self, inner: T) -> ForStatementInit<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        ForStatementInit::VariableDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn alloc_for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForInStatement<'a>> {
        self.for_in_statement(span, left, right, body).into_in(self.allocator)
    }

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

    #[inline]
    pub fn for_statement_left_from_variable_declaration<T>(self, inner: T) -> ForStatementLeft<'a>
    where
        T: IntoIn<'a, Box<'a, VariableDeclaration<'a>>>,
    {
        ForStatementLeft::VariableDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn alloc_for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, ForOfStatement<'a>> {
        self.for_of_statement(span, r#await, left, right, body).into_in(self.allocator)
    }

    #[inline]
    pub fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ContinueStatement<'a> {
        ContinueStatement { span, label }
    }

    #[inline]
    pub fn alloc_continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, ContinueStatement<'a>> {
        self.continue_statement(span, label).into_in(self.allocator)
    }

    #[inline]
    pub fn break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> BreakStatement<'a> {
        BreakStatement { span, label }
    }

    #[inline]
    pub fn alloc_break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Box<'a, BreakStatement<'a>> {
        self.break_statement(span, label).into_in(self.allocator)
    }

    #[inline]
    pub fn return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> ReturnStatement<'a> {
        ReturnStatement { span, argument }
    }

    #[inline]
    pub fn alloc_return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, ReturnStatement<'a>> {
        self.return_statement(span, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> WithStatement<'a> {
        WithStatement { span, object, body }
    }

    #[inline]
    pub fn alloc_with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Box<'a, WithStatement<'a>> {
        self.with_statement(span, object, body).into_in(self.allocator)
    }

    #[inline]
    pub fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> SwitchStatement<'a> {
        SwitchStatement { span, discriminant, cases, scope_id: Default::default() }
    }

    #[inline]
    pub fn alloc_switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
    ) -> Box<'a, SwitchStatement<'a>> {
        self.switch_statement(span, discriminant, cases).into_in(self.allocator)
    }

    #[inline]
    pub fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { span, test, consequent }
    }

    #[inline]
    pub fn alloc_switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
    ) -> Box<'a, SwitchCase<'a>> {
        self.switch_case(span, test, consequent).into_in(self.allocator)
    }

    #[inline]
    pub fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> LabeledStatement<'a> {
        LabeledStatement { span, label, body }
    }

    #[inline]
    pub fn alloc_labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Box<'a, LabeledStatement<'a>> {
        self.labeled_statement(span, label, body).into_in(self.allocator)
    }

    #[inline]
    pub fn throw_statement(self, span: Span, argument: Expression<'a>) -> ThrowStatement<'a> {
        ThrowStatement { span, argument }
    }

    #[inline]
    pub fn alloc_throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, ThrowStatement<'a>> {
        self.throw_statement(span, argument).into_in(self.allocator)
    }

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
        self.try_statement(span, block, handler, finalizer).into_in(self.allocator)
    }

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
        self.catch_clause(span, param, body).into_in(self.allocator)
    }

    #[inline]
    pub fn catch_parameter(self, span: Span, pattern: BindingPattern<'a>) -> CatchParameter<'a> {
        CatchParameter { span, pattern }
    }

    #[inline]
    pub fn alloc_catch_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> Box<'a, CatchParameter<'a>> {
        self.catch_parameter(span, pattern).into_in(self.allocator)
    }

    #[inline]
    pub fn debugger_statement(self, span: Span) -> DebuggerStatement {
        DebuggerStatement { span }
    }

    #[inline]
    pub fn alloc_debugger_statement(self, span: Span) -> Box<'a, DebuggerStatement> {
        self.debugger_statement(span).into_in(self.allocator)
    }

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
        self.binding_pattern(kind, type_annotation, optional).into_in(self.allocator)
    }

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

    #[inline]
    pub fn binding_pattern_kind_from_binding_identifier<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, BindingIdentifier<'a>>>,
    {
        BindingPatternKind::BindingIdentifier(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn binding_pattern_kind_from_object_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, ObjectPattern<'a>>>,
    {
        BindingPatternKind::ObjectPattern(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn binding_pattern_kind_from_array_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, ArrayPattern<'a>>>,
    {
        BindingPatternKind::ArrayPattern(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn binding_pattern_kind_from_assignment_pattern<T>(self, inner: T) -> BindingPatternKind<'a>
    where
        T: IntoIn<'a, Box<'a, AssignmentPattern<'a>>>,
    {
        BindingPatternKind::AssignmentPattern(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> AssignmentPattern<'a> {
        AssignmentPattern { span, left, right }
    }

    #[inline]
    pub fn alloc_assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> Box<'a, AssignmentPattern<'a>> {
        self.assignment_pattern(span, left, right).into_in(self.allocator)
    }

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
        self.object_pattern(span, properties, rest).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> Box<'a, BindingProperty<'a>> {
        self.binding_property(span, key, value, shorthand, computed).into_in(self.allocator)
    }

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
        self.array_pattern(span, elements, rest).into_in(self.allocator)
    }

    #[inline]
    pub fn binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a> {
        BindingRestElement { span, argument }
    }

    #[inline]
    pub fn alloc_binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> Box<'a, BindingRestElement<'a>> {
        self.binding_rest_element(span, argument).into_in(self.allocator)
    }

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
        body: T3,
        return_type: T4,
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
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
            body: body.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

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
        body: T3,
        return_type: T4,
    ) -> Box<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
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
            body,
            return_type,
        )
        .into_in(self.allocator)
    }

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
        self.formal_parameters(span, kind, items, rest).into_in(self.allocator)
    }

    #[inline]
    pub fn formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> FormalParameter<'a> {
        FormalParameter { span, pattern, accessibility, readonly, r#override, decorators }
    }

    #[inline]
    pub fn alloc_formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, FormalParameter<'a>> {
        self.formal_parameter(span, pattern, accessibility, readonly, r#override, decorators)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> FunctionBody<'a> {
        FunctionBody { span, directives, statements }
    }

    #[inline]
    pub fn alloc_function_body(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
    ) -> Box<'a, FunctionBody<'a>> {
        self.function_body(span, directives, statements).into_in(self.allocator)
    }

    #[inline]
    pub fn arrow_function_expression<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: T1,
        body: T2,
        type_parameters: T3,
        return_type: T4,
    ) -> ArrowFunctionExpression<'a>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        ArrowFunctionExpression {
            span,
            expression,
            r#async,
            params: params.into_in(self.allocator),
            body: body.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
            return_type: return_type.into_in(self.allocator),
            scope_id: Default::default(),
        }
    }

    #[inline]
    pub fn alloc_arrow_function_expression<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        params: T1,
        body: T2,
        type_parameters: T3,
        return_type: T4,
    ) -> Box<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T2: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        self.arrow_function_expression(
            span,
            expression,
            r#async,
            params,
            body,
            type_parameters,
            return_type,
        )
        .into_in(self.allocator)
    }

    #[inline]
    pub fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> YieldExpression<'a> {
        YieldExpression { span, delegate, argument }
    }

    #[inline]
    pub fn alloc_yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Box<'a, YieldExpression<'a>> {
        self.yield_expression(span, delegate, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: T1,
        type_parameters: T2,
        super_type_parameters: T3,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> Class<'a>
    where
        T1: IntoIn<'a, Box<'a, ClassBody<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Class {
            r#type,
            span,
            decorators,
            id,
            super_class,
            body: body.into_in(self.allocator),
            type_parameters: type_parameters.into_in(self.allocator),
            super_type_parameters: super_type_parameters.into_in(self.allocator),
            implements,
            r#abstract,
            declare,
            scope_id: Default::default(),
        }
    }

    #[inline]
    pub fn alloc_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: T1,
        type_parameters: T2,
        super_type_parameters: T3,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> Box<'a, Class<'a>>
    where
        T1: IntoIn<'a, Box<'a, ClassBody<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        self.class(
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            declare,
        )
        .into_in(self.allocator)
    }

    #[inline]
    pub fn class_body(self, span: Span, body: Vec<'a, ClassElement<'a>>) -> ClassBody<'a> {
        ClassBody { span, body }
    }

    #[inline]
    pub fn alloc_class_body(
        self,
        span: Span,
        body: Vec<'a, ClassElement<'a>>,
    ) -> Box<'a, ClassBody<'a>> {
        self.class_body(span, body).into_in(self.allocator)
    }

    #[inline]
    pub fn class_element_static_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc(self.static_block(span, body)))
    }

    #[inline]
    pub fn class_element_from_static_block<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, StaticBlock<'a>>>,
    {
        ClassElement::StaticBlock(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn class_element_from_method_definition<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, MethodDefinition<'a>>>,
    {
        ClassElement::MethodDefinition(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn class_element_from_property_definition<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, PropertyDefinition<'a>>>,
    {
        ClassElement::PropertyDefinition(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn class_element_accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::AccessorProperty(self.alloc(
            self.accessor_property(r#type, span, key, value, computed, r#static, decorators),
        ))
    }

    #[inline]
    pub fn class_element_from_accessor_property<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, AccessorProperty<'a>>>,
    {
        ClassElement::AccessorProperty(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn class_element_from_ts_index_signature<T>(self, inner: T) -> ClassElement<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexSignature<'a>>>,
    {
        ClassElement::TSIndexSignature(inner.into_in(self.allocator))
    }

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
        )
        .into_in(self.allocator)
    }

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
        )
        .into_in(self.allocator)
    }

    #[inline]
    pub fn private_identifier<A>(self, span: Span, name: A) -> PrivateIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        PrivateIdentifier { span, name: name.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_private_identifier<A>(self, span: Span, name: A) -> Box<'a, PrivateIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.private_identifier(span, name).into_in(self.allocator)
    }

    #[inline]
    pub fn static_block(self, span: Span, body: Vec<'a, Statement<'a>>) -> StaticBlock<'a> {
        StaticBlock { span, body, scope_id: Default::default() }
    }

    #[inline]
    pub fn alloc_static_block(
        self,
        span: Span,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, StaticBlock<'a>> {
        self.static_block(span, body).into_in(self.allocator)
    }

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

    #[inline]
    pub fn module_declaration_from_import_declaration<T>(self, inner: T) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, ImportDeclaration<'a>>>,
    {
        ModuleDeclaration::ImportDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn module_declaration_from_ts_export_assignment<T>(self, inner: T) -> ModuleDeclaration<'a>
    where
        T: IntoIn<'a, Box<'a, TSExportAssignment<'a>>>,
    {
        ModuleDeclaration::TSExportAssignment(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> AccessorProperty<'a> {
        AccessorProperty { r#type, span, key, value, computed, r#static, decorators }
    }

    #[inline]
    pub fn alloc_accessor_property(
        self,
        r#type: AccessorPropertyType,
        span: Span,
        key: PropertyKey<'a>,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        decorators: Vec<'a, Decorator<'a>>,
    ) -> Box<'a, AccessorProperty<'a>> {
        self.accessor_property(r#type, span, key, value, computed, r#static, decorators)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> ImportExpression<'a> {
        ImportExpression { span, source, arguments }
    }

    #[inline]
    pub fn alloc_import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        arguments: Vec<'a, Expression<'a>>,
    ) -> Box<'a, ImportExpression<'a>> {
        self.import_expression(span, source, arguments).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_import_declaration(
        self,
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportDeclaration<'a>> {
        self.import_declaration(span, specifiers, source, with_clause, import_kind)
            .into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, ImportSpecifier<'a>> {
        self.import_specifier(span, imported, local, import_kind).into_in(self.allocator)
    }

    #[inline]
    pub fn import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDefaultSpecifier<'a> {
        ImportDefaultSpecifier { span, local }
    }

    #[inline]
    pub fn alloc_import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportDefaultSpecifier<'a>> {
        self.import_default_specifier(span, local).into_in(self.allocator)
    }

    #[inline]
    pub fn import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportNamespaceSpecifier<'a> {
        ImportNamespaceSpecifier { span, local }
    }

    #[inline]
    pub fn alloc_import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> Box<'a, ImportNamespaceSpecifier<'a>> {
        self.import_namespace_specifier(span, local).into_in(self.allocator)
    }

    #[inline]
    pub fn with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> WithClause<'a> {
        WithClause { span, attributes_keyword, with_entries }
    }

    #[inline]
    pub fn alloc_with_clause(
        self,
        span: Span,
        attributes_keyword: IdentifierName<'a>,
        with_entries: Vec<'a, ImportAttribute<'a>>,
    ) -> Box<'a, WithClause<'a>> {
        self.with_clause(span, attributes_keyword, with_entries).into_in(self.allocator)
    }

    #[inline]
    pub fn import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> ImportAttribute<'a> {
        ImportAttribute { span, key, value }
    }

    #[inline]
    pub fn alloc_import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> Box<'a, ImportAttribute<'a>> {
        self.import_attribute(span, key, value).into_in(self.allocator)
    }

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

    #[inline]
    pub fn import_attribute_key_from_identifier_name<T>(self, inner: T) -> ImportAttributeKey<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        ImportAttributeKey::Identifier(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn import_attribute_key_from_string_literal<T>(self, inner: T) -> ImportAttributeKey<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        ImportAttributeKey::StringLiteral(inner.into_in(self.allocator))
    }

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
        self.export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        )
        .into_in(self.allocator)
    }

    #[inline]
    pub fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> ExportDefaultDeclaration<'a> {
        ExportDefaultDeclaration { span, declaration, exported }
    }

    #[inline]
    pub fn alloc_export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        exported: ModuleExportName<'a>,
    ) -> Box<'a, ExportDefaultDeclaration<'a>> {
        self.export_default_declaration(span, declaration, exported).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_export_all_declaration(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: Option<WithClause<'a>>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportAllDeclaration<'a>> {
        self.export_all_declaration(span, exported, source, with_clause, export_kind)
            .into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> Box<'a, ExportSpecifier<'a>> {
        self.export_specifier(span, local, exported, export_kind).into_in(self.allocator)
    }

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
        body: T3,
        return_type: T4,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
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
            body,
            return_type,
        )))
    }

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

    #[inline]
    pub fn export_default_declaration_kind_class<T1, T2, T3>(
        self,
        r#type: ClassType,
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        super_class: Option<Expression<'a>>,
        body: T1,
        type_parameters: T2,
        super_type_parameters: T3,
        implements: Option<Vec<'a, TSClassImplements<'a>>>,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Box<'a, ClassBody<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ExportDefaultDeclarationKind::ClassDeclaration(self.alloc(self.class(
            r#type,
            span,
            decorators,
            id,
            super_class,
            body,
            type_parameters,
            super_type_parameters,
            implements,
            r#abstract,
            declare,
        )))
    }

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

    #[inline]
    pub fn module_export_name_identifier_name<A>(self, span: Span, name: A) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::IdentifierName(self.identifier_name(span, name))
    }

    #[inline]
    pub fn module_export_name_from_identifier_name<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, IdentifierName<'a>>,
    {
        ModuleExportName::IdentifierName(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn module_export_name_from_identifier_reference<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, IdentifierReference<'a>>,
    {
        ModuleExportName::IdentifierReference(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn module_export_name_string_literal<A>(self, span: Span, value: A) -> ModuleExportName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        ModuleExportName::StringLiteral(self.string_literal(span, value))
    }

    #[inline]
    pub fn module_export_name_from_string_literal<T>(self, inner: T) -> ModuleExportName<'a>
    where
        T: IntoIn<'a, StringLiteral<'a>>,
    {
        ModuleExportName::StringLiteral(inner.into_in(self.allocator))
    }

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
        self.ts_this_parameter(span, this, type_annotation).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        members: Vec<'a, TSEnumMember<'a>>,
        r#const: bool,
        declare: bool,
    ) -> Box<'a, TSEnumDeclaration<'a>> {
        self.ts_enum_declaration(span, id, members, r#const, declare).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> TSEnumMember<'a> {
        TSEnumMember { span, id, initializer }
    }

    #[inline]
    pub fn alloc_ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> Box<'a, TSEnumMember<'a>> {
        self.ts_enum_member(span, id, initializer).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_enum_member_name_identifier_name<A>(self, span: Span, name: A) -> TSEnumMemberName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSEnumMemberName::StaticIdentifier(self.alloc(self.identifier_name(span, name)))
    }

    #[inline]
    pub fn ts_enum_member_name_from_identifier_name<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        TSEnumMemberName::StaticIdentifier(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_enum_member_name_string_literal<A>(self, span: Span, value: A) -> TSEnumMemberName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSEnumMemberName::StaticStringLiteral(self.alloc(self.string_literal(span, value)))
    }

    #[inline]
    pub fn ts_enum_member_name_from_string_literal<T>(self, inner: T) -> TSEnumMemberName<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        TSEnumMemberName::StaticStringLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation { span, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAnnotation<'a>> {
        self.ts_type_annotation(span, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSLiteralType<'a> {
        TSLiteralType { span, literal }
    }

    #[inline]
    pub fn alloc_ts_literal_type(
        self,
        span: Span,
        literal: TSLiteral<'a>,
    ) -> Box<'a, TSLiteralType<'a>> {
        self.ts_literal_type(span, literal).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_literal_boolean_literal(self, span: Span, value: bool) -> TSLiteral<'a> {
        TSLiteral::BooleanLiteral(self.alloc(self.boolean_literal(span, value)))
    }

    #[inline]
    pub fn ts_literal_from_boolean_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, BooleanLiteral>>,
    {
        TSLiteral::BooleanLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_literal_null_literal(self, span: Span) -> TSLiteral<'a> {
        TSLiteral::NullLiteral(self.alloc(self.null_literal(span)))
    }

    #[inline]
    pub fn ts_literal_from_null_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, NullLiteral>>,
    {
        TSLiteral::NullLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_literal_from_numeric_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, NumericLiteral<'a>>>,
    {
        TSLiteral::NumericLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_literal_from_big_int_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, BigIntLiteral<'a>>>,
    {
        TSLiteral::BigIntLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_literal_reg_exp_literal(
        self,
        span: Span,
        value: EmptyObject,
        regex: RegExp<'a>,
    ) -> TSLiteral<'a> {
        TSLiteral::RegExpLiteral(self.alloc(self.reg_exp_literal(span, value, regex)))
    }

    #[inline]
    pub fn ts_literal_from_reg_exp_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, RegExpLiteral<'a>>>,
    {
        TSLiteral::RegExpLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_literal_string_literal<A>(self, span: Span, value: A) -> TSLiteral<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSLiteral::StringLiteral(self.alloc(self.string_literal(span, value)))
    }

    #[inline]
    pub fn ts_literal_from_string_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        TSLiteral::StringLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_literal_template_literal(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
    ) -> TSLiteral<'a> {
        TSLiteral::TemplateLiteral(self.alloc(self.template_literal(span, quasis, expressions)))
    }

    #[inline]
    pub fn ts_literal_from_template_literal<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, TemplateLiteral<'a>>>,
    {
        TSLiteral::TemplateLiteral(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_literal_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> TSLiteral<'a> {
        TSLiteral::UnaryExpression(self.alloc(self.unary_expression(span, operator, argument)))
    }

    #[inline]
    pub fn ts_literal_from_unary_expression<T>(self, inner: T) -> TSLiteral<'a>
    where
        T: IntoIn<'a, Box<'a, UnaryExpression<'a>>>,
    {
        TSLiteral::UnaryExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_any_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSAnyKeyword(self.alloc(self.ts_any_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_any_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSAnyKeyword>>,
    {
        TSType::TSAnyKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_big_int_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc(self.ts_big_int_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_big_int_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSBigIntKeyword>>,
    {
        TSType::TSBigIntKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_boolean_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBooleanKeyword(self.alloc(self.ts_boolean_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_boolean_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSBooleanKeyword>>,
    {
        TSType::TSBooleanKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_intrinsic_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSIntrinsicKeyword(self.alloc(self.ts_intrinsic_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_intrinsic_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIntrinsicKeyword>>,
    {
        TSType::TSIntrinsicKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_never_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNeverKeyword(self.alloc(self.ts_never_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_never_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNeverKeyword>>,
    {
        TSType::TSNeverKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_null_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNullKeyword(self.alloc(self.ts_null_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_null_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNullKeyword>>,
    {
        TSType::TSNullKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_number_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNumberKeyword(self.alloc(self.ts_number_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_number_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNumberKeyword>>,
    {
        TSType::TSNumberKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_object_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSObjectKeyword(self.alloc(self.ts_object_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_object_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSObjectKeyword>>,
    {
        TSType::TSObjectKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_string_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSStringKeyword(self.alloc(self.ts_string_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_string_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSStringKeyword>>,
    {
        TSType::TSStringKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_symbol_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSSymbolKeyword(self.alloc(self.ts_symbol_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_symbol_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSSymbolKeyword>>,
    {
        TSType::TSSymbolKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_undefined_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUndefinedKeyword(self.alloc(self.ts_undefined_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_undefined_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUndefinedKeyword>>,
    {
        TSType::TSUndefinedKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_unknown_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUnknownKeyword(self.alloc(self.ts_unknown_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_unknown_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUnknownKeyword>>,
    {
        TSType::TSUnknownKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_void_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSVoidKeyword(self.alloc(self.ts_void_keyword(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_void_keyword<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSVoidKeyword>>,
    {
        TSType::TSVoidKeyword(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_array_type(self, span: Span, element_type: TSType<'a>) -> TSType<'a> {
        TSType::TSArrayType(self.alloc(self.ts_array_type(span, element_type)))
    }

    #[inline]
    pub fn ts_type_from_ts_array_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSArrayType<'a>>>,
    {
        TSType::TSArrayType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_conditional_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSConditionalType<'a>>>,
    {
        TSType::TSConditionalType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_constructor_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSConstructorType<'a>>>,
    {
        TSType::TSConstructorType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_function_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSFunctionType<'a>>>,
    {
        TSType::TSFunctionType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_import_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSImportType<'a>>>,
    {
        TSType::TSImportType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_indexed_access_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexedAccessType<'a>>>,
    {
        TSType::TSIndexedAccessType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSType::TSInferType(self.alloc(self.ts_infer_type(span, type_parameter)))
    }

    #[inline]
    pub fn ts_type_from_ts_infer_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSInferType<'a>>>,
    {
        TSType::TSInferType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_intersection_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSIntersectionType(self.alloc(self.ts_intersection_type(span, types)))
    }

    #[inline]
    pub fn ts_type_from_ts_intersection_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSIntersectionType<'a>>>,
    {
        TSType::TSIntersectionType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSType<'a> {
        TSType::TSLiteralType(self.alloc(self.ts_literal_type(span, literal)))
    }

    #[inline]
    pub fn ts_type_from_ts_literal_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSLiteralType<'a>>>,
    {
        TSType::TSLiteralType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_mapped_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSMappedType<'a>>>,
    {
        TSType::TSMappedType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_named_tuple_member<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSNamedTupleMember<'a>>>,
    {
        TSType::TSNamedTupleMember(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSType<'a> {
        TSType::TSQualifiedName(self.alloc(self.ts_qualified_name(span, left, right)))
    }

    #[inline]
    pub fn ts_type_from_ts_qualified_name<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSQualifiedName<'a>>>,
    {
        TSType::TSQualifiedName(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_template_literal_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTemplateLiteralType<'a>>>,
    {
        TSType::TSTemplateLiteralType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_this_type(self, span: Span) -> TSType<'a> {
        TSType::TSThisType(self.alloc(self.ts_this_type(span)))
    }

    #[inline]
    pub fn ts_type_from_ts_this_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSThisType>>,
    {
        TSType::TSThisType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSType<'a> {
        TSType::TSTupleType(self.alloc(self.ts_tuple_type(span, element_types)))
    }

    #[inline]
    pub fn ts_type_from_ts_tuple_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTupleType<'a>>>,
    {
        TSType::TSTupleType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_type_literal(self, span: Span, members: Vec<'a, TSSignature<'a>>) -> TSType<'a> {
        TSType::TSTypeLiteral(self.alloc(self.ts_type_literal(span, members)))
    }

    #[inline]
    pub fn ts_type_from_ts_type_literal<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeLiteral<'a>>>,
    {
        TSType::TSTypeLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_type_operator<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeOperator<'a>>>,
    {
        TSType::TSTypeOperatorType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_type_predicate<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypePredicate<'a>>>,
    {
        TSType::TSTypePredicate(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_type_query<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeQuery<'a>>>,
    {
        TSType::TSTypeQuery(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_ts_type_reference<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSTypeReference<'a>>>,
    {
        TSType::TSTypeReference(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc(self.ts_union_type(span, types)))
    }

    #[inline]
    pub fn ts_type_from_ts_union_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSUnionType<'a>>>,
    {
        TSType::TSUnionType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_parenthesized_type(self, span: Span, type_annotation: TSType<'a>) -> TSType<'a> {
        TSType::TSParenthesizedType(self.alloc(self.ts_parenthesized_type(span, type_annotation)))
    }

    #[inline]
    pub fn ts_type_from_ts_parenthesized_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, TSParenthesizedType<'a>>>,
    {
        TSType::TSParenthesizedType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_js_doc_nullable_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocNullableType<'a>>>,
    {
        TSType::JSDocNullableType(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_type_from_js_doc_non_nullable_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocNonNullableType<'a>>>,
    {
        TSType::JSDocNonNullableType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_js_doc_unknown_type(self, span: Span) -> TSType<'a> {
        TSType::JSDocUnknownType(self.alloc(self.js_doc_unknown_type(span)))
    }

    #[inline]
    pub fn ts_type_from_js_doc_unknown_type<T>(self, inner: T) -> TSType<'a>
    where
        T: IntoIn<'a, Box<'a, JSDocUnknownType>>,
    {
        TSType::JSDocUnknownType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSConditionalType<'a> {
        TSConditionalType { span, check_type, extends_type, true_type, false_type }
    }

    #[inline]
    pub fn alloc_ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> Box<'a, TSConditionalType<'a>> {
        self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn ts_union_type(self, span: Span, types: Vec<'a, TSType<'a>>) -> TSUnionType<'a> {
        TSUnionType { span, types }
    }

    #[inline]
    pub fn alloc_ts_union_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSUnionType<'a>> {
        self.ts_union_type(span, types).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> TSIntersectionType<'a> {
        TSIntersectionType { span, types }
    }

    #[inline]
    pub fn alloc_ts_intersection_type(
        self,
        span: Span,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSIntersectionType<'a>> {
        self.ts_intersection_type(span, types).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSParenthesizedType<'a> {
        TSParenthesizedType { span, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSParenthesizedType<'a>> {
        self.ts_parenthesized_type(span, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSTypeOperator<'a> {
        TSTypeOperator { span, operator, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeOperator<'a>> {
        self.ts_type_operator(span, operator, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> TSArrayType<'a> {
        TSArrayType { span, element_type }
    }

    #[inline]
    pub fn alloc_ts_array_type(
        self,
        span: Span,
        element_type: TSType<'a>,
    ) -> Box<'a, TSArrayType<'a>> {
        self.ts_array_type(span, element_type).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSIndexedAccessType<'a> {
        TSIndexedAccessType { span, object_type, index_type }
    }

    #[inline]
    pub fn alloc_ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> Box<'a, TSIndexedAccessType<'a>> {
        self.ts_indexed_access_type(span, object_type, index_type).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> TSTupleType<'a> {
        TSTupleType { span, element_types }
    }

    #[inline]
    pub fn alloc_ts_tuple_type(
        self,
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
    ) -> Box<'a, TSTupleType<'a>> {
        self.ts_tuple_type(span, element_types).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_ts_named_tuple_member(
        self,
        span: Span,
        element_type: TSTupleElement<'a>,
        label: IdentifierName<'a>,
        optional: bool,
    ) -> Box<'a, TSNamedTupleMember<'a>> {
        self.ts_named_tuple_member(span, element_type, label, optional).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_optional_type(self, span: Span, type_annotation: TSType<'a>) -> TSOptionalType<'a> {
        TSOptionalType { span, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSOptionalType<'a>> {
        self.ts_optional_type(span, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_rest_type(self, span: Span, type_annotation: TSType<'a>) -> TSRestType<'a> {
        TSRestType { span, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSRestType<'a>> {
        self.ts_rest_type(span, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_tuple_element_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSOptionalType(self.alloc(self.ts_optional_type(span, type_annotation)))
    }

    #[inline]
    pub fn ts_tuple_element_from_ts_optional_type<T>(self, inner: T) -> TSTupleElement<'a>
    where
        T: IntoIn<'a, Box<'a, TSOptionalType<'a>>>,
    {
        TSTupleElement::TSOptionalType(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_tuple_element_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSRestType(self.alloc(self.ts_rest_type(span, type_annotation)))
    }

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

    #[inline]
    pub fn ts_any_keyword(self, span: Span) -> TSAnyKeyword {
        TSAnyKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_any_keyword(self, span: Span) -> Box<'a, TSAnyKeyword> {
        self.ts_any_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_string_keyword(self, span: Span) -> TSStringKeyword {
        TSStringKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_string_keyword(self, span: Span) -> Box<'a, TSStringKeyword> {
        self.ts_string_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_boolean_keyword(self, span: Span) -> TSBooleanKeyword {
        TSBooleanKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_boolean_keyword(self, span: Span) -> Box<'a, TSBooleanKeyword> {
        self.ts_boolean_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_number_keyword(self, span: Span) -> TSNumberKeyword {
        TSNumberKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_number_keyword(self, span: Span) -> Box<'a, TSNumberKeyword> {
        self.ts_number_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_never_keyword(self, span: Span) -> TSNeverKeyword {
        TSNeverKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_never_keyword(self, span: Span) -> Box<'a, TSNeverKeyword> {
        self.ts_never_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_intrinsic_keyword(self, span: Span) -> TSIntrinsicKeyword {
        TSIntrinsicKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_intrinsic_keyword(self, span: Span) -> Box<'a, TSIntrinsicKeyword> {
        self.ts_intrinsic_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_unknown_keyword(self, span: Span) -> TSUnknownKeyword {
        TSUnknownKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_unknown_keyword(self, span: Span) -> Box<'a, TSUnknownKeyword> {
        self.ts_unknown_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_null_keyword(self, span: Span) -> TSNullKeyword {
        TSNullKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_null_keyword(self, span: Span) -> Box<'a, TSNullKeyword> {
        self.ts_null_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_undefined_keyword(self, span: Span) -> TSUndefinedKeyword {
        TSUndefinedKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_undefined_keyword(self, span: Span) -> Box<'a, TSUndefinedKeyword> {
        self.ts_undefined_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_void_keyword(self, span: Span) -> TSVoidKeyword {
        TSVoidKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_void_keyword(self, span: Span) -> Box<'a, TSVoidKeyword> {
        self.ts_void_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_symbol_keyword(self, span: Span) -> TSSymbolKeyword {
        TSSymbolKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_symbol_keyword(self, span: Span) -> Box<'a, TSSymbolKeyword> {
        self.ts_symbol_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_this_type(self, span: Span) -> TSThisType {
        TSThisType { span }
    }

    #[inline]
    pub fn alloc_ts_this_type(self, span: Span) -> Box<'a, TSThisType> {
        self.ts_this_type(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_object_keyword(self, span: Span) -> TSObjectKeyword {
        TSObjectKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_object_keyword(self, span: Span) -> Box<'a, TSObjectKeyword> {
        self.ts_object_keyword(span).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_big_int_keyword(self, span: Span) -> TSBigIntKeyword {
        TSBigIntKeyword { span }
    }

    #[inline]
    pub fn alloc_ts_big_int_keyword(self, span: Span) -> Box<'a, TSBigIntKeyword> {
        self.ts_big_int_keyword(span).into_in(self.allocator)
    }

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
        self.ts_type_reference(span, type_name, type_parameters).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_name_identifier_reference<A>(self, span: Span, name: A) -> TSTypeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        TSTypeName::IdentifierReference(self.alloc(self.identifier_reference(span, name)))
    }

    #[inline]
    pub fn ts_type_name_from_identifier_reference<T>(self, inner: T) -> TSTypeName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierReference<'a>>>,
    {
        TSTypeName::IdentifierReference(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_name_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSTypeName<'a> {
        TSTypeName::QualifiedName(self.alloc(self.ts_qualified_name(span, left, right)))
    }

    #[inline]
    pub fn ts_type_name_from_ts_qualified_name<T>(self, inner: T) -> TSTypeName<'a>
    where
        T: IntoIn<'a, Box<'a, TSQualifiedName<'a>>>,
    {
        TSTypeName::QualifiedName(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSQualifiedName<'a> {
        TSQualifiedName { span, left, right }
    }

    #[inline]
    pub fn alloc_ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> Box<'a, TSQualifiedName<'a>> {
        self.ts_qualified_name(span, left, right).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> TSTypeParameterInstantiation<'a> {
        TSTypeParameterInstantiation { span, params }
    }

    #[inline]
    pub fn alloc_ts_type_parameter_instantiation(
        self,
        span: Span,
        params: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTypeParameterInstantiation<'a>> {
        self.ts_type_parameter_instantiation(span, params).into_in(self.allocator)
    }

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
        TSTypeParameter {
            span,
            name,
            constraint,
            default,
            r#in,
            out,
            r#const,
            scope_id: Default::default(),
        }
    }

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
        self.ts_type_parameter(span, name, constraint, default, r#in, out, r#const)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> TSTypeParameterDeclaration<'a> {
        TSTypeParameterDeclaration { span, params }
    }

    #[inline]
    pub fn alloc_ts_type_parameter_declaration(
        self,
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
    ) -> Box<'a, TSTypeParameterDeclaration<'a>> {
        self.ts_type_parameter_declaration(span, params).into_in(self.allocator)
    }

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
        }
    }

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
        self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
            .into_in(self.allocator)
    }

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
        self.ts_class_implements(span, expression, type_parameters).into_in(self.allocator)
    }

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
        }
    }

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
        self.ts_interface_declaration(span, id, extends, type_parameters, body, declare)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> TSInterfaceBody<'a> {
        TSInterfaceBody { span, body }
    }

    #[inline]
    pub fn alloc_ts_interface_body(
        self,
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSInterfaceBody<'a>> {
        self.ts_interface_body(span, body).into_in(self.allocator)
    }

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
        self.ts_property_signature(span, computed, optional, readonly, key, type_annotation)
            .into_in(self.allocator)
    }

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

    #[inline]
    pub fn ts_signature_from_ts_index_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSIndexSignature<'a>>>,
    {
        TSSignature::TSIndexSignature(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_signature_from_ts_property_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSPropertySignature<'a>>>,
    {
        TSSignature::TSPropertySignature(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_signature_from_ts_call_signature_declaration<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSCallSignatureDeclaration<'a>>>,
    {
        TSSignature::TSCallSignatureDeclaration(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn ts_signature_from_ts_method_signature<T>(self, inner: T) -> TSSignature<'a>
    where
        T: IntoIn<'a, Box<'a, TSMethodSignature<'a>>>,
    {
        TSSignature::TSMethodSignature(inner.into_in(self.allocator))
    }

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
        self.ts_index_signature(span, parameters, type_annotation, readonly).into_in(self.allocator)
    }

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
        self.ts_call_signature_declaration(span, this_param, params, return_type, type_parameters)
            .into_in(self.allocator)
    }

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
        }
    }

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
        )
        .into_in(self.allocator)
    }

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
        }
    }

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
        self.ts_construct_signature_declaration(span, params, return_type, type_parameters)
            .into_in(self.allocator)
    }

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
        self.ts_index_signature_name(span, name, type_annotation).into_in(self.allocator)
    }

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
        self.ts_interface_heritage(span, expression, type_parameters).into_in(self.allocator)
    }

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
        self.ts_type_predicate(span, parameter_name, asserts, type_annotation)
            .into_in(self.allocator)
    }

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

    #[inline]
    pub fn ts_type_predicate_name_from_identifier_name<T>(self, inner: T) -> TSTypePredicateName<'a>
    where
        T: IntoIn<'a, Box<'a, IdentifierName<'a>>>,
    {
        TSTypePredicateName::Identifier(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn ts_type_predicate_name_this_type(self, span: Span) -> TSTypePredicateName<'a> {
        TSTypePredicateName::This(self.ts_this_type(span))
    }

    #[inline]
    pub fn ts_type_predicate_name_from_ts_this_type<T>(self, inner: T) -> TSTypePredicateName<'a>
    where
        T: IntoIn<'a, TSThisType>,
    {
        TSTypePredicateName::This(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn alloc_ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> Box<'a, TSModuleDeclaration<'a>> {
        self.ts_module_declaration(span, id, body, kind, declare).into_in(self.allocator)
    }

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

    #[inline]
    pub fn ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> TSModuleBlock<'a> {
        TSModuleBlock { span, directives, body }
    }

    #[inline]
    pub fn alloc_ts_module_block(
        self,
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
    ) -> Box<'a, TSModuleBlock<'a>> {
        self.ts_module_block(span, directives, body).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> TSTypeLiteral<'a> {
        TSTypeLiteral { span, members }
    }

    #[inline]
    pub fn alloc_ts_type_literal(
        self,
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
    ) -> Box<'a, TSTypeLiteral<'a>> {
        self.ts_type_literal(span, members).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSInferType<'a>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        TSInferType { span, type_parameter: type_parameter.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> Box<'a, TSInferType<'a>>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        self.ts_infer_type(span, type_parameter).into_in(self.allocator)
    }

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
        self.ts_type_query(span, expr_name, type_parameters).into_in(self.allocator)
    }

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
        self.ts_import_type(span, is_type_of, parameter, qualifier, attributes, type_parameters)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn ts_import_attributes(
        self,
        span: Span,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> TSImportAttributes<'a> {
        TSImportAttributes { span, elements }
    }

    #[inline]
    pub fn alloc_ts_import_attributes(
        self,
        span: Span,
        elements: Vec<'a, TSImportAttribute<'a>>,
    ) -> Box<'a, TSImportAttributes<'a>> {
        self.ts_import_attributes(span, elements).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> TSImportAttribute<'a> {
        TSImportAttribute { span, name, value }
    }

    #[inline]
    pub fn alloc_ts_import_attribute(
        self,
        span: Span,
        name: TSImportAttributeName<'a>,
        value: Expression<'a>,
    ) -> Box<'a, TSImportAttribute<'a>> {
        self.ts_import_attribute(span, name, value).into_in(self.allocator)
    }

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
        self.ts_function_type(span, this_param, params, return_type, type_parameters)
            .into_in(self.allocator)
    }

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
        self.ts_constructor_type(span, r#abstract, params, return_type, type_parameters)
            .into_in(self.allocator)
    }

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
        }
    }

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
        self.ts_mapped_type(span, type_parameter, name_type, type_annotation, optional, readonly)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> TSTemplateLiteralType<'a> {
        TSTemplateLiteralType { span, quasis, types }
    }

    #[inline]
    pub fn alloc_ts_template_literal_type(
        self,
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
    ) -> Box<'a, TSTemplateLiteralType<'a>> {
        self.ts_template_literal_type(span, quasis, types).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSAsExpression<'a> {
        TSAsExpression { span, expression, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSAsExpression<'a>> {
        self.ts_as_expression(span, expression, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSSatisfiesExpression<'a> {
        TSSatisfiesExpression { span, expression, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSSatisfiesExpression<'a>> {
        self.ts_satisfies_expression(span, expression, type_annotation).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSTypeAssertion<'a> {
        TSTypeAssertion { span, expression, type_annotation }
    }

    #[inline]
    pub fn alloc_ts_type_assertion(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Box<'a, TSTypeAssertion<'a>> {
        self.ts_type_assertion(span, expression, type_annotation).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Box<'a, TSImportEqualsDeclaration<'a>> {
        self.ts_import_equals_declaration(span, id, module_reference, import_kind)
            .into_in(self.allocator)
    }

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

    #[inline]
    pub fn ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSExternalModuleReference<'a> {
        TSExternalModuleReference { span, expression }
    }

    #[inline]
    pub fn alloc_ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> Box<'a, TSExternalModuleReference<'a>> {
        self.ts_external_module_reference(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSNonNullExpression<'a> {
        TSNonNullExpression { span, expression }
    }

    #[inline]
    pub fn alloc_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSNonNullExpression<'a>> {
        self.ts_non_null_expression(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn decorator(self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { span, expression }
    }

    #[inline]
    pub fn alloc_decorator(self, span: Span, expression: Expression<'a>) -> Box<'a, Decorator<'a>> {
        self.decorator(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSExportAssignment<'a> {
        TSExportAssignment { span, expression }
    }

    #[inline]
    pub fn alloc_ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, TSExportAssignment<'a>> {
        self.ts_export_assignment(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> TSNamespaceExportDeclaration<'a> {
        TSNamespaceExportDeclaration { span, id }
    }

    #[inline]
    pub fn alloc_ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> Box<'a, TSNamespaceExportDeclaration<'a>> {
        self.ts_namespace_export_declaration(span, id).into_in(self.allocator)
    }

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
        self.ts_instantiation_expression(span, expression, type_parameters).into_in(self.allocator)
    }

    #[inline]
    pub fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNullableType<'a> {
        JSDocNullableType { span, type_annotation, postfix }
    }

    #[inline]
    pub fn alloc_js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNullableType<'a>> {
        self.js_doc_nullable_type(span, type_annotation, postfix).into_in(self.allocator)
    }

    #[inline]
    pub fn js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNonNullableType<'a> {
        JSDocNonNullableType { span, type_annotation, postfix }
    }

    #[inline]
    pub fn alloc_js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> Box<'a, JSDocNonNullableType<'a>> {
        self.js_doc_non_nullable_type(span, type_annotation, postfix).into_in(self.allocator)
    }

    #[inline]
    pub fn js_doc_unknown_type(self, span: Span) -> JSDocUnknownType {
        JSDocUnknownType { span }
    }

    #[inline]
    pub fn alloc_js_doc_unknown_type(self, span: Span) -> Box<'a, JSDocUnknownType> {
        self.js_doc_unknown_type(span).into_in(self.allocator)
    }

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
        self.jsx_element(span, opening_element, closing_element, children).into_in(self.allocator)
    }

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
        self.jsx_opening_element(span, self_closing, name, attributes, type_parameters)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { span, name }
    }

    #[inline]
    pub fn alloc_jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> Box<'a, JSXClosingElement<'a>> {
        self.jsx_closing_element(span, name).into_in(self.allocator)
    }

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

    #[inline]
    pub fn alloc_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        closing_fragment: JSXClosingFragment,
        children: Vec<'a, JSXChild<'a>>,
    ) -> Box<'a, JSXFragment<'a>> {
        self.jsx_fragment(span, opening_fragment, closing_fragment, children)
            .into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_element_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXElementName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXElementName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    #[inline]
    pub fn jsx_element_name_from_jsx_identifier<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXIdentifier<'a>>>,
    {
        JSXElementName::Identifier(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_element_name_from_jsx_namespaced_name<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXNamespacedName<'a>>>,
    {
        JSXElementName::NamespacedName(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_element_name_from_jsx_member_expression<T>(self, inner: T) -> JSXElementName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXMemberExpression<'a>>>,
    {
        JSXElementName::MemberExpression(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXNamespacedName<'a> {
        JSXNamespacedName { span, namespace, property }
    }

    #[inline]
    pub fn alloc_jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXNamespacedName<'a>> {
        self.jsx_namespaced_name(span, namespace, property).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpression<'a> {
        JSXMemberExpression { span, object, property }
    }

    #[inline]
    pub fn alloc_jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> Box<'a, JSXMemberExpression<'a>> {
        self.jsx_member_expression(span, object, property).into_in(self.allocator)
    }

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

    #[inline]
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { span, expression }
    }

    #[inline]
    pub fn alloc_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> Box<'a, JSXExpressionContainer<'a>> {
        self.jsx_expression_container(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_expression_jsx_empty_expression(self, span: Span) -> JSXExpression<'a> {
        JSXExpression::EmptyExpression(self.jsx_empty_expression(span))
    }

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

    #[inline]
    pub fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { span }
    }

    #[inline]
    pub fn alloc_jsx_empty_expression(self, span: Span) -> Box<'a, JSXEmptyExpression> {
        self.jsx_empty_expression(span).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_attribute_item_jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::Attribute(self.alloc(self.jsx_attribute(span, name, value)))
    }

    #[inline]
    pub fn jsx_attribute_item_from_jsx_attribute<T>(self, inner: T) -> JSXAttributeItem<'a>
    where
        T: IntoIn<'a, Box<'a, JSXAttribute<'a>>>,
    {
        JSXAttributeItem::Attribute(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_attribute_item_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::SpreadAttribute(self.alloc(self.jsx_spread_attribute(span, argument)))
    }

    #[inline]
    pub fn jsx_attribute_item_from_jsx_spread_attribute<T>(self, inner: T) -> JSXAttributeItem<'a>
    where
        T: IntoIn<'a, Box<'a, JSXSpreadAttribute<'a>>>,
    {
        JSXAttributeItem::SpreadAttribute(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttribute<'a> {
        JSXAttribute { span, name, value }
    }

    #[inline]
    pub fn alloc_jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> Box<'a, JSXAttribute<'a>> {
        self.jsx_attribute(span, name, value).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXSpreadAttribute<'a> {
        JSXSpreadAttribute { span, argument }
    }

    #[inline]
    pub fn alloc_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> Box<'a, JSXSpreadAttribute<'a>> {
        self.jsx_spread_attribute(span, argument).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_attribute_name_jsx_identifier<A>(self, span: Span, name: A) -> JSXAttributeName<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXAttributeName::Identifier(self.alloc(self.jsx_identifier(span, name)))
    }

    #[inline]
    pub fn jsx_attribute_name_from_jsx_identifier<T>(self, inner: T) -> JSXAttributeName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXIdentifier<'a>>>,
    {
        JSXAttributeName::Identifier(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_attribute_name_from_jsx_namespaced_name<T>(self, inner: T) -> JSXAttributeName<'a>
    where
        T: IntoIn<'a, Box<'a, JSXNamespacedName<'a>>>,
    {
        JSXAttributeName::NamespacedName(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_attribute_value_from_string_literal<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, StringLiteral<'a>>>,
    {
        JSXAttributeValue::StringLiteral(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_attribute_value_from_jsx_element<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        JSXAttributeValue::Element(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_attribute_value_from_jsx_fragment<T>(self, inner: T) -> JSXAttributeValue<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        JSXAttributeValue::Fragment(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_identifier<A>(self, span: Span, name: A) -> JSXIdentifier<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXIdentifier { span, name: name.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_jsx_identifier<A>(self, span: Span, name: A) -> Box<'a, JSXIdentifier<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.jsx_identifier(span, name).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_child_jsx_text<A>(self, span: Span, value: A) -> JSXChild<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXChild::Text(self.alloc(self.jsx_text(span, value)))
    }

    #[inline]
    pub fn jsx_child_from_jsx_text<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXText<'a>>>,
    {
        JSXChild::Text(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_child_from_jsx_element<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXElement<'a>>>,
    {
        JSXChild::Element(inner.into_in(self.allocator))
    }

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

    #[inline]
    pub fn jsx_child_from_jsx_fragment<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXFragment<'a>>>,
    {
        JSXChild::Fragment(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_child_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::ExpressionContainer(self.alloc(self.jsx_expression_container(span, expression)))
    }

    #[inline]
    pub fn jsx_child_from_jsx_expression_container<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXExpressionContainer<'a>>>,
    {
        JSXChild::ExpressionContainer(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_child_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::Spread(self.alloc(self.jsx_spread_child(span, expression)))
    }

    #[inline]
    pub fn jsx_child_from_jsx_spread_child<T>(self, inner: T) -> JSXChild<'a>
    where
        T: IntoIn<'a, Box<'a, JSXSpreadChild<'a>>>,
    {
        JSXChild::Spread(inner.into_in(self.allocator))
    }

    #[inline]
    pub fn jsx_spread_child(self, span: Span, expression: Expression<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { span, expression }
    }

    #[inline]
    pub fn alloc_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Box<'a, JSXSpreadChild<'a>> {
        self.jsx_spread_child(span, expression).into_in(self.allocator)
    }

    #[inline]
    pub fn jsx_text<A>(self, span: Span, value: A) -> JSXText<'a>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        JSXText { span, value: value.into_in(self.allocator) }
    }

    #[inline]
    pub fn alloc_jsx_text<A>(self, span: Span, value: A) -> Box<'a, JSXText<'a>>
    where
        A: IntoIn<'a, Atom<'a>>,
    {
        self.jsx_text(span, value).into_in(self.allocator)
    }
}
