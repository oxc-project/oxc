use std::{alloc::Layout, cmp::max, mem};

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
