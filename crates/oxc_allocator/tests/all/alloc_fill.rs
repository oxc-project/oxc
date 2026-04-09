use bumpalo::Bump;
use std::alloc::Layout;
use std::cmp;
use std::iter::repeat;
use std::mem;

#[test]
fn alloc_slice_fill_zero() {
    let b = Bump::new();
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
    let alignment = cmp::max(mem::align_of::<u64>(), mem::align_of::<String>());
    assert_eq!(
        ptr1.as_ptr() as usize & !(alignment - 1),
        ptr2 as *mut _ as usize
    );

    let ptr3 = b.alloc_layout(u8_layout);
    dbg!(ptr2 as *mut _);
    dbg!(ptr3);
    assert_eq!(
        ptr2 as *mut _ as usize,
        (ptr3.as_ptr() as usize) + b.min_align().max(u8_layout.align()),
    );
}

#[test]
fn alloc_slice_try_fill_with_succeeds() {
    let b = Bump::new();
    let res: Result<&mut [usize], ()> = b.alloc_slice_try_fill_with(100, |n| Ok(n));
    assert_eq!(res.map(|arr| arr[50]), Ok(50));
}

#[test]
fn alloc_slice_try_fill_with_fails() {
    let b = Bump::new();
    let res: Result<&mut [u16], ()> =
        b.alloc_slice_try_fill_with(1000, |n| if n == 100 { Err(()) } else { Ok(42) });
    assert_eq!(res, Err(()));
}

#[test]
fn alloc_slice_try_fill_iter_succeeds() {
    let b = Bump::new();
    let elems = repeat(42).take(10).collect::<Vec<_>>();
    let res: Result<&mut [u16], ()> = b.alloc_slice_try_fill_iter(elems.into_iter().map(Ok));
    assert_eq!(res.map(|arr| arr[5]), Ok(42));
}

#[test]
fn alloc_slice_try_fill_iter_fails() {
    let b = Bump::new();
    let elems = repeat(()).take(10).collect::<Vec<_>>();
    let res: Result<&mut [u16], ()> = b.alloc_slice_try_fill_iter(elems.into_iter().map(Err));
    assert_eq!(res, Err(()));
}

#[test]
#[should_panic(expected = "out of memory")]
fn alloc_slice_overflow() {
    let b = Bump::new();

    b.alloc_slice_fill_default::<u64>(usize::max_value());
}
