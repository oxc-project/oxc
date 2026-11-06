use std::panic::Location;

use crate::simd::{Vec128, Vec256};

use super::{Avx2, BASELINE, Cpu, DynCpu, Neon, cpu_multiversion, cpu_version, to_dyn};

#[cpu_multiversion]
const fn add_one(x: u32, _cpu: impl Cpu) -> u32 {
    x + 1
}

#[cpu_multiversion]
const unsafe fn read_byte(ptr: *const u8, _cpu: impl Cpu) -> u8 {
    // SAFETY: Caller guarantees `ptr` is valid for reads
    unsafe { *ptr }
}

#[cpu_multiversion]
const fn larger<'a>(x: &'a u8, y: &'a u8, _cpu: impl Cpu) -> &'a u8 {
    if *x >= *y { x } else { y }
}

#[cpu_multiversion]
const fn larger_at<'a, const N: usize>(xs: &'a [u8], ys: &'a [u8], _cpu: impl Cpu) -> &'a u8 {
    if xs[N] >= ys[N] { &xs[N] } else { &ys[N] }
}

// `T` can only be passed with a turbofish
#[cpu_multiversion]
const fn size<T>(_cpu: impl Cpu) -> usize {
    size_of::<T>()
}

#[cpu_multiversion]
const fn first<T: Copy, const N: usize>(xs: [T; N], _cpu: impl Cpu) -> T {
    xs[0]
}

// `T` might need dropping
#[cpu_multiversion]
const unsafe fn identity<T>(x: T, _cpu: impl Cpu) -> T {
    x
}

#[cpu_multiversion]
#[inline(never)]
fn with_bounds<X, Y: Copy, Z: Into<u32>>(x: X, y: Y, z: Z, _cpu: impl Cpu) -> (X, Y, u32) {
    (x, y, z.into())
}

// `Q` is another generic parameter
#[cpu_multiversion]
fn param_in_bound<Q, Z: Into<Q>>(z: Z, _cpu: impl Cpu) -> Q {
    z.into()
}

// `>>` closes 2 `<`s, before the end of the generic parameters
#[cpu_multiversion]
fn nested<Z: Into<Vec<u8>>, W>(z: Z, w: W, _cpu: impl Cpu) -> (Vec<u8>, W) {
    (z.into(), w)
}

// `>>>>` is `>>` twice
#[cpu_multiversion]
fn nested_at_end<Z: Into<Vec<Vec<u8>>>>(z: Z, _cpu: impl Cpu) -> Vec<Vec<u8>> {
    z.into()
}

// `>>>` is `>>` then `>`
#[cpu_multiversion]
fn nested_3<Z: Into<Vec<Vec<u8>>>, W>(z: Z, w: W, _cpu: impl Cpu) -> (Vec<Vec<u8>>, W) {
    (z.into(), w)
}

// `<<` opens 2 `<`s
#[cpu_multiversion]
fn qualified_path<U: IntoIterator, Z: Into<<U as IntoIterator>::Item>>(
    z: Z,
    _cpu: impl Cpu,
) -> <U as IntoIterator>::Item {
    z.into()
}

#[cpu_multiversion]
fn multiple_bounds<'a, T: ?Sized + AsRef<[u8]> + 'a>(x: &'a T, _cpu: impl Cpu) -> &'a [u8] {
    x.as_ref()
}

#[cpu_multiversion]
fn lifetime_bounds<'a, 'b: 'a, T: 'b + Copy>(x: &'a &'b T, _cpu: impl Cpu) -> T {
    **x
}

// Bound contains `,` inside `(...)`, and `->`
#[cpu_multiversion]
fn fn_bound<F: Fn(u8, u8) -> Vec<u8>, W>(f: F, w: W, _cpu: impl Cpu) -> (Vec<u8>, W) {
    (f(1, 2), w)
}

#[cpu_multiversion]
fn higher_ranked<F: for<'x> Fn(&'x u8) -> &'x u8>(f: F, x: &u8, _cpu: impl Cpu) -> u8 {
    *f(x)
}

#[cpu_multiversion]
fn assoc_type<I: Iterator<Item = u8>>(iter: I, _cpu: impl Cpu) -> u32 {
    iter.map(u32::from).sum()
}

// Const and type parameters in any order, with a trailing comma
#[cpu_multiversion]
#[rustfmt::skip]
fn mixed<A: Copy, const N: usize, B: Copy, const M: usize,>(
    a: [A; N],
    b: [B; M],
    _cpu: impl Cpu,
) -> (A, B) {
    (a[0], b[0])
}

#[cpu_multiversion]
#[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` function")]
#[inline(always)]
const fn add_two(x: u32, _cpu: impl Cpu) -> u32 {
    x + 2
}

// `Self` in nested items isn't the function's, so this isn't a method
#[cpu_multiversion]
fn nested_items(x: u32, _cpu: impl Cpu) -> u32 {
    struct Inner(u32, Option<Box<Self>>);
    impl Inner {
        fn new(x: u32) -> Self {
            Self(x, Some(Box::new(Self(1, None))))
        }
        fn total(&self) -> u32 {
            self.0 + self.1.as_ref().map_or(0, |inner| inner.0)
        }
    }
    Inner::new(x).total()
}

// Lint attributes go on `__impl`, where the lint fires
#[cpu_multiversion]
#[expect(unused_variables, reason = "tests lint attributes go on the function with the body")]
fn unused_in_body(x: u32, _cpu: impl Cpu) -> u32 {
    let unused = x;
    x
}

// `#[cold]` goes on the entry points, with `#[inline]`
#[cpu_multiversion]
#[cold]
#[inline(never)]
fn cold_add_one(x: u32, _cpu: impl Cpu) -> u32 {
    x + 1
}

// `#[track_caller]` goes on every generated function, so the caller's location passes through
#[cpu_multiversion]
#[track_caller]
fn caller_line(_cpu: impl Cpu) -> u32 {
    Location::caller().line()
}

struct Counter(u32);

impl Counter {
    #[cpu_multiversion]
    const fn plus(&self, x: u32, _cpu: impl Cpu) -> u32 {
        self.0 + x
    }

    #[cpu_multiversion]
    const unsafe fn plus_byte(&self, ptr: *const u8, _cpu: impl Cpu) -> u32 {
        // SAFETY: Caller guarantees `ptr` is valid for reads
        self.0 + unsafe { *ptr } as u32
    }

    #[cpu_multiversion]
    const fn at<'a, const N: usize>(&self, bytes: &'a [u8], _cpu: impl Cpu) -> &'a u8 {
        &bytes[self.0 as usize + N]
    }

    #[cpu_multiversion]
    const fn plus_size<T>(&self, _cpu: impl Cpu) -> usize {
        self.0 as usize + size_of::<T>()
    }

    // `>>` ends the generic parameters. They go through `pastey`, unlike a function's.
    #[cpu_multiversion]
    fn plus_len<const N: usize, Z: Into<Vec<u8>>>(&self, z: Z, _cpu: impl Cpu) -> usize {
        self.0 as usize + z.into().len() + N
    }

    // Bound mentions `Self`
    #[cpu_multiversion]
    fn plus_counter<Z: Into<Self>>(self, z: Z, _cpu: impl Cpu) -> u32 {
        self.0 + z.into().0
    }

    // No `self` parameter, but `Self` in the return type and body
    #[cpu_multiversion]
    const fn of(n: u32, _cpu: impl Cpu) -> Self {
        Self(n)
    }

    // No `self` parameter, but `Self` in the body only
    #[cpu_multiversion]
    const fn double_of(n: u32, _cpu: impl Cpu) -> u32 {
        Self(n).0 * 2
    }

    // Lint attributes go on `__plus_unused_impl`, where the lint fires.
    // On `plus_unused` the expectation would be unfulfilled, as its body only calls entry points.
    #[cpu_multiversion]
    #[expect(unused_variables, reason = "tests lint attributes go on the function with the body")]
    fn plus_unused(&self, x: u32, _cpu: impl Cpu) -> u32 {
        let unused = x;
        self.0
    }

    // rustc evaluates `#[cfg]` before the macro runs, wherever it is in the list.
    // The macro doesn't have to remove the sibling functions, or this would fail to compile.
    #[cpu_multiversion]
    #[cfg(any())]
    fn removed(&self, _cpu: impl Cpu) -> Missing {
        Missing
    }

    // rustc expands `#[cfg_attr]` before the macro runs too, so the macro sees `#[expect]`
    #[cpu_multiversion]
    #[cfg_attr(all(), expect(unused_variables, reason = "tests `cfg_attr` is expanded first"))]
    fn plus_unused_cfg_attr(&self, x: u32, _cpu: impl Cpu) -> u32 {
        let unused = x;
        self.0
    }

    #[cpu_multiversion]
    #[track_caller]
    fn caller_line(&self, _cpu: impl Cpu) -> u32 {
        Location::caller().line()
    }

    #[cpu_multiversion]
    #[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` method")]
    #[inline(always)]
    const fn minus(&self, x: u32, _cpu: impl Cpu) -> u32 {
        self.0 - x
    }
}

impl From<u8> for Counter {
    fn from(n: u8) -> Self {
        Self(n.into())
    }
}

struct Wrapper<W>(W);

impl<W: Copy> Wrapper<W> {
    // Bound mentions the `impl`'s generic parameter
    #[cpu_multiversion]
    fn convert<T: From<W>>(&self, _cpu: impl Cpu) -> T {
        T::from(self.0)
    }
}

/// [`BASELINE`] as a [`DynCpu`].
const BASELINE_DYN: DynCpu = to_dyn(BASELINE);

/// Calls to `const` functions made with `#[cpu_multiversion]`, evaluated at compile time.
const AT_COMPILE_TIME: [u32; 13] = {
    let bytes: [u8; 2] = [1, 2];
    [
        add_one(1, BASELINE),
        add_one(10, BASELINE_DYN),
        Counter(100).plus(1, BASELINE),
        // SAFETY: Pointer is valid for reads of 1 byte
        unsafe { read_byte(&raw const bytes[1], BASELINE) } as u32,
        // SAFETY: Pointer is valid for reads of 1 byte
        unsafe { Counter(100).plus_byte(&raw const bytes[1], BASELINE) },
        *larger(&bytes[0], &bytes[1], BASELINE) as u32,
        *larger_at::<0>(&bytes, &[3, 0], BASELINE_DYN) as u32,
        *Counter(1).at::<0>(&bytes, BASELINE) as u32,
        first([4, 5], BASELINE),
        add_two(1, BASELINE_DYN),
        Counter(100).minus(1, BASELINE),
        Counter::of(5, BASELINE).0,
        Counter::double_of(6, BASELINE_DYN),
    ]
};

/// Calls to `const` functions which return a `usize`, evaluated at compile time.
const SIZES_AT_COMPILE_TIME: [usize; 2] =
    [size::<u64>(BASELINE_DYN), Counter(1).plus_size::<u16>(BASELINE)];

#[test]
fn const_fn_at_compile_time() {
    assert_eq!(AT_COMPILE_TIME, [2, 11, 101, 2, 102, 2, 3, 2, 4, 3, 99, 5, 12]);
    assert_eq!(SIZES_AT_COMPILE_TIME, [8, 3]);
}

#[test]
fn const_fn_at_runtime() {
    let cpu = DynCpu::detect();
    let bytes: [u8; 2] = [1, 2];

    assert_eq!(add_one(1, cpu), 2);
    assert_eq!(add_one(1, BASELINE), 2);
    assert_eq!(Counter(100).plus(1, cpu), 101);
    // SAFETY: Pointer is valid for reads of 1 byte
    assert_eq!(unsafe { read_byte(&raw const bytes[1], cpu) }, 2);
    // SAFETY: Pointer is valid for reads of 1 byte
    assert_eq!(unsafe { Counter(100).plus_byte(&raw const bytes[1], cpu) }, 102);
    assert_eq!(*larger(&bytes[0], &bytes[1], cpu), 2);
    assert_eq!(*larger_at::<0>(&bytes, &[3, 0], cpu), 3);
    assert_eq!(*Counter(1).at::<0>(&bytes, cpu), 2);
    assert_eq!(size::<u64>(cpu), 8);
    assert_eq!(first([4, 5], cpu), 4);
    assert_eq!(Counter(1).plus_size::<u16>(cpu), 3);
    assert_eq!(add_two(1, cpu), 3);
    assert_eq!(Counter(100).minus(1, cpu), 99);
    assert_eq!(Counter::of(5, cpu).0, 5);
    assert_eq!(Counter::double_of(6, cpu), 12);

    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => assert_eq!(add_one(1, cpu), 2),
        DynCpu::Neon(cpu) => assert_eq!(add_one(1, cpu), 2),
        DynCpu::Generic(cpu) => assert_eq!(add_one(1, cpu), 2),
    }
}

#[test]
fn generic_params() {
    for cpu in [DynCpu::detect(), BASELINE_DYN] {
        // SAFETY: `identity` is safe to call
        assert_eq!(unsafe { identity(String::from("x"), cpu) }, "x");
        assert_eq!(with_bounds("x", 2u8, 3u8, cpu), ("x", 2, 3));
        assert_eq!(param_in_bound::<u32, u8>(3, cpu), 3);
        assert_eq!(nested([1u8], 'w', cpu), (vec![1], 'w'));
        assert_eq!(nested_at_end([vec![1u8]], cpu), vec![vec![1]]);
        assert_eq!(nested_3([vec![1u8]], 'w', cpu), (vec![vec![1]], 'w'));
        assert_eq!(qualified_path::<Vec<u32>, u8>(3, cpu), 3);
        assert_eq!(multiple_bounds("ab", cpu), b"ab");
        assert_eq!(lifetime_bounds(&&5u8, cpu), 5);
        assert_eq!(fn_bound(|a, b| vec![a, b], 'w', cpu), (vec![1, 2], 'w'));
        assert_eq!(higher_ranked(|x| x, &7, cpu), 7);
        assert_eq!(assoc_type([1, 2].into_iter(), cpu), 3);
        assert_eq!(mixed([1u8, 2], [3u16], cpu), (1, 3));

        assert_eq!(Counter(100).plus_len::<5, _>([1u8, 2], cpu), 107);
        assert_eq!(Counter(100).plus_counter(1u8, cpu), 101);
        assert_eq!(Wrapper(3u8).convert::<u32>(cpu), 3);
    }
}

#[test]
fn nested_items_with_self() {
    for cpu in [DynCpu::detect(), BASELINE_DYN] {
        assert_eq!(nested_items(5, cpu), 6);
    }
}

#[test]
fn attributes() {
    for cpu in [DynCpu::detect(), BASELINE_DYN] {
        assert_eq!(unused_in_body(3, cpu), 3);
        assert_eq!(cold_add_one(1, cpu), 2);
        assert_eq!(caller_line(cpu), line!());
        assert_eq!(Counter(5).plus_unused(1, cpu), 5);
        assert_eq!(Counter(5).plus_unused_cfg_attr(1, cpu), 5);
        assert_eq!(Counter(5).caller_line(cpu), line!());
    }
}

#[cpu_version(Avx2)]
#[inline]
fn double_avx2(x: u32, _cpu: Avx2) -> u32 {
    x * 2
}

#[cpu_version(Neon)]
#[inline]
fn double_neon(x: u32, _cpu: Neon) -> u32 {
    x * 2
}

#[cpu_version(Generic)]
#[inline]
fn double_generic(x: u32) -> u32 {
    x * 2
}

/// `double_avx2` and `double_neon` have a parameter which proves the CPU supports the feature set,
/// so calling them is safe.
fn double(x: u32, cpu: impl Cpu) -> u32 {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => double_avx2(x, cpu),
        DynCpu::Neon(cpu) => double_neon(x, cpu),
        DynCpu::Generic(_) => double_generic(x),
    }
}

#[cpu_version(Avx2)]
#[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` function")]
#[inline(always)]
fn triple_avx2(x: u32, _cpu: Avx2) -> u32 {
    x * 3
}

#[cpu_version(Neon)]
#[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` function")]
#[inline(always)]
fn triple_neon(x: u32, _cpu: Neon) -> u32 {
    x * 3
}

/// `triple_avx2` and `triple_neon` are `#[inline(always)]`, so have no `#[target_feature]`,
/// and calling them is safe.
fn triple(x: u32, cpu: impl Cpu) -> u32 {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => triple_avx2(x, cpu),
        DynCpu::Neon(cpu) => triple_neon(x, cpu),
        DynCpu::Generic(_) => x * 3,
    }
}

/// A `cpu` parameter doesn't have to be the last parameter.
#[cpu_version(Avx2)]
#[inline]
fn quadruple_avx2(_cpu: Avx2, x: u32) -> u32 {
    x * 4
}

#[cpu_version(Neon)]
#[inline]
fn quadruple_neon(_cpu: Neon, x: u32) -> u32 {
    x * 4
}

fn quadruple(x: u32, cpu: impl Cpu) -> u32 {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => quadruple_avx2(cpu, x),
        DynCpu::Neon(cpu) => quadruple_neon(cpu, x),
        DynCpu::Generic(_) => x * 4,
    }
}

/// A vector parameter proves the CPU supports the feature set, in place of a `cpu` parameter.
#[cpu_version(Avx2)]
#[inline]
fn is_zero_avx2(v: Vec128<Avx2>) -> bool {
    v.is_zero()
}

#[cpu_version(Neon)]
#[inline]
fn is_zero_neon(v: Vec128<Neon>) -> bool {
    v.is_zero()
}

fn is_zero(cpu: impl Cpu) -> bool {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => is_zero_avx2(Vec128::splat(0, cpu)),
        DynCpu::Neon(cpu) => is_zero_neon(Vec128::splat(0, cpu)),
        DynCpu::Generic(_) => true,
    }
}

/// A vector parameter doesn't have to be the last parameter.
#[cpu_version(Avx2)]
#[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` function")]
#[inline(always)]
fn add_if_zero_avx2(x: u32, v: Vec256<Avx2>, y: u32) -> u32 {
    if v.is_zero() { x + y } else { x }
}

#[cpu_version(Neon)]
#[expect(clippy::inline_always, reason = "tests an `#[inline(always)]` function")]
#[inline(always)]
fn add_if_zero_neon(x: u32, v: Vec256<Neon>, y: u32) -> u32 {
    if v.into_halves().0.is_zero() { x + y } else { x }
}

fn add_if_zero(x: u32, y: u32, cpu: impl Cpu) -> u32 {
    match to_dyn(cpu) {
        DynCpu::Avx2(cpu) => add_if_zero_avx2(x, Vec256::splat(0, cpu), y),
        DynCpu::Neon(cpu) => add_if_zero_neon(x, Vec256::splat(0, cpu), y),
        DynCpu::Generic(_) => x + y,
    }
}

/// An `unsafe` function isn't wrapped, so it has `#[target_feature]`, as well as being unsafe.
#[cpu_version(Avx2)]
#[inline]
unsafe fn halve_avx2(x: u32, _cpu: Avx2) -> u32 {
    x / 2
}

#[cpu_version(Neon)]
#[inline]
unsafe fn halve_neon(x: u32, _cpu: Neon) -> u32 {
    x / 2
}

fn halve(x: u32, cpu: impl Cpu) -> u32 {
    match to_dyn(cpu) {
        // SAFETY: `Avx2` type guarantees the CPU supports the features `halve_avx2` uses
        DynCpu::Avx2(cpu) => unsafe { halve_avx2(x, cpu) },
        // SAFETY: `Neon` type guarantees the CPU supports the features `halve_neon` uses
        DynCpu::Neon(cpu) => unsafe { halve_neon(x, cpu) },
        DynCpu::Generic(_) => x / 2,
    }
}

/// Tests of functions which `cpu_version!` wraps in a safe function, for one CPU feature set.
macro_rules! wrapped_tests {
    ($module:ident, $cpu:ident) => {
        mod $module {
            use super::*;

            #[cpu_version($cpu)]
            #[inline(never)]
            fn generics<'a, 'b: 'a, T: Copy + Into<u32>, const N: usize>(
                mut xs: &'a [T; N],
                ys: &'b [T; N],
                use_ys: bool,
                _cpu: $cpu,
            ) -> u32 {
                if use_ys {
                    xs = ys;
                }
                xs[N - 1].into()
            }

            // `T` can only be passed with a turbofish
            #[cpu_version($cpu)]
            #[inline]
            const fn size<T>(_cpu: $cpu) -> usize {
                size_of::<T>()
            }

            // `T` might need dropping
            #[cpu_version($cpu)]
            #[inline]
            const fn identity<T>(x: T, _cpu: $cpu) -> T {
                x
            }

            // Only `v` proves the CPU supports the feature set, and it's not the last parameter.
            // Function has no inline attribute, and has a `,` after the last parameter.
            #[cpu_version($cpu)]
            fn add_if_zero(x: u32, v: Vec128<$cpu>, y: u32) -> u32 {
                if v.is_zero() { x + y } else { x }
            }

            // `Self` in nested items isn't the function's, so this isn't a method
            #[cpu_version($cpu)]
            fn nested_items(x: u32, _cpu: $cpu) -> u32 {
                struct Inner(u32, Option<Box<Self>>);
                impl Inner {
                    fn new(x: u32) -> Self {
                        Self(x, Some(Box::new(Self(1, None))))
                    }
                    fn total(&self) -> u32 {
                        self.0 + self.1.as_ref().map_or(0, |inner| inner.0)
                    }
                }
                Inner::new(x).total()
            }

            struct Total(u32);

            impl Total {
                #[cpu_version($cpu)]
                #[inline]
                fn plus(&self, x: u32, _cpu: $cpu) -> u32 {
                    self.0 + x
                }

                #[cpu_version($cpu)]
                #[inline(never)]
                fn add(&mut self, x: u32, _cpu: $cpu) {
                    self.0 += x;
                }

                // `Self` in the signature, and in the body
                #[cpu_version($cpu)]
                fn doubled(self, _cpu: $cpu) -> Self {
                    Self(self.0 * 2)
                }

                // No `self` parameter, but `Self` in the return type and body
                #[cpu_version($cpu)]
                fn of(n: u32, _cpu: $cpu) -> Self {
                    Self(n)
                }

                // No `self` parameter, but `Self` in the body only
                #[cpu_version($cpu)]
                #[inline]
                fn double_of(n: u32, _cpu: $cpu) -> u32 {
                    Self(n).0 * 2
                }

                // Lint attributes go on `__plus_unused_impl`, where the lint fires
                #[cpu_version($cpu)]
                #[expect(
                    unused_variables,
                    reason = "tests lint attributes go on the function with the body"
                )]
                fn plus_unused(&self, x: u32, _cpu: $cpu) -> u32 {
                    let unused = x;
                    self.0
                }

                // rustc evaluates `#[cfg]` before the macro runs, wherever it is in the list.
                // The macro doesn't have to remove `__removed_impl`, or this would fail to compile.
                #[cpu_version($cpu)]
                #[cfg(any())]
                fn removed(&self, _cpu: $cpu) -> Missing {
                    Missing
                }

                // `#[track_caller]` goes on `__caller_line_impl` too, once
                #[cpu_version($cpu)]
                #[track_caller]
                fn caller_line(&self, _cpu: $cpu) -> u32 {
                    Location::caller().line()
                }

                #[cpu_version($cpu)]
                fn into_incremented(mut self, _cpu: $cpu) -> u32 {
                    self.0 += 1;
                    self.0
                }

                #[cpu_version($cpu)]
                #[inline]
                const fn plus_size<T, const N: usize>(&self, _cpu: $cpu) -> usize {
                    self.0 as usize + size_of::<T>() + N
                }

                // Only `v` proves the CPU supports the feature set
                #[cpu_version($cpu)]
                #[inline]
                fn plus_if_zero(&self, x: u32, v: Vec128<$cpu>) -> u32 {
                    if v.is_zero() { self.0 + x } else { self.0 }
                }
            }

            struct Wrapper<W>(W);

            impl<W: Copy> Wrapper<W> {
                // Bound mentions the `impl`'s generic parameter, and `>>` ends the generics
                #[cpu_version($cpu)]
                #[inline]
                fn convert<T: From<Vec<W>>>(&self, _cpu: $cpu) -> T {
                    T::from(vec![self.0])
                }
            }

            /// All these calls are safe.
            #[test]
            fn wrapped() {
                // This module can see the CPU types' private fields. So on aarch64 with Neon,
                // rustc knows that `Avx2` and `Generic` are uninhabited, and for `Neon` this always
                // matches. `allow`, not `expect`, because for `Avx2` it doesn't.
                #[allow(irrefutable_let_patterns, clippy::allow_attributes)]
                let DynCpu::$cpu(cpu) = DynCpu::detect() else { return };

                assert_eq!(generics(&[1u8, 2], &[3, 4], false, cpu), 2);
                assert_eq!(generics(&[1u8, 2], &[3, 4], true, cpu), 4);
                assert_eq!(size::<u64>(cpu), 8);
                assert_eq!(identity(String::from("x"), cpu), "x");
                assert_eq!(add_if_zero(1, Vec128::splat(0, cpu), 2), 3);
                assert_eq!(add_if_zero(1, Vec128::splat(1, cpu), 2), 1);
                assert_eq!(nested_items(5, cpu), 6);

                let mut total = Total(1);
                total.add(2, cpu);
                assert_eq!(total.plus(3, cpu), 6);
                assert_eq!(total.plus_size::<u16, 1>(cpu), 6);
                assert_eq!(total.plus_if_zero(1, Vec128::splat(0, cpu)), 4);
                assert_eq!(total.doubled(cpu).into_incremented(cpu), 7);
                assert_eq!(Total::of(5, cpu).0, 5);
                assert_eq!(Total::double_of(6, cpu), 12);
                assert_eq!(Total(2).plus_unused(1, cpu), 2);
                assert_eq!(Total(2).caller_line(cpu), line!());

                assert_eq!(Wrapper(3u8).convert::<Vec<u8>>(cpu), vec![3]);
            }
        }
    };
}

wrapped_tests!(wrapped_avx2, Avx2);

wrapped_tests!(wrapped_neon, Neon);

/// Generics with `>>` and a `(...)` in them.
#[cpu_version(Generic)]
fn nth_mapped<T: Into<Vec<u8>>, F: FnOnce(u8) -> u8, const N: usize>(bytes: T, f: F) -> u8 {
    f(bytes.into()[N])
}

impl Counter {
    #[cpu_version(Generic)]
    const fn plus_generic(&self, x: u32) -> u32 {
        self.0 + x
    }
}

#[test]
fn cpu_version_fns() {
    let cpu = DynCpu::detect();

    assert_eq!(double(21, cpu), 42);
    assert_eq!(double(21, BASELINE), 42);
    assert_eq!(triple(5, cpu), 15);
    assert_eq!(triple(5, BASELINE), 15);
    assert_eq!(quadruple(5, cpu), 20);
    assert_eq!(quadruple(5, BASELINE), 20);
    assert_eq!(halve(20, cpu), 10);
    assert_eq!(halve(20, BASELINE), 10);
    assert!(is_zero(cpu));
    assert!(is_zero(BASELINE));
    assert_eq!(add_if_zero(1, 2, cpu), 3);
    assert_eq!(add_if_zero(1, 2, BASELINE), 3);
    assert_eq!(nth_mapped::<_, _, 1>([1, 2], |byte| byte + 1), 3);
    assert_eq!(Counter(100).plus_generic(1), 101);
}
