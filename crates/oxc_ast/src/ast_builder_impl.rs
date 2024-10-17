#![allow(
    clippy::fn_params_excessive_bools,
    clippy::must_use_candidate, // must_use_candidate is too annoying for this file
    clippy::too_many_arguments,
    clippy::unused_self,
)]
#![warn(missing_docs)]

use std::{cell::Cell, mem};

use oxc_allocator::{Allocator, Box, FromIn, IntoIn, String, Vec};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::{number::NumberBase, operator::UnaryOperator, scope::ScopeId};

#[allow(clippy::wildcard_imports)]
use crate::ast::*;
use crate::AstBuilder;

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Anything<'a>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
#[allow(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<Box<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}

impl<'a> AstBuilder<'a> {
    /// Create a new AST builder that will allocate nodes in the given allocator.
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    /// Move a value into the memory arena.
    #[inline]
    pub fn alloc<T>(self, value: T) -> Box<'a, T> {
        Box::new_in(value, self.allocator)
    }

    /// Create a new empty [`Vec`] that stores its elements in the memory arena.
    #[inline]
    pub fn vec<T>(self) -> Vec<'a, T> {
        Vec::new_in(self.allocator)
    }

    /// Create a new empty [`Vec`] that stores its elements in the memory arena.
    /// Enough memory will be pre-allocated to store at least `capacity`
    /// elements.
    #[inline]
    pub fn vec_with_capacity<T>(self, capacity: usize) -> Vec<'a, T> {
        Vec::with_capacity_in(capacity, self.allocator)
    }

    /// Create a new arena-allocated [`Vec`] initialized with a single element.
    #[inline]
    pub fn vec1<T>(self, value: T) -> Vec<'a, T> {
        let mut vec = self.vec_with_capacity(1);
        vec.push(value);
        vec
    }

    /// Collect an iterator into a new arena-allocated [`Vec`].
    #[inline]
    pub fn vec_from_iter<T, I: IntoIterator<Item = T>>(self, iter: I) -> Vec<'a, T> {
        Vec::from_iter_in(iter, self.allocator)
    }

    /// Move a string slice into the memory arena, returning a reference to the slice
    /// in the heap.
    #[inline]
    pub fn str(self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    /// Allocate an [`Atom`] from a string slice.
    #[inline]
    pub fn atom(self, value: &str) -> Atom<'a> {
        Atom::from(String::from_str_in(value, self.allocator).into_bump_str())
    }

    /// # SAFETY
    /// This method is completely unsound and should not be used.
    /// We need to remove all uses of it. Please don't add any more!
    /// <https://github.com/oxc-project/oxc/issues/3483>
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn copy<T>(self, src: &T) -> T {
        // SAFETY: Not safe (see above)
        unsafe { std::mem::transmute_copy(src) }
    }

    /// Moves the expression out by replacing it with a [null
    /// expression](Expression::NullLiteral).
    #[inline]
    pub fn move_expression(self, expr: &mut Expression<'a>) -> Expression<'a> {
        let null_expr = self.expression_null_literal(expr.span());
        mem::replace(expr, null_expr)
    }

    /// Moves the statement out by replacing it with an [empty
    /// statement](Statement::EmptyStatement).
    #[inline]
    pub fn move_statement(self, stmt: &mut Statement<'a>) -> Statement<'a> {
        let empty_stmt = self.empty_statement(stmt.span());
        mem::replace(stmt, Statement::EmptyStatement(self.alloc(empty_stmt)))
    }

    /// Moves the assignment target out by replacing it with a dummy target with
    /// no name and an empty [`Span`].
    #[inline]
    pub fn move_assignment_target(self, target: &mut AssignmentTarget<'a>) -> AssignmentTarget<'a> {
        let dummy =
            self.simple_assignment_target_identifier_reference(Span::default(), Atom::from(""));
        mem::replace(target, dummy.into())
    }

    /// Move a declaration out by replacing it with an empty [variable
    /// declaration](Declaration::VariableDeclaration).
    #[inline]
    pub fn move_declaration(self, decl: &mut Declaration<'a>) -> Declaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.vec(),
            false,
        );
        let empty_decl = Declaration::VariableDeclaration(self.alloc(empty_decl));
        mem::replace(decl, empty_decl)
    }

    /// Move a variable declaration out by replacing it with an empty [variable
    /// declaration](VariableDeclaration).
    #[inline]
    pub fn move_variable_declaration(
        self,
        decl: &mut VariableDeclaration<'a>,
    ) -> VariableDeclaration<'a> {
        let empty_decl = self.variable_declaration(
            Span::default(),
            VariableDeclarationKind::Var,
            self.vec(),
            false,
        );
        mem::replace(decl, empty_decl)
    }

    /// Move an array element out by replacing it with an
    /// [elision](ArrayExpressionElement::Elision).
    pub fn move_array_expression_element(
        self,
        element: &mut ArrayExpressionElement<'a>,
    ) -> ArrayExpressionElement<'a> {
        let empty_element = self.array_expression_element_elision(Span::default());
        mem::replace(element, empty_element)
    }

    /// Take the contents of a arena-allocated [`Vec`], leaving an empty vec in
    /// its place. This is akin to [`std::mem::take`].
    #[inline]
    pub fn move_vec<T>(self, vec: &mut Vec<'a, T>) -> Vec<'a, T> {
        mem::replace(vec, self.vec())
    }

    /* ---------- Constructors ---------- */

    /// `0`
    #[inline]
    pub fn number_0(self) -> Expression<'a> {
        self.expression_numeric_literal(Span::default(), 0.0, "0", NumberBase::Decimal)
    }

    /// `void 0`
    #[inline]
    pub fn void_0(self, span: Span) -> Expression<'a> {
        let num = self.number_0();
        Expression::UnaryExpression(self.alloc(self.unary_expression(
            span,
            UnaryOperator::Void,
            num,
        )))
    }

    /* ---------- Functions ---------- */

    /// Create a [`FormalParameter`] with no type annotations, modifiers,
    /// decorators, or initializer.
    #[inline]
    pub fn plain_formal_parameter(
        self,
        span: Span,
        pattern: BindingPattern<'a>,
    ) -> FormalParameter<'a> {
        self.formal_parameter(span, self.vec(), pattern, None, false, false)
    }

    /// Create a [`Function`] with no "extras", i.e. decorators, type
    /// annotations, accessibility modifiers, etc.
    #[inline]
    pub fn plain_function(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        params: FormalParameters<'a>,
        body: FunctionBody<'a>,
    ) -> Box<'a, Function<'a>> {
        self.alloc(self.function(
            r#type,
            span,
            id,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
        ))
    }

    /* ---------- Modules ---------- */

    /// Create an empty [`ExportNamedDeclaration`] with no modifiers
    #[inline]
    pub fn plain_export_named_declaration_declaration(
        self,
        span: Span,
        declaration: Declaration<'a>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(self.export_named_declaration(
            span,
            Some(declaration),
            self.vec(),
            None,
            ImportOrExportKind::Value,
            NONE,
        ))
    }

    /// Create an [`ExportNamedDeclaration`] with no modifiers that contains a
    /// set of [exported symbol names](ExportSpecifier).
    #[inline]
    pub fn plain_export_named_declaration(
        self,
        span: Span,
        specifiers: Vec<'a, ExportSpecifier<'a>>,
        source: Option<StringLiteral<'a>>,
    ) -> Box<'a, ExportNamedDeclaration<'a>> {
        self.alloc(self.export_named_declaration(
            span,
            None,
            specifiers,
            source,
            ImportOrExportKind::Value,
            NONE,
        ))
    }

    /* ---------- TypeScript ---------- */

    /// Create a [`TSInterfaceHeritage`] that extends from the given list of
    /// other interfaces.
    #[inline]
    pub fn ts_interface_heritages(
        self,
        extends: Vec<'a, (Expression<'a>, Option<Box<'a, TSTypeParameterInstantiation<'a>>>, Span)>,
    ) -> Vec<'a, TSInterfaceHeritage<'a>> {
        Vec::from_iter_in(
            extends.into_iter().map(|(expression, type_parameters, span)| TSInterfaceHeritage {
                span,
                expression,
                type_parameters,
            }),
            self.allocator,
        )
    }

    /// Create an [`JSXOpeningElement`].
    #[inline]
    pub fn jsx_opening_fragment(self, span: Span) -> JSXOpeningFragment {
        JSXOpeningFragment { span }
    }

    /// Create an [`JSXClosingElement`].
    #[inline]
    pub fn jsx_closing_fragment(self, span: Span) -> JSXClosingFragment {
        JSXClosingFragment { span }
    }

    /* ---------- Constructors with `ScopeId`s ---------- */

    /// Build an [`ArrowFunctionExpression`] with specified `ScopeId`.
    ///
    /// If you want the built node to be allocated in the memory arena,
    /// use [`AstBuilder::alloc_arrow_function_expression_with_scope_id`] instead.
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

    /// Build an [`ArrowFunctionExpression`] with specified `ScopeId`, and stores it in the memory arena.
    ///
    /// Returns a [`Box`] containing the newly-allocated node.
    /// If you want a stack-allocated node, use [`AstBuilder::arrow_function_expression_with_scope_id`] instead.
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

    /// Build an [`Expression::ArrowFunctionExpression`] with specified `ScopeId`.
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
    /// - scope_id
    #[inline]
    pub fn expression_arrow_function_with_scope_id<T1, T2, T3, T4>(
        self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: T1,
        params: T2,
        return_type: T3,
        body: T4,
        scope_id: ScopeId,
    ) -> Expression<'a>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T3: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T4: IntoIn<'a, Box<'a, FunctionBody<'a>>>,
    {
        Expression::ArrowFunctionExpression(self.alloc_arrow_function_expression_with_scope_id(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
            scope_id,
        ))
    }
}
