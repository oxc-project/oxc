// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/ast_builder.rs`.

//! AST node factories

#![allow(unused_imports)]
#![expect(deprecated, clippy::default_trait_access, clippy::unused_self)]

use std::cell::Cell;

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, GetAllocator, IntoIn};
use oxc_str::{Ident, Str};
use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::{ast::*, builder::AstBuilder};

impl<'a> AstBuilder<'a> {
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn program(
        self,
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: ArenaVec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: ArenaVec<'a, Directive<'a>>,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> Program<'a> {
        Program {
            node_id: Default::default(),
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn program_with_scope_id(
        self,
        span: Span,
        source_type: SourceType,
        source_text: &'a str,
        comments: ArenaVec<'a, Comment>,
        hashbang: Option<Hashbang<'a>>,
        directives: ArenaVec<'a, Directive<'a>>,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Program<'a> {
        Program {
            node_id: Default::default(),
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

    /// Build an [`Expression::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_boolean_literal(self, span: Span, value: bool) -> Expression<'a> {
        Expression::BooleanLiteral(self.alloc_boolean_literal(span, value))
    }

    /// Build an [`Expression::NullLiteral`].
    ///
    /// This node contains a [`NullLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_null_literal(self, span: Span) -> Expression<'a> {
        Expression::NullLiteral(self.alloc_null_literal(span))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> Expression<'a> {
        Expression::NumericLiteral(self.alloc_numeric_literal(span, value, raw, base))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_big_int_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
    ) -> Expression<'a>
    where
        S1: Into<Str<'a>>,
    {
        Expression::BigIntLiteral(self.alloc_big_int_literal(span, value, raw, base))
    }

    /// Build an [`Expression::RegExpLiteral`].
    ///
    /// This node contains a [`RegExpLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_reg_exp_literal(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
    ) -> Expression<'a> {
        Expression::RegExpLiteral(self.alloc_reg_exp_literal(span, regex, raw))
    }

    /// Build an [`Expression::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> Expression<'a>
    where
        S1: Into<Str<'a>>,
    {
        Expression::StringLiteral(self.alloc_string_literal(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> Expression<'a>
    where
        S1: Into<Str<'a>>,
    {
        Expression::StringLiteral(self.alloc_string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_template_literal(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::TemplateLiteral(self.alloc_template_literal(span, quasis, expressions))
    }

    /// Build an [`Expression::Identifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_identifier<S1>(self, span: Span, name: S1) -> Expression<'a>
    where
        S1: Into<Ident<'a>>,
    {
        Expression::Identifier(self.alloc_identifier_reference(span, name))
    }

    /// Build an [`Expression::Identifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_identifier_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> Expression<'a>
    where
        S1: Into<Ident<'a>>,
    {
        Expression::Identifier(self.alloc_identifier_reference_with_reference_id(
            span,
            name,
            reference_id,
        ))
    }

    /// Build an [`Expression::Super`].
    ///
    /// This node contains a [`Super`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_super(self, span: Span) -> Expression<'a> {
        Expression::Super(self.alloc_super(span))
    }

    /// Build an [`Expression::ArrayExpression`].
    ///
    /// This node contains an [`ArrayExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_array(
        self,
        span: Span,
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) -> Expression<'a> {
        Expression::ArrayExpression(self.alloc_array_expression(span, elements))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Expression::ArrowFunctionExpression(self.alloc_arrow_function_expression(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_arrow_function_with_scope_id_and_pure_and_pife<T1, T2, T3, T4>(
        self,
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
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        Expression::ArrowFunctionExpression(
            self.alloc_arrow_function_expression_with_scope_id_and_pure_and_pife(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_assignment(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::AssignmentExpression(
            self.alloc_assignment_expression(span, operator, left, right),
        )
    }

    /// Build an [`Expression::AwaitExpression`].
    ///
    /// This node contains an [`AwaitExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_await(self, span: Span, argument: Expression<'a>) -> Expression<'a> {
        Expression::AwaitExpression(self.alloc_await_expression(span, argument))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_binary(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::BinaryExpression(self.alloc_binary_expression(span, left, operator, right))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_call<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::CallExpression(self.alloc_call_expression(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_call_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::CallExpression(self.alloc_call_expression_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
        ))
    }

    /// Build an [`Expression::ChainExpression`].
    ///
    /// This node contains a [`ChainExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_chain(self, span: Span, expression: ChainElement<'a>) -> Expression<'a> {
        Expression::ChainExpression(self.alloc_chain_expression(span, expression))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_class<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Expression::ClassExpression(self.alloc_class(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_class_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Expression::ClassExpression(self.alloc_class_with_scope_id(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_conditional(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ConditionalExpression(
            self.alloc_conditional_expression(span, test, consequent, alternate),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_function<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Expression::FunctionExpression(self.alloc_function(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_function_with_scope_id_and_pure_and_pife<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Expression::FunctionExpression(self.alloc_function_with_scope_id_and_pure_and_pife(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_import(
        self,
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
    ) -> Expression<'a> {
        Expression::ImportExpression(self.alloc_import_expression(span, source, options, phase))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_logical(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::LogicalExpression(self.alloc_logical_expression(span, left, operator, right))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_new<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::NewExpression(self.alloc_new_expression(
            span,
            callee,
            type_arguments,
            arguments,
        ))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_new_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        pure: bool,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::NewExpression(self.alloc_new_expression_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            pure,
        ))
    }

    /// Build an [`Expression::ObjectExpression`].
    ///
    /// This node contains an [`ObjectExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_object(
        self,
        span: Span,
        properties: ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) -> Expression<'a> {
        Expression::ObjectExpression(self.alloc_object_expression(span, properties))
    }

    /// Build an [`Expression::ParenthesizedExpression`].
    ///
    /// This node contains a [`ParenthesizedExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_parenthesized(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::ParenthesizedExpression(self.alloc_parenthesized_expression(span, expression))
    }

    /// Build an [`Expression::SequenceExpression`].
    ///
    /// This node contains a [`SequenceExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_sequence(
        self,
        span: Span,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> Expression<'a> {
        Expression::SequenceExpression(self.alloc_sequence_expression(span, expressions))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_tagged_template<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        Expression::TaggedTemplateExpression(self.alloc_tagged_template_expression(
            span,
            tag,
            type_arguments,
            quasi,
        ))
    }

    /// Build an [`Expression::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_this(self, span: Span) -> Expression<'a> {
        Expression::ThisExpression(self.alloc_this_expression(span))
    }

    /// Build an [`Expression::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_unary(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> Expression<'a> {
        Expression::UnaryExpression(self.alloc_unary_expression(span, operator, argument))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_update(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> Expression<'a> {
        Expression::UpdateExpression(self.alloc_update_expression(span, operator, prefix, argument))
    }

    /// Build an [`Expression::YieldExpression`].
    ///
    /// This node contains a [`YieldExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_yield(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> Expression<'a> {
        Expression::YieldExpression(self.alloc_yield_expression(span, delegate, argument))
    }

    /// Build an [`Expression::PrivateInExpression`].
    ///
    /// This node contains a [`PrivateInExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_private_in(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
    ) -> Expression<'a> {
        Expression::PrivateInExpression(self.alloc_private_in_expression(span, left, right))
    }

    /// Build an [`Expression::ImportMeta`].
    ///
    /// This node contains an [`ImportMeta`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_import_meta(self, span: Span) -> Expression<'a> {
        Expression::ImportMeta(self.alloc_import_meta(span))
    }

    /// Build an [`Expression::NewTarget`].
    ///
    /// This node contains a [`NewTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_new_target(self, span: Span) -> Expression<'a> {
        Expression::NewTarget(self.alloc_new_target(span))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        Expression::JSXElement(self.alloc_jsx_element(
            span,
            opening_element,
            children,
            closing_element,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> Expression<'a> {
        Expression::JSXFragment(self.alloc_jsx_fragment(
            span,
            opening_fragment,
            children,
            closing_fragment,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_ts_as(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSAsExpression(self.alloc_ts_as_expression(span, expression, type_annotation))
    }

    /// Build an [`Expression::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_ts_satisfies(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> Expression<'a> {
        Expression::TSSatisfiesExpression(self.alloc_ts_satisfies_expression(
            span,
            expression,
            type_annotation,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_ts_type_assertion(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> Expression<'a> {
        Expression::TSTypeAssertion(self.alloc_ts_type_assertion(span, type_annotation, expression))
    }

    /// Build an [`Expression::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_ts_non_null(self, span: Span, expression: Expression<'a>) -> Expression<'a> {
        Expression::TSNonNullExpression(self.alloc_ts_non_null_expression(span, expression))
    }

    /// Build an [`Expression::TSInstantiationExpression`].
    ///
    /// This node contains a [`TSInstantiationExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_ts_instantiation<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        Expression::TSInstantiationExpression(self.alloc_ts_instantiation_expression(
            span,
            expression,
            type_arguments,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_v8_intrinsic(
        self,
        span: Span,
        name: IdentifierName<'a>,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> Expression<'a> {
        Expression::V8IntrinsicExpression(self.alloc_v8_intrinsic_expression(span, name, arguments))
    }

    /// Build an [`IdentifierName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_identifier_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn identifier_name<S1>(self, span: Span, name: S1) -> IdentifierName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        IdentifierName { node_id: Default::default(), span, name: name.into() }
    }

    /// Build an [`IdentifierName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::identifier_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_identifier_name<S1>(self, span: Span, name: S1) -> ArenaBox<'a, IdentifierName<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(self.identifier_name(span, name), &self)
    }

    /// Build an [`IdentifierReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_identifier_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn identifier_reference<S1>(self, span: Span, name: S1) -> IdentifierReference<'a>
    where
        S1: Into<Ident<'a>>,
    {
        IdentifierReference {
            node_id: Default::default(),
            span,
            name: name.into(),
            reference_id: Default::default(),
        }
    }

    /// Build an [`IdentifierReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::identifier_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_identifier_reference<S1>(
        self,
        span: Span,
        name: S1,
    ) -> ArenaBox<'a, IdentifierReference<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(self.identifier_reference(span, name), &self)
    }

    /// Build an [`IdentifierReference`] with `reference_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_identifier_reference_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> IdentifierReference<'a>
    where
        S1: Into<Ident<'a>>,
    {
        IdentifierReference {
            node_id: Default::default(),
            span,
            name: name.into(),
            reference_id: Cell::new(Some(reference_id)),
        }
    }

    /// Build an [`IdentifierReference`] with `reference_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::identifier_reference_with_reference_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> ArenaBox<'a, IdentifierReference<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(
            self.identifier_reference_with_reference_id(span, name, reference_id),
            &self,
        )
    }

    /// Build a [`BindingIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_binding_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_identifier<S1>(self, span: Span, name: S1) -> BindingIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingIdentifier {
            node_id: Default::default(),
            span,
            name: name.into(),
            symbol_id: Default::default(),
        }
    }

    /// Build a [`BindingIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::binding_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_binding_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> ArenaBox<'a, BindingIdentifier<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(self.binding_identifier(span, name), &self)
    }

    /// Build a [`BindingIdentifier`] with `symbol_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_binding_identifier_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_identifier_with_symbol_id<S1>(
        self,
        span: Span,
        name: S1,
        symbol_id: SymbolId,
    ) -> BindingIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingIdentifier {
            node_id: Default::default(),
            span,
            name: name.into(),
            symbol_id: Cell::new(Some(symbol_id)),
        }
    }

    /// Build a [`BindingIdentifier`] with `symbol_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::binding_identifier_with_symbol_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_binding_identifier_with_symbol_id<S1>(
        self,
        span: Span,
        name: S1,
        symbol_id: SymbolId,
    ) -> ArenaBox<'a, BindingIdentifier<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(self.binding_identifier_with_symbol_id(span, name, symbol_id), &self)
    }

    /// Build a [`LabelIdentifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn label_identifier<S1>(self, span: Span, name: S1) -> LabelIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        LabelIdentifier { node_id: Default::default(), span, name: name.into() }
    }

    /// Build a [`ThisExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_this_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn this_expression(self, span: Span) -> ThisExpression {
        ThisExpression { node_id: Default::default(), span }
    }

    /// Build a [`ThisExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::this_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_this_expression(self, span: Span) -> ArenaBox<'a, ThisExpression> {
        ArenaBox::new_in(self.this_expression(span), &self)
    }

    /// Build an [`ArrayExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_array_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn array_expression(
        self,
        span: Span,
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) -> ArrayExpression<'a> {
        ArrayExpression { node_id: Default::default(), span, elements }
    }

    /// Build an [`ArrayExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::array_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_array_expression(
        self,
        span: Span,
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
    ) -> ArenaBox<'a, ArrayExpression<'a>> {
        ArenaBox::new_in(self.array_expression(span, elements), &self)
    }

    /// Build an [`ArrayExpressionElement::SpreadElement`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn array_expression_element_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::SpreadElement(self.alloc_spread_element(span, argument))
    }

    /// Build an [`ArrayExpressionElement::Elision`].
    ///
    /// This node contains an [`Elision`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn array_expression_element_elision(self, span: Span) -> ArrayExpressionElement<'a> {
        ArrayExpressionElement::Elision(self.alloc_elision(span))
    }

    /// Build an [`Elision`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_elision`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn elision(self, span: Span) -> Elision {
        Elision { node_id: Default::default(), span }
    }

    /// Build an [`Elision`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::elision`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_elision(self, span: Span) -> ArenaBox<'a, Elision> {
        ArenaBox::new_in(self.elision(span), &self)
    }

    /// Build an [`ObjectExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_object_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_expression(
        self,
        span: Span,
        properties: ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) -> ObjectExpression<'a> {
        ObjectExpression { node_id: Default::default(), span, properties }
    }

    /// Build an [`ObjectExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::object_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`: Properties declared in the object
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_object_expression(
        self,
        span: Span,
        properties: ArenaVec<'a, ObjectPropertyKind<'a>>,
    ) -> ArenaBox<'a, ObjectExpression<'a>> {
        ArenaBox::new_in(self.object_expression(span, properties), &self)
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_property_kind_object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectPropertyKind<'a> {
        ObjectPropertyKind::ObjectProperty(
            self.alloc_object_property(span, kind, key, value, method, shorthand, computed),
        )
    }

    /// Build an [`ObjectPropertyKind::SpreadProperty`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_property_kind_spread_property(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ObjectPropertyKind<'a> {
        ObjectPropertyKind::SpreadProperty(self.alloc_spread_element(span, argument))
    }

    /// Build an [`ObjectProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_object_property`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `key`
    /// * `value`
    /// * `method`
    /// * `shorthand`
    /// * `computed`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'a> {
        ObjectProperty {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::object_property`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `key`
    /// * `value`
    /// * `method`
    /// * `shorthand`
    /// * `computed`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_object_property(
        self,
        span: Span,
        kind: PropertyKind,
        key: PropertyKey<'a>,
        value: Expression<'a>,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ArenaBox<'a, ObjectProperty<'a>> {
        ArenaBox::new_in(
            self.object_property(span, kind, key, value, method, shorthand, computed),
            &self,
        )
    }

    /// Build a [`PropertyKey::StaticIdentifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn property_key_static_identifier<S1>(self, span: Span, name: S1) -> PropertyKey<'a>
    where
        S1: Into<Ident<'a>>,
    {
        PropertyKey::StaticIdentifier(self.alloc_identifier_name(span, name))
    }

    /// Build a [`PropertyKey::PrivateIdentifier`].
    ///
    /// This node contains a [`PrivateIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn property_key_private_identifier<S1>(self, span: Span, name: S1) -> PropertyKey<'a>
    where
        S1: Into<Ident<'a>>,
    {
        PropertyKey::PrivateIdentifier(self.alloc_private_identifier(span, name))
    }

    /// Build a [`TemplateLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_template_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn template_literal(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> TemplateLiteral<'a> {
        TemplateLiteral { node_id: Default::default(), span, quasis, expressions }
    }

    /// Build a [`TemplateLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::template_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`
    /// * `expressions`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_template_literal(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> ArenaBox<'a, TemplateLiteral<'a>> {
        ArenaBox::new_in(self.template_literal(span, quasis, expressions), &self)
    }

    /// Build a [`TaggedTemplateExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_tagged_template_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn tagged_template_expression<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
    ) -> TaggedTemplateExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TaggedTemplateExpression {
            node_id: Default::default(),
            span,
            tag,
            type_arguments: type_arguments.into_in(self.allocator()),
            quasi,
        }
    }

    /// Build a [`TaggedTemplateExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::tagged_template_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `tag`
    /// * `type_arguments`
    /// * `quasi`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_tagged_template_expression<T1>(
        self,
        span: Span,
        tag: Expression<'a>,
        type_arguments: T1,
        quasi: TemplateLiteral<'a>,
    ) -> ArenaBox<'a, TaggedTemplateExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(self.tagged_template_expression(span, tag, type_arguments, quasi), &self)
    }

    /// Build a [`TemplateElement`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    /// * `tail`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn template_element(
        self,
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
    ) -> TemplateElement<'a> {
        TemplateElement {
            node_id: Default::default(),
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn template_element_with_lone_surrogates(
        self,
        span: Span,
        value: TemplateElementValue<'a>,
        tail: bool,
        lone_surrogates: bool,
    ) -> TemplateElement<'a> {
        TemplateElement { node_id: Default::default(), span, value, tail, lone_surrogates }
    }

    /// Build a [`MemberExpression::ComputedMemberExpression`].
    ///
    /// This node contains a [`ComputedMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn member_expression_computed(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::ComputedMemberExpression(
            self.alloc_computed_member_expression(span, object, expression, optional),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn member_expression_static(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::StaticMemberExpression(
            self.alloc_static_member_expression(span, object, property, optional),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn member_expression_private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> MemberExpression<'a> {
        MemberExpression::PrivateFieldExpression(
            self.alloc_private_field_expression(span, object, field, optional),
        )
    }

    /// Build a [`ComputedMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_computed_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ComputedMemberExpression<'a> {
        ComputedMemberExpression { node_id: Default::default(), span, object, expression, optional }
    }

    /// Build a [`ComputedMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::computed_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `expression`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_computed_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        expression: Expression<'a>,
        optional: bool,
    ) -> ArenaBox<'a, ComputedMemberExpression<'a>> {
        ArenaBox::new_in(self.computed_member_expression(span, object, expression, optional), &self)
    }

    /// Build a [`StaticMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_static_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> StaticMemberExpression<'a> {
        StaticMemberExpression { node_id: Default::default(), span, object, property, optional }
    }

    /// Build a [`StaticMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::static_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `property`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_static_member_expression(
        self,
        span: Span,
        object: Expression<'a>,
        property: IdentifierName<'a>,
        optional: bool,
    ) -> ArenaBox<'a, StaticMemberExpression<'a>> {
        ArenaBox::new_in(self.static_member_expression(span, object, property, optional), &self)
    }

    /// Build a [`PrivateFieldExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_private_field_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> PrivateFieldExpression<'a> {
        PrivateFieldExpression { node_id: Default::default(), span, object, field, optional }
    }

    /// Build a [`PrivateFieldExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::private_field_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `field`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_private_field_expression(
        self,
        span: Span,
        object: Expression<'a>,
        field: PrivateIdentifier<'a>,
        optional: bool,
    ) -> ArenaBox<'a, PrivateFieldExpression<'a>> {
        ArenaBox::new_in(self.private_field_expression(span, object, field, optional), &self)
    }

    /// Build a [`CallExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_call_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> CallExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        CallExpression {
            node_id: Default::default(),
            span,
            callee,
            type_arguments: type_arguments.into_in(self.allocator()),
            arguments,
            optional,
            pure: Default::default(),
        }
    }

    /// Build a [`CallExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::call_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> ArenaBox<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(
            self.call_expression(span, callee, type_arguments, arguments, optional),
            &self,
        )
    }

    /// Build a [`CallExpression`] with `pure`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_call_expression_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn call_expression_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
    ) -> CallExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        CallExpression {
            node_id: Default::default(),
            span,
            callee,
            type_arguments: type_arguments.into_in(self.allocator()),
            arguments,
            optional,
            pure,
        }
    }

    /// Build a [`CallExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::call_expression_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`
    /// * `optional`
    /// * `pure`: `true` if the call expression is marked with a `/* @__PURE__ */` comment
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_call_expression_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
    ) -> ArenaBox<'a, CallExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(
            self.call_expression_with_pure(span, callee, type_arguments, arguments, optional, pure),
            &self,
        )
    }

    /// Build a [`NewExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_new_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn new_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> NewExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        NewExpression {
            node_id: Default::default(),
            span,
            callee,
            type_arguments: type_arguments.into_in(self.allocator()),
            arguments,
            pure: Default::default(),
        }
    }

    /// Build a [`NewExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::new_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_new_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> ArenaBox<'a, NewExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(self.new_expression(span, callee, type_arguments, arguments), &self)
    }

    /// Build a [`NewExpression`] with `pure`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_new_expression_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn new_expression_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        pure: bool,
    ) -> NewExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        NewExpression {
            node_id: Default::default(),
            span,
            callee,
            type_arguments: type_arguments.into_in(self.allocator()),
            arguments,
            pure,
        }
    }

    /// Build a [`NewExpression`] with `pure`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::new_expression_with_pure`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `callee`
    /// * `type_arguments`
    /// * `arguments`: `true` if the new expression is marked with a `/* @__PURE__ */` comment
    /// * `pure`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_new_expression_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        pure: bool,
    ) -> ArenaBox<'a, NewExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(
            self.new_expression_with_pure(span, callee, type_arguments, arguments, pure),
            &self,
        )
    }

    /// Build an [`ImportMeta`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_meta`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_meta(self, span: Span) -> ImportMeta {
        ImportMeta { node_id: Default::default(), span }
    }

    /// Build an [`ImportMeta`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_meta`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_meta(self, span: Span) -> ArenaBox<'a, ImportMeta> {
        ArenaBox::new_in(self.import_meta(span), &self)
    }

    /// Build a [`NewTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_new_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn new_target(self, span: Span) -> NewTarget {
        NewTarget { node_id: Default::default(), span }
    }

    /// Build a [`NewTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::new_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_new_target(self, span: Span) -> ArenaBox<'a, NewTarget> {
        ArenaBox::new_in(self.new_target(span), &self)
    }

    /// Build a [`SpreadElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_spread_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn spread_element(self, span: Span, argument: Expression<'a>) -> SpreadElement<'a> {
        SpreadElement { node_id: Default::default(), span, argument }
    }

    /// Build a [`SpreadElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::spread_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_spread_element(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, SpreadElement<'a>> {
        ArenaBox::new_in(self.spread_element(span, argument), &self)
    }

    /// Build an [`Argument::SpreadElement`].
    ///
    /// This node contains a [`SpreadElement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn argument_spread_element(self, span: Span, argument: Expression<'a>) -> Argument<'a> {
        Argument::SpreadElement(self.alloc_spread_element(span, argument))
    }

    /// Build an [`UpdateExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_update_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> UpdateExpression<'a> {
        UpdateExpression { node_id: Default::default(), span, operator, prefix, argument }
    }

    /// Build an [`UpdateExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::update_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `prefix`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_update_expression(
        self,
        span: Span,
        operator: UpdateOperator,
        prefix: bool,
        argument: SimpleAssignmentTarget<'a>,
    ) -> ArenaBox<'a, UpdateExpression<'a>> {
        ArenaBox::new_in(self.update_expression(span, operator, prefix, argument), &self)
    }

    /// Build an [`UnaryExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_unary_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> UnaryExpression<'a> {
        UnaryExpression { node_id: Default::default(), span, operator, argument }
    }

    /// Build an [`UnaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::unary_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, UnaryExpression<'a>> {
        ArenaBox::new_in(self.unary_expression(span, operator, argument), &self)
    }

    /// Build a [`BinaryExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_binary_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> BinaryExpression<'a> {
        BinaryExpression { node_id: Default::default(), span, left, operator, right }
    }

    /// Build a [`BinaryExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::binary_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_binary_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: BinaryOperator,
        right: Expression<'a>,
    ) -> ArenaBox<'a, BinaryExpression<'a>> {
        ArenaBox::new_in(self.binary_expression(span, left, operator, right), &self)
    }

    /// Build a [`PrivateInExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_private_in_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
    ) -> PrivateInExpression<'a> {
        PrivateInExpression { node_id: Default::default(), span, left, right }
    }

    /// Build a [`PrivateInExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::private_in_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_private_in_expression(
        self,
        span: Span,
        left: PrivateIdentifier<'a>,
        right: Expression<'a>,
    ) -> ArenaBox<'a, PrivateInExpression<'a>> {
        ArenaBox::new_in(self.private_in_expression(span, left, right), &self)
    }

    /// Build a [`LogicalExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_logical_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> LogicalExpression<'a> {
        LogicalExpression { node_id: Default::default(), span, left, operator, right }
    }

    /// Build a [`LogicalExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::logical_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `operator`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_logical_expression(
        self,
        span: Span,
        left: Expression<'a>,
        operator: LogicalOperator,
        right: Expression<'a>,
    ) -> ArenaBox<'a, LogicalExpression<'a>> {
        ArenaBox::new_in(self.logical_expression(span, left, operator, right), &self)
    }

    /// Build a [`ConditionalExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_conditional_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> ConditionalExpression<'a> {
        ConditionalExpression { node_id: Default::default(), span, test, consequent, alternate }
    }

    /// Build a [`ConditionalExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::conditional_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_conditional_expression(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Expression<'a>,
        alternate: Expression<'a>,
    ) -> ArenaBox<'a, ConditionalExpression<'a>> {
        ArenaBox::new_in(self.conditional_expression(span, test, consequent, alternate), &self)
    }

    /// Build an [`AssignmentExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> AssignmentExpression<'a> {
        AssignmentExpression { node_id: Default::default(), span, operator, left, right }
    }

    /// Build an [`AssignmentExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_expression(
        self,
        span: Span,
        operator: AssignmentOperator,
        left: AssignmentTarget<'a>,
        right: Expression<'a>,
    ) -> ArenaBox<'a, AssignmentExpression<'a>> {
        ArenaBox::new_in(self.assignment_expression(span, operator, left, right), &self)
    }

    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_assignment_target_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> SimpleAssignmentTarget<'a>
    where
        S1: Into<Ident<'a>>,
    {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(
            self.alloc_identifier_reference(span, name),
        )
    }

    /// Build a [`SimpleAssignmentTarget::AssignmentTargetIdentifier`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_assignment_target_identifier_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> SimpleAssignmentTarget<'a>
    where
        S1: Into<Ident<'a>>,
    {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(
            self.alloc_identifier_reference_with_reference_id(span, name, reference_id),
        )
    }

    /// Build a [`SimpleAssignmentTarget::TSAsExpression`].
    ///
    /// This node contains a [`TSAsExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSAsExpression(self.alloc_ts_as_expression(
            span,
            expression,
            type_annotation,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::TSSatisfiesExpression`].
    ///
    /// This node contains a [`TSSatisfiesExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSSatisfiesExpression(self.alloc_ts_satisfies_expression(
            span,
            expression,
            type_annotation,
        ))
    }

    /// Build a [`SimpleAssignmentTarget::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSNonNullExpression(
            self.alloc_ts_non_null_expression(span, expression),
        )
    }

    /// Build a [`SimpleAssignmentTarget::TSTypeAssertion`].
    ///
    /// This node contains a [`TSTypeAssertion`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn simple_assignment_target_ts_type_assertion(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> SimpleAssignmentTarget<'a> {
        SimpleAssignmentTarget::TSTypeAssertion(self.alloc_ts_type_assertion(
            span,
            type_annotation,
            expression,
        ))
    }

    /// Build an [`AssignmentTargetPattern::ArrayAssignmentTarget`].
    ///
    /// This node contains an [`ArrayAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_pattern_array_assignment_target<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
    ) -> AssignmentTargetPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        AssignmentTargetPattern::ArrayAssignmentTarget(
            self.alloc_array_assignment_target(span, elements, rest),
        )
    }

    /// Build an [`AssignmentTargetPattern::ObjectAssignmentTarget`].
    ///
    /// This node contains an [`ObjectAssignmentTarget`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_pattern_object_assignment_target<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
    ) -> AssignmentTargetPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        AssignmentTargetPattern::ObjectAssignmentTarget(
            self.alloc_object_assignment_target(span, properties, rest),
        )
    }

    /// Build an [`ArrayAssignmentTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_array_assignment_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn array_assignment_target<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
    ) -> ArrayAssignmentTarget<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ArrayAssignmentTarget {
            node_id: Default::default(),
            span,
            elements,
            rest: rest.into_in(self.allocator()),
        }
    }

    /// Build an [`ArrayAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::array_assignment_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_array_assignment_target<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<AssignmentTargetMaybeDefault<'a>>>,
        rest: T1,
    ) -> ArenaBox<'a, ArrayAssignmentTarget<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ArenaBox::new_in(self.array_assignment_target(span, elements, rest), &self)
    }

    /// Build an [`ObjectAssignmentTarget`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_object_assignment_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_assignment_target<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
    ) -> ObjectAssignmentTarget<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ObjectAssignmentTarget {
            node_id: Default::default(),
            span,
            properties,
            rest: rest.into_in(self.allocator()),
        }
    }

    /// Build an [`ObjectAssignmentTarget`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::object_assignment_target`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_object_assignment_target<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, AssignmentTargetProperty<'a>>,
        rest: T1,
    ) -> ArenaBox<'a, ObjectAssignmentTarget<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, AssignmentTargetRest<'a>>>>,
    {
        ArenaBox::new_in(self.object_assignment_target(span, properties, rest), &self)
    }

    /// Build an [`AssignmentTargetRest`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_target_rest`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `target`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> AssignmentTargetRest<'a> {
        AssignmentTargetRest { node_id: Default::default(), span, target }
    }

    /// Build an [`AssignmentTargetRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_target_rest`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `target`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_target_rest(
        self,
        span: Span,
        target: AssignmentTarget<'a>,
    ) -> ArenaBox<'a, AssignmentTargetRest<'a>> {
        ArenaBox::new_in(self.assignment_target_rest(span, target), &self)
    }

    /// Build an [`AssignmentTargetMaybeDefault::AssignmentTargetWithDefault`].
    ///
    /// This node contains an [`AssignmentTargetWithDefault`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_maybe_default_assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetMaybeDefault<'a> {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
            self.alloc_assignment_target_with_default(span, binding, init),
        )
    }

    /// Build an [`AssignmentTargetWithDefault`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_target_with_default`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> AssignmentTargetWithDefault<'a> {
        AssignmentTargetWithDefault { node_id: Default::default(), span, binding, init }
    }

    /// Build an [`AssignmentTargetWithDefault`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_target_with_default`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_target_with_default(
        self,
        span: Span,
        binding: AssignmentTarget<'a>,
        init: Expression<'a>,
    ) -> ArenaBox<'a, AssignmentTargetWithDefault<'a>> {
        ArenaBox::new_in(self.assignment_target_with_default(span, binding, init), &self)
    }

    /// Build an [`AssignmentTargetProperty::AssignmentTargetPropertyIdentifier`].
    ///
    /// This node contains an [`AssignmentTargetPropertyIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_property_assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetProperty<'a> {
        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
            self.alloc_assignment_target_property_identifier(span, binding, init),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_property_assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
    ) -> AssignmentTargetProperty<'a> {
        AssignmentTargetProperty::AssignmentTargetPropertyProperty(
            self.alloc_assignment_target_property_property(span, name, binding, computed),
        )
    }

    /// Build an [`AssignmentTargetPropertyIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_target_property_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> AssignmentTargetPropertyIdentifier<'a> {
        AssignmentTargetPropertyIdentifier { node_id: Default::default(), span, binding, init }
    }

    /// Build an [`AssignmentTargetPropertyIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_target_property_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `binding`
    /// * `init`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_target_property_identifier(
        self,
        span: Span,
        binding: IdentifierReference<'a>,
        init: Option<Expression<'a>>,
    ) -> ArenaBox<'a, AssignmentTargetPropertyIdentifier<'a>> {
        ArenaBox::new_in(self.assignment_target_property_identifier(span, binding, init), &self)
    }

    /// Build an [`AssignmentTargetPropertyProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_target_property_property`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
    ) -> AssignmentTargetPropertyProperty<'a> {
        AssignmentTargetPropertyProperty {
            node_id: Default::default(),
            span,
            name,
            binding,
            computed,
        }
    }

    /// Build an [`AssignmentTargetPropertyProperty`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_target_property_property`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The property key
    /// * `binding`: The binding part of the property
    /// * `computed`: Property was declared with a computed key
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_target_property_property(
        self,
        span: Span,
        name: PropertyKey<'a>,
        binding: AssignmentTargetMaybeDefault<'a>,
        computed: bool,
    ) -> ArenaBox<'a, AssignmentTargetPropertyProperty<'a>> {
        ArenaBox::new_in(
            self.assignment_target_property_property(span, name, binding, computed),
            &self,
        )
    }

    /// Build a [`SequenceExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_sequence_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn sequence_expression(
        self,
        span: Span,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> SequenceExpression<'a> {
        SequenceExpression { node_id: Default::default(), span, expressions }
    }

    /// Build a [`SequenceExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::sequence_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expressions`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_sequence_expression(
        self,
        span: Span,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> ArenaBox<'a, SequenceExpression<'a>> {
        ArenaBox::new_in(self.sequence_expression(span, expressions), &self)
    }

    /// Build a [`Super`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_super`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn super_(self, span: Span) -> Super {
        Super { node_id: Default::default(), span }
    }

    /// Build a [`Super`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::super_`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_super(self, span: Span) -> ArenaBox<'a, Super> {
        ArenaBox::new_in(self.super_(span), &self)
    }

    /// Build an [`AwaitExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_await_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn await_expression(self, span: Span, argument: Expression<'a>) -> AwaitExpression<'a> {
        AwaitExpression { node_id: Default::default(), span, argument }
    }

    /// Build an [`AwaitExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::await_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_await_expression(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, AwaitExpression<'a>> {
        ArenaBox::new_in(self.await_expression(span, argument), &self)
    }

    /// Build a [`ChainExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_chain_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn chain_expression(self, span: Span, expression: ChainElement<'a>) -> ChainExpression<'a> {
        ChainExpression { node_id: Default::default(), span, expression }
    }

    /// Build a [`ChainExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::chain_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_chain_expression(
        self,
        span: Span,
        expression: ChainElement<'a>,
    ) -> ArenaBox<'a, ChainExpression<'a>> {
        ArenaBox::new_in(self.chain_expression(span, expression), &self)
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn chain_element_call_expression<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
    ) -> ChainElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ChainElement::CallExpression(self.alloc_call_expression(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn chain_element_call_expression_with_pure<T1>(
        self,
        span: Span,
        callee: Expression<'a>,
        type_arguments: T1,
        arguments: ArenaVec<'a, Argument<'a>>,
        optional: bool,
        pure: bool,
    ) -> ChainElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ChainElement::CallExpression(self.alloc_call_expression_with_pure(
            span,
            callee,
            type_arguments,
            arguments,
            optional,
            pure,
        ))
    }

    /// Build a [`ChainElement::TSNonNullExpression`].
    ///
    /// This node contains a [`TSNonNullExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn chain_element_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ChainElement<'a> {
        ChainElement::TSNonNullExpression(self.alloc_ts_non_null_expression(span, expression))
    }

    /// Build a [`ParenthesizedExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_parenthesized_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ParenthesizedExpression<'a> {
        ParenthesizedExpression { node_id: Default::default(), span, expression }
    }

    /// Build a [`ParenthesizedExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::parenthesized_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_parenthesized_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, ParenthesizedExpression<'a>> {
        ArenaBox::new_in(self.parenthesized_expression(span, expression), &self)
    }

    /// Build a [`Statement::BlockStatement`].
    ///
    /// This node contains a [`BlockStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_block(self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> Statement<'a> {
        Statement::BlockStatement(self.alloc_block_statement(span, body))
    }

    /// Build a [`Statement::BlockStatement`] with `scope_id`.
    ///
    /// This node contains a [`BlockStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_block_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::BlockStatement(self.alloc_block_statement_with_scope_id(span, body, scope_id))
    }

    /// Build a [`Statement::BreakStatement`].
    ///
    /// This node contains a [`BreakStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_break(self, span: Span, label: Option<LabelIdentifier<'a>>) -> Statement<'a> {
        Statement::BreakStatement(self.alloc_break_statement(span, label))
    }

    /// Build a [`Statement::ContinueStatement`].
    ///
    /// This node contains a [`ContinueStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_continue(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> Statement<'a> {
        Statement::ContinueStatement(self.alloc_continue_statement(span, label))
    }

    /// Build a [`Statement::DebuggerStatement`].
    ///
    /// This node contains a [`DebuggerStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_debugger(self, span: Span) -> Statement<'a> {
        Statement::DebuggerStatement(self.alloc_debugger_statement(span))
    }

    /// Build a [`Statement::DoWhileStatement`].
    ///
    /// This node contains a [`DoWhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_do_while(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> Statement<'a> {
        Statement::DoWhileStatement(self.alloc_do_while_statement(span, body, test))
    }

    /// Build a [`Statement::EmptyStatement`].
    ///
    /// This node contains an [`EmptyStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_empty(self, span: Span) -> Statement<'a> {
        Statement::EmptyStatement(self.alloc_empty_statement(span))
    }

    /// Build a [`Statement::ExpressionStatement`].
    ///
    /// This node contains an [`ExpressionStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_expression(self, span: Span, expression: Expression<'a>) -> Statement<'a> {
        Statement::ExpressionStatement(self.alloc_expression_statement(span, expression))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for_in(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForInStatement(self.alloc_for_in_statement(span, left, right, body))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for_in_with_scope_id(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::ForInStatement(
            self.alloc_for_in_statement_with_scope_id(span, left, right, body, scope_id),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for_of(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForOfStatement(self.alloc_for_of_statement(span, r#await, left, right, body))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for_of_with_scope_id(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::ForOfStatement(
            self.alloc_for_of_statement_with_scope_id(span, r#await, left, right, body, scope_id),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::ForStatement(self.alloc_for_statement(span, init, test, update, body))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_for_with_scope_id(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::ForStatement(
            self.alloc_for_statement_with_scope_id(span, init, test, update, body, scope_id),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_if(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> Statement<'a> {
        Statement::IfStatement(self.alloc_if_statement(span, test, consequent, alternate))
    }

    /// Build a [`Statement::LabeledStatement`].
    ///
    /// This node contains a [`LabeledStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_labeled(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::LabeledStatement(self.alloc_labeled_statement(span, label, body))
    }

    /// Build a [`Statement::ReturnStatement`].
    ///
    /// This node contains a [`ReturnStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_return(self, span: Span, argument: Option<Expression<'a>>) -> Statement<'a> {
        Statement::ReturnStatement(self.alloc_return_statement(span, argument))
    }

    /// Build a [`Statement::SwitchStatement`].
    ///
    /// This node contains a [`SwitchStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_switch(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc_switch_statement(span, discriminant, cases))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_switch_with_scope_id(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::SwitchStatement(self.alloc_switch_statement_with_scope_id(
            span,
            discriminant,
            cases,
            scope_id,
        ))
    }

    /// Build a [`Statement::ThrowStatement`].
    ///
    /// This node contains a [`ThrowStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_throw(self, span: Span, argument: Expression<'a>) -> Statement<'a> {
        Statement::ThrowStatement(self.alloc_throw_statement(span, argument))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_try<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> Statement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        Statement::TryStatement(self.alloc_try_statement(span, block, handler, finalizer))
    }

    /// Build a [`Statement::WhileStatement`].
    ///
    /// This node contains a [`WhileStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_while(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WhileStatement(self.alloc_while_statement(span, test, body))
    }

    /// Build a [`Statement::WithStatement`].
    ///
    /// This node contains a [`WithStatement`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_with(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> Statement<'a> {
        Statement::WithStatement(self.alloc_with_statement(span, object, body))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn statement_with_with_scope_id(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> Statement<'a> {
        Statement::WithStatement(
            self.alloc_with_statement_with_scope_id(span, object, body, scope_id),
        )
    }

    /// Build a [`Directive`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Directive with any escapes unescaped
    /// * `directive`: Raw content of directive as it appears in source, any escapes left as is
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn directive<S1>(
        self,
        span: Span,
        expression: StringLiteral<'a>,
        directive: S1,
    ) -> Directive<'a>
    where
        S1: Into<Str<'a>>,
    {
        Directive { node_id: Default::default(), span, expression, directive: directive.into() }
    }

    /// Build a [`Hashbang`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn hashbang<S1>(self, span: Span, value: S1) -> Hashbang<'a>
    where
        S1: Into<Str<'a>>,
    {
        Hashbang { node_id: Default::default(), span, value: value.into() }
    }

    /// Build a [`BlockStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_block_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn block_statement(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> BlockStatement<'a> {
        BlockStatement { node_id: Default::default(), span, body, scope_id: Default::default() }
    }

    /// Build a [`BlockStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::block_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_block_statement(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, BlockStatement<'a>> {
        ArenaBox::new_in(self.block_statement(span, body), &self)
    }

    /// Build a [`BlockStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_block_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn block_statement_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> BlockStatement<'a> {
        BlockStatement {
            node_id: Default::default(),
            span,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`BlockStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::block_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_block_statement_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, BlockStatement<'a>> {
        ArenaBox::new_in(self.block_statement_with_scope_id(span, body, scope_id), &self)
    }

    /// Build a [`Declaration::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_variable(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::VariableDeclaration(self.alloc_variable_declaration(
            span,
            kind,
            declarations,
            declare,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_function<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Declaration::FunctionDeclaration(self.alloc_function(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_function_with_scope_id_and_pure_and_pife<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Declaration::FunctionDeclaration(self.alloc_function_with_scope_id_and_pure_and_pife(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_class<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Declaration::ClassDeclaration(self.alloc_class(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_class_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Declaration::ClassDeclaration(self.alloc_class_with_scope_id(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Declaration::TSTypeAliasDeclaration(self.alloc_ts_type_alias_declaration(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_type_alias_with_scope_id<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        Declaration::TSTypeAliasDeclaration(self.alloc_ts_type_alias_declaration_with_scope_id(
            span,
            id,
            type_parameters,
            type_annotation,
            declare,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_interface<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Declaration::TSInterfaceDeclaration(self.alloc_ts_interface_declaration(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_interface_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
    ) -> Declaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        Declaration::TSInterfaceDeclaration(self.alloc_ts_interface_declaration_with_scope_id(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_enum(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSEnumDeclaration(
            self.alloc_ts_enum_declaration(span, id, body, r#const, declare),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
            self.alloc_ts_module_declaration(span, id, body, kind, declare),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_module_with_scope_id(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
    ) -> Declaration<'a> {
        Declaration::TSModuleDeclaration(
            self.alloc_ts_module_declaration_with_scope_id(span, id, body, kind, declare, scope_id),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_global(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
    ) -> Declaration<'a> {
        Declaration::TSGlobalDeclaration(self.alloc_ts_global_declaration(
            span,
            global_span,
            body,
            declare,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_global_with_scope_id(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> Declaration<'a> {
        Declaration::TSGlobalDeclaration(self.alloc_ts_global_declaration_with_scope_id(
            span,
            global_span,
            body,
            declare,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn declaration_ts_import_equals(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> Declaration<'a> {
        Declaration::TSImportEqualsDeclaration(self.alloc_ts_import_equals_declaration(
            span,
            id,
            module_reference,
            import_kind,
        ))
    }

    /// Build a [`VariableDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_variable_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> VariableDeclaration<'a> {
        VariableDeclaration { node_id: Default::default(), span, kind, declarations, declare }
    }

    /// Build a [`VariableDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::variable_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ArenaBox<'a, VariableDeclaration<'a>> {
        ArenaBox::new_in(self.variable_declaration(span, kind, declarations, declare), &self)
    }

    /// Build a [`VariableDeclarator`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `id`
    /// * `type_annotation`
    /// * `init`
    /// * `definite`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn variable_declarator<T1>(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        id: BindingPattern<'a>,
        type_annotation: T1,
        init: Option<Expression<'a>>,
        definite: bool,
    ) -> VariableDeclarator<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        VariableDeclarator {
            node_id: Default::default(),
            span,
            kind,
            id,
            type_annotation: type_annotation.into_in(self.allocator()),
            init,
            definite,
        }
    }

    /// Build an [`EmptyStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_empty_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn empty_statement(self, span: Span) -> EmptyStatement {
        EmptyStatement { node_id: Default::default(), span }
    }

    /// Build an [`EmptyStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::empty_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_empty_statement(self, span: Span) -> ArenaBox<'a, EmptyStatement> {
        ArenaBox::new_in(self.empty_statement(span), &self)
    }

    /// Build an [`ExpressionStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_expression_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ExpressionStatement<'a> {
        ExpressionStatement { node_id: Default::default(), span, expression }
    }

    /// Build an [`ExpressionStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::expression_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_expression_statement(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, ExpressionStatement<'a>> {
        ArenaBox::new_in(self.expression_statement(span, expression), &self)
    }

    /// Build an [`IfStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_if_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> IfStatement<'a> {
        IfStatement { node_id: Default::default(), span, test, consequent, alternate }
    }

    /// Build an [`IfStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::if_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    /// * `alternate`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_if_statement(
        self,
        span: Span,
        test: Expression<'a>,
        consequent: Statement<'a>,
        alternate: Option<Statement<'a>>,
    ) -> ArenaBox<'a, IfStatement<'a>> {
        ArenaBox::new_in(self.if_statement(span, test, consequent, alternate), &self)
    }

    /// Build a [`DoWhileStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_do_while_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> DoWhileStatement<'a> {
        DoWhileStatement { node_id: Default::default(), span, body, test }
    }

    /// Build a [`DoWhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::do_while_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `test`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_do_while_statement(
        self,
        span: Span,
        body: Statement<'a>,
        test: Expression<'a>,
    ) -> ArenaBox<'a, DoWhileStatement<'a>> {
        ArenaBox::new_in(self.do_while_statement(span, body, test), &self)
    }

    /// Build a [`WhileStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_while_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> WhileStatement<'a> {
        WhileStatement { node_id: Default::default(), span, test, body }
    }

    /// Build a [`WhileStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::while_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_while_statement(
        self,
        span: Span,
        test: Expression<'a>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, WhileStatement<'a>> {
        ArenaBox::new_in(self.while_statement(span, test, body), &self)
    }

    /// Build a [`ForStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> ForStatement<'a> {
        ForStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_statement(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, ForStatement<'a>> {
        ArenaBox::new_in(self.for_statement(span, init, test, update, body), &self)
    }

    /// Build a [`ForStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        ForStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `init`
    /// * `test`
    /// * `update`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_statement_with_scope_id(
        self,
        span: Span,
        init: Option<ForStatementInit<'a>>,
        test: Option<Expression<'a>>,
        update: Option<Expression<'a>>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, ForStatement<'a>> {
        ArenaBox::new_in(
            self.for_statement_with_scope_id(span, init, test, update, body, scope_id),
            &self,
        )
    }

    /// Build a [`ForStatementInit::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_statement_init_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ForStatementInit<'a> {
        ForStatementInit::VariableDeclaration(self.alloc_variable_declaration(
            span,
            kind,
            declarations,
            declare,
        ))
    }

    /// Build a [`ForInStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_in_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForInStatement<'a> {
        ForInStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_in_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_in_statement(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, ForInStatement<'a>> {
        ArenaBox::new_in(self.for_in_statement(span, left, right, body), &self)
    }

    /// Build a [`ForInStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_in_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_in_statement_with_scope_id(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ForInStatement<'a> {
        ForInStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_in_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_in_statement_with_scope_id(
        self,
        span: Span,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, ForInStatement<'a>> {
        ArenaBox::new_in(
            self.for_in_statement_with_scope_id(span, left, right, body, scope_id),
            &self,
        )
    }

    /// Build a [`ForStatementLeft::VariableDeclaration`].
    ///
    /// This node contains a [`VariableDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `declarations`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_statement_left_variable_declaration(
        self,
        span: Span,
        kind: VariableDeclarationKind,
        declarations: ArenaVec<'a, VariableDeclarator<'a>>,
        declare: bool,
    ) -> ForStatementLeft<'a> {
        ForStatementLeft::VariableDeclaration(self.alloc_variable_declaration(
            span,
            kind,
            declarations,
            declare,
        ))
    }

    /// Build a [`ForOfStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_of_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ForOfStatement<'a> {
        ForOfStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_of_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_of_statement(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, ForOfStatement<'a>> {
        ArenaBox::new_in(self.for_of_statement(span, r#await, left, right, body), &self)
    }

    /// Build a [`ForOfStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_for_of_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        ForOfStatement {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::for_of_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `await`
    /// * `left`
    /// * `right`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_for_of_statement_with_scope_id(
        self,
        span: Span,
        r#await: bool,
        left: ForStatementLeft<'a>,
        right: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, ForOfStatement<'a>> {
        ArenaBox::new_in(
            self.for_of_statement_with_scope_id(span, r#await, left, right, body, scope_id),
            &self,
        )
    }

    /// Build a [`ContinueStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_continue_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ContinueStatement<'a> {
        ContinueStatement { node_id: Default::default(), span, label }
    }

    /// Build a [`ContinueStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::continue_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_continue_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ArenaBox<'a, ContinueStatement<'a>> {
        ArenaBox::new_in(self.continue_statement(span, label), &self)
    }

    /// Build a [`BreakStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_break_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> BreakStatement<'a> {
        BreakStatement { node_id: Default::default(), span, label }
    }

    /// Build a [`BreakStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::break_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_break_statement(
        self,
        span: Span,
        label: Option<LabelIdentifier<'a>>,
    ) -> ArenaBox<'a, BreakStatement<'a>> {
        ArenaBox::new_in(self.break_statement(span, label), &self)
    }

    /// Build a [`ReturnStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_return_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> ReturnStatement<'a> {
        ReturnStatement { node_id: Default::default(), span, argument }
    }

    /// Build a [`ReturnStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::return_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_return_statement(
        self,
        span: Span,
        argument: Option<Expression<'a>>,
    ) -> ArenaBox<'a, ReturnStatement<'a>> {
        ArenaBox::new_in(self.return_statement(span, argument), &self)
    }

    /// Build a [`WithStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_with_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> WithStatement<'a> {
        WithStatement {
            node_id: Default::default(),
            span,
            object,
            body,
            scope_id: Default::default(),
        }
    }

    /// Build a [`WithStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::with_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_with_statement(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, WithStatement<'a>> {
        ArenaBox::new_in(self.with_statement(span, object, body), &self)
    }

    /// Build a [`WithStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_with_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn with_statement_with_scope_id(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> WithStatement<'a> {
        WithStatement {
            node_id: Default::default(),
            span,
            object,
            body,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`WithStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::with_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object`
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_with_statement_with_scope_id(
        self,
        span: Span,
        object: Expression<'a>,
        body: Statement<'a>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, WithStatement<'a>> {
        ArenaBox::new_in(self.with_statement_with_scope_id(span, object, body, scope_id), &self)
    }

    /// Build a [`SwitchStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_switch_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
    ) -> SwitchStatement<'a> {
        SwitchStatement {
            node_id: Default::default(),
            span,
            discriminant,
            cases,
            scope_id: Default::default(),
        }
    }

    /// Build a [`SwitchStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::switch_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_switch_statement(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
    ) -> ArenaBox<'a, SwitchStatement<'a>> {
        ArenaBox::new_in(self.switch_statement(span, discriminant, cases), &self)
    }

    /// Build a [`SwitchStatement`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_switch_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn switch_statement_with_scope_id(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
    ) -> SwitchStatement<'a> {
        SwitchStatement {
            node_id: Default::default(),
            span,
            discriminant,
            cases,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`SwitchStatement`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::switch_statement_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `discriminant`
    /// * `cases`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_switch_statement_with_scope_id(
        self,
        span: Span,
        discriminant: Expression<'a>,
        cases: ArenaVec<'a, SwitchCase<'a>>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, SwitchStatement<'a>> {
        ArenaBox::new_in(
            self.switch_statement_with_scope_id(span, discriminant, cases, scope_id),
            &self,
        )
    }

    /// Build a [`SwitchCase`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `test`
    /// * `consequent`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn switch_case(
        self,
        span: Span,
        test: Option<Expression<'a>>,
        consequent: ArenaVec<'a, Statement<'a>>,
    ) -> SwitchCase<'a> {
        SwitchCase { node_id: Default::default(), span, test, consequent }
    }

    /// Build a [`LabeledStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_labeled_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> LabeledStatement<'a> {
        LabeledStatement { node_id: Default::default(), span, label, body }
    }

    /// Build a [`LabeledStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::labeled_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_labeled_statement(
        self,
        span: Span,
        label: LabelIdentifier<'a>,
        body: Statement<'a>,
    ) -> ArenaBox<'a, LabeledStatement<'a>> {
        ArenaBox::new_in(self.labeled_statement(span, label, body), &self)
    }

    /// Build a [`ThrowStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_throw_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn throw_statement(self, span: Span, argument: Expression<'a>) -> ThrowStatement<'a> {
        ThrowStatement { node_id: Default::default(), span, argument }
    }

    /// Build a [`ThrowStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::throw_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`: The expression being thrown, e.g. `err` in `throw err;`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_throw_statement(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, ThrowStatement<'a>> {
        ArenaBox::new_in(self.throw_statement(span, argument), &self)
    }

    /// Build a [`TryStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_try_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn try_statement<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> TryStatement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        TryStatement {
            node_id: Default::default(),
            span,
            block: block.into_in(self.allocator()),
            handler: handler.into_in(self.allocator()),
            finalizer: finalizer.into_in(self.allocator()),
        }
    }

    /// Build a [`TryStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::try_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `block`: Statements in the `try` block
    /// * `handler`: The `catch` clause, including the parameter and the block statement
    /// * `finalizer`: The `finally` clause
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_try_statement<T1, T2, T3>(
        self,
        span: Span,
        block: T1,
        handler: T2,
        finalizer: T3,
    ) -> ArenaBox<'a, TryStatement<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, CatchClause<'a>>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, BlockStatement<'a>>>>,
    {
        ArenaBox::new_in(self.try_statement(span, block, handler, finalizer), &self)
    }

    /// Build a [`CatchClause`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_catch_clause`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn catch_clause<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        CatchClause {
            node_id: Default::default(),
            span,
            param,
            body: body.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`CatchClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::catch_clause`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_catch_clause<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
    ) -> ArenaBox<'a, CatchClause<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        ArenaBox::new_in(self.catch_clause(span, param, body), &self)
    }

    /// Build a [`CatchClause`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_catch_clause_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn catch_clause_with_scope_id<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
    ) -> CatchClause<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        CatchClause {
            node_id: Default::default(),
            span,
            param,
            body: body.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`CatchClause`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::catch_clause_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `param`: The caught error parameter, e.g. `e` in `catch (e) {}`
    /// * `body`: The statements run when an error is caught
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_catch_clause_with_scope_id<T1>(
        self,
        span: Span,
        param: Option<CatchParameter<'a>>,
        body: T1,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, CatchClause<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, BlockStatement<'a>>>,
    {
        ArenaBox::new_in(self.catch_clause_with_scope_id(span, param, body, scope_id), &self)
    }

    /// Build a [`CatchParameter`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `pattern`: The bound error
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn catch_parameter<T1>(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
    ) -> CatchParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        CatchParameter {
            node_id: Default::default(),
            span,
            pattern,
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`DebuggerStatement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_debugger_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn debugger_statement(self, span: Span) -> DebuggerStatement {
        DebuggerStatement { node_id: Default::default(), span }
    }

    /// Build a [`DebuggerStatement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::debugger_statement`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_debugger_statement(self, span: Span) -> ArenaBox<'a, DebuggerStatement> {
        ArenaBox::new_in(self.debugger_statement(span), &self)
    }

    /// Build a [`BindingPattern::BindingIdentifier`].
    ///
    /// This node contains a [`BindingIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_pattern_binding_identifier<S1>(self, span: Span, name: S1) -> BindingPattern<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingPattern::BindingIdentifier(self.alloc_binding_identifier(span, name))
    }

    /// Build a [`BindingPattern::BindingIdentifier`] with `symbol_id`.
    ///
    /// This node contains a [`BindingIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_pattern_binding_identifier_with_symbol_id<S1>(
        self,
        span: Span,
        name: S1,
        symbol_id: SymbolId,
    ) -> BindingPattern<'a>
    where
        S1: Into<Ident<'a>>,
    {
        BindingPattern::BindingIdentifier(
            self.alloc_binding_identifier_with_symbol_id(span, name, symbol_id),
        )
    }

    /// Build a [`BindingPattern::ObjectPattern`].
    ///
    /// This node contains an [`ObjectPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_pattern_object_pattern<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        BindingPattern::ObjectPattern(self.alloc_object_pattern(span, properties, rest))
    }

    /// Build a [`BindingPattern::ArrayPattern`].
    ///
    /// This node contains an [`ArrayPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_pattern_array_pattern<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> BindingPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        BindingPattern::ArrayPattern(self.alloc_array_pattern(span, elements, rest))
    }

    /// Build a [`BindingPattern::AssignmentPattern`].
    ///
    /// This node contains an [`AssignmentPattern`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_pattern_assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> BindingPattern<'a> {
        BindingPattern::AssignmentPattern(self.alloc_assignment_pattern(span, left, right))
    }

    /// Build an [`AssignmentPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_assignment_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> AssignmentPattern<'a> {
        AssignmentPattern { node_id: Default::default(), span, left, right }
    }

    /// Build an [`AssignmentPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::assignment_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_assignment_pattern(
        self,
        span: Span,
        left: BindingPattern<'a>,
        right: Expression<'a>,
    ) -> ArenaBox<'a, AssignmentPattern<'a>> {
        ArenaBox::new_in(self.assignment_pattern(span, left, right), &self)
    }

    /// Build an [`ObjectPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_object_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn object_pattern<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> ObjectPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ObjectPattern {
            node_id: Default::default(),
            span,
            properties,
            rest: rest.into_in(self.allocator()),
        }
    }

    /// Build an [`ObjectPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::object_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `properties`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_object_pattern<T1>(
        self,
        span: Span,
        properties: ArenaVec<'a, BindingProperty<'a>>,
        rest: T1,
    ) -> ArenaBox<'a, ObjectPattern<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ArenaBox::new_in(self.object_pattern(span, properties, rest), &self)
    }

    /// Build a [`BindingProperty`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `value`
    /// * `shorthand`
    /// * `computed`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_property(
        self,
        span: Span,
        key: PropertyKey<'a>,
        value: BindingPattern<'a>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'a> {
        BindingProperty { node_id: Default::default(), span, key, value, shorthand, computed }
    }

    /// Build an [`ArrayPattern`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_array_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn array_pattern<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> ArrayPattern<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ArrayPattern {
            node_id: Default::default(),
            span,
            elements,
            rest: rest.into_in(self.allocator()),
        }
    }

    /// Build an [`ArrayPattern`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::array_pattern`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `elements`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_array_pattern<T1>(
        self,
        span: Span,
        elements: ArenaVec<'a, Option<BindingPattern<'a>>>,
        rest: T1,
    ) -> ArenaBox<'a, ArrayPattern<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, BindingRestElement<'a>>>>,
    {
        ArenaBox::new_in(self.array_pattern(span, elements, rest), &self)
    }

    /// Build a [`BindingRestElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_binding_rest_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> BindingRestElement<'a> {
        BindingRestElement { node_id: Default::default(), span, argument }
    }

    /// Build a [`BindingRestElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::binding_rest_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_binding_rest_element(
        self,
        span: Span,
        argument: BindingPattern<'a>,
    ) -> ArenaBox<'a, BindingRestElement<'a>> {
        ArenaBox::new_in(self.binding_rest_element(span, argument), &self)
    }

    /// Build a [`Function`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_function`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn function<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Function {
            node_id: Default::default(),
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            body: body.into_in(self.allocator()),
            scope_id: Default::default(),
            pure: Default::default(),
            pife: Default::default(),
        }
    }

    /// Build a [`Function`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::function`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_function<T1, T2, T3, T4, T5>(
        self,
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
    ) -> ArenaBox<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ArenaBox::new_in(
            self.function(
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
            ),
            &self,
        )
    }

    /// Build a [`Function`] with `scope_id` and `pure` and `pife`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_function_with_scope_id_and_pure_and_pife`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn function_with_scope_id_and_pure_and_pife<T1, T2, T3, T4, T5>(
        self,
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
    ) -> Function<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        Function {
            node_id: Default::default(),
            span,
            r#type,
            id,
            generator,
            r#async,
            declare,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            body: body.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
            pure,
            pife,
        }
    }

    /// Build a [`Function`] with `scope_id` and `pure` and `pife`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::function_with_scope_id_and_pure_and_pife`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_function_with_scope_id_and_pure_and_pife<T1, T2, T3, T4, T5>(
        self,
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
    ) -> ArenaBox<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ArenaBox::new_in(
            self.function_with_scope_id_and_pure_and_pife(
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
            ),
            &self,
        )
    }

    /// Build a [`FormalParameters`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_formal_parameters`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `items`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn formal_parameters<T1>(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> FormalParameters<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        FormalParameters {
            node_id: Default::default(),
            span,
            kind,
            items,
            rest: rest.into_in(self.allocator()),
        }
    }

    /// Build a [`FormalParameters`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::formal_parameters`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `kind`
    /// * `items`
    /// * `rest`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_formal_parameters<T1>(
        self,
        span: Span,
        kind: FormalParameterKind,
        items: ArenaVec<'a, FormalParameter<'a>>,
        rest: T1,
    ) -> ArenaBox<'a, FormalParameters<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, FormalParameterRest<'a>>>>,
    {
        ArenaBox::new_in(self.formal_parameters(span, kind, items, rest), &self)
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn formal_parameter<T1, T2>(
        self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        pattern: BindingPattern<'a>,
        type_annotation: T1,
        initializer: T2,
        optional: bool,
        accessibility: Option<TSAccessibility>,
        readonly: bool,
        r#override: bool,
    ) -> FormalParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, Expression<'a>>>>,
    {
        FormalParameter {
            node_id: Default::default(),
            span,
            decorators,
            pattern,
            type_annotation: type_annotation.into_in(self.allocator()),
            initializer: initializer.into_in(self.allocator()),
            optional,
            accessibility,
            readonly,
            r#override,
        }
    }

    /// Build a [`FormalParameterRest`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_formal_parameter_rest`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `rest`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn formal_parameter_rest<T1>(
        self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
    ) -> FormalParameterRest<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        FormalParameterRest {
            node_id: Default::default(),
            span,
            decorators,
            rest,
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`FormalParameterRest`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::formal_parameter_rest`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `decorators`
    /// * `rest`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_formal_parameter_rest<T1>(
        self,
        span: Span,
        decorators: ArenaVec<'a, Decorator<'a>>,
        rest: BindingRestElement<'a>,
        type_annotation: T1,
    ) -> ArenaBox<'a, FormalParameterRest<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(self.formal_parameter_rest(span, decorators, rest, type_annotation), &self)
    }

    /// Build a [`FunctionBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_function_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `statements`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn function_body(
        self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> FunctionBody<'a> {
        FunctionBody { node_id: Default::default(), span, directives, statements }
    }

    /// Build a [`FunctionBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::function_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `statements`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_function_body(
        self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        statements: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, FunctionBody<'a>> {
        ArenaBox::new_in(self.function_body(span, directives, statements), &self)
    }

    /// Build an [`ArrowFunctionExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_arrow_function_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArrowFunctionExpression {
            node_id: Default::default(),
            span,
            expression,
            r#async,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            body: body.into_in(self.allocator()),
            scope_id: Default::default(),
            pure: Default::default(),
            pife: Default::default(),
        }
    }

    /// Build an [`ArrowFunctionExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::arrow_function_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: Is the function body an arrow expression? i.e. `() => expr` instead of `() => {}`
    /// * `async`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `body`: See `expression` for whether this arrow expression returns an expression.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
    ) -> ArenaBox<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArenaBox::new_in(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            ),
            &self,
        )
    }

    /// Build an [`ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_arrow_function_expression_with_scope_id_and_pure_and_pife`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn arrow_function_expression_with_scope_id_and_pure_and_pife<T1, T2, T3, T4>(
        self,
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
    ) -> ArrowFunctionExpression<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArrowFunctionExpression {
            node_id: Default::default(),
            span,
            expression,
            r#async,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            body: body.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
            pure,
            pife,
        }
    }

    /// Build an [`ArrowFunctionExpression`] with `scope_id` and `pure` and `pife`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::arrow_function_expression_with_scope_id_and_pure_and_pife`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_arrow_function_expression_with_scope_id_and_pure_and_pife<T1, T2, T3, T4>(
        self,
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
    ) -> ArenaBox<'a, ArrowFunctionExpression<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, ArenaBox<'a, FunctionBody<'a>>>,
    {
        ArenaBox::new_in(
            self.arrow_function_expression_with_scope_id_and_pure_and_pife(
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
            ),
            &self,
        )
    }

    /// Build a [`YieldExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_yield_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> YieldExpression<'a> {
        YieldExpression { node_id: Default::default(), span, delegate, argument }
    }

    /// Build a [`YieldExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::yield_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `delegate`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_yield_expression(
        self,
        span: Span,
        delegate: bool,
        argument: Option<Expression<'a>>,
    ) -> ArenaBox<'a, YieldExpression<'a>> {
        ArenaBox::new_in(self.yield_expression(span, delegate, argument), &self)
    }

    /// Build a [`Class`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_class`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> Class<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Class {
            node_id: Default::default(),
            span,
            r#type,
            decorators,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(self.allocator()),
            implements,
            body: body.into_in(self.allocator()),
            r#abstract,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`Class`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::class`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_class<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> ArenaBox<'a, Class<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ArenaBox::new_in(
            self.class(
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
            ),
            &self,
        )
    }

    /// Build a [`Class`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_class_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> Class<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        Class {
            node_id: Default::default(),
            span,
            r#type,
            decorators,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            super_class,
            super_type_arguments: super_type_arguments.into_in(self.allocator()),
            implements,
            body: body.into_in(self.allocator()),
            r#abstract,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`Class`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::class_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_class_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, Class<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ArenaBox::new_in(
            self.class_with_scope_id(
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
            ),
            &self,
        )
    }

    /// Build a [`ClassBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_class_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_body(self, span: Span, body: ArenaVec<'a, ClassElement<'a>>) -> ClassBody<'a> {
        ClassBody { node_id: Default::default(), span, body }
    }

    /// Build a [`ClassBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::class_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_class_body(
        self,
        span: Span,
        body: ArenaVec<'a, ClassElement<'a>>,
    ) -> ArenaBox<'a, ClassBody<'a>> {
        ArenaBox::new_in(self.class_body(span, body), &self)
    }

    /// Build a [`ClassElement::StaticBlock`].
    ///
    /// This node contains a [`StaticBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_static_block(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc_static_block(span, body))
    }

    /// Build a [`ClassElement::StaticBlock`] with `scope_id`.
    ///
    /// This node contains a [`StaticBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_static_block_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> ClassElement<'a> {
        ClassElement::StaticBlock(self.alloc_static_block_with_scope_id(span, body, scope_id))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_constructor<T1>(
        self,
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ClassElement::Constructor(self.alloc_class_constructor(span, key, accessibility, value))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_method_definition<T1>(
        self,
        span: Span,
        r#type: MethodDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ClassElement::MethodDefinition(self.alloc_method_definition(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_property_definition<T1>(
        self,
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ClassElement::PropertyDefinition(self.alloc_property_definition(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_accessor_property<T1>(
        self,
        span: Span,
        r#type: AccessorPropertyType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ClassElement::AccessorProperty(self.alloc_accessor_property(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_element_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: ArenaVec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> ClassElement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ClassElement::TSIndexSignature(self.alloc_ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
        ))
    }

    /// Build a [`ClassConstructor`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_class_constructor`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `accessibility`
    /// * `value`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn class_constructor<T1>(
        self,
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
    ) -> ClassConstructor<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ClassConstructor {
            node_id: Default::default(),
            span,
            key,
            accessibility,
            value: value.into_in(self.allocator()),
        }
    }

    /// Build a [`ClassConstructor`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::class_constructor`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `accessibility`
    /// * `value`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_class_constructor<T1>(
        self,
        span: Span,
        key: ClassConstructorKey<'a>,
        accessibility: Option<TSAccessibility>,
        value: T1,
    ) -> ArenaBox<'a, ClassConstructor<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ArenaBox::new_in(self.class_constructor(span, key, accessibility, value), &self)
    }

    /// Build a [`MethodDefinition`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_method_definition`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn method_definition<T1>(
        self,
        span: Span,
        r#type: MethodDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        MethodDefinition {
            node_id: Default::default(),
            span,
            r#type,
            decorators,
            key,
            value: value.into_in(self.allocator()),
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
    /// If you want a stack-allocated node, use [`AstBuilder::method_definition`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_method_definition<T1>(
        self,
        span: Span,
        r#type: MethodDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        value: T1,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ArenaBox<'a, MethodDefinition<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, Function<'a>>>,
    {
        ArenaBox::new_in(
            self.method_definition(
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
            ),
            &self,
        )
    }

    /// Build a [`PropertyDefinition`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_property_definition`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn property_definition<T1>(
        self,
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
    ) -> PropertyDefinition<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        PropertyDefinition {
            node_id: Default::default(),
            span,
            r#type,
            decorators,
            key,
            type_annotation: type_annotation.into_in(self.allocator()),
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
    /// If you want a stack-allocated node, use [`AstBuilder::property_definition`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_property_definition<T1>(
        self,
        span: Span,
        r#type: PropertyDefinitionType,
        decorators: ArenaVec<'a, Decorator<'a>>,
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
    ) -> ArenaBox<'a, PropertyDefinition<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.property_definition(
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
            ),
            &self,
        )
    }

    /// Build a [`PrivateIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_private_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn private_identifier<S1>(self, span: Span, name: S1) -> PrivateIdentifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        PrivateIdentifier { node_id: Default::default(), span, name: name.into() }
    }

    /// Build a [`PrivateIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::private_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_private_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> ArenaBox<'a, PrivateIdentifier<'a>>
    where
        S1: Into<Ident<'a>>,
    {
        ArenaBox::new_in(self.private_identifier(span, name), &self)
    }

    /// Build a [`StaticBlock`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_static_block`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn static_block(self, span: Span, body: ArenaVec<'a, Statement<'a>>) -> StaticBlock<'a> {
        StaticBlock { node_id: Default::default(), span, body, scope_id: Default::default() }
    }

    /// Build a [`StaticBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::static_block`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_static_block(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, StaticBlock<'a>> {
        ArenaBox::new_in(self.static_block(span, body), &self)
    }

    /// Build a [`StaticBlock`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_static_block_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn static_block_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> StaticBlock<'a> {
        StaticBlock { node_id: Default::default(), span, body, scope_id: Cell::new(Some(scope_id)) }
    }

    /// Build a [`StaticBlock`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::static_block_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_static_block_with_scope_id(
        self,
        span: Span,
        body: ArenaVec<'a, Statement<'a>>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, StaticBlock<'a>> {
        ArenaBox::new_in(self.static_block_with_scope_id(span, body, scope_id), &self)
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_declaration_import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ModuleDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ImportDeclaration(self.alloc_import_declaration(
            span,
            specifiers,
            source,
            phase,
            with_clause,
            import_kind,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ExportAllDeclaration(self.alloc_export_all_declaration(
            span,
            exported,
            source,
            with_clause,
            export_kind,
        ))
    }

    /// Build a [`ModuleDeclaration::ExportDefaultDeclaration`].
    ///
    /// This node contains an [`ExportDefaultDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_declaration_export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::ExportDefaultDeclaration(
            self.alloc_export_default_declaration(span, declaration),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_declaration_export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: ArenaVec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ModuleDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ModuleDeclaration::ExportNamedDeclaration(self.alloc_export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause,
        ))
    }

    /// Build a [`ModuleDeclaration::TSExportAssignment`].
    ///
    /// This node contains a [`TSExportAssignment`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_declaration_ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::TSExportAssignment(self.alloc_ts_export_assignment(span, expression))
    }

    /// Build a [`ModuleDeclaration::TSNamespaceExportDeclaration`].
    ///
    /// This node contains a [`TSNamespaceExportDeclaration`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_declaration_ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> ModuleDeclaration<'a> {
        ModuleDeclaration::TSNamespaceExportDeclaration(
            self.alloc_ts_namespace_export_declaration(span, id),
        )
    }

    /// Build an [`AccessorProperty`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_accessor_property`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn accessor_property<T1>(
        self,
        span: Span,
        r#type: AccessorPropertyType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
    ) -> AccessorProperty<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        AccessorProperty {
            node_id: Default::default(),
            span,
            r#type,
            decorators,
            key,
            type_annotation: type_annotation.into_in(self.allocator()),
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
    /// If you want a stack-allocated node, use [`AstBuilder::accessor_property`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_accessor_property<T1>(
        self,
        span: Span,
        r#type: AccessorPropertyType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        key: PropertyKey<'a>,
        type_annotation: T1,
        value: Option<Expression<'a>>,
        computed: bool,
        r#static: bool,
        r#override: bool,
        definite: bool,
        accessibility: Option<TSAccessibility>,
    ) -> ArenaBox<'a, AccessorProperty<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.accessor_property(
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
            ),
            &self,
        )
    }

    /// Build an [`ImportExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
    ) -> ImportExpression<'a> {
        ImportExpression { node_id: Default::default(), span, source, options, phase }
    }

    /// Build an [`ImportExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `phase`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_expression(
        self,
        span: Span,
        source: Expression<'a>,
        options: Option<Expression<'a>>,
        phase: Option<ImportPhase>,
    ) -> ArenaBox<'a, ImportExpression<'a>> {
        ArenaBox::new_in(self.import_expression(span, source, options, phase), &self)
    }

    /// Build an [`ImportDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ImportDeclaration {
            node_id: Default::default(),
            span,
            specifiers,
            source,
            phase,
            with_clause: with_clause.into_in(self.allocator()),
            import_kind,
        }
    }

    /// Build an [`ImportDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `specifiers`: `None` for `import 'foo'`, `Some([])` for `import {} from 'foo'`
    /// * `source`
    /// * `phase`
    /// * `with_clause`: Some(vec![]) for empty assertion
    /// * `import_kind`: `import type { foo } from 'bar'`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_declaration<T1>(
        self,
        span: Span,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        source: StringLiteral<'a>,
        phase: Option<ImportPhase>,
        with_clause: T1,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            self.import_declaration(span, specifiers, source, phase, with_clause, import_kind),
            &self,
        )
    }

    /// Build an [`ImportDeclarationSpecifier::ImportSpecifier`].
    ///
    /// This node contains an [`ImportSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_declaration_specifier_import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportSpecifier(self.alloc_import_specifier(
            span,
            imported,
            local,
            import_kind,
        ))
    }

    /// Build an [`ImportDeclarationSpecifier::ImportDefaultSpecifier`].
    ///
    /// This node contains an [`ImportDefaultSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_declaration_specifier_import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(
            self.alloc_import_default_specifier(span, local),
        )
    }

    /// Build an [`ImportDeclarationSpecifier::ImportNamespaceSpecifier`].
    ///
    /// This node contains an [`ImportNamespaceSpecifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_declaration_specifier_import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDeclarationSpecifier<'a> {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            self.alloc_import_namespace_specifier(span, local),
        )
    }

    /// Build an [`ImportSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ImportSpecifier<'a> {
        ImportSpecifier { node_id: Default::default(), span, imported, local, import_kind }
    }

    /// Build an [`ImportSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `imported`: Imported symbol.
    /// * `local`: Binding for local symbol.
    /// * `import_kind`: Value or type.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_specifier(
        self,
        span: Span,
        imported: ModuleExportName<'a>,
        local: BindingIdentifier<'a>,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ImportSpecifier<'a>> {
        ArenaBox::new_in(self.import_specifier(span, imported, local, import_kind), &self)
    }

    /// Build an [`ImportDefaultSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_default_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportDefaultSpecifier<'a> {
        ImportDefaultSpecifier { node_id: Default::default(), span, local }
    }

    /// Build an [`ImportDefaultSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_default_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`: The name of the imported symbol.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_default_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ArenaBox<'a, ImportDefaultSpecifier<'a>> {
        ArenaBox::new_in(self.import_default_specifier(span, local), &self)
    }

    /// Build an [`ImportNamespaceSpecifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_import_namespace_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ImportNamespaceSpecifier<'a> {
        ImportNamespaceSpecifier { node_id: Default::default(), span, local }
    }

    /// Build an [`ImportNamespaceSpecifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::import_namespace_specifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_import_namespace_specifier(
        self,
        span: Span,
        local: BindingIdentifier<'a>,
    ) -> ArenaBox<'a, ImportNamespaceSpecifier<'a>> {
        ArenaBox::new_in(self.import_namespace_specifier(span, local), &self)
    }

    /// Build a [`WithClause`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_with_clause`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `keyword`
    /// * `with_entries`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn with_clause(
        self,
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: ArenaVec<'a, ImportAttribute<'a>>,
    ) -> WithClause<'a> {
        WithClause { node_id: Default::default(), span, keyword, with_entries }
    }

    /// Build a [`WithClause`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::with_clause`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `keyword`
    /// * `with_entries`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_with_clause(
        self,
        span: Span,
        keyword: WithClauseKeyword,
        with_entries: ArenaVec<'a, ImportAttribute<'a>>,
    ) -> ArenaBox<'a, WithClause<'a>> {
        ArenaBox::new_in(self.with_clause(span, keyword, with_entries), &self)
    }

    /// Build an [`ImportAttribute`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`
    /// * `value`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_attribute(
        self,
        span: Span,
        key: ImportAttributeKey<'a>,
        value: StringLiteral<'a>,
    ) -> ImportAttribute<'a> {
        ImportAttribute { node_id: Default::default(), span, key, value }
    }

    /// Build an [`ImportAttributeKey::Identifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_attribute_key_identifier<S1>(self, span: Span, name: S1) -> ImportAttributeKey<'a>
    where
        S1: Into<Ident<'a>>,
    {
        ImportAttributeKey::Identifier(self.identifier_name(span, name))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_attribute_key_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ImportAttributeKey<'a>
    where
        S1: Into<Str<'a>>,
    {
        ImportAttributeKey::StringLiteral(self.string_literal(span, value, raw))
    }

    /// Build an [`ImportAttributeKey::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn import_attribute_key_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> ImportAttributeKey<'a>
    where
        S1: Into<Str<'a>>,
    {
        ImportAttributeKey::StringLiteral(self.string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
        ))
    }

    /// Build an [`ExportNamedDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_export_named_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: ArenaVec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ExportNamedDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ExportNamedDeclaration {
            node_id: Default::default(),
            span,
            declaration,
            specifiers,
            source,
            export_kind,
            with_clause: with_clause.into_in(self.allocator()),
        }
    }

    /// Build an [`ExportNamedDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::export_named_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    /// * `specifiers`
    /// * `source`
    /// * `export_kind`: `export type { foo }`
    /// * `with_clause`: Some(vec![]) for empty assertion
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_export_named_declaration<T1>(
        self,
        span: Span,
        declaration: Option<Declaration<'a>>,
        specifiers: ArenaVec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
        export_kind: ImportOrExportKind,
        with_clause: T1,
    ) -> ArenaBox<'a, ExportNamedDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            self.export_named_declaration(
                span,
                declaration,
                specifiers,
                source,
                export_kind,
                with_clause,
            ),
            &self,
        )
    }

    /// Build an [`ExportDefaultDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_export_default_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
    ) -> ExportDefaultDeclaration<'a> {
        ExportDefaultDeclaration { node_id: Default::default(), span, declaration }
    }

    /// Build an [`ExportDefaultDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::export_default_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `declaration`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_export_default_declaration(
        self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'a>,
    ) -> ArenaBox<'a, ExportDefaultDeclaration<'a>> {
        ArenaBox::new_in(self.export_default_declaration(span, declaration), &self)
    }

    /// Build an [`ExportAllDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_export_all_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ExportAllDeclaration {
            node_id: Default::default(),
            span,
            exported,
            source,
            with_clause: with_clause.into_in(self.allocator()),
            export_kind,
        }
    }

    /// Build an [`ExportAllDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::export_all_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `exported`: If this declaration is re-named
    /// * `source`
    /// * `with_clause`: Will be `Some(vec![])` for empty assertion
    /// * `export_kind`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_export_all_declaration<T1>(
        self,
        span: Span,
        exported: Option<ModuleExportName<'a>>,
        source: StringLiteral<'a>,
        with_clause: T1,
        export_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, ExportAllDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, WithClause<'a>>>>,
    {
        ArenaBox::new_in(
            self.export_all_declaration(span, exported, source, with_clause, export_kind),
            &self,
        )
    }

    /// Build an [`ExportSpecifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `local`
    /// * `exported`
    /// * `export_kind`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_specifier(
        self,
        span: Span,
        local: ModuleExportName<'a>,
        exported: ModuleExportName<'a>,
        export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'a> {
        ExportSpecifier { node_id: Default::default(), span, local, exported, export_kind }
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_function_declaration<T1, T2, T3, T4, T5>(
        self,
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
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ExportDefaultDeclarationKind::FunctionDeclaration(self.alloc_function(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_function_declaration_with_scope_id_and_pure_and_pife<
        T1,
        T2,
        T3,
        T4,
        T5,
    >(
        self,
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
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<ArenaBox<'a, FunctionBody<'a>>>>,
    {
        ExportDefaultDeclarationKind::FunctionDeclaration(
            self.alloc_function_with_scope_id_and_pure_and_pife(
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
            ),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_class_declaration<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ExportDefaultDeclarationKind::ClassDeclaration(self.alloc_class(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_class_declaration_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#type: ClassType,
        decorators: ArenaVec<'a, Decorator<'a>>,
        id: Option<BindingIdentifier<'a>>,
        type_parameters: T1,
        super_class: Option<Expression<'a>>,
        super_type_arguments: T2,
        implements: ArenaVec<'a, TSClassImplements<'a>>,
        body: T3,
        r#abstract: bool,
        declare: bool,
        scope_id: ScopeId,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, ClassBody<'a>>>,
    {
        ExportDefaultDeclarationKind::ClassDeclaration(self.alloc_class_with_scope_id(
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(self.alloc_ts_interface_declaration(
            span,
            id,
            type_parameters,
            extends,
            body,
            declare,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn export_default_declaration_kind_ts_interface_declaration_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
    ) -> ExportDefaultDeclarationKind<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(
            self.alloc_ts_interface_declaration_with_scope_id(
                span,
                id,
                type_parameters,
                extends,
                body,
                declare,
                scope_id,
            ),
        )
    }

    /// Build a [`ModuleExportName::IdentifierName`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_export_name_identifier_name<S1>(
        self,
        span: Span,
        name: S1,
    ) -> ModuleExportName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        ModuleExportName::IdentifierName(self.identifier_name(span, name))
    }

    /// Build a [`ModuleExportName::IdentifierReference`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_export_name_identifier_reference<S1>(
        self,
        span: Span,
        name: S1,
    ) -> ModuleExportName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        ModuleExportName::IdentifierReference(self.identifier_reference(span, name))
    }

    /// Build a [`ModuleExportName::IdentifierReference`] with `reference_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_export_name_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> ModuleExportName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        ModuleExportName::IdentifierReference(self.identifier_reference_with_reference_id(
            span,
            name,
            reference_id,
        ))
    }

    /// Build a [`ModuleExportName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_export_name_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ModuleExportName<'a>
    where
        S1: Into<Str<'a>>,
    {
        ModuleExportName::StringLiteral(self.string_literal(span, value, raw))
    }

    /// Build a [`ModuleExportName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn module_export_name_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> ModuleExportName<'a>
    where
        S1: Into<Str<'a>>,
    {
        ModuleExportName::StringLiteral(self.string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
        ))
    }

    /// Build a [`V8IntrinsicExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_v8_intrinsic_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn v8_intrinsic_expression(
        self,
        span: Span,
        name: IdentifierName<'a>,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> V8IntrinsicExpression<'a> {
        V8IntrinsicExpression { node_id: Default::default(), span, name, arguments }
    }

    /// Build a [`V8IntrinsicExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::v8_intrinsic_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_v8_intrinsic_expression(
        self,
        span: Span,
        name: IdentifierName<'a>,
        arguments: ArenaVec<'a, Argument<'a>>,
    ) -> ArenaBox<'a, V8IntrinsicExpression<'a>> {
        ArenaBox::new_in(self.v8_intrinsic_expression(span, name, arguments), &self)
    }

    /// Build a [`BooleanLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_boolean_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn boolean_literal(self, span: Span, value: bool) -> BooleanLiteral {
        BooleanLiteral { node_id: Default::default(), span, value }
    }

    /// Build a [`BooleanLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::boolean_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_boolean_literal(self, span: Span, value: bool) -> ArenaBox<'a, BooleanLiteral> {
        ArenaBox::new_in(self.boolean_literal(span, value), &self)
    }

    /// Build a [`NullLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_null_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn null_literal(self, span: Span) -> NullLiteral {
        NullLiteral { node_id: Default::default(), span }
    }

    /// Build a [`NullLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::null_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_null_literal(self, span: Span) -> ArenaBox<'a, NullLiteral> {
        ArenaBox::new_in(self.null_literal(span), &self)
    }

    /// Build a [`NumericLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_numeric_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> NumericLiteral<'a> {
        NumericLiteral { node_id: Default::default(), span, value, raw, base }
    }

    /// Build a [`NumericLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::numeric_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the number, converted into base 10
    /// * `raw`: The number as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> ArenaBox<'a, NumericLiteral<'a>> {
        ArenaBox::new_in(self.numeric_literal(span, value, raw, base), &self)
    }

    /// Build a [`StringLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_string_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> StringLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        StringLiteral {
            node_id: Default::default(),
            span,
            value: value.into(),
            raw,
            lone_surrogates: Default::default(),
        }
    }

    /// Build a [`StringLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::string_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ArenaBox<'a, StringLiteral<'a>>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(self.string_literal(span, value, raw), &self)
    }

    /// Build a [`StringLiteral`] with `lone_surrogates`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_string_literal_with_lone_surrogates`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> StringLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        StringLiteral {
            node_id: Default::default(),
            span,
            value: value.into(),
            raw,
            lone_surrogates,
        }
    }

    /// Build a [`StringLiteral`] with `lone_surrogates`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::string_literal_with_lone_surrogates`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> ArenaBox<'a, StringLiteral<'a>>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(
            self.string_literal_with_lone_surrogates(span, value, raw, lone_surrogates),
            &self,
        )
    }

    /// Build a [`BigIntLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_big_int_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn big_int_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
    ) -> BigIntLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        BigIntLiteral { node_id: Default::default(), span, value: value.into(), raw, base }
    }

    /// Build a [`BigIntLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::big_int_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: Bigint value in base 10 with no underscores
    /// * `raw`: The bigint as it appears in source code
    /// * `base`: The base representation used by the literal in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_big_int_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
    ) -> ArenaBox<'a, BigIntLiteral<'a>>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(self.big_int_literal(span, value, raw, base), &self)
    }

    /// Build a [`RegExpLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn reg_exp_literal(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
    ) -> RegExpLiteral<'a> {
        RegExpLiteral { node_id: Default::default(), span, regex, raw }
    }

    /// Build a [`RegExpLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::reg_exp_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `regex`: The parsed regular expression. See [`oxc_regular_expression`] for more
    /// * `raw`: The regular expression as it appears in source code
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_reg_exp_literal(
        self,
        span: Span,
        regex: RegExp<'a>,
        raw: Option<Str<'a>>,
    ) -> ArenaBox<'a, RegExpLiteral<'a>> {
        ArenaBox::new_in(self.reg_exp_literal(span, regex, raw), &self)
    }

    /// Build a [`JSXElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXElement<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        JSXElement {
            node_id: Default::default(),
            span,
            opening_element: opening_element.into_in(self.allocator()),
            children,
            closing_element: closing_element.into_in(self.allocator()),
        }
    }

    /// Build a [`JSXElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_element`: Opening tag of the element.
    /// * `children`: Children of the element.
    /// * `closing_element`: Closing tag of the element.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> ArenaBox<'a, JSXElement<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        ArenaBox::new_in(self.jsx_element(span, opening_element, children, closing_element), &self)
    }

    /// Build a [`JSXOpeningElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    /// * `type_arguments`: Type parameters for generic JSX elements.
    /// * `attributes`: List of JSX attributes. In React-like applications, these become props.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_opening_element<T1>(
        self,
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: ArenaVec<'a, JSXAttributeItem<'a>>,
    ) -> JSXOpeningElement<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        JSXOpeningElement {
            node_id: Default::default(),
            span,
            name,
            type_arguments: type_arguments.into_in(self.allocator()),
            attributes,
        }
    }

    /// Build a [`JSXOpeningElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_opening_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The possibly-namespaced tag name, e.g. `Foo` in `<Foo />`.
    /// * `type_arguments`: Type parameters for generic JSX elements.
    /// * `attributes`: List of JSX attributes. In React-like applications, these become props.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_opening_element<T1>(
        self,
        span: Span,
        name: JSXElementName<'a>,
        type_arguments: T1,
        attributes: ArenaVec<'a, JSXAttributeItem<'a>>,
    ) -> ArenaBox<'a, JSXOpeningElement<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(self.jsx_opening_element(span, name, type_arguments, attributes), &self)
    }

    /// Build a [`JSXClosingElement`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The tag name, e.g. `Foo` in `</Foo>`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> JSXClosingElement<'a> {
        JSXClosingElement { node_id: Default::default(), span, name }
    }

    /// Build a [`JSXClosingElement`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_closing_element`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The tag name, e.g. `Foo` in `</Foo>`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_closing_element(
        self,
        span: Span,
        name: JSXElementName<'a>,
    ) -> ArenaBox<'a, JSXClosingElement<'a>> {
        ArenaBox::new_in(self.jsx_closing_element(span, name), &self)
    }

    /// Build a [`JSXFragment`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXFragment<'a> {
        JSXFragment {
            node_id: Default::default(),
            span,
            opening_fragment,
            children,
            closing_fragment,
        }
    }

    /// Build a [`JSXFragment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_fragment`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `opening_fragment`: `<>`
    /// * `children`: Elements inside the fragment.
    /// * `closing_fragment`: `</>`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> ArenaBox<'a, JSXFragment<'a>> {
        ArenaBox::new_in(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment),
            &self,
        )
    }

    /// Build a [`JSXOpeningFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_opening_fragment(self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { node_id: Default::default(), span }
    }

    /// Build a [`JSXClosingFragment`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_closing_fragment(self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { node_id: Default::default(), span }
    }

    /// Build a [`JSXElementName::Identifier`].
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_identifier<S1>(self, span: Span, name: S1) -> JSXElementName<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXElementName::Identifier(self.alloc_jsx_identifier(span, name))
    }

    /// Build a [`JSXElementName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_identifier_reference<S1>(
        self,
        span: Span,
        name: S1,
    ) -> JSXElementName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXElementName::IdentifierReference(self.alloc_identifier_reference(span, name))
    }

    /// Build a [`JSXElementName::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> JSXElementName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXElementName::IdentifierReference(self.alloc_identifier_reference_with_reference_id(
            span,
            name,
            reference_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::NamespacedName(self.alloc_jsx_namespaced_name(span, namespace, name))
    }

    /// Build a [`JSXElementName::MemberExpression`].
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXElementName<'a> {
        JSXElementName::MemberExpression(self.alloc_jsx_member_expression(span, object, property))
    }

    /// Build a [`JSXElementName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_element_name_this_expression(self, span: Span) -> JSXElementName<'a> {
        JSXElementName::ThisExpression(self.alloc_this_expression(span))
    }

    /// Build a [`JSXNamespacedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXNamespacedName<'a> {
        JSXNamespacedName { node_id: Default::default(), span, namespace, name }
    }

    /// Build a [`JSXNamespacedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_namespaced_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> ArenaBox<'a, JSXNamespacedName<'a>> {
        ArenaBox::new_in(self.jsx_namespaced_name(span, namespace, name), &self)
    }

    /// Build a [`JSXMemberExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpression<'a> {
        JSXMemberExpression { node_id: Default::default(), span, object, property }
    }

    /// Build a [`JSXMemberExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_member_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> ArenaBox<'a, JSXMemberExpression<'a>> {
        ArenaBox::new_in(self.jsx_member_expression(span, object, property), &self)
    }

    /// Build a [`JSXMemberExpressionObject::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_member_expression_object_identifier_reference<S1>(
        self,
        span: Span,
        name: S1,
    ) -> JSXMemberExpressionObject<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXMemberExpressionObject::IdentifierReference(self.alloc_identifier_reference(span, name))
    }

    /// Build a [`JSXMemberExpressionObject::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_member_expression_object_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> JSXMemberExpressionObject<'a>
    where
        S1: Into<Ident<'a>>,
    {
        JSXMemberExpressionObject::IdentifierReference(
            self.alloc_identifier_reference_with_reference_id(span, name, reference_id),
        )
    }

    /// Build a [`JSXMemberExpressionObject::MemberExpression`].
    ///
    /// This node contains a [`JSXMemberExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `object`: The object being accessed. This is everything before the last `.`.
    /// * `property`: The property being accessed. This is everything after the last `.`.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_member_expression_object_member_expression(
        self,
        span: Span,
        object: JSXMemberExpressionObject<'a>,
        property: JSXIdentifier<'a>,
    ) -> JSXMemberExpressionObject<'a> {
        JSXMemberExpressionObject::MemberExpression(
            self.alloc_jsx_member_expression(span, object, property),
        )
    }

    /// Build a [`JSXMemberExpressionObject::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_member_expression_object_this_expression(
        self,
        span: Span,
    ) -> JSXMemberExpressionObject<'a> {
        JSXMemberExpressionObject::ThisExpression(self.alloc_this_expression(span))
    }

    /// Build a [`JSXExpressionContainer`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXExpressionContainer<'a> {
        JSXExpressionContainer { node_id: Default::default(), span, expression }
    }

    /// Build a [`JSXExpressionContainer`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_expression_container`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> ArenaBox<'a, JSXExpressionContainer<'a>> {
        ArenaBox::new_in(self.jsx_expression_container(span, expression), &self)
    }

    /// Build a [`JSXExpression::EmptyExpression`].
    ///
    /// This node contains a [`JSXEmptyExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_expression_empty_expression(self, span: Span) -> JSXExpression<'a> {
        JSXExpression::EmptyExpression(self.alloc_jsx_empty_expression(span))
    }

    /// Build a [`JSXEmptyExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_empty_expression(self, span: Span) -> JSXEmptyExpression {
        JSXEmptyExpression { node_id: Default::default(), span }
    }

    /// Build a [`JSXEmptyExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_empty_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_empty_expression(self, span: Span) -> ArenaBox<'a, JSXEmptyExpression> {
        ArenaBox::new_in(self.jsx_empty_expression(span), &self)
    }

    /// Build a [`JSXAttributeItem::Attribute`].
    ///
    /// This node contains a [`JSXAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_item_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::Attribute(self.alloc_jsx_attribute(span, name, value))
    }

    /// Build a [`JSXAttributeItem::SpreadAttribute`].
    ///
    /// This node contains a [`JSXSpreadAttribute`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_item_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXAttributeItem<'a> {
        JSXAttributeItem::SpreadAttribute(self.alloc_jsx_spread_attribute(span, argument))
    }

    /// Build a [`JSXAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> JSXAttribute<'a> {
        JSXAttribute { node_id: Default::default(), span, name, value }
    }

    /// Build a [`JSXAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_attribute`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the attribute. This is a prop in React-like applications.
    /// * `value`: The value of the attribute. This can be a string literal, an expression,
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_attribute(
        self,
        span: Span,
        name: JSXAttributeName<'a>,
        value: Option<JSXAttributeValue<'a>>,
    ) -> ArenaBox<'a, JSXAttribute<'a>> {
        ArenaBox::new_in(self.jsx_attribute(span, name, value), &self)
    }

    /// Build a [`JSXSpreadAttribute`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> JSXSpreadAttribute<'a> {
        JSXSpreadAttribute { node_id: Default::default(), span, argument }
    }

    /// Build a [`JSXSpreadAttribute`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_spread_attribute`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `argument`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_spread_attribute(
        self,
        span: Span,
        argument: Expression<'a>,
    ) -> ArenaBox<'a, JSXSpreadAttribute<'a>> {
        ArenaBox::new_in(self.jsx_spread_attribute(span, argument), &self)
    }

    /// Build a [`JSXAttributeName::Identifier`].
    ///
    /// This node contains a [`JSXIdentifier`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_name_identifier<S1>(self, span: Span, name: S1) -> JSXAttributeName<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXAttributeName::Identifier(self.alloc_jsx_identifier(span, name))
    }

    /// Build a [`JSXAttributeName::NamespacedName`].
    ///
    /// This node contains a [`JSXNamespacedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `namespace`: Namespace portion of the name, e.g. `Apple` in `<Apple:Orange />`
    /// * `name`: Name portion of the name, e.g. `Orange` in `<Apple:Orange />`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_name_namespaced_name(
        self,
        span: Span,
        namespace: JSXIdentifier<'a>,
        name: JSXIdentifier<'a>,
    ) -> JSXAttributeName<'a> {
        JSXAttributeName::NamespacedName(self.alloc_jsx_namespaced_name(span, namespace, name))
    }

    /// Build a [`JSXAttributeValue::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_value_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> JSXAttributeValue<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXAttributeValue::StringLiteral(self.alloc_string_literal(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_value_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> JSXAttributeValue<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXAttributeValue::StringLiteral(self.alloc_string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
        ))
    }

    /// Build a [`JSXAttributeValue::ExpressionContainer`].
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_value_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXAttributeValue<'a> {
        JSXAttributeValue::ExpressionContainer(
            self.alloc_jsx_expression_container(span, expression),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_value_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXAttributeValue<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        JSXAttributeValue::Element(self.alloc_jsx_element(
            span,
            opening_element,
            children,
            closing_element,
        ))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_attribute_value_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXAttributeValue<'a> {
        JSXAttributeValue::Fragment(self.alloc_jsx_fragment(
            span,
            opening_fragment,
            children,
            closing_fragment,
        ))
    }

    /// Build a [`JSXIdentifier`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_identifier<S1>(self, span: Span, name: S1) -> JSXIdentifier<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXIdentifier { node_id: Default::default(), span, name: name.into() }
    }

    /// Build a [`JSXIdentifier`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_identifier`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `name`: The name of the identifier.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_identifier<S1>(self, span: Span, name: S1) -> ArenaBox<'a, JSXIdentifier<'a>>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(self.jsx_identifier(span, name), &self)
    }

    /// Build a [`JSXChild::Text`].
    ///
    /// This node contains a [`JSXText`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_child_text<S1>(self, span: Span, value: S1, raw: Option<Str<'a>>) -> JSXChild<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXChild::Text(self.alloc_jsx_text(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_child_element<T1, T2>(
        self,
        span: Span,
        opening_element: T1,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_element: T2,
    ) -> JSXChild<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, JSXOpeningElement<'a>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, JSXClosingElement<'a>>>>,
    {
        JSXChild::Element(self.alloc_jsx_element(span, opening_element, children, closing_element))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_child_fragment(
        self,
        span: Span,
        opening_fragment: JSXOpeningFragment,
        children: ArenaVec<'a, JSXChild<'a>>,
        closing_fragment: JSXClosingFragment,
    ) -> JSXChild<'a> {
        JSXChild::Fragment(self.alloc_jsx_fragment(
            span,
            opening_fragment,
            children,
            closing_fragment,
        ))
    }

    /// Build a [`JSXChild::ExpressionContainer`].
    ///
    /// This node contains a [`JSXExpressionContainer`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression inside the container.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_child_expression_container(
        self,
        span: Span,
        expression: JSXExpression<'a>,
    ) -> JSXChild<'a> {
        JSXChild::ExpressionContainer(self.alloc_jsx_expression_container(span, expression))
    }

    /// Build a [`JSXChild::Spread`].
    ///
    /// This node contains a [`JSXSpreadChild`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_child_spread(self, span: Span, expression: Expression<'a>) -> JSXChild<'a> {
        JSXChild::Spread(self.alloc_jsx_spread_child(span, expression))
    }

    /// Build a [`JSXSpreadChild`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_spread_child(self, span: Span, expression: Expression<'a>) -> JSXSpreadChild<'a> {
        JSXSpreadChild { node_id: Default::default(), span, expression }
    }

    /// Build a [`JSXSpreadChild`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_spread_child`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `expression`: The expression being spread.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_spread_child(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, JSXSpreadChild<'a>> {
        ArenaBox::new_in(self.jsx_spread_child(span, expression), &self)
    }

    /// Build a [`JSXText`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_jsx_text`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn jsx_text<S1>(self, span: Span, value: S1, raw: Option<Str<'a>>) -> JSXText<'a>
    where
        S1: Into<Str<'a>>,
    {
        JSXText { node_id: Default::default(), span, value: value.into(), raw }
    }

    /// Build a [`JSXText`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::jsx_text`] instead.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The text content.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_jsx_text<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> ArenaBox<'a, JSXText<'a>>
    where
        S1: Into<Str<'a>>,
    {
        ArenaBox::new_in(self.jsx_text(span, value, raw), &self)
    }

    /// Build a [`TSThisParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `this_span`
    /// * `type_annotation`: Type type the `this` keyword will have in the function
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_this_parameter<T1>(
        self,
        span: Span,
        this_span: Span,
        type_annotation: T1,
    ) -> TSThisParameter<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSThisParameter {
            node_id: Default::default(),
            span,
            this_span,
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`TSThisParameter`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_this_parameter`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `this_span`
    /// * `type_annotation`: Type type the `this` keyword will have in the function
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_this_parameter<T1>(
        self,
        span: Span,
        this_span: Span,
        type_annotation: T1,
    ) -> ArenaBox<'a, TSThisParameter<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(self.ts_this_parameter(span, this_span, type_annotation), &self)
    }

    /// Build a [`TSEnumDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
    ) -> TSEnumDeclaration<'a> {
        TSEnumDeclaration { node_id: Default::default(), span, id, body, r#const, declare }
    }

    /// Build a [`TSEnumDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_enum_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `body`
    /// * `const`: `true` for const enums
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_enum_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        body: TSEnumBody<'a>,
        r#const: bool,
        declare: bool,
    ) -> ArenaBox<'a, TSEnumDeclaration<'a>> {
        ArenaBox::new_in(self.ts_enum_declaration(span, id, body, r#const, declare), &self)
    }

    /// Build a [`TSEnumBody`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_body(
        self,
        span: Span,
        members: ArenaVec<'a, TSEnumMember<'a>>,
    ) -> TSEnumBody<'a> {
        TSEnumBody { node_id: Default::default(), span, members, scope_id: Default::default() }
    }

    /// Build a [`TSEnumBody`] with `scope_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_body_with_scope_id(
        self,
        span: Span,
        members: ArenaVec<'a, TSEnumMember<'a>>,
        scope_id: ScopeId,
    ) -> TSEnumBody<'a> {
        TSEnumBody {
            node_id: Default::default(),
            span,
            members,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSEnumMember`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `initializer`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member(
        self,
        span: Span,
        id: TSEnumMemberName<'a>,
        initializer: Option<Expression<'a>>,
    ) -> TSEnumMember<'a> {
        TSEnumMember { node_id: Default::default(), span, id, initializer }
    }

    /// Build a [`TSEnumMemberName::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_identifier<S1>(self, span: Span, name: S1) -> TSEnumMemberName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSEnumMemberName::Identifier(self.alloc_identifier_name(span, name))
    }

    /// Build a [`TSEnumMemberName::String`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_string<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> TSEnumMemberName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSEnumMemberName::String(self.alloc_string_literal(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_string_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> TSEnumMemberName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSEnumMemberName::String(self.alloc_string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_computed_string<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> TSEnumMemberName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSEnumMemberName::ComputedString(self.alloc_string_literal(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_computed_string_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> TSEnumMemberName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSEnumMemberName::ComputedString(self.alloc_string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_enum_member_name_computed_template_string(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> TSEnumMemberName<'a> {
        TSEnumMemberName::ComputedTemplateString(self.alloc_template_literal(
            span,
            quasis,
            expressions,
        ))
    }

    /// Build a [`TSTypeAnnotation`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`: The actual type in the annotation
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTypeAnnotation<'a> {
        TSTypeAnnotation { node_id: Default::default(), span, type_annotation }
    }

    /// Build a [`TSTypeAnnotation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_annotation`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`: The actual type in the annotation
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_annotation(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSTypeAnnotation<'a>> {
        ArenaBox::new_in(self.ts_type_annotation(span, type_annotation), &self)
    }

    /// Build a [`TSLiteralType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_literal_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSLiteralType<'a> {
        TSLiteralType { node_id: Default::default(), span, literal }
    }

    /// Build a [`TSLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_literal_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_literal_type(
        self,
        span: Span,
        literal: TSLiteral<'a>,
    ) -> ArenaBox<'a, TSLiteralType<'a>> {
        ArenaBox::new_in(self.ts_literal_type(span, literal), &self)
    }

    /// Build a [`TSLiteral::BooleanLiteral`].
    ///
    /// This node contains a [`BooleanLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The boolean value itself
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_boolean_literal(self, span: Span, value: bool) -> TSLiteral<'a> {
        TSLiteral::BooleanLiteral(self.alloc_boolean_literal(span, value))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_numeric_literal(
        self,
        span: Span,
        value: f64,
        raw: Option<Str<'a>>,
        base: NumberBase,
    ) -> TSLiteral<'a> {
        TSLiteral::NumericLiteral(self.alloc_numeric_literal(span, value, raw, base))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_big_int_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        base: BigintBase,
    ) -> TSLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSLiteral::BigIntLiteral(self.alloc_big_int_literal(span, value, raw, base))
    }

    /// Build a [`TSLiteral::StringLiteral`].
    ///
    /// This node contains a [`StringLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> TSLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSLiteral::StringLiteral(self.alloc_string_literal(span, value, raw))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> TSLiteral<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSLiteral::StringLiteral(self.alloc_string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_template_literal(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        expressions: ArenaVec<'a, Expression<'a>>,
    ) -> TSLiteral<'a> {
        TSLiteral::TemplateLiteral(self.alloc_template_literal(span, quasis, expressions))
    }

    /// Build a [`TSLiteral::UnaryExpression`].
    ///
    /// This node contains an [`UnaryExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `argument`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_literal_unary_expression(
        self,
        span: Span,
        operator: UnaryOperator,
        argument: Expression<'a>,
    ) -> TSLiteral<'a> {
        TSLiteral::UnaryExpression(self.alloc_unary_expression(span, operator, argument))
    }

    /// Build a [`TSType::TSAnyKeyword`].
    ///
    /// This node contains a [`TSAnyKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_any_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSAnyKeyword(self.alloc_ts_any_keyword(span))
    }

    /// Build a [`TSType::TSBigIntKeyword`].
    ///
    /// This node contains a [`TSBigIntKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_big_int_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBigIntKeyword(self.alloc_ts_big_int_keyword(span))
    }

    /// Build a [`TSType::TSBooleanKeyword`].
    ///
    /// This node contains a [`TSBooleanKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_boolean_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSBooleanKeyword(self.alloc_ts_boolean_keyword(span))
    }

    /// Build a [`TSType::TSIntrinsicKeyword`].
    ///
    /// This node contains a [`TSIntrinsicKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_intrinsic_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSIntrinsicKeyword(self.alloc_ts_intrinsic_keyword(span))
    }

    /// Build a [`TSType::TSNeverKeyword`].
    ///
    /// This node contains a [`TSNeverKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_never_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNeverKeyword(self.alloc_ts_never_keyword(span))
    }

    /// Build a [`TSType::TSNullKeyword`].
    ///
    /// This node contains a [`TSNullKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_null_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNullKeyword(self.alloc_ts_null_keyword(span))
    }

    /// Build a [`TSType::TSNumberKeyword`].
    ///
    /// This node contains a [`TSNumberKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_number_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSNumberKeyword(self.alloc_ts_number_keyword(span))
    }

    /// Build a [`TSType::TSObjectKeyword`].
    ///
    /// This node contains a [`TSObjectKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_object_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSObjectKeyword(self.alloc_ts_object_keyword(span))
    }

    /// Build a [`TSType::TSStringKeyword`].
    ///
    /// This node contains a [`TSStringKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_string_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSStringKeyword(self.alloc_ts_string_keyword(span))
    }

    /// Build a [`TSType::TSSymbolKeyword`].
    ///
    /// This node contains a [`TSSymbolKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_symbol_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSSymbolKeyword(self.alloc_ts_symbol_keyword(span))
    }

    /// Build a [`TSType::TSUndefinedKeyword`].
    ///
    /// This node contains a [`TSUndefinedKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_undefined_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUndefinedKeyword(self.alloc_ts_undefined_keyword(span))
    }

    /// Build a [`TSType::TSUnknownKeyword`].
    ///
    /// This node contains a [`TSUnknownKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_unknown_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSUnknownKeyword(self.alloc_ts_unknown_keyword(span))
    }

    /// Build a [`TSType::TSVoidKeyword`].
    ///
    /// This node contains a [`TSVoidKeyword`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_void_keyword(self, span: Span) -> TSType<'a> {
        TSType::TSVoidKeyword(self.alloc_ts_void_keyword(span))
    }

    /// Build a [`TSType::TSArrayType`].
    ///
    /// This node contains a [`TSArrayType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_array_type(self, span: Span, element_type: TSType<'a>) -> TSType<'a> {
        TSType::TSArrayType(self.alloc_ts_array_type(span, element_type))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSConditionalType(self.alloc_ts_conditional_type(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_conditional_type_with_scope_id(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
    ) -> TSType<'a> {
        TSType::TSConditionalType(self.alloc_ts_conditional_type_with_scope_id(
            span,
            check_type,
            extends_type,
            true_type,
            false_type,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSConstructorType(self.alloc_ts_constructor_type(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_constructor_type_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSConstructorType(self.alloc_ts_constructor_type_with_scope_id(
            span,
            r#abstract,
            type_parameters,
            params,
            return_type,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSFunctionType(self.alloc_ts_function_type(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_function_type_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSType::TSFunctionType(self.alloc_ts_function_type_with_scope_id(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_import_type<T1, T2>(
        self,
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSImportType(self.alloc_ts_import_type(
            span,
            source,
            options,
            qualifier,
            type_arguments,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSIndexedAccessType(self.alloc_ts_indexed_access_type(
            span,
            object_type,
            index_type,
        ))
    }

    /// Build a [`TSType::TSInferType`].
    ///
    /// This node contains a [`TSInferType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSType<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        TSType::TSInferType(self.alloc_ts_infer_type(span, type_parameter))
    }

    /// Build a [`TSType::TSIntersectionType`].
    ///
    /// This node contains a [`TSIntersectionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_intersection_type(
        self,
        span: Span,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> TSType<'a> {
        TSType::TSIntersectionType(self.alloc_ts_intersection_type(span, types))
    }

    /// Build a [`TSType::TSLiteralType`].
    ///
    /// This node contains a [`TSLiteralType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `literal`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_literal_type(self, span: Span, literal: TSLiteral<'a>) -> TSType<'a> {
        TSType::TSLiteralType(self.alloc_ts_literal_type(span, literal))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_mapped_type(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
    ) -> TSType<'a> {
        TSType::TSMappedType(self.alloc_ts_mapped_type(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_mapped_type_with_scope_id(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
    ) -> TSType<'a> {
        TSType::TSMappedType(self.alloc_ts_mapped_type_with_scope_id(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
            scope_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_named_tuple_member(
        self,
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
    ) -> TSType<'a> {
        TSType::TSNamedTupleMember(self.alloc_ts_named_tuple_member(
            span,
            label,
            element_type,
            optional,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_template_literal_type(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> TSType<'a> {
        TSType::TSTemplateLiteralType(self.alloc_ts_template_literal_type(span, quasis, types))
    }

    /// Build a [`TSType::TSThisType`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_this_type(self, span: Span) -> TSType<'a> {
        TSType::TSThisType(self.alloc_ts_this_type(span))
    }

    /// Build a [`TSType::TSTupleType`].
    ///
    /// This node contains a [`TSTupleType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_tuple_type(
        self,
        span: Span,
        element_types: ArenaVec<'a, TSTupleElement<'a>>,
    ) -> TSType<'a> {
        TSType::TSTupleType(self.alloc_ts_tuple_type(span, element_types))
    }

    /// Build a [`TSType::TSTypeLiteral`].
    ///
    /// This node contains a [`TSTypeLiteral`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_type_literal(
        self,
        span: Span,
        members: ArenaVec<'a, TSSignature<'a>>,
    ) -> TSType<'a> {
        TSType::TSTypeLiteral(self.alloc_ts_type_literal(span, members))
    }

    /// Build a [`TSType::TSTypeOperatorType`].
    ///
    /// This node contains a [`TSTypeOperator`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_type_operator_type(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSType<'a> {
        TSType::TSTypeOperatorType(self.alloc_ts_type_operator(span, operator, type_annotation))
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSType::TSTypePredicate(self.alloc_ts_type_predicate(
            span,
            parameter_name,
            asserts,
            type_annotation,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSTypeQuery(self.alloc_ts_type_query(span, expr_name, type_arguments))
    }

    /// Build a [`TSType::TSTypeReference`].
    ///
    /// This node contains a [`TSTypeReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
    ) -> TSType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSType::TSTypeReference(self.alloc_ts_type_reference(span, type_name, type_arguments))
    }

    /// Build a [`TSType::TSUnionType`].
    ///
    /// This node contains a [`TSUnionType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_union_type(self, span: Span, types: ArenaVec<'a, TSType<'a>>) -> TSType<'a> {
        TSType::TSUnionType(self.alloc_ts_union_type(span, types))
    }

    /// Build a [`TSType::TSParenthesizedType`].
    ///
    /// This node contains a [`TSParenthesizedType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_parenthesized_type(self, span: Span, type_annotation: TSType<'a>) -> TSType<'a> {
        TSType::TSParenthesizedType(self.alloc_ts_parenthesized_type(span, type_annotation))
    }

    /// Build a [`TSType::JSDocNullableType`].
    ///
    /// This node contains a [`JSDocNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNullableType(self.alloc_js_doc_nullable_type(span, type_annotation, postfix))
    }

    /// Build a [`TSType::JSDocNonNullableType`].
    ///
    /// This node contains a [`JSDocNonNullableType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> TSType<'a> {
        TSType::JSDocNonNullableType(self.alloc_js_doc_non_nullable_type(
            span,
            type_annotation,
            postfix,
        ))
    }

    /// Build a [`TSType::JSDocUnknownType`].
    ///
    /// This node contains a [`JSDocUnknownType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_js_doc_unknown_type(self, span: Span) -> TSType<'a> {
        TSType::JSDocUnknownType(self.alloc_js_doc_unknown_type(span))
    }

    /// Build a [`TSConditionalType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_conditional_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_conditional_type(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
    ) -> ArenaBox<'a, TSConditionalType<'a>> {
        ArenaBox::new_in(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type),
            &self,
        )
    }

    /// Build a [`TSConditionalType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_conditional_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_conditional_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `check_type`: The type before `extends` in the test expression.
    /// * `extends_type`: The type `check_type` is being tested against.
    /// * `true_type`: The type evaluated to if the test is true.
    /// * `false_type`: The type evaluated to if the test is false.
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_conditional_type_with_scope_id(
        self,
        span: Span,
        check_type: TSType<'a>,
        extends_type: TSType<'a>,
        true_type: TSType<'a>,
        false_type: TSType<'a>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSConditionalType<'a>> {
        ArenaBox::new_in(
            self.ts_conditional_type_with_scope_id(
                span,
                check_type,
                extends_type,
                true_type,
                false_type,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSUnionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_union_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_union_type(self, span: Span, types: ArenaVec<'a, TSType<'a>>) -> TSUnionType<'a> {
        TSUnionType { node_id: Default::default(), span, types }
    }

    /// Build a [`TSUnionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_union_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`: The types in the union.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_union_type(
        self,
        span: Span,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> ArenaBox<'a, TSUnionType<'a>> {
        ArenaBox::new_in(self.ts_union_type(span, types), &self)
    }

    /// Build a [`TSIntersectionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_intersection_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_intersection_type(
        self,
        span: Span,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> TSIntersectionType<'a> {
        TSIntersectionType { node_id: Default::default(), span, types }
    }

    /// Build a [`TSIntersectionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_intersection_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_intersection_type(
        self,
        span: Span,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> ArenaBox<'a, TSIntersectionType<'a>> {
        ArenaBox::new_in(self.ts_intersection_type(span, types), &self)
    }

    /// Build a [`TSParenthesizedType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_parenthesized_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSParenthesizedType<'a> {
        TSParenthesizedType { node_id: Default::default(), span, type_annotation }
    }

    /// Build a [`TSParenthesizedType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_parenthesized_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_parenthesized_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSParenthesizedType<'a>> {
        ArenaBox::new_in(self.ts_parenthesized_type(span, type_annotation), &self)
    }

    /// Build a [`TSTypeOperator`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> TSTypeOperator<'a> {
        TSTypeOperator { node_id: Default::default(), span, operator, type_annotation }
    }

    /// Build a [`TSTypeOperator`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_operator`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `operator`
    /// * `type_annotation`: The type being operated on
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_operator(
        self,
        span: Span,
        operator: TSTypeOperatorOperator,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSTypeOperator<'a>> {
        ArenaBox::new_in(self.ts_type_operator(span, operator, type_annotation), &self)
    }

    /// Build a [`TSArrayType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_array_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_array_type(self, span: Span, element_type: TSType<'a>) -> TSArrayType<'a> {
        TSArrayType { node_id: Default::default(), span, element_type }
    }

    /// Build a [`TSArrayType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_array_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_array_type(
        self,
        span: Span,
        element_type: TSType<'a>,
    ) -> ArenaBox<'a, TSArrayType<'a>> {
        ArenaBox::new_in(self.ts_array_type(span, element_type), &self)
    }

    /// Build a [`TSIndexedAccessType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_indexed_access_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> TSIndexedAccessType<'a> {
        TSIndexedAccessType { node_id: Default::default(), span, object_type, index_type }
    }

    /// Build a [`TSIndexedAccessType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_indexed_access_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `object_type`
    /// * `index_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_indexed_access_type(
        self,
        span: Span,
        object_type: TSType<'a>,
        index_type: TSType<'a>,
    ) -> ArenaBox<'a, TSIndexedAccessType<'a>> {
        ArenaBox::new_in(self.ts_indexed_access_type(span, object_type, index_type), &self)
    }

    /// Build a [`TSTupleType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_tuple_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_tuple_type(
        self,
        span: Span,
        element_types: ArenaVec<'a, TSTupleElement<'a>>,
    ) -> TSTupleType<'a> {
        TSTupleType { node_id: Default::default(), span, element_types }
    }

    /// Build a [`TSTupleType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_tuple_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `element_types`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_tuple_type(
        self,
        span: Span,
        element_types: ArenaVec<'a, TSTupleElement<'a>>,
    ) -> ArenaBox<'a, TSTupleType<'a>> {
        ArenaBox::new_in(self.ts_tuple_type(span, element_types), &self)
    }

    /// Build a [`TSNamedTupleMember`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_named_tuple_member`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_named_tuple_member(
        self,
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
    ) -> TSNamedTupleMember<'a> {
        TSNamedTupleMember { node_id: Default::default(), span, label, element_type, optional }
    }

    /// Build a [`TSNamedTupleMember`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_named_tuple_member`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `label`
    /// * `element_type`
    /// * `optional`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_named_tuple_member(
        self,
        span: Span,
        label: IdentifierName<'a>,
        element_type: TSTupleElement<'a>,
        optional: bool,
    ) -> ArenaBox<'a, TSNamedTupleMember<'a>> {
        ArenaBox::new_in(self.ts_named_tuple_member(span, label, element_type, optional), &self)
    }

    /// Build a [`TSOptionalType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_optional_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_optional_type(self, span: Span, type_annotation: TSType<'a>) -> TSOptionalType<'a> {
        TSOptionalType { node_id: Default::default(), span, type_annotation }
    }

    /// Build a [`TSOptionalType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_optional_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSOptionalType<'a>> {
        ArenaBox::new_in(self.ts_optional_type(span, type_annotation), &self)
    }

    /// Build a [`TSRestType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_rest_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_rest_type(self, span: Span, type_annotation: TSType<'a>) -> TSRestType<'a> {
        TSRestType { node_id: Default::default(), span, type_annotation }
    }

    /// Build a [`TSRestType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_rest_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSRestType<'a>> {
        ArenaBox::new_in(self.ts_rest_type(span, type_annotation), &self)
    }

    /// Build a [`TSTupleElement::TSOptionalType`].
    ///
    /// This node contains a [`TSOptionalType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_tuple_element_optional_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSOptionalType(self.alloc_ts_optional_type(span, type_annotation))
    }

    /// Build a [`TSTupleElement::TSRestType`].
    ///
    /// This node contains a [`TSRestType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_tuple_element_rest_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
    ) -> TSTupleElement<'a> {
        TSTupleElement::TSRestType(self.alloc_ts_rest_type(span, type_annotation))
    }

    /// Build a [`TSAnyKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_any_keyword(self, span: Span) -> TSAnyKeyword {
        TSAnyKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSAnyKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_any_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_any_keyword(self, span: Span) -> ArenaBox<'a, TSAnyKeyword> {
        ArenaBox::new_in(self.ts_any_keyword(span), &self)
    }

    /// Build a [`TSStringKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_string_keyword(self, span: Span) -> TSStringKeyword {
        TSStringKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSStringKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_string_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_string_keyword(self, span: Span) -> ArenaBox<'a, TSStringKeyword> {
        ArenaBox::new_in(self.ts_string_keyword(span), &self)
    }

    /// Build a [`TSBooleanKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_boolean_keyword(self, span: Span) -> TSBooleanKeyword {
        TSBooleanKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSBooleanKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_boolean_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_boolean_keyword(self, span: Span) -> ArenaBox<'a, TSBooleanKeyword> {
        ArenaBox::new_in(self.ts_boolean_keyword(span), &self)
    }

    /// Build a [`TSNumberKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_number_keyword(self, span: Span) -> TSNumberKeyword {
        TSNumberKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSNumberKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_number_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_number_keyword(self, span: Span) -> ArenaBox<'a, TSNumberKeyword> {
        ArenaBox::new_in(self.ts_number_keyword(span), &self)
    }

    /// Build a [`TSNeverKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_never_keyword(self, span: Span) -> TSNeverKeyword {
        TSNeverKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSNeverKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_never_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_never_keyword(self, span: Span) -> ArenaBox<'a, TSNeverKeyword> {
        ArenaBox::new_in(self.ts_never_keyword(span), &self)
    }

    /// Build a [`TSIntrinsicKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_intrinsic_keyword(self, span: Span) -> TSIntrinsicKeyword {
        TSIntrinsicKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSIntrinsicKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_intrinsic_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_intrinsic_keyword(self, span: Span) -> ArenaBox<'a, TSIntrinsicKeyword> {
        ArenaBox::new_in(self.ts_intrinsic_keyword(span), &self)
    }

    /// Build a [`TSUnknownKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_unknown_keyword(self, span: Span) -> TSUnknownKeyword {
        TSUnknownKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSUnknownKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_unknown_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_unknown_keyword(self, span: Span) -> ArenaBox<'a, TSUnknownKeyword> {
        ArenaBox::new_in(self.ts_unknown_keyword(span), &self)
    }

    /// Build a [`TSNullKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_null_keyword(self, span: Span) -> TSNullKeyword {
        TSNullKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSNullKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_null_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_null_keyword(self, span: Span) -> ArenaBox<'a, TSNullKeyword> {
        ArenaBox::new_in(self.ts_null_keyword(span), &self)
    }

    /// Build a [`TSUndefinedKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_undefined_keyword(self, span: Span) -> TSUndefinedKeyword {
        TSUndefinedKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSUndefinedKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_undefined_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_undefined_keyword(self, span: Span) -> ArenaBox<'a, TSUndefinedKeyword> {
        ArenaBox::new_in(self.ts_undefined_keyword(span), &self)
    }

    /// Build a [`TSVoidKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_void_keyword(self, span: Span) -> TSVoidKeyword {
        TSVoidKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSVoidKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_void_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_void_keyword(self, span: Span) -> ArenaBox<'a, TSVoidKeyword> {
        ArenaBox::new_in(self.ts_void_keyword(span), &self)
    }

    /// Build a [`TSSymbolKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_symbol_keyword(self, span: Span) -> TSSymbolKeyword {
        TSSymbolKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSSymbolKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_symbol_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_symbol_keyword(self, span: Span) -> ArenaBox<'a, TSSymbolKeyword> {
        ArenaBox::new_in(self.ts_symbol_keyword(span), &self)
    }

    /// Build a [`TSThisType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_this_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_this_type(self, span: Span) -> TSThisType {
        TSThisType { node_id: Default::default(), span }
    }

    /// Build a [`TSThisType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_this_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_this_type(self, span: Span) -> ArenaBox<'a, TSThisType> {
        ArenaBox::new_in(self.ts_this_type(span), &self)
    }

    /// Build a [`TSObjectKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_object_keyword(self, span: Span) -> TSObjectKeyword {
        TSObjectKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSObjectKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_object_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_object_keyword(self, span: Span) -> ArenaBox<'a, TSObjectKeyword> {
        ArenaBox::new_in(self.ts_object_keyword(span), &self)
    }

    /// Build a [`TSBigIntKeyword`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_big_int_keyword(self, span: Span) -> TSBigIntKeyword {
        TSBigIntKeyword { node_id: Default::default(), span }
    }

    /// Build a [`TSBigIntKeyword`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_big_int_keyword`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_big_int_keyword(self, span: Span) -> ArenaBox<'a, TSBigIntKeyword> {
        ArenaBox::new_in(self.ts_big_int_keyword(span), &self)
    }

    /// Build a [`TSTypeReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
    ) -> TSTypeReference<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeReference {
            node_id: Default::default(),
            span,
            type_name,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSTypeReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_name`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_reference<T1>(
        self,
        span: Span,
        type_name: TSTypeName<'a>,
        type_arguments: T1,
    ) -> ArenaBox<'a, TSTypeReference<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(self.ts_type_reference(span, type_name, type_arguments), &self)
    }

    /// Build a [`TSTypeName::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_name_identifier_reference<S1>(self, span: Span, name: S1) -> TSTypeName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSTypeName::IdentifierReference(self.alloc_identifier_reference(span, name))
    }

    /// Build a [`TSTypeName::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_name_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> TSTypeName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSTypeName::IdentifierReference(self.alloc_identifier_reference_with_reference_id(
            span,
            name,
            reference_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_name_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSTypeName<'a> {
        TSTypeName::QualifiedName(self.alloc_ts_qualified_name(span, left, right))
    }

    /// Build a [`TSTypeName::ThisExpression`].
    ///
    /// This node contains a [`ThisExpression`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_name_this_expression(self, span: Span) -> TSTypeName<'a> {
        TSTypeName::ThisExpression(self.alloc_this_expression(span))
    }

    /// Build a [`TSQualifiedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_qualified_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSQualifiedName<'a> {
        TSQualifiedName { node_id: Default::default(), span, left, right }
    }

    /// Build a [`TSQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_qualified_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> ArenaBox<'a, TSQualifiedName<'a>> {
        ArenaBox::new_in(self.ts_qualified_name(span, left, right), &self)
    }

    /// Build a [`TSTypeParameterInstantiation`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_parameter_instantiation`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_parameter_instantiation(
        self,
        span: Span,
        params: ArenaVec<'a, TSType<'a>>,
    ) -> TSTypeParameterInstantiation<'a> {
        TSTypeParameterInstantiation { node_id: Default::default(), span, params }
    }

    /// Build a [`TSTypeParameterInstantiation`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter_instantiation`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_parameter_instantiation(
        self,
        span: Span,
        params: ArenaVec<'a, TSType<'a>>,
    ) -> ArenaBox<'a, TSTypeParameterInstantiation<'a>> {
        ArenaBox::new_in(self.ts_type_parameter_instantiation(span, params), &self)
    }

    /// Build a [`TSTypeParameter`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// * `constraint`: Constrains what types can be passed to the type parameter.
    /// * `default`: Default value of the type parameter if no type is provided when using the type.
    /// * `in`: Was an `in` modifier keyword present?
    /// * `out`: Was an `out` modifier keyword present?
    /// * `const`: Was a `const` modifier keyword present?
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the parameter, e.g. `T` in `type Foo<T> = ...`.
    /// * `constraint`: Constrains what types can be passed to the type parameter.
    /// * `default`: Default value of the type parameter if no type is provided when using the type.
    /// * `in`: Was an `in` modifier keyword present?
    /// * `out`: Was an `out` modifier keyword present?
    /// * `const`: Was a `const` modifier keyword present?
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
    ) -> ArenaBox<'a, TSTypeParameter<'a>> {
        ArenaBox::new_in(
            self.ts_type_parameter(span, name, constraint, default, r#in, out, r#const),
            &self,
        )
    }

    /// Build a [`TSTypeParameterDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_parameter_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_parameter_declaration(
        self,
        span: Span,
        params: ArenaVec<'a, TSTypeParameter<'a>>,
    ) -> TSTypeParameterDeclaration<'a> {
        TSTypeParameterDeclaration { node_id: Default::default(), span, params }
    }

    /// Build a [`TSTypeParameterDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_parameter_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `params`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_parameter_declaration(
        self,
        span: Span,
        params: ArenaVec<'a, TSTypeParameter<'a>>,
    ) -> ArenaBox<'a, TSTypeParameterDeclaration<'a>> {
        ArenaBox::new_in(self.ts_type_parameter_declaration(span, params), &self)
    }

    /// Build a [`TSTypeAliasDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSTypeAliasDeclaration {
            node_id: Default::default(),
            span,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            type_annotation,
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSTypeAliasDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_alias_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_alias_declaration<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
    ) -> ArenaBox<'a, TSTypeAliasDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare),
            &self,
        )
    }

    /// Build a [`TSTypeAliasDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_alias_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        TSTypeAliasDeclaration {
            node_id: Default::default(),
            span,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            type_annotation,
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSTypeAliasDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_alias_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: Type alias's identifier, e.g. `Foo` in `type Foo = number`.
    /// * `type_parameters`
    /// * `type_annotation`
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_alias_declaration_with_scope_id<T1>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        type_annotation: TSType<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSTypeAliasDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_type_alias_declaration_with_scope_id(
                span,
                id,
                type_parameters,
                type_annotation,
                declare,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSClassImplements`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_class_implements<T1>(
        self,
        span: Span,
        expression: TSTypeName<'a>,
        type_arguments: T1,
    ) -> TSClassImplements<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSClassImplements {
            node_id: Default::default(),
            span,
            expression,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSInterfaceDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
    ) -> TSInterfaceDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        TSInterfaceDeclaration {
            node_id: Default::default(),
            span,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            extends,
            body: body.into_in(self.allocator()),
            declare,
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSInterfaceDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_interface_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_interface_declaration<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
    ) -> ArenaBox<'a, TSInterfaceDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare),
            &self,
        )
    }

    /// Build a [`TSInterfaceDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_interface_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_interface_declaration_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
    ) -> TSInterfaceDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        TSInterfaceDeclaration {
            node_id: Default::default(),
            span,
            id,
            type_parameters: type_parameters.into_in(self.allocator()),
            extends,
            body: body.into_in(self.allocator()),
            declare,
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSInterfaceDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_interface_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The identifier (name) of the interface.
    /// * `type_parameters`: Type parameters that get bound to the interface.
    /// * `extends`: Other interfaces/types this interface extends.
    /// * `body`
    /// * `declare`: `true` for `declare interface Foo {}`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_interface_declaration_with_scope_id<T1, T2>(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        type_parameters: T1,
        extends: ArenaVec<'a, TSInterfaceHeritage<'a>>,
        body: T2,
        declare: bool,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSInterfaceDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, TSInterfaceBody<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_interface_declaration_with_scope_id(
                span,
                id,
                type_parameters,
                extends,
                body,
                declare,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSInterfaceBody`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_interface_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_interface_body(
        self,
        span: Span,
        body: ArenaVec<'a, TSSignature<'a>>,
    ) -> TSInterfaceBody<'a> {
        TSInterfaceBody { node_id: Default::default(), span, body }
    }

    /// Build a [`TSInterfaceBody`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_interface_body`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_interface_body(
        self,
        span: Span,
        body: ArenaVec<'a, TSSignature<'a>>,
    ) -> ArenaBox<'a, TSInterfaceBody<'a>> {
        ArenaBox::new_in(self.ts_interface_body(span, body), &self)
    }

    /// Build a [`TSPropertySignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_property_signature`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `computed`
    /// * `optional`
    /// * `readonly`
    /// * `key`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSPropertySignature {
            node_id: Default::default(),
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`TSPropertySignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_property_signature`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `computed`
    /// * `optional`
    /// * `readonly`
    /// * `key`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_property_signature<T1>(
        self,
        span: Span,
        computed: bool,
        optional: bool,
        readonly: bool,
        key: PropertyKey<'a>,
        type_annotation: T1,
    ) -> ArenaBox<'a, TSPropertySignature<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_property_signature(span, computed, optional, readonly, key, type_annotation),
            &self,
        )
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_index_signature<T1>(
        self,
        span: Span,
        parameters: ArenaVec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSSignature::TSIndexSignature(self.alloc_ts_index_signature(
            span,
            parameters,
            type_annotation,
            readonly,
            r#static,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSPropertySignature(self.alloc_ts_property_signature(
            span,
            computed,
            optional,
            readonly,
            key,
            type_annotation,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_call_signature_declaration<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSCallSignatureDeclaration(self.alloc_ts_call_signature_declaration(
            span,
            type_parameters,
            this_param,
            params,
            return_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_call_signature_declaration_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSCallSignatureDeclaration(
            self.alloc_ts_call_signature_declaration_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
            ),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSConstructSignatureDeclaration(self.alloc_ts_construct_signature_declaration(
            span,
            type_parameters,
            params,
            return_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_construct_signature_declaration_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSConstructSignatureDeclaration(
            self.alloc_ts_construct_signature_declaration_with_scope_id(
                span,
                type_parameters,
                params,
                return_type,
                scope_id,
            ),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSMethodSignature(self.alloc_ts_method_signature(
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters,
            this_param,
            params,
            return_type,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_signature_method_signature_with_scope_id<T1, T2, T3, T4>(
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
    ) -> TSSignature<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSSignature::TSMethodSignature(self.alloc_ts_method_signature_with_scope_id(
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
        ))
    }

    /// Build a [`TSIndexSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_index_signature<T1>(
        self,
        span: Span,
        parameters: ArenaVec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> TSIndexSignature<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSIndexSignature {
            node_id: Default::default(),
            span,
            parameters,
            type_annotation: type_annotation.into_in(self.allocator()),
            readonly,
            r#static,
        }
    }

    /// Build a [`TSIndexSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_index_signature`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameters`
    /// * `type_annotation`
    /// * `readonly`
    /// * `static`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_index_signature<T1>(
        self,
        span: Span,
        parameters: ArenaVec<'a, TSIndexSignatureName<'a>>,
        type_annotation: T1,
        readonly: bool,
        r#static: bool,
    ) -> ArenaBox<'a, TSIndexSignature<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static),
            &self,
        )
    }

    /// Build a [`TSCallSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_call_signature_declaration<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> TSCallSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSCallSignatureDeclaration {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSCallSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_call_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_call_signature_declaration<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> ArenaBox<'a, TSCallSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_call_signature_declaration(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
            ),
            &self,
        )
    }

    /// Build a [`TSCallSignatureDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_call_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_call_signature_declaration_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> TSCallSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSCallSignatureDeclaration {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSCallSignatureDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_call_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `this_param`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_call_signature_declaration_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSCallSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_call_signature_declaration_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSMethodSignature`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_method_signature`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSMethodSignature {
            node_id: Default::default(),
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSMethodSignature`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_method_signature`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
    ) -> ArenaBox<'a, TSMethodSignature<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
            &self,
        )
    }

    /// Build a [`TSMethodSignature`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_method_signature_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSMethodSignature {
            node_id: Default::default(),
            span,
            key,
            computed,
            optional,
            kind,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSMethodSignature`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_method_signature_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
    ) -> ArenaBox<'a, TSMethodSignature<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
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
            &self,
        )
    }

    /// Build a [`TSConstructSignatureDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> TSConstructSignatureDeclaration<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSConstructSignatureDeclaration {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_construct_signature_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_construct_signature_declaration<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> ArenaBox<'a, TSConstructSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_construct_signature_declaration(span, type_parameters, params, return_type),
            &self,
        )
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_construct_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSConstructSignatureDeclaration {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConstructSignatureDeclaration`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_construct_signature_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_construct_signature_declaration_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSConstructSignatureDeclaration<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_construct_signature_declaration_with_scope_id(
                span,
                type_parameters,
                params,
                return_type,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSIndexSignatureName`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_index_signature_name<S1, T1>(
        self,
        span: Span,
        name: S1,
        type_annotation: T1,
    ) -> TSIndexSignatureName<'a>
    where
        S1: Into<Str<'a>>,
        T1: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSIndexSignatureName {
            node_id: Default::default(),
            span,
            name: name.into(),
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`TSInterfaceHeritage`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_interface_heritage<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
    ) -> TSInterfaceHeritage<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSInterfaceHeritage {
            node_id: Default::default(),
            span,
            expression,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSTypePredicate`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> TSTypePredicate<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        TSTypePredicate {
            node_id: Default::default(),
            span,
            parameter_name,
            asserts,
            type_annotation: type_annotation.into_in(self.allocator()),
        }
    }

    /// Build a [`TSTypePredicate`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_predicate`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `parameter_name`: The identifier the predicate operates on
    /// * `asserts`: Does this predicate include an `asserts` modifier?
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_predicate<T1>(
        self,
        span: Span,
        parameter_name: TSTypePredicateName<'a>,
        asserts: bool,
        type_annotation: T1,
    ) -> ArenaBox<'a, TSTypePredicate<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeAnnotation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_type_predicate(span, parameter_name, asserts, type_annotation),
            &self,
        )
    }

    /// Build a [`TSTypePredicateName::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_predicate_name_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> TSTypePredicateName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSTypePredicateName::Identifier(self.alloc_identifier_name(span, name))
    }

    /// Build a [`TSTypePredicateName::This`].
    ///
    /// This node contains a [`TSThisType`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_predicate_name_this(self, span: Span) -> TSTypePredicateName<'a> {
        TSTypePredicateName::This(self.alloc_ts_this_type(span))
    }

    /// Build a [`TSModuleDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> TSModuleDeclaration<'a> {
        TSModuleDeclaration {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_module_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_module_declaration(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
    ) -> ArenaBox<'a, TSModuleDeclaration<'a>> {
        ArenaBox::new_in(self.ts_module_declaration(span, id, body, kind, declare), &self)
    }

    /// Build a [`TSModuleDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_module_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        TSModuleDeclaration {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_module_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The name of the module/namespace being declared.
    /// * `body`
    /// * `kind`: The keyword used to define this module declaration.
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_module_declaration_with_scope_id(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSModuleDeclaration<'a>> {
        ArenaBox::new_in(
            self.ts_module_declaration_with_scope_id(span, id, body, kind, declare, scope_id),
            &self,
        )
    }

    /// Build a [`TSModuleDeclarationName::Identifier`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_name_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> TSModuleDeclarationName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSModuleDeclarationName::Identifier(self.binding_identifier(span, name))
    }

    /// Build a [`TSModuleDeclarationName::Identifier`] with `symbol_id`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The identifier name being bound.
    /// * `symbol_id`: Unique identifier for this binding.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_name_identifier_with_symbol_id<S1>(
        self,
        span: Span,
        name: S1,
        symbol_id: SymbolId,
    ) -> TSModuleDeclarationName<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSModuleDeclarationName::Identifier(
            self.binding_identifier_with_symbol_id(span, name, symbol_id),
        )
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`].
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_name_string_literal<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
    ) -> TSModuleDeclarationName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSModuleDeclarationName::StringLiteral(self.string_literal(span, value, raw))
    }

    /// Build a [`TSModuleDeclarationName::StringLiteral`] with `lone_surrogates`.
    ///
    /// ## Parameters
    /// * `span`: Node location in source code.
    /// * `value`: The value of the string.
    /// * `raw`: The raw string as it appears in source code.
    /// * `lone_surrogates`: The string value contains lone surrogates.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_name_string_literal_with_lone_surrogates<S1>(
        self,
        span: Span,
        value: S1,
        raw: Option<Str<'a>>,
        lone_surrogates: bool,
    ) -> TSModuleDeclarationName<'a>
    where
        S1: Into<Str<'a>>,
    {
        TSModuleDeclarationName::StringLiteral(self.string_literal_with_lone_surrogates(
            span,
            value,
            raw,
            lone_surrogates,
        ))
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
            self.alloc_ts_module_declaration(span, id, body, kind, declare),
        )
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_body_module_declaration_with_scope_id(
        self,
        span: Span,
        id: TSModuleDeclarationName<'a>,
        body: Option<TSModuleDeclarationBody<'a>>,
        kind: TSModuleDeclarationKind,
        declare: bool,
        scope_id: ScopeId,
    ) -> TSModuleDeclarationBody<'a> {
        TSModuleDeclarationBody::TSModuleDeclaration(
            self.alloc_ts_module_declaration_with_scope_id(span, id, body, kind, declare, scope_id),
        )
    }

    /// Build a [`TSModuleDeclarationBody::TSModuleBlock`].
    ///
    /// This node contains a [`TSModuleBlock`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_declaration_body_module_block(
        self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> TSModuleDeclarationBody<'a> {
        TSModuleDeclarationBody::TSModuleBlock(self.alloc_ts_module_block(span, directives, body))
    }

    /// Build a [`TSGlobalDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_global_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_global_declaration(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
    ) -> TSGlobalDeclaration<'a> {
        TSGlobalDeclaration {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_global_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_global_declaration(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
    ) -> ArenaBox<'a, TSGlobalDeclaration<'a>> {
        ArenaBox::new_in(self.ts_global_declaration(span, global_span, body, declare), &self)
    }

    /// Build a [`TSGlobalDeclaration`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_global_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_global_declaration_with_scope_id(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> TSGlobalDeclaration<'a> {
        TSGlobalDeclaration {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_global_declaration_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `global_span`: Span of `global` keyword
    /// * `body`
    /// * `declare`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_global_declaration_with_scope_id(
        self,
        span: Span,
        global_span: Span,
        body: TSModuleBlock<'a>,
        declare: bool,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSGlobalDeclaration<'a>> {
        ArenaBox::new_in(
            self.ts_global_declaration_with_scope_id(span, global_span, body, declare, scope_id),
            &self,
        )
    }

    /// Build a [`TSModuleBlock`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_module_block`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_block(
        self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> TSModuleBlock<'a> {
        TSModuleBlock { node_id: Default::default(), span, directives, body }
    }

    /// Build a [`TSModuleBlock`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_module_block`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `directives`
    /// * `body`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_module_block(
        self,
        span: Span,
        directives: ArenaVec<'a, Directive<'a>>,
        body: ArenaVec<'a, Statement<'a>>,
    ) -> ArenaBox<'a, TSModuleBlock<'a>> {
        ArenaBox::new_in(self.ts_module_block(span, directives, body), &self)
    }

    /// Build a [`TSTypeLiteral`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_literal(
        self,
        span: Span,
        members: ArenaVec<'a, TSSignature<'a>>,
    ) -> TSTypeLiteral<'a> {
        TSTypeLiteral { node_id: Default::default(), span, members }
    }

    /// Build a [`TSTypeLiteral`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_literal`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `members`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_literal(
        self,
        span: Span,
        members: ArenaVec<'a, TSSignature<'a>>,
    ) -> ArenaBox<'a, TSTypeLiteral<'a>> {
        ArenaBox::new_in(self.ts_type_literal(span, members), &self)
    }

    /// Build a [`TSInferType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_infer_type<T1>(self, span: Span, type_parameter: T1) -> TSInferType<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        TSInferType {
            node_id: Default::default(),
            span,
            type_parameter: type_parameter.into_in(self.allocator()),
        }
    }

    /// Build a [`TSInferType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_infer_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameter`: The type bound when the
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_infer_type<T1>(
        self,
        span: Span,
        type_parameter: T1,
    ) -> ArenaBox<'a, TSInferType<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameter<'a>>>,
    {
        ArenaBox::new_in(self.ts_infer_type(span, type_parameter), &self)
    }

    /// Build a [`TSTypeQuery`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_query`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
    ) -> TSTypeQuery<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeQuery {
            node_id: Default::default(),
            span,
            expr_name,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSTypeQuery`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_query`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expr_name`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_query<T1>(
        self,
        span: Span,
        expr_name: TSTypeQueryExprName<'a>,
        type_arguments: T1,
    ) -> ArenaBox<'a, TSTypeQuery<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(self.ts_type_query(span, expr_name, type_arguments), &self)
    }

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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_query_expr_name_import_type<T1, T2>(
        self,
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
    ) -> TSTypeQueryExprName<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSTypeQueryExprName::TSImportType(self.alloc_ts_import_type(
            span,
            source,
            options,
            qualifier,
            type_arguments,
        ))
    }

    /// Build a [`TSImportType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_import_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_import_type<T1, T2>(
        self,
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
    ) -> TSImportType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        TSImportType {
            node_id: Default::default(),
            span,
            source,
            options: options.into_in(self.allocator()),
            qualifier,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSImportType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_import_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `source`
    /// * `options`
    /// * `qualifier`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_import_type<T1, T2>(
        self,
        span: Span,
        source: StringLiteral<'a>,
        options: T1,
        qualifier: Option<TSImportTypeQualifier<'a>>,
        type_arguments: T2,
    ) -> ArenaBox<'a, TSImportType<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, ObjectExpression<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>>,
    {
        ArenaBox::new_in(
            self.ts_import_type(span, source, options, qualifier, type_arguments),
            &self,
        )
    }

    /// Build a [`TSImportTypeQualifier::Identifier`].
    ///
    /// This node contains an [`IdentifierName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_import_type_qualifier_identifier<S1>(
        self,
        span: Span,
        name: S1,
    ) -> TSImportTypeQualifier<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSImportTypeQualifier::Identifier(self.alloc_identifier_name(span, name))
    }

    /// Build a [`TSImportTypeQualifier::QualifiedName`].
    ///
    /// This node contains a [`TSImportTypeQualifiedName`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_import_type_qualifier_qualified_name(
        self,
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
    ) -> TSImportTypeQualifier<'a> {
        TSImportTypeQualifier::QualifiedName(
            self.alloc_ts_import_type_qualified_name(span, left, right),
        )
    }

    /// Build a [`TSImportTypeQualifiedName`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_import_type_qualified_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_import_type_qualified_name(
        self,
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
    ) -> TSImportTypeQualifiedName<'a> {
        TSImportTypeQualifiedName { node_id: Default::default(), span, left, right }
    }

    /// Build a [`TSImportTypeQualifiedName`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_import_type_qualified_name`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `left`
    /// * `right`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_import_type_qualified_name(
        self,
        span: Span,
        left: TSImportTypeQualifier<'a>,
        right: IdentifierName<'a>,
    ) -> ArenaBox<'a, TSImportTypeQualifiedName<'a>> {
        ArenaBox::new_in(self.ts_import_type_qualified_name(span, left, right), &self)
    }

    /// Build a [`TSFunctionType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_function_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSFunctionType {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSFunctionType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_function_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_function_type<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
    ) -> ArenaBox<'a, TSFunctionType<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_function_type(span, type_parameters, this_param, params, return_type),
            &self,
        )
    }

    /// Build a [`TSFunctionType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_function_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_function_type_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> TSFunctionType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSFunctionType {
            node_id: Default::default(),
            span,
            type_parameters: type_parameters.into_in(self.allocator()),
            this_param: this_param.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSFunctionType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_function_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_parameters`: Generic type parameters
    /// * `this_param`: `this` parameter
    /// * `params`: Function parameters. Akin to [`Function::params`].
    /// * `return_type`: Return type of the function.
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_function_type_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        type_parameters: T1,
        this_param: T2,
        params: T3,
        return_type: T4,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSFunctionType<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<ArenaBox<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_function_type_with_scope_id(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSConstructorType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
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
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSConstructorType {
            node_id: Default::default(),
            span,
            r#abstract,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Default::default(),
        }
    }

    /// Build a [`TSConstructorType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_constructor_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_constructor_type<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
    ) -> ArenaBox<'a, TSConstructorType<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_constructor_type(span, r#abstract, type_parameters, params, return_type),
            &self,
        )
    }

    /// Build a [`TSConstructorType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_constructor_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_constructor_type_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> TSConstructorType<'a>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        TSConstructorType {
            node_id: Default::default(),
            span,
            r#abstract,
            type_parameters: type_parameters.into_in(self.allocator()),
            params: params.into_in(self.allocator()),
            return_type: return_type.into_in(self.allocator()),
            scope_id: Cell::new(Some(scope_id)),
        }
    }

    /// Build a [`TSConstructorType`] with `scope_id`, and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_constructor_type_with_scope_id`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `abstract`
    /// * `type_parameters`
    /// * `params`
    /// * `return_type`
    /// * `scope_id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_constructor_type_with_scope_id<T1, T2, T3>(
        self,
        span: Span,
        r#abstract: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSConstructorType<'a>>
    where
        T1: IntoIn<'a, Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, ArenaBox<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, ArenaBox<'a, TSTypeAnnotation<'a>>>,
    {
        ArenaBox::new_in(
            self.ts_constructor_type_with_scope_id(
                span,
                r#abstract,
                type_parameters,
                params,
                return_type,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSMappedType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_mapped_type(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
    ) -> TSMappedType<'a> {
        TSMappedType {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_mapped_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `key`: Key type parameter, e.g. `P` in `[P in keyof T]`.
    /// * `constraint`: Constraint type, e.g. `keyof T` in `[P in keyof T]`.
    /// * `name_type`
    /// * `type_annotation`
    /// * `optional`: Optional modifier on type annotation
    /// * `readonly`: Readonly modifier before keyed index signature
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_mapped_type(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
    ) -> ArenaBox<'a, TSMappedType<'a>> {
        ArenaBox::new_in(
            self.ts_mapped_type(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
            ),
            &self,
        )
    }

    /// Build a [`TSMappedType`] with `scope_id`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_mapped_type_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_mapped_type_with_scope_id(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
    ) -> TSMappedType<'a> {
        TSMappedType {
            node_id: Default::default(),
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
    /// If you want a stack-allocated node, use [`AstBuilder::ts_mapped_type_with_scope_id`] instead.
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_mapped_type_with_scope_id(
        self,
        span: Span,
        key: BindingIdentifier<'a>,
        constraint: TSType<'a>,
        name_type: Option<TSType<'a>>,
        type_annotation: Option<TSType<'a>>,
        optional: Option<TSMappedTypeModifierOperator>,
        readonly: Option<TSMappedTypeModifierOperator>,
        scope_id: ScopeId,
    ) -> ArenaBox<'a, TSMappedType<'a>> {
        ArenaBox::new_in(
            self.ts_mapped_type_with_scope_id(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
                scope_id,
            ),
            &self,
        )
    }

    /// Build a [`TSTemplateLiteralType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_template_literal_type(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> TSTemplateLiteralType<'a> {
        TSTemplateLiteralType { node_id: Default::default(), span, quasis, types }
    }

    /// Build a [`TSTemplateLiteralType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_template_literal_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `quasis`: The string parts of the template literal.
    /// * `types`: The interpolated expressions in the template literal.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_template_literal_type(
        self,
        span: Span,
        quasis: ArenaVec<'a, TemplateElement<'a>>,
        types: ArenaVec<'a, TSType<'a>>,
    ) -> ArenaBox<'a, TSTemplateLiteralType<'a>> {
        ArenaBox::new_in(self.ts_template_literal_type(span, quasis, types), &self)
    }

    /// Build a [`TSAsExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_as_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSAsExpression<'a> {
        TSAsExpression { node_id: Default::default(), span, expression, type_annotation }
    }

    /// Build a [`TSAsExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_as_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_annotation`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_as_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSAsExpression<'a>> {
        ArenaBox::new_in(self.ts_as_expression(span, expression, type_annotation), &self)
    }

    /// Build a [`TSSatisfiesExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> TSSatisfiesExpression<'a> {
        TSSatisfiesExpression { node_id: Default::default(), span, expression, type_annotation }
    }

    /// Build a [`TSSatisfiesExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_satisfies_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`: The value expression being constrained.
    /// * `type_annotation`: The type `expression` must satisfy.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_satisfies_expression(
        self,
        span: Span,
        expression: Expression<'a>,
        type_annotation: TSType<'a>,
    ) -> ArenaBox<'a, TSSatisfiesExpression<'a>> {
        ArenaBox::new_in(self.ts_satisfies_expression(span, expression, type_annotation), &self)
    }

    /// Build a [`TSTypeAssertion`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_type_assertion`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_type_assertion(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> TSTypeAssertion<'a> {
        TSTypeAssertion { node_id: Default::default(), span, type_annotation, expression }
    }

    /// Build a [`TSTypeAssertion`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_type_assertion`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_type_assertion(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, TSTypeAssertion<'a>> {
        ArenaBox::new_in(self.ts_type_assertion(span, type_annotation, expression), &self)
    }

    /// Build a [`TSImportEqualsDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_import_equals_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> TSImportEqualsDeclaration<'a> {
        TSImportEqualsDeclaration {
            node_id: Default::default(),
            span,
            id,
            module_reference,
            import_kind,
        }
    }

    /// Build a [`TSImportEqualsDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_import_equals_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    /// * `module_reference`
    /// * `import_kind`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_import_equals_declaration(
        self,
        span: Span,
        id: BindingIdentifier<'a>,
        module_reference: TSModuleReference<'a>,
        import_kind: ImportOrExportKind,
    ) -> ArenaBox<'a, TSImportEqualsDeclaration<'a>> {
        ArenaBox::new_in(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind),
            &self,
        )
    }

    /// Build a [`TSModuleReference::ExternalModuleReference`].
    ///
    /// This node contains a [`TSExternalModuleReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_reference_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSModuleReference<'a> {
        TSModuleReference::ExternalModuleReference(
            self.alloc_ts_external_module_reference(span, expression),
        )
    }

    /// Build a [`TSModuleReference::IdentifierReference`].
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_reference_identifier_reference<S1>(
        self,
        span: Span,
        name: S1,
    ) -> TSModuleReference<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSModuleReference::IdentifierReference(self.alloc_identifier_reference(span, name))
    }

    /// Build a [`TSModuleReference::IdentifierReference`] with `reference_id`.
    ///
    /// This node contains an [`IdentifierReference`] that will be stored in the memory arena.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `name`: The name of the identifier being referenced.
    /// * `reference_id`: Reference ID
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_reference_identifier_reference_with_reference_id<S1>(
        self,
        span: Span,
        name: S1,
        reference_id: ReferenceId,
    ) -> TSModuleReference<'a>
    where
        S1: Into<Ident<'a>>,
    {
        TSModuleReference::IdentifierReference(self.alloc_identifier_reference_with_reference_id(
            span,
            name,
            reference_id,
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
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_module_reference_qualified_name(
        self,
        span: Span,
        left: TSTypeName<'a>,
        right: IdentifierName<'a>,
    ) -> TSModuleReference<'a> {
        TSModuleReference::QualifiedName(self.alloc_ts_qualified_name(span, left, right))
    }

    /// Build a [`TSExternalModuleReference`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_external_module_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> TSExternalModuleReference<'a> {
        TSExternalModuleReference { node_id: Default::default(), span, expression }
    }

    /// Build a [`TSExternalModuleReference`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_external_module_reference`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_external_module_reference(
        self,
        span: Span,
        expression: StringLiteral<'a>,
    ) -> ArenaBox<'a, TSExternalModuleReference<'a>> {
        ArenaBox::new_in(self.ts_external_module_reference(span, expression), &self)
    }

    /// Build a [`TSNonNullExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_non_null_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSNonNullExpression<'a> {
        TSNonNullExpression { node_id: Default::default(), span, expression }
    }

    /// Build a [`TSNonNullExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_non_null_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_non_null_expression(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, TSNonNullExpression<'a>> {
        ArenaBox::new_in(self.ts_non_null_expression(span, expression), &self)
    }

    /// Build a [`Decorator`].
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn decorator(self, span: Span, expression: Expression<'a>) -> Decorator<'a> {
        Decorator { node_id: Default::default(), span, expression }
    }

    /// Build a [`TSExportAssignment`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_export_assignment`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> TSExportAssignment<'a> {
        TSExportAssignment { node_id: Default::default(), span, expression }
    }

    /// Build a [`TSExportAssignment`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_export_assignment`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_export_assignment(
        self,
        span: Span,
        expression: Expression<'a>,
    ) -> ArenaBox<'a, TSExportAssignment<'a>> {
        ArenaBox::new_in(self.ts_export_assignment(span, expression), &self)
    }

    /// Build a [`TSNamespaceExportDeclaration`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_namespace_export_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> TSNamespaceExportDeclaration<'a> {
        TSNamespaceExportDeclaration { node_id: Default::default(), span, id }
    }

    /// Build a [`TSNamespaceExportDeclaration`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_namespace_export_declaration`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `id`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_namespace_export_declaration(
        self,
        span: Span,
        id: IdentifierName<'a>,
    ) -> ArenaBox<'a, TSNamespaceExportDeclaration<'a>> {
        ArenaBox::new_in(self.ts_namespace_export_declaration(span, id), &self)
    }

    /// Build a [`TSInstantiationExpression`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_ts_instantiation_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn ts_instantiation_expression<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
    ) -> TSInstantiationExpression<'a>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        TSInstantiationExpression {
            node_id: Default::default(),
            span,
            expression,
            type_arguments: type_arguments.into_in(self.allocator()),
        }
    }

    /// Build a [`TSInstantiationExpression`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::ts_instantiation_expression`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `expression`
    /// * `type_arguments`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_ts_instantiation_expression<T1>(
        self,
        span: Span,
        expression: Expression<'a>,
        type_arguments: T1,
    ) -> ArenaBox<'a, TSInstantiationExpression<'a>>
    where
        T1: IntoIn<'a, ArenaBox<'a, TSTypeParameterInstantiation<'a>>>,
    {
        ArenaBox::new_in(self.ts_instantiation_expression(span, expression, type_arguments), &self)
    }

    /// Build a [`JSDocNullableType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_js_doc_nullable_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNullableType<'a> {
        JSDocNullableType { node_id: Default::default(), span, type_annotation, postfix }
    }

    /// Build a [`JSDocNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::js_doc_nullable_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`: Was `?` after the type annotation?
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_js_doc_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> ArenaBox<'a, JSDocNullableType<'a>> {
        ArenaBox::new_in(self.js_doc_nullable_type(span, type_annotation, postfix), &self)
    }

    /// Build a [`JSDocNonNullableType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_js_doc_non_nullable_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> JSDocNonNullableType<'a> {
        JSDocNonNullableType { node_id: Default::default(), span, type_annotation, postfix }
    }

    /// Build a [`JSDocNonNullableType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::js_doc_non_nullable_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `type_annotation`
    /// * `postfix`
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_js_doc_non_nullable_type(
        self,
        span: Span,
        type_annotation: TSType<'a>,
        postfix: bool,
    ) -> ArenaBox<'a, JSDocNonNullableType<'a>> {
        ArenaBox::new_in(self.js_doc_non_nullable_type(span, type_annotation, postfix), &self)
    }

    /// Build a [`JSDocUnknownType`].
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn js_doc_unknown_type(self, span: Span) -> JSDocUnknownType {
        JSDocUnknownType { node_id: Default::default(), span }
    }

    /// Build a [`JSDocUnknownType`], and store it in the memory arena.
    ///
    /// Returns a [`Box`](ArenaBox) containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::js_doc_unknown_type`] instead.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[deprecated(
        note = "Migrate to new `AstBuilder` interface. See https://github.com/oxc-project/oxc/issues/23043"
    )]
    #[inline]
    pub fn alloc_js_doc_unknown_type(self, span: Span) -> ArenaBox<'a, JSDocUnknownType> {
        ArenaBox::new_in(self.js_doc_unknown_type(span), &self)
    }
}
