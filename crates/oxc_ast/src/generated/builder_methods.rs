// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/builder_methods.rs`.

//! AST node builder methods.

#![expect(clippy::default_trait_access)]

use std::cell::Cell;

use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator, IntoIn};
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: T1,
        hashbang: Option<Hashbang<'a>>,
        directives: T2,
        body: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Comment>>,
        T2: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T3: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        Program {
            node_id: Cell::new(builder.node_id()),
            span,
            source_type,
            source_text,
            comments: comments.into_in(builder.allocator()),
            hashbang,
            directives: directives.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: T1,
        hashbang: Option<Hashbang<'a>>,
        directives: T2,
        body: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Comment>>,
        T2: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T3: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        Program {
            node_id: Cell::new(builder.node_id()),
            span,
            source_type,
            source_text,
            comments: comments.into_in(builder.allocator()),
            hashbang,
            directives: directives.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
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
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build an [`Expression::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build an [`Expression::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build an [`Expression::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build an [`Expression::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build an [`Expression::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build an [`Expression::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build an [`Expression::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build an [`Expression::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build an [`Expression::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build an [`Expression::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        IdentifierName { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build an [`IdentifierName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        IdentifierReference {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            reference_id: Default::default(),
        }
    }

    /// Build an [`IdentifierReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        IdentifierReference {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            reference_id: Cell::new(Some(reference_id)),
        }
    }

    /// Build an [`IdentifierReference`] with `reference_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IdentifierReference::new_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn boxed_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(
            Self::new_with_reference_id(span, name, reference_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        BindingIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            symbol_id: Default::default(),
        }
    }

    /// Build a [`BindingIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new_with_symbol_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        BindingIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            name: name.into(),
            symbol_id: Cell::new(Some(symbol_id)),
        }
    }

    /// Build a [`BindingIdentifier`] with `symbol_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingIdentifier::new_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn boxed_with_symbol_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(
            Self::new_with_symbol_id(span, name, symbol_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        ThisExpression { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`ThisExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ThisExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, elements: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        let builder = builder.builder();
        ArrayExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            elements: elements.into_in(builder.allocator()),
        }
    }

    /// Build an [`ArrayExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, elements, builder), builder.builder())
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
    pub fn new_spread_element<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::SpreadElement(SpreadElement::boxed(span, argument, builder))
    }

    /// Build an [`ArrayExpressionElement::Elision`].
    ///
    /// This node contains an [`Elision`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_elision<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Elision(Elision::boxed(span, builder))
    }

    /// Build an [`ArrayExpressionElement::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build an [`ArrayExpressionElement::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build an [`ArrayExpressionElement::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build an [`ArrayExpressionElement::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build an [`ArrayExpressionElement::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build an [`ArrayExpressionElement::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build an [`ArrayExpressionElement::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build an [`ArrayExpressionElement::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build an [`ArrayExpressionElement::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build an [`ArrayExpressionElement::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build an [`ArrayExpressionElement::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        Elision { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build an [`Elision`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Elision::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, properties: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        let builder = builder.builder();
        ObjectExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            properties: properties.into_in(builder.allocator()),
        }
    }

    /// Build an [`ObjectExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, properties, builder), builder.builder())
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
    pub fn new_object_property<B: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        builder: &B,
    ) -> Self {
        Self::ObjectProperty(ObjectProperty::boxed(
            span, kind, key, value, method, shorthand, computed, builder,
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
    pub fn new_spread_property<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::SpreadProperty(SpreadElement::boxed(span, argument, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new(span, kind, key, value, method, shorthand, computed, builder),
            builder.builder(),
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
    pub fn new_static_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::StaticIdentifier(IdentifierName::boxed(span, name, builder))
    }

    /// Build a [`PropertyKey::PrivateIdentifier`].
    ///
    /// This node contains a [`PrivateIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn new_private_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::PrivateIdentifier(PrivateIdentifier::boxed(span, name, builder))
    }

    /// Build a [`PropertyKey::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build a [`PropertyKey::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build a [`PropertyKey::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build a [`PropertyKey::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build a [`PropertyKey::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build a [`PropertyKey::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build a [`PropertyKey::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build a [`PropertyKey::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build a [`PropertyKey::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build a [`PropertyKey::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build a [`PropertyKey::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        let builder = builder.builder();
        TemplateLiteral {
            node_id: Cell::new(builder.node_id()),
            span,
            quasis: quasis.into_in(builder.allocator()),
            expressions: expressions.into_in(builder.allocator()),
        }
    }

    /// Build a [`TemplateLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TemplateLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, quasis, expressions, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TaggedTemplateExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, tag, type_arguments, quasi, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new_with_lone_surrogates<B: GetAstBuilder<'a>>(
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ComputedMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object, expression, optional, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object, property, optional, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateFieldExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object, field, optional, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        let builder = builder.builder();
        CallExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments: arguments.into_in(builder.allocator()),
            optional,
            pure: Default::default(),
        }
    }

    /// Build a [`CallExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CallExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, callee, type_arguments, arguments, optional, builder),
            builder.builder(),
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
    pub fn new_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        let builder = builder.builder();
        CallExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments: arguments.into_in(builder.allocator()),
            optional,
            pure,
        }
    }

    /// Build a [`CallExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_pure(span, callee, type_arguments, arguments, optional, pure, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        let builder = builder.builder();
        NewExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments: arguments.into_in(builder.allocator()),
            pure: Default::default(),
        }
    }

    /// Build a [`NewExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NewExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, callee, type_arguments, arguments, builder),
            builder.builder(),
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
    pub fn new_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        let builder = builder.builder();
        NewExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            callee,
            type_arguments: type_arguments.into_in(builder.allocator()),
            arguments: arguments.into_in(builder.allocator()),
            pure,
        }
    }

    /// Build a [`NewExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NewExpression::new_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[inline]
    pub fn boxed_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_pure(span, callee, type_arguments, arguments, pure, builder),
            builder.builder(),
        )
    }
}

impl ImportMeta {
    /// Build an [`ImportMeta`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ImportMeta::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        ImportMeta { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build an [`ImportMeta`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportMeta::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
    }
}

impl NewTarget {
    /// Build a [`NewTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`NewTarget::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        NewTarget { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`NewTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NewTarget::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        SpreadElement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`SpreadElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SpreadElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new_spread_element<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::SpreadElement(SpreadElement::boxed(span, argument, builder))
    }

    /// Build an [`Argument::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build an [`Argument::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build an [`Argument::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build an [`Argument::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build an [`Argument::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build an [`Argument::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build an [`Argument::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build an [`Argument::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build an [`Argument::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build an [`Argument::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build an [`Argument::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        UpdateExpression { node_id: Cell::new(builder.node_id()), span, operator, prefix, argument }
    }

    /// Build an [`UpdateExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`UpdateExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, operator, prefix, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        UnaryExpression { node_id: Cell::new(builder.node_id()), span, operator, argument }
    }

    /// Build an [`UnaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`UnaryExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, operator, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        BinaryExpression { node_id: Cell::new(builder.node_id()), span, left, operator, right }
    }

    /// Build a [`BinaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BinaryExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, operator, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        PrivateInExpression { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`PrivateInExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateInExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        LogicalExpression { node_id: Cell::new(builder.node_id()), span, left, operator, right }
    }

    /// Build a [`LogicalExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`LogicalExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, operator, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ConditionalExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, test, consequent, alternate, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        AssignmentExpression { node_id: Cell::new(builder.node_id()), span, operator, left, right }
    }

    /// Build an [`AssignmentExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, operator, left, right, builder), builder.builder())
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
    pub fn new_assignment_target_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_assignment_target_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new_array_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, builder))
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
    pub fn new_object_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(span, properties, rest, builder))
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
    pub fn new_assignment_target_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_assignment_target_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new_array_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, builder))
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
    pub fn new_object_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(span, properties, rest, builder))
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        let builder = builder.builder();
        ArrayAssignmentTarget {
            node_id: Cell::new(builder.node_id()),
            span,
            elements: elements.into_in(builder.allocator()),
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ArrayAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayAssignmentTarget::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, elements, rest, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        let builder = builder.builder();
        ObjectAssignmentTarget {
            node_id: Cell::new(builder.node_id()),
            span,
            properties: properties.into_in(builder.allocator()),
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ObjectAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectAssignmentTarget::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, properties, rest, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        target: AssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        AssignmentTargetRest { node_id: Cell::new(builder.node_id()), span, target }
    }

    /// Build an [`AssignmentTargetRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetRest::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `target`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        target: AssignmentTarget<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, target, builder), builder.builder())
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
    pub fn new_assignment_target_with_default<B: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentTargetWithDefault(AssignmentTargetWithDefault::boxed(
            span, binding, init, builder,
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
    pub fn new_assignment_target_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_assignment_target_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new_array_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, builder))
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
    pub fn new_object_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(span, properties, rest, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        AssignmentTargetWithDefault { node_id: Cell::new(builder.node_id()), span, binding, init }
    }

    /// Build an [`AssignmentTargetWithDefault`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetWithDefault::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, binding, init, builder), builder.builder())
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
    pub fn new_assignment_target_property_identifier<B: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::AssignmentTargetPropertyIdentifier(AssignmentTargetPropertyIdentifier::boxed(
            span, binding, init, builder,
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
    pub fn new_assignment_target_property_property<B: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        builder: &B,
    ) -> Self {
        Self::AssignmentTargetPropertyProperty(AssignmentTargetPropertyProperty::boxed(
            span, name, binding, computed, builder,
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        AssignmentTargetPropertyIdentifier {
            node_id: Cell::new(builder.node_id()),
            span,
            binding,
            init,
        }
    }

    /// Build an [`AssignmentTargetPropertyIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetPropertyIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, binding, init, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentTargetPropertyProperty::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, name, binding, computed, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, expressions: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        let builder = builder.builder();
        SequenceExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expressions: expressions.into_in(builder.allocator()),
        }
    }

    /// Build a [`SequenceExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SequenceExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, expressions, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        Super { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`Super`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`Super::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        AwaitExpression { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build an [`AwaitExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AwaitExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ChainExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`ChainExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ChainExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        ParenthesizedExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`ParenthesizedExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ParenthesizedExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new_block_statement<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        Self::BlockStatement(BlockStatement::boxed(span, body, builder))
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
    pub fn new_block_statement_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        Self::BlockStatement(BlockStatement::boxed_with_scope_id(span, body, scope_id, builder))
    }

    /// Build a [`Statement::BreakStatement`].
    ///
    /// This node contains a [`BreakStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new_break_statement<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> Self {
        Self::BreakStatement(BreakStatement::boxed(span, label, builder))
    }

    /// Build a [`Statement::ContinueStatement`].
    ///
    /// This node contains a [`ContinueStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn new_continue_statement<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> Self {
        Self::ContinueStatement(ContinueStatement::boxed(span, label, builder))
    }

    /// Build a [`Statement::DebuggerStatement`].
    ///
    /// This node contains a [`DebuggerStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_debugger_statement<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::DebuggerStatement(DebuggerStatement::boxed(span, builder))
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
    pub fn new_do_while_statement<B: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::DoWhileStatement(DoWhileStatement::boxed(span, body, test, builder))
    }

    /// Build a [`Statement::EmptyStatement`].
    ///
    /// This node contains an [`EmptyStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_empty_statement<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::EmptyStatement(EmptyStatement::boxed(span, builder))
    }

    /// Build a [`Statement::ExpressionStatement`].
    ///
    /// This node contains an [`ExpressionStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_expression_statement<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ExpressionStatement(ExpressionStatement::boxed(span, expression, builder))
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
    pub fn new_for_in_statement<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::ForInStatement(ForInStatement::boxed(span, left, right, body, builder))
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
    pub fn new_for_in_statement_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::ForInStatement(ForInStatement::boxed_with_scope_id(
            span, left, right, body, scope_id, builder,
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
    pub fn new_for_of_statement<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::ForOfStatement(ForOfStatement::boxed(span, r#await, left, right, body, builder))
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
    pub fn new_for_of_statement_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::ForOfStatement(ForOfStatement::boxed_with_scope_id(
            span, r#await, left, right, body, scope_id, builder,
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
    pub fn new_for_statement<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::ForStatement(ForStatement::boxed(span, init, test, update, body, builder))
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
    pub fn new_for_statement_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::ForStatement(ForStatement::boxed_with_scope_id(
            span, init, test, update, body, scope_id, builder,
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
    pub fn new_if_statement<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        builder: &B,
    ) -> Self {
        Self::IfStatement(IfStatement::boxed(span, test, consequent, alternate, builder))
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
    pub fn new_labeled_statement<B: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::LabeledStatement(LabeledStatement::boxed(span, label, body, builder))
    }

    /// Build a [`Statement::ReturnStatement`].
    ///
    /// This node contains a [`ReturnStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn new_return_statement<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::ReturnStatement(ReturnStatement::boxed(span, argument, builder))
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
    pub fn new_switch_statement<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        Self::SwitchStatement(SwitchStatement::boxed(span, discriminant, cases, builder))
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
    pub fn new_switch_statement_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        Self::SwitchStatement(SwitchStatement::boxed_with_scope_id(
            span,
            discriminant,
            cases,
            scope_id,
            builder,
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
    pub fn new_throw_statement<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ThrowStatement(ThrowStatement::boxed(span, argument, builder))
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
    pub fn new_try_statement<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        Self::TryStatement(TryStatement::boxed(span, block, handler, finalizer, builder))
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
    pub fn new_while_statement<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::WhileStatement(WhileStatement::boxed(span, test, body, builder))
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
    pub fn new_with_statement<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        Self::WithStatement(WithStatement::boxed(span, object, body, builder))
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
    pub fn new_with_statement_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::WithStatement(WithStatement::boxed_with_scope_id(
            span, object, body, scope_id, builder,
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
    pub fn new_variable_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            builder,
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
    pub fn new_function_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_class_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_ts_type_alias_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            builder,
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
    pub fn new_ts_type_alias_declaration_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_interface_declaration<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            builder,
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
    pub fn new_ts_interface_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_enum_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSEnumDeclaration(TSEnumDeclaration::boxed(span, id, body, r#const, declare, builder))
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
    pub fn new_ts_module_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, builder,
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
    pub fn new_ts_module_declaration_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, builder,
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
    pub fn new_ts_global_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed(
            span,
            global_span,
            body,
            declare,
            builder,
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
    pub fn new_ts_global_declaration_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed_with_scope_id(
            span,
            global_span,
            body,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_import_equals_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        Self::TSImportEqualsDeclaration(TSImportEqualsDeclaration::boxed(
            span,
            id,
            module_reference,
            import_kind,
            builder,
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
    pub fn new_import_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ImportDeclaration(ImportDeclaration::boxed(
            span,
            specifiers,
            source,
            phase,
            with_clause,
            import_kind,
            builder,
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
    pub fn new_export_all_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ExportAllDeclaration(ExportAllDeclaration::boxed(
            span,
            exported,
            source,
            with_clause,
            export_kind,
            builder,
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
    pub fn new_export_default_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        builder: &B,
    ) -> Self {
        Self::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(span, declaration, builder))
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
    pub fn new_export_named_declaration<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: T1,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ExportSpecifier<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ExportNamedDeclaration(ExportNamedDeclaration::boxed(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            builder,
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
    pub fn new_ts_export_assignment<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSExportAssignment(TSExportAssignment::boxed(span, expression, builder))
    }

    /// Build a [`Statement::TSNamespaceExportDeclaration`].
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn new_ts_namespace_export_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNamespaceExportDeclaration(TSNamespaceExportDeclaration::boxed(span, id, builder))
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
    pub fn new<B: GetAstBuilder<'a>, S1>(
        span: Span,
        expression: StringLiteral<'a>,
        directive: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, value: S1, builder: &B) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        BlockStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`BlockStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BlockStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        BlockStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`BlockStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BlockStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new_with_scope_id(span, body, scope_id, builder), builder.builder())
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
    pub fn new_variable_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            builder,
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
    pub fn new_function_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_class_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_ts_type_alias_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            builder,
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
    pub fn new_ts_type_alias_declaration_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Self::TSTypeAliasDeclaration(TSTypeAliasDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_interface_declaration<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            builder,
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
    pub fn new_ts_interface_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_enum_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSEnumDeclaration(TSEnumDeclaration::boxed(span, id, body, r#const, declare, builder))
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
    pub fn new_ts_module_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, builder,
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
    pub fn new_ts_module_declaration_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, builder,
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
    pub fn new_ts_global_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed(
            span,
            global_span,
            body,
            declare,
            builder,
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
    pub fn new_ts_global_declaration_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSGlobalDeclaration(TSGlobalDeclaration::boxed_with_scope_id(
            span,
            global_span,
            body,
            declare,
            scope_id,
            builder,
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
    pub fn new_ts_import_equals_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        Self::TSImportEqualsDeclaration(TSImportEqualsDeclaration::boxed(
            span,
            id,
            module_reference,
            import_kind,
            builder,
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        let builder = builder.builder();
        VariableDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            declarations: declarations.into_in(builder.allocator()),
            declare,
        }
    }

    /// Build a [`VariableDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`VariableDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, kind, declarations, declare, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        type_annotation: T1,
        init: Option<Expression<'a>>,
        definite: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        EmptyStatement { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build an [`EmptyStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`EmptyStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        ExpressionStatement { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build an [`ExpressionStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExpressionStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        IfStatement { node_id: Cell::new(builder.node_id()), span, test, consequent, alternate }
    }

    /// Build an [`IfStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`IfStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, test, consequent, alternate, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        DoWhileStatement { node_id: Cell::new(builder.node_id()), span, body, test }
    }

    /// Build a [`DoWhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`DoWhileStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, body, test, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        WhileStatement { node_id: Cell::new(builder.node_id()), span, test, body }
    }

    /// Build a [`WhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WhileStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, test, body, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, init, test, update, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, init, test, update, body, scope_id, builder),
            builder.builder(),
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
    pub fn new_variable_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            builder,
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
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build a [`ForStatementInit::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build a [`ForStatementInit::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build a [`ForStatementInit::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build a [`ForStatementInit::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build a [`ForStatementInit::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build a [`ForStatementInit::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build a [`ForStatementInit::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build a [`ForStatementInit::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build a [`ForStatementInit::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build a [`ForStatementInit::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForInStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, right, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForInStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, left, right, body, scope_id, builder),
            builder.builder(),
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
    pub fn new_variable_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        kind: VariableDeclarationKind,
        declarations: T1,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, VariableDeclarator<'a>>>,
    {
        Self::VariableDeclaration(VariableDeclaration::boxed(
            span,
            kind,
            declarations,
            declare,
            builder,
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
    pub fn new_assignment_target_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_assignment_target_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::AssignmentTargetIdentifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new_array_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ArrayAssignmentTarget(ArrayAssignmentTarget::boxed(span, elements, rest, builder))
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
    pub fn new_object_assignment_target<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, AssignmentTargetProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        Self::ObjectAssignmentTarget(ObjectAssignmentTarget::boxed(span, properties, rest, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ForOfStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, r#await, left, right, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, r#await, left, right, body, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ContinueStatement { node_id: Cell::new(builder.node_id()), span, label }
    }

    /// Build a [`ContinueStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ContinueStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, label, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        BreakStatement { node_id: Cell::new(builder.node_id()), span, label }
    }

    /// Build a [`BreakStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BreakStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        label: Option<LabelIdentifier<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, label, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ReturnStatement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`ReturnStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ReturnStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, object, body, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        let builder = builder.builder();
        SwitchStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            discriminant,
            cases: cases.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`SwitchStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SwitchStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, discriminant, cases, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        let builder = builder.builder();
        SwitchStatement {
            node_id: Cell::new(builder.node_id()),
            span,
            discriminant,
            cases: cases.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`SwitchStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`SwitchStatement::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        discriminant: Expression<'a>,
        cases: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, SwitchCase<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, discriminant, cases, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        test: Option<Expression<'a>>,
        consequent: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        SwitchCase {
            node_id: Cell::new(builder.node_id()),
            span,
            test,
            consequent: consequent.into_in(builder.allocator()),
        }
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        LabeledStatement { node_id: Cell::new(builder.node_id()), span, label, body }
    }

    /// Build a [`LabeledStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`LabeledStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, label, body, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        ThrowStatement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`ThrowStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ThrowStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TryStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, block, handler, finalizer, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CatchClause::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, param, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`CatchClause::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, param, body, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        DebuggerStatement { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`DebuggerStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`DebuggerStatement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new_binding_identifier<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::BindingIdentifier(BindingIdentifier::boxed(span, name, builder))
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
    pub fn new_binding_identifier_with_symbol_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::BindingIdentifier(BindingIdentifier::boxed_with_symbol_id(
            span, name, symbol_id, builder,
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
    pub fn new_object_pattern<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, BindingProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        Self::ObjectPattern(ObjectPattern::boxed(span, properties, rest, builder))
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
    pub fn new_array_pattern<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<BindingPattern<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        Self::ArrayPattern(ArrayPattern::boxed(span, elements, rest, builder))
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
    pub fn new_assignment_pattern<B: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentPattern(AssignmentPattern::boxed(span, left, right, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        AssignmentPattern { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build an [`AssignmentPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AssignmentPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, BindingProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        let builder = builder.builder();
        ObjectPattern {
            node_id: Cell::new(builder.node_id()),
            span,
            properties: properties.into_in(builder.allocator()),
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ObjectPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ObjectPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        properties: T1,
        rest: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, BindingProperty<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, properties, rest, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<BindingPattern<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        let builder = builder.builder();
        ArrayPattern {
            node_id: Cell::new(builder.node_id()),
            span,
            elements: elements.into_in(builder.allocator()),
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build an [`ArrayPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ArrayPattern::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        elements: T1,
        rest: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Option<BindingPattern<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, elements, rest, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        argument: BindingPattern<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        BindingRestElement { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`BindingRestElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BindingRestElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: BindingPattern<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new_with_scope_id_and_pure_and_pife<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id_and_pure_and_pife<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        kind: FormalParameterKind,
        items: T1,
        rest: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, FormalParameter<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        let builder = builder.builder();
        FormalParameters {
            node_id: Cell::new(builder.node_id()),
            span,
            kind,
            items: items.into_in(builder.allocator()),
            rest: rest.into_in(builder.allocator()),
        }
    }

    /// Build a [`FormalParameters`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FormalParameters::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `items`
    /// * `rest`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        kind: FormalParameterKind,
        items: T1,
        rest: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, FormalParameter<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, kind, items, rest, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        decorators: T1,
        pattern: BindingPattern<'a>,
        type_annotation: T2,
        initializer: T3,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, Expression<'a>>>>,
    {
        let builder = builder.builder();
        FormalParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            decorators: decorators.into_in(builder.allocator()),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        decorators: T1,
        rest: BindingRestElement<'a>,
        type_annotation: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
        FormalParameterRest {
            node_id: Cell::new(builder.node_id()),
            span,
            decorators: decorators.into_in(builder.allocator()),
            rest,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`FormalParameterRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FormalParameterRest::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `rest`
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        decorators: T1,
        rest: BindingRestElement<'a>,
        type_annotation: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, decorators, rest, type_annotation, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        directives: T1,
        statements: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        FunctionBody {
            node_id: Cell::new(builder.node_id()),
            span,
            directives: directives.into_in(builder.allocator()),
            statements: statements.into_in(builder.allocator()),
        }
    }

    /// Build a [`FunctionBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`FunctionBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `statements`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        directives: T1,
        statements: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, directives, statements, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
                builder,
            ),
            builder.builder(),
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
    pub fn new_with_scope_id_and_pure_and_pife<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id_and_pure_and_pife<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        YieldExpression { node_id: Cell::new(builder.node_id()), span, delegate, argument }
    }

    /// Build a [`YieldExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`YieldExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, delegate, argument, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        let builder = builder.builder();
        Class {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators: decorators.into_in(builder.allocator()),
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(builder.allocator()),
            implements: implements.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            r#abstract,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`Class`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        let builder = builder.builder();
        Class {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators: decorators.into_in(builder.allocator()),
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(builder.allocator()),
            implements: implements.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            r#abstract,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Class`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ClassElement<'a>>>,
    {
        let builder = builder.builder();
        ClassBody {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
        }
    }

    /// Build a [`ClassBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ClassBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, ClassElement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, body, builder), builder.builder())
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
    pub fn new_static_block<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        Self::StaticBlock(StaticBlock::boxed(span, body, builder))
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
    pub fn new_static_block_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        Self::StaticBlock(StaticBlock::boxed_with_scope_id(span, body, scope_id, builder))
    }

    /// Build a [`ClassElement::Constructor`].
    ///
    /// This node contains a [`ClassConstructor`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `accessibility`
    /// * `value`
    #[inline]
    pub fn new_constructor<B: GetAstBuilder<'a>, T1>(
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        Self::Constructor(ClassConstructor::boxed(span, key, accessibility, value, builder))
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
    pub fn new_method_definition<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        value: T2,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
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
            builder,
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
    pub fn new_property_definition<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
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
            builder,
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
    pub fn new_accessor_property<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
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
            builder,
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
    pub fn new_ts_index_signature<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        parameters: T1,
        type_annotation: T2,
        readonly: bool,
        r#static: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSIndexSignatureName<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSIndexSignature(TSIndexSignature::boxed(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
            builder,
        ))
    }
}

impl<'a> ClassConstructor<'a> {
    /// Build a [`ClassConstructor`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`ClassConstructor::boxed`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `accessibility`
    /// * `value`
    #[inline]
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        let builder = builder.builder();
        ClassConstructor {
            node_id: Cell::new(builder.node_id()),
            span,
            key,
            accessibility,
            value: value.into_in(builder.allocator()),
        }
    }

    /// Build a [`ClassConstructor`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ClassConstructor::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `accessibility`
    /// * `value`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, key, accessibility, value, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        value: T2,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        let builder = builder.builder();
        MethodDefinition {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators: decorators.into_in(builder.allocator()),
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: MethodDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        value: T2,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
        PropertyDefinition {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators: decorators.into_in(builder.allocator()),
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        let builder = builder.builder();
        PrivateIdentifier { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build a [`PrivateIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`PrivateIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> ArenaBox<'a, Self>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        StaticBlock {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`StaticBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticBlock::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, body, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        StaticBlock {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`StaticBlock`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StaticBlock::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        body: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new_with_scope_id(span, body, scope_id, builder), builder.builder())
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
    pub fn new_import_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ImportDeclaration(ImportDeclaration::boxed(
            span,
            specifiers,
            source,
            phase,
            with_clause,
            import_kind,
            builder,
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
    pub fn new_export_all_declaration<B: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ExportAllDeclaration(ExportAllDeclaration::boxed(
            span,
            exported,
            source,
            with_clause,
            export_kind,
            builder,
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
    pub fn new_export_default_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        builder: &B,
    ) -> Self {
        Self::ExportDefaultDeclaration(ExportDefaultDeclaration::boxed(span, declaration, builder))
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
    pub fn new_export_named_declaration<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: T1,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ExportSpecifier<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        Self::ExportNamedDeclaration(ExportNamedDeclaration::boxed(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
            builder,
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
    pub fn new_ts_export_assignment<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSExportAssignment(TSExportAssignment::boxed(span, expression, builder))
    }

    /// Build a [`ModuleDeclaration::TSNamespaceExportDeclaration`].
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn new_ts_namespace_export_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNamespaceExportDeclaration(TSNamespaceExportDeclaration::boxed(span, id, builder))
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
        AccessorProperty {
            node_id: Cell::new(builder.node_id()),
            span,
            r#type,
            decorators: decorators.into_in(builder.allocator()),
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        r#type: AccessorPropertyType,
        decorators: T1,
        key: PropertyKey<'a>,
        type_annotation: T2,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ImportExpression { node_id: Cell::new(builder.node_id()), span, source, options, phase }
    }

    /// Build an [`ImportExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, source, options, phase, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, specifiers, source, phase, with_clause, import_kind, builder),
            builder.builder(),
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
    pub fn new_import_specifier<B: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        Self::ImportSpecifier(ImportSpecifier::boxed(span, imported, local, import_kind, builder))
    }

    /// Build an [`ImportDeclarationSpecifier::ImportDefaultSpecifier`].
    ///
    /// This node contains an [`ImportDefaultSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[inline]
    pub fn new_import_default_specifier<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::ImportDefaultSpecifier(ImportDefaultSpecifier::boxed(span, local, builder))
    }

    /// Build an [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`].
    ///
    /// This node contains an [`ImportNamespaceSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[inline]
    pub fn new_import_namespace_specifier<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::ImportNamespaceSpecifier(ImportNamespaceSpecifier::boxed(span, local, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, imported, local, import_kind, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ImportDefaultSpecifier { node_id: Cell::new(builder.node_id()), span, local }
    }

    /// Build an [`ImportDefaultSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportDefaultSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, local, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ImportNamespaceSpecifier { node_id: Cell::new(builder.node_id()), span, local }
    }

    /// Build an [`ImportNamespaceSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ImportNamespaceSpecifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        local: BindingIdentifier<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, local, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ImportAttribute<'a>>>,
    {
        let builder = builder.builder();
        WithClause {
            node_id: Cell::new(builder.node_id()),
            span,
            keyword,
            with_entries: with_entries.into_in(builder.allocator()),
        }
    }

    /// Build a [`WithClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`WithClause::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `keyword`
    /// * `with_entries`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, ImportAttribute<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, keyword, with_entries, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::new(span, name, builder))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, builder))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: T1,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ExportSpecifier<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        let builder = builder.builder();
        ExportNamedDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            declaration,
            specifiers: specifiers.into_in(builder.allocator()),
            source,
            export_kind,
            with_clause: with_clause.into_in(builder.allocator()),
        }
    }

    /// Build an [`ExportNamedDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: T1,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, ExportSpecifier<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, declaration, specifiers, source, export_kind, with_clause, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        ExportDefaultDeclaration { node_id: Cell::new(builder.node_id()), span, declaration }
    }

    /// Build an [`ExportDefaultDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExportDefaultDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, declaration, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`ExportAllDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, exported, source, with_clause, export_kind, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new_function_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_class_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_ts_interface_declaration<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            builder,
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
    pub fn new_ts_interface_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Self::TSInterfaceDeclaration(TSInterfaceDeclaration::boxed_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
            builder,
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
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build an [`ExportDefaultDeclarationKind::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build an [`ExportDefaultDeclarationKind::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new_identifier_name<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierName(IdentifierName::new(span, name, builder))
    }

    /// Build a [`ModuleExportName::IdentifierReference`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::new(span, name, builder))
    }

    /// Build a [`ModuleExportName::IdentifierReference`] with `reference_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[inline]
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::new_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build a [`ModuleExportName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, builder))
    }

    /// Build a [`ModuleExportName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        let builder = builder.builder();
        V8IntrinsicExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            name,
            arguments: arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`V8IntrinsicExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`V8IntrinsicExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, name, arguments, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        let builder = builder.builder();
        BooleanLiteral { node_id: Cell::new(builder.node_id()), span, value }
    }

    /// Build a [`BooleanLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BooleanLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(
        span: Span,
        value: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, value, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        NullLiteral { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`NullLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NullLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        NumericLiteral { node_id: Cell::new(builder.node_id()), span, value, raw, base }
    }

    /// Build a [`NumericLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`NumericLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, value, raw, base, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StringLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(Self::new(span, value, raw, builder), builder.builder())
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
    pub fn new_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`StringLiteral::new_with_lone_surrogates`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn boxed_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(
            Self::new_with_lone_surrogates(span, value, raw, lone_surrogates, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`BigIntLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(Self::new(span, value, raw, base, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        RegExpLiteral { node_id: Cell::new(builder.node_id()), span, regex, raw }
    }

    /// Build a [`RegExpLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`RegExpLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, regex, raw, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        let builder = builder.builder();
        JSXElement {
            node_id: Cell::new(builder.node_id()),
            span,
            opening_element: opening_element.into_in(builder.allocator()),
            children: children.into_in(builder.allocator()),
            closing_element: closing_element.into_in(builder.allocator()),
        }
    }

    /// Build a [`JSXElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, opening_element, children, closing_element, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXAttributeItem<'a>>>,
    {
        let builder = builder.builder();
        JSXOpeningElement {
            node_id: Cell::new(builder.node_id()),
            span,
            name,
            type_arguments: type_arguments.into_in(builder.allocator()),
            attributes: attributes.into_in(builder.allocator()),
        }
    }

    /// Build a [`JSXOpeningElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXOpeningElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    /// * `type_arguments`: Type parameters for generic JSX elements.
    /// * `attributes`: List of JSX attributes. In React-like applications, these become props.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXAttributeItem<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, name, type_arguments, attributes, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, name: JSXElementName<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        JSXClosingElement { node_id: Cell::new(builder.node_id()), span, name }
    }

    /// Build a [`JSXClosingElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXClosingElement::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The tag name, e.g. `Foo` in `</Foo>`.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        name: JSXElementName<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        let builder = builder.builder();
        JSXFragment {
            node_id: Cell::new(builder.node_id()),
            span,
            opening_fragment,
            children: children.into_in(builder.allocator()),
            closing_fragment,
        }
    }

    /// Build a [`JSXFragment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXFragment::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, opening_fragment, children, closing_fragment, builder),
            builder.builder(),
        )
    }
}

impl JSXOpeningFragment {
    /// Build a [`JSXOpeningFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        JSXOpeningFragment { node_id: Cell::new(builder.node_id()), span }
    }
}

impl JSXClosingFragment {
    /// Build a [`JSXClosingFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Identifier(JSXIdentifier::boxed(span, name, builder))
    }

    /// Build a [`JSXElementName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_namespaced_name<B: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::NamespacedName(JSXNamespacedName::boxed(span, namespace, name, builder))
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
    pub fn new_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::MemberExpression(JSXMemberExpression::boxed(span, object, property, builder))
    }

    /// Build a [`JSXElementName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSXNamespacedName { node_id: Cell::new(builder.node_id()), span, namespace, name }
    }

    /// Build a [`JSXNamespacedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXNamespacedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, namespace, name, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSXMemberExpression { node_id: Cell::new(builder.node_id()), span, object, property }
    }

    /// Build a [`JSXMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXMemberExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object, property, builder), builder.builder())
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
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::MemberExpression(JSXMemberExpression::boxed(span, object, property, builder))
    }

    /// Build a [`JSXMemberExpressionObject::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSXExpressionContainer { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`JSXExpressionContainer`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXExpressionContainer::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new_empty_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::EmptyExpression(JSXEmptyExpression::boxed(span, builder))
    }

    /// Build a [`JSXExpression::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[inline]
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
    }

    /// Build a [`JSXExpression::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn new_null_literal<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NullLiteral(NullLiteral::boxed(span, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_reg_exp_literal<B: GetAstBuilder<'a>>(
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self {
        Self::RegExpLiteral(RegExpLiteral::boxed(span, regex, raw, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
    }

    /// Build a [`JSXExpression::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
        ))
    }

    /// Build a [`JSXExpression::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_super<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::Super(Super::boxed(span, builder))
    }

    /// Build a [`JSXExpression::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[inline]
    pub fn new_array_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        elements: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ArrayExpressionElement<'a>>>,
    {
        Self::ArrayExpression(ArrayExpression::boxed(span, elements, builder))
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
    pub fn new_arrow_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Self::ArrowFunctionExpression(ArrowFunctionExpression::boxed(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
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
                builder,
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
    pub fn new_assignment_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AssignmentExpression(AssignmentExpression::boxed(
            span, operator, left, right, builder,
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
    pub fn new_await_expression<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::AwaitExpression(AwaitExpression::boxed(span, argument, builder))
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
    pub fn new_binary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::BinaryExpression(BinaryExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_call_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            builder,
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
    pub fn new_call_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        optional: bool,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::CallExpression(CallExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
            builder,
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
    pub fn new_chain_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: ChainElement<'a>,
        builder: &B,
    ) -> Self {
        Self::ChainExpression(ChainExpression::boxed(span, expression, builder))
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
    pub fn new_class_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_class_expression_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
        span: Span,
        r#type: ClassType,
        decorators: T1,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T2,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T3,
        implements: T4,
        body: T5,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Decorator<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T4: IntoIn<'a, ArenaVec<'a, TSClassImplements<'a>>>,
        T5: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
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
            builder,
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
    pub fn new_conditional_expression<B: GetAstBuilder<'a>>(
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ConditionalExpression(ConditionalExpression::boxed(
            span, test, consequent, alternate, builder,
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
    pub fn new_function_expression<B: GetAstBuilder<'a>, T1, T2, T3, T4, T5>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
        B: GetAstBuilder<'a>,
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
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
            builder,
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
    pub fn new_import_expression<B: GetAstBuilder<'a>>(
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
        builder: &B,
    ) -> Self {
        Self::ImportExpression(ImportExpression::boxed(span, source, options, phase, builder))
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
    pub fn new_logical_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::LogicalExpression(LogicalExpression::boxed(span, left, operator, right, builder))
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
    pub fn new_new_expression<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed(span, callee, type_arguments, arguments, builder))
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
    pub fn new_new_expression_with_pure<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: T2,
        pure: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::NewExpression(NewExpression::boxed_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
            builder,
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
    pub fn new_object_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        properties: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, ObjectPropertyKind<'a>>>,
    {
        Self::ObjectExpression(ObjectExpression::boxed(span, properties, builder))
    }

    /// Build a [`JSXExpression::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_parenthesized_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::ParenthesizedExpression(ParenthesizedExpression::boxed(span, expression, builder))
    }

    /// Build a [`JSXExpression::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[inline]
    pub fn new_sequence_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expressions: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::SequenceExpression(SequenceExpression::boxed(span, expressions, builder))
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
    pub fn new_tagged_template_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TaggedTemplateExpression(TaggedTemplateExpression::boxed(
            span,
            tag,
            type_arguments,
            quasi,
            builder,
        ))
    }

    /// Build a [`JSXExpression::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_update_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
        builder: &B,
    ) -> Self {
        Self::UpdateExpression(UpdateExpression::boxed(span, operator, prefix, argument, builder))
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
    pub fn new_yield_expression<B: GetAstBuilder<'a>>(
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        Self::YieldExpression(YieldExpression::boxed(span, delegate, argument, builder))
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
    pub fn new_private_in_expression<B: GetAstBuilder<'a>>(
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::PrivateInExpression(PrivateInExpression::boxed(span, left, right, builder))
    }

    /// Build a [`JSXExpression::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_import_meta<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ImportMeta(ImportMeta::boxed(span, builder))
    }

    /// Build a [`JSXExpression::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_new_target<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::NewTarget(NewTarget::boxed(span, builder))
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
    pub fn new_jsx_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::JSXElement(JSXElement::boxed(
            span,
            opening_element,
            children,
            closing_element,
            builder,
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
    pub fn new_jsx_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::JSXFragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_ts_as_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSAsExpression(TSAsExpression::boxed(span, expression, type_annotation, builder))
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
    pub fn new_ts_satisfies_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSSatisfiesExpression(TSSatisfiesExpression::boxed(
            span,
            expression,
            type_annotation,
            builder,
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
    pub fn new_ts_type_assertion<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeAssertion(TSTypeAssertion::boxed(span, type_annotation, expression, builder))
    }

    /// Build a [`JSXExpression::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new_ts_non_null_expression<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::TSNonNullExpression(TSNonNullExpression::boxed(span, expression, builder))
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
    pub fn new_ts_instantiation_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Self::TSInstantiationExpression(TSInstantiationExpression::boxed(
            span,
            expression,
            type_arguments,
            builder,
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
    pub fn new_v8_intrinsic_expression<B: GetAstBuilder<'a>, T1>(
        span: Span,
        name: IdentifierName<'a>,
        arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Argument<'a>>>,
    {
        Self::V8IntrinsicExpression(V8IntrinsicExpression::boxed(span, name, arguments, builder))
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
    pub fn new_computed_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::ComputedMemberExpression(ComputedMemberExpression::boxed(
            span, object, expression, optional, builder,
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
    pub fn new_static_member_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::StaticMemberExpression(StaticMemberExpression::boxed(
            span, object, property, optional, builder,
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
    pub fn new_private_field_expression<B: GetAstBuilder<'a>>(
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::PrivateFieldExpression(PrivateFieldExpression::boxed(
            span, object, field, optional, builder,
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        JSXEmptyExpression { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`JSXEmptyExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXEmptyExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new_attribute<B: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        builder: &B,
    ) -> Self {
        Self::Attribute(JSXAttribute::boxed(span, name, value, builder))
    }

    /// Build a [`JSXAttributeItem::SpreadAttribute`].
    ///
    /// This node contains a [`JSXSpreadAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn new_spread_attribute<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::SpreadAttribute(JSXSpreadAttribute::boxed(span, argument, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSXAttribute { node_id: Cell::new(builder.node_id()), span, name, value }
    }

    /// Build a [`JSXAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXAttribute::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, name, value, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, argument: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        JSXSpreadAttribute { node_id: Cell::new(builder.node_id()), span, argument }
    }

    /// Build a [`JSXSpreadAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXSpreadAttribute::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        argument: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, argument, builder), builder.builder())
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Identifier(JSXIdentifier::boxed(span, name, builder))
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
    pub fn new_namespaced_name<B: GetAstBuilder<'a>>(
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
        builder: &B,
    ) -> Self {
        Self::NamespacedName(JSXNamespacedName::boxed(span, namespace, name, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_expression_container<B: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        builder: &B,
    ) -> Self {
        Self::ExpressionContainer(JSXExpressionContainer::boxed(span, expression, builder))
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
    pub fn new_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::Element(JSXElement::boxed(span, opening_element, children, closing_element, builder))
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
    pub fn new_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::Fragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
        JSXIdentifier { node_id: Cell::new(builder.node_id()), span, name: name.into() }
    }

    /// Build a [`JSXIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXIdentifier::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> ArenaBox<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(Self::new(span, name, builder), builder.builder())
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
    pub fn new_text<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::Text(JSXText::boxed(span, value, raw, builder))
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
    pub fn new_element<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        opening_element: T1,
        children: T2,
        closing_element: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Self::Element(JSXElement::boxed(span, opening_element, children, closing_element, builder))
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
    pub fn new_fragment<B: GetAstBuilder<'a>, T1>(
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: T1,
        closing_fragment: JSXClosingFragment,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, JSXChild<'a>>>,
    {
        Self::Fragment(JSXFragment::boxed(
            span,
            opening_fragment,
            children,
            closing_fragment,
            builder,
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
    pub fn new_expression_container<B: GetAstBuilder<'a>>(
        span: Span,
        expression: JSXExpression<'a>,
        builder: &B,
    ) -> Self {
        Self::ExpressionContainer(JSXExpressionContainer::boxed(span, expression, builder))
    }

    /// Build a [`JSXChild::Spread`].
    ///
    /// This node contains a [`JSXSpreadChild`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[inline]
    pub fn new_spread<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::Spread(JSXSpreadChild::boxed(span, expression, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        JSXSpreadChild { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`JSXSpreadChild`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXSpreadChild::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        let builder = builder.builder();
        JSXText { node_id: Cell::new(builder.node_id()), span, value: value.into(), raw }
    }

    /// Build a [`JSXText`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSXText::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(Self::new(span, value, raw, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        this_span: Span,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
        TSThisParameter {
            node_id: Cell::new(builder.node_id()),
            span,
            this_span,
            type_annotation: type_annotation.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSThisParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSThisParameter::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `this_span`
    /// * `type_annotation`: Type type the `this` keyword will have in the function
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        this_span: Span,
        type_annotation: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, this_span, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSEnumDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, id, body, r#const, declare, builder), builder.builder())
    }
}

impl<'a> TSEnumBody<'a> {
    /// Build a [`TSEnumBody`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, members: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSEnumMember<'a>>>,
    {
        let builder = builder.builder();
        TSEnumBody {
            node_id: Cell::new(builder.node_id()),
            span,
            members: members.into_in(builder.allocator()),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        members: T1,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSEnumMember<'a>>>,
    {
        let builder = builder.builder();
        TSEnumBody {
            node_id: Cell::new(builder.node_id()),
            span,
            members: members.into_in(builder.allocator()),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, builder))
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
    pub fn new_string<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::String(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::String(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_computed_string<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::ComputedString(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_computed_string_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::ComputedString(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_computed_template_string<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::ComputedTemplateString(TemplateLiteral::boxed(span, quasis, expressions, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, type_annotation: TSType<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSTypeAnnotation { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSTypeAnnotation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAnnotation::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`: The actual type in the annotation
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, literal: TSLiteral<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSLiteralType { node_id: Cell::new(builder.node_id()), span, literal }
    }

    /// Build a [`TSLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSLiteralType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, literal, builder), builder.builder())
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
    pub fn new_boolean_literal<B: GetAstBuilder<'a>>(span: Span, value: bool, builder: &B) -> Self {
        Self::BooleanLiteral(BooleanLiteral::boxed(span, value, builder))
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
    pub fn new_numeric_literal<B: GetAstBuilder<'a>>(
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
        builder: &B,
    ) -> Self {
        Self::NumericLiteral(NumericLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_big_int_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::BigIntLiteral(BigIntLiteral::boxed(span, value, raw, base, builder))
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
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed(span, value, raw, builder))
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
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::boxed_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_template_literal<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        expressions: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Expression<'a>>>,
    {
        Self::TemplateLiteral(TemplateLiteral::boxed(span, quasis, expressions, builder))
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
    pub fn new_unary_expression<B: GetAstBuilder<'a>>(
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
        builder: &B,
    ) -> Self {
        Self::UnaryExpression(UnaryExpression::boxed(span, operator, argument, builder))
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
    pub fn new_ts_any_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSAnyKeyword(TSAnyKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSBigIntKeyword`].
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_big_int_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSBigIntKeyword(TSBigIntKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSBooleanKeyword`].
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_boolean_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSBooleanKeyword(TSBooleanKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSIntrinsicKeyword`].
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_intrinsic_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSIntrinsicKeyword(TSIntrinsicKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSNeverKeyword`].
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_never_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNeverKeyword(TSNeverKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSNullKeyword`].
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_null_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNullKeyword(TSNullKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSNumberKeyword`].
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_number_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNumberKeyword(TSNumberKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSObjectKeyword`].
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_object_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSObjectKeyword(TSObjectKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSStringKeyword`].
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_string_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSStringKeyword(TSStringKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSSymbolKeyword`].
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_symbol_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSSymbolKeyword(TSSymbolKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSUndefinedKeyword`].
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_undefined_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSUndefinedKeyword(TSUndefinedKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSUnknownKeyword`].
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_unknown_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSUnknownKeyword(TSUnknownKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSVoidKeyword`].
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_void_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSVoidKeyword(TSVoidKeyword::boxed(span, builder))
    }

    /// Build a [`TSType::TSArrayType`].
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn new_ts_array_type<B: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSArrayType(TSArrayType::boxed(span, element_type, builder))
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
    pub fn new_ts_conditional_type<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            builder,
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
    pub fn new_ts_conditional_type_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed_with_scope_id(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id,
            builder,
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
    pub fn new_ts_constructor_type<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            builder,
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
    pub fn new_ts_constructor_type_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed_with_scope_id(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_function_type<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            builder,
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
    pub fn new_ts_function_type_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_import_type<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            builder,
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
    pub fn new_ts_indexed_access_type<B: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSIndexedAccessType(TSIndexedAccessType::boxed(
            span,
            object_type,
            index_type,
            builder,
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
    pub fn new_ts_infer_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        Self::TSInferType(TSInferType::boxed(span, type_parameter, builder))
    }

    /// Build a [`TSType::TSIntersectionType`].
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn new_ts_intersection_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        types: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSIntersectionType(TSIntersectionType::boxed(span, types, builder))
    }

    /// Build a [`TSType::TSLiteralType`].
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn new_ts_literal_type<B: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        builder: &B,
    ) -> Self {
        Self::TSLiteralType(TSLiteralType::boxed(span, literal, builder))
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
    pub fn new_ts_mapped_type<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        builder: &B,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            builder,
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
    pub fn new_ts_mapped_type_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        builder: &B,
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
            builder,
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
    pub fn new_ts_named_tuple_member<B: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::TSNamedTupleMember(TSNamedTupleMember::boxed(
            span,
            label,
            element_type,
            optional,
            builder,
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
    pub fn new_ts_template_literal_type<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        types: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSTemplateLiteralType(TSTemplateLiteralType::boxed(span, quasis, types, builder))
    }

    /// Build a [`TSType::TSThisType`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_this_type<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSThisType(TSThisType::boxed(span, builder))
    }

    /// Build a [`TSType::TSTupleType`].
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn new_ts_tuple_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        element_types: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTupleElement<'a>>>,
    {
        Self::TSTupleType(TSTupleType::boxed(span, element_types, builder))
    }

    /// Build a [`TSType::TSTypeLiteral`].
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new_ts_type_literal<B: GetAstBuilder<'a>, T1>(
        span: Span,
        members: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        Self::TSTypeLiteral(TSTypeLiteral::boxed(span, members, builder))
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
    pub fn new_ts_type_operator_type<B: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeOperatorType(TSTypeOperator::boxed(span, operator, type_annotation, builder))
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
    pub fn new_ts_type_predicate<B: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSTypePredicate(TSTypePredicate::boxed(
            span,
            parameter_name,
            asserts,
            type_annotation,
            builder,
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
    pub fn new_ts_type_query<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeQuery(TSTypeQuery::boxed(span, expr_name, type_arguments, builder))
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
    pub fn new_ts_type_reference<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeReference(TSTypeReference::boxed(span, type_name, type_arguments, builder))
    }

    /// Build a [`TSType::TSUnionType`].
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn new_ts_union_type<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSUnionType(TSUnionType::boxed(span, types, builder))
    }

    /// Build a [`TSType::TSParenthesizedType`].
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_parenthesized_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSParenthesizedType(TSParenthesizedType::boxed(span, type_annotation, builder))
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
    pub fn new_js_doc_nullable_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        Self::JSDocNullableType(JSDocNullableType::boxed(span, type_annotation, postfix, builder))
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
    pub fn new_js_doc_non_nullable_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        Self::JSDocNonNullableType(JSDocNonNullableType::boxed(
            span,
            type_annotation,
            postfix,
            builder,
        ))
    }

    /// Build a [`TSType::JSDocUnknownType`].
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_js_doc_unknown_type<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::JSDocUnknownType(JSDocUnknownType::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConditionalType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new(span, check_type, extends_type, true_type, false_type, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                check_type,
                extends_type,
                true_type,
                false_type,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        let builder = builder.builder();
        TSUnionType {
            node_id: Cell::new(builder.node_id()),
            span,
            types: types.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSUnionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUnionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, types, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        let builder = builder.builder();
        TSIntersectionType {
            node_id: Cell::new(builder.node_id()),
            span,
            types: types.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSIntersectionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIntersectionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, types, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, type_annotation: TSType<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSParenthesizedType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSParenthesizedType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSParenthesizedType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSTypeOperator { node_id: Cell::new(builder.node_id()), span, operator, type_annotation }
    }

    /// Build a [`TSTypeOperator`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeOperator::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, operator, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, element_type: TSType<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSArrayType { node_id: Cell::new(builder.node_id()), span, element_type }
    }

    /// Build a [`TSArrayType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSArrayType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, element_type, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSIndexedAccessType { node_id: Cell::new(builder.node_id()), span, object_type, index_type }
    }

    /// Build a [`TSIndexedAccessType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIndexedAccessType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, object_type, index_type, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, element_types: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTupleElement<'a>>>,
    {
        let builder = builder.builder();
        TSTupleType {
            node_id: Cell::new(builder.node_id()),
            span,
            element_types: element_types.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTupleType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTupleType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        element_types: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTupleElement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, element_types, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNamedTupleMember::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, label, element_type, optional, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, type_annotation: TSType<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSOptionalType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSOptionalType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSOptionalType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, type_annotation: TSType<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSRestType { node_id: Cell::new(builder.node_id()), span, type_annotation }
    }

    /// Build a [`TSRestType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSRestType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, builder), builder.builder())
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
    pub fn new_ts_optional_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSOptionalType(TSOptionalType::boxed(span, type_annotation, builder))
    }

    /// Build a [`TSTupleElement::TSRestType`].
    ///
    /// This node contains a [`TSRestType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_rest_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSRestType(TSRestType::boxed(span, type_annotation, builder))
    }

    /// Build a [`TSTupleElement::TSAnyKeyword`].
    ///
    /// This node contains a [`TSAnyKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_any_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSAnyKeyword(TSAnyKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSBigIntKeyword`].
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_big_int_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSBigIntKeyword(TSBigIntKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSBooleanKeyword`].
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_boolean_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSBooleanKeyword(TSBooleanKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSIntrinsicKeyword`].
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_intrinsic_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSIntrinsicKeyword(TSIntrinsicKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSNeverKeyword`].
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_never_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNeverKeyword(TSNeverKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSNullKeyword`].
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_null_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNullKeyword(TSNullKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSNumberKeyword`].
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_number_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSNumberKeyword(TSNumberKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSObjectKeyword`].
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_object_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSObjectKeyword(TSObjectKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSStringKeyword`].
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_string_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSStringKeyword(TSStringKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSSymbolKeyword`].
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_symbol_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSSymbolKeyword(TSSymbolKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSUndefinedKeyword`].
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_undefined_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSUndefinedKeyword(TSUndefinedKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSUnknownKeyword`].
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_unknown_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSUnknownKeyword(TSUnknownKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSVoidKeyword`].
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_void_keyword<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSVoidKeyword(TSVoidKeyword::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSArrayType`].
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[inline]
    pub fn new_ts_array_type<B: GetAstBuilder<'a>>(
        span: Span,
        element_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSArrayType(TSArrayType::boxed(span, element_type, builder))
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
    pub fn new_ts_conditional_type<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            builder,
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
    pub fn new_ts_conditional_type_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSConditionalType(TSConditionalType::boxed_with_scope_id(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id,
            builder,
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
    pub fn new_ts_constructor_type<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            builder,
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
    pub fn new_ts_constructor_type_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSConstructorType(TSConstructorType::boxed_with_scope_id(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_function_type<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            builder,
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
    pub fn new_ts_function_type_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSFunctionType(TSFunctionType::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_import_type<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            builder,
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
    pub fn new_ts_indexed_access_type<B: GetAstBuilder<'a>>(
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSIndexedAccessType(TSIndexedAccessType::boxed(
            span,
            object_type,
            index_type,
            builder,
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
    pub fn new_ts_infer_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        Self::TSInferType(TSInferType::boxed(span, type_parameter, builder))
    }

    /// Build a [`TSTupleElement::TSIntersectionType`].
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[inline]
    pub fn new_ts_intersection_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        types: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSIntersectionType(TSIntersectionType::boxed(span, types, builder))
    }

    /// Build a [`TSTupleElement::TSLiteralType`].
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[inline]
    pub fn new_ts_literal_type<B: GetAstBuilder<'a>>(
        span: Span,
        literal: TSLiteral<'a>,
        builder: &B,
    ) -> Self {
        Self::TSLiteralType(TSLiteralType::boxed(span, literal, builder))
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
    pub fn new_ts_mapped_type<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        builder: &B,
    ) -> Self {
        Self::TSMappedType(TSMappedType::boxed(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            builder,
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
    pub fn new_ts_mapped_type_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        builder: &B,
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
            builder,
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
    pub fn new_ts_named_tuple_member<B: GetAstBuilder<'a>>(
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
        builder: &B,
    ) -> Self {
        Self::TSNamedTupleMember(TSNamedTupleMember::boxed(
            span,
            label,
            element_type,
            optional,
            builder,
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
    pub fn new_ts_template_literal_type<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        types: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSTemplateLiteralType(TSTemplateLiteralType::boxed(span, quasis, types, builder))
    }

    /// Build a [`TSTupleElement::TSThisType`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_ts_this_type<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::TSThisType(TSThisType::boxed(span, builder))
    }

    /// Build a [`TSTupleElement::TSTupleType`].
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[inline]
    pub fn new_ts_tuple_type<B: GetAstBuilder<'a>, T1>(
        span: Span,
        element_types: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTupleElement<'a>>>,
    {
        Self::TSTupleType(TSTupleType::boxed(span, element_types, builder))
    }

    /// Build a [`TSTupleElement::TSTypeLiteral`].
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn new_ts_type_literal<B: GetAstBuilder<'a>, T1>(
        span: Span,
        members: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        Self::TSTypeLiteral(TSTypeLiteral::boxed(span, members, builder))
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
    pub fn new_ts_type_operator_type<B: GetAstBuilder<'a>>(
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSTypeOperatorType(TSTypeOperator::boxed(span, operator, type_annotation, builder))
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
    pub fn new_ts_type_predicate<B: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSTypePredicate(TSTypePredicate::boxed(
            span,
            parameter_name,
            asserts,
            type_annotation,
            builder,
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
    pub fn new_ts_type_query<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeQuery(TSTypeQuery::boxed(span, expr_name, type_arguments, builder))
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
    pub fn new_ts_type_reference<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSTypeReference(TSTypeReference::boxed(span, type_name, type_arguments, builder))
    }

    /// Build a [`TSTupleElement::TSUnionType`].
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[inline]
    pub fn new_ts_union_type<B: GetAstBuilder<'a>, T1>(span: Span, types: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        Self::TSUnionType(TSUnionType::boxed(span, types, builder))
    }

    /// Build a [`TSTupleElement::TSParenthesizedType`].
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[inline]
    pub fn new_ts_parenthesized_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        Self::TSParenthesizedType(TSParenthesizedType::boxed(span, type_annotation, builder))
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
    pub fn new_js_doc_nullable_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        Self::JSDocNullableType(JSDocNullableType::boxed(span, type_annotation, postfix, builder))
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
    pub fn new_js_doc_non_nullable_type<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        Self::JSDocNonNullableType(JSDocNonNullableType::boxed(
            span,
            type_annotation,
            postfix,
            builder,
        ))
    }

    /// Build a [`TSTupleElement::JSDocUnknownType`].
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_js_doc_unknown_type<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::JSDocUnknownType(JSDocUnknownType::boxed(span, builder))
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSAnyKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSAnyKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSAnyKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSStringKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSStringKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSStringKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSBooleanKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSBooleanKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSBooleanKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSNumberKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNumberKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNumberKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSNeverKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNeverKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNeverKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSIntrinsicKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSIntrinsicKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIntrinsicKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSUnknownKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSUnknownKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUnknownKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSNullKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSNullKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNullKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSUndefinedKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSUndefinedKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSUndefinedKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSVoidKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSVoidKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSVoidKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSSymbolKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSSymbolKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSSymbolKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSThisType { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSThisType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSThisType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSObjectKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSObjectKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSObjectKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        TSBigIntKeyword { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`TSBigIntKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSBigIntKeyword::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
        TSTypeReference {
            node_id: Cell::new(builder.node_id()),
            span,
            type_name,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, type_name, type_arguments, builder), builder.builder())
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
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_qualified_name<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, builder))
    }

    /// Build a [`TSTypeName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSQualifiedName { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`TSQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSQualifiedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, params: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        let builder = builder.builder();
        TSTypeParameterInstantiation {
            node_id: Cell::new(builder.node_id()),
            span,
            params: params.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeParameterInstantiation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeParameterInstantiation::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        params: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, params, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        name: BindingIdentifier<'a>,
        constraint: Option<TSType<'a>>,
        default: Option<TSType<'a>>,
        r#in: bool,
        out: bool,
        r#const: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new(span, name, constraint, default, r#in, out, r#const, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, params: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTypeParameter<'a>>>,
    {
        let builder = builder.builder();
        TSTypeParameterDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            params: params.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeParameterDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeParameterDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        params: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSTypeParameter<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, params, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAliasDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, id, type_parameters, type_annotation, declare, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                id,
                type_parameters,
                type_annotation,
                declare,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: TSTypeName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        let builder = builder.builder();
        TSInterfaceDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            extends: extends.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSInterfaceDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, id, type_parameters, extends, body, declare, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        let builder = builder.builder();
        TSInterfaceDeclaration {
            node_id: Cell::new(builder.node_id()),
            span,
            id,
            type_parameters: type_parameters.into_in(builder.allocator()),
            extends: extends.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSInterfaceDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: T2,
        body: T3,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSInterfaceHeritage<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                id,
                type_parameters,
                extends,
                body,
                declare,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        let builder = builder.builder();
        TSInterfaceBody {
            node_id: Cell::new(builder.node_id()),
            span,
            body: body.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSInterfaceBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInterfaceBody::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(span: Span, body: T1, builder: &B) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, body, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, computed, optional, readonly, key, type_annotation, builder),
            builder.builder(),
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
    pub fn new_ts_index_signature<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        parameters: T1,
        type_annotation: T2,
        readonly: bool,
        r#static: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSIndexSignatureName<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        Self::TSIndexSignature(TSIndexSignature::boxed(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
            builder,
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
    pub fn new_ts_property_signature<B: GetAstBuilder<'a>, T1>(
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSPropertySignature(TSPropertySignature::boxed(
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
            builder,
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
    pub fn new_ts_call_signature_declaration<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSCallSignatureDeclaration(TSCallSignatureDeclaration::boxed(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            builder,
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
    pub fn new_ts_call_signature_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSCallSignatureDeclaration(TSCallSignatureDeclaration::boxed_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_construct_signature_declaration<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSConstructSignatureDeclaration(TSConstructSignatureDeclaration::boxed(
            span,
            type_parameters,
            params,
            return_type,
            builder,
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
    pub fn new_ts_construct_signature_declaration_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        Self::TSConstructSignatureDeclaration(TSConstructSignatureDeclaration::boxed_with_scope_id(
            span,
            type_parameters,
            params,
            return_type,
            scope_id,
            builder,
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
    pub fn new_ts_method_signature<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
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
            builder,
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
    pub fn new_ts_method_signature_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
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
            builder,
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        parameters: T1,
        type_annotation: T2,
        readonly: bool,
        r#static: bool,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSIndexSignatureName<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
        TSIndexSignature {
            node_id: Cell::new(builder.node_id()),
            span,
            parameters: parameters.into_in(builder.allocator()),
            type_annotation: type_annotation.into_in(builder.allocator()),
            readonly,
            r#static,
        }
    }

    /// Build a [`TSIndexSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSIndexSignature::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        parameters: T1,
        type_annotation: T2,
        readonly: bool,
        r#static: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSIndexSignatureName<'a>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, parameters, type_annotation, readonly, r#static, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSCallSignatureDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, type_parameters, this_param, params, return_type, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        key: PropertyKey<'a>,
        computed: bool,
        optional: bool,
        kind: TSMethodSignatureKind,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
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
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructSignatureDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, type_parameters, params, return_type, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructSignatureDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, type_parameters, params, return_type, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, S1, T1>(
        span: Span,
        name: S1,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypePredicate::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, parameter_name, asserts, type_annotation, builder),
            builder.builder(),
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, builder))
    }

    /// Build a [`TSTypePredicateName::This`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::This(TSThisType::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSModuleDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, id, body, kind, declare, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, id, body, kind, declare, scope_id, builder),
            builder.builder(),
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(BindingIdentifier::new(span, name, builder))
    }

    /// Build a [`TSModuleDeclarationName::Identifier`] with `symbol_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[inline]
    pub fn new_identifier_with_symbol_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        symbol_id: SymbolId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(BindingIdentifier::new_with_symbol_id(span, name, symbol_id, builder))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[inline]
    pub fn new_string_literal<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new(span, value, raw, builder))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[inline]
    pub fn new_string_literal_with_lone_surrogates<B: GetAstBuilder<'a>, S1>(
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
        builder: &B,
    ) -> Self
    where
        S1: Into<Str<'a>>,
    {
        Self::StringLiteral(StringLiteral::new_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
            builder,
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
    pub fn new_ts_module_declaration<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed(
            span, id, body, kind, declare, builder,
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
    pub fn new_ts_module_declaration_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        Self::TSModuleDeclaration(TSModuleDeclaration::boxed_with_scope_id(
            span, id, body, kind, declare, scope_id, builder,
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
    pub fn new_ts_module_block<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        directives: T1,
        body: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        Self::TSModuleBlock(TSModuleBlock::boxed(span, directives, body, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSGlobalDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, global_span, body, declare, builder), builder.builder())
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSGlobalDeclaration::new_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[inline]
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(span, global_span, body, declare, scope_id, builder),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        directives: T1,
        body: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        let builder = builder.builder();
        TSModuleBlock {
            node_id: Cell::new(builder.node_id()),
            span,
            directives: directives.into_in(builder.allocator()),
            body: body.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSModuleBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSModuleBlock::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        directives: T1,
        body: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, Directive<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, Statement<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, directives, body, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, members: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        let builder = builder.builder();
        TSTypeLiteral {
            node_id: Cell::new(builder.node_id()),
            span,
            members: members.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeLiteral::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        members: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TSSignature<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, members, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(span: Span, type_parameter: T1, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        let builder = builder.builder();
        TSInferType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_parameter: type_parameter.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSInferType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInferType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        type_parameter: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, type_parameter, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
        TSTypeQuery {
            node_id: Cell::new(builder.node_id()),
            span,
            expr_name,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTypeQuery`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeQuery::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(Self::new(span, expr_name, type_arguments, builder), builder.builder())
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
    pub fn new_ts_import_type<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Self::TSImportType(TSImportType::boxed(
            span,
            source,
            options,
            qualifier,
            type_arguments,
            builder,
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
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_qualified_name<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, builder))
    }

    /// Build a [`TSTypeQueryExprName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_this_expression<B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        Self::ThisExpression(ThisExpression::boxed(span, builder))
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(
            Self::new(span, source, options, qualifier, type_arguments, builder),
            builder.builder(),
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
    pub fn new_identifier<B: GetAstBuilder<'a>, S1>(span: Span, name: S1, builder: &B) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::Identifier(IdentifierName::boxed(span, name, builder))
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
    pub fn new_qualified_name<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::QualifiedName(TSImportTypeQualifiedName::boxed(span, left, right, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSImportTypeQualifiedName { node_id: Cell::new(builder.node_id()), span, left, right }
    }

    /// Build a [`TSImportTypeQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportTypeQualifiedName::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, left, right, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSFunctionType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, type_parameters, this_param, params, return_type, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3, T4>(
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSConstructorType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            Self::new(span, r#abstract, type_parameters, params, return_type, builder),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>, T1, T2, T3>(
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                r#abstract,
                type_parameters,
                params,
                return_type,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
                builder,
            ),
            builder.builder(),
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
    pub fn new_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
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
    pub fn boxed_with_scope_id<B: GetAstBuilder<'a>>(
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new_with_scope_id(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
                scope_id,
                builder,
            ),
            builder.builder(),
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
    pub fn new<B: GetAstBuilder<'a>, T1, T2>(span: Span, quasis: T1, types: T2, builder: &B) -> Self
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        let builder = builder.builder();
        TSTemplateLiteralType {
            node_id: Cell::new(builder.node_id()),
            span,
            quasis: quasis.into_in(builder.allocator()),
            types: types.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSTemplateLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTemplateLiteralType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1, T2>(
        span: Span,
        quasis: T1,
        types: T2,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaVec<'a, TemplateElement<'a>>>,
        T2: IntoIn<'a, ArenaVec<'a, TSType<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, quasis, types, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSAsExpression { node_id: Cell::new(builder.node_id()), span, expression, type_annotation }
    }

    /// Build a [`TSAsExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSAsExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSSatisfiesExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_annotation,
        }
    }

    /// Build a [`TSSatisfiesExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSSatisfiesExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, type_annotation, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSTypeAssertion { node_id: Cell::new(builder.node_id()), span, type_annotation, expression }
    }

    /// Build a [`TSTypeAssertion`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSTypeAssertion::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, expression, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
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
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSImportEqualsDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(
            Self::new(span, id, module_reference, import_kind, builder),
            builder.builder(),
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
    pub fn new_external_module_reference<B: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        builder: &B,
    ) -> Self {
        Self::ExternalModuleReference(TSExternalModuleReference::boxed(span, expression, builder))
    }

    /// Build a [`TSModuleReference::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[inline]
    pub fn new_identifier_reference<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed(span, name, builder))
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
    pub fn new_identifier_reference_with_reference_id<B: GetAstBuilder<'a>, S1>(
        span: Span,
        name: S1,
        reference_id: ReferenceId,
        builder: &B,
    ) -> Self
    where
        S1: Into<Ident<'a>>,
    {
        Self::IdentifierReference(IdentifierReference::boxed_with_reference_id(
            span,
            name,
            reference_id,
            builder,
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
    pub fn new_qualified_name<B: GetAstBuilder<'a>>(
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
        builder: &B,
    ) -> Self {
        Self::QualifiedName(TSQualifiedName::boxed(span, left, right, builder))
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        TSExternalModuleReference { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSExternalModuleReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSExternalModuleReference::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: StringLiteral<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSNonNullExpression { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSNonNullExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNonNullExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
    }
}

impl<'a> Decorator<'a> {
    /// Build a [`Decorator`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, expression: Expression<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSExportAssignment { node_id: Cell::new(builder.node_id()), span, expression }
    }

    /// Build a [`TSExportAssignment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSExportAssignment::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        expression: Expression<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, expression, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(span: Span, id: IdentifierName<'a>, builder: &B) -> Self {
        let builder = builder.builder();
        TSNamespaceExportDeclaration { node_id: Cell::new(builder.node_id()), span, id }
    }

    /// Build a [`TSNamespaceExportDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSNamespaceExportDeclaration::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        id: IdentifierName<'a>,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, id, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> Self
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        let builder = builder.builder();
        TSInstantiationExpression {
            node_id: Cell::new(builder.node_id()),
            span,
            expression,
            type_arguments: type_arguments.into_in(builder.allocator()),
        }
    }

    /// Build a [`TSInstantiationExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`TSInstantiationExpression::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>, T1>(
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
        builder: &B,
    ) -> ArenaBox<'a, Self>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        ArenaBox::new_in(Self::new(span, expression, type_arguments, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSDocNullableType { node_id: Cell::new(builder.node_id()), span, type_annotation, postfix }
    }

    /// Build a [`JSDocNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocNullableType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, postfix, builder), builder.builder())
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
    pub fn new<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> Self {
        let builder = builder.builder();
        JSDocNonNullableType {
            node_id: Cell::new(builder.node_id()),
            span,
            type_annotation,
            postfix,
        }
    }

    /// Build a [`JSDocNonNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocNonNullableType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[inline]
    pub fn boxed<B: GetAstBuilder<'a>>(
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
        builder: &B,
    ) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, type_annotation, postfix, builder), builder.builder())
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
    pub fn new<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> Self {
        let builder = builder.builder();
        JSDocUnknownType { node_id: Cell::new(builder.node_id()), span }
    }

    /// Build a [`JSDocUnknownType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`JSDocUnknownType::new`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn boxed<'a, B: GetAstBuilder<'a>>(span: Span, builder: &B) -> ArenaBox<'a, Self> {
        ArenaBox::new_in(Self::new(span, builder), builder.builder())
    }
}
