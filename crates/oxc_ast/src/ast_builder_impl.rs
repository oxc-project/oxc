#![warn(missing_docs)]

use std::borrow::Cow;

use oxc_allocator::{Allocator, AllocatorAccessor, Box, FromIn, IntoIn, Vec};
use oxc_span::{Atom, SPAN, Span};
use oxc_syntax::{
    comment_node::CommentNodeId, number::NumberBase, operator::UnaryOperator, scope::ScopeId,
};

use crate::ast::*;

/// Type that can be used in any AST builder method call which requires an `IntoIn<'a, Anything<'a>>`.
/// Pass `NONE` instead of `None::<Anything<'a>>`.
#[expect(clippy::upper_case_acronyms)]
pub struct NONE;

impl<'a, T> FromIn<'a, NONE> for Option<Box<'a, T>> {
    fn from_in(_: NONE, _: &'a Allocator) -> Self {
        None
    }
}

impl<'a> AllocatorAccessor<'a> for AstBuilder<'a> {
    #[inline]
    fn allocator(self) -> &'a Allocator {
        self.allocator
    }
}

/// AST builder for creating AST nodes.
#[derive(Clone, Copy)]
pub struct AstBuilder<'a> {
    /// The memory allocator used to allocate AST nodes in the arena.
    pub allocator: &'a Allocator,
}

impl<'a> AstBuilder<'a> {
    /// Create a new AST builder that will allocate nodes in the given allocator.
    #[inline]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    /// Create [`CommentNodeId`] for an AST node.
    #[expect(dead_code, clippy::unused_self, clippy::trivially_copy_pass_by_ref)]
    pub(crate) fn get_comment_node_id(&self) -> CommentNodeId {
        // TODO: Generate a real ID
        CommentNodeId::DUMMY
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
    pub fn atom_from_strs_array<const N: usize>(self, strings: [&str; N]) -> Atom<'a> {
        Atom::from_strs_array_in(strings, self.allocator)
    }

    /// Convert a [`Cow<'a, str>`] to an [`Atom<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Atom` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Atom`.
    #[inline]
    pub fn atom_from_cow(self, value: &Cow<'a, str>) -> Atom<'a> {
        Atom::from_cow_in(value, self.allocator)
    }

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
    /// `NaN`
    #[inline]
    pub fn nan(self, span: Span) -> Expression<'a> {
        self.expression_numeric_literal(span, f64::NAN, None, NumberBase::Decimal)
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
        self.alloc_function_with_scope_id_and_pure_and_pife(
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
            false,
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
}
