//! Reusable parser-owned storage for AST lists under construction.

use std::marker::PhantomData;

use oxc_allocator::{Allocator, ArenaVec, GetAllocator};
use oxc_ast::ast::*;

use crate::{ParserConfig, ParserImpl};

pub struct ScratchStack<T> {
    items: Vec<T>,
}

impl<T> Default for ScratchStack<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

/// Position of an active list in parser scratch storage.
///
/// This does not borrow [`ParserScratch`], so parsing can recurse while a list is active.
pub struct ScratchMark<T> {
    start: usize,
    #[cfg(debug_assertions)]
    depth: usize,
    marker: PhantomData<fn() -> T>,
}

pub trait ScratchElement<'a>: Sized {
    /// Preserve the old arena vector's geometric spare capacity when downstream passes grow it.
    const PRESERVE_CAPACITY_HEADROOM: bool = false;

    fn stack<'s>(scratch: &'s mut ParserScratch<'a>) -> &'s mut ScratchStack<Self>;
}

macro_rules! define_parser_scratch {
    ($($(#[$meta:meta])* $field:ident: $ty:ty $(=> $preserve_headroom:expr)?),+ $(,)?) => {
        #[derive(Default)]
        pub struct ParserScratch<'a> {
            #[cfg(debug_assertions)]
            active_depth: usize,
            $($(#[$meta])* $field: ScratchStack<$ty>,)+
        }

        $(
            $(#[$meta])*
            impl<'a> ScratchElement<'a> for $ty {
                $(const PRESERVE_CAPACITY_HEADROOM: bool = $preserve_headroom;)?

                #[inline]
                fn stack<'s>(
                    scratch: &'s mut ParserScratch<'a>,
                ) -> &'s mut ScratchStack<Self> {
                    &mut scratch.$field
                }
            }
        )+

        impl ParserScratch<'_> {
            #[inline]
            pub(crate) fn debug_assert_empty(&self) {
                #[cfg(debug_assertions)]
                {
                    debug_assert_eq!(self.active_depth, 0, "scratch list was not finished");
                    $($(#[$meta])*
                        debug_assert!(
                            self.$field.items.is_empty(),
                            "scratch buffer `{}` was not rewound",
                            stringify!($field),
                        );
                    )+
                }
            }
        }
    };
}

define_parser_scratch! {
    statements: Statement<'a>,
    switch_cases: SwitchCase<'a>,
    // Transformers insert synthesized elements into parsed class bodies.
    class_elements: ClassElement<'a> => true,
    expressions: Expression<'a>,
    array_expression_elements: ArrayExpressionElement<'a>,
    object_properties: ObjectPropertyKind<'a>,
    arguments: Argument<'a>,
    binding_properties: BindingProperty<'a>,
    array_binding_elements: Option<BindingPattern<'a>>,
    ts_enum_members: TSEnumMember<'a>,
    ts_signatures: TSSignature<'a>,
    ts_type_parameters: TSTypeParameter<'a>,
    ts_types: TSType<'a>,
    ts_tuple_elements: TSTupleElement<'a>,
    ts_index_signature_names: TSIndexSignatureName<'a>,
    import_attributes: ImportAttribute<'a>,
    export_specifiers: ExportSpecifier<'a>,
    directives: Directive<'a>,
    variable_declarators: VariableDeclarator<'a>,
    formal_parameters: FormalParameter<'a>,
    jsx_children: JSXChild<'a>,
    jsx_attributes: JSXAttributeItem<'a>,
    ts_interface_heritage: TSInterfaceHeritage<'a>,
    import_declaration_specifiers: ImportDeclarationSpecifier<'a>,
    #[cfg(test)]
    test_values: u64,
    #[cfg(test)]
    test_other_values: i64,
}

impl<'a> ParserScratch<'a> {
    #[inline]
    fn begin<T: ScratchElement<'a>>(&mut self) -> ScratchMark<T> {
        let start = T::stack(self).items.len();

        #[cfg(debug_assertions)]
        {
            self.active_depth += 1;
            ScratchMark { start, depth: self.active_depth, marker: PhantomData }
        }

        #[cfg(not(debug_assertions))]
        ScratchMark { start, marker: PhantomData }
    }

    #[inline]
    fn push<T: ScratchElement<'a>>(&mut self, mark: &ScratchMark<T>, value: T) {
        self.assert_active(mark);
        T::stack(self).items.push(value);
    }

    #[inline]
    fn slice<T: ScratchElement<'a>>(&mut self, mark: &ScratchMark<T>) -> &[T] {
        self.assert_active(mark);
        &T::stack(self).items[mark.start..]
    }

    #[inline]
    fn finish<T: ScratchElement<'a>>(
        &mut self,
        mark: ScratchMark<T>,
        allocator: &'a Allocator,
    ) -> ArenaVec<'a, T> {
        self.finish_with_capacity(mark, allocator, T::PRESERVE_CAPACITY_HEADROOM)
    }

    #[inline]
    #[expect(clippy::needless_pass_by_value, reason = "a mark can only be finished once")]
    fn finish_with_capacity<T: ScratchElement<'a>>(
        &mut self,
        mark: ScratchMark<T>,
        allocator: &'a Allocator,
        preserve_capacity_headroom: bool,
    ) -> ArenaVec<'a, T> {
        self.assert_active(&mark);
        let items = &mut T::stack(self).items;
        assert!(mark.start <= items.len(), "scratch mark is outside its typed buffer");
        let len = items.len() - mark.start;
        let capacity =
            if preserve_capacity_headroom && len > 0 { len.next_power_of_two() } else { len };
        let mut list = ArenaVec::with_capacity_in(capacity, &allocator);
        list.extend(items.drain(mark.start..));

        self.finish_mark(&mark);
        list
    }

    #[inline]
    #[expect(clippy::needless_pass_by_value, reason = "a mark can only be cancelled once")]
    fn cancel<T: ScratchElement<'a>>(&mut self, mark: ScratchMark<T>) {
        self.assert_active(&mark);
        T::stack(self).items.truncate(mark.start);
        self.finish_mark(&mark);
    }

    #[inline]
    fn assert_active<T>(&self, mark: &ScratchMark<T>) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            mark.depth, self.active_depth,
            "scratch lists must be accessed in LIFO order"
        );

        #[cfg(not(debug_assertions))]
        let _ = mark;
    }

    #[inline]
    fn finish_mark<T>(&mut self, mark: &ScratchMark<T>) {
        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(
                mark.depth, self.active_depth,
                "scratch lists must be finished in LIFO order"
            );
            self.active_depth -= 1;
        }

        #[cfg(not(debug_assertions))]
        let _ = mark;
    }
}

impl<'a, C: ParserConfig> ParserImpl<'a, C> {
    #[inline]
    pub(crate) fn start_scratch<T: ScratchElement<'a>>(&mut self) -> ScratchMark<T> {
        self.scratch.begin()
    }

    #[inline]
    pub(crate) fn scratch_push<T: ScratchElement<'a>>(&mut self, mark: &ScratchMark<T>, value: T) {
        self.scratch.push(mark, value);
    }

    #[inline]
    pub(crate) fn scratch_slice<T: ScratchElement<'a>>(&mut self, mark: &ScratchMark<T>) -> &[T] {
        self.scratch.slice(mark)
    }

    #[inline]
    pub(crate) fn finish_scratch<T: ScratchElement<'a>>(
        &mut self,
        mark: ScratchMark<T>,
    ) -> ArenaVec<'a, T> {
        self.scratch.finish(mark, self.ast.allocator())
    }

    #[inline]
    pub(crate) fn finish_scratch_with_capacity_headroom<T: ScratchElement<'a>>(
        &mut self,
        mark: ScratchMark<T>,
    ) -> ArenaVec<'a, T> {
        self.scratch.finish_with_capacity(mark, self.ast.allocator(), true)
    }

    #[inline]
    pub(crate) fn cancel_scratch<T: ScratchElement<'a>>(&mut self, mark: ScratchMark<T>) {
        self.scratch.cancel(mark);
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::ParserScratch;

    #[test]
    fn collects_exact_arena_vector_and_reuses_storage() {
        let allocator = Allocator::default();
        let mut scratch = ParserScratch::default();

        let mark = scratch.begin::<u64>();
        for value in 0..128 {
            scratch.push(&mark, value);
        }
        let capacity = scratch.test_values.items.capacity();
        let values = scratch.finish(mark, &allocator);

        assert_eq!(values.as_slice(), (0..128).collect::<Vec<_>>());
        assert_eq!(values.capacity(), values.len());
        assert!(scratch.test_values.items.is_empty());
        assert_eq!(scratch.test_values.items.capacity(), capacity);
    }

    #[test]
    fn preserves_requested_capacity_headroom() {
        let allocator = Allocator::default();
        let mut scratch = ParserScratch::default();

        let mark = scratch.begin::<u64>();
        scratch.push(&mark, 1);
        scratch.push(&mark, 2);
        scratch.push(&mark, 3);
        let scratch_capacity = scratch.test_values.items.capacity();
        let values = scratch.finish_with_capacity(mark, &allocator, true);

        assert_eq!(values.as_slice(), &[1, 2, 3]);
        assert_eq!(values.capacity(), 4);
        assert!(scratch.test_values.items.is_empty());
        assert_eq!(scratch.test_values.items.capacity(), scratch_capacity);
    }

    #[test]
    fn supports_nested_lists() {
        let allocator = Allocator::default();
        let mut scratch = ParserScratch::default();

        let outer = scratch.begin::<u64>();
        scratch.push(&outer, 1);
        let inner = scratch.begin::<u64>();
        scratch.push(&inner, 2);
        let inner_values = scratch.finish(inner, &allocator);
        scratch.push(&outer, 3);
        let outer_values = scratch.finish(outer, &allocator);

        assert_eq!(inner_values.as_slice(), &[2]);
        assert_eq!(outer_values.as_slice(), &[1, 3]);
    }

    #[test]
    fn supports_nested_lists_with_different_types() {
        let allocator = Allocator::default();
        let mut scratch = ParserScratch::default();

        let outer = scratch.begin::<u64>();
        scratch.push(&outer, 1);
        let inner = scratch.begin::<i64>();
        scratch.push(&inner, -2);
        assert_eq!(scratch.finish(inner, &allocator).as_slice(), &[-2]);
        scratch.push(&outer, 3);
        assert_eq!(scratch.finish(outer, &allocator).as_slice(), &[1, 3]);
        scratch.debug_assert_empty();
    }

    #[test]
    fn cancel_discards_only_the_active_list() {
        let allocator = Allocator::default();
        let mut scratch = ParserScratch::default();

        let outer = scratch.begin::<u64>();
        scratch.push(&outer, 1);
        let inner = scratch.begin::<u64>();
        scratch.push(&inner, 2);
        scratch.cancel(inner);
        assert_eq!(scratch.slice(&outer), &[1]);
        assert_eq!(scratch.finish(outer, &allocator).as_slice(), &[1]);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "scratch lists must be accessed in LIFO order")]
    fn rejects_non_lifo_access() {
        let mut scratch = ParserScratch::default();

        let outer = scratch.begin::<u64>();
        let _inner = scratch.begin::<u64>();
        scratch.push(&outer, 1);
    }
}
