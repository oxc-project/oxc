//! Convenience builder methods defined on AST types.
//!
//! All delegate to generated builder methods, but take less params (with common defaults),
//! add additional functionality, or are shortcuts for common patterns.
//!
//! Like the generated methods, each first calls [`GetAstBuilder::builder`] to obtain the concrete
//! [`AstBuild`]er before forwarding it on, so callees are monomorphized over the `AstBuild` type,
//! rather than over every [`GetAstBuilder`] type.
//!
//! [`AstBuild`]: super::AstBuild

use std::{alloc::Layout, mem::MaybeUninit, slice, str};

use oxc_allocator::{Allocator, ArenaBox, GetAllocator};
use oxc_span::{SPAN, Span};
use oxc_str::Str;
use oxc_syntax::{number::NumberBase, operator::UnaryOperator, scope::ScopeId};

use crate::ast::*;

use super::GetAstBuilder;

impl<'a> Expression<'a> {
    /// Build an [`Expression`] representing the number `0`.
    #[inline]
    pub fn new_number_0(builder: &impl GetAstBuilder<'a>) -> Self {
        let builder = builder.builder();
        Expression::new_numeric_literal(SPAN, 0.0, None, NumberBase::Decimal, builder)
    }

    /// Build an [`Expression`] representing `void 0`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_void_0(span: Span, builder: &impl GetAstBuilder<'a>) -> Self {
        let builder = builder.builder();
        let argument = Expression::new_number_0(builder);
        Expression::new_unary_expression(span, UnaryOperator::Void, argument, builder)
    }

    /// Build an [`Expression`] representing `NaN`.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    #[inline]
    pub fn new_nan(span: Span, builder: &impl GetAstBuilder<'a>) -> Self {
        let builder = builder.builder();
        Expression::new_numeric_literal(span, f64::NAN, None, NumberBase::Decimal, builder)
    }
}

impl<'a> Directive<'a> {
    /// Build a `"use strict"` [`Directive`].
    #[inline]
    pub fn new_use_strict(builder: &impl GetAstBuilder<'a>) -> Self {
        let builder = builder.builder();
        let use_strict = Str::from("use strict");
        Directive::new(
            SPAN,
            StringLiteral::new(SPAN, use_strict, None, builder),
            use_strict,
            builder,
        )
    }
}

impl<'a> FormalParameter<'a> {
    /// Build a [`FormalParameter`] with no type annotations, modifiers, decorators, or initializer.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `pattern`
    #[inline]
    pub fn new_plain(
        span: Span,
        pattern: BindingPattern<'a>,
        builder: &impl GetAstBuilder<'a>,
    ) -> Self {
        let builder = builder.builder();
        FormalParameter::new(span, [], pattern, None, None, false, None, false, false, builder)
    }
}

impl<'a> Function<'a> {
    /// Build a [`Function`] with `scope_id` and no "extras", and store it in the memory arena.
    ///
    /// i.e. no decorators, type annotations, accessibility modifiers, etc.
    ///
    /// ## Parameters
    /// * `type`
    /// * `span`: The [`Span`] covering this node
    /// * `id`: The function identifier. [`None`] for anonymous function expressions.
    /// * `params`: Function parameters.
    /// * `body`: The function body.
    /// * `scope_id`
    #[inline]
    pub fn boxed_plain_with_scope_id(
        r#type: FunctionType,
        span: Span,
        id: Option<BindingIdentifier<'a>>,
        params: ArenaBox<'a, FormalParameters<'a>>,
        body: ArenaBox<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        builder: &impl GetAstBuilder<'a>,
    ) -> ArenaBox<'a, Self> {
        let builder = builder.builder();
        Function::boxed_with_scope_id_and_pure_and_pife(
            span,
            r#type,
            id,
            false,
            false,
            false,
            None,
            None,
            params,
            None,
            Some(body),
            scope_id,
            false,
            false,
            builder,
        )
    }

    /// Build a [`Function`] with `scope_id` (with `pure` and `pife` defaulting to `false`),
    /// and store it in the memory arena.
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
    #[inline]
    pub fn boxed_with_scope_id(
        span: Span,
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        generator: bool,
        r#async: bool,
        declare: bool,
        type_parameters: Option<ArenaBox<'a, TSTypeParameterDeclaration<'a>>>,
        this_param: Option<ArenaBox<'a, TSThisParameter<'a>>>,
        params: ArenaBox<'a, FormalParameters<'a>>,
        return_type: Option<ArenaBox<'a, TSTypeAnnotation<'a>>>,
        body: Option<ArenaBox<'a, FunctionBody<'a>>>,
        scope_id: ScopeId,
        builder: &impl GetAstBuilder<'a>,
    ) -> ArenaBox<'a, Self> {
        let builder = builder.builder();
        Function::boxed_with_scope_id_and_pure_and_pife(
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
            builder,
        )
    }
}

impl<'a> TemplateElement<'a> {
    /// Build a [`TemplateElement`], escaping special characters in the raw value.
    ///
    /// Like [`TemplateElement::new`], but escapes backticks, `${`, backslashes, and carriage
    /// returns in `value.raw` first.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    /// * `tail`
    #[inline]
    pub fn new_escape_raw(
        span: Span,
        mut value: TemplateElementValue<'a>,
        tail: bool,
        builder: &impl GetAstBuilder<'a>,
    ) -> Self {
        let builder = builder.builder();
        value.raw = escape_template_element_raw(value.raw, builder.allocator());
        TemplateElement::new(span, value, tail, builder)
    }

    /// Build a [`TemplateElement`] with `lone_surrogates`, escaping special characters in the raw value.
    ///
    /// Like [`TemplateElement::new_with_lone_surrogates`], but escapes backticks, `${`,
    /// backslashes, and carriage returns in `value.raw` first.
    ///
    /// ## Parameters
    /// * `span`: The [`Span`] covering this node
    /// * `value`
    /// * `tail`
    /// * `lone_surrogates`: The template element contains lone surrogates.
    #[inline]
    pub fn new_escape_raw_with_lone_surrogates(
        span: Span,
        mut value: TemplateElementValue<'a>,
        tail: bool,
        lone_surrogates: bool,
        builder: &impl GetAstBuilder<'a>,
    ) -> Self {
        let builder = builder.builder();
        value.raw = escape_template_element_raw(value.raw, builder.allocator());
        TemplateElement::new_with_lone_surrogates(span, value, tail, lone_surrogates, builder)
    }
}

/// Escape special characters for template element raw value.
///
/// Escapes: backticks, `${`, backslashes, and carriage returns.
fn escape_template_element_raw<'a>(raw: Str<'a>, allocator: &'a Allocator) -> Str<'a> {
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
    let ptr = allocator.alloc_layout(layout);

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
