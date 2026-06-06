// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/builder_methods.rs`.

//! AST node builder methods.

#![expect(clippy::default_trait_access)]

use std::cell::Cell;

use oxc_allocator::{Box, GetAllocator, IntoIn, Vec};
use oxc_str::{Ident, Str};
use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::{
    ast::*,
    builder::{AstBuild, GetAstBuilder},
};

impl<'a> Program<'a> {
    /// Build a [`Program`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source_type`
    /// * `source_text`
    /// * `comments`: Sorted comments
    /// * `hashbang`
    /// * `directives`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        Program {
            node_id: Cell::new(builder.node_id()),
            span,
            source_type,
            source_text,
            comments,
            hashbang,
            directives,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`Program`] with `scope_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source_type`
    /// * `source_text`
    /// * `comments`: Sorted comments
    /// * `hashbang`
    /// * `directives`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: Vec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        Program {
            node_id: Cell::new(builder.node_id()),
            span,
            source_type,
            source_text,
            comments,
            hashbang,
            directives,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }
}

impl<'a> Expression<'a> {
    /// Build an [`Expression::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build an [`Expression::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build an [`Expression::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`Expression::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`Expression::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build an [`Expression::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build an [`Expression::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build an [`Expression::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build an [`Expression::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`Expression::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`Expression::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build an [`Expression::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build an [`Expression::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build an [`Expression::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`Expression::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build an [`Expression::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build an [`Expression::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build an [`Expression::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`Expression::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build an [`Expression::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build an [`Expression::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Expression::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build an [`Expression::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`Expression::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build an [`Expression::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`Expression::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build an [`Expression::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build an [`Expression::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`Expression::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build an [`Expression::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build an [`Expression::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build an [`Expression::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Expression::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build an [`Expression::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build an [`Expression::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build an [`Expression::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build an [`Expression::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build an [`Expression::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build an [`Expression::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build an [`Expression::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build an [`Expression::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build an [`Expression::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`Expression::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`Expression::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`Expression::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Expression::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build an [`Expression::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build an [`Expression::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`Expression::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`Expression::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> IdentifierName<'a> {
    /// Build an [`IdentifierName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`IdentifierName::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        IdentifierName { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build an [`IdentifierName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }
}

impl<'a> IdentifierReference<'a> {
    /// Build an [`IdentifierReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`IdentifierReference::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        IdentifierReference {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            reference_id: Default::default(),
        }
    }

    /// Build an [`IdentifierReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }

    /// Build an [`IdentifierReference`] with `reference_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`IdentifierReference::boxed_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        IdentifierReference {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            reference_id: Cell::new(Some(reference_id)),
        }
    }

    /// Build an [`IdentifierReference`] with `reference_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierReference::new_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn boxed_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(
            Self::new_with_reference_id(span, name, reference_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> BindingIdentifier<'a> {
    /// Build a [`BindingIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BindingIdentifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        BindingIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            symbol_id: Default::default(),
        }
    }

    /// Build a [`BindingIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }

    /// Build a [`BindingIdentifier`] with `symbol_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BindingIdentifier::boxed_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn new_with_symbol_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        BindingIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            symbol_id: Cell::new(Some(symbol_id)),
        }
    }

    /// Build a [`BindingIdentifier`] with `symbol_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingIdentifier::new_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn boxed_with_symbol_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(
            Self::new_with_symbol_id(span, name, symbol_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> LabelIdentifier<'a> {
    /// Build a [`LabelIdentifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        LabelIdentifier { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }
}

impl ThisExpression {
    /// Build a [`ThisExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ThisExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        ThisExpression { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`ThisExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ThisExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> ArrayExpression<'a> {
    /// Build an [`ArrayExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ArrayExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ArrayExpression { node_id: Cell::new(builder.node_id()), span, elements }
    }

    /// Build an [`ArrayExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, elements, accessor), accessor.builder().allocator())
    }
}

impl<'a> ArrayExpressionElement<'a> {
    /// Build an [`ArrayExpressionElement::SpreadElement`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new_spread_element<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::SpreadElement(SpreadElement::boxed(span, argument, accessor))
    }

    /// Build an [`ArrayExpressionElement::Elision`].
    ///
    /// This node contains an [`Elision`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_elision<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Elision(Elision::boxed(span, accessor))
    }

    /// Build an [`ArrayExpressionElement::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build an [`ArrayExpressionElement::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build an [`ArrayExpressionElement::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`ArrayExpressionElement::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`ArrayExpressionElement::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build an [`ArrayExpressionElement::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build an [`ArrayExpressionElement::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build an [`ArrayExpressionElement::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`ArrayExpressionElement::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build an [`ArrayExpressionElement::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build an [`ArrayExpressionElement::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build an [`ArrayExpressionElement::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build an [`ArrayExpressionElement::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build an [`ArrayExpressionElement::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`ArrayExpressionElement::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ArrayExpressionElement::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build an [`ArrayExpressionElement::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`ArrayExpressionElement::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build an [`ArrayExpressionElement::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build an [`ArrayExpressionElement::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ArrayExpressionElement::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build an [`ArrayExpressionElement::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build an [`ArrayExpressionElement::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build an [`ArrayExpressionElement::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build an [`ArrayExpressionElement::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build an [`ArrayExpressionElement::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build an [`ArrayExpressionElement::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`ArrayExpressionElement::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`ArrayExpressionElement::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ArrayExpressionElement::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build an [`ArrayExpressionElement::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`ArrayExpressionElement::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl Elision {
    /// Build an [`Elision`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Elision::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        Elision { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build an [`Elision`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Elision::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> ObjectExpression<'a> {
    /// Build an [`ObjectExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ObjectExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ObjectExpression { node_id: Cell::new(builder.node_id()), span, properties }
    }

    /// Build an [`ObjectExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, properties, accessor), accessor.builder().allocator())
    }
}

impl<'a> ObjectPropertyKind<'a> {
    /// Build an [`ObjectPropertyKind::ObjectProperty`].
    ///
    /// This node contains an [`ObjectProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `key`
    /// * `value`
    /// * `method`
    /// * `shorthand`
    /// * `computed`
    #[inline]
    pub fn new_object_property<A: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        accessor: &A,
    ) -> Self {
        Self::ObjectProperty(ObjectProperty::boxed(
            span, kind, key, value, method, shorthand, computed, accessor,
        ))
    }

    /// Build an [`ObjectPropertyKind::SpreadProperty`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new_spread_property<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::SpreadProperty(SpreadElement::boxed(span, argument, accessor))
    }
}

impl<'a> ObjectProperty<'a> {
    /// Build an [`ObjectProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ObjectProperty::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `key`
    /// * `value`
    /// * `method`
    /// * `shorthand`
    /// * `computed`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ObjectProperty {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            key,
            value,
            method,
            shorthand,
            computed,
        }
    }

    /// Build an [`ObjectProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectProperty::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `key`
    /// * `value`
    /// * `method`
    /// * `shorthand`
    /// * `computed`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, kind, key, value, method, shorthand, computed, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> PropertyKey<'a> {
    /// Build a [`PropertyKey::StaticIdentifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_static_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::StaticIdentifier(IdentifierName::boxed(span, name, accessor))
    }

    /// Build a [`PropertyKey::PrivateIdentifier`].
    ///
    /// This node contains a [`PrivateIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_private_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::PrivateIdentifier(PrivateIdentifier::boxed(span, name, accessor))
    }

    /// Build a [`PropertyKey::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build a [`PropertyKey::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build a [`PropertyKey::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`PropertyKey::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`PropertyKey::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build a [`PropertyKey::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`PropertyKey::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build a [`PropertyKey::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`PropertyKey::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build a [`PropertyKey::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build a [`PropertyKey::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build a [`PropertyKey::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build a [`PropertyKey::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build a [`PropertyKey::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build a [`PropertyKey::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`PropertyKey::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build a [`PropertyKey::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build a [`PropertyKey::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build a [`PropertyKey::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`PropertyKey::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build a [`PropertyKey::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build a [`PropertyKey::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build a [`PropertyKey::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build a [`PropertyKey::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build a [`PropertyKey::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build a [`PropertyKey::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build a [`PropertyKey::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build a [`PropertyKey::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build a [`PropertyKey::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build a [`PropertyKey::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build a [`PropertyKey::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`PropertyKey::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`PropertyKey::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build a [`PropertyKey::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`PropertyKey::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`PropertyKey::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> TemplateLiteral<'a> {
    /// Build a [`TemplateLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TemplateLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TemplateLiteral { node_id: Cell::new(builder.node_id()), span, quasis, expressions }
    }

    /// Build a [`TemplateLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TemplateLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, quasis, expressions, accessor), accessor.builder().allocator())
    }
}

impl<'a> TaggedTemplateExpression<'a> {
    /// Build a [`TaggedTemplateExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TaggedTemplateExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TaggedTemplateExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            tag,
            type_arguments: type_arguments.into_in(builder.allocator()),
            quasi,
        }
    }

    /// Build a [`TaggedTemplateExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TaggedTemplateExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, tag, type_arguments, quasi, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TemplateElement<'a> {
    /// Build a [`TemplateElement`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    /// * `tail`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TemplateElement {
            node_id: Cell::new(builder.node_id()),
            span,
            value,
            tail,
            lone_surrogates: Default::default(),
        }
    }

    /// Build a [`TemplateElement`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    /// * `tail`
    /// * `lone_surrogates`: The template element contains lone surrogates.
    #[inline]
    pub fn new_with_lone_surrogates<A: GetAstBuilder<'a>>(
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TemplateElement {
            node_id: Cell::new(builder.node_id()),
            span,
            value,
            tail,
            lone_surrogates,
        }
    }
}

impl<'a> MemberExpression<'a> {
    /// Build a [`MemberExpression::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`MemberExpression::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`MemberExpression::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> ComputedMemberExpression<'a> {
    /// Build a [`ComputedMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ComputedMemberExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ComputedMemberExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            object,
            expression,
            optional,
        }
    }

    /// Build a [`ComputedMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ComputedMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, object, expression, optional, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> StaticMemberExpression<'a> {
    /// Build a [`StaticMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`StaticMemberExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        StaticMemberExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            object,
            property,
            optional,
        }
    }

    /// Build a [`StaticMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, object, property, optional, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> PrivateFieldExpression<'a> {
    /// Build a [`PrivateFieldExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`PrivateFieldExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        PrivateFieldExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            object,
            field,
            optional,
        }
    }

    /// Build a [`PrivateFieldExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateFieldExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, object, field, optional, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> CallExpression<'a> {
    /// Build a [`CallExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`CallExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        CallExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments,
            optional,
            pure: Default::default(),
        }
    }

    /// Build a [`CallExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CallExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, callee, type_arguments, arguments, optional, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`CallExpression`] with `pure`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`CallExpression::boxed_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        CallExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments,
            optional,
            pure,
        }
    }

    /// Build a [`CallExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CallExpression::new_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn boxed_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new_with_pure(span, callee, type_arguments, arguments, optional, pure, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> NewExpression<'a> {
    /// Build a [`NewExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`NewExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        NewExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments,
            pure: Default::default(),
        }
    }

    /// Build a [`NewExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NewExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, callee, type_arguments, arguments, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`NewExpression`] with `pure`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`NewExpression::boxed_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        NewExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments,
            pure,
        }
    }

    /// Build a [`NewExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NewExpression::new_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn boxed_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new_with_pure(span, callee, type_arguments, arguments, pure, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> MetaProperty<'a> {
    /// Build a [`MetaProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`MetaProperty::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        MetaProperty { node_id: Cell::new(builder.node_id()), span, meta, property }
    }

    /// Build a [`MetaProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`MetaProperty::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, meta, property, accessor), accessor.builder().allocator())
    }
}

impl<'a> SpreadElement<'a> {
    /// Build a [`SpreadElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`SpreadElement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        SpreadElement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`SpreadElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SpreadElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> Argument<'a> {
    /// Build an [`Argument::SpreadElement`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new_spread_element<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::SpreadElement(SpreadElement::boxed(span, argument, accessor))
    }

    /// Build an [`Argument::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build an [`Argument::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build an [`Argument::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`Argument::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`Argument::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build an [`Argument::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build an [`Argument::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build an [`Argument::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build an [`Argument::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`Argument::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`Argument::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build an [`Argument::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build an [`Argument::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build an [`Argument::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`Argument::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build an [`Argument::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build an [`Argument::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build an [`Argument::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`Argument::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build an [`Argument::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build an [`Argument::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Argument::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build an [`Argument::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`Argument::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build an [`Argument::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`Argument::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build an [`Argument::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build an [`Argument::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`Argument::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build an [`Argument::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build an [`Argument::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build an [`Argument::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Argument::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build an [`Argument::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build an [`Argument::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build an [`Argument::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build an [`Argument::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build an [`Argument::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build an [`Argument::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build an [`Argument::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build an [`Argument::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build an [`Argument::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`Argument::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`Argument::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`Argument::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`Argument::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build an [`Argument::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build an [`Argument::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`Argument::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`Argument::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> UpdateExpression<'a> {
    /// Build an [`UpdateExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`UpdateExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        UpdateExpression { node_id: Cell::new(builder.node_id()), span, operator, prefix, argument }
    }

    /// Build an [`UpdateExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`UpdateExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, operator, prefix, argument, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> UnaryExpression<'a> {
    /// Build an [`UnaryExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`UnaryExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        UnaryExpression { node_id: Cell::new(builder.node_id()), span, operator, argument }
    }

    /// Build an [`UnaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`UnaryExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, operator, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> BinaryExpression<'a> {
    /// Build a [`BinaryExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BinaryExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BinaryExpression { node_id: Cell::new(builder.node_id()), span, left, operator, right }
    }

    /// Build a [`BinaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BinaryExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, left, operator, right, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> PrivateInExpression<'a> {
    /// Build a [`PrivateInExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`PrivateInExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        PrivateInExpression { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`PrivateInExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateInExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, left, right, accessor), accessor.builder().allocator())
    }
}

impl<'a> LogicalExpression<'a> {
    /// Build a [`LogicalExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`LogicalExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        LogicalExpression { node_id: Cell::new(builder.node_id()), span, left, operator, right }
    }

    /// Build a [`LogicalExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`LogicalExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, left, operator, right, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ConditionalExpression<'a> {
    /// Build a [`ConditionalExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ConditionalExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ConditionalExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            test,
            consequent,
            alternate,
        }
    }

    /// Build a [`ConditionalExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ConditionalExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, test, consequent, alternate, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> AssignmentExpression<'a> {
    /// Build an [`AssignmentExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentExpression { node_id: Cell::new(builder.node_id()), span, operator, left, right }
    }

    /// Build an [`AssignmentExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, operator, left, right, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> AssignmentTarget<'a> {
    /// Build an [`AssignmentTarget::AssignmentTargetIdentifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_assignment_target_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`AssignmentTarget::AssignmentTargetIdentifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_assignment_target_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`AssignmentTarget::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`AssignmentTarget::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`AssignmentTarget::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`AssignmentTarget::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`AssignmentTarget::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTarget::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTarget::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTarget::ArrayAssignmentTarget`].
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new_array_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, accessor))
    }

    /// Build an [`AssignmentTarget::ObjectAssignmentTarget`].
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new_object_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(
            span, properties, rest, accessor,
        ))
    }
}

impl<'a> SimpleAssignmentTarget<'a> {
    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_assignment_target_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_assignment_target_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build a [`SimpleAssignmentTarget::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`SimpleAssignmentTarget::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build a [`SimpleAssignmentTarget::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> AssignmentTargetPattern<'a> {
    /// Build an [`AssignmentTargetPattern::ArrayAssignmentTarget`].
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new_array_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, accessor))
    }

    /// Build an [`AssignmentTargetPattern::ObjectAssignmentTarget`].
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new_object_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(
            span, properties, rest, accessor,
        ))
    }
}

impl<'a> ArrayAssignmentTarget<'a> {
    /// Build an [`ArrayAssignmentTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ArrayAssignmentTarget::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        let builder = accessor.builder();
        ArrayAssignmentTarget {
            node_id: Cell::new(builder.node_id()),
            span,
            elements,
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ArrayAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayAssignmentTarget::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Box::new_in(Self::new(span, elements, rest, accessor), accessor.builder().allocator())
    }
}

impl<'a> ObjectAssignmentTarget<'a> {
    /// Build an [`ObjectAssignmentTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ObjectAssignmentTarget::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        let builder = accessor.builder();
        ObjectAssignmentTarget {
            node_id: Cell::new(builder.node_id()),
            span,
            properties,
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ObjectAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectAssignmentTarget::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Box::new_in(Self::new(span, properties, rest, accessor), accessor.builder().allocator())
    }
}

impl<'a> AssignmentTargetRest<'a> {
    /// Build an [`AssignmentTargetRest`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentTargetRest::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `target`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        target: AssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentTargetRest { node_id: Cell::new(builder.node_id()), span, target }
    }

    /// Build an [`AssignmentTargetRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetRest::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `target`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        target: AssignmentTarget<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, target, accessor), accessor.builder().allocator())
    }
}

impl<'a> AssignmentTargetMaybeDefault<'a> {
    /// Build an [`AssignmentTargetMaybeDefault::AssignmentTargetWithDefault`].
    ///
    /// This node contains an [`AssignmentTargetWithDefault`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn new_assignment_target_with_default<A: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentTargetWithDefault(AssignmentTargetWithDefault::boxed(
            span, binding, init, accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::AssignmentTargetIdentifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_assignment_target_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`AssignmentTargetMaybeDefault::AssignmentTargetIdentifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_assignment_target_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`AssignmentTargetMaybeDefault::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`AssignmentTargetMaybeDefault::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`AssignmentTargetMaybeDefault::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }

    /// Build an [`AssignmentTargetMaybeDefault::ArrayAssignmentTarget`].
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new_array_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, accessor))
    }

    /// Build an [`AssignmentTargetMaybeDefault::ObjectAssignmentTarget`].
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new_object_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(
            span, properties, rest, accessor,
        ))
    }
}

impl<'a> AssignmentTargetWithDefault<'a> {
    /// Build an [`AssignmentTargetWithDefault`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentTargetWithDefault::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentTargetWithDefault { node_id: Cell::new(builder.node_id()), span, binding, init }
    }

    /// Build an [`AssignmentTargetWithDefault`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetWithDefault::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, binding, init, accessor), accessor.builder().allocator())
    }
}

impl<'a> AssignmentTargetProperty<'a> {
    /// Build an [`AssignmentTargetProperty::AssignmentTargetPropertyIdentifier`].
    ///
    /// This node contains an [`AssignmentTargetPropertyIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn new_assignment_target_property_identifier<A: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentTargetPropertyIdentifier(AssignmentTargetPropertyIdentifier::boxed(
            span, binding, init, accessor,
        ))
    }

    /// Build an [`AssignmentTargetProperty::AssignmentTargetPropertyProperty`].
    ///
    /// This node contains an [`AssignmentTargetPropertyProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[inline]
    pub fn new_assignment_target_property_property<A: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        accessor: &A,
    ) -> Self {
        Self::AssignmentTargetPropertyProperty(AssignmentTargetPropertyProperty::boxed(
            span, name, binding, computed, accessor,
        ))
    }
}

impl<'a> AssignmentTargetPropertyIdentifier<'a> {
    /// Build an [`AssignmentTargetPropertyIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentTargetPropertyIdentifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentTargetPropertyIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            binding,
            init,
        }
    }

    /// Build an [`AssignmentTargetPropertyIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetPropertyIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, binding, init, accessor), accessor.builder().allocator())
    }
}

impl<'a> AssignmentTargetPropertyProperty<'a> {
    /// Build an [`AssignmentTargetPropertyProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentTargetPropertyProperty::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentTargetPropertyProperty {
            node_id: Cell::new(builder.node_id()),
            span,
            name,
            binding,
            computed,
        }
    }

    /// Build an [`AssignmentTargetPropertyProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetPropertyProperty::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, name, binding, computed, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> SequenceExpression<'a> {
    /// Build a [`SequenceExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`SequenceExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        SequenceExpression { node_id: Cell::new(builder.node_id()), span, expressions }
    }

    /// Build a [`SequenceExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SequenceExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expressions, accessor), accessor.builder().allocator())
    }
}

impl Super {
    /// Build a [`Super`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Super::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        Super { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`Super`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Super::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> AwaitExpression<'a> {
    /// Build an [`AwaitExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AwaitExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        AwaitExpression { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build an [`AwaitExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AwaitExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> ChainExpression<'a> {
    /// Build a [`ChainExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ChainExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ChainExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`ChainExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ChainExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> ChainElement<'a> {
    /// Build a [`ChainElement::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build a [`ChainElement::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build a [`ChainElement::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`ChainElement::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`ChainElement::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`ChainElement::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> ParenthesizedExpression<'a> {
    /// Build a [`ParenthesizedExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ParenthesizedExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        ParenthesizedExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`ParenthesizedExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ParenthesizedExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> Statement<'a> {
    /// Build a [`Statement::BlockStatement`].
    ///
    /// This node contains a [`BlockStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new_block_statement<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::BlockStatement(BlockStatement::boxed(span, body, accessor))
    }

    /// Build a [`Statement::BlockStatement`] with `scope_id`.
    ///
    /// This node contains a [`BlockStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_block_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::BlockStatement(BlockStatement::boxed_with_scope_id(span, body, scope_id, accessor))
    }

    /// Build a [`Statement::BreakStatement`].
    ///
    /// This node contains a [`BreakStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new_break_statement<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Self {
        Self::BreakStatement(BreakStatement::boxed(span, label, accessor))
    }

    /// Build a [`Statement::ContinueStatement`].
    ///
    /// This node contains a [`ContinueStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new_continue_statement<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ContinueStatement(ContinueStatement::boxed(span, label, accessor))
    }

    /// Build a [`Statement::DebuggerStatement`].
    ///
    /// This node contains a [`DebuggerStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_debugger_statement<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::DebuggerStatement(DebuggerStatement::boxed(span, accessor))
    }

    /// Build a [`Statement::DoWhileStatement`].
    ///
    /// This node contains a [`DoWhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[inline]
    pub fn new_do_while_statement<A: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::DoWhileStatement(DoWhileStatement::boxed(span, body, test, accessor))
    }

    /// Build a [`Statement::EmptyStatement`].
    ///
    /// This node contains an [`EmptyStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_empty_statement<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::EmptyStatement(EmptyStatement::boxed(span, accessor))
    }

    /// Build a [`Statement::ExpressionStatement`].
    ///
    /// This node contains an [`ExpressionStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_expression_statement<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExpressionStatement(ExpressionStatement::boxed(span, expression, accessor))
    }

    /// Build a [`Statement::ForInStatement`].
    ///
    /// This node contains a [`ForInStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn new_for_in_statement<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ForInStatement(ForInStatement::boxed(span, left, right, body, accessor))
    }

    /// Build a [`Statement::ForInStatement`] with `scope_id`.
    ///
    /// This node contains a [`ForInStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_for_in_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::ForInStatement(ForInStatement::boxed_with_scope_id(
            span, left, right, body, scope_id, accessor,
        ))
    }

    /// Build a [`Statement::ForOfStatement`].
    ///
    /// This node contains a [`ForOfStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn new_for_of_statement<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ForOfStatement(ForOfStatement::boxed(span, r#await, left, right, body, accessor))
    }

    /// Build a [`Statement::ForOfStatement`] with `scope_id`.
    ///
    /// This node contains a [`ForOfStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_for_of_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::ForOfStatement(ForOfStatement::boxed_with_scope_id(
            span, r#await, left, right, body, scope_id, accessor,
        ))
    }

    /// Build a [`Statement::ForStatement`].
    ///
    /// This node contains a [`ForStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[inline]
    pub fn new_for_statement<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ForStatement(ForStatement::boxed(span, init, test, update, body, accessor))
    }

    /// Build a [`Statement::ForStatement`] with `scope_id`.
    ///
    /// This node contains a [`ForStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_for_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::ForStatement(ForStatement::boxed_with_scope_id(
            span, init, test, update, body, scope_id, accessor,
        ))
    }

    /// Build a [`Statement::IfStatement`].
    ///
    /// This node contains an [`IfStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_if_statement<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::IfStatement(IfStatement::boxed(span, test, consequent, alternate, accessor))
    }

    /// Build a [`Statement::LabeledStatement`].
    ///
    /// This node contains a [`LabeledStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[inline]
    pub fn new_labeled_statement<A: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::LabeledStatement(LabeledStatement::boxed(span, label, body, accessor))
    }

    /// Build a [`Statement::ReturnStatement`].
    ///
    /// This node contains a [`ReturnStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_return_statement<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ReturnStatement(ReturnStatement::boxed(span, argument, accessor))
    }

    /// Build a [`Statement::SwitchStatement`].
    ///
    /// This node contains a [`SwitchStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[inline]
    pub fn new_switch_statement<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SwitchStatement(SwitchStatement::boxed(span, discriminant, cases, accessor))
    }

    /// Build a [`Statement::SwitchStatement`] with `scope_id`.
    ///
    /// This node contains a [`SwitchStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[inline]
    pub fn new_switch_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::SwitchStatement(SwitchStatement::boxed_with_scope_id(
            span,
            discriminant,
            cases,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Statement::ThrowStatement`].
    ///
    /// This node contains a [`ThrowStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn new_throw_statement<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ThrowStatement(ThrowStatement::boxed(span, argument, accessor))
    }

    /// Build a [`Statement::TryStatement`].
    ///
    /// This node contains a [`TryStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[inline]
    pub fn new_try_statement<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        Self::TryStatement(TryStatement::boxed(span, block, handler, finalizer, accessor))
    }

    /// Build a [`Statement::WhileStatement`].
    ///
    /// This node contains a [`WhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[inline]
    pub fn new_while_statement<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::WhileStatement(WhileStatement::boxed(span, test, body, accessor))
    }

    /// Build a [`Statement::WithStatement`].
    ///
    /// This node contains a [`WithStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[inline]
    pub fn new_with_statement<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        Self::WithStatement(WithStatement::boxed(span, object, body, accessor))
    }

    /// Build a [`Statement::WithStatement`] with `scope_id`.
    ///
    /// This node contains a [`WithStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_statement_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::WithStatement(WithStatement::boxed_with_scope_id(
            span, object, body, scope_id, accessor,
        ))
    }

    /// Build a [`Statement::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn new_variable_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            accessor,
        ))
    }

    /// Build a [`Statement::FunctionDeclaration`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_declaration<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`Statement::FunctionDeclaration`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_declaration_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build a [`Statement::ClassDeclaration`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_declaration<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build a [`Statement::ClassDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Statement::TSTypeAliasDeclaration`].
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[inline]
    pub fn new_ts_type_alias_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            accessor,
        ))
    }

    /// Build a [`Statement::TSTypeAliasDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_type_alias_declaration_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Statement::TSInterfaceDeclaration`].
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[inline]
    pub fn new_ts_interface_declaration<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            accessor,
        ))
    }

    /// Build a [`Statement::TSInterfaceDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_interface_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Statement::TSEnumDeclaration`].
    ///
    /// This node contains a [`TSEnumDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[inline]
    pub fn new_ts_enum_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSEnumDeclaration(TSEnumDeclaration::boxed(
            span, id, body, r#const, declare, accessor,
        ))
    }

    /// Build a [`Statement::TSModuleDeclaration`].
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn new_ts_module_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, accessor,
        ))
    }

    /// Build a [`Statement::TSModuleDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_module_declaration_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, accessor,
        ))
    }

    /// Build a [`Statement::TSGlobalDeclaration`].
    ///
    /// This node contains a [`TSGlobalDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[inline]
    pub fn new_ts_global_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed(
            span,
            global_span,
            body,
            declare,
            accessor,
        ))
    }

    /// Build a [`Statement::TSGlobalDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSGlobalDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_global_declaration_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed_with_scope_id(
            span,
            global_span,
            body,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Statement::TSImportEqualsDeclaration`].
    ///
    /// This node contains a [`TSImportEqualsDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[inline]
    pub fn new_ts_import_equals_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        Self::TSImportEqualsDeclaration(TSImportEqualsDeclaration::boxed(
            span,
            id,
            module_reference,
            import_kind,
            accessor,
        ))
    }

    /// Build a [`Statement::ImportDeclaration`].
    ///
    /// This node contains an [`ImportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[inline]
    pub fn new_import_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ImportDeclaration(ImportDeclaration::boxed(
            span,
            specifiers,
            source,
            phase,
            with_clause,
            import_kind,
            accessor,
        ))
    }

    /// Build a [`Statement::ExportAllDeclaration`].
    ///
    /// This node contains an [`ExportAllDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[inline]
    pub fn new_export_all_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ExportAllDeclaration(ExportAllDeclaration::boxed(
            span,
            exported,
            source,
            with_clause,
            export_kind,
            accessor,
        ))
    }

    /// Build a [`Statement::ExportDefaultDeclaration`].
    ///
    /// This node contains an [`ExportDefaultDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[inline]
    pub fn new_export_default_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(span, declaration, accessor))
    }

    /// Build a [`Statement::ExportNamedDeclaration`].
    ///
    /// This node contains an [`ExportNamedDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[inline]
    pub fn new_export_named_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ExportNamedDeclaration(ExportNamedDeclaration::boxed(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            accessor,
        ))
    }

    /// Build a [`Statement::TSExportAssignment`].
    ///
    /// This node contains a [`TSExportAssignment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_export_assignment<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSExportAssignment(TSExportAssignment::boxed(span, expression, accessor))
    }

    /// Build a [`Statement::TSNamespaceExportDeclaration`].
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn new_ts_namespace_export_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNamespaceExportDeclaration(TSNamespaceExportDeclaration::boxed(span, id, accessor))
    }
}

impl<'a> Directive<'a> {
    /// Build a [`Directive`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Directive with any escapes unescaped
    /// * `directive`: Raw content of directive as it appears in source, any escapes left as is
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(
        span: Span,
        expression: StringLiteral<'a>,
        directive: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        Directive {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            directive: directive.into(),
        }
    }
}

impl<'a> Hashbang<'a> {
    /// Build a [`Hashbang`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, value: S1, accessor: &A) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        Hashbang { node_id: Cell::new(builder.node_id()), span, value: value.into() }
    }
}

impl<'a> BlockStatement<'a> {
    /// Build a [`BlockStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BlockStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BlockStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`BlockStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BlockStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, body, accessor), accessor.builder().allocator())
    }

    /// Build a [`BlockStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BlockStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BlockStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`BlockStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BlockStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> Declaration<'a> {
    /// Build a [`Declaration::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn new_variable_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            accessor,
        ))
    }

    /// Build a [`Declaration::FunctionDeclaration`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_declaration<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`Declaration::FunctionDeclaration`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_declaration_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build a [`Declaration::ClassDeclaration`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_declaration<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build a [`Declaration::ClassDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSTypeAliasDeclaration`].
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[inline]
    pub fn new_ts_type_alias_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSTypeAliasDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSTypeAliasDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_type_alias_declaration_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSInterfaceDeclaration`].
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[inline]
    pub fn new_ts_interface_declaration<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSInterfaceDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_interface_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSEnumDeclaration`].
    ///
    /// This node contains a [`TSEnumDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[inline]
    pub fn new_ts_enum_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSEnumDeclaration(TSEnumDeclaration::boxed(
            span, id, body, r#const, declare, accessor,
        ))
    }

    /// Build a [`Declaration::TSModuleDeclaration`].
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn new_ts_module_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, accessor,
        ))
    }

    /// Build a [`Declaration::TSModuleDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_module_declaration_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, accessor,
        ))
    }

    /// Build a [`Declaration::TSGlobalDeclaration`].
    ///
    /// This node contains a [`TSGlobalDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[inline]
    pub fn new_ts_global_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed(
            span,
            global_span,
            body,
            declare,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSGlobalDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSGlobalDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_global_declaration_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed_with_scope_id(
            span,
            global_span,
            body,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`Declaration::TSImportEqualsDeclaration`].
    ///
    /// This node contains a [`TSImportEqualsDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[inline]
    pub fn new_ts_import_equals_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        Self::TSImportEqualsDeclaration(TSImportEqualsDeclaration::boxed(
            span,
            id,
            module_reference,
            import_kind,
            accessor,
        ))
    }
}

impl<'a> VariableDeclaration<'a> {
    /// Build a [`VariableDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`VariableDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        VariableDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            declarations,
            declare,
        }
    }

    /// Build a [`VariableDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`VariableDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, kind, declarations, declare, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> VariableDeclarator<'a> {
    /// Build a [`VariableDeclarator`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `id`
    /// * `type_annotation`
    /// * `init`
    /// * `definite`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        type_annotation: T1,
        init: Option<Expression<'a>>,
        definite: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        VariableDeclarator {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            id,
            type_annotation: type_annotation.into_in(builder.allocator()),
            init,
            definite,
        }
    }
}

impl EmptyStatement {
    /// Build an [`EmptyStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`EmptyStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        EmptyStatement { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build an [`EmptyStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`EmptyStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> ExpressionStatement<'a> {
    /// Build an [`ExpressionStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ExpressionStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        ExpressionStatement { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build an [`ExpressionStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExpressionStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> IfStatement<'a> {
    /// Build an [`IfStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`IfStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        IfStatement { node_id: Cell::new(builder.node_id()), span, test, consequent, alternate }
    }

    /// Build an [`IfStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IfStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, test, consequent, alternate, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> DoWhileStatement<'a> {
    /// Build a [`DoWhileStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`DoWhileStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        DoWhileStatement { node_id: Cell::new(builder.node_id()), span, body, test }
    }

    /// Build a [`DoWhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`DoWhileStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, body, test, accessor), accessor.builder().allocator())
    }
}

impl<'a> WhileStatement<'a> {
    /// Build a [`WhileStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`WhileStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        WhileStatement { node_id: Cell::new(builder.node_id()), span, test, body }
    }

    /// Build a [`WhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WhileStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, test, body, accessor), accessor.builder().allocator())
    }
}

impl<'a> ForStatement<'a> {
    /// Build a [`ForStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            init,
            test,
            update,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`ForStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, init, test, update, body, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`ForStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            init,
            test,
            update,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`ForStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, init, test, update, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ForStatementInit<'a> {
    /// Build a [`ForStatementInit::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn new_variable_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build a [`ForStatementInit::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build a [`ForStatementInit::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`ForStatementInit::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`ForStatementInit::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build a [`ForStatementInit::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`ForStatementInit::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build a [`ForStatementInit::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`ForStatementInit::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build a [`ForStatementInit::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build a [`ForStatementInit::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build a [`ForStatementInit::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build a [`ForStatementInit::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build a [`ForStatementInit::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build a [`ForStatementInit::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`ForStatementInit::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build a [`ForStatementInit::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build a [`ForStatementInit::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build a [`ForStatementInit::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`ForStatementInit::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build a [`ForStatementInit::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build a [`ForStatementInit::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build a [`ForStatementInit::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build a [`ForStatementInit::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build a [`ForStatementInit::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build a [`ForStatementInit::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build a [`ForStatementInit::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build a [`ForStatementInit::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build a [`ForStatementInit::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build a [`ForStatementInit::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build a [`ForStatementInit::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`ForStatementInit::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`ForStatementInit::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build a [`ForStatementInit::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`ForStatementInit::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`ForStatementInit::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> ForInStatement<'a> {
    /// Build a [`ForInStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForInStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForInStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            left,
            right,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`ForInStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForInStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, left, right, body, accessor), accessor.builder().allocator())
    }

    /// Build a [`ForInStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForInStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForInStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            left,
            right,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`ForInStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForInStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, left, right, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ForStatementLeft<'a> {
    /// Build a [`ForStatementLeft::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn new_variable_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: Vec<'a, VariableDeclarator<'a>>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            accessor,
        ))
    }

    /// Build a [`ForStatementLeft::AssignmentTargetIdentifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_assignment_target_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`ForStatementLeft::AssignmentTargetIdentifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_assignment_target_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`ForStatementLeft::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build a [`ForStatementLeft::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`ForStatementLeft::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`ForStatementLeft::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build a [`ForStatementLeft::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`ForStatementLeft::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`ForStatementLeft::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }

    /// Build a [`ForStatementLeft::ArrayAssignmentTarget`].
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new_array_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, accessor))
    }

    /// Build a [`ForStatementLeft::ObjectAssignmentTarget`].
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new_object_assignment_target<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(
            span, properties, rest, accessor,
        ))
    }
}

impl<'a> ForOfStatement<'a> {
    /// Build a [`ForOfStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForOfStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForOfStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            r#await,
            left,
            right,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`ForOfStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForOfStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, r#await, left, right, body, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`ForOfStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ForOfStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ForOfStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            r#await,
            left,
            right,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`ForOfStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForOfStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, r#await, left, right, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ContinueStatement<'a> {
    /// Build a [`ContinueStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ContinueStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ContinueStatement { node_id: Cell::new(builder.node_id()), span, label }
    }

    /// Build a [`ContinueStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ContinueStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, label, accessor), accessor.builder().allocator())
    }
}

impl<'a> BreakStatement<'a> {
    /// Build a [`BreakStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BreakStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BreakStatement { node_id: Cell::new(builder.node_id()), span, label }
    }

    /// Build a [`BreakStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BreakStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, label, accessor), accessor.builder().allocator())
    }
}

impl<'a> ReturnStatement<'a> {
    /// Build a [`ReturnStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ReturnStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ReturnStatement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`ReturnStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ReturnStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> WithStatement<'a> {
    /// Build a [`WithStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`WithStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        WithStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            object,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`WithStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, object, body, accessor), accessor.builder().allocator())
    }

    /// Build a [`WithStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`WithStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        WithStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            object,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`WithStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, object, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> SwitchStatement<'a> {
    /// Build a [`SwitchStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`SwitchStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        SwitchStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            discriminant,
            cases,
            scope_id: Default::default(),
        }
    }

    /// Build a [`SwitchStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SwitchStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, discriminant, cases, accessor), accessor.builder().allocator())
    }

    /// Build a [`SwitchStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`SwitchStatement::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        SwitchStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            discriminant,
            cases,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`SwitchStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SwitchStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        discriminant: Expression<'a>,
        cases: Vec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, discriminant, cases, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> SwitchCase<'a> {
    /// Build a [`SwitchCase`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        test: Option<Expression<'a>>,
        consequent: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        SwitchCase { node_id: Cell::new(builder.node_id()), span, test, consequent }
    }
}

impl<'a> LabeledStatement<'a> {
    /// Build a [`LabeledStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`LabeledStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        LabeledStatement { node_id: Cell::new(builder.node_id()), span, label, body }
    }

    /// Build a [`LabeledStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`LabeledStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, label, body, accessor), accessor.builder().allocator())
    }
}

impl<'a> ThrowStatement<'a> {
    /// Build a [`ThrowStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ThrowStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        ThrowStatement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`ThrowStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ThrowStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> TryStatement<'a> {
    /// Build a [`TryStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TryStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        let builder = accessor.builder();
        TryStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            block: block.into_in(builder.allocator()),
            handler: handler.into_in(builder.allocator()),
            finalizer: finalizer.into_in(builder.allocator()),
        }
    }

    /// Build a [`TryStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TryStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<Box<'a, BlockStatement<'a>>>>,
    {
        Box::new_in(
            Self::new(span, block, handler, finalizer, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> CatchClause<'a> {
    /// Build a [`CatchClause`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`CatchClause::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        let builder = accessor.builder();
        CatchClause {
            node_id: Cell::new(builder.node_id()),
            span,
            param,
            body: body.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`CatchClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CatchClause::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Box::new_in(Self::new(span, param, body, accessor), accessor.builder().allocator())
    }

    /// Build a [`CatchClause`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`CatchClause::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        let builder = accessor.builder();
        CatchClause {
            node_id: Cell::new(builder.node_id()),
            span,
            param,
            body: body.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`CatchClause`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CatchClause::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, BlockStatement<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(span, param, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> CatchParameter<'a> {
    /// Build a [`CatchParameter`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `pattern`: The bound error
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        CatchParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            pattern,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }
}

impl DebuggerStatement {
    /// Build a [`DebuggerStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`DebuggerStatement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        DebuggerStatement { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`DebuggerStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`DebuggerStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> BindingPattern<'a> {
    /// Build a [`BindingPattern::BindingIdentifier`].
    ///
    /// This node contains a [`BindingIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[inline]
    pub fn new_binding_identifier<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::BindingIdentifier(BindingIdentifier::boxed(span, name, accessor))
    }

    /// Build a [`BindingPattern::BindingIdentifier`] with `symbol_id`.
    ///
    /// This node contains a [`BindingIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn new_binding_identifier_with_symbol_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::BindingIdentifier(BindingIdentifier::boxed_with_symbol_id(
            span, name, symbol_id, accessor,
        ))
    }

    /// Build a [`BindingPattern::ObjectPattern`].
    ///
    /// This node contains an [`ObjectPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new_object_pattern<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Self::ObjectPattern(ObjectPattern::boxed(span, properties, rest, accessor))
    }

    /// Build a [`BindingPattern::ArrayPattern`].
    ///
    /// This node contains an [`ArrayPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new_array_pattern<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Self::ArrayPattern(ArrayPattern::boxed(span, elements, rest, accessor))
    }

    /// Build a [`BindingPattern::AssignmentPattern`].
    ///
    /// This node contains an [`AssignmentPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_pattern<A: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentPattern(AssignmentPattern::boxed(span, left, right, accessor))
    }
}

impl<'a> AssignmentPattern<'a> {
    /// Build an [`AssignmentPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AssignmentPattern::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        AssignmentPattern { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build an [`AssignmentPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, left, right, accessor), accessor.builder().allocator())
    }
}

impl<'a> ObjectPattern<'a> {
    /// Build an [`ObjectPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ObjectPattern::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        let builder = accessor.builder();
        ObjectPattern {
            node_id: Cell::new(builder.node_id()),
            span,
            properties,
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ObjectPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: Vec<'a, BindingProperty<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Box::new_in(Self::new(span, properties, rest, accessor), accessor.builder().allocator())
    }
}

impl<'a> BindingProperty<'a> {
    /// Build a [`BindingProperty`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `value`
    /// * `shorthand`
    /// * `computed`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BindingProperty {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            value,
            shorthand,
            computed,
        }
    }
}

impl<'a> ArrayPattern<'a> {
    /// Build an [`ArrayPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ArrayPattern::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        let builder = accessor.builder();
        ArrayPattern {
            node_id: Cell::new(builder.node_id()),
            span,
            elements,
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ArrayPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: Vec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, BindingRestElement<'a>>>>,
    {
        Box::new_in(Self::new(span, elements, rest, accessor), accessor.builder().allocator())
    }
}

impl<'a> BindingRestElement<'a> {
    /// Build a [`BindingRestElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BindingRestElement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        argument: BindingPattern<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        BindingRestElement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`BindingRestElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingRestElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: BindingPattern<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> Function<'a> {
    /// Build a [`Function`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Function::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        let builder = accessor.builder();
        Function {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            scope_id: Default::default(),
            pure: Default::default(),
            pife: Default::default(),
        }
    }

    /// Build a [`Function`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Function::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Box::new_in(
            Self::new(
                span,
                r#type,
                id,
                generator,
                r#async,
                declare,
                type_parameters,
                this_param,
                params,
                return_type,
                body,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`Function`] with `scope_id` and `pure` and `pife`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Function::boxed_with_scope_id_and_pure_and_pife`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_with_scope_id_and_pure_and_pife<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        let builder = accessor.builder();
        Function {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
            pure,
            pife,
        }
    }

    /// Build a [`Function`] with `scope_id` and `pure` and `pife`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Function::new_with_scope_id_and_pure_and_pife`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn boxed_with_scope_id_and_pure_and_pife<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Box::new_in(
            Self::new_with_scope_id_and_pure_and_pife(
                span,
                r#type,
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
                pure,
                pife,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> FormalParameters<'a> {
    /// Build a [`FormalParameters`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`FormalParameters::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `items`
    /// * `rest`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, FormalParameterRest<'a>>>>,
    {
        let builder = accessor.builder();
        FormalParameters {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            items,
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build a [`FormalParameters`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FormalParameters::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `items`
    /// * `rest`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: FormalParameterKind,
        items: Vec<'a, FormalParameter<'a>>,
        rest: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, FormalParameterRest<'a>>>>,
    {
        Box::new_in(Self::new(span, kind, items, rest, accessor), accessor.builder().allocator())
    }
}

impl<'a> FormalParameter<'a> {
    /// Build a [`FormalParameter`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `pattern`
    /// * `type_annotation`
    /// * `initializer`
    /// * `optional`
    /// * `accessibility`
    /// * `readonly`
    /// * `override`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        initializer: T2,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, Expression<'a>>>>,
    {
        let builder = accessor.builder();
        FormalParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            decorators,
            pattern,
            type_annotation: type_annotation.into_in(builder.allocator()),
            initializer: initializer.into_in(builder.allocator()),
            optional,
            accessibility,
            readonly,
            r#override,
        }
    }
}

impl<'a> FormalParameterRest<'a> {
    /// Build a [`FormalParameterRest`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`FormalParameterRest::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `rest`
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        FormalParameterRest {
            node_id: Cell::new(builder.node_id()),
            span,
            decorators,
            rest,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`FormalParameterRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FormalParameterRest::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `rest`
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        decorators: Vec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, decorators, rest, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> FunctionBody<'a> {
    /// Build a [`FunctionBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`FunctionBody::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `statements`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        FunctionBody { node_id: Cell::new(builder.node_id()), span, directives, statements }
    }

    /// Build a [`FunctionBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FunctionBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `statements`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        statements: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, directives, statements, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ArrowFunctionExpression<'a> {
    /// Build an [`ArrowFunctionExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ArrowFunctionExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        let builder = accessor.builder();
        ArrowFunctionExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            r#async,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            scope_id: Default::default(),
            pure: Default::default(),
            pife: Default::default(),
        }
    }

    /// Build an [`ArrowFunctionExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrowFunctionExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Box::new_in(
            Self::new(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }

    /// Build an [`ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_with_scope_id_and_pure_and_pife<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        let builder = accessor.builder();
        ArrowFunctionExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            r#async,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
            pure,
            pife,
        }
    }

    /// Build an [`ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrowFunctionExpression::new_with_scope_id_and_pure_and_pife`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn boxed_with_scope_id_and_pure_and_pife<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> YieldExpression<'a> {
    /// Build a [`YieldExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`YieldExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        YieldExpression { node_id: Cell::new(builder.node_id()), span, delegate, argument }
    }

    /// Build a [`YieldExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`YieldExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, delegate, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> Class<'a> {
    /// Build a [`Class`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Class::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        let builder = accessor.builder();
        Class {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(builder.allocator()),
            implements,
            body: body.into_in(builder.allocator()),
            r#abstract,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`Class`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Class::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Box::new_in(
            Self::new(
                span,
                r#type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`Class`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`Class::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        let builder = accessor.builder();
        Class {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(builder.allocator()),
            implements,
            body: body.into_in(builder.allocator()),
            r#abstract,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Class`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Class::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                r#type,
                decorators,
                id,
                type_parameters,
                super_class,
                super_type_arguments,
                implements,
                body,
                r#abstract,
                declare,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ClassBody<'a> {
    /// Build a [`ClassBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ClassBody::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, ClassElement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ClassBody { node_id: Cell::new(builder.node_id()), span, body }
    }

    /// Build a [`ClassBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ClassBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, ClassElement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, body, accessor), accessor.builder().allocator())
    }
}

impl<'a> ClassElement<'a> {
    /// Build a [`ClassElement::StaticBlock`].
    ///
    /// This node contains a [`StaticBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new_static_block<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::StaticBlock(StaticBlock::boxed(span, body, accessor))
    }

    /// Build a [`ClassElement::StaticBlock`] with `scope_id`.
    ///
    /// This node contains a [`StaticBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_static_block_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::StaticBlock(StaticBlock::boxed_with_scope_id(span, body, scope_id, accessor))
    }

    /// Build a [`ClassElement::MethodDefinition`].
    ///
    /// This node contains a [`MethodDefinition`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`: Method definition type
    /// * `decorators`
    /// * `key`
    /// * `value`
    /// * `kind`
    /// * `computed`
    /// * `static`
    /// * `override`
    /// * `optional`
    /// * `accessibility`
    #[inline]
    pub fn new_method_definition<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Self::MethodDefinition(MethodDefinition::boxed(
            span,
            r#type,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            accessibility,
            accessor,
        ))
    }

    /// Build a [`ClassElement::PropertyDefinition`].
    ///
    /// This node contains a [`PropertyDefinition`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `declare`: Property is declared with a `declare` modifier.
    /// * `override`
    /// * `optional`: `true` when created with an optional modifier (`?`)
    /// * `definite`
    /// * `readonly`: `true` when declared with a `readonly` modifier
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn new_property_definition<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::PropertyDefinition(PropertyDefinition::boxed(
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            accessibility,
            accessor,
        ))
    }

    /// Build a [`ClassElement::AccessorProperty`].
    ///
    /// This node contains an [`AccessorProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the accessor property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration, if present.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `override`: Property was declared with a `override` modifier
    /// * `definite`: Property has a `!` after its key.
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn new_accessor_property<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::AccessorProperty(AccessorProperty::boxed(
            span,
            r#type,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            r#override,
            definite,
            accessibility,
            accessor,
        ))
    }

    /// Build a [`ClassElement::TSIndexSignature`].
    ///
    /// This node contains a [`TSIndexSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[inline]
    pub fn new_ts_index_signature<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSIndexSignature(TSIndexSignature::boxed(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
            accessor,
        ))
    }
}

impl<'a> MethodDefinition<'a> {
    /// Build a [`MethodDefinition`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`MethodDefinition::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`: Method definition type
    /// * `decorators`
    /// * `key`
    /// * `value`
    /// * `kind`
    /// * `computed`
    /// * `static`
    /// * `override`
    /// * `optional`
    /// * `accessibility`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        let builder = accessor.builder();
        MethodDefinition {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators,
            key,
            value: value.into_in(builder.allocator()),
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
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`MethodDefinition::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`: Method definition type
    /// * `decorators`
    /// * `key`
    /// * `value`
    /// * `kind`
    /// * `computed`
    /// * `static`
    /// * `override`
    /// * `optional`
    /// * `accessibility`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, Function<'a>>>,
    {
        Box::new_in(
            Self::new(
                span,
                r#type,
                decorators,
                key,
                value,
                kind,
                computed,
                r#static,
                r#override,
                optional,
                accessibility,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> PropertyDefinition<'a> {
    /// Build a [`PropertyDefinition`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`PropertyDefinition::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `declare`: Property is declared with a `declare` modifier.
    /// * `override`
    /// * `optional`: `true` when created with an optional modifier (`?`)
    /// * `definite`
    /// * `readonly`: `true` when declared with a `readonly` modifier
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        PropertyDefinition {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators,
            key,
            type_annotation: type_annotation.into_in(builder.allocator()),
            value,
            computed,
            r#static,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            accessibility,
        }
    }

    /// Build a [`PropertyDefinition`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PropertyDefinition::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `declare`: Property is declared with a `declare` modifier.
    /// * `override`
    /// * `optional`: `true` when created with an optional modifier (`?`)
    /// * `definite`
    /// * `readonly`: `true` when declared with a `readonly` modifier
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(
                span,
                r#type,
                decorators,
                key,
                type_annotation,
                value,
                computed,
                r#static,
                declare,
                r#override,
                optional,
                definite,
                readonly,
                accessibility,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> PrivateIdentifier<'a> {
    /// Build a [`PrivateIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`PrivateIdentifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = accessor.builder();
        PrivateIdentifier { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build a [`PrivateIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Box<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }
}

impl<'a> StaticBlock<'a> {
    /// Build a [`StaticBlock`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`StaticBlock::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        StaticBlock {
            node_id: Cell::new(builder.node_id()),
            span,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`StaticBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticBlock::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, body, accessor), accessor.builder().allocator())
    }

    /// Build a [`StaticBlock`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`StaticBlock::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        StaticBlock {
            node_id: Cell::new(builder.node_id()),
            span,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`StaticBlock`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticBlock::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, Statement<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, body, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ModuleDeclaration<'a> {
    /// Build a [`ModuleDeclaration::ImportDeclaration`].
    ///
    /// This node contains an [`ImportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[inline]
    pub fn new_import_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ImportDeclaration(ImportDeclaration::boxed(
            span,
            specifiers,
            source,
            phase,
            with_clause,
            import_kind,
            accessor,
        ))
    }

    /// Build a [`ModuleDeclaration::ExportAllDeclaration`].
    ///
    /// This node contains an [`ExportAllDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[inline]
    pub fn new_export_all_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ExportAllDeclaration(ExportAllDeclaration::boxed(
            span,
            exported,
            source,
            with_clause,
            export_kind,
            accessor,
        ))
    }

    /// Build a [`ModuleDeclaration::ExportDefaultDeclaration`].
    ///
    /// This node contains an [`ExportDefaultDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[inline]
    pub fn new_export_default_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(span, declaration, accessor))
    }

    /// Build a [`ModuleDeclaration::ExportNamedDeclaration`].
    ///
    /// This node contains an [`ExportNamedDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[inline]
    pub fn new_export_named_declaration<A: GetAstBuilder<'a>, T1>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Self::ExportNamedDeclaration(ExportNamedDeclaration::boxed(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            accessor,
        ))
    }

    /// Build a [`ModuleDeclaration::TSExportAssignment`].
    ///
    /// This node contains a [`TSExportAssignment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_export_assignment<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSExportAssignment(TSExportAssignment::boxed(span, expression, accessor))
    }

    /// Build a [`ModuleDeclaration::TSNamespaceExportDeclaration`].
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn new_ts_namespace_export_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNamespaceExportDeclaration(TSNamespaceExportDeclaration::boxed(span, id, accessor))
    }
}

impl<'a> AccessorProperty<'a> {
    /// Build an [`AccessorProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AccessorProperty::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the accessor property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration, if present.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `override`: Property was declared with a `override` modifier
    /// * `definite`: Property has a `!` after its key.
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        AccessorProperty {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators,
            key,
            type_annotation: type_annotation.into_in(builder.allocator()),
            value,
            computed,
            r#static,
            r#override,
            definite,
            accessibility,
        }
    }

    /// Build an [`AccessorProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AccessorProperty::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the accessor property.
    /// * `key`: The expression used to declare the property.
    /// * `type_annotation`: Type annotation on the property.
    /// * `value`: Initialized value in the declaration, if present.
    /// * `computed`: Property was declared with a computed key
    /// * `static`: Property was declared with a `static` modifier
    /// * `override`: Property was declared with a `override` modifier
    /// * `definite`: Property has a `!` after its key.
    /// * `accessibility`: Accessibility modifier.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: Vec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(
                span,
                r#type,
                decorators,
                key,
                type_annotation,
                value,
                computed,
                r#static,
                r#override,
                definite,
                accessibility,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ImportExpression<'a> {
    /// Build an [`ImportExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ImportExpression { node_id: Cell::new(builder.node_id()), span, source, options, phase }
    }

    /// Build an [`ImportExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, source, options, phase, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ImportDeclaration<'a> {
    /// Build an [`ImportDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        let builder = accessor.builder();
        ImportDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            specifiers,
            source,
            phase,
            with_clause: with_clause.into_in(builder.allocator()),
            import_kind,
        }
    }

    /// Build an [`ImportDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<Vec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Box::new_in(
            Self::new(span, specifiers, source, phase, with_clause, import_kind, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ImportDeclarationSpecifier<'a> {
    /// Build an [`ImportDeclarationSpecifier::ImportSpecifier`].
    ///
    /// This node contains an [`ImportSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[inline]
    pub fn new_import_specifier<A: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        Self::ImportSpecifier(ImportSpecifier::boxed(span, imported, local, import_kind, accessor))
    }

    /// Build an [`ImportDeclarationSpecifier::ImportDefaultSpecifier`].
    ///
    /// This node contains an [`ImportDefaultSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[inline]
    pub fn new_import_default_specifier<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::ImportDefaultSpecifier(ImportDefaultSpecifier::boxed(span, local, accessor))
    }

    /// Build an [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`].
    ///
    /// This node contains an [`ImportNamespaceSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[inline]
    pub fn new_import_namespace_specifier<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::ImportNamespaceSpecifier(ImportNamespaceSpecifier::boxed(span, local, accessor))
    }
}

impl<'a> ImportSpecifier<'a> {
    /// Build an [`ImportSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportSpecifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ImportSpecifier {
            node_id: Cell::new(builder.node_id()),
            span,
            imported,
            local,
            import_kind,
        }
    }

    /// Build an [`ImportSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, imported, local, import_kind, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ImportDefaultSpecifier<'a> {
    /// Build an [`ImportDefaultSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportDefaultSpecifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ImportDefaultSpecifier { node_id: Cell::new(builder.node_id()), span, local }
    }

    /// Build an [`ImportDefaultSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportDefaultSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, local, accessor), accessor.builder().allocator())
    }
}

impl<'a> ImportNamespaceSpecifier<'a> {
    /// Build an [`ImportNamespaceSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportNamespaceSpecifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ImportNamespaceSpecifier { node_id: Cell::new(builder.node_id()), span, local }
    }

    /// Build an [`ImportNamespaceSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportNamespaceSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, local, accessor), accessor.builder().allocator())
    }
}

impl<'a> WithClause<'a> {
    /// Build a [`WithClause`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`WithClause::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `keyword`
    /// * `with_entries`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: Vec<'a, ImportAttribute<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        WithClause { node_id: Cell::new(builder.node_id()), span, keyword, with_entries }
    }

    /// Build a [`WithClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithClause::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `keyword`
    /// * `with_entries`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: Vec<'a, ImportAttribute<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, keyword, with_entries, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ImportAttribute<'a> {
    /// Build an [`ImportAttribute`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `value`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ImportAttribute { node_id: Cell::new(builder.node_id()), span, key, value }
    }
}

impl<'a> ImportAttributeKey<'a> {
    /// Build an [`ImportAttributeKey::Identifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::new(span, name, accessor))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, accessor))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }
}

impl<'a> ExportNamedDeclaration<'a> {
    /// Build an [`ExportNamedDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ExportNamedDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        let builder = accessor.builder();
        ExportNamedDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause: with_clause.into_in(builder.allocator()),
        }
    }

    /// Build an [`ExportNamedDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExportNamedDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Box::new_in(
            Self::new(span, declaration, specifiers, source, export_kind, with_clause, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ExportDefaultDeclaration<'a> {
    /// Build an [`ExportDefaultDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ExportDefaultDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ExportDefaultDeclaration { node_id: Cell::new(builder.node_id()), span, declaration }
    }

    /// Build an [`ExportDefaultDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExportDefaultDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, declaration, accessor), accessor.builder().allocator())
    }
}

impl<'a> ExportAllDeclaration<'a> {
    /// Build an [`ExportAllDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ExportAllDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        let builder = accessor.builder();
        ExportAllDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            exported,
            source,
            with_clause: with_clause.into_in(builder.allocator()),
            export_kind,
        }
    }

    /// Build an [`ExportAllDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExportAllDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, WithClause<'a>>>>,
    {
        Box::new_in(
            Self::new(span, exported, source, with_clause, export_kind, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> ExportSpecifier<'a> {
    /// Build an [`ExportSpecifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    /// * `exported`
    /// * `export_kind`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        ExportSpecifier {
            node_id: Cell::new(builder.node_id()),
            span,
            local,
            exported,
            export_kind,
        }
    }
}

impl<'a> ExportDefaultDeclarationKind<'a> {
    /// Build an [`ExportDefaultDeclarationKind::FunctionDeclaration`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_declaration<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::FunctionDeclaration`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_declaration_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionDeclaration(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ClassDeclaration`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_declaration<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ClassDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassDeclaration(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSInterfaceDeclaration`].
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[inline]
    pub fn new_ts_interface_declaration<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSInterfaceDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSInterfaceDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_interface_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build an [`ExportDefaultDeclarationKind::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build an [`ExportDefaultDeclarationKind::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl<'a> ModuleExportName<'a> {
    /// Build a [`ModuleExportName::IdentifierName`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_identifier_name<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierName(IdentifierName::new(span, name, accessor))
    }

    /// Build a [`ModuleExportName::IdentifierReference`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::new(span, name, accessor))
    }

    /// Build a [`ModuleExportName::IdentifierReference`] with `reference_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::new_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`ModuleExportName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, accessor))
    }

    /// Build a [`ModuleExportName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }
}

impl<'a> V8IntrinsicExpression<'a> {
    /// Build a [`V8IntrinsicExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`V8IntrinsicExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        V8IntrinsicExpression { node_id: Cell::new(builder.node_id()), span, name, arguments }
    }

    /// Build a [`V8IntrinsicExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`V8IntrinsicExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, name, arguments, accessor), accessor.builder().allocator())
    }
}

impl BooleanLiteral {
    /// Build a [`BooleanLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BooleanLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, value: bool, accessor: &A) -> Self {
        let builder = accessor.builder();
        BooleanLiteral { node_id: Cell::new(builder.node_id()), span, value }
    }

    /// Build a [`BooleanLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BooleanLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, value: bool, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, value, accessor), accessor.builder().allocator())
    }
}

impl NullLiteral {
    /// Build a [`NullLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`NullLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        NullLiteral { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`NullLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NullLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> NumericLiteral<'a> {
    /// Build a [`NumericLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`NumericLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        NumericLiteral { node_id: Cell::new(builder.node_id()), span, value, raw, base }
    }

    /// Build a [`NumericLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NumericLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, value, raw, base, accessor), accessor.builder().allocator())
    }
}

impl<'a> StringLiteral<'a> {
    /// Build a [`StringLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`StringLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        StringLiteral {
            node_id: Cell::new(builder.node_id()),
            span,
            value: value.into(),
            raw,
            lone_surrogates: Default::default(),
        }
    }

    /// Build a [`StringLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StringLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        Box::new_in(Self::new(span, value, raw, accessor), accessor.builder().allocator())
    }

    /// Build a [`StringLiteral`] with `lone_surrogates`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`StringLiteral::boxed_with_lone_surrogates`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        StringLiteral {
            node_id: Cell::new(builder.node_id()),
            span,
            value: value.into(),
            raw,
            lone_surrogates,
        }
    }

    /// Build a [`StringLiteral`] with `lone_surrogates`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StringLiteral::new_with_lone_surrogates`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn boxed_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        Box::new_in(
            Self::new_with_lone_surrogates(span, value, raw, lone_surrogates, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> BigIntLiteral<'a> {
    /// Build a [`BigIntLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`BigIntLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        BigIntLiteral {
            node_id: Cell::new(builder.node_id()),
            span,
            value: value.into(),
            raw,
            base,
        }
    }

    /// Build a [`BigIntLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BigIntLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        Box::new_in(Self::new(span, value, raw, base, accessor), accessor.builder().allocator())
    }
}

impl<'a> RegExpLiteral<'a> {
    /// Build a [`RegExpLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`RegExpLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        RegExpLiteral { node_id: Cell::new(builder.node_id()), span, regex, raw }
    }

    /// Build a [`RegExpLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`RegExpLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, regex, raw, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXElement<'a> {
    /// Build a [`JSXElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXElement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        let builder = accessor.builder();
        JSXElement {
            node_id: Cell::new(builder.node_id()),
            span,
            opening_element: opening_element.into_in(builder.allocator()),
            children,
            closing_element: closing_element.into_in(builder.allocator()),
        }
    }

    /// Build a [`JSXElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Box::new_in(
            Self::new(span, opening_element, children, closing_element, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> JSXOpeningElement<'a> {
    /// Build a [`JSXOpeningElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXOpeningElement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    /// * `type_arguments`: Type parameters for generic JSX elements.
    /// * `attributes`: List of JSX attributes. In React-like applications, these become props.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        JSXOpeningElement {
            node_id: Cell::new(builder.node_id()),
            span,
            name,
            type_arguments: type_arguments.into_in(builder.allocator()),
            attributes,
        }
    }

    /// Build a [`JSXOpeningElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXOpeningElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    /// * `type_arguments`: Type parameters for generic JSX elements.
    /// * `attributes`: List of JSX attributes. In React-like applications, these become props.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: Vec<'a, JSXAttributeItem<'a>>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, name, type_arguments, attributes, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> JSXClosingElement<'a> {
    /// Build a [`JSXClosingElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXClosingElement::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The tag name, e.g. `Foo` in `</Foo>`.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, name: JSXElementName<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXClosingElement { node_id: Cell::new(builder.node_id()), span, name }
    }

    /// Build a [`JSXClosingElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXClosingElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The tag name, e.g. `Foo` in `</Foo>`.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        name: JSXElementName<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXFragment<'a> {
    /// Build a [`JSXFragment`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXFragment::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSXFragment {
            node_id: Cell::new(builder.node_id()),
            span,
            opening_fragment,
            children,
            closing_fragment,
        }
    }

    /// Build a [`JSXFragment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXFragment::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, opening_fragment, children, closing_fragment, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl JSXOpeningFragment {
    /// Build a [`JSXOpeningFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXOpeningFragment { node_id: Cell::new(builder.node_id()), span }
    }
}

impl JSXClosingFragment {
    /// Build a [`JSXClosingFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXClosingFragment { node_id: Cell::new(builder.node_id()), span }
    }
}

impl<'a> JSXElementName<'a> {
    /// Build a [`JSXElementName::Identifier`].
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Identifier(JSXIdentifier::boxed(span, name, accessor))
    }

    /// Build a [`JSXElementName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`JSXElementName::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`JSXElementName::NamespacedName`].
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn new_namespaced_name<A: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::NamespacedName(JSXNamespacedName::boxed(span, namespace, name, accessor))
    }

    /// Build a [`JSXElementName::MemberExpression`].
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn new_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::MemberExpression(JSXMemberExpression::boxed(span, object, property, accessor))
    }

    /// Build a [`JSXElementName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }
}

impl<'a> JSXNamespacedName<'a> {
    /// Build a [`JSXNamespacedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXNamespacedName::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSXNamespacedName { node_id: Cell::new(builder.node_id()), span, namespace, name }
    }

    /// Build a [`JSXNamespacedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXNamespacedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, namespace, name, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXMemberExpression<'a> {
    /// Build a [`JSXMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXMemberExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSXMemberExpression { node_id: Cell::new(builder.node_id()), span, object, property }
    }

    /// Build a [`JSXMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, object, property, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXMemberExpressionObject<'a> {
    /// Build a [`JSXMemberExpressionObject::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`JSXMemberExpressionObject::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`JSXMemberExpressionObject::MemberExpression`].
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn new_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::MemberExpression(JSXMemberExpression::boxed(span, object, property, accessor))
    }

    /// Build a [`JSXMemberExpressionObject::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }
}

impl<'a> JSXExpressionContainer<'a> {
    /// Build a [`JSXExpressionContainer`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXExpressionContainer::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSXExpressionContainer { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`JSXExpressionContainer`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXExpressionContainer::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXExpression<'a> {
    /// Build a [`JSXExpression::EmptyExpression`].
    ///
    /// This node contains a [`JSXEmptyExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_empty_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::EmptyExpression(JSXEmptyExpression::boxed(span, accessor))
    }

    /// Build a [`JSXExpression::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build a [`JSXExpression::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, accessor))
    }

    /// Build a [`JSXExpression::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`JSXExpression::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`JSXExpression::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn new_reg_exp_literal<A: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, accessor))
    }

    /// Build a [`JSXExpression::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`JSXExpression::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build a [`JSXExpression::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`JSXExpression::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::MetaProperty`].
    ///
    /// This node contains a [`MetaProperty`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `meta`
    /// * `property`
    #[inline]
    pub fn new_meta_property<A: GetAstBuilder<'a>>(
        span: Span,
        meta: IdentifierName<'a>,
        property: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::MetaProperty(MetaProperty::boxed(span, meta, property, accessor))
    }

    /// Build a [`JSXExpression::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::Super(Super::boxed(span, accessor))
    }

    /// Build a [`JSXExpression::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<A: GetAstBuilder<'a>>(
        span: Span,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, accessor))
    }

    /// Build a [`JSXExpression::ArrowFunctionExpression`].
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[inline]
    pub fn new_arrow_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains an [`ArrowFunctionExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_arrow_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
    >(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(
            ArrowFunctionExpression::boxed_with_scope_id_and_pure_and_pife(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                scope_id,
                pure,
                pife,
                accessor,
            ),
        )
    }

    /// Build a [`JSXExpression::AssignmentExpression`].
    ///
    /// This node contains an [`AssignmentExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_assignment_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, accessor,
        ))
    }

    /// Build a [`JSXExpression::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_await_expression<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, accessor))
    }

    /// Build a [`JSXExpression::BinaryExpression`].
    ///
    /// This node contains a [`BinaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_binary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`JSXExpression::CallExpression`].
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn new_call_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::CallExpression`] with `pure`.
    ///
    /// This node contains a [`CallExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_call_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_chain_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        accessor: &A,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, accessor))
    }

    /// Build a [`JSXExpression::ClassExpression`].
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    #[inline]
    pub fn new_class_expression<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ClassExpression`] with `scope_id`.
    ///
    /// This node contains a [`Class`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `decorators`: Decorators applied to the class.
    /// * `id`: Class identifier, AKA the name
    /// * `type_parameters`
    /// * `super_class`: Super class. When present, this will usually be an [`IdentifierReference`].
    /// * `super_type_arguments`: Type parameters passed to super class.
    /// * `implements`: Interface implementation clause for TypeScript classes.
    /// * `body`
    /// * `abstract`: Whether the class is abstract
    /// * `declare`: Whether the class was `declare`ed
    /// * `scope_id`: Id of the scope created by the [`Class`], including type parameters and
    #[inline]
    pub fn new_class_expression_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#type: ClassType,
        decorators: Vec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: Vec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, Box<'a, ClassBody<'a>>>,
    {
        Self::ClassExpression(Class::boxed_with_scope_id(
            span,
            r#type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ConditionalExpression`].
    ///
    /// This node contains a [`ConditionalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn new_conditional_expression<A: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, accessor,
        ))
    }

    /// Build a [`JSXExpression::FunctionExpression`].
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    #[inline]
    pub fn new_function_expression<A: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        body: T5,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed(
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::FunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// This node contains a [`Function`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type`
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `generator`: Is this a generator function?
    /// * `async`
    /// * `declare`
    /// * `type_parameters`
    /// * `this_param`: Declaring `this` in a Function <https://www.typescriptlang.org/docs/handbook/2/functions.html#declaring-this-in-a-function>
    /// * `params`: Function parameters.
    /// * `return_type`: The TypeScript return type annotation.
    /// * `body`: The function body.
    /// * `scope_id`
    /// * `pure`: `true` if the function is marked with a `/*#__NO_SIDE_EFFECTS__*/` comment
    /// * `pife`: `true` if the function should be marked as "Possibly-Invoked Function Expression" (PIFE).
    #[inline]
    pub fn new_function_expression_with_scope_id_and_pure_and_pife<
        A: GetAstBuilder<'a>,
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        span: Span,
        r#type: FunctionType,
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
        pure: bool,
        pife: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        Self::FunctionExpression(Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
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
            pure,
            pife,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ImportExpression`].
    ///
    /// This node contains an [`ImportExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn new_import_expression<A: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        accessor: &A,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, accessor))
    }

    /// Build a [`JSXExpression::LogicalExpression`].
    ///
    /// This node contains a [`LogicalExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn new_logical_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, accessor))
    }

    /// Build a [`JSXExpression::NewExpression`].
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn new_new_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, accessor))
    }

    /// Build a [`JSXExpression::NewExpression`] with `pure`.
    ///
    /// This node contains a [`NewExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn new_new_expression_with_pure<A: GetAstBuilder<'a>, T1>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: Vec<'a, Argument<'a>>,
        pure: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn new_object_expression<A: GetAstBuilder<'a>>(
        span: Span,
        properties: Vec<'a, ObjectPropertyKind<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, accessor))
    }

    /// Build a [`JSXExpression::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, accessor))
    }

    /// Build a [`JSXExpression::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, accessor))
    }

    /// Build a [`JSXExpression::TaggedTemplateExpression`].
    ///
    /// This node contains a [`TaggedTemplateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn new_tagged_template_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }

    /// Build a [`JSXExpression::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }

    /// Build a [`JSXExpression::UpdateExpression`].
    ///
    /// This node contains an [`UpdateExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn new_update_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        accessor: &A,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, accessor))
    }

    /// Build a [`JSXExpression::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn new_yield_expression<A: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, accessor))
    }

    /// Build a [`JSXExpression::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_private_in_expression<A: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, accessor))
    }

    /// Build a [`JSXExpression::JSXElement`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_jsx_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::JSXFragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_jsx_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_as_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, accessor))
    }

    /// Build a [`JSXExpression::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new_ts_satisfies_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new_ts_type_assertion<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, accessor))
    }

    /// Build a [`JSXExpression::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, accessor))
    }

    /// Build a [`JSXExpression::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_instantiation_expression<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`JSXExpression::V8IntrinsicExpression`].
    ///
    /// This node contains a [`V8IntrinsicExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn new_v8_intrinsic_expression<A: GetAstBuilder<'a>>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: Vec<'a, Argument<'a>>,
        accessor: &A,
    ) -> Self {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, accessor))
    }

    /// Build a [`JSXExpression::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn new_computed_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, accessor,
        ))
    }

    /// Build a [`JSXExpression::StaticMemberExpression`].
    ///
    /// This node contains a [`StaticMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn new_static_member_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, accessor,
        ))
    }

    /// Build a [`JSXExpression::PrivateFieldExpression`].
    ///
    /// This node contains a [`PrivateFieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn new_private_field_expression<A: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, accessor,
        ))
    }
}

impl JSXEmptyExpression {
    /// Build a [`JSXEmptyExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXEmptyExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXEmptyExpression { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`JSXEmptyExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXEmptyExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXAttributeItem<'a> {
    /// Build a [`JSXAttributeItem::Attribute`].
    ///
    /// This node contains a [`JSXAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn new_attribute<A: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        accessor: &A,
    ) -> Self {
        Self::Attribute(JSXAttribute::boxed(span, name, value, accessor))
    }

    /// Build a [`JSXAttributeItem::SpreadAttribute`].
    ///
    /// This node contains a [`JSXSpreadAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new_spread_attribute<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::SpreadAttribute(JSXSpreadAttribute::boxed(span, argument, accessor))
    }
}

impl<'a> JSXAttribute<'a> {
    /// Build a [`JSXAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXAttribute::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSXAttribute { node_id: Cell::new(builder.node_id()), span, name, value }
    }

    /// Build a [`JSXAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXAttribute::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, name, value, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXSpreadAttribute<'a> {
    /// Build a [`JSXSpreadAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXSpreadAttribute::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXSpreadAttribute { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`JSXSpreadAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXSpreadAttribute::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, argument, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXAttributeName<'a> {
    /// Build a [`JSXAttributeName::Identifier`].
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Identifier(JSXIdentifier::boxed(span, name, accessor))
    }

    /// Build a [`JSXAttributeName::NamespacedName`].
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn new_namespaced_name<A: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        accessor: &A,
    ) -> Self {
        Self::NamespacedName(JSXNamespacedName::boxed(span, namespace, name, accessor))
    }
}

impl<'a> JSXAttributeValue<'a> {
    /// Build a [`JSXAttributeValue::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`JSXAttributeValue::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`JSXAttributeValue::ExpressionContainer`].
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[inline]
    pub fn new_expression_container<A: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExpressionContainer(JSXExpressionContainer::boxed(span, expression, accessor))
    }

    /// Build a [`JSXAttributeValue::Element`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::Element(JSXElement::boxed(span, opening_element, children, closing_element, accessor))
    }

    /// Build a [`JSXAttributeValue::Fragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::Fragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }
}

impl<'a> JSXIdentifier<'a> {
    /// Build a [`JSXIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXIdentifier::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        JSXIdentifier { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build a [`JSXIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Box<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        Box::new_in(Self::new(span, name, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXChild<'a> {
    /// Build a [`JSXChild::Text`].
    ///
    /// This node contains a [`JSXText`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_text<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Text(JSXText::boxed(span, value, raw, accessor))
    }

    /// Build a [`JSXChild::Element`].
    ///
    /// This node contains a [`JSXElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn new_element<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        opening_element: T1,
        children: Vec<'a, JSXChild<'a>>,
        closing_element: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<Box<'a, JSXClosingElement<'a>>>>,
    {
        Self::Element(JSXElement::boxed(span, opening_element, children, closing_element, accessor))
    }

    /// Build a [`JSXChild::Fragment`].
    ///
    /// This node contains a [`JSXFragment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn new_fragment<A: GetAstBuilder<'a>>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: Vec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
        accessor: &A,
    ) -> Self {
        Self::Fragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            accessor,
        ))
    }

    /// Build a [`JSXChild::ExpressionContainer`].
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[inline]
    pub fn new_expression_container<A: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExpressionContainer(JSXExpressionContainer::boxed(span, expression, accessor))
    }

    /// Build a [`JSXChild::Spread`].
    ///
    /// This node contains a [`JSXSpreadChild`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[inline]
    pub fn new_spread<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::Spread(JSXSpreadChild::boxed(span, expression, accessor))
    }
}

impl<'a> JSXSpreadChild<'a> {
    /// Build a [`JSXSpreadChild`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXSpreadChild::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSXSpreadChild { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`JSXSpreadChild`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXSpreadChild::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> JSXText<'a> {
    /// Build a [`JSXText`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSXText::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = accessor.builder();
        JSXText { node_id: Cell::new(builder.node_id()), span, value: value.into(), raw }
    }

    /// Build a [`JSXText`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXText::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        Box::new_in(Self::new(span, value, raw, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSThisParameter<'a> {
    /// Build a [`TSThisParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSThisParameter::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `this_span`
    /// * `type_annotation`: Type type the `this` keyword will have in the function
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        this_span: Span,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSThisParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            this_span,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSThisParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSThisParameter::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `this_span`
    /// * `type_annotation`: Type type the `this` keyword will have in the function
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        this_span: Span,
        type_annotation: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, this_span, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSEnumDeclaration<'a> {
    /// Build a [`TSEnumDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSEnumDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSEnumDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            body,
            r#const,
            declare,
        }
    }

    /// Build a [`TSEnumDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSEnumDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, id, body, r#const, declare, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSEnumBody<'a> {
    /// Build a [`TSEnumBody`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSEnumMember<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSEnumBody {
            node_id: Cell::new(builder.node_id()),
            span,
            members,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSEnumBody`] with `scope_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSEnumMember<'a>>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSEnumBody {
            node_id: Cell::new(builder.node_id()),
            span,
            members,
            scope_id: Cell::new(Some(scope_id)),
        }
    }
}

impl<'a> TSEnumMember<'a> {
    /// Build a [`TSEnumMember`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `initializer`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSEnumMember { node_id: Cell::new(builder.node_id()), span, id, initializer }
    }
}

impl<'a> TSEnumMemberName<'a> {
    /// Build a [`TSEnumMemberName::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, accessor))
    }

    /// Build a [`TSEnumMemberName::String`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::String(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`TSEnumMemberName::String`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::String(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`TSEnumMemberName::ComputedString`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_computed_string<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::ComputedString(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`TSEnumMemberName::ComputedString`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_computed_string_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::ComputedString(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`TSEnumMemberName::ComputedTemplateString`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_computed_template_string<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::ComputedTemplateString(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }
}

impl<'a> TSTypeAnnotation<'a> {
    /// Build a [`TSTypeAnnotation`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeAnnotation::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`: The actual type in the annotation
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeAnnotation { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSTypeAnnotation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAnnotation::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`: The actual type in the annotation
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, type_annotation, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSLiteralType<'a> {
    /// Build a [`TSLiteralType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSLiteralType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, literal: TSLiteral<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSLiteralType { node_id: Cell::new(builder.node_id()), span, literal }
    }

    /// Build a [`TSLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSLiteralType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, literal, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSLiteral<'a> {
    /// Build a [`TSLiteral::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        accessor: &A,
    ) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, accessor))
    }

    /// Build a [`TSLiteral::NumericLiteral`].
    ///
    /// This node contains a [`NumericLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_numeric_literal<A: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        accessor: &A,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`TSLiteral::BigIntLiteral`].
    ///
    /// This node contains a [`BigIntLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn new_big_int_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, accessor))
    }

    /// Build a [`TSLiteral::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, accessor))
    }

    /// Build a [`TSLiteral::StringLiteral`] with `lone_surrogates`.
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }

    /// Build a [`TSLiteral::TemplateLiteral`].
    ///
    /// This node contains a [`TemplateLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn new_template_literal<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        expressions: Vec<'a, Expression<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, accessor))
    }

    /// Build a [`TSLiteral::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn new_unary_expression<A: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        accessor: &A,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, accessor))
    }
}

impl<'a> TSType<'a> {
    /// Build a [`TSType::TSAnyKeyword`].
    ///
    /// This node contains a [`TSAnyKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_any_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSAnyKeyword(TSAnyKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSBigIntKeyword`].
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_big_int_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSBigIntKeyword(TSBigIntKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSBooleanKeyword`].
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_boolean_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSBooleanKeyword(TSBooleanKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSIntrinsicKeyword`].
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_intrinsic_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSIntrinsicKeyword(TSIntrinsicKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSNeverKeyword`].
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_never_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNeverKeyword(TSNeverKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSNullKeyword`].
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_null_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNullKeyword(TSNullKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSNumberKeyword`].
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_number_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNumberKeyword(TSNumberKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSObjectKeyword`].
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_object_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSObjectKeyword(TSObjectKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSStringKeyword`].
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_string_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSStringKeyword(TSStringKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSSymbolKeyword`].
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_symbol_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSSymbolKeyword(TSSymbolKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSUndefinedKeyword`].
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_undefined_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSUndefinedKeyword(TSUndefinedKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSUnknownKeyword`].
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_unknown_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSUnknownKeyword(TSUnknownKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSVoidKeyword`].
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_void_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSVoidKeyword(TSVoidKeyword::boxed(span, accessor))
    }

    /// Build a [`TSType::TSArrayType`].
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn new_ts_array_type<A: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSArrayType(TSArrayType::boxed(span, element_type, accessor))
    }

    /// Build a [`TSType::TSConditionalType`].
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[inline]
    pub fn new_ts_conditional_type<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            accessor,
        ))
    }

    /// Build a [`TSType::TSConditionalType`] with `scope_id`.
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[inline]
    pub fn new_ts_conditional_type_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed_with_scope_id(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSType::TSConstructorType`].
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new_ts_constructor_type<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSType::TSConstructorType`] with `scope_id`.
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_constructor_type_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed_with_scope_id(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSType::TSFunctionType`].
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[inline]
    pub fn new_ts_function_type<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSType::TSFunctionType`] with `scope_id`.
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[inline]
    pub fn new_ts_function_type_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSType::TSImportType`].
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_import_type<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`TSType::TSIndexedAccessType`].
    ///
    /// This node contains a [`TSIndexedAccessType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[inline]
    pub fn new_ts_indexed_access_type<A: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSIndexedAccessType(TSIndexedAccessType::boxed(
            span,
            object_type,
            index_type,
            accessor,
        ))
    }

    /// Build a [`TSType::TSInferType`].
    ///
    /// This node contains a [`TSInferType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[inline]
    pub fn new_ts_infer_type<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Self::TSInferType(TSInferType::boxed(span, type_parameter, accessor))
    }

    /// Build a [`TSType::TSIntersectionType`].
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn new_ts_intersection_type<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSIntersectionType(TSIntersectionType::boxed(span, types, accessor))
    }

    /// Build a [`TSType::TSLiteralType`].
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn new_ts_literal_type<A: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSLiteralType(TSLiteralType::boxed(span, literal, accessor))
    }

    /// Build a [`TSType::TSMappedType`].
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[inline]
    pub fn new_ts_mapped_type<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        accessor: &A,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            accessor,
        ))
    }

    /// Build a [`TSType::TSMappedType`] with `scope_id`.
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    /// * `scope_id`
    #[inline]
    pub fn new_ts_mapped_type_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed_with_scope_id(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSType::TSNamedTupleMember`].
    ///
    /// This node contains a [`TSNamedTupleMember`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[inline]
    pub fn new_ts_named_tuple_member<A: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::TSNamedTupleMember(TSNamedTupleMember::boxed(
            span,
            label,
            element_type,
            optional,
            accessor,
        ))
    }

    /// Build a [`TSType::TSTemplateLiteralType`].
    ///
    /// This node contains a [`TSTemplateLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[inline]
    pub fn new_ts_template_literal_type<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTemplateLiteralType(TSTemplateLiteralType::boxed(span, quasis, types, accessor))
    }

    /// Build a [`TSType::TSThisType`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_this_type<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSThisType(TSThisType::boxed(span, accessor))
    }

    /// Build a [`TSType::TSTupleType`].
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn new_ts_tuple_type<A: GetAstBuilder<'a>>(
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTupleType(TSTupleType::boxed(span, element_types, accessor))
    }

    /// Build a [`TSType::TSTypeLiteral`].
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new_ts_type_literal<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeLiteral(TSTypeLiteral::boxed(span, members, accessor))
    }

    /// Build a [`TSType::TSTypeOperatorType`].
    ///
    /// This node contains a [`TSTypeOperator`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[inline]
    pub fn new_ts_type_operator_type<A: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeOperatorType(TSTypeOperator::boxed(span, operator, type_annotation, accessor))
    }

    /// Build a [`TSType::TSTypePredicate`].
    ///
    /// This node contains a [`TSTypePredicate`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_type_predicate<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSTypePredicate(TSTypePredicate::boxed(
            span,
            parameter_name,
            asserts,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`TSType::TSTypeQuery`].
    ///
    /// This node contains a [`TSTypeQuery`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_type_query<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeQuery(TSTypeQuery::boxed(span, expr_name, type_arguments, accessor))
    }

    /// Build a [`TSType::TSTypeReference`].
    ///
    /// This node contains a [`TSTypeReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_type_reference<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeReference(TSTypeReference::boxed(span, type_name, type_arguments, accessor))
    }

    /// Build a [`TSType::TSUnionType`].
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn new_ts_union_type<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSUnionType(TSUnionType::boxed(span, types, accessor))
    }

    /// Build a [`TSType::TSParenthesizedType`].
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_parenthesized_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSParenthesizedType(TSParenthesizedType::boxed(span, type_annotation, accessor))
    }

    /// Build a [`TSType::JSDocNullableType`].
    ///
    /// This node contains a [`JSDocNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[inline]
    pub fn new_js_doc_nullable_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        Self::JSDocNullableType(JSDocNullableType::boxed(span, type_annotation, postfix, accessor))
    }

    /// Build a [`TSType::JSDocNonNullableType`].
    ///
    /// This node contains a [`JSDocNonNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[inline]
    pub fn new_js_doc_non_nullable_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        Self::JSDocNonNullableType(JSDocNonNullableType::boxed(
            span,
            type_annotation,
            postfix,
            accessor,
        ))
    }

    /// Build a [`TSType::JSDocUnknownType`].
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_js_doc_unknown_type<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::JSDocUnknownType(JSDocUnknownType::boxed(span, accessor))
    }
}

impl<'a> TSConditionalType<'a> {
    /// Build a [`TSConditionalType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConditionalType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSConditionalType {
            node_id: Cell::new(builder.node_id()),
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
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConditionalType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, check_type, extends_type, true_type, false_type, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSConditionalType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConditionalType::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSConditionalType {
            node_id: Cell::new(builder.node_id()),
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConditionalType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConditionalType::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                check_type,
                extends_type,
                true_type,
                false_type,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSUnionType<'a> {
    /// Build a [`TSUnionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSUnionType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, types: Vec<'a, TSType<'a>>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSUnionType { node_id: Cell::new(builder.node_id()), span, types }
    }

    /// Build a [`TSUnionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUnionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, types, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSIntersectionType<'a> {
    /// Build a [`TSIntersectionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSIntersectionType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, types: Vec<'a, TSType<'a>>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSIntersectionType { node_id: Cell::new(builder.node_id()), span, types }
    }

    /// Build a [`TSIntersectionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIntersectionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, types, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSParenthesizedType<'a> {
    /// Build a [`TSParenthesizedType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSParenthesizedType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSParenthesizedType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSParenthesizedType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSParenthesizedType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, type_annotation, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeOperator<'a> {
    /// Build a [`TSTypeOperator`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeOperator::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeOperator { node_id: Cell::new(builder.node_id()), span, operator, type_annotation }
    }

    /// Build a [`TSTypeOperator`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeOperator::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, operator, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSArrayType<'a> {
    /// Build a [`TSArrayType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSArrayType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, element_type: TSType<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSArrayType { node_id: Cell::new(builder.node_id()), span, element_type }
    }

    /// Build a [`TSArrayType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSArrayType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, element_type, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSIndexedAccessType<'a> {
    /// Build a [`TSIndexedAccessType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSIndexedAccessType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSIndexedAccessType { node_id: Cell::new(builder.node_id()), span, object_type, index_type }
    }

    /// Build a [`TSIndexedAccessType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIndexedAccessType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, object_type, index_type, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTupleType<'a> {
    /// Build a [`TSTupleType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTupleType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTupleType { node_id: Cell::new(builder.node_id()), span, element_types }
    }

    /// Build a [`TSTupleType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTupleType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, element_types, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSNamedTupleMember<'a> {
    /// Build a [`TSNamedTupleMember`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNamedTupleMember::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSNamedTupleMember {
            node_id: Cell::new(builder.node_id()),
            span,
            label,
            element_type,
            optional,
        }
    }

    /// Build a [`TSNamedTupleMember`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNamedTupleMember::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, label, element_type, optional, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSOptionalType<'a> {
    /// Build a [`TSOptionalType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSOptionalType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSOptionalType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSOptionalType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSOptionalType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, type_annotation, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSRestType<'a> {
    /// Build a [`TSRestType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSRestType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSRestType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSRestType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSRestType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, type_annotation, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTupleElement<'a> {
    /// Build a [`TSTupleElement::TSOptionalType`].
    ///
    /// This node contains a [`TSOptionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_optional_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSOptionalType(TSOptionalType::boxed(span, type_annotation, accessor))
    }

    /// Build a [`TSTupleElement::TSRestType`].
    ///
    /// This node contains a [`TSRestType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_rest_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSRestType(TSRestType::boxed(span, type_annotation, accessor))
    }

    /// Build a [`TSTupleElement::TSAnyKeyword`].
    ///
    /// This node contains a [`TSAnyKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_any_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSAnyKeyword(TSAnyKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSBigIntKeyword`].
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_big_int_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSBigIntKeyword(TSBigIntKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSBooleanKeyword`].
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_boolean_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSBooleanKeyword(TSBooleanKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSIntrinsicKeyword`].
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_intrinsic_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSIntrinsicKeyword(TSIntrinsicKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSNeverKeyword`].
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_never_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNeverKeyword(TSNeverKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSNullKeyword`].
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_null_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNullKeyword(TSNullKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSNumberKeyword`].
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_number_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSNumberKeyword(TSNumberKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSObjectKeyword`].
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_object_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSObjectKeyword(TSObjectKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSStringKeyword`].
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_string_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSStringKeyword(TSStringKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSSymbolKeyword`].
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_symbol_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSSymbolKeyword(TSSymbolKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSUndefinedKeyword`].
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_undefined_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSUndefinedKeyword(TSUndefinedKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSUnknownKeyword`].
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_unknown_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSUnknownKeyword(TSUnknownKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSVoidKeyword`].
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_void_keyword<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSVoidKeyword(TSVoidKeyword::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSArrayType`].
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn new_ts_array_type<A: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSArrayType(TSArrayType::boxed(span, element_type, accessor))
    }

    /// Build a [`TSTupleElement::TSConditionalType`].
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[inline]
    pub fn new_ts_conditional_type<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSConditionalType`] with `scope_id`.
    ///
    /// This node contains a [`TSConditionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[inline]
    pub fn new_ts_conditional_type_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed_with_scope_id(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSConstructorType`].
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new_ts_constructor_type<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSConstructorType`] with `scope_id`.
    ///
    /// This node contains a [`TSConstructorType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_constructor_type_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed_with_scope_id(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSFunctionType`].
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[inline]
    pub fn new_ts_function_type<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSFunctionType`] with `scope_id`.
    ///
    /// This node contains a [`TSFunctionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[inline]
    pub fn new_ts_function_type_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSImportType`].
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_import_type<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSIndexedAccessType`].
    ///
    /// This node contains a [`TSIndexedAccessType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[inline]
    pub fn new_ts_indexed_access_type<A: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSIndexedAccessType(TSIndexedAccessType::boxed(
            span,
            object_type,
            index_type,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSInferType`].
    ///
    /// This node contains a [`TSInferType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[inline]
    pub fn new_ts_infer_type<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Self::TSInferType(TSInferType::boxed(span, type_parameter, accessor))
    }

    /// Build a [`TSTupleElement::TSIntersectionType`].
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn new_ts_intersection_type<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSIntersectionType(TSIntersectionType::boxed(span, types, accessor))
    }

    /// Build a [`TSTupleElement::TSLiteralType`].
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn new_ts_literal_type<A: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSLiteralType(TSLiteralType::boxed(span, literal, accessor))
    }

    /// Build a [`TSTupleElement::TSMappedType`].
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[inline]
    pub fn new_ts_mapped_type<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        accessor: &A,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSMappedType`] with `scope_id`.
    ///
    /// This node contains a [`TSMappedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    /// * `scope_id`
    #[inline]
    pub fn new_ts_mapped_type_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed_with_scope_id(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSNamedTupleMember`].
    ///
    /// This node contains a [`TSNamedTupleMember`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[inline]
    pub fn new_ts_named_tuple_member<A: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        accessor: &A,
    ) -> Self {
        Self::TSNamedTupleMember(TSNamedTupleMember::boxed(
            span,
            label,
            element_type,
            optional,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSTemplateLiteralType`].
    ///
    /// This node contains a [`TSTemplateLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[inline]
    pub fn new_ts_template_literal_type<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTemplateLiteralType(TSTemplateLiteralType::boxed(span, quasis, types, accessor))
    }

    /// Build a [`TSTupleElement::TSThisType`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_this_type<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::TSThisType(TSThisType::boxed(span, accessor))
    }

    /// Build a [`TSTupleElement::TSTupleType`].
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn new_ts_tuple_type<A: GetAstBuilder<'a>>(
        span: Span,
        element_types: Vec<'a, TSTupleElement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTupleType(TSTupleType::boxed(span, element_types, accessor))
    }

    /// Build a [`TSTupleElement::TSTypeLiteral`].
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new_ts_type_literal<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeLiteral(TSTypeLiteral::boxed(span, members, accessor))
    }

    /// Build a [`TSTupleElement::TSTypeOperatorType`].
    ///
    /// This node contains a [`TSTypeOperator`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[inline]
    pub fn new_ts_type_operator_type<A: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSTypeOperatorType(TSTypeOperator::boxed(span, operator, type_annotation, accessor))
    }

    /// Build a [`TSTupleElement::TSTypePredicate`].
    ///
    /// This node contains a [`TSTypePredicate`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_type_predicate<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSTypePredicate(TSTypePredicate::boxed(
            span,
            parameter_name,
            asserts,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::TSTypeQuery`].
    ///
    /// This node contains a [`TSTypeQuery`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_type_query<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeQuery(TSTypeQuery::boxed(span, expr_name, type_arguments, accessor))
    }

    /// Build a [`TSTupleElement::TSTypeReference`].
    ///
    /// This node contains a [`TSTypeReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_type_reference<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeReference(TSTypeReference::boxed(span, type_name, type_arguments, accessor))
    }

    /// Build a [`TSTupleElement::TSUnionType`].
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn new_ts_union_type<A: GetAstBuilder<'a>>(
        span: Span,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSUnionType(TSUnionType::boxed(span, types, accessor))
    }

    /// Build a [`TSTupleElement::TSParenthesizedType`].
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_parenthesized_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        Self::TSParenthesizedType(TSParenthesizedType::boxed(span, type_annotation, accessor))
    }

    /// Build a [`TSTupleElement::JSDocNullableType`].
    ///
    /// This node contains a [`JSDocNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[inline]
    pub fn new_js_doc_nullable_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        Self::JSDocNullableType(JSDocNullableType::boxed(span, type_annotation, postfix, accessor))
    }

    /// Build a [`TSTupleElement::JSDocNonNullableType`].
    ///
    /// This node contains a [`JSDocNonNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[inline]
    pub fn new_js_doc_non_nullable_type<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        Self::JSDocNonNullableType(JSDocNonNullableType::boxed(
            span,
            type_annotation,
            postfix,
            accessor,
        ))
    }

    /// Build a [`TSTupleElement::JSDocUnknownType`].
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_js_doc_unknown_type<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::JSDocUnknownType(JSDocUnknownType::boxed(span, accessor))
    }
}

impl TSAnyKeyword {
    /// Build a [`TSAnyKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSAnyKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSAnyKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSAnyKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSAnyKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSStringKeyword {
    /// Build a [`TSStringKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSStringKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSStringKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSStringKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSStringKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSBooleanKeyword {
    /// Build a [`TSBooleanKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSBooleanKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSBooleanKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSBooleanKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSBooleanKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSNumberKeyword {
    /// Build a [`TSNumberKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNumberKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSNumberKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNumberKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNumberKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSNeverKeyword {
    /// Build a [`TSNeverKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNeverKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSNeverKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNeverKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNeverKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSIntrinsicKeyword {
    /// Build a [`TSIntrinsicKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSIntrinsicKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSIntrinsicKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSIntrinsicKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIntrinsicKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSUnknownKeyword {
    /// Build a [`TSUnknownKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSUnknownKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSUnknownKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSUnknownKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUnknownKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSNullKeyword {
    /// Build a [`TSNullKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNullKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSNullKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNullKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNullKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSUndefinedKeyword {
    /// Build a [`TSUndefinedKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSUndefinedKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSUndefinedKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSUndefinedKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUndefinedKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSVoidKeyword {
    /// Build a [`TSVoidKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSVoidKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSVoidKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSVoidKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSVoidKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSSymbolKeyword {
    /// Build a [`TSSymbolKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSSymbolKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSSymbolKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSSymbolKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSSymbolKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSThisType {
    /// Build a [`TSThisType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSThisType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSThisType { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSThisType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSThisType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSObjectKeyword {
    /// Build a [`TSObjectKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSObjectKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSObjectKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSObjectKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSObjectKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl TSBigIntKeyword {
    /// Build a [`TSBigIntKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSBigIntKeyword::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSBigIntKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSBigIntKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSBigIntKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeReference<'a> {
    /// Build a [`TSTypeReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeReference::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TSTypeReference {
            node_id: Cell::new(builder.node_id()),
            span,
            type_name,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, type_name, type_arguments, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTypeName<'a> {
    /// Build a [`TSTypeName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`TSTypeName::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`TSTypeName::QualifiedName`].
    ///
    /// This node contains a [`TSQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_qualified_name<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, accessor))
    }

    /// Build a [`TSTypeName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }
}

impl<'a> TSQualifiedName<'a> {
    /// Build a [`TSQualifiedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSQualifiedName::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSQualifiedName { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`TSQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSQualifiedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, left, right, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeParameterInstantiation<'a> {
    /// Build a [`TSTypeParameterInstantiation`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeParameterInstantiation::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        params: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeParameterInstantiation { node_id: Cell::new(builder.node_id()), span, params }
    }

    /// Build a [`TSTypeParameterInstantiation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeParameterInstantiation::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        params: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, params, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeParameter<'a> {
    /// Build a [`TSTypeParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeParameter::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// * `constraint`: Constrains what types can be passed to the type parameter.
    /// * `default`: Default value of the type parameter if no type is provided when using the type.
    /// * `in`: Was an `in` modifier keyword present?
    /// * `out`: Was an `out` modifier keyword present?
    /// * `const`: Was a `const` modifier keyword present?
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            name,
            constraint,
            default,
            r#in,
            out,
            r#const,
        }
    }

    /// Build a [`TSTypeParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeParameter::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// * `constraint`: Constrains what types can be passed to the type parameter.
    /// * `default`: Default value of the type parameter if no type is provided when using the type.
    /// * `in`: Was an `in` modifier keyword present?
    /// * `out`: Was an `out` modifier keyword present?
    /// * `const`: Was a `const` modifier keyword present?
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, name, constraint, default, r#in, out, r#const, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTypeParameterDeclaration<'a> {
    /// Build a [`TSTypeParameterDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeParameterDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeParameterDeclaration { node_id: Cell::new(builder.node_id()), span, params }
    }

    /// Build a [`TSTypeParameterDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeParameterDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        params: Vec<'a, TSTypeParameter<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, params, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeAliasDeclaration<'a> {
    /// Build a [`TSTypeAliasDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeAliasDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        let builder = accessor.builder();
        TSTypeAliasDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            type_annotation,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSTypeAliasDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAliasDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            Self::new(span, id, type_parameters, type_annotation, declare, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSTypeAliasDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeAliasDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        let builder = accessor.builder();
        TSTypeAliasDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            type_annotation,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSTypeAliasDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAliasDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                id,
                type_parameters,
                type_annotation,
                declare,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSClassImplements<'a> {
    /// Build a [`TSClassImplements`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: TSTypeName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TSClassImplements {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }
}

impl<'a> TSInterfaceDeclaration<'a> {
    /// Build a [`TSInterfaceDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSInterfaceDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        let builder = accessor.builder();
        TSInterfaceDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            extends,
            body: body.into_in(builder.allocator()),
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSInterfaceDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInterfaceDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Box::new_in(
            Self::new(span, id, type_parameters, extends, body, declare, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSInterfaceDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSInterfaceDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        let builder = accessor.builder();
        TSInterfaceDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            extends,
            body: body.into_in(builder.allocator()),
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSInterfaceDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInterfaceDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: Vec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, TSInterfaceBody<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                id,
                type_parameters,
                extends,
                body,
                declare,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSInterfaceBody<'a> {
    /// Build a [`TSInterfaceBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSInterfaceBody::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSInterfaceBody { node_id: Cell::new(builder.node_id()), span, body }
    }

    /// Build a [`TSInterfaceBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInterfaceBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        body: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, body, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSPropertySignature<'a> {
    /// Build a [`TSPropertySignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSPropertySignature::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `computed`
    /// * `optional`
    /// * `readonly`
    /// * `key`
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSPropertySignature {
            node_id: Cell::new(builder.node_id()),
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSPropertySignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSPropertySignature::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `computed`
    /// * `optional`
    /// * `readonly`
    /// * `key`
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, computed, optional, readonly, key, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSSignature<'a> {
    /// Build a [`TSSignature::TSIndexSignature`].
    ///
    /// This node contains a [`TSIndexSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[inline]
    pub fn new_ts_index_signature<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSIndexSignature(TSIndexSignature::boxed(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSPropertySignature`].
    ///
    /// This node contains a [`TSPropertySignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `computed`
    /// * `optional`
    /// * `readonly`
    /// * `key`
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_property_signature<A: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSPropertySignature(TSPropertySignature::boxed(
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSCallSignatureDeclaration`].
    ///
    /// This node contains a [`TSCallSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new_ts_call_signature_declaration<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSCallSignatureDeclaration(TSCallSignatureDeclaration::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSCallSignatureDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSCallSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_call_signature_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSCallSignatureDeclaration(TSCallSignatureDeclaration::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSConstructSignatureDeclaration`].
    ///
    /// This node contains a [`TSConstructSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new_ts_construct_signature_declaration<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSConstructSignatureDeclaration(TSConstructSignatureDeclaration::boxed(
            span,
            type_parameters,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSConstructSignatureDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSConstructSignatureDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_construct_signature_declaration_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSConstructSignatureDeclaration(TSConstructSignatureDeclaration::boxed_with_scope_id(
            span,
            type_parameters,
            params,
            return_type,
            scope_id,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSMethodSignature`].
    ///
    /// This node contains a [`TSMethodSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new_ts_method_signature<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSMethodSignature(TSMethodSignature::boxed(
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters,
            this_param,
            params,
            return_type,
            accessor,
        ))
    }

    /// Build a [`TSSignature::TSMethodSignature`] with `scope_id`.
    ///
    /// This node contains a [`TSMethodSignature`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_method_signature_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSMethodSignature(TSMethodSignature::boxed_with_scope_id(
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
            accessor,
        ))
    }
}

impl<'a> TSIndexSignature<'a> {
    /// Build a [`TSIndexSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSIndexSignature::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSIndexSignature {
            node_id: Cell::new(builder.node_id()),
            span,
            parameters,
            type_annotation: type_annotation.into_in(builder.allocator()),
            readonly,
            r#static,
        }
    }

    /// Build a [`TSIndexSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIndexSignature::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameters: Vec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            Self::new(span, parameters, type_annotation, readonly, r#static, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSCallSignatureDeclaration<'a> {
    /// Build a [`TSCallSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSCallSignatureDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSCallSignatureDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSCallSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSCallSignatureDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, type_parameters, this_param, params, return_type, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSCallSignatureDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSCallSignatureDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSCallSignatureDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSCallSignatureDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSCallSignatureDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSMethodSignature<'a> {
    /// Build a [`TSMethodSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSMethodSignature::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSMethodSignature {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSMethodSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSMethodSignature::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(
                span,
                key,
                computed,
                optional,
                kind,
                type_parameters,
                this_param,
                params,
                return_type,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSMethodSignature`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSMethodSignature::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSMethodSignature {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSMethodSignature`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSMethodSignature::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `computed`
    /// * `optional`
    /// * `kind`
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
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
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSConstructSignatureDeclaration<'a> {
    /// Build a [`TSConstructSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConstructSignatureDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSConstructSignatureDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructSignatureDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, type_parameters, params, return_type, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConstructSignatureDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSConstructSignatureDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructSignatureDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(span, type_parameters, params, return_type, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSIndexSignatureName<'a> {
    /// Build a [`TSIndexSignatureName`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, S1, T1>(
        span: Span,
        name: S1,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
        T1: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSIndexSignatureName {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }
}

impl<'a> TSInterfaceHeritage<'a> {
    /// Build a [`TSInterfaceHeritage`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TSInterfaceHeritage {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }
}

impl<'a> TSTypePredicate<'a> {
    /// Build a [`TSTypePredicate`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypePredicate::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = accessor.builder();
        TSTypePredicate {
            node_id: Cell::new(builder.node_id()),
            span,
            parameter_name,
            asserts,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypePredicate`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypePredicate::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, parameter_name, asserts, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTypePredicateName<'a> {
    /// Build a [`TSTypePredicateName::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, accessor))
    }

    /// Build a [`TSTypePredicateName::This`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::This(TSThisType::boxed(span, accessor))
    }
}

impl<'a> TSModuleDeclaration<'a> {
    /// Build a [`TSModuleDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSModuleDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSModuleDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            body,
            kind,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSModuleDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSModuleDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, id, body, kind, declare, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSModuleDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSModuleDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSModuleDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            body,
            kind,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSModuleDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSModuleDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, id, body, kind, declare, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSModuleDeclarationName<'a> {
    /// Build a [`TSModuleDeclarationName::Identifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(BindingIdentifier::new(span, name, accessor))
    }

    /// Build a [`TSModuleDeclarationName::Identifier`] with `symbol_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn new_identifier_with_symbol_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(BindingIdentifier::new_with_symbol_id(span, name, symbol_id, accessor))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, accessor))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<A: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            accessor,
        ))
    }
}

impl<'a> TSModuleDeclarationBody<'a> {
    /// Build a [`TSModuleDeclarationBody::TSModuleDeclaration`].
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn new_ts_module_declaration<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, accessor,
        ))
    }

    /// Build a [`TSModuleDeclarationBody::TSModuleDeclaration`] with `scope_id`.
    ///
    /// This node contains a [`TSModuleDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_ts_module_declaration_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, accessor,
        ))
    }

    /// Build a [`TSModuleDeclarationBody::TSModuleBlock`].
    ///
    /// This node contains a [`TSModuleBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[inline]
    pub fn new_ts_module_block<A: GetAstBuilder<'a>>(
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        Self::TSModuleBlock(TSModuleBlock::boxed(span, directives, body, accessor))
    }
}

impl<'a> TSGlobalDeclaration<'a> {
    /// Build a [`TSGlobalDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSGlobalDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSGlobalDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            global_span,
            body,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSGlobalDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSGlobalDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, global_span, body, declare, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSGlobalDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSGlobalDeclaration::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSGlobalDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            global_span,
            body,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSGlobalDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSGlobalDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(span, global_span, body, declare, scope_id, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSModuleBlock<'a> {
    /// Build a [`TSModuleBlock`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSModuleBlock::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSModuleBlock { node_id: Cell::new(builder.node_id()), span, directives, body }
    }

    /// Build a [`TSModuleBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSModuleBlock::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        directives: Vec<'a, Directive<'a>>,
        body: Vec<'a, Statement<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, directives, body, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeLiteral<'a> {
    /// Build a [`TSTypeLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeLiteral::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeLiteral { node_id: Cell::new(builder.node_id()), span, members }
    }

    /// Build a [`TSTypeLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        members: Vec<'a, TSSignature<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, members, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSInferType<'a> {
    /// Build a [`TSInferType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSInferType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(span: Span, type_parameter: T1, accessor: &A) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        let builder = accessor.builder();
        TSInferType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameter: type_parameter.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSInferType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInferType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameter<'a>>>,
    {
        Box::new_in(Self::new(span, type_parameter, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSTypeQuery<'a> {
    /// Build a [`TSTypeQuery`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeQuery::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TSTypeQuery {
            node_id: Cell::new(builder.node_id()),
            span,
            expr_name,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeQuery`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeQuery::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, expr_name, type_arguments, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTypeQueryExprName<'a> {
    /// Build a [`TSTypeQueryExprName::TSImportType`].
    ///
    /// This node contains a [`TSImportType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn new_ts_import_type<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            accessor,
        ))
    }

    /// Build a [`TSTypeQueryExprName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`TSTypeQueryExprName::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`TSTypeQueryExprName::QualifiedName`].
    ///
    /// This node contains a [`TSQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_qualified_name<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, accessor))
    }

    /// Build a [`TSTypeQueryExprName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, accessor))
    }
}

impl<'a> TSImportType<'a> {
    /// Build a [`TSImportType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSImportType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = accessor.builder();
        TSImportType {
            node_id: Cell::new(builder.node_id()),
            span,
            source,
            options: options.into_in(builder.allocator()),
            qualifier,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSImportType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Box::new_in(
            Self::new(span, source, options, qualifier, type_arguments, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSImportTypeQualifier<'a> {
    /// Build a [`TSImportTypeQualifier::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_identifier<A: GetAstBuilder<'a>, S1>(span: Span, name: S1, accessor: &A) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, accessor))
    }

    /// Build a [`TSImportTypeQualifier::QualifiedName`].
    ///
    /// This node contains a [`TSImportTypeQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_qualified_name<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::QualifiedName(TSImportTypeQualifiedName::boxed(span, left, right, accessor))
    }
}

impl<'a> TSImportTypeQualifiedName<'a> {
    /// Build a [`TSImportTypeQualifiedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSImportTypeQualifiedName::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSImportTypeQualifiedName { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`TSImportTypeQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportTypeQualifiedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, left, right, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSFunctionType<'a> {
    /// Build a [`TSFunctionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSFunctionType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSFunctionType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSFunctionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSFunctionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            Self::new(span, type_parameters, this_param, params, return_type, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSFunctionType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSFunctionType::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSFunctionType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameters: type_parameters.into_in(builder.allocator()),
            this_param: this_param.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSFunctionType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSFunctionType::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSConstructorType<'a> {
    /// Build a [`TSConstructorType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConstructorType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSConstructorType {
            node_id: Cell::new(builder.node_id()),
            span,
            r#abstract,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSConstructorType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructorType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            Self::new(span, r#abstract, type_parameters, params, return_type, accessor),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSConstructorType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSConstructorType::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = accessor.builder();
        TSConstructorType {
            node_id: Cell::new(builder.node_id()),
            span,
            r#abstract,
            type_parameters: type_parameters.into_in(builder.allocator()),
            params: params.into_in(builder.allocator()),
            return_type: return_type.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConstructorType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructorType::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Box<'a, TSTypeAnnotation<'a>>>,
    {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                r#abstract,
                type_parameters,
                params,
                return_type,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSMappedType<'a> {
    /// Build a [`TSMappedType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSMappedType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSMappedType {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSMappedType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSMappedType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }

    /// Build a [`TSMappedType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSMappedType::boxed_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    /// * `scope_id`
    #[inline]
    pub fn new_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSMappedType {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSMappedType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSMappedType::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<A: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new_with_scope_id(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
                scope_id,
                accessor,
            ),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTemplateLiteralType<'a> {
    /// Build a [`TSTemplateLiteralType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTemplateLiteralType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTemplateLiteralType { node_id: Cell::new(builder.node_id()), span, quasis, types }
    }

    /// Build a [`TSTemplateLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTemplateLiteralType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        quasis: Vec<'a, TemplateElement<'a>>,
        types: Vec<'a, TSType<'a>>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, quasis, types, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSAsExpression<'a> {
    /// Build a [`TSAsExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSAsExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSAsExpression { node_id: Cell::new(builder.node_id()), span, expression, type_annotation }
    }

    /// Build a [`TSAsExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSAsExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, expression, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSSatisfiesExpression<'a> {
    /// Build a [`TSSatisfiesExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSSatisfiesExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSSatisfiesExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_annotation,
        }
    }

    /// Build a [`TSSatisfiesExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSSatisfiesExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, expression, type_annotation, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSTypeAssertion<'a> {
    /// Build a [`TSTypeAssertion`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSTypeAssertion::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSTypeAssertion { node_id: Cell::new(builder.node_id()), span, type_annotation, expression }
    }

    /// Build a [`TSTypeAssertion`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAssertion::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, type_annotation, expression, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSImportEqualsDeclaration<'a> {
    /// Build a [`TSImportEqualsDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSImportEqualsDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSImportEqualsDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            module_reference,
            import_kind,
        }
    }

    /// Build a [`TSImportEqualsDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportEqualsDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, id, module_reference, import_kind, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> TSModuleReference<'a> {
    /// Build a [`TSModuleReference::ExternalModuleReference`].
    ///
    /// This node contains a [`TSExternalModuleReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_external_module_reference<A: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        accessor: &A,
    ) -> Self {
        Self::ExternalModuleReference(TSExternalModuleReference::boxed(span, expression, accessor))
    }

    /// Build a [`TSModuleReference::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, accessor))
    }

    /// Build a [`TSModuleReference::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<A: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        accessor: &A,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            accessor,
        ))
    }

    /// Build a [`TSModuleReference::QualifiedName`].
    ///
    /// This node contains a [`TSQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn new_qualified_name<A: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        accessor: &A,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, accessor))
    }
}

impl<'a> TSExternalModuleReference<'a> {
    /// Build a [`TSExternalModuleReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSExternalModuleReference::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        TSExternalModuleReference { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSExternalModuleReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSExternalModuleReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSNonNullExpression<'a> {
    /// Build a [`TSNonNullExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNonNullExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSNonNullExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSNonNullExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNonNullExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> Decorator<'a> {
    /// Build a [`Decorator`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        Decorator { node_id: Cell::new(builder.node_id()), span, expression }
    }
}

impl<'a> TSExportAssignment<'a> {
    /// Build a [`TSExportAssignment`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSExportAssignment::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSExportAssignment { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSExportAssignment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSExportAssignment::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, expression, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSNamespaceExportDeclaration<'a> {
    /// Build a [`TSNamespaceExportDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSNamespaceExportDeclaration::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(span: Span, id: IdentifierName<'a>, accessor: &A) -> Self {
        let builder = accessor.builder();
        TSNamespaceExportDeclaration { node_id: Cell::new(builder.node_id()), span, id }
    }

    /// Build a [`TSNamespaceExportDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNamespaceExportDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(Self::new(span, id, accessor), accessor.builder().allocator())
    }
}

impl<'a> TSInstantiationExpression<'a> {
    /// Build a [`TSInstantiationExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`TSInstantiationExpression::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Self
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        let builder = accessor.builder();
        TSInstantiationExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSInstantiationExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInstantiationExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        accessor: &A,
    ) -> Box<'a, Self>
    where
        T1: IntoIn<'a, Box<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Box::new_in(
            Self::new(span, expression, type_arguments, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> JSDocNullableType<'a> {
    /// Build a [`JSDocNullableType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSDocNullableType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSDocNullableType { node_id: Cell::new(builder.node_id()), span, type_annotation, postfix }
    }

    /// Build a [`JSDocNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocNullableType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, type_annotation, postfix, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl<'a> JSDocNonNullableType<'a> {
    /// Build a [`JSDocNonNullableType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSDocNonNullableType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[inline]
    pub fn new<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Self {
        let builder = accessor.builder();
        JSDocNonNullableType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_annotation,
            postfix,
        }
    }

    /// Build a [`JSDocNonNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocNonNullableType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[inline]
    pub fn boxed<A: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        accessor: &A,
    ) -> Box<'a, Self> {
        Box::new_in(
            Self::new(span, type_annotation, postfix, accessor),
            accessor.builder().allocator(),
        )
    }
}

impl JSDocUnknownType {
    /// Build a [`JSDocUnknownType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`JSDocUnknownType::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Self {
        let builder = accessor.builder();
        JSDocUnknownType { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`JSDocUnknownType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocUnknownType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, A: GetAstBuilder<'a>>(span: Span, accessor: &A) -> Box<'a, Self> {
        Box::new_in(Self::new(span, accessor), accessor.builder().allocator())
    }
}
