use std::{borrow::Cow, collections::HashMap};

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Ident};

use crate::{
    generators::{ast_kind::BLACK_LIST as KIND_BLACK_LIST, insert},
    markers::{
        get_scope_attr, get_scope_markers, get_visit_markers, ScopeMarkers, VisitArg, VisitMarkers,
    },
    output,
    schema::{Inherit, REnum, RStruct, RType},
    util::{StrExt, TokenStreamExt, TypeExt, TypeWrapper},
    CodegenCtx, Generator, GeneratorOutput, TypeRef,
};

use super::{define_generator, generated_header};

define_generator! {
    pub struct VisitGenerator;
}

define_generator! {
    pub struct VisitMutGenerator;
}

impl Generator for VisitGenerator {
    fn name(&self) -> &'static str {
        stringify!(VisitGenerator)
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "visit.rs"),
            generate_visit::<false>(ctx),
        ))
    }
}

impl Generator for VisitMutGenerator {
    fn name(&self) -> &'static str {
        stringify!(VisitMutGenerator)
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "visit_mut.rs"),
            generate_visit::<true>(ctx),
        ))
    }
}

const CLIPPY_ALLOW: &str = "\
    unused_variables,\
    clippy::extra_unused_type_parameters,\
    clippy::explicit_iter_loop,\
    clippy::self_named_module_files,\
    clippy::semicolon_if_nothing_returned,\
    clippy::match_wildcard_for_single_variants";

fn generate_visit<const MUT: bool>(ctx: &CodegenCtx) -> TokenStream {
    let header = generated_header!();
    // we evaluate it outside of quote to take advantage of expression evaluation
    // otherwise the `\n\` wouldn't work!
    let file_docs = insert! {"\
        //! Visitor Pattern\n\
        //!\n\
        //! See:\n\
        //! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)\n\
        //! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)\n\
    "};

    let (visits, walks) = VisitBuilder::new(ctx, MUT).build();
    let clippy_attr = insert!("#![allow({})]", CLIPPY_ALLOW);

    let walk_mod = if MUT { quote!(walk_mut) } else { quote!(walk) };
    let trait_name = if MUT { quote!(VisitMut) } else { quote!(Visit) };
    let ast_kind_type = if MUT { quote!(AstType) } else { quote!(AstKind) };
    let ast_kind_life = if MUT { TokenStream::default() } else { quote!(<'a>) };

    let may_alloc = if MUT {
        TokenStream::default()
    } else {
        quote! {
            #[inline]
            fn alloc<T>(&self, t: &T) -> &'a T {
                insert!("// SAFETY:");
                insert!("// This should be safe as long as `src` is an reference from the allocator.");
                insert!("// But honestly, I'm not really sure if this is safe.");

                unsafe {
                    std::mem::transmute(t)
                }
            }
        }
    };

    quote! {
        #header
        #file_docs
        #clippy_attr

        endl!();

        use std::cell::Cell;

        endl!();

        use oxc_allocator::Vec;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        endl!();

        use crate::{ast::*, ast_kind::#ast_kind_type};

        endl!();

        use #walk_mod::*;

        endl!();

        /// Syntax tree traversal
        pub trait #trait_name <'a>: Sized {
            #[inline]
            fn enter_node(&mut self, kind: #ast_kind_type #ast_kind_life) {}
            #[inline]
            fn leave_node(&mut self, kind: #ast_kind_type #ast_kind_life) {}

            endl!();

            #[inline]
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
            #[inline]
            fn leave_scope(&mut self) {}

            endl!();

            #may_alloc

            #(#visits)*
        }

        endl!();

        pub mod #walk_mod {
            use super::*;

            #(#walks)*

        }
    }
}

struct VisitBuilder<'a> {
    ctx: &'a CodegenCtx,

    is_mut: bool,

    visits: Vec<TokenStream>,
    walks: Vec<TokenStream>,
    cache: HashMap<Ident, [Option<Cow<'a, Ident>>; 2]>,
}

impl<'a> VisitBuilder<'a> {
    fn new(ctx: &'a CodegenCtx, is_mut: bool) -> Self {
        Self { ctx, is_mut, visits: Vec::new(), walks: Vec::new(), cache: HashMap::new() }
    }

    fn build(mut self) -> (/* visits */ Vec<TokenStream>, /* walks */ Vec<TokenStream>) {
        let program = {
            let types: Vec<&TypeRef> =
                self.ctx.ty_table.iter().filter(|it| it.borrow().visitable()).collect_vec();
            TypeRef::clone(
                types
                    .iter()
                    .find(|it| it.borrow().ident().is_some_and(|ident| ident == "Program"))
                    .expect("Couldn't find the `Program` type!"),
            )
        };

        self.get_visitor(&program, false, None);
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

    fn get_iter(&self) -> TokenStream {
        if self.is_mut {
            quote!(iter_mut)
        } else {
            quote!(iter)
        }
    }

    fn get_visitor(
        &mut self,
        ty: &TypeRef,
        collection: bool,
        visit_as: Option<&Ident>,
    ) -> Cow<'a, Ident> {
        let cache_ix = usize::from(collection);
        let (ident, as_type) = {
            let ty = ty.borrow();
            debug_assert!(ty.visitable(), "{ty:?}");

            let ident = ty.ident().unwrap();
            let as_type = ty.as_type().unwrap();

            let ident = visit_as.unwrap_or(ident);

            (ident.clone(), if collection { parse_quote!(Vec<'a, #as_type>) } else { as_type })
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
            endl!();
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
            let singular_visit = self.get_visitor(ty, false, None);
            let iter = self.get_iter();
            (
                quote! {
                    for el in it.#iter() {
                        visitor.#singular_visit(el);
                    }
                },
                true,
            )
        } else {
            match &*ty.borrow() {
                // TODO: this one is a hot-fix to prevent flattening aliased `Expression`s,
                // Such as `ExpressionArrayElement` and `ClassHeritage`.
                // Shouldn't be an edge case, <https://github.com/oxc-project/oxc/issues/4060>
                RType::Enum(enum_)
                    if enum_.item.ident == "Expression"
                        && visit_as.is_some_and(|it| {
                            it == "ExpressionArrayElement" || it == "ClassHeritage"
                        }) =>
                {
                    let kind = self.kind_type(visit_as.unwrap());
                    (
                        quote! {
                            let kind = #kind;
                            visitor.enter_node(kind);
                            visitor.visit_expression(it);
                            visitor.leave_node(kind);
                        },
                        false,
                    )
                }
                RType::Enum(enum_) => self.generate_enum_walk(enum_, visit_as),
                RType::Struct(struct_) => self.generate_struct_walk(struct_, visit_as),
                _ => panic!(),
            }
        };

        let visit_trait = if self.is_mut { quote!(VisitMut) } else { quote!(Visit) };
        let may_inline = if may_inline { Some(quote!(#[inline])) } else { None };

        // replace the placeholder walker with the actual one!
        self.walks[this_walker] = quote! {
            endl!();
            #may_inline
            pub fn #walk_name <'a, V: #visit_trait<'a>>(visitor: &mut V, it: #as_param_type #extra_params) {
                #walk_body
            }
        };

        visit_name
    }

    fn generate_enum_walk(
        &mut self,
        enum_: &REnum,
        visit_as: Option<&Ident>,
    ) -> (TokenStream, /* inline */ bool) {
        let ident = enum_.ident();
        let mut non_exhaustive = false;
        let variants_matches = enum_
            .item
            .variants
            .iter()
            .filter(|it| !it.attrs.iter().any(|a| a.path().is_ident("inherit")))
            .map(|it| (it, get_visit_markers(&it.attrs).transpose().unwrap()))
            .filter(|(_, markers)| {
                if markers.as_ref().is_some_and(|mk| mk.ignore) {
                    // We are ignoring some variants so the match is no longer exhaustive.
                    non_exhaustive = true;
                    false
                } else {
                    true
                }
            })
            .filter_map(|(it, markers)| {
                let typ = it
                    .fields
                    .iter()
                    .exactly_one()
                    .map(|f| &f.ty)
                    .map_err(|_| "We only support visited enum nodes with exactly one field!")
                    .unwrap();
                let variant_name = &it.ident;
                let typ = self.ctx.find(&typ.get_ident().inner_ident().to_string())?;
                let borrowed = typ.borrow();
                let visitable = borrowed.visitable();
                if visitable {
                    let visit = self.get_visitor(&typ, false, None);
                    let (args_def, args) = markers
                        .map(|mk| mk.visit_args.unwrap_or_default())
                        .into_iter()
                        .flatten()
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

        let inherit_matches = enum_.meta.inherits.iter().filter_map(|it| {
            let Inherit::Linked { super_, .. } = it else { panic!("Unresolved inheritance!") };
            let type_name = super_.get_ident().as_ident().unwrap().to_string();
            let typ = self.ctx.find(&type_name)?;
            if typ.borrow().visitable() {
                let snake_name = type_name.to_case(Case::Snake);
                let match_macro = format_ident!("match_{snake_name}");
                let match_macro = quote!(#match_macro!(#ident));
                // HACK: edge case till we get attributes to work with inheritance.
                let visit_as = if ident == "ArrayExpressionElement"
                    && super_.get_ident().inner_ident() == "Expression"
                {
                    Some(format_ident!("ExpressionArrayElement"))
                } else {
                    None
                };
                let to_child = if self.is_mut {
                    format_ident!("to_{snake_name}_mut")
                } else {
                    format_ident!("to_{snake_name}")
                };
                let visit = self.get_visitor(&typ, false, visit_as.as_ref());
                Some(quote!(#match_macro => visitor.#visit(it.#to_child())))
            } else {
                None
            }
        });

        let matches = variants_matches.into_iter().chain(inherit_matches).collect_vec();

        let with_node_events = |tk| {
            let ident = visit_as.unwrap_or(ident);
            if KIND_BLACK_LIST.contains(&ident.to_string().as_str()) {
                tk
            } else {
                let kind = self.kind_type(ident);
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

    fn generate_struct_walk(
        &mut self,
        struct_: &RStruct,
        visit_as: Option<&Ident>,
    ) -> (TokenStream, /* inline */ bool) {
        let ident = visit_as.unwrap_or_else(|| struct_.ident());
        let scope_events = get_scope_attr(struct_.item.attrs.iter())
            .transpose()
            .unwrap()
            .map_or_else(Default::default, |scope_args| {
                let cond = scope_args.r#if.map(|cond| {
                    let cond = cond.to_token_stream().replace_ident("self", &format_ident!("it"));
                    quote!(let scope_events_cond = #cond;)
                });
                let maybe_conditional = |tk: TokenStream| {
                    if cond.is_some() {
                        quote! {
                            if scope_events_cond {
                                #tk
                            }
                        }
                    } else {
                        tk
                    }
                };
                let flags = scope_args
                    .flags
                    .map_or_else(|| quote!(ScopeFlags::empty()), |it| it.to_token_stream());
                let flags = if let Some(strict_if) = scope_args.strict_if {
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
                let mut enter = cond.as_ref().into_token_stream();
                enter.extend(maybe_conditional(quote!(visitor.enter_scope(#flags, &it.scope_id);)));
                let leave = maybe_conditional(quote!(visitor.leave_scope();));
                (enter, leave)
            });

        let node_events = if KIND_BLACK_LIST.contains(&ident.to_string().as_str()) {
            (
                insert!(
                    "// NOTE: {} doesn't exists!",
                    if self.is_mut { "AstType" } else { "AstKind" }
                ),
                TokenStream::default(),
            )
        } else {
            let kind = self.kind_type(ident);
            (
                quote! {
                    let kind = #kind;
                    visitor.enter_node(kind);
                },
                quote!(visitor.leave_node(kind);),
            )
        };

        let mut enter_scope_at = 0;
        let mut enter_node_at = 0;
        let fields_visits: Vec<TokenStream> = struct_
            .item
            .fields
            .iter()
            .enumerate()
            .map(|(ix, it)| (ix, it, get_visit_markers(&it.attrs).transpose()))
            .filter_map(|(ix, it, markers)| {
                let ty_res = it.ty.analyze(self.ctx);
                let typ = ty_res.type_ref?;
                if !typ.borrow().visitable() {
                    return None;
                }
                let typ_wrapper = ty_res.wrapper;
                let markers = markers.unwrap();
                let visit_as = markers.as_ref().and_then(|mk| mk.visit_as.clone());
                let visit_args = markers.and_then(|mk| mk.visit_args);

                let have_enter_scope = get_scope_markers(&it.attrs)
                    .is_some_and(|it| matches!(it, Ok(ScopeMarkers { enter_before: true })));
                let have_enter_node = get_visit_markers(&it.attrs)
                    .is_some_and(|it| matches!(it, Ok(VisitMarkers { enter_before: true, .. })));

                let (args_def, args) = visit_args
                    .map(|it| it.into_iter().fold((Vec::new(), Vec::new()), Self::visit_args_fold))
                    .unwrap_or_default();
                let visit = self.get_visitor(
                    &typ,
                    matches!(
                        typ_wrapper,
                        TypeWrapper::Vec | TypeWrapper::VecBox | TypeWrapper::OptVec
                    ),
                    visit_as.as_ref(),
                );
                let name = it.ident.as_ref().expect("expected named fields!");
                let borrowed_field = self.with_ref_pat(quote!(it.#name));
                let mut result = match typ_wrapper {
                    TypeWrapper::Opt | TypeWrapper::OptBox | TypeWrapper::OptVec => quote! {
                        if let Some(#name) = #borrowed_field {
                            visitor.#visit(#name #(#args)*);
                        }
                    },
                    TypeWrapper::VecOpt => {
                        let iter = self.get_iter();
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

                #[allow(unreachable_code)]
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

        let with_scope_events = |body: TokenStream| match (scope_events, enter_scope_at) {
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
