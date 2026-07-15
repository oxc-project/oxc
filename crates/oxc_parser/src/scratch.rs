//! Thread-persistent scratch buffers for building AST lists.
//!
//! Building a list by pushing into an [`ArenaVec`] reallocates within the arena as the vec grows.
//! Child AST nodes are allocated in between pushes, so the growing vec's buffer is almost never
//! the arena's most recent allocation, and every growth allocates a new buffer and abandons the
//! old one as dead space in the arena.
//!
//! Instead, the parser builds every list in a scratch `std::Vec`, and moves the completed list
//! into the arena with a single exact-sized allocation ([`ParserImpl::scratch_take`]).
//!
//! There is one scratch vec per list element type ([`ScratchBuffers`]). Nested lists with the
//! same element type (statements of nested blocks, arguments of nested calls) share one scratch
//! vec, used like a stack: a list records the vec's length (its "mark") when it starts, pushes
//! elements on the end, and drains everything from its mark upwards when it completes. List
//! parsing is strictly LIFO — every list-building code path exits by normal return (parse errors
//! are stored, not thrown; element closures cannot early-exit their caller) — so a list always
//! drains its own elements before its enclosing list resumes. This is the scratch pattern used
//! by Zig's `std.zig` parser, split per element type since our lists are heterogeneous.
//!
//! The buffers are checked out of a thread-local cache when the parser is created and returned
//! (emptied) when it is dropped ([`ScratchGuard`]), so they persist for the lifetime of the
//! thread. Once a thread is warm, list building performs no heap allocation at all.

use std::{
    cell::Cell,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use oxc_allocator::{ArenaBox, ArenaVec, GetAllocator};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, BindingProperty, ClassElement, Decorator,
    Directive, ExportSpecifier, Expression, FormalParameter, ImportAttribute,
    ImportDeclarationSpecifier, JSXAttributeItem, JSXChild, ObjectPropertyKind, Statement,
    SwitchCase, TSClassImplements, TSEnumMember, TSIndexSignatureName, TSInterfaceHeritage,
    TSSignature, TSTupleElement, TSType, TSTypeParameter, TSTypeParameterInstantiation,
    TemplateElement, VariableDeclarator,
};

use crate::{ParserConfig, ParserImpl};

/// Access to the scratch buffer holding `T`s in [`ScratchBuffers`].
pub trait ScratchFor<T> {
    fn buf(&mut self) -> &mut Vec<T>;
}

macro_rules! scratch_buffers {
    ($($field:ident: $ty:ty),* $(,)?) => {
        /// Scratch vecs for building AST lists — one per list element type.
        ///
        /// See module docs for the usage pattern.
        #[derive(Default)]
        pub struct ScratchBuffers<'a> {
            $(pub(crate) $field: Vec<$ty>,)*
        }

        impl ScratchBuffers<'_> {
            /// Empty every buffer.
            ///
            /// Element types never need `Drop` (asserted in [`ScratchFor::buf`]),
            /// so this only resets lengths.
            fn clear_all(&mut self) {
                $(self.$field.clear();)*
            }

            /// `true` if every buffer is empty.
            fn is_all_empty(&self) -> bool {
                $(self.$field.is_empty() &&)* true
            }
        }

        $(
            impl<'a> ScratchFor<$ty> for ScratchBuffers<'a> {
                #[inline(always)]
                fn buf(&mut self) -> &mut Vec<$ty> {
                    // Elements are materialized into `ArenaVec`s, whose arena never runs `Drop`,
                    // and `scratch_take` moves them out bytewise. `ArenaVec` asserts the same
                    // requirement; asserting it here too reports errors at the offending buffer.
                    const { assert!(!std::mem::needs_drop::<$ty>()) };
                    &mut self.$field
                }
            }
        )*
    };
}

scratch_buffers! {
    statements: Statement<'a>,
    directives: Directive<'a>,
    switch_cases: SwitchCase<'a>,
    variable_declarators: VariableDeclarator<'a>,
    formal_parameters: FormalParameter<'a>,
    arguments: Argument<'a>,
    expressions: Expression<'a>,
    array_expression_elements: ArrayExpressionElement<'a>,
    object_property_kinds: ObjectPropertyKind<'a>,
    class_elements: ClassElement<'a>,
    binding_properties: BindingProperty<'a>,
    array_binding_elements: Option<BindingPattern<'a>>,
    template_elements: TemplateElement<'a>,
    import_declaration_specifiers: ImportDeclarationSpecifier<'a>,
    import_attributes: ImportAttribute<'a>,
    export_specifiers: ExportSpecifier<'a>,
    ts_types: TSType<'a>,
    ts_signatures: TSSignature<'a>,
    ts_enum_members: TSEnumMember<'a>,
    ts_type_parameters: TSTypeParameter<'a>,
    ts_tuple_elements: TSTupleElement<'a>,
    ts_index_signature_names: TSIndexSignatureName<'a>,
    ts_class_implements: TSClassImplements<'a>,
    ts_interface_heritages: TSInterfaceHeritage<'a>,
    class_extends: (Expression<'a>, Option<ArenaBox<'a, TSTypeParameterInstantiation<'a>>>),
    decorators: Decorator<'a>,
    jsx_children: JSXChild<'a>,
    jsx_attribute_items: JSXAttributeItem<'a>,
}

thread_local! {
    /// This thread's cached scratch buffers.
    /// `None` before first use on the thread, or while a parser has them checked out.
    static CACHE: Cell<Option<Box<ScratchBuffers<'static>>>> = const { Cell::new(None) };
}

/// Change the lifetime parameter of `ScratchBuffers`.
///
/// # SAFETY
///
/// Every vec must be empty, so that no element typed with the old lifetime survives.
/// The lifetime parameter does not affect layout, so an empty vec's allocation (a length,
/// a capacity, and a buffer of uninitialized elements) is valid for the new lifetime.
unsafe fn change_lifetime<'to>(buffers: Box<ScratchBuffers<'_>>) -> Box<ScratchBuffers<'to>> {
    debug_assert!(buffers.is_all_empty());
    // SAFETY: same layout (see above); the `Box` is recreated from the pointer it was split into.
    unsafe { Box::from_raw(Box::into_raw(buffers).cast::<ScratchBuffers<'to>>()) }
}

/// Owner of [`ScratchBuffers`] for the duration of one parse.
///
/// Checks the buffers out of this thread's cache on creation, and returns them (emptied) on
/// drop — including a drop during panic unwinding — so the buffers survive for the thread's
/// lifetime and stay grown to their high-water capacity across parses.
pub struct ScratchGuard<'a>(ManuallyDrop<Box<ScratchBuffers<'a>>>);

impl<'a> ScratchGuard<'a> {
    /// Take this thread's scratch buffers out of the cache.
    ///
    /// Falls back to fresh (empty, unallocated) buffers if the cache is unavailable: on the
    /// thread's first parse, while another parser on this thread has the buffers checked out
    /// (only possible in tests — see `UniquePromise`), or during thread teardown.
    pub(crate) fn checkout() -> Self {
        let buffers = CACHE.try_with(Cell::take).ok().flatten().unwrap_or_default();
        // SAFETY: cached buffers are emptied before being cached in `drop` below,
        // and fresh buffers start empty.
        let buffers: Box<ScratchBuffers<'a>> = unsafe { change_lifetime(buffers) };
        Self(ManuallyDrop::new(buffers))
    }
}

impl Drop for ScratchGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.0` is only taken here, and drop runs at most once.
        let mut buffers = unsafe { ManuallyDrop::take(&mut self.0) };
        // On normal completion, every list site has drained the elements it pushed.
        // Anything left over means a list site broke the mark/push/drain discipline —
        // except when unwinding from a panic, which abandons in-progress lists (harmless,
        // elements have no `Drop`).
        #[cfg(debug_assertions)]
        if !std::thread::panicking() {
            assert!(
                buffers.is_all_empty(),
                "scratch buffers must be empty at end of parse — \
                 a list site pushed elements without draining them"
            );
        }
        buffers.clear_all();
        // SAFETY: all vecs were just emptied.
        let buffers: Box<ScratchBuffers<'static>> = unsafe { change_lifetime(buffers) };
        // Cache for the next parse on this thread.
        // If the thread-local is already destroyed (thread teardown), drop the buffers instead.
        let _ = CACHE.try_with(|cache| cache.set(Some(buffers)));
    }
}

impl<'a> Deref for ScratchGuard<'a> {
    type Target = ScratchBuffers<'a>;

    #[inline]
    fn deref(&self) -> &ScratchBuffers<'a> {
        &self.0
    }
}

impl<'a> DerefMut for ScratchGuard<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut ScratchBuffers<'a> {
        &mut self.0
    }
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    /// Current length of the scratch buffer for `T`.
    ///
    /// Take a mark before pushing a list's elements with [`scratch_push`], and pass it to
    /// [`scratch_take`] once the list is complete. Elements pushed by nested lists in between
    /// are always drained again before this list resumes (see module docs of [`crate::scratch`]).
    ///
    /// [`scratch_push`]: Self::scratch_push
    /// [`scratch_take`]: Self::scratch_take
    #[inline]
    pub(crate) fn scratch_mark<T>(&mut self) -> usize
    where
        ScratchBuffers<'a>: ScratchFor<T>,
    {
        ScratchFor::<T>::buf(&mut *self.scratch).len()
    }

    /// Push a list element onto the scratch buffer for `T`.
    #[inline]
    pub(crate) fn scratch_push<T>(&mut self, element: T)
    where
        ScratchBuffers<'a>: ScratchFor<T>,
    {
        ScratchFor::<T>::buf(&mut *self.scratch).push(element);
    }

    /// Move the elements pushed since `mark` into an exact-sized arena vec.
    #[inline]
    pub(crate) fn scratch_take<T>(&mut self, mark: usize) -> ArenaVec<'a, T>
    where
        ScratchBuffers<'a>: ScratchFor<T>,
    {
        let allocator = self.allocator();
        let buf = ScratchFor::<T>::buf(&mut *self.scratch);
        let len = buf.len() - mark;
        let mut vec = ArenaVec::with_capacity_in(len, &allocator);
        // SAFETY: `vec` has capacity for `len` elements, and `buf[mark..]` are initialized `T`s.
        // The copy is a move: `buf`'s length is cut back to `mark`, so the moved elements are
        // never touched through `buf` again, and `T` never needs `Drop` (asserted in `buf`),
        // so skipping their destructors is sound.
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr().add(mark), vec.as_mut_ptr(), len);
            vec.set_len(len);
            buf.set_len(mark);
        }
        vec
    }

    /// Discard the elements pushed since `mark` — for abandoning a list part-way
    /// (e.g. bailing out with [`Self::unexpected`]).
    #[inline]
    pub(crate) fn scratch_discard<T>(&mut self, mark: usize)
    where
        ScratchBuffers<'a>: ScratchFor<T>,
    {
        ScratchFor::<T>::buf(&mut *self.scratch).truncate(mark);
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::builder::AstBuilder;
    use oxc_span::SPAN;

    use super::*;

    fn statement(allocator: &Allocator) -> Statement<'_> {
        Statement::new_empty_statement(SPAN, &AstBuilder::new(allocator))
    }

    #[test]
    fn warm_across_checkouts() {
        let allocator = Allocator::default();
        // Buffer capacity must survive a checkout/checkin cycle on the same thread.
        let high_water = {
            let mut guard = ScratchGuard::checkout();
            for _ in 0..64 {
                guard.statements.push(statement(&allocator));
            }
            let capacity = guard.statements.capacity();
            assert!(capacity >= 64);
            guard.statements.clear();
            capacity
        };
        let guard = ScratchGuard::checkout();
        assert!(guard.statements.is_empty());
        assert!(guard.statements.capacity() >= high_water);
    }

    #[test]
    fn nested_marks() {
        let allocator = Allocator::default();
        let mut guard = ScratchGuard::checkout();
        let outer_mark = guard.statements.len();
        guard.statements.push(statement(&allocator));
        guard.statements.push(statement(&allocator));
        // A nested list starts, pushes, and drains its own tail.
        let inner_mark = guard.statements.len();
        guard.statements.push(statement(&allocator));
        guard.statements.push(statement(&allocator));
        assert_eq!(guard.statements.drain(inner_mark..).count(), 2);
        // The outer list still sees exactly its own elements.
        guard.statements.push(statement(&allocator));
        assert_eq!(guard.statements.drain(outer_mark..).count(), 3);
        assert!(guard.statements.is_empty());
    }

    #[test]
    fn cached_even_on_panic() {
        let result = std::panic::catch_unwind(|| {
            let allocator = Allocator::default();
            let mut guard = ScratchGuard::checkout();
            for _ in 0..32 {
                guard.statements.push(statement(&allocator));
            }
            // Panic with the buffer non-empty, mid-"parse".
            panic!("boom");
        });
        assert!(result.is_err());
        // The buffers were emptied and returned to the cache by the guard's drop.
        let guard = ScratchGuard::checkout();
        assert!(guard.statements.is_empty());
        assert!(guard.statements.capacity() >= 32);
    }
}
