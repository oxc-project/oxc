#[cfg(any(test, feature = "serialize"))]
mod ser;

#[cfg(any(test, feature = "serialize"))]
pub use ser::*;

#[cfg(not(any(test, feature = "serialize")))]
pub trait ToRust {}

#[doc(hidden)]
pub mod private {
    pub trait ToChainIter: Sized {
        type Output;
        fn to_chain(self) -> impl IntoIterator<Item = Self::Output>;
    }

    impl<T> ToChainIter for T
    where
        T: crate::ToRust,
    {
        type Output = Self;

        fn to_chain(self) -> impl IntoIterator<Item = Self::Output> {
            [self]
        }
    }

    impl<E, T> ToChainIter for oxc_allocator::Vec<'_, E>
    where
        T: crate::ToRust,
        E: ToChainIter<Output = T>,
    {
        type Output = T;

        fn to_chain(self) -> impl IntoIterator<Item = Self::Output> {
            self.into_iter().flat_map(ToChainIter::to_chain)
        }
    }
}
