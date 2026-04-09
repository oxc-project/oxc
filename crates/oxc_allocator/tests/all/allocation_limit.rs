use bumpalo::Bump;

#[test]
fn allocation_limit_trivial() {
    let bump = Bump::with_capacity(0);
    bump.set_allocation_limit(Some(0));

    assert!(bump.try_alloc(5).is_err());
    assert!(bump.allocation_limit().unwrap() >= bump.allocated_bytes());

    bump.set_allocation_limit(None);

    assert!(bump.try_alloc(5).is_ok());
}

#[test]
fn change_allocation_limit_with_live_allocations() {
    let bump = Bump::new();

    bump.set_allocation_limit(Some(512));

    bump.alloc(10);

    assert!(bump.try_alloc([0; 2048]).is_err());

    bump.set_allocation_limit(Some(16384));

    assert!(bump.try_alloc([0; 2048]).is_ok());
    assert!(bump.allocation_limit().unwrap() >= bump.allocated_bytes());
}

#[test]
fn remove_allocation_limit_with_live_allocations() {
    let bump = Bump::new();

    bump.set_allocation_limit(Some(512));

    bump.alloc(10);

    assert!(bump.try_alloc([0; 2048]).is_err());
    assert!(bump.allocation_limit().unwrap() >= bump.allocated_bytes());

    bump.set_allocation_limit(None);

    assert!(bump.try_alloc([0; 2048]).is_ok());
}

#[test]
fn reset_preserves_allocation_limits() {
    let mut bump = Bump::new();

    bump.set_allocation_limit(Some(512));
    bump.reset();

    assert!(bump.try_alloc([0; 2048]).is_err());
    assert!(bump.allocation_limit().unwrap() >= bump.allocated_bytes());
}

#[test]
fn reset_updates_allocated_bytes() {
    let mut bump = Bump::new();

    bump.alloc([0; 1 << 9]);

    // This second allocation should be a big enough one
    // after the first to force a new chunk allocation
    bump.alloc([0; 1 << 9]);

    let allocated_bytes_before_reset = bump.allocated_bytes();

    bump.reset();

    let allocated_bytes_after_reset = bump.allocated_bytes();

    assert!(allocated_bytes_after_reset < allocated_bytes_before_reset);
}

#[test]
fn new_bump_allocated_bytes_is_zero() {
    let bump = Bump::new();

    assert_eq!(bump.allocated_bytes(), 0);
}

#[test]
fn small_allocation_limit() {
    let bump = Bump::new();

    bump.set_allocation_limit(Some(64));
    assert!(bump.try_alloc([0; 1]).is_ok());
}
