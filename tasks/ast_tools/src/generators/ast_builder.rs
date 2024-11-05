use std::{borrow::Cow, stringify};

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Ident, Type};

use crate::{
    output::{output_path, Output},
    schema::{
        EnumDef, FieldDef, GetIdent, Schema, StructDef, ToType, TypeDef, TypeName, VariantDef,
    },
    util::{TypeAnalysis, TypeWrapper},
    Generator,
};

use super::define_generator;

pub struct AstBuilderGenerator;

define_generator!(AstBuilderGenerator);

impl Generator for AstBuilderGenerator {
    fn generate(&mut self, schema: &Schema) -> Output {
        let fns = schema
            .defs
            .iter()
            .filter(|it| it.is_visitable())
            .map(|it| generate_builder_fn(it, schema))
            .collect_vec();

        Output::Rust {
            path: output_path(crate::AST_CRATE, "ast_builder.rs"),
            tokens: quote! {
                //! AST node factories

                //!@@line_break
                #![allow(
                    clippy::default_trait_access,
                    clippy::too_many_arguments,
                    clippy::fn_params_excessive_bools,
                )]

                ///@@line_break
                use std::cell::Cell;

                ///@@line_break
                use oxc_allocator::{Allocator, Box, IntoIn, Vec};
                use oxc_syntax::{scope::ScopeId, symbol::SymbolId, reference::ReferenceId};

                ///@@line_break
                use crate::ast::*;

                ///@@line_break
                /// AST builder for creating AST nodes
                #[derive(Clone, Copy)]
                pub struct AstBuilder<'a> {
                    /// The memory allocator used to allocate AST nodes in the arena.
                    pub allocator: &'a Allocator,
                }

                ///@@line_break
                impl<'a> AstBuilder<'a> {
                    #(#fns)*
                }
            },
        }
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

fn struct_builder_name(struct_: &StructDef) -> Ident {
    static RUST_KEYWORDS: [&str; 1] = ["super"];
    let mut ident = fn_ident_name(struct_.name.as_str());
    if RUST_KEYWORDS.contains(&ident.as_str()) {
        ident.push('_');
    }
    format_ident!("{ident}")
}

fn generate_builder_fn(def: &TypeDef, schema: &Schema) -> TokenStream {
    match def {
        TypeDef::Enum(def) => generate_enum_builder_fn(def, schema),
        TypeDef::Struct(def) => generate_struct_builder_fn(def, schema),
    }
}

fn generate_enum_builder_fn(def: &EnumDef, schema: &Schema) -> TokenStream {
    def.variants.iter().map(|it| generate_enum_variant_builder_fn(def, it, schema)).collect()
}

/// Create a builder function for an enum variant (e.g. for `Expression::Binary`)
fn generate_enum_variant_builder_fn(
    enum_: &EnumDef,
    variant: &VariantDef,
    schema: &Schema,
) -> TokenStream {
    assert_eq!(variant.fields.len(), 1);
    let enum_ident = enum_.ident();
    let enum_type = &enum_.to_type();
    let var_ident = &variant.ident();
    let var_type = &variant.fields.first().expect("we have already asserted this one!").typ;
    let var_type_name = &var_type.name();
    let fn_name = enum_builder_name(enum_ident.to_string(), var_type_name.inner_name().to_string());
    let ty = var_type
        .type_id()
        .or_else(|| var_type.transparent_type_id())
        .and_then(|id| schema.get(id))
        .expect("type not found!");
    let (params, inner_builder) = match ty {
        TypeDef::Struct(it) => (get_struct_params(it, schema), struct_builder_name(it)),
        TypeDef::Enum(_) => panic!("Unsupported!"),
    };

    let params = params.into_iter().filter(Param::not_default).collect_vec();
    let fields = params.iter().map(|it| it.ident.clone());
    let (generic_params, where_clause) = get_generic_params(&params);

    let mut inner = quote!(self.#inner_builder(#(#fields),*));
    let mut does_alloc = false;
    if matches!(var_type_name, TypeName::Box(_)) {
        inner = quote!(self.alloc(#inner));
        does_alloc = true;
    }

    let article = article_for(enum_ident.to_string());
    let mut docs = DocComment::new(format!(" Build {article} [`{enum_ident}::{var_ident}`]"))
        .with_params(&params);
    if does_alloc {
        let inner_name = var_type_name.inner_name();
        let inner_article = article_for(inner_name);
        docs = docs.with_description(format!(
            "This node contains {inner_article} [`{inner_name}`] that will be stored in the memory arena."
        ));
    }

    quote! {
        ///@@line_break
        #docs
        #[inline]
        pub fn #fn_name #generic_params (self, #(#params),*) -> #enum_type #where_clause {
            #enum_ident::#var_ident(#inner)
        }
    }
}

fn default_init_field(field: &FieldDef) -> bool {
    let ident = field.ident().expect("expected named field");
    matches!(
        (ident.to_string().as_str(), field.typ.raw()),
        ("scope_id", "Cell<Option<ScopeId>>")
            | ("symbol_id", "Cell<Option<SymbolId>>")
            | ("reference_id", "Cell<Option<ReferenceId>>")
    )
}

/// Generate builder function for struct.
///
/// Generates functions:
/// 1. to create owned object.
/// 2. to create boxed object.
///
/// If type has default fields (`scope_id`, `symbol_id`, `reference_id`), also generates functions:
///
/// 3. to create owned object with provided `ScopeId` / `SymbolId` / `ReferenceId`.
/// 4. to create boxed object with provided `ScopeId` / `SymbolId` / `ReferenceId`.
fn generate_struct_builder_fn(ty: &StructDef, schema: &Schema) -> TokenStream {
    let ident = ty.ident();
    let as_type = ty.to_type();
    let fn_name = struct_builder_name(ty);
    let alloc_fn_name = format_ident!("alloc_{fn_name}");

    let params_incl_defaults = get_struct_params(ty, schema);
    let (generic_params, where_clause) = get_generic_params(&params_incl_defaults);

    let mut has_default_fields = false;
    let mut params = vec![];
    let mut fn_params_incl_defaults = vec![];
    let mut default_field_names = vec![];
    let mut default_field_type_names = vec![];
    let mut fields = vec![];
    let mut fields_incl_defaults = vec![];
    let mut args = vec![];
    let mut args_incl_defaults = vec![];

    for param in &params_incl_defaults {
        let mut field = if param.into_in {
            let ident = &param.ident;
            quote!(#ident: #ident.into_in(self.allocator))
        } else {
            param.ident.to_token_stream()
        };

        if param.is_default && !has_default_fields {
            has_default_fields = true;
            fn_params_incl_defaults = params.iter().map(Param::to_token_stream).collect();
            fields_incl_defaults.clone_from(&fields);
            args_incl_defaults.clone_from(&args);
        }

        if param.is_default {
            let field_ident = &param.ident;
            field = quote!(#field_ident: Default::default());

            let field_name = field_ident.to_string();
            let field_type_name = match field_name.as_str() {
                "scope_id" => "ScopeId",
                "symbol_id" => "SymbolId",
                "reference_id" => "ReferenceId",
                _ => unreachable!(),
            };
            let field_type_ident = format_ident!("{field_type_name}");
            fn_params_incl_defaults.push(quote!(#field_ident: #field_type_ident));
            fields_incl_defaults.push(quote!( #field_ident: Cell::new(Some(#field_ident)) ));

            default_field_names.push(field_name);
            default_field_type_names.push(field_type_name);
        } else {
            params.push(param.clone());
            args.push(param.ident.clone());

            if has_default_fields {
                fn_params_incl_defaults.push(param.to_token_stream());
                fields_incl_defaults.push(field.clone());
            }
        }

        if has_default_fields {
            args_incl_defaults.push(param.ident.clone());
        }

        fields.push(field);
    }

    let article = article_for(ident.to_string());
    let create_docs = |fn_name, alloc_fn_name, params, extra| {
        let fn_docs = DocComment::new(format!("Build {article} [`{ident}`]{extra}."))
            .with_description(format!("If you want the built node to be allocated in the memory arena, use [`AstBuilder::{alloc_fn_name}`] instead."))
            .with_params(params);

        let alloc_docs = DocComment::new(format!("Build {article} [`{ident}`]{extra}, and store it in the memory arena."))
            .with_description(format!("Returns a [`Box`] containing the newly-allocated node. If you want a stack-allocated node, use [`AstBuilder::{fn_name}`] instead."))
            .with_params(params);

        (fn_docs, alloc_docs)
    };

    let (fn_docs, alloc_docs) = create_docs(&fn_name, &alloc_fn_name, &params, "");

    let mut output = quote! {
        ///@@line_break
        #fn_docs
        #[inline]
        pub fn #fn_name #generic_params (self, #(#params),*) -> #as_type #where_clause {
            #ident { #(#fields),* }
        }

        ///@@line_break
        #alloc_docs
        #[inline]
        pub fn #alloc_fn_name #generic_params (self, #(#params),*) -> Box<'a, #as_type> #where_clause {
            Box::new_in(self.#fn_name(#(#args),*), self.allocator)
        }
    };

    if has_default_fields {
        let fn_name = format_ident!("{fn_name}_with_{}", default_field_names.join("_and_"));
        let alloc_fn_name = format_ident!("alloc_{fn_name}");

        let with = format!(" with `{}`", default_field_type_names.iter().join("` and `"));
        let (fn_docs, alloc_docs) =
            create_docs(&fn_name, &alloc_fn_name, &params_incl_defaults, &with);

        output = quote! {
            #output

            ///@@line_break
            #fn_docs
            #[inline]
            pub fn #fn_name #generic_params (self, #(#fn_params_incl_defaults),*) -> #as_type #where_clause {
                #ident { #(#fields_incl_defaults),* }
            }

            ///@@line_break
            #alloc_docs
            #[inline]
            pub fn #alloc_fn_name #generic_params (self, #(#fn_params_incl_defaults),*) -> Box<'a, #as_type> #where_clause {
                Box::new_in(self.#fn_name(#(#args_incl_defaults),*), self.allocator)
            }
        };
    }

    output
}

// TODO: remove me
#[expect(dead_code)]
#[derive(Clone, Debug)]
struct Param {
    is_default: bool,
    analysis: TypeAnalysis,
    ident: Ident,
    ty: Type,
    generic: Option<(/* predicate */ TokenStream, /* param name */ TokenStream)>,
    into_in: bool,
    docs: Vec<String>,
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
    // TODO: remove me
    #[expect(dead_code)]
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
/// Panics if `word` is empty.
fn article_for<S: AsRef<str>>(word: S) -> &'static str {
    match word.as_ref().chars().next().unwrap().to_ascii_lowercase() {
        'a' | 'e' | 'i' | 'o' | 'u' => "an",
        _ => "a",
    }
}

impl ToTokens for DocComment<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let summary = &self.summary;
        tokens.extend(quote!( #[doc = #summary]));

        // print description
        for line in &self.description {
            // extra newline needed to create a new paragraph
            tokens.extend(quote!( #[doc = ""]));
            tokens.extend(quote!( #[doc = #line]));
        }

        // print docs for function parameters
        if !self.params.is_empty() {
            tokens.extend(quote!( #[doc = ""]));
            tokens.extend(quote!( #[doc = " ## Parameters"]));
            for param in self.params {
                let docs = param.docs.first();
                let docs = match docs {
                    Some(docs) => {
                        format!(" - {}: {}", param.ident, docs.trim())
                    }
                    None if param.ident == "span" => {
                        " - span: The [`Span`] covering this node".to_string()
                    }
                    None => {
                        format!(" - {}", param.ident)
                    }
                };
                tokens.extend(quote!(#[doc = #docs]));
            }
        }
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
fn get_struct_params(struct_: &StructDef, schema: &Schema) -> Vec<Param> {
    // generic param postfix
    let mut t_count = 0;
    let mut t_param = move || {
        t_count += 1;
        format_ident!("T{t_count}").to_token_stream()
    };
    struct_.fields.iter().fold(Vec::new(), |mut acc, field| {
        let analysis = field.typ.analysis();
        let type_def = field.typ.transparent_type_id().and_then(|id| schema.get(id));
        let (interface_typ, generic_typ) = match (&analysis.wrapper, type_def) {
            (TypeWrapper::Box, Some(def)) => {
                let t = t_param();
                let typ = def.to_type();
                (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, Box<'a, #typ>>), t)))
            }
            (TypeWrapper::OptBox, Some(def)) => {
                let t = t_param();
                let typ = def.to_type();
                (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, Option<Box<'a, #typ>>>), t)))
            }
            (TypeWrapper::Ref, None) if field.typ.is_str_slice() => {
                let t = format_ident!("S").to_token_stream();
                (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, &'a str>), t)))
            }
            (TypeWrapper::None, None) if field.typ.name().inner_name() == "Atom" => {
                let t = format_ident!("A").to_token_stream();
                (Some(parse_quote!(#t)), Some((quote!(#t: IntoIn<'a, Atom<'a>>), t)))
            }
            _ => (None, None),
        };
        let ty = interface_typ.unwrap_or_else(|| field.typ.to_type());
        acc.push(Param {
            is_default: default_init_field(field),
            analysis: analysis.clone(),
            ident: field.ident().expect("expected named ident! on struct"),
            ty,
            into_in: generic_typ.is_some(),
            generic: generic_typ,
            docs: field.docs.clone(),
        });
        acc
    })
}
