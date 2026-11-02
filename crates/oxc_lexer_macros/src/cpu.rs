//! The CPU feature sets which the macros generate code for.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// A CPU feature set.
///
/// `oxc_lexer`'s `cpu` module has a type with the same name for each,
/// and a `DynCpu` variant which holds it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cpu {
    /// x86_64 with AVX2, BMI2 and POPCNT
    Avx2,
    /// aarch64 with Neon
    Neon,
    /// No SIMD. Exists on all platforms.
    Generic,
}

impl Cpu {
    /// The feature set called `ident`, e.g. the `Avx2` in `#[cpu_version(Avx2)]`.
    pub fn from_ident(ident: &Ident) -> Option<Cpu> {
        [Cpu::Avx2, Cpu::Neon, Cpu::Generic].into_iter().find(|cpu| ident == cpu.name())
    }

    /// Name of the type, and of the `DynCpu` variant.
    pub fn name(self) -> &'static str {
        match self {
            Cpu::Avx2 => "Avx2",
            Cpu::Neon => "Neon",
            Cpu::Generic => "Generic",
        }
    }

    /// Name in lower case, for the names of generated functions, e.g. `__avx2`.
    pub fn lower_name(self) -> &'static str {
        match self {
            Cpu::Avx2 => "avx2",
            Cpu::Neon => "neon",
            Cpu::Generic => "generic",
        }
    }

    /// Name of the type, as an identifier.
    pub fn ident(self) -> Ident {
        Ident::new(self.name(), Span::call_site())
    }

    /// The type, with its path, e.g. `crate::cpu::Avx2`.
    pub fn ty(self) -> TokenStream {
        let name = self.ident();
        quote!(crate::cpu::#name)
    }

    /// The `DynCpu` variant which holds the type, e.g. `crate::cpu::DynCpu::Avx2`.
    pub fn variant(self) -> TokenStream {
        let name = self.ident();
        quote!(crate::cpu::DynCpu::#name)
    }

    /// `#[cfg(...)]` attribute for the platform the feature set exists on.
    /// `None` for `Generic`, which exists everywhere.
    pub fn cfg_attr(self) -> Option<TokenStream> {
        let arch = match self {
            Cpu::Avx2 => "x86_64",
            Cpu::Neon => "aarch64",
            Cpu::Generic => return None,
        };
        Some(quote!(#[cfg(target_arch = #arch)]))
    }

    /// Attribute which compiles a function with the feature set's target features.
    pub fn target_feature_attr(self) -> TokenStream {
        match self {
            Cpu::Avx2 => quote!(#[target_feature(enable = "avx2,bmi2,popcnt")]),
            Cpu::Neon => quote!(#[target_feature(enable = "neon")]),
            // Generic code has no target features of its own. This stops the MIR inliner
            // inlining a generic entry point into the function which calls it.
            // See the docs on `cpu_multiversion`.
            Cpu::Generic => {
                quote!(#[cfg_attr(target_feature = "sse2", target_feature(enable = "sse2"))])
            }
        }
    }
}
