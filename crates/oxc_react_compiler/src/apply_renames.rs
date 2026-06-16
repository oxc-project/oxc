// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Apply the React Compiler's binding renames to identifiers in uncompiled
//! sibling code (compiled functions already carry the renamed identifiers).

use indexmap::IndexMap;
use oxc_ast::ast::*;
use oxc_ast_visit::VisitMut;
use oxc_ast_visit::walk_mut;
use react_compiler::entrypoint::compile_result::BindingRenameInfo;
use react_compiler_ast::scope::BindingData;
use react_compiler_ast::scope::BindingId;
use rustc_hash::FxHashMap;

/// Map each `span.start` of a renamed-binding reference to its new name.
pub fn build_rename_plan(
    bindings: &[BindingData],
    ref_node_id_to_binding: &IndexMap<u32, BindingId>,
    renames: &[BindingRenameInfo],
) -> FxHashMap<u32, String> {
    if renames.is_empty() {
        return FxHashMap::default();
    }

    let renames_by_declaration: FxHashMap<u32, &BindingRenameInfo> =
        renames.iter().map(|rename| (rename.declaration_start, rename)).collect();

    let mut renamed_bindings: FxHashMap<BindingId, String> = FxHashMap::default();
    for binding in bindings {
        let Some(rename) =
            binding.declaration_start.and_then(|start| renames_by_declaration.get(&start))
        else {
            continue;
        };
        if binding.name == rename.original {
            renamed_bindings.insert(binding.id, rename.renamed.clone());
        }
    }

    if renamed_bindings.is_empty() {
        return FxHashMap::default();
    }

    // `ref_node_id_to_binding` is node-id keyed; in OXC node_id == span.start.
    ref_node_id_to_binding
        .iter()
        .filter_map(|(&position, binding_id)| {
            if bindings[binding_id.0 as usize].declaration_start == Some(position) {
                return None;
            }
            renamed_bindings.get(binding_id).map(|renamed| (position, renamed.clone()))
        })
        .collect()
}

/// Walk the program and apply `rename_plan` (keyed by `span.start`), expanding
/// shorthand properties `{x}` → `{x: x_0}` when the value is renamed.
pub fn apply_renames<'a>(
    program: &mut Program<'a>,
    rename_plan: &FxHashMap<u32, String>,
    allocator: &'a oxc_allocator::Allocator,
) {
    if rename_plan.is_empty() {
        return;
    }
    walk_mut::walk_program(&mut RenameApplyVisitor { rename_plan, allocator }, program);
}

struct RenameApplyVisitor<'a, 'p> {
    rename_plan: &'p FxHashMap<u32, String>,
    allocator: &'a oxc_allocator::Allocator,
}

impl<'a, 'p> RenameApplyVisitor<'a, 'p> {
    fn renamed_at(&self, position: u32) -> Option<&str> {
        self.rename_plan.get(&position).map(|s| s.as_str())
    }
}

impl<'a> VisitMut<'a> for RenameApplyVisitor<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        if let Some(renamed) = self.renamed_at(ident.span.start) {
            ident.name = oxc_allocator::StringBuilder::from_str_in(renamed, self.allocator)
                .into_str()
                .into();
        }
    }

    fn visit_binding_identifier(&mut self, ident: &mut BindingIdentifier<'a>) {
        if let Some(renamed) = self.renamed_at(ident.span.start) {
            ident.name = oxc_allocator::StringBuilder::from_str_in(renamed, self.allocator)
                .into_str()
                .into();
        }
    }

    /// Rename the object of a member expression, never a static property name.
    fn visit_member_expression(&mut self, expr: &mut MemberExpression<'a>) {
        match expr {
            MemberExpression::StaticMemberExpression(static_expr) => {
                self.visit_expression(&mut static_expr.object);
            }
            MemberExpression::ComputedMemberExpression(computed_expr) => {
                self.visit_expression(&mut computed_expr.object);
                self.visit_expression(&mut computed_expr.expression);
            }
            MemberExpression::PrivateFieldExpression(private_expr) => {
                self.visit_expression(&mut private_expr.object);
            }
        }
    }

    /// Expand a renamed shorthand object property `{x}` into `{x: x_0}`.
    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        if prop.shorthand {
            if let Expression::Identifier(ref ident) = prop.value {
                if let Some(renamed) = self.renamed_at(ident.span.start) {
                    prop.shorthand = false;
                    if let Expression::Identifier(ref mut ident) = prop.value {
                        ident.name =
                            oxc_allocator::StringBuilder::from_str_in(renamed, self.allocator)
                                .into_str()
                                .into();
                    }
                    return;
                }
            }
        }
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
        self.visit_expression(&mut prop.value);
    }

    /// Expand a renamed shorthand binding property `{x}` into `{x: x_0}`.
    fn visit_binding_property(&mut self, prop: &mut BindingProperty<'a>) {
        if prop.shorthand {
            if let BindingPattern::BindingIdentifier(ref ident) = prop.value {
                if let Some(renamed) = self.renamed_at(ident.span.start) {
                    prop.shorthand = false;
                    if let BindingPattern::BindingIdentifier(ref mut ident) = prop.value {
                        ident.name =
                            oxc_allocator::StringBuilder::from_str_in(renamed, self.allocator)
                                .into_str()
                                .into();
                    }
                    return;
                }
            }
        }
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
        self.visit_binding_pattern(&mut prop.value);
    }
}
