//! Generator for `oxc_traverse` crate code.
//!
//! Generates:
//! - `traverse.rs` - `Traverse` trait with `enter_*` and `exit_*` methods
//! - `ancestor.rs` - `Ancestor` type and related types for tracking parent nodes
//! - `walk.rs` - `walk_*` functions for traversing AST

use convert_case::{Case, Casing};
use cow_utils::CowUtils;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::{
    Codegen, Generator, TRAVERSE_CRATE_PATH,
    output::{Output, output_path},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, Visibility},
    utils::{create_ident, create_safe_ident},
};

use super::define_generator;

// =============================================================================
// Custom snake_case conversion (matches original JS behavior)
// =============================================================================

/// Convert camelCase to snake_case, matching the original JS `camelToSnake` function.
///
/// This handles special prefixes (TS, JSX, JS) and preserves digits without underscore.
/// e.g., `V8IntrinsicExpression` -> `v8_intrinsic_expression` (not `v_8_intrinsic_expression`)
fn camel_to_snake(name: &str) -> String {
    let prefix_len = if name.starts_with("TS") {
        2
    } else if name.starts_with("JSX") {
        3
    } else if name.starts_with("JS") {
        2
    } else {
        1
    };

    let mut result = name[..prefix_len].cow_to_lowercase().into_owned();
    for c in name[prefix_len..].chars() {
        if c.is_ascii_uppercase() {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert camelCase to SCREAMING_CASE, matching the original JS behavior.
///
/// This handles special prefixes (TS, JSX, JS) and preserves digits without underscore.
/// e.g., `V8IntrinsicExpression` -> `V8_INTRINSIC_EXPRESSION` (not `V_8_INTRINSIC_EXPRESSION`)
fn camel_to_screaming(name: &str) -> String {
    let prefix_len = if name.starts_with("TS") {
        2
    } else if name.starts_with("JSX") {
        3
    } else if name.starts_with("JS") {
        2
    } else {
        1
    };

    let mut result = name[..prefix_len].cow_to_uppercase().into_owned();
    for c in name[prefix_len..].chars() {
        if c.is_ascii_uppercase() {
            result.push('_');
            result.push(c);
        } else {
            result.push(c.to_ascii_uppercase());
        }
    }
    result
}

// =============================================================================
// Type filtering - only include types from oxc_ast/src/ast/
// =============================================================================

/// Check if a struct is from the AST module (oxc_ast/src/ast/).
fn is_ast_type_struct(struct_def: &StructDef, schema: &Schema) -> bool {
    let file = struct_def.file(schema);
    file.krate() == "oxc_ast" && file.import_path().starts_with("::ast::")
}

/// Check if an enum is from the AST module (oxc_ast/src/ast/).
fn is_ast_type_enum(enum_def: &EnumDef, schema: &Schema) -> bool {
    let file = enum_def.file(schema);
    file.krate() == "oxc_ast" && file.import_path().starts_with("::ast::")
}

/// Check if a type definition is from the AST module (oxc_ast/src/ast/).
fn is_ast_type(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(def) => is_ast_type_struct(def, schema),
        TypeDef::Enum(def) => is_ast_type_enum(def, schema),
        _ => false,
    }
}

/// Check if a field's inner type is from the AST module and has a visitor.
fn field_has_ast_visitor(field: &FieldDef, schema: &Schema) -> bool {
    let inner_type = get_inner_type(field.type_def(schema), schema);
    match inner_type {
        TypeDef::Struct(def) => def.visit.has_visitor() && is_ast_type_struct(def, schema),
        TypeDef::Enum(def) => def.visit.has_visitor() && is_ast_type_enum(def, schema),
        _ => false,
    }
}

/// Get file order key for sorting types.
/// Matches original JS file processing order: js.rs, jsx.rs, literal.rs, ts.rs
fn get_file_order(import_path: &str) -> usize {
    match import_path {
        "::ast::js" => 0,
        "::ast::jsx" => 1,
        "::ast::literal" => 2,
        "::ast::ts" => 3,
        _ => 4,
    }
}

/// Get types sorted by file order (matching original JS behavior).
fn get_sorted_ast_types(schema: &Schema) -> Vec<&TypeDef> {
    let mut types: Vec<_> = schema
        .types
        .iter()
        .filter(|type_def| match type_def {
            TypeDef::Struct(def) => is_ast_type_struct(def, schema) && def.visit.has_visitor(),
            TypeDef::Enum(def) => is_ast_type_enum(def, schema) && def.visit.has_visitor(),
            _ => false,
        })
        .collect();

    types.sort_by_key(|type_def| {
        let (file_id, type_id) = match type_def {
            TypeDef::Struct(def) => (def.file_id, def.id.index()),
            TypeDef::Enum(def) => (def.file_id, def.id.index()),
            _ => unreachable!(),
        };
        let file = &schema.files[file_id];
        (get_file_order(file.import_path()), type_id)
    });

    types
}

/// Generator for `oxc_traverse` crate.
pub struct TraverseGenerator;

define_generator!(TraverseGenerator);

impl Generator for TraverseGenerator {
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let traverse_output = generate_traverse(schema);
        let ancestor_output = generate_ancestor(schema);
        let walk_output = generate_walk(schema);

        vec![
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "traverse.rs"),
                tokens: traverse_output,
            },
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "ancestor.rs"),
                tokens: ancestor_output,
            },
            Output::Rust { path: output_path(TRAVERSE_CRATE_PATH, "walk.rs"), tokens: walk_output },
        ]
    }
}

// =============================================================================
// Type generation helpers - generate types without lifetime parameters
// =============================================================================

/// Generate a type without lifetime parameters.
/// This is used for raw pointer casts where lifetimes are erased.
fn ty_without_lifetime(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    match type_def {
        TypeDef::Struct(def) => {
            let ident = def.ident();
            quote!(#ident)
        }
        TypeDef::Enum(def) => {
            let ident = def.ident();
            quote!(#ident)
        }
        TypeDef::Primitive(def) => {
            // Handle special primitive types
            match def.name() {
                "&str" => quote!(&str),
                "Atom" => quote!(Atom),
                _ => {
                    let ident = def.ident();
                    quote!(#ident)
                }
            }
        }
        TypeDef::Option(def) => {
            let inner = ty_without_lifetime(def.inner_type(schema), schema);
            quote!(Option<#inner>)
        }
        TypeDef::Box(def) => {
            let inner = ty_without_lifetime(def.inner_type(schema), schema);
            quote!(Box<#inner>)
        }
        TypeDef::Vec(def) => {
            let inner = ty_without_lifetime(def.inner_type(schema), schema);
            quote!(Vec<#inner>)
        }
        TypeDef::Cell(def) => {
            let inner = ty_without_lifetime(def.inner_type(schema), schema);
            quote!(Cell<#inner>)
        }
        TypeDef::Pointer(_) => {
            panic!("Pointer type should not appear in traverse generation")
        }
    }
}

// =============================================================================
// traverse.rs generation
// =============================================================================

/// Generate the `Traverse` trait.
fn generate_traverse(schema: &Schema) -> TokenStream {
    let mut traverse_methods = TokenStream::new();

    // Generate methods for all visited types from AST module (sorted to match original JS order)
    for type_def in get_sorted_ast_types(schema) {
        let (type_name, type_ty) = match type_def {
            TypeDef::Struct(struct_def) => (struct_def.name(), struct_def.ty(schema)),
            TypeDef::Enum(enum_def) => (enum_def.name(), enum_def.ty(schema)),
            _ => continue,
        };

        let snake_name = camel_to_snake(type_name);
        let enter_ident = format_ident!("enter_{snake_name}");
        let exit_ident = format_ident!("exit_{snake_name}");

        traverse_methods.extend(quote! {
            ///@@line_break
            #[inline]
            fn #enter_ident(&mut self, node: &mut #type_ty, ctx: &mut TraverseCtx<'a, State>) {}
            #[inline]
            fn #exit_ident(&mut self, node: &mut #type_ty, ctx: &mut TraverseCtx<'a, State>) {}
        });
    }

    // Add special method for `Vec<Statement>` (Statements)
    traverse_methods.extend(quote! {
        ///@@line_break
        #[inline]
        fn enter_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, State>) {}
        #[inline]
        fn exit_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, State>) {}
    });

    quote! {
        use oxc_allocator::Vec;
        use oxc_ast::ast::*;

        ///@@line_break
        use crate::TraverseCtx;

        ///@@line_break
        #[expect(unused_variables)]
        pub trait Traverse<'a, State> {
            #traverse_methods
        }
    }
}

// =============================================================================
// ancestor.rs generation
// =============================================================================

/// Generate the `Ancestor` enum and related types.
fn generate_ancestor(schema: &Schema) -> TokenStream {
    let mut ancestor_type_variants = TokenStream::new();
    let mut ancestor_variants = TokenStream::new();
    let mut is_functions = TokenStream::new();
    let mut address_match_arms = TokenStream::new();
    // Combined offset constants + structs (interleaved per type, matching original)
    let mut offset_consts_and_structs = TokenStream::new();
    let mut discriminant: u16 = 1;

    // Track which enum types have variants for generating `is_parent_of_*` functions
    let mut enum_parent_variants: Vec<(String, Vec<String>)> = vec![];

    // Process types in sorted order (matching original JS)
    for type_def in get_sorted_ast_types(schema) {
        let TypeDef::Struct(struct_def) = type_def else { continue };

        // Collect fields that have visited inner types from AST module
        let visited_fields: Vec<_> =
            struct_def.fields.iter().filter(|field| field_has_ast_visitor(field, schema)).collect();

        // Only generate content for structs that have visited fields
        if visited_fields.is_empty() {
            continue;
        }

        let type_name = struct_def.name();
        let type_ident = struct_def.ident();
        let type_snake_name = camel_to_snake(type_name);
        let type_screaming_name = camel_to_screaming(type_name);
        // Use type WITH lifetime for the raw pointer in *Without* structs
        let type_ptr_ty = struct_def.ty(schema);

        // Generate offset constants for all public/restricted fields (for this type)
        let mut type_offset_consts = TokenStream::new();
        for field in &struct_def.fields {
            // Skip private fields
            if matches!(field.visibility, Visibility::Private) {
                continue;
            }

            let field_name = field.name();
            let field_raw_name = create_ident(field_name);
            let field_screaming_name = field_name.to_case(Case::UpperSnake);
            let offset_const_name =
                format_ident!("OFFSET_{type_screaming_name}_{field_screaming_name}");

            type_offset_consts.extend(quote! {
                pub(crate) const #offset_const_name: usize = offset_of!(#type_ident, #field_raw_name);
            });
        }

        // Accumulate *Without* structs for this type
        let mut type_structs = TokenStream::new();

        let mut type_variant_names = vec![];

        for field in &visited_fields {
            let field_name = field.name();
            let field_camel_name = field_name.to_case(Case::UpperCamel);
            let variant_name = format!("{type_name}{field_camel_name}");
            let variant_ident = create_safe_ident(&variant_name);

            type_variant_names.push(variant_name.clone());

            // Generate AncestorType variant
            let disc = Literal::u16_unsuffixed(discriminant);
            ancestor_type_variants.extend(quote! {
                #variant_ident = #disc,
            });

            // Check if the field's inner type is an enum for `is_parent_of_*` tracking
            let inner_type = get_inner_type(field.type_def(schema), schema);
            if let TypeDef::Enum(enum_def) = inner_type {
                let enum_name = enum_def.name().to_string();
                if let Some(existing) =
                    enum_parent_variants.iter_mut().find(|(name, _)| name == &enum_name)
                {
                    existing.1.push(variant_name.clone());
                } else {
                    enum_parent_variants.push((enum_name, vec![variant_name.clone()]));
                }
            }

            // Generate struct for this field's ancestor
            let struct_name_str = format!("{type_name}Without{field_camel_name}");
            let struct_ident = create_safe_ident(&struct_name_str);

            let has_lifetime = struct_def.has_lifetime;
            let lifetimes = if has_lifetime { quote!(<'a, 't>) } else { quote!(<'t>) };

            // Generate accessor methods for other fields (only public/restricted ones)
            let mut accessor_methods = TokenStream::new();
            for other_field in &struct_def.fields {
                if other_field.name() == field_name {
                    continue;
                }

                // Skip private fields
                if matches!(other_field.visibility, Visibility::Private) {
                    continue;
                }

                let other_field_name = other_field.name();
                let other_field_ident = create_ident(other_field_name);
                let other_field_ty = other_field.type_def(schema).ty(schema);
                let other_field_screaming_name = other_field_name.to_case(Case::UpperSnake);
                let other_offset_const =
                    format_ident!("OFFSET_{type_screaming_name}_{other_field_screaming_name}");

                accessor_methods.extend(quote! {
                    ///@@line_break
                    #[inline]
                    pub fn #other_field_ident(self) -> &'t #other_field_ty {
                        unsafe {
                            &*(
                                (self.0 as *const u8).add(#other_offset_const)
                                as *const #other_field_ty
                            )
                        }
                    }
                });
            }

            type_structs.extend(quote! {
                ///@@line_break
                #[repr(transparent)]
                #[derive(Clone, Copy, Debug)]
                pub struct #struct_ident #lifetimes(
                    pub(crate) *const #type_ptr_ty,
                    pub(crate) PhantomData<&'t ()>,
                );

                ///@@line_break
                impl #lifetimes #struct_ident #lifetimes {
                    #accessor_methods
                }

                ///@@line_break
                impl #lifetimes GetAddress for #struct_ident #lifetimes {
                    #[inline]
                    fn address(&self) -> Address {
                        unsafe { Address::from_ptr(self.0) }
                    }
                }
            });

            // Generate Ancestor variant
            let ancestor_type_ident = create_safe_ident("AncestorType");
            ancestor_variants.extend(quote! {
                #variant_ident(#struct_ident #lifetimes) = #ancestor_type_ident::#variant_ident as u16,
            });

            // Generate address match arm
            address_match_arms.extend(quote! {
                Self::#variant_ident(a) => a.address(),
            });

            discriminant += 1;
        }

        // Append offset constants and structs for this type (interleaved per type, matching original JS)
        offset_consts_and_structs.extend(quote! {
            ///@@line_break
            #type_offset_consts
            #type_structs
        });

        // Generate is_* function for this type
        if !type_variant_names.is_empty() {
            let is_fn_ident = format_ident!("is_{type_snake_name}");
            let variant_patterns: Vec<_> = type_variant_names
                .iter()
                .map(|name| {
                    let ident = create_safe_ident(name);
                    quote!(Self::#ident(_))
                })
                .collect();

            is_functions.extend(quote! {
                ///@@line_break
                #[inline]
                pub fn #is_fn_ident(self) -> bool {
                    matches!(self, #(#variant_patterns)|*)
                }
            });
        }
    }

    // Generate `is_parent_of_*` functions for enums
    for (enum_name, variant_names) in enum_parent_variants {
        let enum_snake_name = camel_to_snake(&enum_name);
        let is_fn_ident = format_ident!("is_parent_of_{enum_snake_name}");
        let variant_patterns: Vec<_> = variant_names
            .iter()
            .map(|name| {
                let ident = create_safe_ident(name);
                quote!(Self::#ident(_))
            })
            .collect();

        is_functions.extend(quote! {
            ///@@line_break
            #[inline]
            pub fn #is_fn_ident(self) -> bool {
                matches!(self, #(#variant_patterns)|*)
            }
        });
    }

    quote! {
        #![expect(
            clippy::cast_ptr_alignment,
            clippy::elidable_lifetime_names,
            clippy::ptr_as_ptr,
            clippy::ref_option,
            clippy::undocumented_unsafe_blocks
        )]

        ///@@line_break
        use std::{cell::Cell, marker::PhantomData, mem::offset_of};

        ///@@line_break
        use oxc_allocator::{Address, Box, GetAddress, Vec};
        use oxc_ast::ast::*;
        use oxc_syntax::scope::ScopeId;

        ///@@line_break
        /// Type of [`Ancestor`].
        /// Used in [`crate::TraverseCtx::retag_stack`].
        #[repr(u16)]
        #[derive(Clone, Copy)]
        pub(crate) enum AncestorType {
            None = 0,
            #ancestor_type_variants
        }

        ///@@line_break
        /// Ancestor type used in AST traversal.
        ///
        /// Encodes both the type of the parent, and child's location in the parent.
        /// i.e. variants for `BinaryExpressionLeft` and `BinaryExpressionRight`, not just `BinaryExpression`.
        ///
        /// `'a` is lifetime of AST nodes.
        /// `'t` is lifetime of the `Ancestor` (which inherits lifetime from `&'t TraverseCtx'`).
        /// i.e. `Ancestor`s can only exist within the body of `enter_*` and `exit_*` methods
        /// and cannot "escape" from them.
        ///@
        ///@ SAFETY
        ///@ * This type must be `#[repr(u16)]`.
        ///@ * Variant discriminants must correspond to those in `AncestorType`.
        ///@
        ///@ These invariants make it possible to set the discriminant of an `Ancestor` without altering
        ///@ the "payload" pointer with:
        ///@ `*(ancestor as *mut _ as *mut AncestorType) = AncestorType::Program`.
        ///@ `TraverseCtx::retag_stack` uses this technique.
        #[repr(C, u16)]
        #[derive(Clone, Copy, Debug)]
        pub enum Ancestor<'a, 't> {
            None = AncestorType::None as u16,
            #ancestor_variants
        }

        ///@@line_break
        impl<'a, 't> Ancestor<'a, 't> {
            #is_functions
        }

        ///@@line_break
        impl<'a, 't> GetAddress for Ancestor<'a, 't> {
            /// Get memory address of node represented by `Ancestor` in the arena.
            ///@ Compiler should reduce this down to only a couple of assembly operations.
            #[inline]
            fn address(&self) -> Address {
                match self {
                    Self::None => Address::DUMMY,
                    #address_match_arms
                }
            }
        }

        #offset_consts_and_structs
    }
}

// =============================================================================
// walk.rs generation
// =============================================================================

/// Generate the `walk_*` functions.
fn generate_walk(schema: &Schema) -> TokenStream {
    let mut walk_functions = TokenStream::new();

    // Process types in sorted order (matching original JS)
    for type_def in get_sorted_ast_types(schema) {
        match type_def {
            TypeDef::Struct(struct_def) => {
                if let Some(walk_fn) = generate_walk_struct(struct_def, schema) {
                    walk_functions.extend(walk_fn);
                }
            }
            TypeDef::Enum(enum_def) => {
                if let Some(walk_fn) = generate_walk_enum(enum_def, schema) {
                    walk_functions.extend(walk_fn);
                }
            }
            _ => {}
        }
    }

    // Add special walk_statements function
    walk_functions.extend(quote! {
        ///@@line_break
        unsafe fn walk_statements<'a, State, Tr: Traverse<'a, State>>(
            traverser: &mut Tr,
            stmts: *mut Vec<'a, Statement<'a>>,
            ctx: &mut TraverseCtx<'a, State>
        ) {
            traverser.enter_statements(&mut *stmts, ctx);
            for stmt in &mut *stmts {
                walk_statement(traverser, stmt, ctx);
            }
            traverser.exit_statements(&mut *stmts, ctx);
        }
    });

    quote! {
        #![expect(
            clippy::semicolon_if_nothing_returned,
            clippy::ptr_as_ptr,
            clippy::ref_as_ptr,
            clippy::cast_ptr_alignment,
            clippy::borrow_as_ptr,
            clippy::match_same_arms,
            unsafe_op_in_unsafe_fn
        )]

        ///@@line_break
        use std::{cell::Cell, marker::PhantomData};

        ///@@line_break
        use oxc_allocator::Vec;
        use oxc_ast::ast::*;
        use oxc_syntax::scope::ScopeId;

        ///@@line_break
        use crate::{Ancestor, Traverse, TraverseCtx, ancestor::{self, AncestorType}};

        ///@@line_break
        /// Walk AST with `Traverse` impl.
        ///
        /// # SAFETY
        /// * `program` must be a pointer to a valid `Program` which has lifetime `'a`
        ///   (`Program<'a>`).
        /// * `ctx` must contain a `TraverseAncestry<'a>` with single `Ancestor::None` on its stack.
        #[inline]
        pub unsafe fn walk_ast<'a, State, Tr: Traverse<'a, State>>(
            traverser: &mut Tr,
            program: *mut Program<'a>,
            ctx: &mut TraverseCtx<'a, State>,
        ) {
            walk_program(traverser, program, ctx);
        }

        #walk_functions
    }
}

/// Generate walk function for a struct.
fn generate_walk_struct(struct_def: &StructDef, schema: &Schema) -> Option<TokenStream> {
    struct_def.visit.visitor_names.as_ref()?;

    let type_name = struct_def.name();
    // Use type WITH lifetime for walk function signature
    let type_param_ty = struct_def.ty(schema);
    let snake_name = camel_to_snake(type_name);
    let walk_fn_ident = format_ident!("walk_{snake_name}");
    let enter_fn_ident = format_ident!("enter_{snake_name}");
    let exit_fn_ident = format_ident!("exit_{snake_name}");

    let type_screaming_name = camel_to_screaming(type_name);

    // Check if this struct has a scope
    let scope_id_field = struct_def.fields.iter().find(|f| {
        f.name() == "scope_id"
            && matches!(f.type_def(schema), TypeDef::Cell(cell)
                if matches!(cell.inner_type(schema), TypeDef::Option(_)))
    });

    // Get visited fields with their indices (only fields with AST type visitors)
    let visited_fields_with_index: Vec<_> = struct_def
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field_has_ast_visitor(field, schema))
        .collect();

    // Generate scope handling code
    let (enter_scope, exit_scope, scope_entry_index, scope_exit_index) =
        generate_scope_code(struct_def, scope_id_field, &type_screaming_name);

    // Generate field visits
    let mut field_visits = TokenStream::new();
    let mut is_first = true;

    for (field_index, (original_index, field)) in visited_fields_with_index.iter().enumerate() {
        let field_visit = generate_field_visit(
            struct_def,
            field,
            *original_index,
            field_index,
            is_first,
            scope_entry_index,
            scope_exit_index,
            &enter_scope,
            &exit_scope,
            schema,
        );
        field_visits.extend(field_visit);
        is_first = false;
    }

    // Pop stack at end if we pushed (before scope exit)
    if !visited_fields_with_index.is_empty() {
        field_visits.extend(quote! {
            ctx.pop_stack(pop_token);
        });
    }

    // Add scope exit at end if not already exited
    if let Some(scope_exit_idx) = scope_exit_index
        && scope_exit_idx >= struct_def.fields.len()
    {
        field_visits.extend(exit_scope);
    }

    Some(quote! {
        ///@@line_break
        unsafe fn #walk_fn_ident<'a, State, Tr: Traverse<'a, State>>(
            traverser: &mut Tr,
            node: *mut #type_param_ty,
            ctx: &mut TraverseCtx<'a, State>
        ) {
            traverser.#enter_fn_ident(&mut *node, ctx);
            #field_visits
            traverser.#exit_fn_ident(&mut *node, ctx);
        }
    })
}

/// Generate scope entry/exit code.
fn generate_scope_code(
    struct_def: &StructDef,
    scope_id_field: Option<&FieldDef>,
    type_screaming_name: &str,
) -> (TokenStream, TokenStream, Option<usize>, Option<usize>) {
    let Some(scope) = &struct_def.visit.scope else {
        return (quote!(), quote!(), None, None);
    };

    let Some(_scope_id_field) = scope_id_field else {
        return (quote!(), quote!(), None, None);
    };

    let offset_const = format_ident!("OFFSET_{type_screaming_name}_SCOPE_ID");

    let mut enter_scope = quote! {
        let previous_scope_id = ctx.current_scope_id();
        let current_scope_id = (*(
            (node as *mut u8).add(ancestor::#offset_const)
            as *mut Cell<Option<ScopeId>>
        )).get().unwrap();
        ctx.set_current_scope_id(current_scope_id);
    };

    let mut exit_scope = quote! {
        ctx.set_current_scope_id(previous_scope_id);
    };

    // Check if this is a var hoisting scope
    let type_name = struct_def.name();
    let is_var_hoisting_scope = type_name == "Function"
        || scope.flags.contains("Top")
        || scope.flags.contains("Function")
        || scope.flags.contains("ClassStaticBlock")
        || scope.flags.contains("TsModuleBlock");

    if is_var_hoisting_scope {
        enter_scope.extend(quote! {
            let previous_hoist_scope_id = ctx.current_hoist_scope_id();
            ctx.set_current_hoist_scope_id(current_scope_id);
        });
        exit_scope.extend(quote! {
            ctx.set_current_hoist_scope_id(previous_hoist_scope_id);
        });
    }

    // Check if this is a block scope
    let is_block_scope = matches!(
        type_name,
        "Program"
            | "BlockStatement"
            | "Function"
            | "ArrowFunctionExpression"
            | "StaticBlock"
            | "TSModuleDeclaration"
            | "TSGlobalDeclaration"
    );

    if is_block_scope {
        enter_scope.extend(quote! {
            let previous_block_scope_id = ctx.current_block_scope_id();
            ctx.set_current_block_scope_id(current_scope_id);
        });
        exit_scope.extend(quote! {
            ctx.set_current_block_scope_id(previous_block_scope_id);
        });
    }

    (enter_scope, exit_scope, Some(scope.enter_before_index), Some(scope.exit_before_index))
}

/// Generate code to visit a struct field.
fn generate_field_visit(
    struct_def: &StructDef,
    field: &FieldDef,
    original_field_index: usize,
    visited_field_index: usize,
    is_first: bool,
    scope_entry_index: Option<usize>,
    scope_exit_index: Option<usize>,
    enter_scope: &TokenStream,
    exit_scope: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    let type_name = struct_def.name();
    let type_screaming_name = camel_to_screaming(type_name);
    let field_name = field.name();
    let field_camel_name = field_name.to_case(Case::UpperCamel);
    let field_screaming_name = field_name.to_case(Case::UpperSnake);
    let offset_const = format_ident!("OFFSET_{type_screaming_name}_{field_screaming_name}");
    let variant_name = format!("{type_name}{field_camel_name}");
    let variant_ident = create_safe_ident(&variant_name);
    let struct_without_ident = format_ident!("{type_name}Without{field_camel_name}");

    let mut result = TokenStream::new();

    // Insert scope entry if needed
    if let Some(entry_idx) = scope_entry_index {
        if entry_idx <= original_field_index && visited_field_index == 0 {
            // Only insert on first visited field that's at or after entry index
        } else if entry_idx == original_field_index {
            result.extend(enter_scope.clone());
        }
    }

    // Insert scope exit if needed before this field
    if let Some(exit_idx) = scope_exit_index
        && exit_idx == original_field_index
    {
        result.extend(exit_scope.clone());
    }

    // Generate stack management
    if is_first {
        result.extend(quote! {
            let pop_token = ctx.push_stack(
                Ancestor::#variant_ident(
                    ancestor::#struct_without_ident(node, PhantomData)
                )
            );
        });

        // If scope should be entered before first visited field, add it after push_stack
        if let Some(entry_idx) = scope_entry_index
            && entry_idx <= original_field_index
        {
            // Insert scope entry at the beginning of the result
            let mut new_result = enter_scope.clone();
            new_result.extend(result);
            result = new_result;
        }
    } else {
        // For optional fields, retag_stack goes inside the if block
        // For non-optional fields, retag_stack goes outside
        let field_type = field.type_def(schema);
        if !matches!(field_type, TypeDef::Option(_)) {
            let ancestor_type_ident = format_ident!("AncestorType");
            result.extend(quote! {
                ctx.retag_stack(#ancestor_type_ident::#variant_ident);
            });
        }
    }

    // Generate the actual visit code
    let field_type = field.type_def(schema);
    let retag_code = if !is_first && matches!(field_type, TypeDef::Option(_)) {
        let ancestor_type_ident = format_ident!("AncestorType");
        Some(quote! { ctx.retag_stack(#ancestor_type_ident::#variant_ident); })
    } else {
        None
    };
    let visit_code =
        generate_visit_code_for_type(field_type, &offset_const, retag_code.as_ref(), schema);
    result.extend(visit_code);

    result
}

/// Generate visit code for a specific type.
/// `retag_code` is optional code to insert inside the if block for optional types.
fn generate_visit_code_for_type(
    type_def: &TypeDef,
    offset_const: &syn::Ident,
    retag_code: Option<&TokenStream>,
    schema: &Schema,
) -> TokenStream {
    match type_def {
        TypeDef::Struct(struct_def) => {
            let snake_name = camel_to_snake(struct_def.name());
            let walk_fn = format_ident!("walk_{snake_name}");
            let field_ty = ty_without_lifetime(type_def, schema);
            quote! {
                #walk_fn(traverser, (node as *mut u8).add(ancestor::#offset_const) as *mut #field_ty, ctx);
            }
        }
        TypeDef::Enum(enum_def) => {
            let snake_name = camel_to_snake(enum_def.name());
            let walk_fn = format_ident!("walk_{snake_name}");
            let field_ty = ty_without_lifetime(type_def, schema);
            quote! {
                #walk_fn(traverser, (node as *mut u8).add(ancestor::#offset_const) as *mut #field_ty, ctx);
            }
        }
        TypeDef::Option(option_def) => {
            let inner_type = option_def.inner_type(schema);
            let option_ty = ty_without_lifetime(type_def, schema);

            match inner_type {
                TypeDef::Vec(vec_def) => {
                    let inner_inner = vec_def.inner_type(schema);
                    let is_statement_vec = match inner_inner {
                        TypeDef::Enum(e) => e.name() == "Statement",
                        _ => false,
                    };
                    if is_statement_vec {
                        return quote! {
                            if let Some(field) = &mut *((node as *mut u8).add(ancestor::#offset_const) as *mut #option_ty) {
                                #retag_code
                                walk_statements(traverser, field as *mut _, ctx);
                            }
                        };
                    }
                    let walk_fn = get_walk_fn_for_inner(inner_inner);
                    quote! {
                        if let Some(field) = &mut *((node as *mut u8).add(ancestor::#offset_const) as *mut #option_ty) {
                            #retag_code
                            for item in field.iter_mut() {
                                #walk_fn(traverser, item as *mut _, ctx);
                            }
                        }
                    }
                }
                TypeDef::Box(box_def) => {
                    let unboxed = box_def.inner_type(schema);
                    let walk_fn = get_walk_fn_for_inner(unboxed);
                    quote! {
                        if let Some(field) = &mut *((node as *mut u8).add(ancestor::#offset_const) as *mut #option_ty) {
                            #retag_code
                            #walk_fn(traverser, (&mut **field) as *mut _, ctx);
                        }
                    }
                }
                _ => {
                    let walk_fn = get_walk_fn_for_inner(inner_type);
                    quote! {
                        if let Some(field) = &mut *((node as *mut u8).add(ancestor::#offset_const) as *mut #option_ty) {
                            #retag_code
                            #walk_fn(traverser, field as *mut _, ctx);
                        }
                    }
                }
            }
        }
        TypeDef::Vec(vec_def) => {
            let vec_ty = ty_without_lifetime(type_def, schema);
            let inner_type = vec_def.inner_type(schema);

            // Special case for Vec<Statement>
            let is_statement_vec = match inner_type {
                TypeDef::Enum(e) => e.name() == "Statement",
                _ => false,
            };
            if is_statement_vec {
                return quote! {
                    walk_statements(traverser, (node as *mut u8).add(ancestor::#offset_const) as *mut #vec_ty, ctx);
                };
            }

            // Check if inner is Option
            if let TypeDef::Option(opt) = inner_type {
                let inner_inner = opt.inner_type(schema);
                let walk_fn = get_walk_fn_for_inner(inner_inner);
                return quote! {
                    for item in (*(
                        (node as *mut u8).add(ancestor::#offset_const) as *mut #vec_ty
                    )).iter_mut().flatten() {
                        #walk_fn(traverser, item as *mut _, ctx);
                    }
                };
            }

            let walk_fn = get_walk_fn_for_inner(inner_type);
            quote! {
                for item in &mut *((node as *mut u8).add(ancestor::#offset_const) as *mut #vec_ty) {
                    #walk_fn(traverser, item as *mut _, ctx);
                }
            }
        }
        TypeDef::Box(box_def) => {
            let box_ty = ty_without_lifetime(type_def, schema);
            let inner_type = box_def.inner_type(schema);
            let walk_fn = get_walk_fn_for_inner(inner_type);
            quote! {
                #walk_fn(traverser, (&mut **(
                    (node as *mut u8).add(ancestor::#offset_const) as *mut #box_ty
                )) as *mut _, ctx);
            }
        }
        _ => quote!(),
    }
}

/// Get walk function ident for an inner type.
fn get_walk_fn_for_inner(type_def: &TypeDef) -> syn::Ident {
    match type_def {
        TypeDef::Struct(def) => {
            let snake_name = camel_to_snake(def.name());
            format_ident!("walk_{snake_name}")
        }
        TypeDef::Enum(def) => {
            let snake_name = camel_to_snake(def.name());
            format_ident!("walk_{snake_name}")
        }
        TypeDef::Box(_) => {
            // Recursively get inner
            panic!("Unexpected Box in get_walk_fn_for_inner")
        }
        _ => panic!("Unexpected type in get_walk_fn_for_inner: {type_def:?}"),
    }
}

/// Generate walk function for an enum.
fn generate_walk_enum(enum_def: &EnumDef, schema: &Schema) -> Option<TokenStream> {
    enum_def.visit.visitor_names.as_ref()?;

    let enum_name = enum_def.name();
    let enum_ident = enum_def.ident();
    // Use type WITH lifetime for walk function signature
    let enum_param_ty = enum_def.ty(schema);
    let snake_name = camel_to_snake(enum_name);
    let walk_fn_ident = format_ident!("walk_{snake_name}");
    let enter_fn_ident = format_ident!("enter_{snake_name}");
    let exit_fn_ident = format_ident!("exit_{snake_name}");

    let mut match_arms = TokenStream::new();

    // Own variants
    for variant in &enum_def.variants {
        let variant_ident = variant.ident();
        let Some(field_type) = variant.field_type(schema) else { continue };

        let inner_type = get_inner_type(field_type, schema);

        // Only include variants with inner types from the AST module
        if !is_ast_type(inner_type, schema) {
            continue;
        }

        let (node_expr, walk_fn) = match field_type {
            TypeDef::Box(_) => {
                let walk_fn = match inner_type {
                    TypeDef::Struct(def) => {
                        let snake_name = camel_to_snake(def.name());
                        format_ident!("walk_{snake_name}")
                    }
                    TypeDef::Enum(def) => {
                        let snake_name = camel_to_snake(def.name());
                        format_ident!("walk_{snake_name}")
                    }
                    _ => continue,
                };
                (quote!((&mut **node)), walk_fn)
            }
            TypeDef::Struct(def) => {
                if def.visit.visitor_names.is_none() {
                    continue;
                }
                let snake_name = camel_to_snake(def.name());
                (quote!(node), format_ident!("walk_{snake_name}"))
            }
            TypeDef::Enum(def) => {
                if def.visit.visitor_names.is_none() {
                    continue;
                }
                let snake_name = camel_to_snake(def.name());
                (quote!(node), format_ident!("walk_{snake_name}"))
            }
            _ => continue,
        };

        match_arms.extend(quote! {
            #enum_ident::#variant_ident(node) => #walk_fn(traverser, #node_expr as *mut _, ctx),
        });
    }

    // Inherited variants - expand all variants from inherited enums
    for inherits_type in enum_def.inherits_types(schema) {
        let inherits_enum = inherits_type.as_enum().unwrap();

        // Only include inherited enums from the AST module
        if !is_ast_type_enum(inherits_enum, schema) {
            continue;
        }

        let inherits_snake_name = camel_to_snake(inherits_enum.name());
        let inherits_walk_fn = format_ident!("walk_{inherits_snake_name}");

        // Collect all variant names from the inherited enum (including its inherited variants)
        let variant_names = collect_all_variant_names(inherits_enum, schema);

        // Generate match arm pattern with all variants using | separator
        let variant_patterns: Vec<_> = variant_names
            .iter()
            .map(|name| {
                let variant_ident = create_safe_ident(name);
                quote!(#enum_ident::#variant_ident(_))
            })
            .collect();

        match_arms.extend(quote! {
            #(#variant_patterns)|* => #inherits_walk_fn(traverser, node as *mut _, ctx),
        });
    }

    Some(quote! {
        ///@@line_break
        unsafe fn #walk_fn_ident<'a, State, Tr: Traverse<'a, State>>(
            traverser: &mut Tr,
            node: *mut #enum_param_ty,
            ctx: &mut TraverseCtx<'a, State>
        ) {
            traverser.#enter_fn_ident(&mut *node, ctx);
            match &mut *node {
                #match_arms
            }
            traverser.#exit_fn_ident(&mut *node, ctx);
        }
    })
}

// =============================================================================
// Helper functions
// =============================================================================

/// Collect all variant names from an enum, including inherited variants.
/// Uses breadth-first traversal to match the original JS behavior.
fn collect_all_variant_names(enum_def: &EnumDef, schema: &Schema) -> Vec<String> {
    let mut names = Vec::new();
    let mut pending_inherited: Vec<&EnumDef> = vec![];

    // Own variants
    for variant in &enum_def.variants {
        names.push(variant.name().to_string());
    }

    // Collect direct inherited types
    for inherits_type in enum_def.inherits_types(schema) {
        if let TypeDef::Enum(inherited_enum) = inherits_type {
            pending_inherited.push(inherited_enum);
        }
    }

    // Process inherited types breadth-first
    while !pending_inherited.is_empty() {
        let mut next_pending: Vec<&EnumDef> = vec![];

        for inherited_enum in &pending_inherited {
            // Add own variants of this inherited enum
            for variant in &inherited_enum.variants {
                names.push(variant.name().to_string());
            }

            // Queue its inherited types for next level
            for inherits_type in inherited_enum.inherits_types(schema) {
                if let TypeDef::Enum(inner_enum) = inherits_type {
                    next_pending.push(inner_enum);
                }
            }
        }

        pending_inherited = next_pending;
    }

    names
}

/// Get the innermost visited type, unwrapping Box, Option, Vec.
fn get_inner_type<'s>(type_def: &'s TypeDef, schema: &'s Schema) -> &'s TypeDef {
    match type_def {
        TypeDef::Box(box_def) => get_inner_type(box_def.inner_type(schema), schema),
        TypeDef::Option(opt_def) => get_inner_type(opt_def.inner_type(schema), schema),
        TypeDef::Vec(vec_def) => get_inner_type(vec_def.inner_type(schema), schema),
        _ => type_def,
    }
}
