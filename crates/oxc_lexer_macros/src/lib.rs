//! Proc macros for `oxc_lexer`'s CPU feature sets.
//!
//! `oxc_lexer` has a type for each CPU feature set which it has SIMD code for (`Avx2` and `Neon`),
//! and `Generic` for none. Holding one proves that the CPU supports that feature set.
//! See the docs on its `cpu` module.
//!
//! These macros generate the code which compiles a function for each feature set:
//!
//! - [`cpu_multiversion`]: One function, compiled once for each feature set.
//! - [`cpu_version`]: One version of some code, for one feature set.
//! - [`cpu_impl`]: Methods of one feature set's types, which exist on every platform,
//!   but only work on the platform which has the feature set.
//!
//! `oxc_lexer` code chooses between versions with a `match` on `to_dyn(cpu)`.
//!
//! The macros are only for `oxc_lexer`.
//! Their output refers to its `cpu` and `simd` modules by path.

#![warn(clippy::missing_docs_in_private_items)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Result};

mod cpu;
mod func;
mod impl_block;
mod multiversion;
mod version;

/// Turns a function into one with a separate entry point for each CPU feature set.
///
/// Used for every function which is compiled for each CPU feature set.
/// Each entry point is compiled with its feature set's target features enabled,
/// so SIMD functions which the function calls can be inlined into it.
///
/// ```ignore
/// use crate::cpu::{Cpu, cpu_multiversion};
///
/// /// Compress tokens.
/// #[cpu_multiversion]
/// #[inline(never)]
/// pub unsafe fn compress(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///     // ...
/// }
/// ```
///
/// `compress` becomes a function which calls one of 3 entry points, depending on the type of `cpu`.
/// It's `#[inline(always)]`, so when called with a specific type (e.g. `Avx2`),
/// it compiles to a direct call to that entry point.
///
/// The entry points are functions defined inside `compress`:
///
/// - `__avx2`: Takes `cpu: Avx2`. Compiled with AVX2 enabled on x86_64.
/// - `__neon`: Takes `cpu: Neon`. Compiled with Neon enabled on aarch64.
/// - `__generic`: Takes `cpu: Generic`.
///
/// All 3 exist on every platform. On a platform which doesn't have a SIMD feature set,
/// that feature set's type is uninhabited, so its entry point can never be called,
/// and the compiler removes it. Its target features only apply on the platform which has them,
/// so there are none there. `Generic` is uninhabited on aarch64 with Neon enabled at compile time,
/// so the same applies to `__generic` there.
///
/// Where the feature set's type is uninhabited, the entry point starts with `unreachable!()`,
/// before it calls `__impl`.
/// Without it, the copies of every function for that feature set (e.g. `__impl::<Neon>`
/// on x86_64) would be part of the call graph, even though they can never run. That changes how
/// rustc's MIR inliner treats other code, which changes the code it compiles to, e.g. in functions
/// which only have a generic version. rustc removes code after `unreachable!()` early enough
/// that this doesn't happen.
///
/// The call to `__impl` stays, so the entry point still uses its parameters. Without the call,
/// clippy would report lints on their types which don't apply, e.g. `ptr_arg` on a `&mut Vec`.
/// `#[allow(unreachable_code)]` silences rustc's warning that the call can't be reached.
/// On the platform which has the feature set, there's no `unreachable!()`, so it does nothing.
/// It can't affect the original code, which is in `__impl`, a separate function.
///
/// `__impl` is still compiled, type-checked and linted on every platform, because it's generic
/// over the CPU type, and rustc checks a generic function's body once, for all types.
/// Only the copy of it for that feature set isn't generated.
///
/// Each entry point calls `__impl`, an `#[inline(always)]` function containing the original
/// function body. So the body is compiled as part of each entry point, and the AVX2 functions
/// it calls can be inlined into the AVX2 entry point. They can't be inlined into a function
/// which doesn't have AVX2 enabled.
///
/// A function defined inside `compress` shadows a parameter with the same name in its body.
/// The names start with `__`, so they don't clash with any real parameter names.
///
/// Methods are supported too, with a `self`, `&self`, `&mut self`, `mut self` or `self: Type`
/// parameter, and so are associated functions which use `Self`. A function nested inside them
/// can't use `self` or `Self`, so their entry points are methods next to them in the same `impl`,
/// rather than functions inside them. Their names include the function's name, e.g. for a method
/// `pipeline`: `__pipeline_impl`, `__pipeline_generic`, `__pipeline_avx2` and `__pipeline_neon`.
/// They're private, so they're only visible in the module which defines the method.
///
/// They aren't methods of a trait defined inside the method, because rustc gives trait methods
/// external linkage. That stops LLVM from inlining helpers into them, and from removing the
/// generic entry point on aarch64 with Neon, where it can never be called.
///
/// Attributes go on the generated function where they take effect:
///
/// - `#[inline]` and `#[cold]` go on the entry points. They're the functions which get called.
/// - Lint attributes (`#[expect]`, `#[allow]`, etc.) go on `__impl`, where the lints fire.
///   On a method, they wouldn't cover its sibling `__pipeline_impl`.
/// - `#[track_caller]` goes on all of them, so the caller's location passes through.
/// - All other attributes (doc comments, `#[must_use]`, etc.) go on `compress`.
///
/// `#[cfg]` and `#[cfg_attr]` are evaluated by rustc before the macro runs, wherever they are
/// in the list. `#[target_feature]` is an error, because the macro adds it.
///
/// `#[inline(never)]` keeps the entry points as separate functions.
/// Without it, they can be inlined into their callers in builds which enable AVX2 globally.
///
/// An `#[inline(always)]` function is output unchanged, after the same checks as any other.
/// It's compiled for each CPU feature set as part of its callers' entry points, so it needs
/// none of its own. The macro only marks it as code which is compiled for each feature set.
/// It couldn't have entry points anyway, because `#[inline(always)]` can't be combined with
/// `#[target_feature]`.
///
/// When SSE2 is enabled for the whole crate, `__generic` also gets
/// `#[target_feature(enable = "sse2")]`. That's the case on nearly all x86_64 and x86 targets.
/// It doesn't change the code `__generic` compiles to, because SSE2 is enabled anyway.
/// It stops rustc's MIR inliner from inlining `__generic` into `compress`.
///
/// Without it, an `#[inline]` function would act as if it was `#[inline(always)]` in its
/// generic version. `__generic` only contains a call to `__impl`, so it's small enough for
/// the MIR inliner to inline it. `compress` and `__impl` are both `#[inline(always)]`,
/// so the whole body would then be inlined into every caller, which bloats the generic version
/// of the callers.
///
/// The MIR inliner doesn't inline a function with target features which its caller doesn't have,
/// and it ignores features which are enabled for the whole crate. `__avx2` and `__neon` are
/// protected by their own target features in the same way, on the platforms which have them.
///
/// Only `#[inline]` functions need this. The MIR inliner never inlines `#[inline(never)]`
/// functions, and only inlines functions without an inline attribute if they don't call
/// any other function.
///
/// The only x86_64 targets which don't enable SSE2 are these 2, and Oxc can't or won't run
/// on either:
///
/// - `x86_64-unknown-none`: Bare metal, e.g. operating system kernels.
///   It has no `std`, which Oxc requires.
/// - `x86_64-unknown-uefi`: UEFI firmware programs, which run before an operating system starts.
///   `std` only partly supports it.
///
/// SSE2 can also be disabled with `-C target-feature=-sse2`, but rustc warns that the x86_64 ABI
/// needs it, and that this will become a hard error.
///
/// Most 32-bit x86 targets enable SSE2 too. The exceptions are the `i586` targets, for CPUs from
/// the 1990s, and `i686-unknown-uefi`. 32-bit x86 has no AVX2 version, so there's only one
/// version of the code there.
///
/// Where SSE2 isn't enabled, `__generic` doesn't get the attribute. The code is still correct,
/// only bigger.
///
/// The last parameter must be `cpu: impl Cpu` (any name instead of `cpu` is fine).
/// The other parameters must be `name: Type` or `mut name: Type` (no other patterns),
/// and can't be called `__impl`, `__avx2`, etc.
/// `mut` only goes on `__impl`'s parameters. The other functions just pass the values on.
///
/// The function can be a `const fn`, `unsafe fn` or `const unsafe fn`,
/// and can have generic parameters of any kind, with bounds or a `where` clause.
///
/// ## Expansion
///
/// The example above expands to:
///
/// ```ignore
/// /// Compress tokens.
/// #[inline(always)]
/// pub unsafe fn compress(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///     #[inline(always)]
///     unsafe fn __impl(src: &[u8], n: usize, cpu: impl Cpu) -> usize {
///         // ...
///     }
///
///     #[inline(never)]
///     #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2,bmi2,popcnt"))]
///     #[allow(unreachable_code)]
///     unsafe fn __avx2(src: &[u8], n: usize, cpu: Avx2) -> usize {
///         // `Avx2` is uninhabited on other platforms. See above.
///         #[cfg(not(target_arch = "x86_64"))]
///         unreachable!();
///         __impl(src, n, cpu)
///     }
///
///     #[inline(never)]
///     #[cfg_attr(target_arch = "aarch64", target_feature(enable = "neon"))]
///     #[allow(unreachable_code)]
///     unsafe fn __neon(src: &[u8], n: usize, cpu: Neon) -> usize {
///         // `Neon` is uninhabited on other platforms. See above.
///         #[cfg(not(target_arch = "aarch64"))]
///         unreachable!();
///         __impl(src, n, cpu)
///     }
///
///     #[inline(never)]
///     // Stops the MIR inliner inlining `__generic` into this function. See the docs above.
///     #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
///     #[allow(unreachable_code)]
///     unsafe fn __generic(src: &[u8], n: usize, cpu: Generic) -> usize {
///         // `Generic` is uninhabited on aarch64 with Neon enabled. See above.
///         #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
///         unreachable!();
///         __impl(src, n, cpu)
///     }
///
///     match to_dyn(cpu) {
///         // SAFETY: `Avx2` type guarantees the CPU supports the features `__avx2` uses
///         DynCpu::Avx2(cpu) => unsafe { __avx2(src, n, cpu) },
///         // SAFETY: `Neon` type guarantees the CPU supports the features `__neon` uses
///         DynCpu::Neon(cpu) => unsafe { __neon(src, n, cpu) },
///         // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
///         // when SSE2 is enabled for the whole crate, so the CPU supports it
///         DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///     }
/// }
/// ```
///
/// For a method, the `match` is the whole body of the outer method,
/// and the other functions are methods next to it.
#[proc_macro_attribute]
pub fn cpu_multiversion(args: TokenStream, item: TokenStream) -> TokenStream {
    output(multiversion::expand(args.into(), item.into()))
}

/// Marks a function as the version of some code for one CPU feature set.
///
/// A `match` on `to_dyn(cpu)` picks between the versions.
///
/// Applied to a function with one of:
///
/// - `#[cpu_version(Avx2)]`
/// - `#[cpu_version(Neon)]`
/// - `#[cpu_version(Generic)]`
///
/// ```ignore
/// use crate::cpu::{Avx2, cpu_version};
///
/// #[cpu_version(Avx2)]
/// #[inline]
/// fn bm_any_avx2(bm: &[u64], _cpu: Avx2) -> bool {
///     // ...
/// }
/// ```
///
/// - `Avx2`:
///   A parameter must prove that the CPU supports AVX2.
///   Any parameter can do that, if its type is `Avx2`, `Vec128<Avx2>` or `Vec256<Avx2>`.
///   A vector can only exist if the CPU supports its feature set.
///   The type must be written with its `<Avx2>`, because the macro only sees the type's name.
///
///   What the function gets depends on its kind:
///   - `#[inline(always)]`: Nothing, because it can't be combined with `#[target_feature]`.
///     It's compiled as part of its callers, with their features.
///   - `unsafe fn`: It gets
///     `#[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2,bmi2,popcnt"))]`.
///   - Any other function: It's wrapped in a safe function. See below.
///
/// - `Neon`:
///   The same, with `Neon`, `Vec128<Neon>` or `Vec256<Neon>`,
///   and `#[cfg_attr(target_arch = "aarch64", target_feature(enable = "neon"))]`.
///
/// `Avx2` and `Neon` versions exist on every platform, so they're compiled and type-checked
/// everywhere. Their target features only apply on the platform which has them.
/// Elsewhere, the proof parameter's type is uninhabited, so the function can never be called.
///
/// - `Generic`:
///   No parameter can have a CPU type (`Avx2`, `Neon`, `Generic`, `DynCpu` or `impl Cpu`),
///   or be a `Vec128` or `Vec256`.
///   A generic version only runs where the CPU feature set is `BASELINE`'s,
///   so it uses `BASELINE` where it needs a CPU type. The function gets no attributes.
///
///   It's also the attribute for functions which only have a generic version, like the whole
///   cold path. It checks that nothing passes them a CPU type, which would compile a copy of
///   them for each one.
///
/// A function which isn't wrapped is otherwise output from its original tokens, with the `let _`
/// check described below at the start of its body. So it compiles, and is linted, as written.
///
/// The parameters must be `name: Type` or `mut name: Type` (no other patterns).
///
/// ## Safe functions
///
/// Calling a function with `#[target_feature]` needs an `unsafe` block, unless the caller
/// has the same features. There's no reason for that when the function has to receive proof that
/// the CPU supports those features. So the function is wrapped in one which is safe to call.
///
/// ```ignore
/// #[cpu_version(Avx2)]
/// #[inline(never)]
/// fn foo<const N: usize>(mut arr: [u64; N], cpu: Avx2) -> bool {
///     // ...
/// }
/// ```
///
/// expands to:
///
/// ```ignore
/// #[inline(always)]
/// fn foo<const N: usize>(arr: [u64; N], cpu: Avx2) -> bool {
///     #[inline(never)]
///     #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2,bmi2,popcnt"))]
///     fn __impl<const N: usize>(mut arr: [u64; N], cpu: Avx2) -> bool {
///         // ...
///     }
///
///     let _: crate::cpu::Avx2 = cpu;
///
///     // SAFETY: The check above shows this parameter is a type which guarantees
///     // that the CPU supports the target features which `__impl` has
///     unsafe { __impl::<N>(arr, cpu) }
/// }
/// ```
///
/// The `let _` is required for soundness. The macro only sees the type's name, so it can't tell
/// that a parameter's type `Avx2` is really `crate::cpu::Avx2`. It might be a type alias or
/// a generic parameter with that name. The `let _` fails to compile if so, and generates no code.
///
/// Attributes go where [`cpu_multiversion`] puts them. `__impl` is the entry point here, so it
/// gets `#[inline]`, `#[cold]` and lint attributes. All other attributes go on `foo`, and `mut`
/// only goes on `__impl`'s parameters.
///
/// A nested function can't use `self` or `Self`, so for a method, or an associated function
/// which uses `Self`, `__impl` is a private method next to it, called `__foo_impl`. So it can't
/// be in a trait implementation.
///
/// A nested function can't use the generic parameters of an `impl` either.
/// So a wrapped associated function which doesn't use `Self` can't use them.
#[proc_macro_attribute]
pub fn cpu_version(args: TokenStream, item: TokenStream) -> TokenStream {
    output(version::expand(args.into(), item.into()))
}

/// Marks an `impl` block as methods of one CPU feature set's types,
/// which only work on the platform which has that feature set.
///
/// Applied to an `impl` block with one of:
///
/// - `#[cpu_impl(Avx2)]`: The method bodies are only compiled on x86_64.
/// - `#[cpu_impl(Neon)]`: The method bodies are only compiled on aarch64.
///
/// On other platforms, each method's body is replaced with `unreachable!()`.
/// Its signature is unchanged, so the method exists on every platform,
/// and code which calls it compiles everywhere.
///
/// Every method must take a value which proves that the CPU supports the feature set,
/// e.g. `self` of a `Vec256<Avx2>`, or a `cpu: Avx2` parameter. On other platforms, the feature
/// set's type is uninhabited, so that value can't exist, and the method can never be called.
/// The macro doesn't check this. A method which doesn't take one would panic if called
/// on another platform.
///
/// ```ignore
/// #[cpu_impl(Avx2)]
/// impl Vec256<Avx2> {
///     /// Gather the top bit of each byte of this [`Vec256`] into a 32-bit mask.
///     #[inline(always)]
///     pub fn movemask(self) -> u32 {
///         // SAFETY: `Vec256` type guarantees platform supports AVX2
///         unsafe { _mm256_movemask_epi8(self.inner).cast_unsigned() }
///     }
/// }
/// ```
///
/// expands to:
///
/// ```ignore
/// #[cfg_attr(
///     not(target_arch = "x86_64"),
///     allow(dead_code, unused_variables, clippy::unused_self, clippy::needless_pass_by_value)
/// )]
/// impl Vec256<Avx2> {
///     /// Gather the top bit of each byte of this [`Vec256`] into a 32-bit mask.
///     #[inline(always)]
///     pub fn movemask(self) -> u32 {
///         cfg_select! {
///             target_arch = "x86_64" => {
///                 // SAFETY: `Vec256` type guarantees platform supports AVX2
///                 unsafe { _mm256_movemask_epi8(self.inner).cast_unsigned() }
///             }
///             _ => unreachable!(),
///         }
///     }
/// }
/// ```
///
/// On other platforms, nothing calls the methods, and their bodies don't use their parameters.
/// The `allow` silences the lints for that. It's `allow`, not `expect`, because which of them fire
/// depends on the methods. e.g. methods of a trait implementation are never dead code,
/// and clippy doesn't lint their parameters.
///
/// Only method bodies change. Everything else is output unchanged, so it must compile on every
/// platform. That includes the types in the methods' signatures, and other items in the `impl`
/// block (e.g. `type Inner = __m256i;`). `__m256i` only exists on x86_64.
///
/// The tokens of the `impl` block are scanned, not parsed, so the macro is quick to compile.
/// A method's body is the first `{ ... }` after `fn`.
#[proc_macro_attribute]
pub fn cpu_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    output(impl_block::expand(args.into(), item.into()))
}

/// The macro's output, or the error as a `compile_error!`.
fn output(result: Result<TokenStream2>) -> TokenStream {
    result.unwrap_or_else(Error::into_compile_error).into()
}
