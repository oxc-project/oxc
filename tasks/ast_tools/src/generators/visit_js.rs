//! Generator for `VisitJs` and `VisitJsMut` traits.
//!
//! `VisitJs` and `VisitJsMut` are variants of [`Visit`] and [`VisitMut`] which traverse only the
//! JavaScript parts of the AST. They skip pure TypeScript type-space nodes (`TSType`,
//! `TSTypeAnnotation`, interfaces, type aliases, ...) entirely, but still descend into the
//! JavaScript nested inside a small set of TS wrapper nodes (`x as T`, decorators, enum
//! initializers, namespace bodies, `export = expr`, `import x = require(..)`).
//!
//! These visitors are for visiting the JavaScript parts of a *TypeScript* AST — they are not for
//! ASTs where TypeScript has already been transformed out. The TS constructs carrying runtime
//! JavaScript must therefore still be walked, so their walk code remains in the binary; only the
//! pure type grammar is pruned.
//!
//! Because the generated `walk_*` functions in `mod walk_js` and `mod walk_js_mut` never reference
//! the pure-type `walk_ts_*` functions, an `impl VisitJs` or `impl VisitJsMut` monomorphizes zero
//! TS type-grammar traversal. This removes the TS-grammar walks (~23% of oxlint's visitor
//! machinery) from the binary contribution of any visitor that only inspects runtime JavaScript.
//!
//! [`Visit`]: super::visit
//! [`VisitMut`]: super::visit

use cow_utils::CowUtils;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, parse_str};

use crate::{
    AST_VISIT_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VecDef},
    utils::{create_ident, create_ident_tokens},
};

use super::{
    define_generator,
    visit::{
        INLINE_LIMIT, Target, VisitorOutputs, generate_enter_and_leave_node, generate_visit_type,
    },
};

/// TypeScript wrapper nodes (defined in `crates/oxc_ast/src/ast/ts.rs`) which carry runtime
/// JavaScript and must still be walked by `VisitJs`.
///
/// Every other type defined in `ts.rs` is pruned by `VisitJs`. This list must be transitively
/// complete: every TS node on a path from JS-space down to real JS has to be present, including
/// intermediate enums/structs, otherwise the JavaScript below a missing entry is silently dropped
/// (the equivalence test in `crates/oxc_ast_visit/tests` guards against that).
///
/// This is an explicit allowlist rather than an automatically-computed predicate on purpose:
/// TS nodes such as `TSLiteralType` (`type X = 1`) or `TSInterfaceDeclaration` (`interface Foo {}`)
/// structurally reach JavaScript literal/identifier nodes through *type* space, so a "reaches a JS
/// node" analysis would wrongly keep the whole of `TSType` and reintroduce exactly the bloat this
/// trait removes. `TSEnumMemberName` (kept) and `TSLiteral` (pruned) are structurally identical
/// TS enums of JS literal variants — only intent separates them.
const VISITED_TS_NODES: &[&str] = &[
    // Expression casts / assertions — `.expression: Expression`
    "TSAsExpression",
    "TSSatisfiesExpression",
    "TSTypeAssertion",
    "TSNonNullExpression",
    "TSInstantiationExpression",
    // Decorators — `.expression: Expression`
    "Decorator",
    // `export = expr` — `.expression: Expression`
    "TSExportAssignment",
    // `import x = require('m')` / `import A = B.C` — value aliases with runtime semantics,
    // the import counterpart of `export =`. (The type-only form is distinguished by
    // `import_kind`, not grammar.) `TSTypeName`/`TSQualifiedName` are needed here as the
    // `B.C` reference chain; they are unreachable from any other visited node.
    "TSImportEqualsDeclaration",
    "TSModuleReference",
    "TSExternalModuleReference",
    "TSTypeName",
    "TSQualifiedName",
    // Enums — member `initializer: Option<Expression>` + computed member names
    "TSEnumDeclaration",
    "TSEnumBody",
    "TSEnumMember",
    "TSEnumMemberName",
    // Namespaces / modules — `body: Vec<Statement>` + directives
    "TSModuleDeclaration",
    "TSModuleDeclarationName",
    "TSModuleDeclarationBody",
    "TSModuleBlock",
    "TSGlobalDeclaration",
];

/// Generator for `VisitJs` trait.
pub struct VisitJsGenerator;

define_generator!(VisitJsGenerator);

impl Generator for VisitJsGenerator {
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let (visit_js, visit_js_mut) = generate_outputs(schema);
        vec![
            Output::Rust {
                path: output_path(AST_VISIT_CRATE_PATH, "visit_js.rs"),
                tokens: visit_js,
            },
            Output::Rust {
                path: output_path(AST_VISIT_CRATE_PATH, "visit_js_mut.rs"),
                tokens: visit_js_mut,
            },
        ]
    }
}

/// Returns `true` if `VisitJs` visits `type_def`.
///
/// A type is visited if it has a visitor and is either a JavaScript node (not defined in
/// `::ast::ts`) or one of the explicitly-allowlisted TS nodes carrying runtime JavaScript.
fn type_is_visited(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(struct_def) => struct_is_visited(struct_def, schema),
        TypeDef::Enum(enum_def) => enum_is_visited(enum_def, schema),
        _ => false,
    }
}

/// Returns `true` if `VisitJs` visits this struct.
fn struct_is_visited(struct_def: &StructDef, schema: &Schema) -> bool {
    struct_def.visit.has_visitor()
        && is_js_node(struct_def.name(), struct_def.file(schema).import_path())
}

/// Returns `true` if `VisitJs` visits this enum.
fn enum_is_visited(enum_def: &EnumDef, schema: &Schema) -> bool {
    enum_def.visit.has_visitor() && is_js_node(enum_def.name(), enum_def.file(schema).import_path())
}

/// A node is visited by `VisitJs` if it's not a TypeScript node (not in `::ast::ts`), or it's an
/// allowlisted TS node carrying runtime JavaScript.
fn is_js_node(name: &str, import_path: &str) -> bool {
    import_path != "::ast::ts" || VISITED_TS_NODES.contains(&name)
}

/// Returns `true` if the innermost type of a field / enum variant is visited by `VisitJs`.
fn innermost_is_visited(type_def: &TypeDef, schema: &Schema) -> bool {
    type_is_visited(type_def.innermost_type(schema), schema)
}

/// [`VisitorOutputs`] for generating visitor calls for `VisitJs` and `VisitJsMut`.
struct JsVisitAndMut {
    visit: TokenStream,
    visit_mut: TokenStream,
}

impl VisitorOutputs for JsVisitAndMut {
    fn gen_each<F: Fn(bool) -> TokenStream>(f: F) -> Self {
        Self { visit: f(false), visit_mut: f(true) }
    }

    fn map<F: Fn(TokenStream, bool) -> TokenStream>(self, f: F) -> Self {
        Self { visit: f(self.visit, false), visit_mut: f(self.visit_mut, true) }
    }
}

/// Generate `VisitJs` / `walk_js` and `VisitJsMut` / `walk_js_mut`.
fn generate_outputs(schema: &Schema) -> (TokenStream, TokenStream) {
    let mut visit_methods = TokenStream::new();
    let mut walk_fns = TokenStream::new();
    let mut visit_mut_methods = TokenStream::new();
    let mut walk_mut_fns = TokenStream::new();

    for type_def in &schema.types {
        match type_def {
            TypeDef::Struct(struct_def) => {
                generate_struct_visitor(
                    struct_def,
                    schema,
                    &mut visit_methods,
                    &mut walk_fns,
                    &mut visit_mut_methods,
                    &mut walk_mut_fns,
                );
            }
            TypeDef::Enum(enum_def) => {
                generate_enum_visitor(
                    enum_def,
                    schema,
                    &mut visit_methods,
                    &mut walk_fns,
                    &mut visit_mut_methods,
                    &mut walk_mut_fns,
                );
            }
            TypeDef::Vec(vec_def) => {
                generate_vec_visitor(
                    vec_def,
                    schema,
                    &mut visit_methods,
                    &mut walk_fns,
                    &mut visit_mut_methods,
                    &mut walk_mut_fns,
                );
            }
            _ => {}
        }
    }

    let visit_js = quote! {
        //! JavaScript-only visitor
        //!
        //! `VisitJs` traverses only the JavaScript parts of the AST, skipping TypeScript type-space
        //! nodes. See [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html).
        //!
        //! This visitor is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
        //! where TypeScript has already been transformed out. TS constructs carrying runtime
        //! JavaScript (enum initializers, namespace bodies, decorators, `x as T` casts,
        //! `export =`, `import x = require(..)`) are still walked, so their walk code remains
        //! in the binary; only the pure type grammar is pruned.

        //!@@line_break
        #![expect(unused_variables)]
        #![allow(
            clippy::match_same_arms,
            clippy::semicolon_if_nothing_returned,
            clippy::needless_pass_by_ref_mut,
            clippy::trivially_copy_pass_by_ref,
            clippy::match_wildcard_for_single_variants,
            clippy::single_match_else,
        )]

        ///@@line_break
        use std::cell::Cell;

        ///@@line_break
        use oxc_allocator::ArenaVec;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        ///@@line_break
        use oxc_ast::ast::*;
        use oxc_ast::ast_kind::AstKind;

        ///@@line_break
        use walk_js::*;

        ///@@line_break
        /// JavaScript-only syntax tree traversal.
        ///
        /// Like [`Visit`], but skips TypeScript type-space nodes. Still descends into JavaScript
        /// nested inside TS wrapper nodes (`x as T`, decorators, enum initializers, namespace
        /// bodies, `export = expr`, `import x = require(..)`).
        ///
        /// This trait is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
        /// where TypeScript has already been transformed out. The walks for those JS-carrying
        /// TS nodes stay in the binary; only the pure type grammar is pruned.
        ///
        /// Pruning is by grammar (node kind), not by TypeScript's erasure semantics: type-only
        /// imports/exports (`import type`, `export type`) and `declare`d items are JS-grammar
        /// nodes and are still visited — filter on `import_kind` / `declare` in the visitor if
        /// needed, exactly as with [`Visit`].
        ///
        /// [`Visit`]: crate::Visit
        pub trait VisitJs<'a>: Sized {
            #[inline]
            fn enter_node(&mut self, kind: AstKind<'a>) {}
            #[inline]
            fn leave_node(&mut self, kind: AstKind<'a>) {}

            ///@@line_break
            #[inline]
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
            #[inline]
            fn leave_scope(&mut self) {}

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

            #visit_methods
        }

        ///@@line_break
        pub mod walk_js {
            use super::*;

            ///@@line_break
            #walk_fns
        }
    };

    let visit_js_mut = quote! {
        //! JavaScript-only mutable visitor
        //!
        //! `VisitJsMut` traverses only the JavaScript parts of the AST, skipping TypeScript
        //! type-space nodes. See [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html).
        //!
        //! This visitor is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
        //! where TypeScript has already been transformed out. TS constructs carrying runtime
        //! JavaScript (enum initializers, namespace bodies, decorators, `x as T` casts,
        //! `export =`, `import x = require(..)`) are still walked, so their walk code remains
        //! in the binary; only the pure type grammar is pruned.

        //!@@line_break
        #![expect(unused_variables)]
        #![allow(
            clippy::match_same_arms,
            clippy::semicolon_if_nothing_returned,
            clippy::needless_pass_by_ref_mut,
            clippy::trivially_copy_pass_by_ref,
            clippy::match_wildcard_for_single_variants,
            clippy::single_match_else,
        )]

        ///@@line_break
        use std::cell::Cell;

        ///@@line_break
        use oxc_allocator::ArenaVec;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        ///@@line_break
        use oxc_ast::ast::*;
        use oxc_ast::ast_kind::AstType;

        ///@@line_break
        use walk_js_mut::*;

        ///@@line_break
        /// JavaScript-only mutable syntax tree traversal.
        ///
        /// Like [`VisitMut`], but skips TypeScript type-space nodes. Still descends into JavaScript
        /// nested inside TS wrapper nodes (`x as T`, decorators, enum initializers, namespace
        /// bodies, `export = expr`, `import x = require(..)`).
        ///
        /// This trait is for visiting the JavaScript parts of a TypeScript AST — not for ASTs
        /// where TypeScript has already been transformed out. The walks for those JS-carrying
        /// TS nodes stay in the binary; only the pure type grammar is pruned.
        ///
        /// Pruning is by grammar (node kind), not by TypeScript's erasure semantics: type-only
        /// imports/exports (`import type`, `export type`) and `declare`d items are JS-grammar
        /// nodes and are still visited — filter on `import_kind` / `declare` in the visitor if
        /// needed, exactly as with [`VisitMut`].
        ///
        /// [`VisitMut`]: crate::VisitMut
        pub trait VisitJsMut<'a>: Sized {
            #[inline]
            fn enter_node(&mut self, kind: AstType) {}
            #[inline]
            fn leave_node(&mut self, kind: AstType) {}

            ///@@line_break
            #[inline]
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
            #[inline]
            fn leave_scope(&mut self) {}

            #visit_mut_methods
        }

        ///@@line_break
        pub mod walk_js_mut {
            use super::*;

            ///@@line_break
            #walk_mut_fns
        }
    };

    (visit_js, visit_js_mut)
}

/// Generate `visit_*` method and `walk_*` function for a struct.
fn generate_struct_visitor(
    struct_def: &StructDef,
    schema: &Schema,
    visit_methods: &mut TokenStream,
    walk_fns: &mut TokenStream,
    visit_mut_methods: &mut TokenStream,
    walk_mut_fns: &mut TokenStream,
) {
    // Exit if this struct is not visited by `VisitJs` (not visited at all, or a pure-type TS node)
    if !struct_is_visited(struct_def, schema) {
        return;
    }
    let Some(visitor_names) = &struct_def.visit.visitor_names else { return };

    let struct_ty = struct_def.ty(schema);
    let visit_fn_ident = visitor_names.visitor_ident();
    let walk_fn_ident = visitor_names.walk_ident();

    // Get additional params (e.g. `flags: ScopeFlags`)
    let (extra_params, extra_args): (TokenStream, TokenStream) = struct_def
        .visit
        .visit_args
        .iter()
        .map(|(arg_name, arg_type_name)| {
            let param_ident = create_ident(arg_name);
            let arg_type_ident = create_ident(arg_type_name);
            (quote!( , #param_ident: #arg_type_ident ), quote!( , #param_ident ))
        })
        .unzip();

    let gen_visit_fn = |reference| {
        quote! {
            ///@@line_break
            #[inline]
            fn #visit_fn_ident(&mut self, it: #reference #struct_ty #extra_params) {
                #walk_fn_ident(self, it #extra_args);
            }
        }
    };
    visit_methods.extend(gen_visit_fn(quote!(&)));
    visit_mut_methods.extend(gen_visit_fn(quote!(&mut)));

    // Generate `enter_node` / `leave_node` calls (if this struct has an `AstKind` / `AstType`)
    let struct_ident = struct_def.ident();
    let has_kind = struct_def.kind.has_kind;
    let (enter_node, leave_node) = generate_enter_and_leave_node(&struct_ident, has_kind, false);
    let (enter_node_mut, leave_node_mut) =
        generate_enter_and_leave_node(&struct_ident, has_kind, true);

    // Generate `enter_scope` / `leave_scope` calls (if this struct has a scope)
    let (mut scope_entry, mut scope_exit) = if let Some(scope) = &struct_def.visit.scope {
        let mut flags = parse_str::<Expr>(&scope.flags).unwrap().to_token_stream();
        if let Some(strict_if) = &scope.strict_if {
            let strict_if = parse_str::<Expr>(&strict_if.cow_replace("self", "it")).unwrap();
            flags = quote! {{
                let mut flags = #flags;
                if #strict_if {
                    flags |= ScopeFlags::StrictMode;
                }
                flags
            }}
        }
        let enter_scope = quote!( visitor.enter_scope(#flags, &it.scope_id); );
        let leave_scope = quote!( visitor.leave_scope(); );
        (
            Some((scope.enter_before_index, enter_scope)),
            Some((scope.exit_before_index, leave_scope)),
        )
    } else {
        (None, None)
    };

    // Generate `visit_*` calls for struct fields, skipping TS-typed fields
    let mut field_visits_count = 0usize;
    let mut field_visits = TokenStream::new();
    let mut field_visits_mut = TokenStream::new();
    for (field_index, field) in struct_def.fields.iter().enumerate() {
        if let Some((visit, visit_mut)) = generate_struct_field_visit(
            field,
            field_index,
            &mut scope_entry,
            &mut scope_exit,
            schema,
        ) {
            field_visits.extend(visit);
            field_visits_mut.extend(visit_mut);
            field_visits_count += 1;
        }
    }

    // If didn't enter or exit scope already, enter/exit after last field
    if let Some((_, enter_scope)) = scope_entry {
        field_visits.extend(enter_scope.clone());
        field_visits_mut.extend(enter_scope);
    }
    if let Some((_, leave_scope)) = scope_exit {
        field_visits.extend(leave_scope.clone());
        field_visits_mut.extend(leave_scope);
    }

    let maybe_inline_attr =
        if field_visits_count <= INLINE_LIMIT { quote!( #[inline] ) } else { quote!() };

    walk_fns.extend(quote! {
        ///@@line_break
        #maybe_inline_attr
        pub fn #walk_fn_ident<'a, V: VisitJs<'a>>(visitor: &mut V, it: &#struct_ty #extra_params) {
            #enter_node
            #field_visits
            #leave_node
        }
    });
    walk_mut_fns.extend(quote! {
        ///@@line_break
        #maybe_inline_attr
        pub fn #walk_fn_ident<'a, V: VisitJsMut<'a>>(
            visitor: &mut V,
            it: &mut #struct_ty
            #extra_params,
        ) {
            #enter_node_mut
            #field_visits_mut
            #leave_node_mut
        }
    });
}

/// Generate visitor call for a struct field, or `None` if the field is not visited by `VisitJs`.
///
/// Also inserts `enter_scope` / `leave_scope` calls before the visit call if needed.
fn generate_struct_field_visit(
    field: &FieldDef,
    field_index: usize,
    scope_entry: &mut Option<(usize, TokenStream)>,
    scope_exit: &mut Option<(usize, TokenStream)>,
    schema: &Schema,
) -> Option<(TokenStream, TokenStream)> {
    let field_type = field.type_def(schema);

    // Skip fields whose innermost type is a TS type node `VisitJs` doesn't visit
    // (e.g. `return_type`, `type_parameters`, `type_annotation`). `Vec<Decorator>` is kept,
    // because `Decorator` is visited.
    if !innermost_is_visited(field_type, schema) {
        return None;
    }

    let field_ident = field.ident();
    let JsVisitAndMut { mut visit, mut visit_mut } = generate_visit_type(
        field_type,
        &Target::Property(quote!( it.#field_ident )),
        &field.visit.visit_args,
        &field_ident,
        &quote!(visitor),
        true,
        schema,
    )?;

    // Insert `leave_scope` / `enter_scope` before this field if scope boundary falls here.
    // Handle exit first, so entering and exiting on the same field produces `enter` before `leave`.
    if let Some((exit_index, _)) = scope_exit
        && *exit_index <= field_index
    {
        let (_, leave_scope) = scope_exit.take().unwrap();
        visit = quote!( #leave_scope #visit );
        visit_mut = quote!( #leave_scope #visit_mut );
    }

    if let Some((enter_index, _)) = scope_entry
        && *enter_index <= field_index
    {
        let (_, enter_scope) = scope_entry.take().unwrap();
        visit = quote!( #enter_scope #visit );
        visit_mut = quote!( #enter_scope #visit_mut );
    }

    Some((visit, visit_mut))
}

/// Generate `visit_*` method and `walk_*` function for an enum.
fn generate_enum_visitor(
    enum_def: &EnumDef,
    schema: &Schema,
    visit_methods: &mut TokenStream,
    walk_fns: &mut TokenStream,
    visit_mut_methods: &mut TokenStream,
    walk_mut_fns: &mut TokenStream,
) {
    // Exit if this enum is not visited by `VisitJs`
    if !enum_is_visited(enum_def, schema) {
        return;
    }
    let Some(visitor_names) = &enum_def.visit.visitor_names else { return };

    let enum_ty = enum_def.ty(schema);
    let visit_fn_ident = visitor_names.visitor_ident();
    let walk_fn_ident = visitor_names.walk_ident();

    let gen_visit_fn = |reference| {
        quote! {
            ///@@line_break
            #[inline]
            fn #visit_fn_ident(&mut self, it: #reference #enum_ty) {
                #walk_fn_ident(self, it);
            }
        }
    };
    visit_methods.extend(gen_visit_fn(quote!(&)));
    visit_mut_methods.extend(gen_visit_fn(quote!(&mut)));

    let enum_ident = enum_def.ident();
    let (enter_node, leave_node) = generate_enter_and_leave_node(&enum_ident, false, false);
    let (enter_node_mut, leave_node_mut) = generate_enter_and_leave_node(&enum_ident, false, true);

    let mut match_arm_count = 0usize;

    // Own variants, skipping those whose inner type is a TS type node
    let (variant_match_arms, variant_match_arms_mut): (TokenStream, TokenStream) = enum_def
        .variants
        .iter()
        .filter_map(|variant| {
            let variant_type = variant.field_type(schema)?;
            if !innermost_is_visited(variant_type, schema) {
                return None;
            }
            let JsVisitAndMut { visit, visit_mut } = generate_visit_type(
                variant_type,
                &Target::Reference(create_ident_tokens("it")),
                &variant.visit.visit_args,
                &create_ident_tokens("it"),
                &quote!(visitor),
                false,
                schema,
            )?;

            match_arm_count += 1;

            let variant_ident = variant.ident();
            let match_pattern = quote!( #enum_ident::#variant_ident(it) );
            Some((quote!( #match_pattern => #visit, ), quote!( #match_pattern => #visit_mut, )))
        })
        .unzip();

    // Inherited enums, skipping any which `VisitJs` doesn't visit
    let (inherits_match_arms, inherits_match_arms_mut): (TokenStream, TokenStream) = enum_def
        .inherits_enums(schema)
        .filter_map(|inherits_type| {
            let inner_visit_fn_ident = inherits_type.visit.visitor_ident()?;
            if !enum_is_visited(inherits_type, schema) {
                return None;
            }

            match_arm_count += 1;

            let inherits_snake_name = inherits_type.snake_name();
            let match_ident = format_ident!("match_{inherits_snake_name}");
            let to_fn_ident = format_ident!("to_{inherits_snake_name}");
            let to_fn_ident_mut = format_ident!("to_{inherits_snake_name}_mut");
            Some((
                quote! {
                    #match_ident!(#enum_ident) => visitor.#inner_visit_fn_ident(it.#to_fn_ident()),
                },
                quote! {
                    #match_ident!(#enum_ident) => visitor.#inner_visit_fn_ident(it.#to_fn_ident_mut()),
                },
            ))
        })
        .unzip();

    // Add catch-all match arm if not all variants are visited
    let catch_all_match_arm = if match_arm_count < enum_def.variants.len() + enum_def.inherits.len()
    {
        quote!( _ => {} )
    } else {
        quote!()
    };

    let maybe_inline_attr =
        if match_arm_count <= INLINE_LIMIT { quote!( #[inline] ) } else { quote!() };

    walk_fns.extend(quote! {
        ///@@line_break
        #maybe_inline_attr
        pub fn #walk_fn_ident<'a, V: VisitJs<'a>>(visitor: &mut V, it: &#enum_ty) {
            #enter_node
            match it {
                #variant_match_arms
                #inherits_match_arms
                #catch_all_match_arm
            }
            #leave_node
        }
    });
    walk_mut_fns.extend(quote! {
        ///@@line_break
        #maybe_inline_attr
        pub fn #walk_fn_ident<'a, V: VisitJsMut<'a>>(visitor: &mut V, it: &mut #enum_ty) {
            #enter_node_mut
            match it {
                #variant_match_arms_mut
                #inherits_match_arms_mut
                #catch_all_match_arm
            }
            #leave_node_mut
        }
    });
}

/// Generate `visit_*` method and `walk_*` function for a `Vec`.
fn generate_vec_visitor(
    vec_def: &VecDef,
    schema: &Schema,
    visit_methods: &mut TokenStream,
    walk_fns: &mut TokenStream,
    visit_mut_methods: &mut TokenStream,
    walk_mut_fns: &mut TokenStream,
) {
    // Exit if this `Vec` does not have its own visitor, or its element type is not visited by `VisitJs`
    let Some(visitor_names) = &vec_def.visit.visitor_names else { return };
    let inner_type = vec_def.inner_type(schema);
    if !type_is_visited(inner_type, schema) {
        return;
    }

    let vec_ty = vec_def.ty(schema);
    let visit_fn_ident = visitor_names.visitor_ident();
    let walk_fn_ident = visitor_names.walk_ident();

    let gen_visit_fn = |reference| {
        quote! {
            ///@@line_break
            #[inline]
            fn #visit_fn_ident(&mut self, it: #reference #vec_ty) {
                #walk_fn_ident(self, it);
            }
        }
    };
    visit_methods.extend(gen_visit_fn(quote!(&)));
    visit_mut_methods.extend(gen_visit_fn(quote!(&mut)));

    let inner_visit_fn_ident = match inner_type {
        TypeDef::Struct(struct_def) => struct_def.visit.visitor_ident().unwrap(),
        TypeDef::Enum(enum_def) => enum_def.visit.visitor_ident().unwrap(),
        _ => unreachable!(),
    };

    // Same `Vec<Argument>` fast-path as `Visit` to reduce stack usage: `Argument` inherits from
    // `Expression`, so dispatch non-spread elements straight to the expression visitor.
    let is_arguments_vec =
        matches!(inner_type, TypeDef::Enum(enum_def) if enum_def.name() == "Argument");

    let gen_loop_body = |is_mut| {
        if is_arguments_vec {
            let to_expression = if is_mut {
                format_ident!("to_expression_mut")
            } else {
                format_ident!("to_expression")
            };
            quote! {
                for el in it {
                    match el {
                        oxc_ast::ast::Argument::SpreadElement(spread) => {
                            visitor.visit_spread_element(spread);
                        }
                        _ => {
                            visitor.visit_expression(el.#to_expression());
                        }
                    }
                }
            }
        } else {
            quote! {
                for el in it {
                    visitor.#inner_visit_fn_ident(el);
                }
            }
        }
    };
    let loop_body = gen_loop_body(false);
    let loop_body_mut = gen_loop_body(true);

    walk_fns.extend(quote! {
        ///@@line_break
        #[inline]
        pub fn #walk_fn_ident<'a, V: VisitJs<'a>>(visitor: &mut V, it: &#vec_ty) {
            #loop_body
        }
    });
    walk_mut_fns.extend(quote! {
        ///@@line_break
        #[inline]
        pub fn #walk_fn_ident<'a, V: VisitJsMut<'a>>(visitor: &mut V, it: &mut #vec_ty) {
            #loop_body_mut
        }
    });
}
