//! Tests for `Arena::alloc_uninit`.

use std::{mem::MaybeUninit, ptr};

use oxc_allocator::arena::Arena;

#[test]
fn write_and_read_back() {
    let arena = Arena::new();

    let n = arena.alloc_uninit::<u32>().write(123);
    assert_eq!(*n, 123);

    let arr = arena.alloc_uninit::<[u8; 20]>().write([7; 20]);
    assert_eq!(arr, &[7; 20]);

    let tuple = arena.alloc_uninit::<(u8, u64)>().write((1, 2));
    assert_eq!(*tuple, (1, 2));
}

#[test]
fn is_aligned_for_type() {
    // Field is never read - it is only there to give the type a size.
    // `u128` is align 16 on the platforms we run on, but not on all of them, so this covers
    // an over-aligned type independently of the target.
    #[expect(dead_code)]
    #[repr(align(16))]
    struct Align16(u8);

    // A 1-byte allocation before each, so the cursor is left at an offset which is not already
    // aligned for the type which follows
    fn check<T>(arena: &Arena) {
        for _ in 0..4 {
            arena.alloc_uninit::<u8>();
            let slot = arena.alloc_uninit::<T>();
            assert!(
                addr(slot).is_multiple_of(align_of::<T>()),
                "address {:#x} is not aligned to {}",
                addr(slot),
                align_of::<T>()
            );
        }
    }

    let arena = Arena::new();
    check::<u16>(&arena);
    check::<u32>(&arena);
    check::<u64>(&arena);
    check::<u128>(&arena);
    check::<Align16>(&arena);
}

#[test]
fn respects_min_align() {
    // A `u8` is placed at an address which is a multiple of `MIN_ALIGN`, not just of 1
    fn check<const MIN_ALIGN: usize>() {
        let arena = Arena::<MIN_ALIGN>::with_min_align();
        for _ in 0..8 {
            let slot = arena.alloc_uninit::<u8>();
            assert!(
                addr(slot).is_multiple_of(MIN_ALIGN),
                "address {:#x} is not aligned to MIN_ALIGN {MIN_ALIGN}",
                addr(slot)
            );
        }
    }

    check::<2>();
    check::<4>();
    check::<8>();
    check::<16>();
}

#[test]
fn allocations_do_not_overlap() {
    let arena = Arena::new();

    // Interleaved with `alloc`, which advances the same cursor
    let mut slots = Vec::new();
    for i in 0..64u64 {
        slots.push(arena.alloc_uninit::<u64>().write(i));
        arena.alloc(i);
    }

    for (i, slot) in slots.iter().enumerate() {
        assert_eq!(**slot, i as u64);
    }
}

#[test]
fn zero_sized_type() {
    let arena = Arena::new();

    arena.alloc_uninit::<()>().write(());

    let empty = arena.alloc_uninit::<[u32; 0]>().write([]);
    assert_eq!(empty, &[]);

    // A ZST is still placed at an address aligned for its type
    let slot = arena.alloc_uninit::<[u32; 0]>();
    assert!(addr(slot).is_multiple_of(align_of::<[u32; 0]>()));
}

#[test]
fn spans_two_chunks() {
    let mut arena = Arena::with_capacity(64);

    let first = arena.alloc_uninit::<[u32; 16]>().write([1; 16]);
    // Larger than the whole first chunk, so this forces the arena to grow
    let second = arena.alloc_uninit::<[u32; 256]>().write([2; 256]);

    // Growing leaves data already in the arena where it is, so the first is still intact
    assert_eq!(*first, [1; 16]);
    assert_eq!(*second, [2; 256]);

    assert_eq!(arena.iter_allocated_chunks().count(), 2);
}

/// Address of the memory a slot covers.
fn addr<T>(slot: &MaybeUninit<T>) -> usize {
    ptr::from_ref(slot).addr()
}
