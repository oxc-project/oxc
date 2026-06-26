use oxc_allocator::arena::Arena;

#[test]
fn try_with_capacity_too_large() {
    // Shouldn't panic even though the capacity is too large for a `Layout`.
    let _ = Arena::try_with_capacity(isize::MAX as usize + 1);
}
