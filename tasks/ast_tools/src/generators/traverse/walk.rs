//! Generator for `walk.rs` in `oxc_traverse` crate.
//!
//! Generates:
//! * `walk_ast` entry point
//! * `walk_statements` special function for `Vec<Statement>`
//! * `walk_*` for each struct (with scope handling, ancestor tagging, field walking)
//! * `walk_*` for each enum (match dispatch to variant walk functions)

use cow_utils::CowUtils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
    utils::{create_ident, upper_case_first},
};

use super::{
    ancestor::{get_visited_fields, is_ast_type_with_visitor},
    traverse_snake_name,
};

pub(super) struct WalkConfig {
    pub use_code: TokenStream,
    pub trait_bound: TokenStream,
    pub ctx_ty: TokenStream,
    pub has_state: bool,
}

impl WalkConfig {
    pub fn traverse() -> Self {
        Self {
            use_code: quote! { use crate::{ancestor::{self, AncestorType}, Ancestor, Traverse, TraverseCtx}; },
            trait_bound: quote! { Tr: Traverse<'a, State> },
            ctx_ty: quote! { TraverseCtx<'a, State> },
            has_state: true,
        }
    }

    pub fn minifier() -> Self {
        Self {
            use_code: quote! {
                use crate::{
                    generated::ancestor::{self, Ancestor, AncestorType},
                    Traverse,
                    TraverseCtx,
                };
            },
            trait_bound: quote! { Tr: Traverse<'a> },
            ctx_ty: quote! { TraverseCtx<'a> },
            has_state: false,
        }
    }
}

/// Generate all walk functions.
pub(super) fn generate_walk(schema: &Schema, config: &WalkConfig) -> TokenStream {
    let mut walk_methods = quote!();
    let use_code = &config.use_code;
    let ctx_ty = &config.ctx_ty;
    let walk_generics = walk_generics_tokens(config);

    for type_def in &schema.types {
        if !is_ast_type_with_visitor(type_def, schema) {
            continue;
        }
        match type_def {
            TypeDef::Struct(struct_def) => {
                walk_methods.extend(generate_walk_for_struct(struct_def, schema, config));
            }
            TypeDef::Enum(enum_def) => {
                walk_methods.extend(generate_walk_for_enum(enum_def, schema, config));
            }
            _ => {}
        }
    }

    quote! {
        #![expect(
            clippy::semicolon_if_nothing_returned,
            clippy::ptr_as_ptr,
            clippy::ref_as_ptr,
            clippy::cast_ptr_alignment,
            clippy::borrow_as_ptr,
            clippy::match_same_arms,
            unsafe_op_in_unsafe_fn,
        )]

        ///@@line_break
        use std::{cell::Cell, marker::PhantomData};

        ///@@line_break
        use oxc_allocator::Vec;
        use oxc_ast::ast::*;
        use oxc_syntax::scope::ScopeId;

        ///@@line_break
        #use_code

        ///@@line_break
        /// Walk AST with `Traverse` impl.
        ///
        /// # SAFETY
        /// * `program` must be a pointer to a valid `Program` which has lifetime `'a`
        ///   (`Program<'a>`).
        /// * `ctx` must contain a `TraverseAncestry<'a>` with single `Ancestor::None` on its stack.
        #[inline]
        pub unsafe fn walk_ast<#walk_generics>(
            traverser: &mut Tr,
            program: *mut Program<'a>,
            ctx: &mut #ctx_ty,
        ) {
            walk_program(traverser, program, ctx);
        }

        #walk_methods

        ///@@line_break
        unsafe fn walk_statements<#walk_generics>(
            traverser: &mut Tr,
            stmts: *mut Vec<'a, Statement<'a>>,
            ctx: &mut #ctx_ty,
        ) {
            traverser.enter_statements(&mut *stmts, ctx);
            for stmt in &mut *stmts {
                walk_statement(traverser, stmt, ctx);
            }
            traverser.exit_statements(&mut *stmts, ctx);
        }
    }
}

fn walk_generics_tokens(config: &WalkConfig) -> TokenStream {
    let trait_bound = &config.trait_bound;
    if config.has_state {
        quote! { 'a, State, #trait_bound }
    } else {
        quote! { 'a, #trait_bound }
    }
}

/// Generate walk function for a struct type.
fn generate_walk_for_struct(
    struct_def: &StructDef,
    schema: &Schema,
    config: &WalkConfig,
) -> TokenStream {
    let walk_generics = walk_generics_tokens(config);
    let ctx_ty = &config.ctx_ty;
    let visitor_names = struct_def.visit.visitor_names.as_ref().unwrap();
    let snake_name = traverse_snake_name(visitor_names);
    let walk_fn_name = format_ident!("walk_{snake_name}");
    let enter_fn_name = format_ident!("enter_{snake_name}");
    let exit_fn_name = format_ident!("exit_{snake_name}");
    let struct_ty = struct_def.ty(schema);

    let visited_fields = get_visited_fields(struct_def, schema);

    // Scope handling
    let (enter_scope_code, exit_scope_code, scope_id_field_exists) =
        generate_scope_code(struct_def);

    // Determine which field to enter/exit scope before
    let scope = struct_def.visit.scope.as_ref();
    let scope_enter_index = scope.map(|s| s.enter_before_index);
    let scope_exit_index = scope.map(|s| s.exit_before_index);

    // Generate field walking code
    let mut fields_code = quote!();
    let mut scope_entered = false;
    let mut scope_exited = false;

    for (i, (field_index, field)) in visited_fields.iter().enumerate() {
        // Scope entry before this field
        if scope_id_field_exists
            && !scope_entered
            && scope_enter_index.is_some_and(|idx| idx <= *field_index)
        {
            fields_code.extend(enter_scope_code.clone());
            scope_entered = true;
        }

        // Scope exit before this field
        if scope_id_field_exists
            && scope_entered
            && !scope_exited
            && scope_exit_index.is_some_and(|idx| idx <= *field_index)
        {
            fields_code.extend(exit_scope_code.clone());
            scope_exited = true;
        }

        let field_camel_name = upper_case_first(&field.camel_name()).into_owned();
        let variant_ident = format_ident!("{}{}", struct_def.name(), field_camel_name);
        let without_struct_ident =
            format_ident!("{}Without{}", struct_def.name(), field_camel_name);

        let field_type = field.type_def(schema);
        let is_option = field_type.is_option();

        // Ancestor stack handling + field walk code
        // For the first field: push_stack before the field walk (outside any Option guard)
        // For subsequent fields:
        //   - If field is Option: retag_stack goes INSIDE the if let Some
        //   - If field is not Option: retag_stack goes before the field walk
        if i == 0 {
            fields_code.extend(quote! {
                let pop_token = ctx.push_stack(
                    Ancestor::#variant_ident(ancestor::#without_struct_ident(node, PhantomData))
                );
            });
            let field_walk_code = generate_field_walk(struct_def, field, schema);
            fields_code.extend(field_walk_code);
        } else if is_option {
            // For Option fields: retag goes inside the if let Some block
            let retag = quote! { ctx.retag_stack(AncestorType::#variant_ident); };
            let field_walk_code =
                generate_field_walk_option_with_retag(struct_def, field, schema, &retag);
            fields_code.extend(field_walk_code);
        } else {
            fields_code.extend(quote! {
                ctx.retag_stack(AncestorType::#variant_ident);
            });
            let field_walk_code = generate_field_walk(struct_def, field, schema);
            fields_code.extend(field_walk_code);
        }
    }

    // Pop stack after all fields
    if !visited_fields.is_empty() {
        fields_code.extend(quote! { ctx.pop_stack(pop_token); });
    }

    // Scope entry/exit if not yet done
    if scope_id_field_exists && !scope_entered {
        fields_code.extend(enter_scope_code);
        scope_entered = true;
    }

    // Exit scope after all fields if scope_exit_index is past last field
    let exit_scope_after = if scope_id_field_exists && scope_entered && !scope_exited {
        exit_scope_code
    } else {
        quote!()
    };

    quote! {
        ///@@line_break
        unsafe fn #walk_fn_name<#walk_generics>(
            traverser: &mut Tr,
            node: *mut #struct_ty,
            ctx: &mut #ctx_ty,
        ) {
            traverser.#enter_fn_name(&mut *node, ctx);
            #fields_code
            #exit_scope_after
            traverser.#exit_fn_name(&mut *node, ctx);
        }
    }
}

/// Generate scope enter/exit code for a struct.
fn generate_scope_code(struct_def: &StructDef) -> (TokenStream, TokenStream, bool) {
    let Some(scope) = &struct_def.visit.scope else {
        return (quote!(), quote!(), false);
    };

    // Check that the struct has a scope_id field
    let has_scope_id = struct_def.fields.iter().any(|f| f.name() == "scope_id");
    if !has_scope_id {
        return (quote!(), quote!(), false);
    }

    let type_snake_name = struct_def.snake_name();
    let type_screaming_name = type_snake_name.cow_to_ascii_uppercase();
    let scope_id_offset = format_ident!("OFFSET_{}_{}", type_screaming_name, "SCOPE_ID");

    let mut enter_code = quote! {
        let previous_scope_id = ctx.current_scope_id();
        let current_scope_id = (*((node as *mut u8).add(ancestor::#scope_id_offset)
            as *mut Cell<Option<ScopeId>>)).get().unwrap();
        ctx.set_current_scope_id(current_scope_id);
    };

    let mut exit_code = quote! {
        ctx.set_current_scope_id(previous_scope_id);
    };

    // Determine if this is a var-hoisting scope
    let is_var_hoisting_scope = struct_def.name() == "Function"
        || scope.flags.contains("Top")
        || scope.flags.contains("Function")
        || scope.flags.contains("ClassStaticBlock")
        || scope.flags.contains("TsModuleBlock");
    if is_var_hoisting_scope {
        enter_code.extend(quote! {
            let previous_hoist_scope_id = ctx.current_hoist_scope_id();
            ctx.set_current_hoist_scope_id(current_scope_id);
        });
        exit_code.extend(quote! {
            ctx.set_current_hoist_scope_id(previous_hoist_scope_id);
        });
    }

    // Determine if this is a block scope
    let is_block_scope = matches!(
        struct_def.name(),
        "Program"
            | "BlockStatement"
            | "Function"
            | "ArrowFunctionExpression"
            | "StaticBlock"
            | "TSModuleDeclaration"
            | "TSGlobalDeclaration"
    );
    if is_block_scope {
        enter_code.extend(quote! {
            let previous_block_scope_id = ctx.current_block_scope_id();
            ctx.set_current_block_scope_id(current_scope_id);
        });
        exit_code.extend(quote! {
            ctx.set_current_block_scope_id(previous_block_scope_id);
        });
    }

    (enter_code, exit_code, true)
}

/// Generate the walk code for a single struct field.
fn generate_field_walk(struct_def: &StructDef, field: &FieldDef, schema: &Schema) -> TokenStream {
    let type_snake_name = struct_def.snake_name();
    let type_screaming_name = type_snake_name.cow_to_ascii_uppercase();
    let field_offset =
        format_ident!("OFFSET_{}_{}", type_screaming_name, field.name().cow_to_ascii_uppercase());

    let field_type = field.type_def(schema);
    let field_type_name = ty_no_lifetime(field_type, schema);

    generate_field_walk_inner(field_type, &field_offset, &field_type_name, schema)
}

/// Generate walk code for a field, handling wrapper types (Option, Vec, Box).
fn generate_field_walk_inner(
    type_def: &TypeDef,
    field_offset: &syn::Ident,
    field_type_name: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    match type_def {
        TypeDef::Option(option_def) => {
            let inner_type = option_def.inner_type(schema);
            match inner_type {
                TypeDef::Vec(vec_def) => {
                    let vec_inner = vec_def.inner_type(schema);
                    let innermost = vec_inner.innermost_type(schema);
                    let inner_snake = innermost.snake_name();
                    // Special case for Option<Vec<Statement>>
                    if inner_snake == "statement" {
                        quote! {
                            if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                                walk_statements(traverser, field as *mut _, ctx);
                            }
                        }
                    } else {
                        let walk_fn = format_ident!("walk_{inner_snake}");
                        quote! {
                            if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                                for item in field.iter_mut() {
                                    #walk_fn(traverser, item as *mut _, ctx);
                                }
                            }
                        }
                    }
                }
                TypeDef::Box(box_def) => {
                    let unboxed = box_def.inner_type(schema);
                    let innermost = unboxed.innermost_type(schema);
                    let inner_snake = innermost.snake_name();
                    let walk_fn = format_ident!("walk_{inner_snake}");
                    quote! {
                        if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                            #walk_fn(traverser, (&mut **field) as *mut _, ctx);
                        }
                    }
                }
                _ => {
                    let innermost = inner_type.innermost_type(schema);
                    let inner_snake = innermost.snake_name();
                    let walk_fn = format_ident!("walk_{inner_snake}");
                    quote! {
                        if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                            #walk_fn(traverser, field as *mut _, ctx);
                        }
                    }
                }
            }
        }
        TypeDef::Vec(vec_def) => {
            let inner_type = vec_def.inner_type(schema);
            let innermost = inner_type.innermost_type(schema);
            let inner_snake = innermost.snake_name();

            // Special case for Vec<Statement>
            if inner_snake == "statement" {
                return quote! {
                    walk_statements(traverser, (node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name, ctx);
                };
            }

            let walk_fn = format_ident!("walk_{inner_snake}");
            // Handle Vec<Option<T>>
            if inner_type.is_option() {
                quote! {
                    for item in (*((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name)).iter_mut().flatten() {
                        #walk_fn(traverser, item as *mut _, ctx);
                    }
                }
            } else {
                quote! {
                    for item in &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                        #walk_fn(traverser, item as *mut _, ctx);
                    }
                }
            }
        }
        TypeDef::Box(box_def) => {
            let inner_type = box_def.inner_type(schema);
            let innermost = inner_type.innermost_type(schema);
            let inner_snake = innermost.snake_name();
            let walk_fn = format_ident!("walk_{inner_snake}");
            quote! {
                #walk_fn(traverser, (&mut **((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name)) as *mut _, ctx);
            }
        }
        TypeDef::Struct(_) | TypeDef::Enum(_) => {
            let innermost = type_def.innermost_type(schema);
            let inner_snake = innermost.snake_name();
            let walk_fn = format_ident!("walk_{inner_snake}");
            quote! {
                #walk_fn(traverser, (node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name, ctx);
            }
        }
        _ => quote!(),
    }
}

/// Generate walk code for an Option field with retag_stack inside the `if let Some`.
///
/// For non-first Option fields, the JS codegen places `retag_stack` INSIDE the `if let Some` block:
/// ```ignore
/// if let Some(field) = &mut *(...) {
///     ctx.retag_stack(AncestorType::...);
///     walk_inner(traverser, field as *mut _, ctx);
/// }
/// ```
fn generate_field_walk_option_with_retag(
    struct_def: &StructDef,
    field: &FieldDef,
    schema: &Schema,
    retag: &TokenStream,
) -> TokenStream {
    let type_snake_name = struct_def.snake_name();
    let type_screaming_name = type_snake_name.cow_to_ascii_uppercase();
    let field_offset =
        format_ident!("OFFSET_{}_{}", type_screaming_name, field.name().cow_to_ascii_uppercase());

    let field_type = field.type_def(schema);
    let field_type_name = ty_no_lifetime(field_type, schema);

    let TypeDef::Option(option_def) = field_type else {
        unreachable!("generate_field_walk_option_with_retag called on non-Option field");
    };

    let inner_type = option_def.inner_type(schema);
    match inner_type {
        TypeDef::Vec(vec_def) => {
            let vec_inner = vec_def.inner_type(schema);
            let innermost = vec_inner.innermost_type(schema);
            let inner_snake = innermost.snake_name();
            if inner_snake == "statement" {
                quote! {
                    if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                        #retag
                        walk_statements(traverser, field as *mut _, ctx);
                    }
                }
            } else {
                let walk_fn = format_ident!("walk_{inner_snake}");
                quote! {
                    if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                        #retag
                        for item in field.iter_mut() {
                            #walk_fn(traverser, item as *mut _, ctx);
                        }
                    }
                }
            }
        }
        TypeDef::Box(box_def) => {
            let unboxed = box_def.inner_type(schema);
            let innermost = unboxed.innermost_type(schema);
            let inner_snake = innermost.snake_name();
            let walk_fn = format_ident!("walk_{inner_snake}");
            quote! {
                if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                    #retag
                    #walk_fn(traverser, (&mut **field) as *mut _, ctx);
                }
            }
        }
        _ => {
            let innermost = inner_type.innermost_type(schema);
            let inner_snake = innermost.snake_name();
            let walk_fn = format_ident!("walk_{inner_snake}");
            quote! {
                if let Some(field) = &mut *((node as *mut u8).add(ancestor::#field_offset) as *mut #field_type_name) {
                    #retag
                    #walk_fn(traverser, field as *mut _, ctx);
                }
            }
        }
    }
}

/// Generate walk function for an enum type.
fn generate_walk_for_enum(enum_def: &EnumDef, schema: &Schema, config: &WalkConfig) -> TokenStream {
    let walk_generics = walk_generics_tokens(config);
    let ctx_ty = &config.ctx_ty;
    let visitor_names = enum_def.visit.visitor_names.as_ref().unwrap();
    let snake_name = traverse_snake_name(visitor_names);
    let walk_fn_name = format_ident!("walk_{snake_name}");
    let enter_fn_name = format_ident!("enter_{snake_name}");
    let exit_fn_name = format_ident!("exit_{snake_name}");
    let enum_ty = enum_def.ty(schema);
    let enum_ident = enum_def.ident();

    let mut match_arms = quote!();

    // Own variants
    for variant in &enum_def.variants {
        let Some(field_type) = variant.field_type(schema) else { continue };
        let inner_type = field_type.innermost_type(schema);

        // Check inner type has a visitor
        let has_visitor = match inner_type {
            TypeDef::Struct(s) => s.visit.has_visitor(),
            TypeDef::Enum(e) => e.visit.has_visitor(),
            _ => false,
        };
        if !has_visitor {
            continue;
        }

        let inner_snake = inner_type.snake_name();
        let walk_fn = format_ident!("walk_{inner_snake}");
        let variant_ident = variant.ident();

        let node_expr = if field_type.is_box() {
            quote!((&mut **node) as *mut _)
        } else {
            quote!(node as *mut _)
        };

        match_arms.extend(quote! {
            #enum_ident::#variant_ident(node) => #walk_fn(traverser, #node_expr, ctx),
        });
    }

    // Inherited variants
    for inherits_type in enum_def.inherits_types(schema) {
        let inherited_enum = inherits_type.as_enum().unwrap();
        let inherited_snake = inherited_enum.snake_name();
        let walk_inherited_fn = format_ident!("walk_{inherited_snake}");

        // Collect all variant patterns from inherited enum (recursively)
        let patterns = collect_inherited_patterns(&enum_ident, inherited_enum, schema);
        match_arms.extend(quote! {
            #(#patterns)|* => #walk_inherited_fn(traverser, node as *mut _, ctx),
        });
    }

    quote! {
        ///@@line_break
        unsafe fn #walk_fn_name<#walk_generics>(
            traverser: &mut Tr,
            node: *mut #enum_ty,
            ctx: &mut #ctx_ty,
        ) {
            traverser.#enter_fn_name(&mut *node, ctx);
            match &mut *node {
                #match_arms
            }
            traverser.#exit_fn_name(&mut *node, ctx);
        }
    }
}

/// Collect all variant match patterns for inherited enums using BFS ordering.
///
/// This matches the old JS output ordering: at each level, collect own variants from all
/// inherited enums first, then recurse into their sub-inherited enums.
fn collect_inherited_patterns(
    parent_ident: &syn::Ident,
    inherited_enum: &EnumDef,
    schema: &Schema,
) -> Vec<TokenStream> {
    let mut patterns = Vec::new();
    let mut queue = vec![inherited_enum];

    while !queue.is_empty() {
        let mut next_queue = Vec::new();
        for enum_def in queue {
            for variant in &enum_def.variants {
                let variant_ident = variant.ident();
                patterns.push(quote!(#parent_ident::#variant_ident(_)));
            }
            for inherits_type in enum_def.inherits_types(schema) {
                let nested_enum = inherits_type.as_enum().unwrap();
                next_queue.push(nested_enum);
            }
        }
        queue = next_queue;
    }

    patterns
}

/// Generate a type signature without lifetimes.
///
/// Matches the JS behavior: `rawTypeName.replace(/<'a>/g, "").replace(/<'a, ?/g, "<")`
/// e.g. `Box<'a, TSTypeAnnotation<'a>>` -> `Box<TSTypeAnnotation>`
/// e.g. `Vec<'a, Statement<'a>>` -> `Vec<Statement>`
/// e.g. `Option<Box<'a, FunctionBody<'a>>>` -> `Option<Box<FunctionBody>>`
fn ty_no_lifetime(type_def: &TypeDef, schema: &Schema) -> TokenStream {
    match type_def {
        TypeDef::Struct(s) => {
            let ident = create_ident(s.name());
            quote!(#ident)
        }
        TypeDef::Enum(e) => {
            let ident = create_ident(e.name());
            quote!(#ident)
        }
        TypeDef::Option(opt) => {
            let inner = ty_no_lifetime(opt.inner_type(schema), schema);
            quote!(Option<#inner>)
        }
        TypeDef::Box(b) => {
            let inner = ty_no_lifetime(b.inner_type(schema), schema);
            quote!(Box<#inner>)
        }
        TypeDef::Vec(v) => {
            let inner = ty_no_lifetime(v.inner_type(schema), schema);
            quote!(Vec<#inner>)
        }
        TypeDef::Cell(c) => {
            let inner = ty_no_lifetime(&schema.types[c.inner_type_id], schema);
            quote!(Cell<#inner>)
        }
        TypeDef::Primitive(p) => {
            let ident = create_ident(p.name());
            quote!(#ident)
        }
        TypeDef::Pointer(_) => quote!(),
    }
}
