/// Choose a vector method's code for the CPU feature set of its vectors.
///
/// ```ignore
/// pub fn splat(n: u8, cpu: C) -> Self {
///     cpu_select!(cpu, {
///         Avx2 => {
///             // SAFETY: `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2
///             Self::new(unsafe { _mm256_set1_epi8(n.cast_signed()) }, cpu)
///         }
///         _ => {
///             let half = Vec128::splat(n, cpu);
///             Self::from_halves(half, half)
///         }
///     })
/// }
/// ```
///
/// The first argument is the method's CPU type value (e.g. `cpu`, or `self.cpu()`).
/// The macro matches on [`DynCpu::from`] of it, and runs the arm for that CPU type:
///
/// - `Avx2` arm runs if the CPU type is [`Avx2`]. It's only compiled on x86_64.
/// - `Neon` arm runs if the CPU type is [`Neon`]. It's only compiled on aarch64.
/// - `_` arm runs for any other CPU type. It's compiled on every platform,
///   so it can't use intrinsics, and it can't use `inner`, whose type depends on the platform.
///
/// The `_` arm is compiled everywhere so that it's type-checked and borrow-checked
/// everywhere, and rust-analyzer can resolve the names in it on any platform.
///
/// The `Avx2` and `Neon` arms are optional, but must be in that order. The `_` arm is required.
/// When no code works for every CPU type, use [`unused!`] for the `_` arm:
/// `_ => unused!(n, cpu)`.
///
/// It expands to a `match` on [`DynCpu::from`] with an arm for each CPU type,
/// each behind a `#[cfg]`, followed by the `_` arm.
/// So every arm must have the method's return type, even on platforms where it can't run.
///
/// The `Avx2` arm only runs if the CPU type is `Avx2`, and a value of that type exists,
/// which proves that the CPU supports AVX2. So the arm can use AVX2 instructions.
/// Likewise for the `Neon` arm. `C` is still generic in the arm, but a vector wraps
/// the platform's SIMD type whatever `C` is, so the arm can pass `inner` to intrinsics.
///
/// [`DynCpu::from`]: crate::cpu::DynCpu::from
/// [`Avx2`]: crate::cpu::Avx2
/// [`Neon`]: crate::cpu::Neon
macro_rules! cpu_select {
    (
        $cpu:expr,
        {
            $(Avx2 => $avx2:block)?
            $(Neon => $neon:block)?
            _ => $other:expr $(,)?
        }
    ) => {
        match $crate::DynCpu::from($cpu) {
            $(
                #[cfg(target_arch = "x86_64")]
                $crate::DynCpu::Avx2(_) => $avx2
            )?
            $(
                #[cfg(target_arch = "aarch64")]
                $crate::DynCpu::Neon(_) => $neon
            )?
            _ => $other,
        }
    };
}

pub(super) use cpu_select;

/// The `_` arm of [`cpu_select!`] in a method which has no code that works on every platform.
///
/// ```ignore
/// cpu_select!(self.cpu(), {
///     Avx2 => { ... }
///     Neon => { ... }
///     _ => unused!(self, other)
/// })
/// ```
///
/// Uses the method's parameters, so the compiler doesn't warn that they're unused on platforms
/// where the `_` arm is the only one compiled, and then panics with `unreachable!()`.
/// It moves them into a tuple, because `let _ = param;` doesn't move `param`,
/// and clippy's `needless_pass_by_value` would then fire for parameters which aren't `Copy`.
///
/// It never runs, because the method has an arm for every CPU type it can be called with.
macro_rules! unused {
    ($($param:expr),+ $(,)?) => {{
        let _ = ($($param,)+);
        unreachable!()
    }};
}

pub(super) use unused;
