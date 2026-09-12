// All of these try_alloc_with tests will fail with "fatal runtime error: stack overflow" unless LLVM
// manages to optimize the stack writes away.
//
// We only run them when debug_assertions are not set, as we expect them to fail outside release
// mode.

use std::ptr;

use oxc_allocator::arena::Arena;

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn try_alloc_with_large_array() {
    let b = Arena::new();

    b.try_alloc_with(|| [4u8; 10_000_000]).unwrap();
}

#[allow(dead_code)]
struct LargeStruct {
    small: usize,
    big1: [u8; 20_000_000],
    big2: [u8; 20_000_000],
    big3: [u8; 20_000_000],
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn try_alloc_with_large_struct() {
    let b = Arena::new();

    b.try_alloc_with(|| LargeStruct {
        small: 1,
        big1: [2; 20_000_000],
        big2: [3; 20_000_000],
        big3: [4; 20_000_000],
    })
    .unwrap();
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn try_alloc_with_large_tuple() {
    let b = Arena::new();

    b.try_alloc_with(|| {
        (
            1u32,
            LargeStruct {
                small: 2,
                big1: [3; 20_000_000],
                big2: [4; 20_000_000],
                big3: [5; 20_000_000],
            },
        )
    })
    .unwrap();
}

enum LargeEnum {
    Small,
    #[allow(dead_code)]
    Large([u8; 10_000_000]),
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn try_alloc_with_large_enum() {
    let b = Arena::new();

    b.try_alloc_with(|| LargeEnum::Small).unwrap();
}

/// `try_alloc_with` must commit the bump pointer after each allocation, on both the fast path
/// and the slow path (1st allocation in an empty arena has to allocate a chunk).
///
/// It must commit *before* calling the closure, so a closure which itself allocates from
/// the same arena does not get overlapping memory.
#[test]
fn try_alloc_with_updates_cursor() {
    const A: u64 = 0x1111_1111_1111_1111;
    const B: u64 = 0x2222_2222_2222_2222;
    const C: u64 = 0x3333_3333_3333_3333;

    fn addr<T>(r: &T) -> usize {
        ptr::from_ref(r).addr()
    }

    // 2 consecutive allocations sit directly below one another (the arena bumps downwards),
    // and the values don't overwrite each other
    fn check(arena: &Arena) {
        let a = arena.try_alloc_with(|| A).unwrap();
        let b = arena.try_alloc_with(|| B).unwrap();
        assert_eq!(addr(b), addr(a) - 8);
        assert_eq!((*a, *b), (A, B));
    }

    // Closure which performs another allocation from the same arena.
    // The inner allocation must sit below the outer slot - no overlap.
    // The outer slot's value is the inner allocation's address.
    fn check_reentrant(arena: &Arena) {
        let outer = arena
            .try_alloc_with(|| {
                let inner = arena.alloc(C);
                assert_eq!(*inner, C);
                addr(inner) as u64
            })
            .unwrap();
        assert_eq!(*outer, (addr(outer) - 8) as u64);
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    check_reentrant(&Arena::with_capacity(0x1000));

    // Slow path
    check(&Arena::new());
    check_reentrant(&Arena::new());
}
