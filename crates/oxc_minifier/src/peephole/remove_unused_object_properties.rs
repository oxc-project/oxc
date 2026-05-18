use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_str::CompactStr;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{TraverseCtx, state::ObjectPropertyUsageState};

use super::PeepholeOptimizations;

type UsedProperties = FxHashMap<SymbolId, FxHashSet<CompactStr>>;

impl<'a> PeepholeOptimizations {
    pub(super) fn collect_object_property_candidate(
        decl: &VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        let BindingPattern::BindingIdentifier(binding) = &decl.id else { return };
        let Some(symbol_id) = binding.symbol_id.get() else { return };
        let Some(Expression::ObjectExpression(object_expr)) = &decl.init else { return };
        let Some(prunable_property_count) = Self::count_prunable_object_properties(object_expr)
        else {
            return;
        };

        if Self::can_prune_symbol(symbol_id, ctx) {
            ctx.state.object_property_usage.candidate_symbols.insert(symbol_id);
            ctx.state
                .object_property_usage
                .prunable_property_counts
                .insert(symbol_id, prunable_property_count);
        }
    }

    fn can_prune_symbol(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };

        symbol_value.is_fresh_value
            && !symbol_value.exported
            && symbol_value.write_references_count == 0
            && !ctx.scoping().scope_flags(symbol_value.scope_id).contains_direct_eval()
    }

    fn count_prunable_object_properties(object_expr: &ObjectExpression<'a>) -> Option<u32> {
        if object_expr.properties.len() < 3 {
            return None;
        }
        let mut prunable_property_count = 0_u32;
        for property in &object_expr.properties {
            if property.is_spread() {
                return None;
            }
            let ObjectPropertyKind::ObjectProperty(property) = property else { return None };
            if property.kind != PropertyKind::Init {
                return None;
            }
            if property.computed || property.shorthand {
                continue;
            }
            prunable_property_count += 1;
        }
        (prunable_property_count >= 2).then_some(prunable_property_count)
    }

    pub(super) fn record_object_property_member_access(
        object: &Expression<'a>,
        property_name: Option<&str>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<SymbolId> {
        if ctx.state.object_property_usage.candidate_symbols.is_empty() {
            return None;
        }

        let Expression::Identifier(ident) = object.without_parentheses() else { return None };
        let reference_id = ident.reference_id();
        let symbol_id = ctx.scoping().get_reference(reference_id).symbol_id()?;
        let usage = &mut ctx.state.object_property_usage;
        if !usage.candidate_symbols.contains(&symbol_id) {
            return None;
        }

        usage.member_object_references.insert(reference_id);

        if let Some(property_name) = property_name {
            let used_properties = usage.used_properties.entry(symbol_id).or_default();
            used_properties.insert(CompactStr::new(property_name));
            if Some(used_properties.len() as u32)
                == usage.prunable_property_counts.get(&symbol_id).copied()
            {
                usage.candidate_symbols.remove(&symbol_id);
            }
        } else {
            usage.escaped_or_unknown_symbols.insert(symbol_id);
            usage.candidate_symbols.remove(&symbol_id);
        }

        Some(symbol_id)
    }

    pub(super) fn mark_object_property_member_call_as_unknown(
        callee: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.object_property_usage.candidate_symbols.is_empty() {
            return;
        }
        match callee.without_parentheses() {
            Expression::StaticMemberExpression(member_expr) => {
                if let Some(symbol_id) = Self::record_object_property_member_access(
                    &member_expr.object,
                    Some(member_expr.property.name.as_str()),
                    ctx,
                ) {
                    ctx.state.object_property_usage.escaped_or_unknown_symbols.insert(symbol_id);
                    ctx.state.object_property_usage.candidate_symbols.remove(&symbol_id);
                }
            }
            Expression::ComputedMemberExpression(member_expr) => {
                let property_name = member_expr.static_property_name();
                if let Some(symbol_id) = Self::record_object_property_member_access(
                    &member_expr.object,
                    property_name.as_deref(),
                    ctx,
                ) {
                    ctx.state.object_property_usage.escaped_or_unknown_symbols.insert(symbol_id);
                    ctx.state.object_property_usage.candidate_symbols.remove(&symbol_id);
                }
            }
            _ => {}
        }
    }

    fn finalize_object_property_usage(usage: &mut ObjectPropertyUsageState, ctx: &TraverseCtx<'a>) {
        for &symbol_id in usage.used_properties.keys() {
            for &reference_id in ctx.scoping().get_resolved_reference_ids(symbol_id) {
                let reference = ctx.scoping().get_reference(reference_id);
                if reference.is_write()
                    || (reference.is_read()
                        && !usage.member_object_references.contains(&reference_id))
                {
                    usage.escaped_or_unknown_symbols.insert(symbol_id);
                    usage.candidate_symbols.remove(&symbol_id);
                    break;
                }
            }
        }
    }
}

struct UnusedObjectPropertyPruner<'ctx, 'a> {
    ctx: &'ctx TraverseCtx<'a>,
    used_properties: &'ctx UsedProperties,
    escaped_or_unknown_symbols: &'ctx FxHashSet<SymbolId>,
    changed: bool,
}

impl<'a> UnusedObjectPropertyPruner<'_, 'a> {
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
        let Some(prunable_property_count) =
            PeepholeOptimizations::count_prunable_object_properties(object_expr)
        else {
            return;
        };

        let Some(used_properties) = self.used_properties.get(&symbol_id) else {
            return;
        };
        if used_properties.len() as u32 >= prunable_property_count {
            return;
        }

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
        let mut usage = std::mem::take(&mut ctx.state.object_property_usage);

        if usage.candidate_symbols.is_empty() || usage.used_properties.is_empty() {
            return;
        }

        Self::finalize_object_property_usage(&mut usage, ctx);

        if !usage.used_properties.keys().any(|sid| !usage.escaped_or_unknown_symbols.contains(sid))
        {
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
