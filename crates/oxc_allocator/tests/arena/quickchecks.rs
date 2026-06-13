use std::{mem, ptr::NonNull};

use ::quickcheck::{Arbitrary, Gen};

use oxc_allocator::arena::Arena;

use crate::quickcheck;

#[derive(Clone, Debug, PartialEq)]
struct BigValue {
    data: [u64; 32],
}

impl BigValue {
    fn new(x: u64) -> BigValue {
        BigValue {
            data: [
                x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x, x,
                x, x, x, x,
            ],
        }
    }
}

impl Arbitrary for BigValue {
    fn arbitrary(g: &mut Gen) -> BigValue {
        BigValue::new(u64::arbitrary(g))
    }
}

#[derive(Clone, Debug)]
enum Elems<T, U> {
    OneT(T),
    TwoT(T, T),
    FourT(T, T, T, T),
    OneU(U),
    TwoU(U, U),
    FourU(U, U, U, U),
}

impl<T, U> Arbitrary for Elems<T, U>
where
    T: Arbitrary + Clone,
    U: Arbitrary + Clone,
{
    fn arbitrary(g: &mut Gen) -> Elems<T, U> {
        let x: u8 = u8::arbitrary(g);
        match x % 6 {
            0 => Elems::OneT(T::arbitrary(g)),
            1 => Elems::TwoT(T::arbitrary(g), T::arbitrary(g)),
            2 => Elems::FourT(T::arbitrary(g), T::arbitrary(g), T::arbitrary(g), T::arbitrary(g)),
            3 => Elems::OneU(U::arbitrary(g)),
            4 => Elems::TwoU(U::arbitrary(g), U::arbitrary(g)),
            5 => Elems::FourU(U::arbitrary(g), U::arbitrary(g), U::arbitrary(g), U::arbitrary(g)),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Elems::OneT(_) => Box::new(vec![].into_iter()),
            Elems::TwoT(a, b) => {
                Box::new(vec![Elems::OneT(a.clone()), Elems::OneT(b.clone())].into_iter())
            }
            Elems::FourT(a, b, c, d) => Box::new(
                vec![
                    Elems::TwoT(a.clone(), b.clone()),
                    Elems::TwoT(a.clone(), c.clone()),
                    Elems::TwoT(a.clone(), d.clone()),
                    Elems::TwoT(b.clone(), c.clone()),
                    Elems::TwoT(b.clone(), d.clone()),
                    Elems::TwoT(c.clone(), d.clone()),
                ]
                .into_iter(),
            ),
            Elems::OneU(_) => Box::new(vec![].into_iter()),
            Elems::TwoU(a, b) => {
                Box::new(vec![Elems::OneU(a.clone()), Elems::OneU(b.clone())].into_iter())
            }
            Elems::FourU(a, b, c, d) => Box::new(
                vec![
                    Elems::TwoU(a.clone(), b.clone()),
                    Elems::TwoU(a.clone(), c.clone()),
                    Elems::TwoU(a.clone(), d.clone()),
                    Elems::TwoU(b.clone(), c.clone()),
                    Elems::TwoU(b.clone(), d.clone()),
                    Elems::TwoU(c.clone(), d.clone()),
                ]
                .into_iter(),
            ),
        }
    }
}

fn overlap((a1, a2): (usize, usize), (b1, b2): (usize, usize)) -> bool {
    assert!(a1 < a2);
    assert!(b1 < b2);
    a1 < b2 && b1 < a2
}

// Returns whether `(b1, b2)` is contained in `(a1, a2)`.
fn contains((a1, a2): (usize, usize), (b1, b2): (usize, usize)) -> bool {
    assert!(a1 < a2);
    assert!(b1 < b2);
    a1 <= b1 && b2 <= a2
}

fn range<T>(t: &T) -> (usize, usize) {
    let start = t as *const _ as usize;
    let end = start + mem::size_of::<T>();
    (start, end)
}

quickcheck! {
    fn can_allocate_big_values(values: Vec<BigValue>) -> () {
        let arena = Arena::new();
        let mut alloced = vec![];

        for vals in values.iter().cloned() {
            alloced.push(arena.alloc(vals));
        }

        for (vals, alloc) in values.iter().zip(alloced.into_iter()) {
            assert_eq!(vals, alloc);
        }
    }

    fn big_allocations_never_overlap(values: Vec<BigValue>) -> () {
        let arena = Arena::new();
        let mut alloced = vec![];

        for v in values {
            let a = arena.alloc(v);
            let start = a as *const _ as usize;
            let end = unsafe { (a as *const BigValue).offset(1) as usize };
            let range = (start, end);

            for r in &alloced {
                assert!(!overlap(*r, range));
            }

            alloced.push(range);
        }
    }

    fn can_allocate_heterogeneous_things_and_they_dont_overlap(things: Vec<Elems<u8, u64>>) -> () {
        let arena = Arena::new();
        let mut ranges = vec![];

        for t in things {
            let r = match t {
                Elems::OneT(a) => {
                    range(arena.alloc(a))
                },
                Elems::TwoT(a, b) => {
                    range(arena.alloc([a, b]))
                },
                Elems::FourT(a, b, c, d) => {
                    range(arena.alloc([a, b, c, d]))
                },
                Elems::OneU(a) => {
                    range(arena.alloc(a))
                },
                Elems::TwoU(a, b) => {
                    range(arena.alloc([a, b]))
                },
                Elems::FourU(a, b, c, d) => {
                    range(arena.alloc([a, b, c, d]))
                },
            };

            for s in &ranges {
                assert!(!overlap(r, *s));
            }

            ranges.push(r);
        }
    }


    fn test_alignment_chunks(sizes: Vec<usize>) -> () {
        const SUPPORTED_ALIGNMENTS: &[usize] = &[1, 2, 4, 8, 16];
        for &alignment in SUPPORTED_ALIGNMENTS {
            let mut b = Arena::<1>::with_min_align_and_capacity(513);
            let mut sizes = sizes.iter().map(|&size| (size % 10) * alignment).collect::<Vec<_>>();

            for &size in &sizes {
                let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
                let ptr = b.alloc_layout(layout).as_ptr() as *const u8 as usize;
                assert_eq!(ptr % alignment, 0);
            }

            for chunk in b.iter_allocated_chunks() {
                let mut remaining = chunk.len();
                while remaining > 0 {
                    let size = sizes.pop().expect("too many bytes in the chunk output");
                    assert!(remaining >= size, "returned chunk contained padding");
                    remaining -= size;
                }
            }
            assert_eq!(sizes.into_iter().sum::<usize>(), 0);
        }
    }

    fn alloc_slices(allocs: Vec<(u8, usize)>) -> () {
        let b = Arena::new();
        let mut allocated: Vec<(usize, usize)> = vec![];
        for (val, len) in allocs {
            let len = len % 100;
            let s = b.alloc_slice_fill_copy(len, val);

            assert_eq!(s.len(), len);
            assert!(s.iter().all(|v| v == &val));

            let range = (s.as_ptr() as usize, unsafe { s.as_ptr().add(s.len()) } as usize);
            for r in &allocated {
                let no_overlap = range.1 <= r.0 || r.1 <= range.0;
                assert!(no_overlap);
            }
            allocated.push(range);
        }
    }

    fn alloc_strs(allocs: Vec<String>) -> () {
        let b = Arena::new();
        let allocated: Vec<&str> = allocs.iter().map(|s| b.alloc_str(s) as &_).collect();
        for (val, alloc) in allocs.into_iter().zip(allocated) {
            assert_eq!(val, alloc);
        }
    }

    fn all_allocations_in_a_chunk(values: Vec<BigValue>) -> () {
        let b = Arena::new();
        let allocated: Vec<&BigValue> = values.into_iter().map(|val| b.alloc(val) as &_).collect();
        let chunks: Vec<(NonNull<u8>, usize)> = unsafe { b.iter_allocated_chunks_raw() }.collect();
        for alloc in allocated.into_iter() {
            assert!(chunks.iter().any(|&(ptr, size)| {
                let ptr = ptr.addr().get();
                let chunk = (ptr, ptr + size);
                contains(chunk, range(alloc))
            }));
        }
    }

    fn chunks_and_raw_chunks_are_same(values: Vec<BigValue>) -> () {
        let mut b = Arena::new();
        for val in values {
            b.alloc(val);
        }
        let raw_chunks: Vec<(NonNull<u8>, usize)> = unsafe { b.iter_allocated_chunks_raw() }.collect();
        let chunks: Vec<&[_]> = b.iter_allocated_chunks().collect();
        assert_eq!(raw_chunks.len(), chunks.len());
        for ((ptr, size), chunk) in raw_chunks.into_iter().zip(chunks) {
            assert_eq!(ptr.as_ptr().cast_const(), chunk.as_ptr().cast::<u8>());
            assert_eq!(size, chunk.len());
        }
    }

    fn allocated_bytes_tracking(allocs: Vec<usize>) -> () {
        let b = Arena::new();
        let mut slice_bytes = 0;
        for len in allocs {
            const MAX_LEN: usize = 512;
            let len = len % MAX_LEN;
            b.alloc_slice_fill_copy(len, 0);
            slice_bytes += len;
            let allocated_bytes = b.allocated_bytes();
            if slice_bytes == 0 {
                assert_eq!(allocated_bytes, 0);
            } else {
                assert!(allocated_bytes >= slice_bytes);
            }
        }
    }

    fn used_bytes_tracking(allocs: Vec<usize>) -> () {
        let mut b = Arena::new();
        let mut slice_bytes = 0;
        for len in allocs {
            const MAX_LEN: usize = 512;
            let len = len % MAX_LEN;
            b.alloc_slice_fill_copy(len, 0u8);
            slice_bytes += len;

            // `Arena<1>` with `align == 1` allocations: cursor advances by exactly `len` per call,
            // so total used bytes equal the sum of requested sizes
            let used_bytes = b.used_bytes();
            assert_eq!(used_bytes, slice_bytes);

            // `used_bytes <= allocated_bytes` (capacity >= used)
            assert!(used_bytes <= b.allocated_bytes());

            // `used_bytes` agrees with summing each chunk's used portion via `iter_allocated_chunks`
            let from_iter = b.iter_allocated_chunks().map(<[_]>::len).sum();
            assert_eq!(used_bytes, from_iter);
        }

        // After `reset`, the surviving chunk's cursor is back at its end - 0 bytes used
        b.reset();
        assert_eq!(b.used_bytes(), 0);
    }
}
