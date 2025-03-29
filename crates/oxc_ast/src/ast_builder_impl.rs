#![warn(missing_docs)]

use std::borrow::Cow;

use oxc_allocator::{Allocator, Box, FromIn, IntoIn, String, TakeIn, Vec};
use oxc_span::{Atom, SPAN, Span};
use oxc_syntax::{number::NumberBase, operator::UnaryOperator, scope::ScopeId};

use crate::{AstBuilder, ast::*};

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Anything<'a>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
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
        self.vec_from_array([value])
    }

    /// Collect an iterator into a new arena-allocated [`Vec`].
    #[inline]
    pub fn vec_from_iter<T, I: IntoIterator<Item = T>>(self, iter: I) -> Vec<'a, T> {
        Vec::from_iter_in(iter, self.allocator)
    }

    /// Create [`Vec`] from a fixed-size array.
    ///
    /// This is preferable to `vec_from_iter` where source is an array, as size is statically known,
    /// and compiler is more likely to construct the values directly in arena, rather than constructing
    /// on stack and then copying to arena.
    #[inline]
    pub fn vec_from_array<T, const N: usize>(self, array: [T; N]) -> Vec<'a, T> {
        Vec::from_array_in(array, self.allocator)
    }

    /// Move a string slice into the memory arena, returning a reference to the slice
    /// in the heap.
    #[inline]
    pub fn str(self, value: &str) -> &'a str {
        self.allocator.alloc_str(value)
    }

    /// Allocate an [`Atom`] from a string slice.
    #[inline]
    pub fn atom(self, value: &str) -> Atom<'a> {
        Atom::from_in(value, self.allocator)
    }

    /// Allocate an [`Atom`] from an array of string slices.
    #[inline]
    pub fn atom_from_strs_array<const N: usize>(self, array: [&str; N]) -> Atom<'a> {
        let string = String::from_strs_array_in(array, self.allocator);
        Atom::from(string)
    }

    /// Convert a [`Cow<'a, str>`] to an [`Atom<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Atom` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Atom`.
    #[inline]
    pub fn atom_from_cow(self, value: &Cow<'a, str>) -> Atom<'a> {
        match value {
            Cow::Borrowed(s) => Atom::from(*s),
            Cow::Owned(s) => self.atom(s),
        }
    }

    /// Replace [`Expression`] with a dummy node, and return the original.
    #[inline]
    pub fn move_expression(self, expr: &mut Expression<'a>) -> Expression<'a> {
        expr.take_in(self.allocator)
    }

    /// Replace [`Statement`] with a dummy node, and return the original.
    #[inline]
    pub fn move_statement(self, stmt: &mut Statement<'a>) -> Statement<'a> {
        stmt.take_in(self.allocator)
    }

    /// Replace [`AssignmentTarget`] with a dummy node, and return the original.
    #[inline]
    pub fn move_assignment_target(self, target: &mut AssignmentTarget<'a>) -> AssignmentTarget<'a> {
        target.take_in(self.allocator)
    }

    /// Replace [`PropertyKey`] with a dummy node, and return the original.
    #[inline]
    pub fn move_property_key(self, key: &mut PropertyKey<'a>) -> PropertyKey<'a> {
        key.take_in(self.allocator)
    }

    /// Replace [`Declaration`] with a dummy node, and return the original.
    #[inline]
    pub fn move_declaration(self, decl: &mut Declaration<'a>) -> Declaration<'a> {
        decl.take_in(self.allocator)
    }

    /// Replace [`VariableDeclaration`] with a dummy node, and return the original.
    #[inline]
    pub fn move_variable_declaration(
        self,
        decl: &mut VariableDeclaration<'a>,
    ) -> VariableDeclaration<'a> {
        decl.take_in(self.allocator)
    }

    /// Replace [`FormalParameters`] with a dummy node, and return the original.
    #[inline]
    pub fn move_formal_parameters(self, params: &mut FormalParameters<'a>) -> FormalParameters<'a> {
        params.take_in(self.allocator)
    }

    /// Replace [`FunctionBody`] with a dummy node, and return the original.
    #[inline]
    pub fn move_function_body(self, body: &mut FunctionBody<'a>) -> FunctionBody<'a> {
        body.take_in(self.allocator)
    }

    /// Replace [`Function`] with a dummy node, and return the original.
    #[inline]
    pub fn move_function(self, function: &mut Function<'a>) -> Function<'a> {
        function.take_in(self.allocator)
    }

    /// Replace [`Class`] with a dummy node, and return the original.
    #[inline]
    pub fn move_class(self, class: &mut Class<'a>) -> Class<'a> {
        class.take_in(self.allocator)
    }

    /// Replace [`ArrayExpressionElement`] with a dummy node, and return the original.
    #[inline]
    pub fn move_array_expression_element(
        self,
        element: &mut ArrayExpressionElement<'a>,
    ) -> ArrayExpressionElement<'a> {
        element.take_in(self.allocator)
    }

    /// Replace [`Vec`] with an empty [`Vec`], and return the original.
    #[inline]
    pub fn move_vec<T>(self, vec: &mut Vec<'a, T>) -> Vec<'a, T> {
        vec.take_in(self.allocator)
    }

    /* ---------- Constructors ---------- */

    /// `0`
    #[inline]
    pub fn number_0(self) -> Expression<'a> {
        self.expression_numeric_literal(SPAN, 0.0, None, NumberBase::Decimal)
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

    /// `"use strict"` directive
    #[inline]
    pub fn use_strict_directive(self) -> Directive<'a> {
        let use_strict = Atom::from("use strict");
        self.directive(SPAN, self.string_literal(SPAN, use_strict, None), use_strict)
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

    /// Create a [`Function`] with no "extras".
    /// i.e. no decorators, type annotations, accessibility modifiers, etc.
    #[inline]
    pub fn alloc_plain_function_with_scope_id(
        self,
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        params: FormalParameters<'a>,
        body: FunctionBody<'a>,
        scope_id: ScopeId,
    ) -> Box<'a, Function<'a>> {
        self.alloc_function_with_scope_id_and_pure(
            span,
            r#type,
            id,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
            scope_id,
            false,
        )
    }

    /// Build a [`Function`] with `scope_id`.
    #[inline]
    pub fn alloc_function_with_scope_id<T1, T2, T3, T4, T5>(
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
    ) -> Box<'a, Function<'a>>
    where
        T1: IntoIn<'a, Option<Box<'a, TSTypeParameterDeclaration<'a>>>>,
        T2: IntoIn<'a, Option<Box<'a, TSThisParameter<'a>>>>,
        T3: IntoIn<'a, Box<'a, FormalParameters<'a>>>,
        T4: IntoIn<'a, Option<Box<'a, TSTypeAnnotation<'a>>>>,
        T5: IntoIn<'a, Option<Box<'a, FunctionBody<'a>>>>,
    {
        self.alloc_function_with_scope_id_and_pure(
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
            false,
        )
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
        self.vec_from_iter(extends.into_iter().map(|(expression, type_parameters, span)| {
            TSInterfaceHeritage { span, expression, type_arguments: type_parameters }
        }))
    }
}
