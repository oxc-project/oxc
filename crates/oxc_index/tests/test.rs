#![allow(
    clippy::assertions_on_constants,
    clippy::eq_op,
    clippy::uninlined_format_args,
    clippy::should_panic_without_expect,
    clippy::cast_possible_truncation
)]

use oxc_index::{index_vec, IndexSlice, IndexVec};

oxc_index::define_index_type! {
    pub struct USize16 = usize;
    MAX_INDEX = u16::MAX as usize;
    DEFAULT = USize16::from_raw_unchecked(usize::MAX);
}

oxc_index::define_index_type! {
    pub struct ZeroMaxIgnore = u16;
    MAX_INDEX = 0;
    DISABLE_MAX_INDEX_CHECK = true;
}

oxc_index::define_index_type! {
    pub struct ZeroMax = u16;
    MAX_INDEX = 0;
}

oxc_index::define_index_type! {
    pub struct IdxSz = usize;
}

oxc_index::define_index_type! {
    pub struct Idx32 = u32;
    DEBUG_FORMAT = "Test({:?})";
    DISPLAY_FORMAT = "foo {}";
    IMPL_RAW_CONVERSIONS = true;
}

oxc_index::define_index_type! {
    pub struct Idx16 = u16;
}

oxc_index::define_index_type! {
    pub struct Idx8 = u8;
}

oxc_index::define_index_type! {
    pub struct SmallCheckedEarly = u8;
    MAX_INDEX = 0x7f;
}

oxc_index::define_index_type! {
    pub struct SmallChecked = u8;
}

oxc_index::define_index_type! {
    pub struct SmallUnchecked = u8;
    DISABLE_MAX_INDEX_CHECK = true;
}

oxc_index::define_index_type! {
    pub struct SmallUncheckedEarly = u8;
    DISABLE_MAX_INDEX_CHECK = true;
    MAX_INDEX = 0x7f;
}

#[test]
fn test_idx_default_max() {
    assert_eq!(Idx32::MAX_INDEX, u32::MAX as usize);
    assert_eq!(IdxSz::MAX_INDEX, usize::MAX);
    assert_eq!(Idx16::MAX_INDEX, u16::MAX as usize);
    assert_eq!(Idx8::MAX_INDEX, u8::MAX as usize);

    assert!(Idx32::CHECKS_MAX_INDEX);
    assert!(IdxSz::CHECKS_MAX_INDEX);
    assert!(Idx16::CHECKS_MAX_INDEX);
    assert!(Idx8::CHECKS_MAX_INDEX);

    assert!(!ZeroMaxIgnore::CHECKS_MAX_INDEX);
    assert_eq!(ZeroMaxIgnore::MAX_INDEX, 0);
}

#[test]
fn test_idx_arith() {
    assert_eq!(Idx32::new(0), 0usize);
    assert_eq!(Idx32::new(0) + 1, 1usize);
    assert_eq!(1 + Idx32::new(0), 1usize);

    assert_eq!(Idx32::new(1) - 1, 0usize);
    assert_eq!(Idx32::new(5) % 4, 1usize);

    let mut m = Idx32::new(5);
    m += 1;
    assert_eq!(m, 6);

    assert!(Idx32::new(5) < Idx32::new(6));
    assert!(Idx32::new(5) < 6usize);

    assert!(Idx32::new(5) < Idx32::new(6));
    assert!(Idx32::new(5) < 6usize);
    assert!(5usize < Idx32::new(6));
}

#[test]
fn test_idx_checks1() {
    let v: u32 = Idx32::new(4).raw();
    assert_eq!(v, 4);

    let u: usize = Idx32::new(4).index();
    assert_eq!(u, 4);

    assert_eq!(SmallCheckedEarly::from_raw_unchecked(0xff).raw(), 0xff);

    assert!(SmallChecked::CHECKS_MAX_INDEX);
    assert!(SmallCheckedEarly::CHECKS_MAX_INDEX);

    assert_eq!(SmallChecked::MAX_INDEX, 255);
    assert_eq!(SmallCheckedEarly::MAX_INDEX, 0x7f);

    assert!(!SmallUnchecked::CHECKS_MAX_INDEX);
    assert!(!SmallUncheckedEarly::CHECKS_MAX_INDEX);

    assert_eq!(SmallUnchecked::MAX_INDEX, 255);
    assert_eq!(SmallUncheckedEarly::MAX_INDEX, 0x7f);
}

#[test]
fn test_idx_checks2() {
    // all shouldn't panic

    let v = SmallChecked::from_raw(150);
    assert_eq!(v, 150);
    let v = SmallChecked::from_usize(150);
    assert_eq!(v, 150);
    let v = SmallChecked::from_usize(255);
    assert_eq!(v, 255);
    let v = SmallChecked::from_usize(0);
    assert_eq!(v, 0);

    let v = SmallCheckedEarly::from_usize(0x7f);
    assert_eq!(v, 0x7f);
    let v = SmallCheckedEarly::from_usize(0);
    assert_eq!(v, 0);

    let v = SmallUncheckedEarly::from_raw(0xff);
    assert_eq!(v, 0xff);
    let v = SmallUncheckedEarly::from_usize(150);
    assert_eq!(v, 150);
    let v = SmallUncheckedEarly::from_usize(300);
    assert_eq!(v.raw(), 300usize as u8);
    let v = SmallUnchecked::from_usize(150);
    assert_eq!(v, 150);
    let v = SmallUnchecked::from_usize(300);
    assert_eq!(v.raw(), 300usize as u8);

    let v = SmallCheckedEarly::from_raw_unchecked(0xff);
    assert_eq!(v, 0xff);
    let v = SmallCheckedEarly::from_usize_unchecked(150);
    assert_eq!(v, 150);
    let v = SmallCheckedEarly::from_usize_unchecked(300);
    assert_eq!(v.raw(), 300usize as u8);
    let v = SmallChecked::from_usize_unchecked(300);
    assert_eq!(v.raw(), 300usize as u8);

    assert_eq!(<USize16 as Default>::default().index(), usize::MAX);

    let v = ZeroMaxIgnore::new((u16::MAX as usize) + 1);
    assert_eq!(v, 0);
    let v = ZeroMaxIgnore::new(0) + 1;
    assert_eq!(v, 1);
    // let _ = ZeroMaxIgnore::new(0) - 1;
    let v = ZeroMaxIgnore::new(2);
    assert_eq!(v, 2);
    let v = ZeroMaxIgnore::new((u16::MAX as usize) + 1);
    assert_eq!(v, 0);
}

#[test]
#[should_panic]
fn test_idx_sc_cf_raw() {
    let _ = SmallCheckedEarly::from_raw(0xff);
}
#[test]
#[should_panic]
fn test_idx_sc_cf_idx0() {
    let _ = SmallCheckedEarly::from_usize(150);
}
#[test]
#[should_panic]
fn test_idx_sc_cf_idx1() {
    let _ = SmallCheckedEarly::from_usize(300);
}
#[test]
#[should_panic]
fn test_idx_sc_cf_idx2() {
    let _ = SmallChecked::from_usize(300);
}
#[test]
#[should_panic]
fn test_idx_sc_of_add() {
    let _ = SmallChecked::from_usize(255) + 1;
}
#[test]
#[should_panic]
fn test_idx_sc_of_addassign() {
    let mut e2 = SmallChecked::from_usize(255);
    e2 += 1;
}
#[test]
#[should_panic]
fn test_idx_sc_of_sub() {
    let _ = SmallChecked::from_usize(0) - 1;
}
#[test]
#[should_panic]
fn test_idx_sc_of_subassign() {
    let mut z2 = SmallChecked::from_usize(0);
    z2 -= 1;
}

#[test]
#[should_panic]
fn test_idx_zm_cf_idx() {
    let _ = ZeroMax::new(2);
}
#[test]
#[should_panic]
fn test_idx_zm_cf_raw() {
    let _ = ZeroMax::from_raw(2);
}

#[test]
#[should_panic]
fn test_idx_zm_of_add0() {
    let _ = ZeroMax::new(0) + 1;
}
#[test]
#[should_panic]
fn test_idx_zm_of_sub0() {
    let _ = ZeroMax::new(0) - 1;
}
#[test]
#[should_panic]
fn test_idx_zm_of_nowrap() {
    let _ = ZeroMax::new((u16::MAX as usize) + 1);
}

#[test]
#[should_panic]
fn test_idx_sce_adde() {
    let _ = SmallCheckedEarly::from_usize(0x7f) + 1;
}
#[test]
#[should_panic]
fn test_idx_sce_addassign() {
    let mut e3 = SmallCheckedEarly::from_usize(0x7f);
    e3 += 1;
}
#[test]
#[should_panic]
fn test_idx_sce_sub() {
    let _ = SmallCheckedEarly::from_usize(0) - 1;
}
#[test]
#[should_panic]
fn test_idx_sce_subassign() {
    let mut z3 = SmallCheckedEarly::from_usize(0);
    z3 -= 1;
}

#[test]
fn test_vec() {
    let mut strs: IndexVec<Idx32, &'static str> = index_vec!["strs", "bar", "baz"];

    let l = strs.last_idx();
    assert_eq!(strs[l], "baz");

    let new_i = strs.push("quux");
    assert_eq!(strs[new_i], "quux");
}

#[test]
fn test_idx() {
    let mut e = Idx32::new(0);
    let one = Idx32::new(1);
    e += 1;
    assert_eq!(e, 1);
    e -= 1;
    assert_eq!(e, 0);
    e += one;
    assert_eq!(e, 1);
    e -= one;
    assert_eq!(e, 0);
    let e2 = e + one;
    assert_eq!(e2, 1);
    let e2 = e2 - one;
    assert_eq!(e2, 0);

    let e2 = e + 1;
    assert_eq!(e2, 1);
    let e2 = e2 - 1;
    assert_eq!(0, e2);

    assert_eq!(40usize - Idx32::new(10), 30);

    assert_eq!(u32::from(Idx32::new(500)), 500);
    assert_eq!(Idx32::from(500u32), 500);
}

#[test]
fn test_fmt() {
    let i = format!("{:?}", Idx32::new(30));
    assert_eq!(i, "Test(30)");
    let i = format!("{}", Idx32::new(30));
    assert_eq!(i, "foo 30");

    let v: IndexVec<Idx32, i32> = index_vec![3, 4, 5];
    assert_eq!(format!("{:?}", v), format!("{:?}", vec![3, 4, 5]));
    assert_eq!(format!("{:#?}", v), format!("{:#?}", vec![3, 4, 5]));
    assert_eq!(format!("{:?}", &v[..]), format!("{:?}", &[3, 4, 5]));
}

#[test]
fn test_partial_eq() {
    let i0: IndexVec<Idx32, usize> = index_vec![0];
    let i1: IndexVec<Idx32, usize> = index_vec![1];
    let i123: IndexVec<Idx32, usize> = index_vec![1, 2, 3];

    assert_eq!(i0, i0);
    assert_ne!(i0, i1);
    assert_eq!(i123, vec![1, 2, 3]);
    assert_eq!(i123, &[1, 2, 3]);
    assert_eq!(i123, [1, 2, 3]);
    assert_eq!(i123[..], [1, 2, 3]);
    assert_eq!(i123[..Idx32::new(1)], [1usize]);
    assert_eq!(i123[..Idx32::new(1)], i1.as_slice());
    assert_eq!(i123[..Idx32::new(1)], i1.as_raw_slice());
}

#[test]
fn test_drain() {
    let mut vec: IndexVec<Idx32, usize> = index_vec![1, 2, 3];
    let mut vec2: IndexVec<Idx32, usize> = index_vec![];
    for i in vec.drain(..) {
        vec2.push(i);
    }
    assert!(vec.is_empty());
    assert_eq!(vec2, [1, 2, 3]);

    let mut vec: IndexVec<Idx32, usize> = index_vec![1, 2, 3];
    let mut vec2: IndexVec<Idx32, usize> = index_vec![];
    for i in vec.drain(Idx32::from_raw(1)..) {
        vec2.push(i);
    }
    assert_eq!(vec, [1]);
    assert_eq!(vec2, [2, 3]);

    let mut vec: IndexVec<Idx32, ()> = index_vec![(), (), ()];
    let mut vec2: IndexVec<Idx32, ()> = index_vec![];
    for _i in vec.drain(..) {
        vec2.push(());
    }
    assert_eq!(vec, []);
    assert_eq!(vec2, [(), (), ()]);
}

#[test]
fn test_drain_enumerated() {
    let mut vec: IndexVec<Idx32, usize> = index_vec![1, 2, 3];
    let mut vec2: IndexVec<Idx32, usize> = index_vec![];
    for (i, j) in vec.drain_enumerated(..) {
        assert_eq!(i.index() + 1, j);
        vec2.push(j);
    }
    assert!(vec.is_empty());
    assert_eq!(vec2, [1, 2, 3]);
}

#[test]
fn test_position() {
    let b: &IndexSlice<IdxSz, [i32]> = IndexSlice::new(&[1, 2, 3, 5, 5]);
    assert_eq!(b.position(|&v| v == 9), None);
    assert_eq!(b.position(|&v| v == 5), Some(IdxSz::from_raw(3)));
    assert_eq!(b.position(|&v| v == 3), Some(IdxSz::from_raw(2)));
    assert_eq!(b.position(|&v| v == 0), None);
}

#[test]
fn test_rposition() {
    let b: &IndexSlice<IdxSz, [i32]> = IndexSlice::new(&[1, 2, 3, 5, 5]);
    assert_eq!(b.rposition(|&v| v == 9), None);
    assert_eq!(b.rposition(|&v| v == 5), Some(IdxSz::from_raw(4)));
    assert_eq!(b.rposition(|&v| v == 3), Some(IdxSz::from_raw(2)));
    assert_eq!(b.rposition(|&v| v == 0), None);
}

#[test]
fn test_binary_search() {
    let b: &IndexSlice<IdxSz, [i32]> = IndexSlice::new(&[]);
    assert_eq!(b.binary_search(&5), Err(IdxSz::new(0)));

    let b: &IndexSlice<IdxSz, [i32]> = IndexSlice::new(&[4]);
    assert_eq!(b.binary_search(&3), Err(IdxSz::new(0)));
    assert_eq!(b.binary_search(&4), Ok(IdxSz::new(0)));
    assert_eq!(b.binary_search(&5), Err(IdxSz::new(1)));
}

#[test]
fn test_chunk_iters() {
    let mut v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    assert_eq!(
        v.chunks(3).collect::<Vec<_>>(),
        &[IndexSlice::new(&[0, 1, 2]), IndexSlice::new(&[3, 4])]
    );
    assert_eq!(
        v.chunks_mut(3).collect::<Vec<_>>(),
        &[IndexSlice::new_mut(&mut [0, 1, 2]), IndexSlice::new_mut(&mut [3, 4])]
    );

    assert_eq!(v.chunks_exact(3).collect::<Vec<_>>(), &[IndexSlice::new(&[0, 1, 2])]);
    assert_eq!(v.chunks_exact_mut(3).collect::<Vec<_>>(), &[IndexSlice::new_mut(&mut [0, 1, 2])]);

    assert_eq!(
        v.rchunks(3).collect::<Vec<_>>(),
        &[IndexSlice::new(&[2, 3, 4]), IndexSlice::new(&[0, 1])]
    );
    assert_eq!(
        v.rchunks_mut(3).collect::<Vec<_>>(),
        &[IndexSlice::new_mut(&mut [2, 3, 4]), IndexSlice::new_mut(&mut [0, 1])]
    );

    assert_eq!(v.rchunks_exact(3).collect::<Vec<_>>(), &[IndexSlice::new(&[2, 3, 4])]);
    assert_eq!(v.rchunks_exact_mut(3).collect::<Vec<_>>(), &[IndexSlice::new_mut(&mut [2, 3, 4])]);
    assert_eq!(
        v.windows(2).collect::<Vec<_>>(),
        &[
            IndexSlice::new(&[0, 1]),
            IndexSlice::new(&[1, 2]),
            IndexSlice::new(&[2, 3]),
            IndexSlice::new(&[3, 4])
        ]
    );
}

#[test]
fn test_indexing() {
    let v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    assert_eq!(v[..], &[0, 1, 2, 3, 4]);
    assert_eq!(v[IdxSz::new(1)..], &[1, 2, 3, 4]);
    assert_eq!(v[IdxSz::new(1)..IdxSz::new(3)], &[1, 2]);
    assert_eq!(v[IdxSz::new(1)..=IdxSz::new(3)], &[1, 2, 3]);
    assert_eq!(v[..=IdxSz::new(3)], &[0, 1, 2, 3]);

    assert_eq!(v[IdxSz::new(3)], 3);

    // Make sure the types are as expected
    let s: &IndexSlice<IdxSz, [i32]> = &v[..];
    assert_eq!(s, &[0, 1, 2, 3, 4]);
    let s: &IndexSlice<IdxSz, [i32]> = &v[IdxSz::new(1)..];
    assert_eq!(s, &[1, 2, 3, 4]);
    let s: &IndexSlice<IdxSz, [i32]> = &v[IdxSz::new(1)..IdxSz::new(3)];
    assert_eq!(s, &[1, 2]);
    let s: &IndexSlice<IdxSz, [i32]> = &v[IdxSz::new(1)..=IdxSz::new(3)];
    assert_eq!(s, &[1, 2, 3]);
    let s: &IndexSlice<IdxSz, [i32]> = &v[..=IdxSz::new(3)];
    assert_eq!(s, &[0, 1, 2, 3]);

    let mut v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    // check `IndexMut`
    {
        let s: &mut IndexSlice<IdxSz, [i32]> = &mut v[..];
        assert_eq!(s, &[0, 1, 2, 3, 4]);
    }
    {
        let s: &mut IndexSlice<IdxSz, [i32]> = &mut v[IdxSz::new(1)..];
        assert_eq!(s, &[1, 2, 3, 4]);
    }
    {
        let s: &mut IndexSlice<IdxSz, [i32]> = &mut v[IdxSz::new(1)..IdxSz::new(3)];
        assert_eq!(s, &[1, 2]);
    }
    {
        let s: &mut IndexSlice<IdxSz, [i32]> = &mut v[IdxSz::new(1)..=IdxSz::new(3)];
        assert_eq!(s, &[1, 2, 3]);
    }
    {
        let s: &mut IndexSlice<IdxSz, [i32]> = &mut v[..=IdxSz::new(3)];
        assert_eq!(s, &[0, 1, 2, 3]);
    }
    assert_eq!(&mut v[IdxSz::new(3)], &mut 3);
}

#[test]
fn test_get() {
    let v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];

    let s: Option<&IndexSlice<IdxSz, [i32]>> = v.get(..);
    assert_eq!(s.unwrap(), &[0, 1, 2, 3, 4]);
    let s: Option<&IndexSlice<IdxSz, [i32]>> = v.get(IdxSz::new(1)..);
    assert_eq!(s.unwrap(), &[1, 2, 3, 4]);
    let s: Option<&IndexSlice<IdxSz, [i32]>> = v.get(IdxSz::new(1)..IdxSz::new(3));
    assert_eq!(s.unwrap(), &[1, 2]);
    let s: Option<&IndexSlice<IdxSz, [i32]>> = v.get(IdxSz::new(1)..=IdxSz::new(3));
    assert_eq!(s.unwrap(), &[1, 2, 3]);
    let s: Option<&IndexSlice<IdxSz, [i32]>> = v.get(..=IdxSz::new(3));
    assert_eq!(s.unwrap(), &[0, 1, 2, 3]);

    assert_eq!(v.get(IdxSz::new(3)), Some(&3));
}

#[test]
fn test_get_mut() {
    let mut v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    let s: Option<&mut IndexSlice<IdxSz, [i32]>> = v.get_mut(..);
    assert_eq!(s.unwrap(), &[0, 1, 2, 3, 4]);
    let s: Option<&mut IndexSlice<IdxSz, [i32]>> = v.get_mut(IdxSz::new(1)..);
    assert_eq!(s.unwrap(), &[1, 2, 3, 4]);
    let s: Option<&mut IndexSlice<IdxSz, [i32]>> = v.get_mut(IdxSz::new(1)..IdxSz::new(3));
    assert_eq!(s.unwrap(), &[1, 2]);
    let s: Option<&mut IndexSlice<IdxSz, [i32]>> = v.get_mut(IdxSz::new(1)..=IdxSz::new(3));
    assert_eq!(s.unwrap(), &[1, 2, 3]);
    let s: Option<&mut IndexSlice<IdxSz, [i32]>> = v.get_mut(..=IdxSz::new(3));
    assert_eq!(s.unwrap(), &[0, 1, 2, 3]);
    assert_eq!(v.get_mut(IdxSz::new(3)), Some(&mut 3));
}

#[test]
fn test_splits() {
    let v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    let (a, b): (&i32, &IndexSlice<IdxSz, [i32]>) = v.split_first().unwrap();
    assert_eq!(a, &0);
    assert_eq!(b, &[1, 2, 3, 4]);

    let (a, b): (&i32, &IndexSlice<IdxSz, [i32]>) = v.split_last().unwrap();
    assert_eq!(a, &4);
    assert_eq!(b, &[0, 1, 2, 3]);
    let mut v: IndexVec<IdxSz, i32> = index_vec![0, 1, 2, 3, 4];
    let (a, b): (&mut i32, &mut IndexSlice<IdxSz, [i32]>) = v.split_first_mut().unwrap();
    assert_eq!(a, &0);
    assert_eq!(b, &[1, 2, 3, 4]);

    let (a, b): (&mut i32, &mut IndexSlice<IdxSz, [i32]>) = v.split_last_mut().unwrap();
    assert_eq!(a, &4);
    assert_eq!(b, &[0, 1, 2, 3]);

    let mut v: IndexVec<IdxSz, i32> = index_vec![];
    assert!(v.split_first().is_none());
    assert!(v.split_last().is_none());
    assert!(v.split_first_mut().is_none());
    assert!(v.split_last_mut().is_none());
}
