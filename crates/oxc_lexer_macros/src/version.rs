//! `#[cpu_version(Avx2)]`: the version of some code for one CPU feature set.
//!
//! See the docs on [`cpu_version`] for what the output looks like.
//! The examples in this file are the output for this function:
//!
//! ```ignore
//! /// Do some stuff (AVX2 version).
//! #[cpu_version(Avx2)]
//! #[inline(never)]
//! fn foo<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: Avx2) -> bool {
//!     // ...
//! }
//! ```
//!
//! [`cpu_version`]: crate::cpu_version

use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use syn::{Error, GenericArgument, Ident, PathArguments, Result, Type, parse2, spanned::Spanned};

use crate::{
    cpu::Cpu,
    func::{Attrs, Func, Macro, is_impl_cpu, respan},
};

/// A parameter which proves that the CPU supports a feature set.
///
/// Both fields have the macro's span, located at the parameter's type. So the check that
/// the type is the real one is macro output to rustc and clippy, but an error in it points
/// at the type.
struct Proof {
    /// The parameter's name.
    param_name: TokenStream,
    /// The type it must be, with its path, e.g. `crate::simd::Vec128<crate::cpu::Avx2>`.
    ty: TokenStream,
}

/// The expansion of `#[cpu_version(Avx2)]` on a function.
///
/// `args` is the attribute's argument: `Avx2`, `Neon` or `Generic`. `item` is the function.
pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let cpu = parse_cpu(args)?;
    let func = Func::parse(item, Macro::Version)?;

    if cpu == Cpu::Generic {
        check_no_cpu_params(&func)?;
        return Ok(func.original);
    }

    let proof = find_proof(&func, cpu)?;

    // An `#[inline(always)]` function can't have `#[target_feature]`.
    // It's compiled as part of its callers, with their features.
    if func.is_inline_always() {
        return Ok(with_proof(&func, &proof));
    }

    // An `unsafe` function's callers need an `unsafe` block anyway, so it isn't wrapped
    if func.unsafety.is_some() {
        let target_feature_attr = cpu.target_feature_attr();
        let checked_fn = with_proof(&func, &proof);
        return Ok(quote!(#target_feature_attr #checked_fn));
    }

    Ok(wrap(&func, cpu, &proof))
}

/// The original function, with the proof check at the start of its body.
/// For a function which isn't wrapped, so there's no wrapper to hold the check.
///
/// For the example function as an `unsafe fn`:
///
/// ```ignore
/// /// Do some stuff (AVX2 version).
/// #[inline(never)]
/// unsafe fn foo<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: Avx2) -> bool {
///     let _: crate::cpu::Avx2 = cpu;
///     // ... original function body ...
/// }
/// ```
fn with_proof(func: &Func, proof: &Proof) -> TokenStream {
    // Everything but the body
    let mut head = func.original.clone().into_iter().collect::<Vec<_>>();
    head.pop();

    let Proof { param_name, ty } = proof;
    let statements = func.body.stream();
    let body = quote! {
        // `find_proof` only matched the type's name.
        // This fails to compile if it's not the real type.
        let _: #ty = #param_name;
        #statements
    };
    let mut body = Group::new(Delimiter::Brace, body);
    body.set_span(func.body.span());

    quote!(#(#head)* #body)
}

/// Wrap the function in one which is safe to call. See the docs on `cpu_version`.
///
/// ```ignore
/// /// Do some stuff (AVX2 version).
/// #[inline(always)]
/// fn foo<'a, const N: usize, T>(arr: &'a [T; N], cpu: Avx2) -> bool {
///     #[cfg_attr(target_arch = "x86_64", target_feature(enable = "avx2,bmi2,popcnt"))]
///     #[inline(never)]
///     fn __impl<'a, const N: usize, T>(mut arr: &'a [T; N], cpu: Avx2) -> bool {
///         // ... original function body ...
///     }
///
///     let _: crate::cpu::Avx2 = cpu;
///
///     unsafe { __impl::<N, T>(arr, cpu) }
/// }
/// ```
///
/// A nested function can't use `self` or `Self`, so a method's `__impl` is a method `__foo_impl`
/// next to it, and the call is `Self::__foo_impl::<N, T>(self, arr, cpu)`.
fn wrap(func: &Func, cpu: Cpu, proof: &Proof) -> TokenStream {
    let Attrs { outer: outer_attrs, entry_points: entry_attrs, impl_fn: impl_attrs } = &func.attrs;
    let Func { vis, constness, name, generics, return_type, body, .. } = func;

    let params = func.params();
    let where_clause = &generics.where_clause;
    let signature = quote! {
        #vis #constness fn #name #generics (#params) #return_type
        #where_clause
    };

    let target_feature_attr = cpu.target_feature_attr();
    let impl_name = func.generated_name("impl");
    let params_with_mut = func.params_with_mut();
    let impl_fn = quote! {
        #target_feature_attr
        #(#impl_attrs)*
        #(#entry_attrs)*
        #constness fn #impl_name #generics (#params_with_mut) #return_type
        #where_clause
        #body
    };

    let Proof { param_name: proof_param_name, ty: proof_ty } = proof;
    let impl_path = func.generated_path("impl");
    let turbofish = func.turbofish();
    let args = func.args();
    let checked_call = quote! {
        // `find_proof` only matched the type's name.
        // This fails to compile if it's not the real type.
        let _: #proof_ty = #proof_param_name;

        // SAFETY: The check above shows this parameter is a type which guarantees
        // that the CPU supports the target features which `__impl` requires
        unsafe { #impl_path #turbofish (#args) }
    };

    if func.is_method {
        quote! {
            #(#outer_attrs)*
            #[inline(always)]
            #signature { #checked_call }

            #impl_fn
        }
    } else {
        quote! {
            #(#outer_attrs)*
            #[inline(always)]
            #signature {
                #impl_fn
                #checked_call
            }
        }
    }
}

/// The feature set in the macro's arguments, e.g. `Cpu::Avx2` for `#[cpu_version(Avx2)]`.
fn parse_cpu(args: TokenStream) -> Result<Cpu> {
    let cpu = parse2::<Ident>(args.clone()).ok().and_then(|ident| Cpu::from_ident(&ident));
    cpu.ok_or_else(|| Error::new_spanned(args, "`cpu_version` takes `Avx2`, `Neon` or `Generic`"))
}

/// Find a parameter which proves that the CPU supports `cpu`'s feature set.
///
/// That's a parameter whose type is written as `Avx2`, `Vec128<Avx2>` or `Vec256<Avx2>`.
/// Only the type's name is checked here. The output checks that it's the real type.
///
/// For `v: Vec128<Avx2>`:
///
/// ```ignore
/// Proof { param_name: v, ty: crate::simd::Vec128<crate::cpu::Avx2> }
/// ```
fn find_proof(func: &Func, cpu: Cpu) -> Result<Proof> {
    let proof = func.params.iter().find_map(|param| {
        let ty = match type_name(&param.ty) {
            Some((name, None)) if name == cpu.name() => cpu.ty(),
            Some((name, Some(arg)))
                if (name == "Vec128" || name == "Vec256") && arg == cpu.name() =>
            {
                let cpu_ty = cpu.ty();
                quote!(crate::simd::#name<#cpu_ty>)
            }
            _ => return None,
        };
        let span = Span::call_site().located_at(param.ty.span());
        Some(Proof { param_name: respan(&param.name, span), ty: respan(&ty, span) })
    });

    proof.ok_or_else(|| {
        let cpu = cpu.name();
        let message = format!(
            "`cpu_version({cpu})` functions must have an `{cpu}`, \
            `Vec128<{cpu}>`, or `Vec256<{cpu}>` parameter"
        );
        Error::new_spanned(&func.name, message)
    })
}

/// Error if a parameter has a CPU type, or is a SIMD vector.
///
/// A generic version only runs where the CPU feature set is `BASELINE`'s,
/// so it uses `BASELINE` where it needs a CPU type.
fn check_no_cpu_params(func: &Func) -> Result<()> {
    const CPU_TYPES: [&str; 6] = ["Avx2", "Neon", "Generic", "DynCpu", "Vec128", "Vec256"];

    let has_cpu_type = |ty: &Type| {
        is_impl_cpu(ty)
            || type_name(ty).is_some_and(|(name, _)| CPU_TYPES.iter().any(|cpu_ty| name == cpu_ty))
    };

    match func.params.iter().find(|param| has_cpu_type(&param.ty)) {
        Some(param) => Err(Error::new_spanned(
            &param.ty,
            "`cpu_version(Generic)` functions can't have a parameter with a CPU or SIMD Vec type",
        )),
        None => Ok(()),
    }
}

/// The name of a type written as `Name` or `Name<Arg>`, with any path prefix, and `Arg`'s name.
///
/// - `crate::cpu::Avx2` -> `Some((Avx2, None))`
/// - `Vec128<Avx2>` -> `Some((Vec128, Some(Avx2)))`
/// - `&Avx2`, `Vec128<Avx2, X>`, `impl Cpu` -> `None`
fn type_name(ty: &Type) -> Option<(&Ident, Option<&Ident>)> {
    let Type::Path(ty) = ty else { return None };
    let segment = ty.path.segments.last()?;
    let arg = match &segment.arguments {
        PathArguments::None => None,
        PathArguments::AngleBracketed(args) if args.args.len() == 1 => match &args.args[0] {
            GenericArgument::Type(Type::Path(arg)) => Some(&arg.path.segments.last()?.ident),
            _ => return None,
        },
        _ => return None,
    };
    Some((&segment.ident, arg))
}
