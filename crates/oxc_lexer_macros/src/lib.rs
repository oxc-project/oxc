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
//!
//! `oxc_lexer`'s `cpu_select!` macro chooses between versions.
//!
//! The macros are only for `oxc_lexer`.
//! Their output refers to its `cpu` and `simd` modules by path.

#![warn(clippy::missing_docs_in_private_items)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Error, Result};

mod cpu;
mod func;
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
/// - `__avx2`: Takes `cpu: Avx2`. Compiled with AVX2 enabled. Only exists on x86_64.
/// - `__neon`: Takes `cpu: Neon`. Compiled with Neon enabled. Only exists on aarch64.
/// - `__generic`: Takes `cpu: Generic`.
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
/// Code for other platforms is removed with `cfg_select!`, not `#[cfg]` attributes.
/// That code contains copies of the function's parameters.
/// Rust Analyzer greys out the original code of anything in a node disabled by `#[cfg]`,
/// so `#[cfg]` (or `cpu_select!`, which uses it) would grey out the function's signature.
/// Branches of `cfg_select!` are just dropped, so nothing is greyed out.
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
/// protected by their own target features in the same way.
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
///     // Stops the MIR inliner inlining `__generic` into this function. See the docs above.
///     #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
///     unsafe fn __generic(src: &[u8], n: usize, cpu: Generic) -> usize {
///         __impl(src, n, cpu)
///     }
///
///     cfg_select! {
///         target_arch = "x86_64" => {
///             #[inline(never)]
///             #[target_feature(enable = "avx2,bmi2,popcnt")]
///             unsafe fn __avx2(src: &[u8], n: usize, cpu: Avx2) -> usize {
///                 __impl(src, n, cpu)
///             }
///
///             match to_dyn(cpu) {
///                 // SAFETY: `Avx2` type guarantees the CPU supports the features `__avx2` uses
///                 DynCpu::Avx2(cpu) => unsafe { __avx2(src, n, cpu) },
///                 // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
///                 // when SSE2 is enabled for the whole crate, so the CPU supports it
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///         target_arch = "aarch64" => {
///             #[inline(never)]
///             #[target_feature(enable = "neon")]
///             unsafe fn __neon(src: &[u8], n: usize, cpu: Neon) -> usize {
///                 __impl(src, n, cpu)
///             }
///
///             match to_dyn(cpu) {
///                 // SAFETY: `Neon` type guarantees the CPU supports the features `__neon` uses
///                 DynCpu::Neon(cpu) => unsafe { __neon(src, n, cpu) },
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///         _ => {
///             match to_dyn(cpu) {
///                 // SAFETY: `__generic` only has `#[target_feature(enable = "sse2")]`
///                 // when SSE2 is enabled for the whole crate, so the CPU supports it
///                 DynCpu::Generic(cpu) => unsafe { __generic(src, n, cpu) },
///             }
///         }
///     }
/// }
/// ```
///
/// For a method, the `match` is the whole body of the outer method, in a `cfg_select!` of its
/// own, and the other functions are methods next to it.
#[proc_macro_attribute]
pub fn cpu_multiversion(args: TokenStream, item: TokenStream) -> TokenStream {
    output(multiversion::expand(args.into(), item.into()))
}

/// Marks a function as the version of some code for one CPU feature set.
///
/// `cpu_select!` picks between the versions.
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
///   The function gets `#[cfg(target_arch = "x86_64")]`. What else it gets depends on its kind:
///   - `#[inline(always)]`: Nothing, because it can't be combined with `#[target_feature]`.
///     It's compiled as part of its callers, with their features.
///   - `unsafe fn`: It gets `#[target_feature(enable = "avx2,bmi2,popcnt")]`.
///   - Any other function: It's wrapped in a safe function. See below.
///
/// - `Neon`:
///   The same, with `Neon`, `Vec128<Neon>` or `Vec256<Neon>`,
///   `#[cfg(target_arch = "aarch64")]` and `#[target_feature(enable = "neon")]`.
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
/// #[cfg(target_arch = "x86_64")]
/// #[inline(always)]
/// fn foo<const N: usize>(arr: [u64; N], cpu: Avx2) -> bool {
///     #[inline(never)]
///     #[target_feature(enable = "avx2,bmi2,popcnt")]
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

/// The macro's output, or the error as a `compile_error!`.
fn output(result: Result<TokenStream2>) -> TokenStream {
    result.unwrap_or_else(Error::into_compile_error).into()
}
