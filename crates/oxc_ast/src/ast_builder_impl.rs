use std::{alloc::Layout, borrow::Cow, mem::MaybeUninit, slice, str};

use oxc_allocator::{Allocator, AllocatorAccessor, Box, FromIn, IntoIn, Vec};
use oxc_span::{SPAN, Span};
use oxc_str::{Ident, Str};
use oxc_syntax::{number::NumberBase, operator::UnaryOperator, scope::ScopeId};

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

    /// Allocate an [`Ident`] from a string slice.
    #[inline]
    pub fn ident(self, value: &str) -> Ident<'a> {
        Ident::from_in(value, self.allocator)
    }

    /// Allocate an [`Ident`] from an array of string slices.
    #[inline]
    pub fn ident_from_strs_array<const N: usize>(self, strings: [&str; N]) -> Ident<'a> {
        Ident::from_strs_array_in(strings, self.allocator)
    }

    /// Convert a [`Cow<'a, str>`] to an [`Ident<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Ident` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Ident`.
    #[inline]
    pub fn ident_from_cow(self, value: &Cow<'a, str>) -> Ident<'a> {
        Ident::from_cow_in(value, self.allocator)
    }

    /// Allocate a [`Str`] from a string slice.
    #[inline]
    pub fn str(self, value: &str) -> Str<'a> {
        Str::from_in(value, self.allocator)
    }

    /// Allocate a [`Str`] from an array of string slices.
    #[inline]
    pub fn str_from_strs_array<const N: usize>(self, strings: [&str; N]) -> Str<'a> {
        Str::from_strs_array_in(strings, self.allocator)
    }

    /// Convert a [`Cow<'a, str>`] to a [`Str<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns a `Str` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Str`.
    #[inline]
    pub fn str_from_cow(self, value: &Cow<'a, str>) -> Str<'a> {
        Str::from_cow_in(value, self.allocator)
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
        let use_strict = Str::from("use strict");
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
        self.formal_parameter(span, self.vec(), pattern, NONE, NONE, false, None, false, false)
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

    /* ---------- Template literals ---------- */

    /// Build a [`TemplateElement`], escaping special characters in the raw value.
    ///
    /// Like [`AstBuilder::template_element`], but escapes backticks, `${`, backslashes, and carriage
    /// returns in `value.raw` first.
    #[inline]
    pub fn template_element_escape_raw(
        self,
        span: Span,
        mut value: TemplateElementValue<'a>,
        tail: bool,
    ) -> TemplateElement<'a> {
        value.raw = escape_template_element_raw(value.raw, self);
        self.template_element(span, value, tail)
    }

    /// Build a [`TemplateElement`] with `lone_surrogates`, escaping special characters in the raw value.
    ///
    /// Like [`AstBuilder::template_element_with_lone_surrogates`], but escapes backticks, `${`,
    /// backslashes, and carriage returns in `value.raw` first.
    #[inline]
    pub fn template_element_escape_raw_with_lone_surrogates(
        self,
        span: Span,
        mut value: TemplateElementValue<'a>,
        tail: bool,
        lone_surrogates: bool,
    ) -> TemplateElement<'a> {
        value.raw = escape_template_element_raw(value.raw, self);
        self.template_element_with_lone_surrogates(span, value, tail, lone_surrogates)
    }
}

/// Escape special characters for template element raw value.
///
/// Escapes: backticks, `${`, backslashes, and carriage returns.
fn escape_template_element_raw<'a>(raw: Str<'a>, ast: AstBuilder<'a>) -> Str<'a> {
    let bytes = raw.as_bytes();

    // Calculate size needed for escaped string
    let mut extra_bytes = 0usize;
    for i in 0..bytes.len() {
        extra_bytes += match bytes[i] {
            b'\\' | b'`' | b'\r' => 1,
            b'$' if bytes.get(i + 1) == Some(&b'{') => 1,
            _ => 0,
        };
    }

    if extra_bytes == 0 {
        return raw;
    }

    // Allocate directly in arena.
    // It's impossible for this addition to overflow, because max length of a `&str` is `isize::MAX`
    // and we've at most doubled the length, which cannot overflow `usize::MAX`.
    let len = bytes.len() + extra_bytes;
    let layout = Layout::array::<u8>(len).unwrap();
    let ptr = ast.allocator.alloc_layout(layout);

    // SAFETY: `ptr` points to `len` bytes of memory allocated by the arena.
    // `MaybeUninit<u8>` has the same layout as `u8` and does not require its contents to be initialized,
    // so it's sound to form a `&mut [MaybeUninit<u8>]` over this uninitialized memory.
    let dest = unsafe { slice::from_raw_parts_mut(ptr.as_ptr().cast::<MaybeUninit<u8>>(), len) };

    let mut j = 0;
    for i in 0..bytes.len() {
        // SAFETY: For each input byte we write either 1 or 2 bytes, and `len` was sized to fit
        // exactly that many bytes, so `j` and `j + 1` are always in bounds.
        // Note: Compiler merges each pair of writes into a single 2-byte write.
        unsafe {
            match bytes[i] {
                b'\\' => {
                    dest.get_unchecked_mut(j).write(b'\\');
                    dest.get_unchecked_mut(j + 1).write(b'\\');
                    j += 2;
                }
                b'`' => {
                    dest.get_unchecked_mut(j).write(b'\\');
                    dest.get_unchecked_mut(j + 1).write(b'`');
                    j += 2;
                }
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    dest.get_unchecked_mut(j).write(b'\\');
                    dest.get_unchecked_mut(j + 1).write(b'$');
                    j += 2;
                }
                b'\r' => {
                    dest.get_unchecked_mut(j).write(b'\\');
                    dest.get_unchecked_mut(j + 1).write(b'r');
                    j += 2;
                }
                b => {
                    dest.get_unchecked_mut(j).write(b);
                    j += 1;
                }
            }
        }
    }

    debug_assert_eq!(j, len);

    // SAFETY: The loop above initialized all `len` bytes of `dest`.
    // `MaybeUninit<u8>` has the same layout as `u8`, so it's sound to read those bytes back as `&[u8]`
    // via a pointer cast. `MaybeUninit::slice_assume_init_ref` would express this directly, but it is unstable.
    let bytes = unsafe { slice::from_raw_parts(dest.as_ptr().cast::<u8>(), len) };
    // SAFETY: Input is valid UTF-8 and we only insert ASCII bytes replacing existing ASCII, so output is valid UTF-8
    let escaped = unsafe { str::from_utf8_unchecked(bytes) };
    Str::from(escaped)
}
