//! Utility functions for `Arena`.

use std::{alloc::Layout, hint::unreachable_unchecked};

pub use super::bumpalo_alloc::AllocErr;

#[inline]
pub fn is_pointer_aligned_to<T>(pointer: *mut T, align: usize) -> bool {
    debug_assert!(align.is_power_of_two());

    let pointer = pointer as usize;
    let pointer_aligned = round_down_to(pointer, align);
    pointer == pointer_aligned
}

#[inline]
pub const fn round_up_to(n: usize, divisor: usize) -> Option<usize> {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    match n.checked_add(divisor - 1) {
        Some(x) => Some(x & !(divisor - 1)),
        None => None,
    }
}

/// Like `round_up_to` but turns overflow into undefined behavior rather than
/// returning `None`.
#[inline]
pub unsafe fn round_up_to_unchecked(n: usize, divisor: usize) -> usize {
    if let Some(x) = round_up_to(n, divisor) {
        x
    } else {
        debug_assert!(false, "round_up_to_unchecked failed");
        unsafe { unreachable_unchecked() }
    }
}

#[inline]
pub fn round_down_to(n: usize, divisor: usize) -> usize {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    n & !(divisor - 1)
}

/// Same as `round_down_to` but preserves pointer provenance.
#[inline]
pub fn round_mut_ptr_down_to(ptr: *mut u8, divisor: usize) -> *mut u8 {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    ptr.wrapping_sub(ptr as usize & (divisor - 1))
}

#[inline]
pub unsafe fn round_mut_ptr_up_to_unchecked(ptr: *mut u8, divisor: usize) -> *mut u8 {
    debug_assert!(divisor > 0);
    debug_assert!(divisor.is_power_of_two());
    unsafe {
        let aligned = round_up_to_unchecked(ptr as usize, divisor);
        let delta = aligned - (ptr as usize);
        ptr.add(delta)
    }
}

/// Wrapper around `Layout::from_size_align` that adds debug assertions.
#[inline]
pub fn layout_from_size_align(size: usize, align: usize) -> Result<Layout, AllocErr> {
    Layout::from_size_align(size, align).map_err(|_| AllocErr)
}

#[inline(never)]
#[cold]
pub fn oom() -> ! {
    panic!("out of memory")
}
