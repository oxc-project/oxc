//! `#[cpu_multiversion]`: a function with a separate entry point for each CPU feature set.
//!
//! See the docs on [`cpu_multiversion`] for what the output looks like.
//! The examples in this file are the output for this function:
//!
//! ```ignore
//! /// Do some stuff.
//! #[cpu_multiversion]
//! #[inline(never)]
//! unsafe fn foo<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: impl Cpu) -> bool {
//!     // ...
//! }
//! ```
//!
//! [`cpu_multiversion`]: crate::cpu_multiversion

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Result};

use crate::{
    cpu::Cpu,
    func::{Func, Macro, Param, is_impl_cpu},
};

/// The function, with its `cpu: impl Cpu` parameter split off.
/// Each generated function declares that parameter with a different type.
struct Multiversion<'a> {
    /// The function, without its `cpu: impl Cpu` parameter.
    func: &'a Func,
    /// The last parameter, `cpu: impl Cpu`. Removed from `func.params`.
    cpu_param: &'a Param,
}

/// The expansion of `#[cpu_multiversion]` on a function.
///
/// `args` is the attribute's arguments, which must be empty. `item` is the function.
pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new_spanned(args, "`cpu_multiversion` takes no arguments"));
    }

    let mut func = Func::parse(item, Macro::Multiversion)?;
    let Some(cpu_param) = func.params.pop_if(|param| is_impl_cpu(&param.ty)) else {
        return Err(Error::new_spanned(&func.name, "the last parameter must be `cpu: impl Cpu`"));
    };

    // An `#[inline(always)]` function is compiled as part of its callers' entry points,
    // so it needs none of its own
    if func.is_inline_always() {
        return Ok(func.original);
    }

    let versions = Multiversion { func: &func, cpu_param: &cpu_param };
    Ok(if func.is_method { versions.expand_method() } else { versions.expand_fn() })
}

impl Multiversion<'_> {
    /// A function. Its entry points are functions inside it.
    ///
    /// ```ignore
    /// /// Do some stuff.
    /// #[inline(always)]
    /// unsafe fn foo<'a, const N: usize, T>(arr: &'a [T; N], cpu: impl Cpu) -> bool {
    ///     #[inline(always)]
    ///     unsafe fn __impl<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: impl Cpu) -> bool {
    ///         // ... original function body ...
    ///     }
    ///
    ///     #[inline(never)]
    ///     #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
    ///     unsafe fn __generic<'a, const N: usize, T>(arr: &'a [T; N], cpu: crate::cpu::Generic) -> bool {
    ///         __impl::<N, T>(arr, cpu)
    ///     }
    ///
    ///     cfg_select! {
    ///         target_arch = "x86_64" => {
    ///             #[inline(never)]
    ///             #[target_feature(enable = "avx2,bmi2,popcnt")]
    ///             unsafe fn __avx2<'a, const N: usize, T>(arr: &'a [T; N], cpu: crate::cpu::Avx2) -> bool {
    ///                 __impl::<N, T>(arr, cpu)
    ///             }
    ///
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Avx2(cpu) => unsafe { __avx2::<N, T>(arr, cpu) },
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe { __generic::<N, T>(arr, cpu) },
    ///             }
    ///         }
    ///         target_arch = "aarch64" => {
    ///             #[inline(never)]
    ///             #[target_feature(enable = "neon")]
    ///             unsafe fn __neon<'a, const N: usize, T>(arr: &'a [T; N], cpu: crate::cpu::Neon) -> bool {
    ///                 __impl::<N, T>(arr, cpu)
    ///             }
    ///
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Neon(cpu) => unsafe { __neon::<N, T>(arr, cpu) },
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe { __generic::<N, T>(arr, cpu) },
    ///             }
    ///         }
    ///         _ => {
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe { __generic::<N, T>(arr, cpu) },
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn expand_fn(&self) -> TokenStream {
        let outer_attrs = &*self.func.attrs.outer;
        let signature = self.signature();
        let impl_fn = self.impl_fn();
        let generic_fn = self.entry_point(Cpu::Generic);
        let simd_fn_and_cpu_match = cfg_select(|simd| {
            let simd_fn = simd.map(|cpu| self.entry_point(cpu));
            let cpu_match = self.cpu_match(simd);
            quote!(#simd_fn #cpu_match)
        });

        quote! {
            #(#outer_attrs)*
            #[inline(always)]
            #signature {
                #impl_fn
                #generic_fn
                #simd_fn_and_cpu_match
            }
        }
    }

    /// A method, or an associated function which uses `Self`.
    /// A nested function can't use `self` or `Self`, so its entry points are methods next to it.
    ///
    /// For the example function with a `&self` parameter:
    ///
    /// ```ignore
    /// /// Do some stuff.
    /// #[inline(always)]
    /// unsafe fn foo<'a, const N: usize, T>(&self, arr: &'a [T; N], cpu: impl Cpu) -> bool {
    ///     cfg_select! {
    ///         target_arch = "x86_64" => {
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Avx2(cpu) => unsafe {
    ///                     Self::__foo_avx2::<N, T>(self, arr, cpu)
    ///                 },
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe {
    ///                     Self::__foo_generic::<N, T>(self, arr, cpu)
    ///                 },
    ///             }
    ///         }
    ///         target_arch = "aarch64" => {
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Neon(cpu) => unsafe {
    ///                     Self::__foo_neon::<N, T>(self, arr, cpu)
    ///                 },
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe {
    ///                     Self::__foo_generic::<N, T>(self, arr, cpu)
    ///                 },
    ///             }
    ///         }
    ///         _ => {
    ///             match crate::cpu::to_dyn(cpu) {
    ///                 crate::cpu::DynCpu::Generic(cpu) => unsafe {
    ///                     Self::__foo_generic::<N, T>(self, arr, cpu)
    ///                 },
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// #[inline(always)]
    /// unsafe fn __foo_impl<'a, const N: usize, T>(&self, mut arr: &'a [T; N], cpu: impl Cpu) -> bool {
    ///     // ... original function body ...
    /// }
    ///
    /// #[inline(never)]
    /// #[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))]
    /// unsafe fn __foo_generic<'a, const N: usize, T>(&self, arr: &'a [T; N], cpu: crate::cpu::Generic) -> bool {
    ///     Self::__foo_impl::<N, T>(self, arr, cpu)
    /// }
    ///
    /// cfg_select! {
    ///     target_arch = "x86_64" => {
    ///         #[inline(never)]
    ///         #[target_feature(enable = "avx2,bmi2,popcnt")]
    ///         unsafe fn __foo_avx2<'a, const N: usize, T>(&self, arr: &'a [T; N], cpu: crate::cpu::Avx2) -> bool {
    ///             Self::__foo_impl::<N, T>(self, arr, cpu)
    ///         }
    ///     }
    ///     target_arch = "aarch64" => {
    ///         #[inline(never)]
    ///         #[target_feature(enable = "neon")]
    ///         unsafe fn __foo_neon<'a, const N: usize, T>(&self, arr: &'a [T; N], cpu: crate::cpu::Neon) -> bool {
    ///             Self::__foo_impl::<N, T>(self, arr, cpu)
    ///         }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    fn expand_method(&self) -> TokenStream {
        let outer_attrs = &*self.func.attrs.outer;
        let signature = self.signature();
        let cpu_match = cfg_select(|simd| self.cpu_match(simd));
        let impl_fn = self.impl_fn();
        let generic_fn = self.entry_point(Cpu::Generic);
        let simd_fn = cfg_select(|simd| match simd {
            Some(cpu) => self.entry_point(cpu),
            None => quote!(),
        });

        quote! {
            #(#outer_attrs)*
            #[inline(always)]
            #signature { #cpu_match }
            #impl_fn
            #generic_fn
            #simd_fn
        }
    }

    /// The outer function's signature. It replaces the original function,
    /// and calls the entry point for the CPU's feature set.
    ///
    /// ```ignore
    /// unsafe fn foo<'a, const N: usize, T>(arr: &'a [T; N], cpu: impl Cpu) -> bool
    /// ```
    fn signature(&self) -> TokenStream {
        let Func { vis, constness, unsafety, name, generics, return_type, .. } = self.func;
        let params = self.func.params();
        let cpu_name = self.cpu_param.arg();
        let cpu_ty = &self.cpu_param.ty;
        let where_clause = &generics.where_clause;

        quote! {
            #vis #constness #unsafety fn #name #generics (#params #cpu_name: #cpu_ty) #return_type
            #where_clause
        }
    }

    /// `__impl`: the original function, which is compiled as part of each entry point.
    /// Lint attributes go on it, as that's where the lints fire.
    ///
    /// ```ignore
    /// #[inline(always)]
    /// unsafe fn __impl<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: impl Cpu) -> bool {
    ///     // ... original function body ...
    /// }
    /// ```
    fn impl_fn(&self) -> TokenStream {
        let impl_attrs = &*self.func.attrs.impl_fn;
        let Func { constness, unsafety, generics, return_type, body, .. } = self.func;
        let name = self.func.generated_name("impl");
        let params = self.func.params_with_mut();
        let Param { name: cpu_name, ty: cpu_ty, .. } = self.cpu_param;
        let where_clause = &generics.where_clause;

        quote! {
            #(#impl_attrs)*
            #[inline(always)]
            #constness #unsafety fn #name #generics (#params #cpu_name: #cpu_ty) #return_type
            #where_clause
            #body
        }
    }

    /// An entry point: calls `__impl`, compiled with one feature set's target features.
    /// `#[inline]` and `#[cold]` go on it, as it's the function which gets called.
    ///
    /// ```ignore
    /// #[inline(never)]
    /// #[target_feature(enable = "avx2,bmi2,popcnt")]
    /// unsafe fn __avx2<'a, const N: usize, T>(arr: &'a [T; N], cpu: crate::cpu::Avx2) -> bool {
    ///     __impl::<N, T>(arr, cpu)
    /// }
    /// ```
    fn entry_point(&self, cpu: Cpu) -> TokenStream {
        let entry_attrs = &*self.func.attrs.entry_points;
        let Func { constness, unsafety, generics, return_type, .. } = self.func;
        let target_feature_attr = cpu.target_feature_attr();
        let name = self.func.generated_name(cpu.lower_name());
        let params = self.func.params();
        let cpu_name = self.cpu_param.arg();
        let cpu_ty = cpu.ty();
        let where_clause = &generics.where_clause;
        let call = self.call("impl");

        quote! {
            #(#entry_attrs)*
            #target_feature_attr
            #constness #unsafety fn #name #generics (#params #cpu_name: #cpu_ty) #return_type
            #where_clause
            {
                #call
            }
        }
    }

    /// `match` which calls the entry point for the CPU's feature set.
    /// `simd` is the SIMD feature set which exists on the platform, if there is one.
    ///
    /// ```ignore
    /// match crate::cpu::to_dyn(cpu) {
    ///     crate::cpu::DynCpu::Avx2(cpu) => unsafe { __avx2::<N, T>(arr, cpu) },
    ///     crate::cpu::DynCpu::Generic(cpu) => unsafe { __generic::<N, T>(arr, cpu) },
    /// }
    /// ```
    fn cpu_match(&self, simd: Option<Cpu>) -> TokenStream {
        let cpu_name = self.cpu_param.arg();
        let arms = simd.into_iter().chain([Cpu::Generic]).map(|cpu| {
            let variant = cpu.variant();
            let call = self.call(cpu.lower_name());

            // SAFETY: `Avx2` and `Neon` types guarantee the CPU supports the features their
            // entry points use. `__generic` only has `#[target_feature(enable = "sse2")]`
            // when SSE2 is enabled for the whole crate, so the CPU supports it.
            quote! {
                #variant(#cpu_name) => unsafe { #call },
            }
        });

        quote! {
            match crate::cpu::to_dyn(#cpu_name) {
                #(#arms)*
            }
        }
    }

    /// A call to `__impl` or an entry point, which passes the parameters on.
    /// For a method, it's `Self::__foo_impl::<N, T>(self, arr, cpu)`.
    ///
    /// ```ignore
    /// __impl::<N, T>(arr, cpu)
    /// ```
    fn call(&self, suffix: &str) -> TokenStream {
        let path = self.func.generated_path(suffix);
        let turbofish = self.func.turbofish();
        let args = self.func.args();
        let cpu_name = self.cpu_param.arg();

        quote! {
            #path #turbofish (#args #cpu_name)
        }
    }
}

/// `cfg_select!` with a branch for each platform which has a SIMD feature set, and a fallback.
/// `branch` makes the contents of each, given the platform's SIMD feature set, if it has one.
///
/// ```ignore
/// cfg_select! {
///     target_arch = "x86_64" => { ... }
///     target_arch = "aarch64" => { ... }
///     _ => { ... }
/// }
/// ```
///
/// Code for other platforms is removed with `cfg_select!`, not `#[cfg]` attributes.
/// That code contains copies of the function's parameters. Rust Analyzer greys out the
/// original code of anything in a node disabled by `#[cfg]`, so `#[cfg]` would grey out
/// the function's signature. Branches of `cfg_select!` are just dropped.
fn cfg_select(branch: impl Fn(Option<Cpu>) -> TokenStream) -> TokenStream {
    let x86_64 = branch(Some(Cpu::Avx2));
    let aarch64 = branch(Some(Cpu::Neon));
    let other = branch(None);

    quote! {
        cfg_select! {
            target_arch = "x86_64" => { #x86_64 }
            target_arch = "aarch64" => { #aarch64 }
            _ => { #other }
        }
    }
}
