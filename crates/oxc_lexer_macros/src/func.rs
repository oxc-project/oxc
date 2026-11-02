//! The function which `#[cpu_multiversion]` or `#[cpu_version]` is applied to.

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Block, Error, FnArg, GenericParam, Generics, Ident, ItemFn, Pat, PatIdent, PatType,
    Receiver, Result, ReturnType, Safety, Token, Type, TypeParamBound, parse2,
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
    /// All attributes except `#[inline]` ones. They go on the outer function.
    pub outer_attrs: Vec<Attribute>,
    /// `#[inline]` attributes. They go on the functions which have `#[target_feature]`.
    pub inline_attrs: Vec<Attribute>,
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
    /// The other parameters.
    pub params: Vec<Param>,
    /// `-> Type`, if the function has a return type.
    pub return_type: ReturnType,
    /// The function body, with its braces.
    pub body: Block,
}

impl Func {
    /// Parse a function. Error if it isn't one, or it's a kind the macros don't support.
    pub fn parse(original: TokenStream) -> Result<Func> {
        let ItemFn { attrs, vis, modifiers, sig, block } = parse2(original.clone())?;

        reject(modifiers.defaultness, "`default` functions")?;
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

        let (inline_attrs, outer_attrs) =
            attrs.into_iter().partition(|attr| attr.path().is_ident("inline"));

        let mut receiver = None;
        let mut params = Vec::new();
        for input in sig.inputs {
            match input {
                FnArg::Receiver(self_param) => receiver = Some(self_param),
                FnArg::Typed(param) => params.push(Param::parse(param)?),
            }
        }

        Ok(Func {
            original,
            outer_attrs,
            inline_attrs,
            vis: call_site(&vis),
            constness: sig.constness.map(|_| <Token![const]>::default()),
            unsafety,
            name: sig.ident,
            generics: sig.generics,
            receiver,
            params,
            return_type: sig.output,
            body: *block,
        })
    }

    /// `true` if the function has a `self` parameter.
    pub fn is_method(&self) -> bool {
        self.receiver.is_some()
    }

    /// `true` if one of the function's `#[inline]` attributes is `#[inline(always)]`.
    pub fn is_inline_always(&self) -> bool {
        self.inline_attrs
            .iter()
            .any(|attr| attr.parse_args::<Ident>().is_ok_and(|arg| arg == "always"))
    }

    /// Name for a function which a macro generates for this one, e.g. `__impl`.
    ///
    /// A function's generated functions are nested inside it, so a suffix is enough.
    /// A method's are methods next to it, so their names include the method's name,
    /// e.g. `__pipeline_impl`.
    pub fn generated_name(&self, suffix: &str) -> Ident {
        if self.is_method() {
            let name = &self.name;
            format_ident!("__{name}_{suffix}")
        } else {
            format_ident!("__{suffix}")
        }
    }

    /// Path which calls a generated function: `__impl`, or `Self::__pipeline_impl` for a method.
    pub fn generated_path(&self, suffix: &str) -> TokenStream {
        let name = self.generated_name(suffix);
        if self.is_method() { quote!(Self::#name) } else { quote!(#name) }
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
