//! Generator for `ChildScopeCollector` visitor.

use std::iter;

use proc_macro2::TokenStream;
use quote::quote;

use oxc_index::IndexVec;

use crate::{
    Codegen, Generator, TRAVERSE_CRATE_PATH,
    output::{Output, output_path},
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, TypeId},
    utils::{create_ident, create_ident_tokens},
};

use super::{
    define_generator,
    visit::{INLINE_LIMIT, Target, VisitorOutputs, generate_visit_type},
};

/// Generator for `ChildScopeCollector` visitor.
pub struct ScopesCollectorGenerator;

define_generator!(ScopesCollectorGenerator);

impl Generator for ScopesCollectorGenerator {
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        ScopesCalculator::new(schema).calculate_all();
    }

    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        Output::Rust {
            path: output_path(TRAVERSE_CRATE_PATH, "scopes_collector.rs"),
            tokens: generate(schema),
        }
    }
}

/// State of calculation for whether a type contains a scope.
///
/// As the AST is a cyclic graph, need to avoid infinite loops.
///
/// * When start calculating for a type, state is set to `Calculating`.
/// * If can definitively determine whether type contains a scope, state is set to `Calculated`.
///   * `true` if struct has a scope itself, or any struct field / enum variant contains a scope.
///   * `false` if all struct fields / enum variants can be definitively determined not to contain a scope.
/// * If hit a circular reference, state is set back to `NotCalculated`.
///   It will be possible to determine definitively whether type contains a scope later on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CalculationState {
    Calculated(bool),
    NotCalculated,
    Calculating,
}

/// Calculator of which types contain scopes.
///
/// `ScopesCalculator::new(schema).calculate_all()` calculates which types contain scopes,
/// and sets `def.visit.contains_scope = true` on [`StructDef`]s and [`EnumDef`]s which do.
struct ScopesCalculator<'s> {
    calculation_states: IndexVec<TypeId, CalculationState>,
    schema: &'s mut Schema,
}

impl<'s> ScopesCalculator<'s> {
    /// Create [`ScopesCalculator`].
    fn new(schema: &'s mut Schema) -> Self {
        // Create bitset indexed by `TypeId` tracking the calculation state of each type
        let calculation_states = IndexVec::<TypeId, _>::from_vec(
            iter::repeat_n(CalculationState::NotCalculated, schema.types.len()).collect::<Vec<_>>(),
        );

        Self { calculation_states, schema }
    }

    /// Calculate whether all types contain a scope or not.
    fn calculate_all(&mut self) {
        for type_id in self.schema.types.indices() {
            let state = self.calculate_type(type_id);
            if state == CalculationState::NotCalculated {
                // No scope found for any of children, but hit a circular reference.
                // As this was the starting point of traversal through AST graph,
                // circular references can't contain a scope. So this type does not contain a scope.
                self.calculation_states[type_id] = CalculationState::Calculated(false);
            }
        }
    }

    /// Calculate if a type contains a scope (either its own, or in a child).
    ///
    /// Returns:
    /// * `Calculated(true)` if definitely contains a scope.
    /// * `Calculated(false)` if definitely does not contain a scope.
    /// * `NotCalculated` if not possible to determine definitively due to circular dependency.
    fn calculate_type(&mut self, type_id: TypeId) -> CalculationState {
        match self.calculation_states[type_id] {
            // Already calculated. Return result.
            state @ CalculationState::Calculated(_) => return state,
            // Currently calculating. Exit to avoid infinite loop.
            CalculationState::Calculating => return CalculationState::NotCalculated,
            // Not yet calculated. Calculate now.
            CalculationState::NotCalculated => {}
        }

        // Flag that currently calculating
        self.calculation_states[type_id] = CalculationState::Calculating;

        let state = match &self.schema.types[type_id] {
            TypeDef::Struct(_) => self.calculate_struct(type_id),
            TypeDef::Enum(_) => self.calculate_enum(type_id),
            // Primitives don't have scopes
            TypeDef::Primitive(_) => CalculationState::Calculated(false),
            // Containers contain a scope if their inner type does
            TypeDef::Option(option_def) => self.calculate_type(option_def.inner_type_id),
            TypeDef::Box(box_def) => self.calculate_type(box_def.inner_type_id),
            TypeDef::Vec(vec_def) => self.calculate_type(vec_def.inner_type_id),
            TypeDef::Cell(cell_def) => self.calculate_type(cell_def.inner_type_id),
            TypeDef::Pointer(pointer_def) => self.calculate_type(pointer_def.inner_type_id),
        };

        // Either `Calculated(true)`, `Calculated(false)`, or `NotCalculated`.
        // Note: Never `Calculating`.
        self.calculation_states[type_id] = state;

        state
    }

    /// Calculate if a struct contains a scope (either has its own scope, or one of fields does).
    ///
    /// Returns:
    /// * `Calculated(true)` if definitely contains a scope.
    /// * `Calculated(false)` if definitely does not contain a scope.
    /// * `NotCalculated` if not possible to determine definitively due to circular dependency.
    fn calculate_struct(&mut self, type_id: TypeId) -> CalculationState {
        let struct_def = self.schema.struct_def(type_id);

        let state = if struct_def.visit.scope.is_some() {
            CalculationState::Calculated(true)
        } else {
            // Check if any field contains a scope
            let mut state = CalculationState::Calculated(false);
            for field_index in struct_def.field_indices() {
                let field_type_id = self.schema.struct_def(type_id).fields[field_index].type_id;
                let field_state = self.calculate_type(field_type_id);
                match field_state {
                    CalculationState::Calculated(true) => {
                        state = CalculationState::Calculated(true);
                        break;
                    }
                    CalculationState::NotCalculated => {
                        state = CalculationState::NotCalculated;
                    }
                    _ => {}
                }
            }
            state
        };

        if state == CalculationState::Calculated(true) {
            self.schema.struct_def_mut(type_id).visit.contains_scope = true;
        }

        state
    }

    /// Calculate if an enum contains a scope (one of its variants contains a scope).
    ///
    /// Returns:
    /// * `Calculated(true)` if definitely contains a scope.
    /// * `Calculated(false)` if definitely does not contain a scope.
    /// * `NotCalculated` if not possible to determine definitively due to circular dependency.
    fn calculate_enum(&mut self, type_id: TypeId) -> CalculationState {
        let enum_def = self.schema.enum_def(type_id);

        let mut state = CalculationState::Calculated(false);
        if !enum_def.is_fieldless() {
            // Check if any variant contains a scope
            for variant_index in enum_def.variant_indices() {
                let variant = &self.schema.enum_def(type_id).variants[variant_index];
                if let Some(variant_type_id) = variant.field_type_id {
                    let variant_state = self.calculate_type(variant_type_id);
                    match variant_state {
                        CalculationState::Calculated(true) => {
                            state = CalculationState::Calculated(true);
                            break;
                        }
                        CalculationState::NotCalculated => {
                            state = CalculationState::NotCalculated;
                        }
                        _ => {}
                    }
                }
            }

            // Check if any inherited enum contains a scope
            if state != CalculationState::Calculated(true) {
                for inherits_index in self.schema.enum_def(type_id).inherits_indices() {
                    let inherits_type_id = self.schema.enum_def(type_id).inherits[inherits_index];
                    let inherits_state = self.calculate_type(inherits_type_id);
                    match inherits_state {
                        CalculationState::Calculated(true) => {
                            state = CalculationState::Calculated(true);
                            break;
                        }
                        CalculationState::NotCalculated => {
                            state = CalculationState::NotCalculated;
                        }
                        _ => {}
                    }
                }
            }
        }

        if state == CalculationState::Calculated(true) {
            self.schema.enum_def_mut(type_id).visit.contains_scope = true;
        }

        state
    }
}

/// Generate `ChildScopeCollector`.
fn generate(schema: &Schema) -> TokenStream {
    // Get `TypeId` for `ScopeId`
    let scope_id_type_id = schema.type_names["ScopeId"];

    let visit_methods = schema
        .types
        .iter()
        .filter_map(|type_def| generate_visit_method_for_type(type_def, scope_id_type_id, schema));

    quote! {
        #![expect(
            unused_variables,
            clippy::semicolon_if_nothing_returned,
            clippy::match_wildcard_for_single_variants,
            clippy::match_same_arms,
            clippy::single_match_else
        )]

        ///@@line_break
        use std::cell::Cell;

        ///@@line_break
        use oxc_ast::ast::*;
        use oxc_ast_visit::Visit;
        use oxc_syntax::scope::{ScopeFlags, ScopeId};

        ///@@line_break
        /// Visitor that locates all child scopes.
        ///
        /// Note: Direct child scopes only, not grandchild scopes.
        /// Does not do full traversal - stops each time it hits a node with a scope.
        pub struct ChildScopeCollector {
            pub(crate) scope_ids: Vec<ScopeId>,
        }

        ///@@line_break
        impl ChildScopeCollector {
            pub(crate) fn new() -> Self {
                Self { scope_ids: vec![] }
            }

            ///@@line_break
            pub(crate) fn add_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
                self.scope_ids.push(scope_id.get().unwrap());
            }
        }

        ///@@line_break
        impl<'a> Visit<'a> for ChildScopeCollector {
            #(#visit_methods)*
        }
    }
}

/// [`VisitorOutputs`] for generating visitor calls for just `Visit` trait.
struct VisitOnly(TokenStream);

impl VisitorOutputs for VisitOnly {
    fn gen_each<F: Fn(bool) -> TokenStream>(f: F) -> Self {
        Self(f(false))
    }

    fn map<F: Fn(TokenStream, bool) -> TokenStream>(self, f: F) -> Self {
        Self(f(self.0, false))
    }
}

/// Generate visitor method for a type.
///
/// Returns `None` if no visitor method is required
/// (either the type is not visited, or the default `visit_*` method can be used).
fn generate_visit_method_for_type(
    type_def: &TypeDef,
    scope_id_type_id: TypeId,
    schema: &Schema,
) -> Option<TokenStream> {
    match type_def {
        TypeDef::Struct(struct_def) => {
            generate_visit_method_for_struct(struct_def, scope_id_type_id, schema)
        }
        TypeDef::Enum(enum_def) => generate_visit_method_for_enum(enum_def, schema),
        _ => None,
    }
}

/// Generate visitor method for a struct.
///
/// Returns `None` if no visitor method is required
/// (either the struct is not visited, or the default `visit_*` method can be used).
fn generate_visit_method_for_struct(
    struct_def: &StructDef,
    scope_id_type_id: TypeId,
    schema: &Schema,
) -> Option<TokenStream> {
    // Get visit method name. Exit if struct is not visited.
    let visit_method_ident = struct_def.visit.visitor_ident()?;

    let (body, stmt_count) = if let Some(scope) = struct_def.visit.scope.as_ref() {
        // Struct has its own scope.
        // Visit fields which contain a scope before entering + after exiting this struct's scope.
        let mut stmts_before = quote!();
        let mut stmts_after = quote!();
        let mut scope_id_field = None;
        let mut stmt_count = 1;

        for (field_index, field) in struct_def.fields.iter().enumerate() {
            // Identify `ScopeId` field
            let field_type = field.type_def(schema);
            if let TypeDef::Cell(cell_def) = field_type
                && let TypeDef::Option(option_def) = cell_def.inner_type(schema)
                && option_def.inner_type_id == scope_id_type_id
            {
                scope_id_field = Some(field);
                continue;
            }

            // Check if field is before enter scope / after exit scope
            let stmts = if field_index < scope.enter_before_index {
                &mut stmts_before
            } else if field_index >= scope.exit_before_index {
                &mut stmts_after
            } else {
                continue;
            };

            if let Some(visit) = generate_visit_stmt_for_struct_field(field, schema) {
                stmts.extend(visit);
                stmt_count += 1;
            }
        }

        let scope_id_field = scope_id_field.unwrap();
        let scope_id_field_ident = scope_id_field.ident();
        let body = quote! {
            #stmts_before
            self.add_scope(&it.#scope_id_field_ident);
            #stmts_after
        };
        (body, stmt_count)
    } else if struct_def.visit.contains_scope {
        // Struct does not have its own scope, but at least one of fields does.
        // Only visit fields which contain a scope.
        let mut body = quote!();
        let mut stmt_count = 0;
        for field in &struct_def.fields {
            if let Some(visit) = generate_visit_stmt_for_struct_field(field, schema) {
                body.extend(visit);
                stmt_count += 1;
            }
        }

        if stmt_count == struct_def.fields.len() {
            // All fields visited. Use default visitor method.
            return None;
        }

        (body, stmt_count)
    } else {
        let body = quote! {
            //!@ Struct does not contain a scope. Halt traversal.
        };
        (body, 0)
    };

    // Generate visit method.
    // `#[inline]` if there are `INLINE_LIMIT` or less statements.
    let maybe_inline_attr = match stmt_count {
        0 => quote!( #[inline(always)] ),
        _ if stmt_count <= INLINE_LIMIT => quote!( #[inline] ),
        _ => quote!(),
    };

    let ty = struct_def.ty(schema);

    let extra_params = struct_def
        .visit
        .visit_args
        .iter()
        .map(|(_, arg_type_name)| {
            let arg_type_ident = create_ident(arg_type_name);
            quote!( , _: #arg_type_ident )
        })
        .collect::<TokenStream>();

    let visit_method = quote! {
        ///@@line_break
        #maybe_inline_attr
        fn #visit_method_ident(&mut self, it: &#ty #extra_params) {
            #body
        }
    };
    Some(visit_method)
}

/// Generate statement to visit a struct field if it contains a scope.
fn generate_visit_stmt_for_struct_field(field: &FieldDef, schema: &Schema) -> Option<TokenStream> {
    let field_type = field.type_def(schema);
    let contains_scope = match field_type.innermost_type(schema) {
        TypeDef::Struct(struct_def) => struct_def.visit.contains_scope,
        TypeDef::Enum(enum_def) => enum_def.visit.contains_scope,
        TypeDef::Primitive(_) => false,
        _ => unreachable!(),
    };
    if !contains_scope {
        return None;
    }

    // Generate visit statement for field
    let field_ident = field.ident();
    generate_visit_type(
        field_type,
        &Target::Property(quote!( it.#field_ident )),
        &field.visit.visit_args,
        &field_ident,
        &quote!(self),
        true,
        schema,
    )
    .map(|VisitOnly(visit)| visit)
}

/// Generate visitor method for an enum.
///
/// Returns `None` if no visitor method is required
/// (either the enum is not visited, or the default `visit_*` method can be used).
fn generate_visit_method_for_enum(enum_def: &EnumDef, schema: &Schema) -> Option<TokenStream> {
    // Get visit method name. Exit if enum is not visited.
    let visit_method_ident = enum_def.visit.visitor_ident()?;

    let (body, maybe_inline_attr) = if enum_def.visit.contains_scope {
        // Some variants contain scopes. Only visit variants which do.
        let enum_ident = enum_def.ident();
        let mut unvisited_variants = quote!();
        let mut match_arm_count = 0;
        let match_arms = enum_def
            .all_variants(schema)
            .filter_map(|variant| {
                let variant_type = variant.field_type(schema)?;
                let contains_scope = match variant_type.innermost_type(schema) {
                    TypeDef::Struct(struct_def) => struct_def.visit.contains_scope,
                    TypeDef::Enum(enum_def) => enum_def.visit.contains_scope,
                    _ => false,
                };

                let visit = if contains_scope {
                    generate_visit_type(
                        variant_type,
                        &Target::Reference(create_ident_tokens("it")),
                        &variant.visit.visit_args,
                        &create_ident_tokens("it"),
                        &quote!(self),
                        false,
                        schema,
                    )
                    .map(|VisitOnly(visit)| visit)
                } else {
                    None
                };

                if let Some(visit) = visit {
                    match_arm_count += 1;
                    let variant_ident = variant.ident();
                    Some(quote! {
                        #enum_ident::#variant_ident(it) => #visit,
                    })
                } else {
                    let doc = format!("@ `{}`", variant.name());
                    unvisited_variants.extend(quote! {
                        #![doc = #doc]
                    });
                    None
                }
            })
            .collect::<TokenStream>();

        // If all variants have scopes, no need for a custom visitor.
        // The default one will visit all variants.
        if unvisited_variants.is_empty() {
            return None;
        }

        let body = quote! {
            match it {
                #match_arms
                _ => {
                    //!@ Remaining variants do not contain scopes:
                    #unvisited_variants
                }
            }
        };
        // `#[inline]` if there are `INLINE_LIMIT` or less match arms
        let maybe_inline_attr =
            if match_arm_count <= INLINE_LIMIT { quote!( #[inline] ) } else { quote!() };
        (body, maybe_inline_attr)
    } else {
        // No variant contains a scope
        let body = quote! {
            //!@ Enum does not contain a scope. Halt traversal.
        };
        let inline_attr = quote!( #[inline(always)] );
        (body, inline_attr)
    };

    // Generate visit method
    let ty = enum_def.ty(schema);
    let visit_method = quote! {
        ///@@line_break
        #maybe_inline_attr
        fn #visit_method_ident(&mut self, it: &#ty) {
            #body
        }
    };
    Some(visit_method)
}
