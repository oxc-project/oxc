use std::cell::OnceCell;

use super::{Buffer, prelude::*};

/// Utility trait that allows memorizing the output of a [Format].
/// Useful to avoid re-formatting the same object twice.
pub trait MemoizeFormat<'a, C> {
    /// Returns a formattable object that memoizes the result of `Format` by cloning.
    /// Mainly useful if the same sub-tree can appear twice in the formatted output because it's
    /// used inside of `if_group_breaks` or `if_group_fits_single_line`.
    fn memoized(self) -> Memoized<'a, Self>
    where
        Self: Sized + Format<'a, C>,
    {
        Memoized::new(self)
    }
}

impl<C, T> MemoizeFormat<'_, C> for T {}

/// Memoizes the output of its inner [Format] to avoid re-formatting a potential expensive object.
#[derive(Debug)]
pub struct Memoized<'ast, F> {
    inner: F,
    memory: OnceCell<Option<FormatElement<'ast>>>,
}

impl<F> Memoized<'_, F> {
    pub fn new(inner: F) -> Self {
        Self { inner, memory: OnceCell::new() }
    }
}

impl<'ast, F> Memoized<'ast, F> {
    /// Gives access to the memoized content.
    ///
    /// Performs the formatting if the content hasn't been formatted at this point.
    pub fn inspect<C>(&self, f: &mut Formatter<'_, 'ast, C>) -> &[FormatElement<'ast>]
    where
        F: Format<'ast, C>,
    {
        let result = self.memory.get_or_init(|| f.intern(&self.inner));

        match result.as_ref() {
            Some(FormatElement::Interned(interned)) => interned,
            Some(other) => std::slice::from_ref(other),
            None => &[],
        }
    }
}

impl<'ast, C, F> Format<'ast, C> for Memoized<'ast, F>
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
