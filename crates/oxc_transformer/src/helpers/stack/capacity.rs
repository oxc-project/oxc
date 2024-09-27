use std::mem::{align_of, size_of};

/// Trait for defining maximum and default capacity of stacks.
///
/// `MAX_CAPACITY` and `MAX_CAPACITY_BYTES` being calculated correctly is required for soundness
/// of stack types.
pub trait StackCapacity {
    /// Type that the stack contains
    type Item: Sized;

    /// Maximum capacity of stack.
    ///
    /// This is guaranteed to be a legal size for a stack of `Item`s, without exceeding Rust's
    /// allocation size limits.
    ///
    /// From [`std::alloc::Layout`]'s docs:
    /// > size, when rounded up to the nearest multiple of align, must not overflow `isize`
    /// > (i.e., the rounded value must be less than or equal to `isize::MAX`).
    const MAX_CAPACITY: usize = {
        // This assertion is not needed as next line will cause a compile failure anyway
        // if `size_of::<Self::Item>() == 0`, due to division by zero.
        // But keep it anyway as soundness depends on it.
        assert!(size_of::<Self::Item>() > 0, "Zero sized types are not supported");
        // As it's always true that `size_of::<T>() >= align_of::<T>()` and `/` rounds down,
        // this fulfills `Layout`'s alignment requirement
        let max_capacity = isize::MAX as usize / size_of::<Self::Item>();
        assert!(max_capacity > 0);
        max_capacity
    };

    /// Maximum capacity of stack in bytes
    const MAX_CAPACITY_BYTES: usize = {
        let capacity_bytes = Self::MAX_CAPACITY * size_of::<Self::Item>();
        // Just double-checking `Layout`'s alignment requirement is fulfilled
        assert!(capacity_bytes <= isize::MAX as usize + 1 - align_of::<Self::Item>());
        capacity_bytes
    };

    /// Default capacity of stack.
    ///
    /// Same defaults as [`std::vec::Vec`] uses.
    const DEFAULT_CAPACITY: usize = {
        // It's impossible for this to exceed `MAX_CAPACITY` because `size_of::<T>() >= align_of::<T>()`
        match size_of::<Self::Item>() {
            1 => 8,
            size if size <= 1024 => 4,
            _ => 1,
        }
    };

    /// Default capacity of stack in bytes
    const DEFAULT_CAPACITY_BYTES: usize = Self::DEFAULT_CAPACITY * size_of::<Self::Item>();
}

#[cfg(test)]
#[expect(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    const ISIZE_MAX: usize = isize::MAX as usize;
    const ISIZE_MAX_PLUS_ONE: usize = ISIZE_MAX + 1;

    #[test]
    fn bool() {
        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = bool;
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, ISIZE_MAX);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 8);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 8);
    }

    #[test]
    fn u64() {
        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = u64;
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 8);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 8);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 8);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 32);
    }

    #[test]
    fn u32_pair() {
        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = [u32; 2];
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 8);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 8);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 32);
    }

    #[test]
    fn u32_triple() {
        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = [u32; 3];
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 12);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 12);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 48);
    }

    #[test]
    fn large_low_alignment() {
        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = [u16; 1000];
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 2000);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 2000);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 2);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 1);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 2000);
    }

    #[test]
    fn large_high_alignment() {
        #[repr(align(4096))]
        #[expect(dead_code)]
        struct TestItem(u8);

        struct TestStack;
        impl StackCapacity for TestStack {
            type Item = TestItem;
        }
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 4096);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 4096);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4096);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 1);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 4096);
    }
}
