use super::{BASELINE, Cpu, DynCpu, apply, cpu_select, cpu_versions, to_dyn};

#[apply(cpu_versions)]
const fn add_one(x: u32, _cpu: impl Cpu) -> u32 {
    x + 1
}

#[apply(cpu_versions)]
const unsafe fn read_byte(ptr: *const u8, _cpu: impl Cpu) -> u8 {
    // SAFETY: Caller guarantees `ptr` is valid for reads
    unsafe { *ptr }
}

#[apply(cpu_versions)]
const fn larger<'a>(x: &'a u8, y: &'a u8, _cpu: impl Cpu) -> &'a u8 {
    if *x >= *y { x } else { y }
}

#[apply(cpu_versions)]
const fn larger_at<'a, const N: usize>(xs: &'a [u8], ys: &'a [u8], _cpu: impl Cpu) -> &'a u8 {
    if xs[N] >= ys[N] { &xs[N] } else { &ys[N] }
}

// `T` can only be passed with a turbofish
#[apply(cpu_versions)]
const fn size<T>(_cpu: impl Cpu) -> usize {
    size_of::<T>()
}

#[apply(cpu_versions)]
const fn first<T: Copy, const N: usize>(xs: [T; N], _cpu: impl Cpu) -> T {
    xs[0]
}

// `T` might need dropping
#[apply(cpu_versions)]
const unsafe fn identity<T>(x: T, _cpu: impl Cpu) -> T {
    x
}

#[apply(cpu_versions)]
#[inline(never)]
fn with_bounds<X, Y: Copy, Z: Into<u32>>(x: X, y: Y, z: Z, _cpu: impl Cpu) -> (X, Y, u32) {
    (x, y, z.into())
}

// `Q` is another generic parameter
#[apply(cpu_versions)]
fn param_in_bound<Q, Z: Into<Q>>(z: Z, _cpu: impl Cpu) -> Q {
    z.into()
}

// `>>` closes 2 `<`s, before the end of the generic parameters
#[apply(cpu_versions)]
fn nested<Z: Into<Vec<u8>>, W>(z: Z, w: W, _cpu: impl Cpu) -> (Vec<u8>, W) {
    (z.into(), w)
}

// `>>>>` is `>>` twice
#[apply(cpu_versions)]
fn nested_at_end<Z: Into<Vec<Vec<u8>>>>(z: Z, _cpu: impl Cpu) -> Vec<Vec<u8>> {
    z.into()
}

// `>>>` is `>>` then `>`
#[apply(cpu_versions)]
fn nested_3<Z: Into<Vec<Vec<u8>>>, W>(z: Z, w: W, _cpu: impl Cpu) -> (Vec<Vec<u8>>, W) {
    (z.into(), w)
}

// `<<` opens 2 `<`s
#[apply(cpu_versions)]
fn qualified_path<U: IntoIterator, Z: Into<<U as IntoIterator>::Item>>(
    z: Z,
    _cpu: impl Cpu,
) -> <U as IntoIterator>::Item {
    z.into()
}

#[apply(cpu_versions)]
fn multiple_bounds<'a, T: ?Sized + AsRef<[u8]> + 'a>(x: &'a T, _cpu: impl Cpu) -> &'a [u8] {
    x.as_ref()
}

#[apply(cpu_versions)]
fn lifetime_bounds<'a, 'b: 'a, T: 'b + Copy>(x: &'a &'b T, _cpu: impl Cpu) -> T {
    **x
}

// Bound contains `,` inside `(...)`, and `->`
#[apply(cpu_versions)]
fn fn_bound<F: Fn(u8, u8) -> Vec<u8>, W>(f: F, w: W, _cpu: impl Cpu) -> (Vec<u8>, W) {
    (f(1, 2), w)
}

#[apply(cpu_versions)]
fn higher_ranked<F: for<'x> Fn(&'x u8) -> &'x u8>(f: F, x: &u8, _cpu: impl Cpu) -> u8 {
    *f(x)
}

#[apply(cpu_versions)]
fn assoc_type<I: Iterator<Item = u8>>(iter: I, _cpu: impl Cpu) -> u32 {
    iter.map(u32::from).sum()
}

// Const and type parameters in any order, with a trailing comma
#[apply(cpu_versions)]
#[rustfmt::skip]
fn mixed<A: Copy, const N: usize, B: Copy, const M: usize,>(
    a: [A; N],
    b: [B; M],
    _cpu: impl Cpu,
) -> (A, B) {
    (a[0], b[0])
}

struct Counter(u32);

impl Counter {
    #[apply(cpu_versions)]
    const fn plus(&self, x: u32, _cpu: impl Cpu) -> u32 {
        self.0 + x
    }

    #[apply(cpu_versions)]
    const unsafe fn plus_byte(&self, ptr: *const u8, _cpu: impl Cpu) -> u32 {
        // SAFETY: Caller guarantees `ptr` is valid for reads
        self.0 + unsafe { *ptr } as u32
    }

    #[apply(cpu_versions)]
    const fn at<'a, const N: usize>(&self, bytes: &'a [u8], _cpu: impl Cpu) -> &'a u8 {
        &bytes[self.0 as usize + N]
    }

    #[apply(cpu_versions)]
    const fn plus_size<T>(&self, _cpu: impl Cpu) -> usize {
        self.0 as usize + size_of::<T>()
    }

    // `>>` ends the generic parameters. They go through `pastey`, unlike a function's.
    #[apply(cpu_versions)]
    fn plus_len<const N: usize, Z: Into<Vec<u8>>>(&self, z: Z, _cpu: impl Cpu) -> usize {
        self.0 as usize + z.into().len() + N
    }

    // Bound mentions `Self`
    #[apply(cpu_versions)]
    fn plus_counter<Z: Into<Self>>(self, z: Z, _cpu: impl Cpu) -> u32 {
        self.0 + z.into().0
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
    #[apply(cpu_versions)]
    fn convert<T: From<W>>(&self, _cpu: impl Cpu) -> T {
        T::from(self.0)
    }
}

/// [`BASELINE`] as a [`DynCpu`].
const BASELINE_DYN: DynCpu = to_dyn(BASELINE);

/// Calls to `const` functions made with `cpu_versions!`, evaluated at compile time.
const AT_COMPILE_TIME: [u32; 9] = {
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
    ]
};

/// Calls to `const` functions which return a `usize`, evaluated at compile time.
const SIZES_AT_COMPILE_TIME: [usize; 2] =
    [size::<u64>(BASELINE_DYN), Counter(1).plus_size::<u16>(BASELINE)];

#[test]
fn const_fn_at_compile_time() {
    assert_eq!(AT_COMPILE_TIME, [2, 11, 101, 2, 102, 2, 3, 2, 4]);
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

    cpu_select!(cpu, {
        Avx2(cpu) => assert_eq!(add_one(1, cpu), 2),
        Neon(cpu) => assert_eq!(add_one(1, cpu), 2),
        Generic(cpu) => assert_eq!(add_one(1, cpu), 2),
    });
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
