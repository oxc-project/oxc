use std::{cell::OnceCell, marker::PhantomData};

use crate::{Buffer, Format, FormatElement, Formatter};

/// Utility trait that allows memorizing the output of a [Format].
/// Useful to avoid re-formatting the same object twice.
pub trait MemoizeFormat<'ast, C> {
    /// Returns a formattable object that memoizes the result of `Format` by cloning.
    /// Mainly useful if the same sub-tree can appear twice in the formatted output because it's
    /// used inside of `if_group_breaks` or `if_group_fits_single_line`.
    fn memoized(self) -> Memoized<'ast, Self, C>
    where
        Self: Sized + Format<'ast, C>,
    {
        Memoized::new(self)
    }
}

impl<'ast, T, C> MemoizeFormat<'ast, C> for T where T: Format<'ast, C> {}

/// Memoizes the output of its inner [Format] to avoid re-formatting a potential expensive object.
#[derive(Debug)]
pub struct Memoized<'ast, F, C> {
    inner: F,
    memory: OnceCell<Option<FormatElement<'ast>>>,
    _marker: PhantomData<C>,
}

impl<'ast, F, C> Memoized<'ast, F, C>
where
    F: Format<'ast, C>,
{
    fn new(inner: F) -> Self {
        Self { inner, memory: OnceCell::new(), _marker: PhantomData }
    }

    /// Gives access to the memoized content.
    ///
    /// Performs the formatting if the content hasn't been formatted at this point.
    pub fn inspect(&self, f: &mut Formatter<'_, 'ast, C>) -> &[FormatElement<'ast>] {
        let result = self.memory.get_or_init(|| f.intern(&self.inner));

        match result.as_ref() {
            Some(FormatElement::Interned(interned)) => interned,
            Some(other) => std::slice::from_ref(other),
            None => &[],
        }
    }
}

impl<'ast, F, C> Format<'ast, C> for Memoized<'ast, F, C>
where
    F: Format<'ast, C>,
{
    fn fmt(&self, f: &mut Formatter<'_, 'ast, C>) {
        let result = self.memory.get_or_init(|| f.intern(&self.inner));

        if let Some(elements) = result {
            f.write_element(elements.clone());
        }
    }
}
