use std::{alloc::Layout, cmp::max, iter, mem, ptr};

use oxc_allocator::arena::Arena;

#[test]
fn alloc_slice_fill_zero() {
    let b = Arena::new();
    let u8_layout = Layout::new::<u8>();

    let ptr1 = b.alloc_layout(u8_layout);

    struct MyZeroSizedType;

    b.alloc_slice_copy::<u64>(&[]);
    b.alloc_slice_clone::<String>(&[]);
    b.alloc_slice_fill_with::<String, _>(0, |_| panic!("should not happen"));
    b.alloc_slice_fill_copy(0, 42u64);
    b.alloc_slice_fill_clone(0, &"hello".to_string());
    b.alloc_slice_fill_default::<String>(0);
    let ptr2 = b.alloc(MyZeroSizedType);
    let alignment = max(mem::align_of::<u64>(), mem::align_of::<String>());
    assert_eq!(ptr1.as_ptr() as usize & !(alignment - 1), ptr2 as *mut _ as usize);

    let ptr3 = b.alloc_layout(u8_layout);
    dbg!(ptr2 as *mut _);
    dbg!(ptr3);
    assert_eq!(
        ptr2 as *mut _ as usize,
        (ptr3.as_ptr() as usize) + max(b.min_align(), u8_layout.align()),
    );
}

#[test]
#[should_panic(expected = "out of memory")]
fn alloc_slice_overflow() {
    let b = Arena::new();

    b.alloc_slice_fill_default::<u64>(usize::max_value());
}

// Constants and helper for the `*_updates_cursor` tests.
const A: u64 = 0x1111_1111_1111_1111;
const B: u64 = 0x2222_2222_2222_2222;
const C: u64 = 0x3333_3333_3333_3333;

fn addr<T>(r: &T) -> usize {
    ptr::from_ref(r).addr()
}

/// `alloc_slice_fill_copy` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
#[test]
fn alloc_slice_fill_copy_updates_cursor() {
    // 2 consecutive allocations sit directly below one another (the arena bumps downwards),
    // and the values don't overwrite each other
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_fill_copy(2, A);
        let b = arena.alloc_slice_fill_copy(2, B);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0], a[1], b[0], b[1]), (A, A, B, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

/// `alloc_slice_fill_with` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
///
/// It must commit *before* calling the closure, so a closure which itself allocates from
/// the same arena does not get overlapping memory.
#[test]
fn alloc_slice_fill_with_updates_cursor() {
    // 2 consecutive allocations sit directly below one another (the arena bumps downwards),
    // and the values don't overwrite each other
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_fill_with(2, |i| A + i as u64);
        let b = arena.alloc_slice_fill_with(2, |i| B + i as u64);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0], a[1], b[0], b[1]), (A, A + 1, B, B + 1));
    }

    // Closure which performs another allocation from the same arena.
    // The inner allocations must sit below the outer slice - no overlap.
    // Each element's value is that inner allocation's address.
    // The closure runs once per element, interleaved with the element writes,
    // so the 1st inner allocation is 8 bytes below the slice, the 2nd is 16 bytes below.
    fn check_reentrant(arena: &Arena) {
        let outer = arena.alloc_slice_fill_with(2, |_| {
            let inner = arena.alloc(C);
            assert_eq!(*inner, C);
            addr(inner) as u64
        });
        assert_eq!(outer[0], (addr(&outer[0]) - 8) as u64);
        assert_eq!(outer[1], (addr(&outer[0]) - 16) as u64);
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    check_reentrant(&Arena::with_capacity(0x1000));

    // Slow path
    check(&Arena::new());
    check_reentrant(&Arena::new());
}

/// `alloc_slice_fill_clone` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
///
/// It must commit *before* cloning the elements, so a `Clone` impl which itself allocates from
/// the same arena does not get overlapping memory.
#[test]
fn alloc_slice_fill_clone_updates_cursor() {
    #[derive(Clone)]
    struct Val(u64);

    // 2 consecutive allocations sit directly below one another (the arena bumps downwards),
    // and the values don't overwrite each other
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_fill_clone(2, &Val(A));
        let b = arena.alloc_slice_fill_clone(2, &Val(B));
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0].0, a[1].0, b[0].0, b[1].0), (A, A, B, B));
    }

    // `Clone` impl which performs another allocation from the same arena.
    // The inner allocations must sit below the outer slice - no overlap.
    // Each element records its inner allocation's address.
    // `Reentrant` is 16 bytes, so the outer slice is 32 bytes. Clones run interleaved with
    // the element writes, so the 1st inner allocation is 8 bytes below the slice, the 2nd 16 below.
    struct Reentrant<'a> {
        arena: &'a Arena,
        inner_addr: usize,
    }

    impl Clone for Reentrant<'_> {
        fn clone(&self) -> Self {
            let inner = self.arena.alloc(C);
            assert_eq!(*inner, C);
            Self { arena: self.arena, inner_addr: addr(inner) }
        }
    }

    fn check_reentrant(arena: &Arena) {
        let value = Reentrant { arena, inner_addr: 0 };
        let outer = arena.alloc_slice_fill_clone(2, &value);
        assert_eq!(outer[0].inner_addr, addr(&outer[0]) - 8);
        assert_eq!(outer[1].inner_addr, addr(&outer[0]) - 16);
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    check_reentrant(&Arena::with_capacity(0x1000));

    // Slow path
    check(&Arena::new());
    check_reentrant(&Arena::new());
}

/// `alloc_slice_fill_iter` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
///
/// It must commit *before* consuming the iterator, so an iterator which itself allocates from
/// the same arena does not get overlapping memory.
#[test]
fn alloc_slice_fill_iter_updates_cursor() {
    // 2 consecutive allocations sit directly below one another (the arena bumps downwards),
    // and the values don't overwrite each other
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_fill_iter([A, A + 1]);
        let b = arena.alloc_slice_fill_iter([B, B + 1]);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0], a[1], b[0], b[1]), (A, A + 1, B, B + 1));
    }

    // Iterator which performs another allocation from the same arena.
    // The inner allocations must sit below the outer slice - no overlap.
    // Each element's value is that inner allocation's address.
    // The iterator is consumed interleaved with the element writes,
    // so the 1st inner allocation is 8 bytes below the slice, the 2nd is 16 bytes below.
    fn check_reentrant(arena: &Arena) {
        let outer = arena.alloc_slice_fill_iter(
            iter::repeat_with(|| {
                let inner = arena.alloc(C);
                assert_eq!(*inner, C);
                addr(inner) as u64
            })
            .take(2),
        );
        assert_eq!(outer[0], (addr(&outer[0]) - 8) as u64);
        assert_eq!(outer[1], (addr(&outer[0]) - 16) as u64);
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    check_reentrant(&Arena::with_capacity(0x1000));

    // Slow path
    check(&Arena::new());
    check_reentrant(&Arena::new());
}

/// `alloc_slice_fill_default` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
#[test]
fn alloc_slice_fill_default_updates_cursor() {
    // 2 consecutive allocations sit directly below one another (the arena bumps downwards)
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_fill_default::<u64>(2);
        let b = arena.alloc_slice_fill_default::<u64>(2);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0], a[1], b[0], b[1]), (0, 0, 0, 0));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}
