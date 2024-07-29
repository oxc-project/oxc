use std::stringify;
use std::{borrow::Cow, collections::HashMap};

use convert_case::{Case, Casing};
use itertools::Itertools;
use lazy_static::lazy_static;
use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote, punctuated::Punctuated, AngleBracketedGenericArguments, Attribute, Expr, Field,
    FnArg, GenericArgument, GenericParam, Ident, ImplItemFn, Lit, Meta, MetaNameValue, PatLit,
    PatType, PathArguments, PredicateType, Token, Type, TypePath, Variant, WhereClause,
};

use crate::{
    generators::generated_header,
    schema::{Inherit, REnum, RStruct, RType},
    util::{TypeAnalyzeResult, TypeExt, TypeIdentResult, TypeWrapper},
    CodegenCtx, Generator, GeneratorOutput, TypeRef,
};

pub struct AstBuilderGenerator;

impl Generator for AstBuilderGenerator {
    fn name(&self) -> &'static str {
        "AstBuilderGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let fns = ctx
            .ty_table
            .iter()
            .filter(|it| it.borrow().visitable())
            .map(|it| (it, ctx))
            .filter_map(|(it, ctx)| generate_builder_fn(it, ctx))
            .collect_vec();

        let header = generated_header!();

        GeneratorOutput::Stream((
            "ast_builder",
            quote! {
                #header
                insert!("#![allow(clippy::default_trait_access, clippy::too_many_arguments, clippy::fn_params_excessive_bools)]");
                endl!();

                use oxc_allocator::{Allocator, Box, IntoIn, Vec};
                use oxc_span::{Atom, SourceType, Span};
                use oxc_syntax::{
                    number::{BigintBase, NumberBase},
                    operator::{
                        AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
                    },
                };

                endl!();

                #[allow(clippy::wildcard_imports)]
                use crate::ast::*;

                endl!();

                /// AST builder for creating AST nodes
                #[derive(Clone, Copy)]
                pub struct AstBuilder<'a> {
                    pub allocator: &'a Allocator,
                }

                endl!();

                impl<'a> AstBuilder<'a> {
                    #(#fns)*
                }
            },
        ))
    }
}

fn fn_ident_name<S: AsRef<str>>(ident: S) -> String {
    ident.as_ref().to_case(Case::Snake)
}

fn enum_builder_name(enum_name: String, var_name: String) -> Ident {
    // replace `xxx_yyy_xxx` with `xxx_yyy`.
    let var_name = if var_name.ends_with(enum_name.as_str()) {
        var_name.chars().take(var_name.len() - enum_name.len()).collect::<String>()
    // replace `ts_xxx_ts_yyy` with `ts_xxx_yyy`
    } else if enum_name.starts_with("TS") && var_name.starts_with("TS") {
        var_name.chars().skip(2).collect::<String>()
    } else {
        var_name
    };

    format_ident!("{}_{}", fn_ident_name(enum_name), fn_ident_name(var_name))
}

fn struct_builder_name(struct_: &RStruct) -> Ident {
    static RUST_KEYWORDS: [&str; 1] = ["super"];
    let mut ident = fn_ident_name(struct_.ident().to_string());
    if RUST_KEYWORDS.contains(&ident.as_str()) {
        ident.push('_');
    }
    format_ident!("{ident}")
}

fn generate_builder_fn(ty: &TypeRef, ctx: &CodegenCtx) -> Option<TokenStream> {
    match &*ty.borrow() {
        RType::Enum(it) => Some(generate_enum_builder_fn(it, ctx)),
        RType::Struct(it) => Some(generate_struct_builder_fn(it, ctx)),
        _ => None,
    }
}

fn generate_enum_builder_fn(ty: &REnum, ctx: &CodegenCtx) -> TokenStream {
    let variants_fns = ty
        .item
        .variants
        .iter()
        .filter(|it| !it.attrs.iter().any(|it| it.path().is_ident("inherit")))
        .map(|it| generate_enum_variant_builder_fn(ty, it, ctx));

    let inherits_fns = ty.meta.inherits.iter().map(|it| {
        let Inherit::Linked { super_, variants } = it else { panic!("Unresolved inheritance!") };
        generate_enum_inherit_builder_fn(ty, super_, variants, ctx)
    });

    variants_fns.chain(inherits_fns).collect()
}

fn generate_enum_inherit_builder_fn(
    enum_: &REnum,
    super_type: &Type,
    _: &Punctuated<Variant, Token![,]>,
    _: &CodegenCtx,
) -> TokenStream {
    let enum_ident = enum_.ident();
    let enum_as_type = enum_.as_type();
    let fn_name =
        enum_builder_name(enum_ident.to_string(), super_type.get_ident().inner_ident().to_string());

    quote! {
        endl!();
        #[inline]
        pub fn #fn_name(self, inner: #super_type) -> #enum_as_type {
            #enum_ident::from(inner)
        }
    }
}

/// Create a builder function for an enum variant (e.g. for `Expression::Binary`)
fn generate_enum_variant_builder_fn(
    enum_: &REnum,
    variant: &Variant,
    ctx: &CodegenCtx,
) -> TokenStream {
    assert_eq!(variant.fields.len(), 1);
    let enum_ident = enum_.ident();
    let enum_type = &enum_.as_type();
    let var_ident = &variant.ident;
    let var_type = &variant.fields.iter().next().expect("we have already asserted this one!").ty;
    let fn_name =
        enum_builder_name(enum_ident.to_string(), var_type.get_ident().inner_ident().to_string());
    let ty = ctx.find(&var_type.get_ident().inner_ident().to_string()).expect("type not found!");
    #[allow(clippy::single_match_else)]
    let (params, inner_builder) = match &*ty.borrow() {
        // RType::Enum(it) => get_enum_params(it, ctx),
        RType::Struct(it) => (get_struct_params(it, ctx), struct_builder_name(it)),
        _ => panic!(),
    };

    let params = params.into_iter().filter(Param::not_default).collect_vec();
    let fields = params.iter().map(|it| it.ident.clone());
    let (generic_params, where_clause) = get_generic_params(&params);

    let inner_ident = var_type.get_ident();

    let mut inner = quote!(self.#inner_builder(#(#fields),*));
    let mut does_alloc = false;
    if matches!(inner_ident, TypeIdentResult::Box(_)) {
        inner = quote!(self.alloc(#inner));
        does_alloc = true;
    }

    let from_variant_builder = generate_enum_from_variant_builder_fn(enum_, variant, ctx);
    let article = article_for(enum_ident.to_string());
    let mut docs = DocComment::new(format!(" Build {article} [`{enum_ident}::{var_ident}`]"))
        .with_params(&params);
    if does_alloc {
        let inner_name = inner_ident.inner_ident().to_string();
        let inner_article = article_for(&inner_name);
        docs = docs.with_description(format!(
            "This node contains {inner_article} [`{inner_name}`] that will be stored in the memory arena."
        ));
    }

    quote! {
        endl!();
        #docs
        #[inline]
        pub fn #fn_name #generic_params (self, #(#params),*) -> #enum_type #where_clause {
            #enum_ident::#var_ident(#inner)
        }

        #from_variant_builder
    }
}

/// Generate a conversion function that takes some struct and creates an enum
/// variant containing that struct using the `IntoIn` trait.
fn generate_enum_from_variant_builder_fn(
    enum_: &REnum,
    variant: &Variant,
    _: &CodegenCtx,
) -> TokenStream {
    assert_eq!(variant.fields.len(), 1);
    let enum_ident = enum_.ident();
    let enum_type = &enum_.as_type();
    let var_ident = &variant.ident;
    let var_type = &variant.fields.iter().next().expect("we have already asserted this one!").ty;
    let struct_ident = var_type.get_ident().inner_ident().to_string();
    let fn_name = enum_builder_name(enum_ident.to_string(), format!("From{struct_ident}"));

    let from_article = article_for(struct_ident);
    let to_article = article_for(enum_ident.to_string());

    let docs = DocComment::new(format!(
        " Convert {from_article} [`{}`] into {to_article} [`{enum_ident}::{var_ident}`]",
        var_type.get_ident().inner_ident()
    ));
    quote! {
        endl!();
        #docs
        #[inline]
        pub fn #fn_name<T>(self, inner: T) -> #enum_type where T: IntoIn<'a, #var_type> {
            #enum_ident::#var_ident(inner.into_in(self.allocator))
        }
    }
}

fn default_init_field((ident, typ): &(&Ident, &Type)) -> bool {
    macro_rules! field {
        ($ident:ident: $ty:ty) => {
            (stringify!($ident), stringify!($ty))
        };
    }
    lazy_static! {
        static ref DEFAULT_FIELDS: HashMap<&'static str, &'static str> = HashMap::from([
            field!(scope_id: Cell<Option<ScopeId>>),
            field!(symbol_id: Cell<Option<SymbolId>>),
            field!(reference_id: Cell<Option<ReferenceId>>),
            field!(reference_flag: ReferenceFlag),
        ]);
    }
    if let Some(default_type) = DEFAULT_FIELDS.get(ident.to_string().as_str()) {
        *default_type == typ.to_token_stream().to_string().replace(' ', "")
    } else {
        false
    }
}

fn generate_struct_builder_fn(ty: &RStruct, ctx: &CodegenCtx) -> TokenStream {
    fn default_field(param: &Param) -> TokenStream {
        debug_assert!(param.is_default);
        let ident = &param.ident;
        quote!(#ident: Default::default())
    }
    let ident = ty.ident();
    let as_type = ty.as_type();
    let fn_name = struct_builder_name(ty);

    let params = get_struct_params(ty, ctx);
    let (generic_params, where_clause) = get_generic_params(&params);

    let fields = params
        .iter()
        .map(|param| {
            if param.is_default {
                default_field(param)
            } else if param.into_in {
                let ident = &param.ident;
                quote!(#ident: #ident.into_in(self.allocator))
            } else {
                param.ident.to_token_stream()
            }
        })
        .collect_vec();

    let params = params.into_iter().filter(Param::not_default).collect_vec();
    let args = params.iter().map(|it| it.ident.clone());

    let alloc_fn_name = format_ident!("alloc_{fn_name}");

    let article = article_for(ident.to_string());
    let fn_docs = DocComment::new(format!("Builds {article} [`{ident}`]"))
        .with_description(format!("If you want the built node to be allocated in the memory arena, use [`AstBuilder::{alloc_fn_name}`] instead."))
        .with_params(&params);

    let alloc_docs =
        DocComment::new(format!("Builds {article} [`{ident}`] and stores it in the memory arena."))
            .with_description(format!("Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::{fn_name}`] instead."))
            .with_params(&params);

    quote! {
        endl!();
        #fn_docs
        #[inline]
        pub fn #fn_name #generic_params (self, #(#params),*) -> #as_type  #where_clause {
            #ident { #(#fields),* }
        }

        endl!();

        #alloc_docs
        #[inline]
        pub fn #alloc_fn_name #generic_params (self, #(#params),*) -> Box<'a, #as_type> #where_clause {
            Box::new_in(self.#fn_name(#(#args),*), self.allocator)
        }
    }
}

#[derive(Debug)]
struct Param {
    is_default: bool,
    info: TypeAnalyzeResult,
    ident: Ident,
    ty: Type,
    generic: Option<(/* predicate */ TokenStream, /* param name */ TokenStream)>,
    into_in: bool,
    docs: Option<String>,
}

impl Param {
    fn is_default(&self) -> bool {
        self.is_default
    }

    fn not_default(&self) -> bool {
        !self.is_default()
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let ty = &self.ty;
        tokens.extend(quote!(#ident: #ty));
    }
}

/// Represents a rusdoc comment that will be added to a generated function,
/// struct, etc.
///
/// [`DocComment`] implements [`ToTokens`], so you can use it in a [`quote!`]
/// block as normal.
///
/// ```ignore
/// let docs = DocComment::new("This is a summary")
///     .with_description("This is a longer description");
///
/// let my_function = quote! {
///     #doc
///     fn my_function() {
///     }
/// }
/// ```
///
/// This generates comments in the following format:
///
/// ```md
/// <summary>
///
/// <description>
///
/// ## Parameters
/// - param1: some docs
/// - param2
/// ```
///
/// 1. [`summary`] is a single-line overview about the thing being documented.
/// 2. [`description`] is a longer-form description that can span multiple
///    lines. It will be split into paragraphs for you.
/// 3. [`parameters`] is a bulleted list of function parameters. Documentation
///    for them can be extracted from struct fields and enums. This really only applies to functions.
///
/// Each section only appears if there is content for it. Only [`summary`] is required.
///
/// [`summary`]: DocComment::summary
/// [`description`]: DocComment::description
/// [`parameters`]: DocComment::params
///
#[derive(Debug)]
struct DocComment<'p> {
    /// Single-line summary. Put at the top of the comment.
    summary: Cow<'static, str>,
    /// Zero or more description paragraphs.
    description: Vec<Cow<'static, str>>,
    /// Function parameters, if applicable. Will be used to create a parameter
    /// section that looks like this:
    ///
    /// ```md
    /// ## Parameters
    /// - first_param: some docs taken from the [`Param`]
    /// - second_param
    /// ```
    params: &'p [Param],
}

impl<'p> DocComment<'p> {
    pub fn new<S>(summary: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self { summary: Self::maybe_add_space(summary.into()), description: vec![], params: &[] }
    }

    /// Add a longer-form description to the doc comment.
    pub fn with_description<S>(mut self, description: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.description = vec![Self::maybe_add_space(description.into())];
        self
    }

    /// Add a description section made up of multiple lines.
    ///
    /// Each line will be turned into its own paragraph.
    pub fn with_description_lines<L, S>(mut self, description: L) -> Self
    where
        S: Into<Cow<'static, str>>,
        L: IntoIterator<Item = S>,
    {
        self.description =
            description.into_iter().map(Into::into).map(Self::maybe_add_space).collect();
        self
    }

    /// Add a section documenting function parameters.
    pub fn with_params(mut self, params: &'p Vec<Param>) -> Self {
        self.params = params.as_slice();
        self
    }

    /// Add a leading space to a doc comment line if it doesn't already have one.
    /// This makes it easier to read, since the comment won't be directly next
    /// to the `///`.
    fn maybe_add_space(s: Cow<'static, str>) -> Cow<'static, str> {
        if s.is_empty() || s.starts_with(' ') {
            s
        } else {
            Cow::Owned(format!(" {s}"))
        }
    }
}

/// Get the correct article (a/an) that should precede a `word`.
///
/// # Panics
/// if `word` is empty
fn article_for<S: AsRef<str>>(word: S) -> &'static str {
    match word.as_ref().chars().next().unwrap() {
        'a' | 'e' | 'i' | 'o' | 'u' => "an",
        _ => "a",
    }
}

impl ToTokens for DocComment<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        macro_rules! newline {
            () => {
                tokens.extend(quote!( #[doc = ""]));
            };
        }

        let summary = &self.summary;
        tokens.extend(quote!( #[doc = #summary]));

        // print description
        for line in &self.description {
            // extra newline needed to create a new paragraph
            newline!();
            tokens.extend(quote!( #[doc = #line]));
        }

        // print docs for function parameters
        if !self.params.is_empty() {
            newline!();
            tokens.extend(quote!( #[doc = " ## Parameters"]));
            for param in self.params {
                match &param.docs {
                    Some(docs) => {
                        let docs = format!(" - {}: {}", param.ident, docs.trim());
                        tokens.extend(quote!(
                            #[doc = #docs]
                        ));
                    }
                    None if param.ident == "span" => {
                        tokens.extend(quote!(
                            #[doc = " - span: The [`Span`] covering this node"]
                        ));
                    }
                    None => {
                        let docs = format!(" - {}", param.ident);
                        tokens.extend(quote!(#[doc = #docs]));
                    }
                }
            }
        }
    }
}
fn get_doc_comment(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        Meta::NameValue(MetaNameValue { path, value: Expr::Lit(lit), .. }) => {
            if !path.is_ident("doc") {
                return None;
            }

            match &lit.lit {
                Lit::Str(lit) => Some(lit.value()),
                _ => None,
            }
        }
        _ => None,
    })
}
fn get_enum_params(enum_: &REnum, ctx: &CodegenCtx) -> Vec<Param> {
    let as_type = enum_.as_type();
    let inner_type = match &as_type {
        ty @ Type::Path(TypePath { path, .. }) if path.get_ident().is_none() => {
            assert_eq!(path.segments.len(), 1);
            let seg1 = &path.segments[0];
            if seg1.ident == "Box" {
                match &seg1.arguments {
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => {
                        assert!(matches!(args[0], GenericArgument::Lifetime(_)));
                        let GenericArgument::Type(ref inner_type) = args[1] else {
                            panic!("Unsupported box type!")
                        };
                        inner_type.clone()
                    }
                    _ => panic!("Unsupported box type!"),
                }
            } else {
                ty.clone()
            }
        }
        ty => ty.clone(),
    };
    let inner =
        ctx.find(&inner_type.get_ident().inner_ident().to_string()).expect("type not found!");
    match &*TypeRef::clone(&inner).borrow() {
        RType::Enum(_) => {
            vec![Param {
                is_default: false,
                info: as_type.analyze(ctx),
                ident: format_ident!("inner"),
                ty: inner_type.clone(),
                generic: None,
                into_in: false,
                docs: None,
            }]
        }
        RType::Struct(it) => get_struct_params(it, ctx),
        _ => panic!(),
    }
}

fn get_generic_params(
    params: &[Param],
) -> (/* generic params */ Option<TokenStream>, /* where clause */ Option<TokenStream>) {
    let params = params.iter().filter(|it| it.generic.is_some()).collect_vec();
    if params.is_empty() {
        return Default::default();
    }

    let len = params.len();
    let (predicates, params) = params.into_iter().fold(
        (Vec::with_capacity(len), Vec::with_capacity(len)),
        |mut acc, it| {
            let generic =
                it.generic.as_ref().expect("non-generics should be filtered out at this point.");
            acc.0.push(&generic.0);
            acc.1.push(&generic.1);
            acc
        },
    );
    (Some(quote!(<#(#params),*>)), Some(quote!(where #(#predicates),*)))
}

// TODO: currently doesn't support multiple `Atom` or `&'a str` params.
fn get_struct_params(struct_: &RStruct, ctx: &CodegenCtx) -> Vec<Param> {
    // generic param postfix
    let mut t_count = 0;
    let mut t_param = move || {
        t_count += 1;
        format_ident!("T{t_count}").to_token_stream()
    };
    struct_
        .item
        .fields
        .iter()
        .map(|f| {
            let id = f.ident.as_ref().expect("expected named ident! on struct");
            let docs = get_doc_comment(&f.attrs);
            ((id, &f.ty), docs)
        })
        .fold(Vec::new(), |mut acc, (ref it, docs)| {
            let (id, ty) = *it;
            let info = ty.analyze(ctx);
            let (interface_typ, generic_typ) = match (&info.wrapper, &info.type_ref) {
                (TypeWrapper::Box, Some(ref type_ref)) => {
                    let t = t_param();
                    let typ = type_ref.borrow().as_type().unwrap();
                    (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, Box<'a, #typ>>), t)))
                }
                (TypeWrapper::OptBox, Some(ref type_ref)) => {
                    let t = t_param();
                    let typ = type_ref.borrow().as_type().unwrap();
                    (
                        Some(parse_quote!(#t)),
                        Some((quote!(#t: IntoIn<'a, Option<Box<'a, #typ>>>), t)),
                    )
                }
                (TypeWrapper::Ref, None) if ty.get_ident().inner_ident() == "str" => {
                    let t = format_ident!("S").to_token_stream();
                    (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, &'a str>), t)))
                }
                (TypeWrapper::None, None) if ty.get_ident().inner_ident() == "Atom" => {
                    let t = format_ident!("A").to_token_stream();
                    (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, Atom<'a>>), t)))
                }
                _ => (None, None),
            };
            let ty = interface_typ.unwrap_or_else(|| ty.clone());
            acc.push(Param {
                is_default: default_init_field(it),
                info,
                ident: id.clone(),
                ty,
                into_in: generic_typ.is_some(),
                generic: generic_typ,
                docs,
            });
            acc
        })
}
