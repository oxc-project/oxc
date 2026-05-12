use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitMut, walk, walk_mut};
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_semantic::ReferenceId;
use oxc_str::CompactStr;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::TraverseCtx;

use super::PeepholeOptimizations;

type UsedProperties = FxHashMap<SymbolId, FxHashSet<CompactStr>>;

/// Object property usage discovered for fresh object literals.
struct ObjectPropertyUsage {
    used_properties: UsedProperties,
    escaped_or_unknown_symbols: FxHashSet<SymbolId>,
}

struct ObjectPropertyUsageCollector<'ctx, 'a> {
    ctx: &'ctx TraverseCtx<'a>,
    candidate_symbols: FxHashSet<SymbolId>,
    used_properties: UsedProperties,
    escaped_or_unknown_symbols: FxHashSet<SymbolId>,
    member_object_references: FxHashSet<ReferenceId>,
}

impl<'ctx, 'a> ObjectPropertyUsageCollector<'ctx, 'a> {
    fn new(ctx: &'ctx TraverseCtx<'a>) -> Self {
        Self {
            ctx,
            candidate_symbols: FxHashSet::default(),
            used_properties: FxHashMap::default(),
            escaped_or_unknown_symbols: FxHashSet::default(),
            member_object_references: FxHashSet::default(),
        }
    }

    fn finish(mut self) -> ObjectPropertyUsage {
        for &symbol_id in &self.candidate_symbols {
            for &reference_id in self.ctx.scoping().get_resolved_reference_ids(symbol_id) {
                let reference = self.ctx.scoping().get_reference(reference_id);
                if reference.is_write()
                    || (reference.is_read()
                        && !self.member_object_references.contains(&reference_id))
                {
                    self.escaped_or_unknown_symbols.insert(symbol_id);
                    break;
                }
            }
        }

        ObjectPropertyUsage {
            used_properties: self.used_properties,
            escaped_or_unknown_symbols: self.escaped_or_unknown_symbols,
        }
    }

    fn collect_candidate(&mut self, decl: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &decl.id else { return };
        let Some(symbol_id) = binding.symbol_id.get() else { return };
        let Some(Expression::ObjectExpression(object_expr)) = &decl.init else { return };
        if object_expr.properties.iter().any(ObjectPropertyKind::is_spread) {
            return;
        }

        if self.can_prune_symbol(symbol_id) {
            self.candidate_symbols.insert(symbol_id);
        }
    }

    fn can_prune_symbol(&self, symbol_id: SymbolId) -> bool {
        let Some(symbol_value) = self.ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };

        symbol_value.is_fresh_value
            && !symbol_value.exported
            && symbol_value.write_references_count == 0
            && !self.ctx.scoping().scope_flags(symbol_value.scope_id).contains_direct_eval()
    }

    fn record_member_access(
        &mut self,
        object: &Expression<'a>,
        property_name: Option<&str>,
    ) -> Option<SymbolId> {
        let Expression::Identifier(ident) = object.without_parentheses() else { return None };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = self.ctx.scoping().get_reference(reference_id).symbol_id() else {
            return None;
        };
        if !self.candidate_symbols.contains(&symbol_id) {
            return None;
        }

        self.member_object_references.insert(reference_id);

        if let Some(property_name) = property_name {
            self.used_properties
                .entry(symbol_id)
                .or_default()
                .insert(CompactStr::new(property_name));
        } else {
            self.escaped_or_unknown_symbols.insert(symbol_id);
        }

        Some(symbol_id)
    }

    fn mark_member_call_as_unknown(&mut self, callee: &Expression<'a>) {
        match callee.without_parentheses() {
            Expression::StaticMemberExpression(member_expr) => {
                if let Some(symbol_id) = self.record_member_access(
                    &member_expr.object,
                    Some(member_expr.property.name.as_str()),
                ) {
                    self.escaped_or_unknown_symbols.insert(symbol_id);
                }
            }
            Expression::ComputedMemberExpression(member_expr) => {
                let property_name = member_expr.static_property_name();
                if let Some(symbol_id) =
                    self.record_member_access(&member_expr.object, property_name.as_deref())
                {
                    self.escaped_or_unknown_symbols.insert(symbol_id);
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for ObjectPropertyUsageCollector<'_, 'a> {
    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        self.collect_candidate(decl);
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_static_member_expression(&mut self, member_expr: &StaticMemberExpression<'a>) {
        self.record_member_access(&member_expr.object, Some(member_expr.property.name.as_str()));
        walk::walk_static_member_expression(self, member_expr);
    }

    fn visit_computed_member_expression(&mut self, member_expr: &ComputedMemberExpression<'a>) {
        let property_name = member_expr.static_property_name();
        self.record_member_access(&member_expr.object, property_name.as_deref());
        walk::walk_computed_member_expression(self, member_expr);
    }

    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        // A method call receives the object as `this`, so the callee can inspect any property.
        self.mark_member_call_as_unknown(&call_expr.callee);
        walk::walk_call_expression(self, call_expr);
    }

    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        self.mark_member_call_as_unknown(&new_expr.callee);
        walk::walk_new_expression(self, new_expr);
    }
}

struct UnusedObjectPropertyPruner<'ctx, 'a> {
    ctx: &'ctx TraverseCtx<'a>,
    used_properties: &'ctx UsedProperties,
    escaped_or_unknown_symbols: &'ctx FxHashSet<SymbolId>,
    changed: bool,
}

impl<'ctx, 'a> UnusedObjectPropertyPruner<'ctx, 'a> {
    fn can_prune_symbol(&self, symbol_id: SymbolId) -> bool {
        let Some(symbol_value) = self.ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };

        symbol_value.is_fresh_value
            && !symbol_value.exported
            && symbol_value.write_references_count == 0
            && !self.escaped_or_unknown_symbols.contains(&symbol_id)
            && !self.ctx.scoping().scope_flags(symbol_value.scope_id).contains_direct_eval()
    }

    fn prune_declarator(&mut self, decl: &mut VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &decl.id else { return };
        let Some(symbol_id) = binding.symbol_id.get() else { return };
        if !self.can_prune_symbol(symbol_id) {
            return;
        }

        let Some(Expression::ObjectExpression(object_expr)) = &mut decl.init else { return };
        if object_expr.properties.iter().any(ObjectPropertyKind::is_spread) {
            return;
        }

        let Some(used_properties) = self.used_properties.get(&symbol_id) else {
            return;
        };

        let old_len = object_expr.properties.len();
        object_expr.properties.retain(|property| match property {
            ObjectPropertyKind::SpreadProperty(_) => true,
            ObjectPropertyKind::ObjectProperty(property) => {
                self.should_keep_property(property, used_properties)
            }
        });

        if object_expr.properties.len() != old_len {
            self.changed = true;
        }
    }

    fn should_keep_property(
        &self,
        property: &ObjectProperty<'a>,
        used_properties: &FxHashSet<CompactStr>,
    ) -> bool {
        if property.kind != PropertyKind::Init || property.computed || property.shorthand {
            return true;
        }

        let Some(name) = property.key.static_name() else {
            return true;
        };
        if used_properties.contains(name.as_ref()) {
            return true;
        }

        property.value.may_have_side_effects(self.ctx)
    }
}

impl<'a> VisitMut<'a> for UnusedObjectPropertyPruner<'_, 'a> {
    fn visit_variable_declarator(&mut self, decl: &mut VariableDeclarator<'a>) {
        self.prune_declarator(decl);
        walk_mut::walk_variable_declarator(self, decl);
    }
}

impl<'a> PeepholeOptimizations {
    /// Remove unused properties from fresh object literals.
    ///
    /// This is intentionally conservative: an object is only pruned when all
    /// references to its binding are statically-known member accesses, and the
    /// removed property initializer is side-effect free.
    pub fn remove_unused_object_properties(program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut collector = ObjectPropertyUsageCollector::new(ctx);
        collector.visit_program(program);
        let usage = collector.finish();

        if usage.used_properties.is_empty() {
            return;
        }

        let mut pruner = UnusedObjectPropertyPruner {
            ctx,
            used_properties: &usage.used_properties,
            escaped_or_unknown_symbols: &usage.escaped_or_unknown_symbols,
            changed: false,
        };
        pruner.visit_program(program);
        let changed = pruner.changed;

        if changed {
            ctx.state.changed = true;
        }
    }
}
