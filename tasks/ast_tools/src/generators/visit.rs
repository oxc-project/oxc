//! Generator for `Visit` and `VisitMut` traits.

use cow_utils::CowUtils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_str, Expr, Ident};

use crate::{
    output::{output_path, Output},
    schema::{
        extensions::visit::{Scope, VisitorNames},
        Def, EnumDef, FieldDef, OptionDef, Schema, StructDef, TypeDef, VecDef,
    },
    utils::{create_ident, create_ident_tokens, create_safe_ident},
    Codegen, Generator, Result, AST_CRATE_PATH,
};

use super::{attr_positions, define_generator, AttrLocation, AttrPart, AttrPositions};

/// Generator for `Visit` and `VisitMut` traits.
pub struct VisitGenerator;

define_generator!(VisitGenerator);

impl Generator for VisitGenerator {
    /// Register that accept:
    /// * `#[visit]` attr on structs, struct fields, or enum variants.
    /// * `#[ast(visit)]` on structs or enums.
    /// * `#[scope]` on structs or struct fields.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[
            ("visit", attr_positions!(AstAttr | Struct | StructField | EnumVariant)),
            ("scope", attr_positions!(Struct | StructField)),
        ]
    }

    /// Parse `#[visit]`, `#[scope]` and `#[ast(visit)]` attrs.
    fn parse_attr(&self, attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        match attr_name {
            "visit" => parse_visit_attr(location, part),
            "scope" => parse_scope_attr(location, part),
            _ => unreachable!(),
        }
    }

    /// Create names for `visit_*` methods and `walk_*` functions for all `Vec`s
    /// whose inner type has a visitor.
    fn prepare(&self, schema: &mut Schema) {
        for type_id in schema.types.indices() {
            let Some(vec_def) = schema.types[type_id].as_vec() else { continue };

            let inner_type = vec_def.inner_type(schema);
            let plural_snake_name = match inner_type {
                TypeDef::Struct(struct_def) => {
                    if !struct_def.visit.has_visitor() {
                        continue;
                    }
                    struct_def.plural_snake_name()
                }
                TypeDef::Enum(enum_def) => {
                    if !enum_def.visit.has_visitor() {
                        continue;
                    }
                    enum_def.plural_snake_name()
                }
                _ => continue,
            };

            let visitor_names = VisitorNames::from_snake_name(&plural_snake_name);
            schema.vec_def_mut(type_id).visit.visitor_names = Some(visitor_names);
        }
    }

    /// Generate `Visit` and `VisitMut` traits.
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let (visit_output, visit_mut_output) = generate_outputs(schema);

        let visit_output =
            Output::Rust { path: output_path(AST_CRATE_PATH, "visit.rs"), tokens: visit_output };
        let visit_mut_output = Output::Rust {
            path: output_path(AST_CRATE_PATH, "visit_mut.rs"),
            tokens: visit_mut_output,
        };

        vec![visit_output, visit_mut_output]
    }
}

/// Parse `#[visit]` or `#[ast(visit)]` attr.
fn parse_visit_attr(location: AttrLocation, part: AttrPart) -> Result<()> {
    match (part, location) {
        // `#[ast(visit)]` on struct
        (AttrPart::None, AttrLocation::StructAstAttr(struct_def)) => {
            struct_def.visit.visitor_names =
                Some(VisitorNames::from_snake_name(&struct_def.snake_name()));
        }
        // `#[ast(visit)]` on enum
        (AttrPart::None, AttrLocation::EnumAstAttr(enum_def)) => {
            enum_def.visit.visitor_names =
                Some(VisitorNames::from_snake_name(&enum_def.snake_name()));
        }
        // `#[visit(args(flags = ...))]` on struct field or enum variant
        (AttrPart::List("args", list), location) => {
            let existing_args = match location {
                AttrLocation::Struct(struct_def) => &mut struct_def.visit.visit_args,
                AttrLocation::StructField(struct_def, field_index) => {
                    &mut struct_def.fields[field_index].visit.visit_args
                }
                AttrLocation::EnumVariant(enum_def, variant_index) => {
                    &mut enum_def.variants[variant_index].visit.visit_args
                }
                _ => return Err(()),
            };

            for list_element in list {
                let (name, value) = list_element.try_into_string()?;
                existing_args.push((name, value));
            }
        }
        _ => return Err(()),
    }

    Ok(())
}

/// Parse `#[scope]` attr.
fn parse_scope_attr(location: AttrLocation, part: AttrPart) -> Result<()> {
    fn get_or_create_scope(struct_def: &mut StructDef) -> Result<&mut Scope> {
        if !struct_def.visit.has_visitor() {
            return Err(());
        }

        Ok(struct_def.visit.scope.get_or_insert_with(|| Scope {
            enter_before_index: 0,
            exit_before_index: struct_def.fields.len(),
            flags: "ScopeFlags::empty()".to_string(),
            strict_if: None,
        }))
    }

    match (part, location) {
        // `#[scope]` on struct
        (AttrPart::None, AttrLocation::Struct(struct_def)) => {
            get_or_create_scope(struct_def)?;
        }
        // `#[scope(flags = ...)` on struct
        (AttrPart::String("flags", value), AttrLocation::Struct(struct_def)) => {
            let scope = get_or_create_scope(struct_def)?;
            scope.flags = value;
        }
        // `#[scope(strict_if = ...)` on struct
        (AttrPart::String("strict_if", value), AttrLocation::Struct(struct_def)) => {
            let scope = get_or_create_scope(struct_def)?;
            scope.strict_if = Some(value);
        }
        // `#[scope(enter_before)]` on struct field
        (AttrPart::Tag("enter_before"), AttrLocation::StructField(struct_def, field_index)) => {
            let scope = struct_def.visit.scope.as_mut().ok_or(())?;
            scope.enter_before_index = field_index;
        }
        // `#[scope(exit_before)]` on struct field
        (AttrPart::Tag("exit_before"), AttrLocation::StructField(struct_def, field_index)) => {
            let scope = struct_def.visit.scope.as_mut().ok_or(())?;
            scope.exit_before_index = field_index;
        }
        _ => return Err(()),
    }

    Ok(())
}

/// Generate outputs for `Visit` and `VisitMut`.
fn generate_outputs(schema: &Schema) -> (/* Visit */ TokenStream, /* VisitMut */ TokenStream) {
    // Generate `visit_*` methods and `walk_*` functions for both `Visit` and `VisitMut`
    let mut builder = VisitBuilder::new(schema);
    builder.generate();
    let VisitBuilder { visit_methods, walk_fns, visit_mut_methods, walk_mut_fns, .. } = builder;

    // Generate `Visit` trait
    let alloc_fn = quote! {
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
    };
    let visit_output = generate_output(
        &create_safe_ident("Visit"),
        &visit_methods,
        &walk_fns,
        &create_safe_ident("walk"),
        &alloc_fn,
        &create_safe_ident("AstKind"),
        &quote!(AstKind<'a>),
    );

    // Generate `VisitMut` trait
    let visit_mut_output = generate_output(
        &create_safe_ident("VisitMut"),
        &visit_mut_methods,
        &walk_mut_fns,
        &create_safe_ident("walk_mut"),
        &quote!(),
        &create_safe_ident("AstType"),
        &quote!(AstType),
    );

    (visit_output, visit_mut_output)
}

/// Generate output for `Visit` or `VisitMut` trait.
fn generate_output(
    trait_ident: &Ident,
    visit_methods: &TokenStream,
    walk_fns: &TokenStream,
    walk_mod_ident: &Ident,
    maybe_alloc: &TokenStream,
    ast_kind_or_type_ident: &Ident,
    ast_kind_or_type_ty: &TokenStream,
) -> TokenStream {
    quote! {
        //! Visitor Pattern
        //!
        //! See:
        //! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
        //! * [rustc visitor](https://github.com/rust-lang/rust/blob/1.82.0/compiler/rustc_ast/src/visit.rs)

        //!@@line_break
        #![expect(unused_variables, clippy::semicolon_if_nothing_returned)]

        ///@@line_break
        use std::cell::Cell;

        ///@@line_break
        use oxc_allocator::Vec;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        ///@@line_break
        use crate::ast::*;
        use crate::ast_kind::#ast_kind_or_type_ident;

        ///@@line_break
        use #walk_mod_ident::*;

        ///@@line_break
        /// Syntax tree traversal
        pub trait #trait_ident<'a>: Sized {
            #[inline]
            fn enter_node(&mut self, kind: #ast_kind_or_type_ty) {}
            #[inline]
            fn leave_node(&mut self, kind: #ast_kind_or_type_ty) {}

            ///@@line_break
            #[inline]
            fn enter_scope(&mut self, flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {}
            #[inline]
            fn leave_scope(&mut self) {}

            #maybe_alloc

            #visit_methods
        }

        ///@@line_break
        pub mod #walk_mod_ident {
            use super::*;

            ///@@line_break
            #walk_fns
        }
    }
}

/// Generator of `visit_*` methods and `walk_*` functions for `Visit` and `VisitMut`.
struct VisitBuilder<'s> {
    schema: &'s Schema,
    /// `visit_*` methods for `Visit`
    visit_methods: TokenStream,
    /// `visit_*` methods for `VisitMut`
    visit_mut_methods: TokenStream,
    /// `walk_*` functions for `Visit`
    walk_fns: TokenStream,
    /// `walk_*` functions for `VisitMut`
    walk_mut_fns: TokenStream,
}

impl<'s> VisitBuilder<'s> {
    /// Create new [`VisitBuilder`].
    fn new(schema: &'s Schema) -> Self {
        Self {
            schema,
            visit_methods: quote!(),
            walk_fns: quote!(),
            visit_mut_methods: quote!(),
            walk_mut_fns: quote!(),
        }
    }

    /// Generate `visit_*` methods and `walk_*` functions for `Visit` and `VisitMut`.
    ///
    /// After calling this method, [`VisitBuilder`] contains all `visit_*` methods and `walk_*` functions
    /// in `visit_methods` etc fields.
    fn generate(&mut self) {
        for type_def in &self.schema.types {
            match type_def {
                TypeDef::Struct(struct_def) => self.generate_struct_visitor(struct_def),
                TypeDef::Enum(enum_def) => self.generate_enum_visitor(enum_def),
                TypeDef::Vec(vec_def) => self.generate_vec_visitor(vec_def),
                _ => {}
            }
        }
    }
}

/// Generate visitors.
impl VisitBuilder<'_> {
    /// Generate `visit_*` methods and `walk_*` functions for a struct.
    ///
    /// Also generates functions for types of struct fields.
    fn generate_struct_visitor(&mut self, struct_def: &StructDef) {
        // Exit if this struct is not visited
        let Some(visitor_names) = &struct_def.visit.visitor_names else { return };

        // Generate visit methods
        let struct_ty = struct_def.ty(self.schema);
        let visit_fn_ident = visitor_names.visitor_ident();
        let walk_fn_ident = visitor_names.walk_ident();

        // Get additional params
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
        self.visit_methods.extend(gen_visit_fn(quote!( & )));
        self.visit_mut_methods.extend(gen_visit_fn(quote!( &mut )));

        // Generate walk functions

        // Generate `enter_node` and `leave_node` calls (if this struct has an `AstKind`)
        let struct_ident = struct_def.ident();
        let has_kind = struct_def.kind.has_kind;
        let (enter_node, leave_node) =
            generate_enter_and_leave_node(&struct_ident, has_kind, false);
        let (enter_node_mut, leave_node_mut) =
            generate_enter_and_leave_node(&struct_ident, has_kind, true);

        // Generate `enter_scope` and `leave_scope` calls (if this struct has a scope).
        // They will be inserted before the relevant fields.
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
            let scope_entry = (scope.enter_before_index, enter_scope);

            let leave_scope = quote!( visitor.leave_scope(); );
            let scope_exit = (scope.exit_before_index, leave_scope);

            (Some(scope_entry), Some(scope_exit))
        } else {
            (None, None)
        };

        // Generate `visit_*` calls for struct fields
        let mut field_visits_count = 0usize;
        let (mut field_visits, mut field_visits_mut): (TokenStream, TokenStream) = struct_def
            .fields
            .iter()
            .enumerate()
            .filter_map(|(field_index, field)| {
                let (visit, visit_mut) = self.generate_struct_field_visit(
                    field,
                    field_index,
                    &mut scope_entry,
                    &mut scope_exit,
                )?;

                field_visits_count += 1;

                Some((visit, visit_mut))
            })
            .unzip();

        // If didn't enter or exit scope already, enter/exit after last field
        if let Some((_, enter_scope)) = scope_entry {
            field_visits.extend(enter_scope.clone());
            field_visits_mut.extend(enter_scope);
        }
        if let Some((_, leave_scope)) = scope_exit {
            field_visits.extend(leave_scope.clone());
            field_visits_mut.extend(leave_scope);
        }

        // `#[inline]` if there are 5 or less fields visited
        // TODO: Is this ideal?
        let maybe_inline_attr =
            if field_visits_count <= 5 { quote!( #[inline] ) } else { quote!() };

        self.walk_fns.extend(quote! {
            ///@@line_break
            #maybe_inline_attr
            pub fn #walk_fn_ident<'a, V: Visit<'a>>(visitor: &mut V, it: &#struct_ty #extra_params) {
                #enter_node
                #field_visits
                #leave_node
            }
        });
        self.walk_mut_fns.extend(quote! {
            ///@@line_break
            #maybe_inline_attr
            pub fn #walk_fn_ident<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut #struct_ty #extra_params) {
                #enter_node_mut
                #field_visits_mut
                #leave_node_mut
            }
        });
    }

    /// Generate visitor calls for a struct field.
    ///
    /// e.g. `visitor.visit_span(&it.span);`.
    ///
    /// Also inserts `enter_scope` / `leave_scope` calls before the visit call if needed.
    fn generate_struct_field_visit(
        &self,
        field: &FieldDef,
        field_index: usize,
        scope_entry: &mut Option<(usize, TokenStream)>,
        scope_exit: &mut Option<(usize, TokenStream)>,
    ) -> Option<(/* visit */ TokenStream, /* visit_mut */ TokenStream)> {
        // Generate `visit_*` method call for struct field
        let field_type = field.type_def(self.schema);
        let field_ident = field.ident();
        let (mut visit, mut visit_mut) = self.generate_visit_type(
            field_type,
            Target::Property(quote!( it.#field_ident )),
            &field.visit.visit_args,
            &field_ident,
            true,
        )?;

        // Insert `enter_scope` / `leave_scope` call, if scope needs to be entered/exited before this field.
        //
        // We handle exiting scope first, to create correct output if entering and exiting on same field.
        // The `if` block for entering scope prepends `enter_scope` call *before* whatever it's passed.
        // If both entering and exiting, that means `enter_scope` is inserted before `leave_scope`.
        if let Some((exit_index, _)) = scope_exit {
            if *exit_index <= field_index {
                let (_, leave_scope) = scope_exit.take().unwrap();
                visit = quote!( #leave_scope #visit );
                visit_mut = quote!( #leave_scope #visit_mut );
            }
        }

        if let Some((enter_index, _)) = scope_entry {
            if *enter_index <= field_index {
                let (_, enter_scope) = scope_entry.take().unwrap();
                visit = quote!( #enter_scope #visit );
                visit_mut = quote!( #enter_scope #visit_mut );
            }
        }

        Some((visit, visit_mut))
    }

    /// Generate `visit_*` methods and `walk_*` functions for an enum.
    ///
    /// Also generates functions for types of enum variants.
    fn generate_enum_visitor(&mut self, enum_def: &EnumDef) {
        // Exit if this enum is not visited
        let Some(visitor_names) = &enum_def.visit.visitor_names else { return };

        // Generate visit methods
        let enum_ty = enum_def.ty(self.schema);
        let visit_fn_ident = visitor_names.visitor_ident();
        let walk_fn_ident = visitor_names.walk_ident();

        let gen_visit = |reference| {
            quote! {
                ///@@line_break
                #[inline]
                fn #visit_fn_ident(&mut self, it: #reference #enum_ty) {
                    #walk_fn_ident(self, it);
                }
            }
        };
        self.visit_methods.extend(gen_visit(quote!( & )));
        self.visit_mut_methods.extend(gen_visit(quote!( &mut )));

        // Generate walk functions
        let enum_ident = enum_def.ident();
        let has_kind = enum_def.kind.has_kind;
        let (enter_node, leave_node) = generate_enter_and_leave_node(&enum_ident, has_kind, false);
        let (enter_node_mut, leave_node_mut) =
            generate_enter_and_leave_node(&enum_ident, has_kind, true);

        let mut match_arm_count = 0usize;
        let (variant_match_arms, variant_match_arms_mut): (TokenStream, TokenStream) = enum_def
            .variants
            .iter()
            .filter_map(|variant| {
                let variant_type = variant.field_type(self.schema)?;
                let (visit, visit_mut) = self.generate_visit_type(
                    variant_type,
                    Target::Reference(create_ident_tokens("it")),
                    &variant.visit.visit_args,
                    &create_ident_tokens("it"),
                    false,
                )?;

                match_arm_count += 1;

                let variant_ident = variant.ident();
                let match_pattern = quote!( #enum_ident::#variant_ident(it) );
                let match_arm = quote!( #match_pattern => #visit, );
                let match_arm_mut = quote!( #match_pattern => #visit_mut, );
                Some((match_arm, match_arm_mut))
            })
            .unzip();

        let (inherits_match_arms, inherits_match_arms_mut): (TokenStream, TokenStream) = enum_def
            .inherits_types(self.schema)
            .map(|inherits_type| {
                let inherits_type = inherits_type.as_enum().unwrap();
                let inner_visit_fn_ident = inherits_type.visit.visitor_ident();
                let Some(inner_visit_fn_ident) = inner_visit_fn_ident else {
                    panic!(
                        "When an enum inherits variants from another enum and the inheritor has a visitor, \
                        the inherited enum must also have a visitor: `{}` inheriting from `{}`",
                        enum_def.name(),
                        inherits_type.name(),
                    );
                };

                match_arm_count += 1;

                let inherits_snake_name = inherits_type.snake_name();
                let match_ident = format_ident!("match_{inherits_snake_name}");

                let to_fn_ident = format_ident!("to_{inherits_snake_name}");
                let match_arm = quote! {
                    #match_ident!(#enum_ident) => visitor.#inner_visit_fn_ident(it.#to_fn_ident()),
                };

                let to_fn_ident_mut = format_ident!("to_{inherits_snake_name}_mut");
                let match_arm_mut = quote! {
                    #match_ident!(#enum_ident) => visitor.#inner_visit_fn_ident(it.#to_fn_ident_mut()),
                };

                (match_arm, match_arm_mut)
            })
            .unzip();

        // Add catch-all match arm if not all variants are visited
        let catch_all_match_arm =
            if match_arm_count < enum_def.variants.len() + enum_def.inherits.len() {
                quote!( _ => {} )
            } else {
                quote!()
            };

        // `#[inline]` if there are 5 or less match cases
        // TODO: Is this ideal?
        let maybe_inline_attr = if match_arm_count <= 5 { quote!( #[inline] ) } else { quote!() };

        self.walk_fns.extend(quote! {
            ///@@line_break
            #maybe_inline_attr
            pub fn #walk_fn_ident<'a, V: Visit<'a>>(visitor: &mut V, it: & #enum_ty) {
                #enter_node
                match it {
                    #variant_match_arms
                    #inherits_match_arms
                    #catch_all_match_arm
                }
                #leave_node
            }
        });
        self.walk_mut_fns.extend(quote! {
            ///@@line_break
            #maybe_inline_attr
            pub fn #walk_fn_ident<'a, V: VisitMut<'a>>(visitor: &mut V, it: &mut #enum_ty) {
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

    /// Generate `visit_*` methods and `walk_*` functions for a `Vec`.
    ///
    /// Also generates functions for inner type (`T` in `Vec<T>`).
    fn generate_vec_visitor(&mut self, vec_def: &VecDef) {
        // Exit if this `Vec` does not have its own visitor
        let Some(visitor_names) = &vec_def.visit.visitor_names else { return };

        // Generate visit methods
        let vec_ty = vec_def.ty(self.schema);

        let visit_fn_ident = visitor_names.visitor_ident();
        let walk_fn_ident = visitor_names.walk_ident();

        let gen_visit = |reference| {
            quote! {
                ///@@line_break
                #[inline]
                fn #visit_fn_ident(&mut self, it: #reference #vec_ty) {
                    #walk_fn_ident(self, it);
                }
            }
        };
        self.visit_methods.extend(gen_visit(quote!( & )));
        self.visit_mut_methods.extend(gen_visit(quote!( &mut )));

        // Generate walk functions
        let inner_type = vec_def.inner_type(self.schema);
        let inner_visit_fn_ident = match inner_type {
            TypeDef::Struct(struct_def) => struct_def.visit.visitor_ident().unwrap(),
            TypeDef::Enum(enum_def) => enum_def.visit.visitor_ident().unwrap(),
            _ => unreachable!(),
        };

        let gen_walk = |visit_trait_name, reference| {
            let visit_trait_ident = create_safe_ident(visit_trait_name);
            quote! {
                ///@@line_break
                #[inline]
                pub fn #walk_fn_ident<'a, V: #visit_trait_ident<'a>>(visitor: &mut V, it: #reference #vec_ty) {
                    for el in it {
                        visitor.#inner_visit_fn_ident(el);
                    }
                }
            }
        };
        self.walk_fns.extend(gen_walk("Visit", quote!( & )));
        self.walk_mut_fns.extend(gen_walk("VisitMut", quote!( &mut )));
    }
}

/// Generate visitor calls.
impl VisitBuilder<'_> {
    /// Generate visitor calls for a type.
    ///
    /// e.g.:
    /// * `visitor.visit_span(&it.span)`
    /// * `if let Some(span) = &it.span { visitor.visit_span(span); }`.
    ///
    /// Returns `None` if this type is not visited.
    ///
    /// * `target` is the expression for the type, represented by a [`Target`].
    ///   e.g. `it.span` in first example above, or `span` in the 2nd.
    ///
    /// * `visit_args` contains details of any extra arguments to be passed to visitor.
    ///   Parsed from `#[visit(args(flags = ScopeFlags::Function))]` attr on struct field / enum variant.
    ///
    /// * `field_ident` is [`Ident`] for the field.
    ///   Is used in output for `Option`s. e.g. `span` in `if let Some(span) = ...`.
    ///
    /// * `trailing_semicolon` indicates if a semicolon postfix is needed.
    ///   This is `true` for struct fields, `false` for enum variants.
    ///
    /// [`Ident`]: struct@Ident
    fn generate_visit_type(
        &self,
        type_def: &TypeDef,
        target: Target,
        visit_args: &[(String, String)],
        field_ident: &TokenStream,
        trailing_semicolon: bool,
    ) -> Option<(/* visit */ TokenStream, /* visit_mut */ TokenStream)> {
        match type_def {
            TypeDef::Struct(_) | TypeDef::Enum(_) => Self::generate_visit_struct_or_enum(
                type_def,
                target,
                visit_args,
                trailing_semicolon,
            ),
            TypeDef::Option(option_def) => {
                self.generate_visit_option(option_def, target, visit_args, field_ident)
            }
            TypeDef::Box(box_def) => {
                // `Box`es can be treated as transparent, as auto-deref handles it
                self.generate_visit_type(
                    box_def.inner_type(self.schema),
                    target,
                    visit_args,
                    field_ident,
                    trailing_semicolon,
                )
            }
            TypeDef::Vec(vec_def) => {
                self.generate_visit_vec(vec_def, target, visit_args, trailing_semicolon)
            }
            // Primitives and `Cell`s are not visited
            TypeDef::Primitive(_) | TypeDef::Cell(_) => None,
        }
    }

    /// Generate visitor calls for a struct or enum.
    ///
    /// e.g. `visitor.visit_span(&it.span)`
    ///
    /// Returns `None` if this type is not visited.
    ///
    /// See comment on [`Self::generate_visit_type`] for details of parameters.
    fn generate_visit_struct_or_enum(
        type_def: &TypeDef,
        target: Target,
        visit_args: &[(String, String)],
        trailing_semicolon: bool,
    ) -> Option<(/* visit */ TokenStream, /* visit_mut */ TokenStream)> {
        let visit_fn_ident = match type_def {
            TypeDef::Struct(struct_def) => struct_def.visit.visitor_ident()?,
            TypeDef::Enum(enum_def) => enum_def.visit.visitor_ident()?,
            _ => None?,
        };

        Some(Self::generate_visit_with_visit_args(
            &visit_fn_ident,
            target,
            visit_args,
            trailing_semicolon,
        ))
    }

    /// Generate visitor calls with specified visitor function name.
    ///
    /// Usually generates `visitor.visit_whatever(target)`, but also handles additional arguments to visitor.
    /// e.g. if `visit_args` was parsed from `#[visit(args(flags = ScopeFlags::Function))]`, generates:
    ///
    /// ```ignore
    /// {
    ///     let flags = ScopeFlags::Function;
    ///     visitor.visit_whatever(target, flags)
    /// }
    /// ```
    ///
    /// See comment on [`Self::generate_visit_type`] for details of other parameters.
    fn generate_visit_with_visit_args(
        visit_fn_ident: &Ident,
        target: Target,
        visit_args: &[(String, String)],
        trailing_semicolon: bool,
    ) -> (/* visit */ TokenStream, /* visit_mut */ TokenStream) {
        let (target_ref, target_mut) = target.generate_refs();

        // Get extra function params for visit args.
        // e.g. if attr on struct field/enum variant is `#[visit(args(x = something, y = something_else))]`,
        // `extra_params` is `, x, y`.
        let arg_params = visit_args.iter().map(|(arg_name, _)| create_ident(arg_name));
        let extra_params = quote!( #(, #arg_params)* );

        let gen_visit = |target| {
            let mut visit = quote!( visitor.#visit_fn_ident(#target #extra_params) );
            if trailing_semicolon {
                visit.extend(quote!(;));
            }

            if extra_params.is_empty() {
                return visit;
            };

            // Wrap a visit call with `let` statements for visit args.
            // e.g. if attr on struct field/enum variant is `#[visit(args(x = something, y = something_else))]`,
            // then output `{ let x = something; let y = something_else; visitor.visit_thing(it, x, y) }`.
            let let_args = visit_args.iter().map(|(arg_name, arg_value)| {
                let arg_ident = create_ident(arg_name);
                let arg_value = parse_str::<Expr>(&arg_value.cow_replace("self", "it")).unwrap();
                quote!( let #arg_ident = #arg_value; )
            });
            quote! {{
                #(#let_args)*
                #visit
            }}
        };
        (gen_visit(target_ref), gen_visit(target_mut))
    }

    /// Generate visitor calls for an `Option`.
    ///
    /// e.g.:
    /// ```ignore
    /// if let Some(span) = &it.span {
    ///     visitor.visit_span(span);
    /// }
    /// ```
    ///
    /// Returns `None` if inner type is not visited.
    ///
    /// See comment on [`Self::generate_visit_type`] for details of parameters.
    fn generate_visit_option(
        &self,
        option_def: &OptionDef,
        target: Target,
        visit_args: &[(String, String)],
        field_ident: &TokenStream,
    ) -> Option<(/* visit */ TokenStream, /* visit_mut */ TokenStream)> {
        let inner_type = option_def.inner_type(self.schema);
        let (inner_visit, inner_visit_mut) = self.generate_visit_type(
            inner_type,
            Target::Reference(field_ident.clone()),
            visit_args,
            field_ident,
            true,
        )?;
        let (target_ref, target_mut) = target.generate_refs();

        let gen_visit = |inner_visit, target| {
            quote! {
                if let Some(#field_ident) = #target {
                    #inner_visit
                }
            }
        };
        Some((gen_visit(inner_visit, target_ref), gen_visit(inner_visit_mut, target_mut)))
    }

    /// Generate visitor calls for a `Vec`.
    ///
    /// If `Vec` has its own visitor (it does when inner type is a struct or enum which has a visitor),
    /// generates a call to that visitor e.g. `visitor.visit_statements(&it.statements)`.
    ///
    /// Otherwise, generates code to loop through the `Vec`'s elements and call the inner type's visitor:
    ///
    /// ```ignore
    /// for statements in it.statements.iter() {
    ///     visitor.visit_statement(statements);
    /// }
    /// ```
    ///
    /// If inner type is an option, adds `.flatten()`:
    ///
    /// ```ignore
    /// for statements in it.statements.iter().flatten() {
    ///     visitor.visit_statement(statements);
    /// }
    /// ```
    ///
    /// Returns `None` if inner type is not visited.
    ///
    /// See comment on [`Self::generate_visit_type`] for details of parameters.
    fn generate_visit_vec(
        &self,
        vec_def: &VecDef,
        target: Target,
        visit_args: &[(String, String)],
        trailing_semicolon: bool,
    ) -> Option<(/* visit */ TokenStream, /* visit_mut */ TokenStream)> {
        if let Some(visit_fn_ident) = vec_def.visit.visitor_ident() {
            // Inner type is a struct or enum which has a visitor. This `Vec` has its own visitor.
            return Some(Self::generate_visit_with_visit_args(
                &visit_fn_ident,
                target,
                visit_args,
                trailing_semicolon,
            ));
        }

        // Flatten any `Option`s with `.flatten()` on the iterator.
        // Treat any `Box`es as transparent - auto-deref means we can ignore them.
        let mut inner_type = vec_def.inner_type(self.schema);

        let mut maybe_flatten = quote!();
        loop {
            match inner_type {
                TypeDef::Option(option_def) => {
                    inner_type = option_def.inner_type(self.schema);
                    maybe_flatten.extend(quote!( .flatten() ));
                }
                TypeDef::Box(box_def) => {
                    inner_type = box_def.inner_type(self.schema);
                }
                _ => break,
            }
        }

        // This `Vec` does not have it's own visitor. Loop through elements and visit each in turn.
        let (inner_visit, inner_visit_mut) = self.generate_visit_type(
            inner_type,
            Target::Reference(create_ident_tokens("el")),
            visit_args,
            &create_ident_tokens("it"),
            true,
        )?;

        let target = target.into_tokens();

        let gen_visit = |inner_visit, iter_method| {
            let iter_method_ident = create_safe_ident(iter_method);
            quote! {
                for el in #target.#iter_method_ident() #maybe_flatten {
                    #inner_visit
                }
            }
        };
        let visit = gen_visit(inner_visit, "iter");
        let visit_mut = gen_visit(inner_visit_mut, "iter_mut");
        Some((visit, visit_mut))
    }
}

/// Target for a visit function call.
///
/// * `Target::Reference` represents a variable which is already a reference.
///   e.g. `span` in `if let Some(span) = &it.span {}`
///   Does not need `&` / `&mut` prepended to it when using it.
///
/// * `Target::Property` represents an object property e.g. `it.span`.
///   Needs `&` / `&mut` prepended to it when using it in most circumstances.
enum Target {
    Reference(TokenStream),
    Property(TokenStream),
}

impl Target {
    /// Prepend target with `&` or `&mut` if required.
    ///
    /// * If this [`Target`] is already a reference, return just the identifier.
    /// * Otherwise, return pair of refs - `&target` and `&mut target`.
    fn generate_refs(self) -> (TokenStream, TokenStream) {
        match self {
            Self::Reference(ident) => (ident.clone(), ident),
            Self::Property(prop) => (quote!( &#prop ), quote!( &mut #prop )),
        }
    }

    /// Get this [`Target`] without prepending `&` / `&mut`.
    fn into_tokens(self) -> TokenStream {
        match self {
            Self::Reference(ident) => ident,
            Self::Property(prop) => prop,
        }
    }
}

/// Generate code for `enter_node` and `leave_node`.
///
/// If the type has no `AstKind`, returns a comment for enter, and empty token stream for exit.
fn generate_enter_and_leave_node(
    type_ident: &Ident,
    has_kind: bool,
    is_mut: bool,
) -> (/* enter_node */ TokenStream, /* leave_node */ TokenStream) {
    if has_kind {
        let kind = if is_mut {
            quote!( AstType::#type_ident )
        } else {
            quote!( AstKind::#type_ident(visitor.alloc(it)) )
        };
        let enter_node = quote! {
            let kind = #kind;
            visitor.enter_node(kind);
        };
        let leave_node = quote!( visitor.leave_node(kind); );
        (enter_node, leave_node)
    } else {
        let comment =
            format!("@ No `{}` for this type", if is_mut { "AstType" } else { "AstKind" });
        (quote!( #![doc = #comment] ), quote!())
    }
}
