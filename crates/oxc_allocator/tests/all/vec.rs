#![cfg(feature = "collections")]

use crate::quickcheck;
use bumpalo::{collections::Vec, vec, Bump};
use std::cell::{Cell, RefCell};
use std::ops::Deref;

#[test]
fn push_a_bunch_of_items() {
    let b = Bump::new();
    let mut v = Vec::new_in(&b);
    for x in 0..10_000 {
        v.push(x);
    }
}

#[test]
fn trailing_comma_in_vec_macro() {
    let b = Bump::new();
    let v = vec![in &b; 1, 2, 3,];
    assert_eq!(v, [1, 2, 3]);
}

#[test]
fn recursive_vecs() {
    // The purpose of this test is to see if the data structures with
    // self references are allowed without causing a compile error
    // because of the dropck
    let b = Bump::new();

    struct Node<'a> {
        myself: Cell<Option<&'a Node<'a>>>,
        edges: Cell<Vec<'a, &'a Node<'a>>>,
    }

    let node1: &Node = b.alloc(Node {
        myself: Cell::new(None),
        edges: Cell::new(Vec::new_in(&b)),
    });
    let node2: &Node = b.alloc(Node {
        myself: Cell::new(None),
        edges: Cell::new(Vec::new_in(&b)),
    });

    node1.myself.set(Some(node1));
    node1.edges.set(bumpalo::vec![in &b; node1, node1, node2]);

    node2.myself.set(Some(node2));
    node2.edges.set(bumpalo::vec![in &b; node1, node2]);
}

#[test]
fn test_into_bump_slice_mut() {
    let b = Bump::new();
    let v = bumpalo::vec![in &b; 1, 2, 3];
    let slice = v.into_bump_slice_mut();

    slice[0] = 3;
    slice[2] = 1;

    assert_eq!(slice, [3, 2, 1]);
}

quickcheck! {
    fn vec_resizes_causing_reallocs(sizes: std::vec::Vec<usize>) -> () {
        // Exercise `realloc` by doing a bunch of `resize`s followed by
        // `shrink_to_fit`s.

        let b = Bump::new();
        let mut v = bumpalo::vec![in &b];

        for len in sizes {
            // We don't want to get too big and OOM.
            const MAX_SIZE: usize = 1 << 15;

            // But we want allocations to get fairly close to the minimum chunk
            // size, so that we are exercising both realloc'ing within a chunk
            // and when we need new chunks.
            const MIN_SIZE: usize = 1 << 7;

            let len = std::cmp::min(len, MAX_SIZE);
            let len = std::cmp::max(len, MIN_SIZE);

            v.resize(len, 0);
            v.shrink_to_fit();
        }
    }
}

#[test]
fn test_vec_items_get_dropped() {
    struct Foo<'a>(&'a RefCell<String>);
    impl<'a> Drop for Foo<'a> {
        fn drop(&mut self) {
            self.0.borrow_mut().push_str("Dropped!");
        }
    }

    let buffer = RefCell::new(String::new());
    let bump = Bump::new();
    {
        let mut vec_foo = Vec::new_in(&bump);
        vec_foo.push(Foo(&buffer));
        vec_foo.push(Foo(&buffer));
    }
    assert_eq!("Dropped!Dropped!", buffer.borrow().deref());
}

#[test]
fn test_extend_from_slice_copy() {
    let bump = Bump::new();
    let mut vec = vec![in &bump; 1, 2, 3];
    assert_eq!(&[1, 2, 3][..], vec.as_slice());

    vec.extend_from_slice_copy(&[4, 5, 6]);
    assert_eq!(&[1, 2, 3, 4, 5, 6][..], vec.as_slice());

    // Confirm that passing an empty slice is a no-op
    vec.extend_from_slice_copy(&[]);
    assert_eq!(&[1, 2, 3, 4, 5, 6][..], vec.as_slice());

    vec.extend_from_slice_copy(&[7]);
    assert_eq!(&[1, 2, 3, 4, 5, 6, 7][..], vec.as_slice());
}

#[test]
fn test_extend_from_slices_copy() {
    let bump = Bump::new();
    let mut vec = vec![in &bump; 1, 2, 3];
    assert_eq!(&[1, 2, 3][..], vec.as_slice());

    // Confirm that passing an empty slice of slices is a no-op
    vec.extend_from_slices_copy(&[]);
    assert_eq!(&[1, 2, 3][..], vec.as_slice());

    // Confirm that an empty slice in the slice-of-slices is a no-op
    vec.extend_from_slices_copy(&[&[4, 5, 6], &[], &[7]]);
    assert_eq!(&[1, 2, 3, 4, 5, 6, 7][..], vec.as_slice());

    vec.extend_from_slices_copy(&[&[8], &[9, 10, 11], &[12]]);
    assert_eq!(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], vec.as_slice());
}

#[cfg(feature = "std")]
#[test]
fn test_vec_write() {
    use std::io::Write;

    let b = Bump::new();
    let mut v = bumpalo::vec![in &b];

    assert_eq!(v.write(&[]).unwrap(), 0);

    v.flush().unwrap();

    assert_eq!(v.write(&[1]).unwrap(), 1);

    v.flush().unwrap();

    v.write_all(&[]).unwrap();

    v.flush().unwrap();

    v.write_all(&[2, 3]).unwrap();

    v.flush().unwrap();

    assert_eq!(v, &[1, 2, 3]);
}
