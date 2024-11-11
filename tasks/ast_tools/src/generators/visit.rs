use std::borrow::Cow;

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use rustc_hash::FxHashMap;
use syn::{parse_quote, Ident};

use crate::{
    generators::ast_kind::BLACK_LIST as KIND_BLACK_LIST,
    markers::VisitArg,
    output::{output_path, Output},
    schema::{EnumDef, GetIdent, Schema, StructDef, ToType, TypeDef},
    util::{StrExt, TokenStreamExt, TypeWrapper},
    Generator,
};

use super::define_generator;

pub struct VisitGenerator;

define_generator!(VisitGenerator);

impl Generator for VisitGenerator {
    fn generate(&mut self, schema: &Schema) -> Output {
        Output::Rust {
            path: output_path(crate::AST_CRATE, "visit.rs"),
            tokens: generate_visit(false, schema),
        }
    }
}

pub struct VisitMutGenerator;

define_generator!(VisitMutGenerator);

impl Generator for VisitMutGenerator {
    fn generate(&mut self, schema: &Schema) -> Output {
        Output::Rust {
            path: output_path(crate::AST_CRATE, "visit_mut.rs"),
            tokens: generate_visit(true, schema),
        }
    }
}

fn generate_visit(is_mut: bool, schema: &Schema) -> TokenStream {
    let (visits, walks) = VisitBuilder::new(schema, is_mut).build();

    let walk_mod = if is_mut { quote!(walk_mut) } else { quote!(walk) };
    let trait_name = if is_mut { quote!(VisitMut) } else { quote!(Visit) };
    let ast_kind_type = if is_mut { quote!(AstType) } else { quote!(AstKind) };
    let ast_kind_life = if is_mut { TokenStream::default() } else { quote!(<'a>) };

    let may_alloc = if is_mut {
        TokenStream::default()
    } else {
        quote! {
            ///@@line_break
            #[inline]
            fn alloc<T>(&self, t: &T) -> &'a T {
                ///@ SAFETY:
                ///@ This should be safe as long as `src` is an reference from the allocator.
                ///@ But honestly, I'm not really sure if this is safe.
                unsafe {
                    std::mem::transmute(t)
                }
            }
        }
    };

    quote! {
        //! Visitor Pattern
        //!
        //! See:
        //! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
        //! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

        //!@@line_break
        #![allow(
            unused_variables,
            clippy::extra_unused_type_parameters,
            clippy::explicit_iter_loop,
            clippy::self_named_module_files,
            clippy::semicolon_if_nothing_returned,
            clippy::match_wildcard_for_single_variants
        )]

        ///@@line_break
        use std::cell::Cell;

        ///@@line_break
        use oxc_allocator::Vec;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        ///@@line_break
        use crate::ast::*;
        use crate::ast_kind::#ast_kind_type;

        ///@@line_break
        use #walk_mod::*;

        ///@@line_break
        /// Syntax tree traversal
        pub trait #trait_name <'a>: Sized {
            #[inline]
            fn enter_node(&mut self, kind: #ast_kind_type #ast_kind_life) {}
            #[inline]
            fn leave_node(&mut self, kind: #ast_kind_type #ast_kind_life) {}

            ///@@line_break
            #[inline]
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
            #[inline]
            fn leave_scope(&mut self) {}

            #may_alloc

            #(#visits)*
        }

        ///@@line_break
        pub mod #walk_mod {
            use super::*;

            ///@@line_break
            #(#walks)*
        }
    }
}

struct VisitBuilder<'a> {
    schema: &'a Schema,

    is_mut: bool,

    visits: Vec<TokenStream>,
    walks: Vec<TokenStream>,
    cache: FxHashMap<Ident, [Option<Cow<'a, Ident>>; 2]>,
}

impl<'a> VisitBuilder<'a> {
    fn new(schema: &'a Schema, is_mut: bool) -> Self {
        Self { schema, is_mut, visits: Vec::new(), walks: Vec::new(), cache: FxHashMap::default() }
    }

    fn build(mut self) -> (/* visits */ Vec<TokenStream>, /* walks */ Vec<TokenStream>) {
        let program = self
            .schema
            .defs
            .iter()
            .filter(|it| it.is_visitable())
            .find(|it| it.name() == "Program")
            .expect("Couldn't find the `Program` type!");

        self.get_visitor(program, false);
        (self.visits, self.walks)
    }

    fn with_ref_pat<T>(&self, tk: T) -> TokenStream
    where
        T: ToTokens,
    {
        if self.is_mut {
            quote!(&mut #tk)
        } else {
            quote!(&#tk)
        }
    }

    fn kind_type(&self, ident: &Ident) -> TokenStream {
        if self.is_mut {
            quote!(AstType::#ident)
        } else {
            quote!(AstKind::#ident(visitor.alloc(it)))
        }
    }

    fn get_visitor(&mut self, def: &TypeDef, collection: bool) -> Cow<'a, Ident> {
        let cache_ix = usize::from(collection);
        let (ident, as_type) = {
            debug_assert!(def.is_visitable(), "{def:?}");

            let ident = def.ident();
            let as_type = def.to_type();

            (ident, if collection { parse_quote!(Vec<'a, #as_type>) } else { as_type })
        };

        // is it already generated?
        if let Some(cached) = self.cache.get(&ident) {
            if let Some(cached) = &cached[cache_ix] {
                return Cow::clone(cached);
            }
        }

        let ident_snake = {
            let it = ident.to_string().to_case(Case::Snake);
            let it = if collection {
                // edge case for `Vec<FormalParameter>` to avoid conflicts with `FormalParameters`
                // which both would generate the same name: `visit_formal_parameters`.
                // and edge case for `Vec<TSImportAttribute>` to avoid conflicts with
                // `TSImportAttributes` which both would generate the same name: `visit_formal_parameters`.
                if matches!(it.as_str(), "formal_parameter" | "ts_import_attribute") {
                    let mut it = it;
                    it.push_str("_list");
                    it
                } else {
                    it.to_plural()
                }
            } else {
                it
            };
            format_ident!("{it}")
        };

        let as_param_type = self.with_ref_pat(&as_type);
        let (extra_params, extra_args) = if ident == "Function" {
            (quote!(, flags: ScopeFlags,), quote!(, flags))
        } else {
            (TokenStream::default(), TokenStream::default())
        };

        let visit_name = {
            let visit_name = format_ident!("visit_{}", ident_snake);
            if !self.cache.contains_key(&ident) {
                debug_assert!(self.cache.insert(ident.clone(), [None, None]).is_none());
            }
            let cached = self.cache.get_mut(&ident).unwrap();
            assert!(cached[cache_ix].replace(Cow::Owned(visit_name)).is_none());
            Cow::clone(cached[cache_ix].as_ref().unwrap())
        };

        let walk_name = format_ident!("walk_{}", ident_snake);

        self.visits.push(quote! {
            ///@@line_break
            #[inline]
            fn #visit_name (&mut self, it: #as_param_type #extra_params) {
                #walk_name(self, it #extra_args);
            }
        });

        // We push an empty walk first, because we evaluate - and generate - each walk as we go,
        // This would let us to maintain the order of first visit.
        let this_walker = self.walks.len();
        self.walks.push(TokenStream::default());

        let (walk_body, may_inline) = if collection {
            let singular_visit = self.get_visitor(def, false);
            let iter = if self.is_mut { quote!(it.iter_mut()) } else { quote!(it) };
            (
                quote! {
                    for el in #iter {
                        visitor.#singular_visit(el);
                    }
                },
                true,
            )
        } else {
            match def {
                TypeDef::Enum(enum_) => self.generate_enum_walk(enum_),
                TypeDef::Struct(struct_) => self.generate_struct_walk(struct_),
            }
        };

        let visit_trait = if self.is_mut { quote!(VisitMut) } else { quote!(Visit) };
        let may_inline = if may_inline { Some(quote!(#[inline])) } else { None };

        // replace the placeholder walker with the actual one!
        self.walks[this_walker] = quote! {
            ///@@line_break
            #may_inline
            pub fn #walk_name <'a, V: #visit_trait<'a>>(visitor: &mut V, it: #as_param_type #extra_params) {
                #walk_body
            }
        };

        visit_name
    }

    fn generate_enum_walk(&mut self, enum_: &EnumDef) -> (TokenStream, /* inline */ bool) {
        let ident = enum_.ident();
        let mut non_exhaustive = false;
        let variants_matches = enum_
            .variants
            .iter()
            .filter(|var| {
                if var.markers.visit.ignore {
                    // We are ignoring some variants so the match is no longer exhaustive.
                    non_exhaustive = true;
                    false
                } else {
                    true
                }
            })
            .filter_map(|var| {
                let typ = var
                    .fields
                    .iter()
                    .exactly_one()
                    .map(|f| &f.typ)
                    .map_err(|_| "We only support visited enum nodes with exactly one field!")
                    .unwrap();
                let variant_name = &var.ident();
                let type_id = typ.transparent_type_id()?;
                let def = self.schema.get(type_id)?;
                let is_visitable = def.is_visitable();
                if is_visitable {
                    let visit = self.get_visitor(def, false);
                    let (args_def, args) = var
                        .markers
                        .visit
                        .visit_args
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .fold((Vec::new(), Vec::new()), Self::visit_args_fold);
                    let body = quote!(visitor.#visit(it #(#args)*));
                    let body = if args_def.is_empty() {
                        body
                    } else {
                        // if we have args wrap the result in a block to prevent ident clashes.
                        quote! {{
                            #(#args_def)*
                            #body
                        }}
                    };
                    Some(quote!(#ident::#variant_name(it) => #body))
                } else {
                    None
                }
            })
            .collect_vec();

        let inherit_matches = enum_.inherits.iter().filter_map(|it| {
            let super_ = &it.super_;
            let type_name = super_.name().as_name().unwrap().to_string();
            let def = super_.type_id().and_then(|id| self.schema.get(id))?;
            if def.is_visitable() {
                let snake_name = type_name.to_case(Case::Snake);
                let match_macro = format_ident!("match_{snake_name}");
                let match_macro = quote!(#match_macro!(#ident));
                let to_child = if self.is_mut {
                    format_ident!("to_{snake_name}_mut")
                } else {
                    format_ident!("to_{snake_name}")
                };
                let visit = self.get_visitor(def, false);
                Some(quote!(#match_macro => visitor.#visit(it.#to_child())))
            } else {
                None
            }
        });

        let matches = variants_matches.into_iter().chain(inherit_matches).collect_vec();

        let with_node_events = |tk| {
            if KIND_BLACK_LIST.contains(&ident.to_string().as_str()) {
                tk
            } else {
                let kind = self.kind_type(&ident);
                quote! {
                    let kind = #kind;
                    visitor.enter_node(kind);
                    #tk
                    visitor.leave_node(kind);
                }
            }
        };
        let non_exhaustive = if non_exhaustive { Some(quote!(,_ => {})) } else { None };
        (
            with_node_events(quote!(match it { #(#matches),* #non_exhaustive })),
            // inline if there are 5 or less match cases
            matches.len() <= 5,
        )
    }

    fn generate_struct_walk(&mut self, struct_: &StructDef) -> (TokenStream, /* inline */ bool) {
        let ident = struct_.ident();
        let scope_events =
            struct_.markers.scope.as_ref().map_or_else(Default::default, |markers| {
                let flags = markers
                    .flags
                    .as_ref()
                    .map_or_else(|| quote!(ScopeFlags::empty()), ToTokens::to_token_stream);
                let flags = if let Some(strict_if) = &markers.strict_if {
                    let strict_if =
                        strict_if.to_token_stream().replace_ident("self", &format_ident!("it"));
                    quote! {{
                        let mut flags = #flags;
                        if #strict_if {
                            flags |= ScopeFlags::StrictMode;
                        }
                        flags
                    }}
                } else {
                    flags
                };
                let enter = quote!(visitor.enter_scope(#flags, &it.scope_id););
                let leave = quote!(visitor.leave_scope(););
                (enter, leave)
            });

        let node_events = if KIND_BLACK_LIST.contains(&ident.to_string().as_str()) {
            let comment = format!(
                "@ NOTE: {} doesn't exists!",
                if self.is_mut { "AstType" } else { "AstKind" }
            );
            (quote!(#![doc = #comment]), TokenStream::default())
        } else {
            let kind = self.kind_type(&ident);
            (
                quote! {
                    let kind = #kind;
                    visitor.enter_node(kind);
                },
                quote!(visitor.leave_node(kind);),
            )
        };

        let mut enter_scope_at = 0;
        let mut exit_scope_at: Option<usize> = None;
        let mut enter_node_at = 0;
        let fields_visits: Vec<TokenStream> = struct_
            .fields
            .iter()
            .enumerate()
            .filter_map(|(ix, field)| {
                let analysis = field.typ.analysis();
                let def = field.typ.transparent_type_id().and_then(|id| self.schema.get(id))?;
                if !def.is_visitable() {
                    return None;
                }
                let typ_wrapper = &analysis.wrapper;
                let markers = &field.markers;
                let visit_args = markers.visit.visit_args.clone();

                let have_enter_scope = markers.scope.enter_before;
                let have_exit_scope = markers.scope.exit_before;
                let have_enter_node = markers.visit.enter_before;

                let (args_def, args) = visit_args
                    .map(|it| it.into_iter().fold((Vec::new(), Vec::new()), Self::visit_args_fold))
                    .unwrap_or_default();
                let visit = self.get_visitor(
                    def,
                    matches!(
                        typ_wrapper,
                        TypeWrapper::Vec | TypeWrapper::VecBox | TypeWrapper::OptVec
                    ),
                );
                let name = field.ident().expect("expected named fields!");
                let borrowed_field = self.with_ref_pat(quote!(it.#name));
                let mut result = match typ_wrapper {
                    TypeWrapper::Opt | TypeWrapper::OptBox | TypeWrapper::OptVec => quote! {
                        if let Some(#name) = #borrowed_field {
                            visitor.#visit(#name #(#args)*);
                        }
                    },
                    TypeWrapper::VecOpt => {
                        let iter = if self.is_mut { quote!(iter_mut) } else { quote!(iter) };
                        quote! {
                            for #name in it.#name.#iter().flatten() {
                                visitor.#visit(#name #(#args)*);
                            }
                        }
                    }
                    _ => quote! {
                        visitor.#visit(#borrowed_field #(#args)*);
                    },
                };

                // This comes first because we would prefer the `enter_node` to be placed on top of `enter_scope`
                if have_enter_scope {
                    assert_eq!(enter_scope_at, 0);
                    let scope_enter = &scope_events.0;
                    result = quote! {
                        #scope_enter
                        #result
                    };
                    enter_scope_at = ix;
                }
                if have_exit_scope {
                    assert!(
                        exit_scope_at.is_none(),
                        "Scopes cannot be exited more than once. Remove the extra `#[scope(exit_before)]` attribute(s)."
                    );
                    let scope_exit = &scope_events.1;
                    result = quote! {
                        #scope_exit
                        #result
                    };
                    exit_scope_at = Some(ix);
                }

                #[expect(unreachable_code)]
                if have_enter_node {
                    // NOTE: this is disabled intentionally <https://github.com/oxc-project/oxc/pull/4147#issuecomment-2220216905>
                    unreachable!("`#[visit(enter_before)]` attribute is disabled!");
                    assert_eq!(enter_node_at, 0);
                    let node_enter = &node_events.0;
                    result = quote! {
                        #node_enter
                        #result
                    };
                    enter_node_at = ix;
                }

                if args_def.is_empty() {
                    Some(result)
                } else {
                    // if we have args wrap the result in a block to prevent ident clashes.
                    Some(quote! {{
                        #(#args_def)*
                        #result
                    }})
                }
            })
            .collect();

        let with_node_events = |body: TokenStream| match (node_events, enter_node_at) {
            ((enter, leave), 0) => quote! {
                #enter
                #body
                #leave
            },
            ((_, leave), _) => quote! {
                #body
                #leave
            },
        };

        let with_scope_events =
            |body: TokenStream| match (scope_events, enter_scope_at, exit_scope_at) {
                ((enter, leave), 0, None) => quote! {
                    #enter
                    #body
                    #leave
                },
                ((_, leave), _, None) => quote! {
                    #body
                    #leave
                },
                ((enter, _), 0, Some(_)) => quote! {
                    #enter
                    #body
                },
                ((_, _), _, Some(_)) => quote! {
                    #body
                },
            };

        let body = with_node_events(with_scope_events(quote!(#(#fields_visits)*)));

        // inline if there are 5 or less fields.
        (body, fields_visits.len() <= 5)
    }

    fn visit_args_fold(
        mut accumulator: (Vec<TokenStream>, Vec<TokenStream>),
        arg: VisitArg,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let VisitArg { ident: id, value: val } = arg;
        let val = val.to_token_stream().replace_ident("self", &format_ident!("it"));
        accumulator.0.push(quote!(let #id = #val;));
        accumulator.1.push(quote!(, #id));
        accumulator
    }
}
