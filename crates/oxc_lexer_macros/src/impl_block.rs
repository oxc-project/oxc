//! `#[cpu_impl(Avx2)]`: an `impl` block whose methods only work on one CPU feature set's platform.
//!
//! See the docs on [`cpu_impl`] for what the output looks like.
//! The examples in this file are the output for this `impl` block:
//!
//! ```ignore
//! #[cpu_impl(Avx2)]
//! impl Vec256<Avx2> {
//!     /// Gather the top bit of each byte of this [`Vec256`] into a 32-bit mask.
//!     #[inline(always)]
//!     pub fn movemask(self) -> u32 {
//!         // SAFETY: `Vec256` type guarantees platform supports AVX2
//!         unsafe { _mm256_movemask_epi8(self.inner).cast_unsigned() }
//!     }
//! }
//! ```
//!
//! [`cpu_impl`]: crate::cpu_impl

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::quote;
use syn::{Error, Ident, Result, parse2};

use crate::cpu::Cpu;

/// The expansion of `#[cpu_impl(Avx2)]` on an `impl` block.
///
/// `args` is the attribute's argument: `Avx2` or `Neon`. `item` is the `impl` block.
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
pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let arch = parse_arch(args)?;

    let mut head = item.into_iter().collect::<Vec<_>>();
    let body = match head.pop() {
        Some(TokenTree::Group(body)) if body.delimiter() == Delimiter::Brace && is_impl(&head) => {
            body
        }
        _ => return Err(Error::new(Span::call_site(), "`cpu_impl` must be on an `impl` block")),
    };

    let mut items = Group::new(Delimiter::Brace, items(body.stream(), arch));
    items.set_span(body.span());

    Ok(quote! {
        #[cfg_attr(
            not(target_arch = #arch),
            allow(dead_code, unused_variables, clippy::unused_self, clippy::needless_pass_by_value)
        )]
        #(#head)* #items
    })
}

/// The `target_arch` of the feature set in the macro's arguments,
/// e.g. `"x86_64"` for `#[cpu_impl(Avx2)]`.
fn parse_arch(args: TokenStream) -> Result<&'static str> {
    let cpu = parse2::<Ident>(args.clone()).ok().and_then(|ident| Cpu::from_ident(&ident));
    cpu.and_then(Cpu::target_arch)
        .ok_or_else(|| Error::new_spanned(args, "`cpu_impl` takes `Avx2` or `Neon`"))
}

/// Check if `head`, the tokens before an item's `{ ... }`, start an `impl` block.
/// That's the `impl` keyword, after any attributes.
fn is_impl(head: &[TokenTree]) -> bool {
    let mut tokens = head.iter();
    loop {
        match tokens.next() {
            // An attribute is `#` followed by `[...]`
            Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => {
                tokens.next();
            }
            Some(TokenTree::Ident(ident)) => return ident == "impl",
            _ => return false,
        }
    }
}

/// The items of the `impl` block, with each method's body replaced by [`arch_only_body`].
///
/// The tokens are scanned, not parsed. A method's body is the first `{ ... }` after `fn`.
/// A `;` ends an item which has no body, e.g. `type Inner = __m256i;`.
/// All other tokens are output unchanged.
fn items(tokens: TokenStream, arch: &str) -> TokenStream {
    let mut in_fn = false;
    tokens
        .into_iter()
        .map(|token| {
            match &token {
                TokenTree::Ident(ident) if ident == "fn" => {
                    in_fn = true;
                }
                TokenTree::Punct(punct) if punct.as_char() == ';' => {
                    in_fn = false;
                }
                TokenTree::Group(body) if in_fn && body.delimiter() == Delimiter::Brace => {
                    in_fn = false;
                    return TokenTree::Group(arch_only_body(body, arch));
                }
                _ => {}
            }
            token
        })
        .collect()
}

/// A method body which only compiles `body` on `arch`, and is `unreachable!()` elsewhere.
///
/// ```ignore
/// {
///     cfg_select! {
///         target_arch = "x86_64" => {
///             // SAFETY: `Vec256` type guarantees platform supports AVX2
///             unsafe { _mm256_movemask_epi8(self.inner).cast_unsigned() }
///         }
///         _ => unreachable!(),
///     }
/// }
/// ```
fn arch_only_body(body: &Group, arch: &str) -> Group {
    let tokens = quote! {
        cfg_select! {
            target_arch = #arch => #body
            _ => unreachable!(),
        }
    };
    let mut new_body = Group::new(Delimiter::Brace, tokens);
    new_body.set_span(body.span());
    new_body
}
