//! The function which `#[cpu_multiversion]` or `#[cpu_version]` is applied to.

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Error, FnArg, GenericParam, Generics, Ident, Pat, PatIdent, PatType, Receiver,
    Result, ReturnType, Safety, Signature, Token, Type, TypeParamBound, Visibility,
    parse::{Parse, ParseStream},
    parse2, token,
};

/// A function, in the parts the macros use.
///
/// The user's tokens keep their spans, so errors point at them. The exceptions are the tokens
/// which start a generated function (visibility, `const` and `unsafe`), and parameter names
/// where they're passed on. They get the macro's span, so rustc and clippy treat the generated
/// functions as macro output, not as the user's code. See [`call_site`].
pub struct Func {
    /// The function's original tokens. Output when a macro leaves the function unchanged.
    pub original: TokenStream,
    /// The attributes, sorted by which of the generated functions they go on.
    pub attrs: Attrs,
    /// Visibility, e.g. `pub(crate)`. Empty if the function is private.
    pub vis: TokenStream,
    /// `const`, if the function is a `const fn`.
    pub constness: Option<Token![const]>,
    /// `unsafe`, if the function is an `unsafe fn`.
    pub unsafety: Option<Token![unsafe]>,
    /// The function's name.
    pub name: Ident,
    /// Generic parameters, with their bounds and the `where` clause.
    pub generics: Generics,
    /// The `self` parameter, if the function is a method.
    pub receiver: Option<Receiver>,
    /// `true` if the function has a `self` parameter, or uses `Self`.
    ///
    /// A nested function can't use `self` or `Self`, so the generated functions are methods
    /// next to it instead. `Self` inside a nested `impl`, `trait`, `struct` or `enum` belongs
    /// to that item, so it doesn't count. See [`contains_self`].
    pub is_method: bool,
    /// The other parameters.
    pub params: Vec<Param>,
    /// `-> Type`, if the function has a return type.
    pub return_type: ReturnType,
    /// The function body, with its braces.
    pub body: Group,
}

impl Func {
    /// Parse a function. Error if it isn't one, or it's a kind the macros don't support.
    /// `mac` is the macro expanding it.
    pub fn parse(original: TokenStream, mac: Macro) -> Result<Func> {
        let FuncParts { attrs, vis, sig, body } = parse2(original.clone())?;

        reject(sig.asyncness, "`async` functions")?;
        reject(sig.abi, "`extern` functions")?;
        reject(sig.variadic, "variadic functions")?;

        let unsafety = match sig.safety {
            Safety::Unsafe(_) => Some(<Token![unsafe]>::default()),
            Safety::Default => None,
            Safety::Safe(safe_token) => {
                return Err(Error::new_spanned(safe_token, "`safe` functions aren't supported"));
            }
        };

        let attrs = Attrs::new(attrs, mac)?;

        let mut receiver = None;
        let mut params = Vec::new();
        for input in sig.inputs {
            match input {
                FnArg::Receiver(self_param) => receiver = Some(self_param),
                FnArg::Typed(param) => params.push(Param::parse(param)?),
            }
        }
        let is_method = receiver.is_some() || contains_self(original.clone());

        Ok(Func {
            original,
            attrs,
            vis: call_site(&vis),
            constness: sig.constness.map(|_| <Token![const]>::default()),
            unsafety,
            name: sig.ident,
            generics: sig.generics,
            receiver,
            is_method,
            params,
            return_type: sig.output,
            body,
        })
    }

    /// `true` if one of the function's `#[inline]` attributes is `#[inline(always)]`.
    pub fn is_inline_always(&self) -> bool {
        self.attrs.entry_points.iter().any(|attr| {
            attr.path().is_ident("inline")
                && attr.parse_args::<Ident>().is_ok_and(|arg| arg == "always")
        })
    }

    /// Name for a function which a macro generates for this one, e.g. `__impl`.
    ///
    /// A function's generated functions are nested inside it, so a suffix is enough.
    /// A method's are methods next to it, so their names include the method's name,
    /// e.g. `__pipeline_impl`.
    pub fn generated_name(&self, suffix: &str) -> Ident {
        if self.is_method {
            let name = &self.name;
            format_ident!("__{name}_{suffix}")
        } else {
            format_ident!("__{suffix}")
        }
    }

    /// Path which calls a generated function: `__impl`, or `Self::__pipeline_impl` for a method.
    pub fn generated_path(&self, suffix: &str) -> TokenStream {
        let name = self.generated_name(suffix);
        if self.is_method { quote!(Self::#name) } else { quote!(#name) }
    }

    /// Generic arguments for a call to a generated function, e.g. `::<T, N>`.
    /// Empty if there are no type or const parameters. Lifetimes are inferred.
    pub fn turbofish(&self) -> TokenStream {
        let args = self.generics.params.iter().filter_map(|param| match param {
            GenericParam::Type(param) => Some(&param.ident),
            GenericParam::Const(param) => Some(&param.ident),
            GenericParam::Lifetime(_) => None,
        });
        let args = args.collect::<Vec<_>>();
        if args.is_empty() { quote!() } else { quote!(::<#(#args),*>) }
    }

    /// The parameters, as declared on a function which passes them on: `&self, a: A, b: B,`.
    pub fn params(&self) -> TokenStream {
        let receiver = self.receiver.as_ref().map(|receiver| {
            let receiver = Receiver { mutability: None, ..receiver.clone() };
            receiver.into_token_stream()
        });
        let params = self.params.iter().map(|param| {
            let name = param.arg();
            let ty = &param.ty;
            quote!(#name: #ty)
        });
        list(receiver, params)
    }

    /// The parameters, as declared on the function which has the body: `&self, mut a: A, b: B,`.
    ///
    /// Only that function uses `mut`. The others just pass the values on.
    pub fn params_with_mut(&self) -> TokenStream {
        let receiver = self.receiver.as_ref().map(Receiver::to_token_stream);
        let params =
            self.params.iter().map(|Param { mutability, name, ty }| quote!(#mutability #name: #ty));
        list(receiver, params)
    }

    /// The parameters, as passed on in a call: `self, a, b,`.
    pub fn args(&self) -> TokenStream {
        let receiver = self.receiver.as_ref().map(|receiver| receiver.self_token.to_token_stream());
        let args = self.params.iter().map(Param::arg);
        list(receiver, args)
    }
}

/// The macro expanding the function. `#[track_caller]` goes on different functions for each.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Macro {
    /// `#[cpu_multiversion]`. Its entry points call `__impl`.
    Multiversion,
    /// `#[cpu_version]`. `__impl` is its entry point.
    Version,
}

/// A function's attributes, separated by which of the generated functions they go on.
///
/// `#[track_caller]` goes on all of them, so the caller's location passes through each.
///
/// `#[cfg]` and `#[cfg_attr]` never reach the macros. rustc evaluates them before it runs
/// an attribute macro, wherever they are in the list.
///
/// `#[target_feature]` is an error, because the macros add it.
pub struct Attrs {
    /// On the outer function, which replaces the original: doc comments, `#[must_use]`,
    /// and anything not listed below.
    pub outer: Vec<Attribute>,
    /// On the entry points, the functions with `#[target_feature]` which get called:
    /// `#[inline]` and `#[cold]`.
    pub entry_points: Vec<Attribute>,
    /// On `__impl`, the function with the body: lint attributes, as that's where the lints fire.
    /// A lint attribute on a method wouldn't cover its sibling `__foo_impl`.
    pub impl_fn: Vec<Attribute>,
}

impl Attrs {
    /// Separate attributes by which of the generated functions they go on.
    /// Error for attributes the macros don't support. `mac` is the macro expanding the function.
    fn new(attrs: Vec<Attribute>, mac: Macro) -> Result<Attrs> {
        let mut sorted = Attrs { outer: Vec::new(), entry_points: Vec::new(), impl_fn: Vec::new() };

        for attr in attrs {
            // Comparing a path to a `&str` allocates a `String` each time, so convert it once.
            // `None` for a path with `::` in it, e.g. `rustfmt::skip`.
            let name = attr.path().get_ident().map(Ident::to_string);
            match name.as_deref() {
                Some("inline" | "cold") => sorted.entry_points.push(attr),
                Some("expect" | "allow" | "warn" | "deny" | "forbid") => sorted.impl_fn.push(attr),
                Some("track_caller") => {
                    // `cpu_version`'s `__impl` is an entry point, and gets it from `impl_fn`
                    sorted.outer.push(attr.clone());
                    if mac == Macro::Multiversion {
                        sorted.entry_points.push(attr.clone());
                    }
                    sorted.impl_fn.push(attr);
                }
                Some("target_feature") => {
                    return Err(Error::new_spanned(attr, "`#[target_feature]` isn't supported"));
                }
                _ => sorted.outer.push(attr),
            }
        }

        Ok(sorted)
    }
}

/// A parameter, `name: Type` or `mut name: Type`.
pub struct Param {
    /// `mut`, if the parameter is `mut name: Type`.
    pub mutability: Option<Token![mut]>,
    /// The parameter's name.
    pub name: Ident,
    /// The parameter's type.
    pub ty: Type,
}

impl Param {
    /// Parse a parameter. Error unless it's `name: Type` or `mut name: Type`.
    fn parse(param: PatType) -> Result<Param> {
        let PatType { attrs, pat, ty, .. } = param;
        if let Some(attr) = attrs.first() {
            return Err(Error::new_spanned(attr, "attributes on parameters aren't supported"));
        }
        match *pat {
            Pat::Ident(PatIdent { by_ref: None, mutability, ident, subpat: None, .. }) => {
                Ok(Param { mutability, name: ident, ty: *ty })
            }
            pat => {
                Err(Error::new_spanned(pat, "parameters must be `name: Type` or `mut name: Type`"))
            }
        }
    }

    /// The name, for a function which passes the parameter on. See the docs on [`Func`].
    pub fn arg(&self) -> TokenStream {
        call_site(&self.name)
    }
}

/// A function as `syn` parses it, except the body, which stays as tokens.
///
/// The macros only pass the body on. Not parsing it speeds them up by 2x.
struct FuncParts {
    /// `#[...]` attributes, including doc comments.
    attrs: Vec<Attribute>,
    /// Visibility, e.g. `pub(crate)`.
    vis: Visibility,
    /// From `const`, `unsafe` or `fn` to the end of the `where` clause.
    sig: Signature,
    /// The body, with its braces.
    body: Group,
}

impl Parse for FuncParts {
    /// Parse a function. Error if it has no body, e.g. it's in a trait.
    fn parse(input: ParseStream) -> Result<FuncParts> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let sig = input.parse()?;
        if !input.peek(token::Brace) {
            return Err(input.error("expected a function body"));
        }
        let body = input.parse()?;
        Ok(FuncParts { attrs, vis, sig, body })
    }
}

/// `true` if `ty` is `impl Cpu`.
pub fn is_impl_cpu(ty: &Type) -> bool {
    let Type::ImplTrait(impl_trait) = ty else { return false };
    impl_trait.bounds.iter().any(|bound| match bound {
        TypeParamBound::Trait(bound) => bound.path.is_ident("Cpu"),
        _ => false,
    })
}

/// Error if `token` is present. `what` is the kind of function it's used in.
fn reject(token: Option<impl ToTokens>, what: &str) -> Result<()> {
    match token {
        Some(token) => Err(Error::new_spanned(token, format!("{what} aren't supported"))),
        None => Ok(()),
    }
}

/// `true` if `tokens` contain `Self`, at any depth, outside nested items.
///
/// Items can only be nested in a block, so skipping them starts at the first `{ ... }`.
/// Before it, `impl` is an `impl Trait` type in the signature, which must not start a skip.
fn contains_self(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => ident == "Self",
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
            block_contains_self(group.stream())
        }
        TokenTree::Group(group) => contains_self(group.stream()),
        _ => false,
    })
}

/// `true` if the tokens of a block contain `Self`, at any depth, outside nested items.
fn block_contains_self(tokens: TokenStream) -> bool {
    let mut tokens = tokens.into_iter();
    while let Some(token) = tokens.next() {
        match token {
            // Comparing an `Ident` to a `&str` allocates a `String` each time, so convert it once.
            // Nested `impl`, `trait`, `struct` and `enum` have a `Self` of their own.
            // `union` isn't a keyword, so it can be the name of a method, e.g. `a.union(b)`.
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "Self" => return true,
                "impl" | "trait" | "struct" | "enum" => skip_item(&mut tokens),
                _ => {}
            },
            TokenTree::Group(group) if block_contains_self(group.stream()) => return true,
            _ => {}
        }
    }
    false
}

/// Skip a nested item's tokens, up to its `{ ... }`, or `;` for a `struct Foo;` or `struct Foo(u8);`.
fn skip_item(tokens: &mut impl Iterator<Item = TokenTree>) {
    for token in tokens {
        match token {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => return,
            TokenTree::Punct(punct) if punct.as_char() == ';' => return,
            _ => {}
        }
    }
}

/// `tokens`, with the macro's span instead of their own.
///
/// rustc and clippy decide whether a function is macro output from the span of its first token,
/// and whether a use of a variable is from the span of its name.
fn call_site(tokens: &impl ToTokens) -> TokenStream {
    respan(tokens, Span::call_site())
}

/// `tokens`, with `span` instead of their own.
pub fn respan(tokens: &impl ToTokens, span: Span) -> TokenStream {
    let set_span = |mut token: TokenTree| {
        token.set_span(span);
        token
    };
    tokens.to_token_stream().into_iter().map(set_span).collect()
}

/// The receiver and parameters, as a list with a trailing comma: `&self, a: A, b: B,`.
fn list(receiver: Option<TokenStream>, params: impl Iterator<Item = TokenStream>) -> TokenStream {
    let items = receiver.into_iter().chain(params);
    quote!(#(#items,)*)
}
