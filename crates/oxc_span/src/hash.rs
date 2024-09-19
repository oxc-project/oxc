use std::{
    hash::{Hash, Hasher},
    mem::{discriminant, Discriminant},
};

/// This trait works similarly to [std::hash::Hash] but it gives the liberty of hashing
/// the object's content loosely. This would mean the implementor can skip some parts of
/// the content while calculating the hash.
///
/// As an example, In AST types we ignore fields such as [crate::Span].
pub trait ContentHash {
    fn content_hash<H: Hasher>(&self, state: &mut H);

    /// The default implementation is usually sufficient.
    fn content_hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.content_hash(state);
        }
    }
}

/// Short-Circuting implementation for [Discriminant] since it is used to hash enums.
impl<T> ContentHash for Discriminant<T> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self, state);
    }
}

impl<T: ContentHash> ContentHash for Option<T> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
        if let Some(it) = self {
            ContentHash::content_hash(it, state);
        }
    }
}

impl<'a, T: ContentHash> ContentHash for oxc_allocator::Box<'a, T> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(self.as_ref(), state);
    }
}

impl<'a, T: ContentHash> ContentHash for oxc_allocator::Vec<'a, T> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash_slice(self.as_slice(), state);
    }
}

mod auto_impl_content_hash {
    use super::ContentHash;

    macro_rules! impl_content_hash {
        ($($t:ty)*) => {
            $(
                impl ContentHash for $t {
                    fn content_hash<H: std::hash::Hasher>(&self, state: &mut H) {
                        std::hash::Hash::hash(self, state);
                    }
                }
            )*
        };
    }

    impl_content_hash! {
        char &str
        bool isize usize
        u8 u16 u32 u64 u128
        i8 i16 i32 i64 i128
    }
}
