use oxc_allocator::arena::Arena;

#[test]
fn allocation_limit_trivial() {
    let arena = Arena::with_capacity(0);
    arena.set_allocation_limit(Some(0));

    assert!(arena.try_alloc(5).is_err());
    assert!(arena.allocation_limit().unwrap() >= arena.allocated_bytes());

    arena.set_allocation_limit(None);

    assert!(arena.try_alloc(5).is_ok());
}

#[test]
fn change_allocation_limit_with_live_allocations() {
    let arena = Arena::with_capacity(448);

    arena.set_allocation_limit(Some(512));

    arena.alloc(10);

    assert!(arena.try_alloc([0; 2048]).is_err());

    arena.set_allocation_limit(Some(32768));

    assert!(arena.try_alloc([0; 2048]).is_ok());
    assert!(arena.allocation_limit().unwrap() >= arena.allocated_bytes());
}

#[test]
fn remove_allocation_limit_with_live_allocations() {
    let arena = Arena::new();

    arena.set_allocation_limit(Some(512));

    arena.alloc(10);

    assert!(arena.try_alloc([0; 2048]).is_err());
    assert!(arena.allocation_limit().unwrap() >= arena.allocated_bytes());

    arena.set_allocation_limit(None);

    assert!(arena.try_alloc([0; 2048]).is_ok());
}

#[test]
fn reset_preserves_allocation_limits() {
    let mut arena = Arena::new();

    arena.set_allocation_limit(Some(512));
    arena.reset();

    assert!(arena.try_alloc([0; 2048]).is_err());
    assert!(arena.allocation_limit().unwrap() >= arena.allocated_bytes());
}

#[test]
fn reset_updates_allocated_bytes() {
    let mut arena = Arena::with_capacity(512);

    arena.alloc([0; 1 << 9]);

    // This second allocation should be a big enough one
    // after the first to force a new chunk allocation
    arena.alloc([0; 1 << 9]);

    let allocated_bytes_before_reset = arena.allocated_bytes();

    arena.reset();

    let allocated_bytes_after_reset = arena.allocated_bytes();

    assert!(allocated_bytes_after_reset < allocated_bytes_before_reset);
}

#[test]
fn new_arena_allocated_bytes_is_zero() {
    let arena = Arena::new();

    assert_eq!(arena.allocated_bytes(), 0);
}

#[test]
fn small_allocation_limit() {
    let arena = Arena::new();

    arena.set_allocation_limit(Some(64));
    assert!(arena.try_alloc([0; 1]).is_ok());
}
