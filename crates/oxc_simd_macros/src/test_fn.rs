//! `#[test_cpus]`: a test which runs with each CPU feature set this CPU supports.
//!
//! See the docs on [`test_cpus`] for what the output looks like.
//! The example in this file is the output for this function:
//!
//! ```ignore
//! /// Check `kw_flush` sets each word's kind.
//! #[test_cpus]
//! fn test_kw_flush(cpu: impl Cpu) {
//!     // ...
//! }
//! ```
//!
//! [`test_cpus`]: crate::test_cpus

use std::mem;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, Result, ReturnType, Signature, Type, Visibility, parse2};

use crate::{cpu::Cpu, func::is_impl_cpu};

/// The type of a test's CPU feature set parameter.
enum CpuParam {
    /// `cpu: impl Cpu`: Run with each CPU feature set this CPU supports, as a specific type.
    Impl,
    /// `cpu: DynCpu`: Run with each CPU feature set this CPU supports, as a `DynCpu`.
    Dyn,
    /// `cpu: Avx2` or `cpu: Neon`: Run with that CPU feature set, if this CPU supports it.
    Simd(Cpu),
}

/// The expansion of `#[test_cpus]` on a function.
///
/// `args` is the attribute's argument, which must be empty. `item` is the function.
///
/// ```ignore
/// /// Check `kw_flush` sets each word's kind.
/// #[test]
/// fn test_kw_flush() {
///     #[::oxc_simd::cpu_multiversion]
///     fn test_kw_flush(cpu: impl Cpu) {
///         // ...
///     }
///
///     ::oxc_simd::DynCpu::for_each_supported(test_kw_flush);
/// }
/// ```
///
/// For a function `vec128_movemask(cpu: Avx2)`, the inner function gets `#[cpu_version(Avx2)]`
/// instead, and is only called with the AVX2 feature set:
///
/// ```ignore
/// ::oxc_simd::DynCpu::for_each_supported(|cpu| {
///     if let ::oxc_simd::DynCpu::Avx2(cpu) = cpu {
///         vec128_movemask(cpu);
///     }
/// });
/// ```
///
/// For a function which takes `cpu: DynCpu`, the inner function has no attribute,
/// and is passed the `DynCpu` directly.
pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !args.is_empty() {
        return Err(Error::new_spanned(args, "`#[test_cpus]` takes no arguments"));
    }

    let mut test_fn = parse2::<ItemFn>(item)?;

    // `#[test]` as well would be a duplicate, which only gets a warning, and runs the test twice
    if let Some(test_attr) = test_fn.attrs.iter().find(|attr| attr.path().is_ident("test")) {
        return Err(Error::new_spanned(test_attr, "`#[test_cpus]` adds `#[test]` itself"));
    }

    let cpu_param = cpu_param(&test_fn.sig)?;

    // A test can return a `Result`. Ignoring it would let a test which returns an `Err` pass.
    if let ReturnType::Type(..) = &test_fn.sig.output {
        let message = "a `#[test_cpus]` test can't return a value";
        return Err(Error::new_spanned(&test_fn.sig.output, message));
    }

    // Attributes and visibility go on the test. The inner function is the original without them.
    let attrs = mem::take(&mut test_fn.attrs);
    let vis = mem::replace(&mut test_fn.vis, Visibility::Inherited);
    let name = &test_fn.sig.ident;

    // Compile the inner function with the target features of the CPU feature set it's called with.
    // So the code under test is called with a specific CPU type (e.g. `Avx2`),
    // and compiled the same way as in the lexer. See the docs on `test_cpus`.
    let inner_attr = match cpu_param {
        CpuParam::Impl => Some(quote!(#[::oxc_simd::cpu_multiversion])),
        CpuParam::Dyn => None,
        CpuParam::Simd(cpu) => {
            let cpu_name = cpu.ident();
            Some(quote!(#[::oxc_simd::cpu_version(#cpu_name)]))
        }
    };

    let call = match cpu_param {
        CpuParam::Impl | CpuParam::Dyn => {
            quote!(::oxc_simd::DynCpu::for_each_supported(#name);)
        }
        // Only call it with that CPU feature set, and only if this CPU supports it
        CpuParam::Simd(cpu) => {
            let variant = cpu.variant();
            quote! {
                ::oxc_simd::DynCpu::for_each_supported(|cpu| {
                    if let #variant(cpu) = cpu {
                        #name(cpu);
                    }
                });
            }
        }
    };

    Ok(quote! {
        #(#attrs)*
        #[test]
        #vis fn #name() {
            #inner_attr
            #test_fn

            #call
        }
    })
}

/// The type of the test's parameter.
///
/// Error if it's not one `#[test_cpus]` accepts, or the test has any other parameters.
/// The parameter can have any name.
fn cpu_param(sig: &Signature) -> Result<CpuParam> {
    let message = concat!(
        "a `#[test_cpus]` test takes one parameter: ",
        "`cpu: impl Cpu`, `cpu: DynCpu`, `cpu: Avx2`, or `cpu: Neon`"
    );

    let mut params = sig.inputs.iter();
    let (Some(FnArg::Typed(param)), None) = (params.next(), params.next()) else {
        return Err(Error::new(sig.paren_token.span.join(), message));
    };

    if is_impl_cpu(&param.ty) {
        return Ok(CpuParam::Impl);
    }

    if let Some(name) = type_name(&param.ty) {
        if name == "DynCpu" {
            return Ok(CpuParam::Dyn);
        }

        if let Some(cpu @ (Cpu::Avx2 | Cpu::Neon)) = Cpu::from_ident(name) {
            return Ok(CpuParam::Simd(cpu));
        }
    }

    Err(Error::new_spanned(&param.ty, message))
}

/// The name of `ty`, if it's a type with a name and no path or generic arguments,
/// e.g. `DynCpu` or `Avx2`.
///
/// The macro only sees the type's name. If it's a different type, the test fails to compile:
/// `for_each_supported` doesn't accept the function, or `#[cpu_version]`'s check rejects it.
fn type_name(ty: &Type) -> Option<&Ident> {
    let Type::Path(path) = ty else { return None };
    if path.qself.is_some() {
        return None;
    }
    path.path.get_ident()
}
