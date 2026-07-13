// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Validates that useEffect is not used for derived computations which could/should
//! be performed in render.
//!
//! See https://react.dev/learn/you-might-not-need-an-effect#updating-state-based-on-props-or-state
//!
//! Port of ValidateNoDerivedComputationsInEffects_exp.ts.

use crate::react_compiler_utils::{FxIndexSet, IdentIndexMap};
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::IdentBuildHasher;
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_index::IndexSlice;
use oxc_str::{Ident, IdentHashSet};

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::BasicBlock;
use crate::react_compiler_hir::Instruction;
use crate::react_compiler_hir::Terminal;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{
    each_instruction_lvalue_ids, each_instruction_operand as canonical_each_instruction_operand,
};
use crate::react_compiler_hir::{
    ArrayElement, BlockId, Effect, EvaluationOrder, FunctionId, HirFunction, Identifier,
    IdentifierId, IdentifierName, InstructionValue, ParamPattern, PlaceOrSpread, ReactFunctionType,
    ReturnVariant, Type, TypeId, is_set_state_type, is_use_effect_hook_type, is_use_ref_type,
    is_use_state_type,
};
use oxc_span::Span;

const MAX_FIXPOINT_ITERATIONS: usize = 100;

/// `IndexSet` keyed by [`Ident`], reusing its precomputed hash.
type IdentIndexSet<'a> = indexmap::IndexSet<Ident<'a>, IdentBuildHasher>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeOfValue {
    Ignored,
    FromProps,
    FromState,
    FromPropsAndState,
}

#[derive(Debug, Clone)]
struct DerivationMetadata<'a> {
    type_of_value: TypeOfValue,
    place_identifier: IdentifierId,
    place_name: Option<IdentifierName<'a>>,
    source_ids: FxIndexSet<IdentifierId>,
    is_state_source: bool,
}

/// Metadata about a useEffect call site.
struct EffectMetadata {
    effect_func_id: FunctionId,
    dep_elements: Vec<DepElement>,
}

#[derive(Debug, Clone)]
struct DepElement {
    identifier: IdentifierId,
    span: Option<Span>,
}

struct ValidationContext<'a> {
    /// Map from lvalue identifier to the FunctionId of function expressions
    functions: FxHashMap<IdentifierId, FunctionId>,
    /// Map from lvalue identifier to ArrayExpression elements (candidate deps)
    candidate_dependencies: FxHashMap<IdentifierId, Vec<DepElement>>,
    derivation_cache: DerivationCache<'a>,
    effects_cache: FxHashMap<IdentifierId, EffectMetadata>,
    set_state_loads: FxHashMap<IdentifierId, Option<IdentifierId>>,
    set_state_usages: FxHashMap<IdentifierId, FxHashSet<Span>>,
}

#[derive(Debug, Clone)]
struct DerivationCache<'a> {
    has_changes: bool,
    cache: FxHashMap<IdentifierId, DerivationMetadata<'a>>,
    previous_cache: Option<FxHashMap<IdentifierId, DerivationMetadata<'a>>>,
}

impl<'a> DerivationCache<'a> {
    fn new() -> Self {
        DerivationCache { has_changes: false, cache: FxHashMap::default(), previous_cache: None }
    }

    fn take_snapshot(&mut self) {
        let mut prev = FxHashMap::default();
        for (key, value) in &self.cache {
            prev.insert(
                *key,
                DerivationMetadata {
                    place_identifier: value.place_identifier,
                    place_name: value.place_name,
                    source_ids: value.source_ids.clone(),
                    type_of_value: value.type_of_value,
                    is_state_source: value.is_state_source,
                },
            );
        }
        self.previous_cache = Some(prev);
    }

    fn check_for_changes(&mut self) {
        let prev = match &self.previous_cache {
            Some(p) => p,
            None => {
                self.has_changes = true;
                return;
            }
        };

        for (key, value) in &self.cache {
            match prev.get(key) {
                None => {
                    self.has_changes = true;
                    return;
                }
                Some(prev_value) => {
                    if !is_derivation_equal(prev_value, value) {
                        self.has_changes = true;
                        return;
                    }
                }
            }
        }

        if self.cache.len() != prev.len() {
            self.has_changes = true;
            return;
        }

        self.has_changes = false;
    }

    fn snapshot(&mut self) -> bool {
        std::mem::replace(&mut self.has_changes, false)
    }

    fn add_derivation_entry(
        &mut self,
        derived_id: IdentifierId,
        derived_name: Option<IdentifierName<'a>>,
        source_ids: FxIndexSet<IdentifierId>,
        type_of_value: TypeOfValue,
        is_state_source: bool,
    ) {
        let mut final_is_source = is_state_source;
        if !final_is_source {
            for source_id in &source_ids {
                if let Some(source_metadata) = self.cache.get(source_id) {
                    if source_metadata.is_state_source
                        && !matches!(&source_metadata.place_name, Some(IdentifierName::Named(_)))
                    {
                        final_is_source = true;
                        break;
                    }
                }
            }
        }

        self.cache.insert(
            derived_id,
            DerivationMetadata {
                place_identifier: derived_id,
                place_name: derived_name,
                source_ids,
                type_of_value,
                is_state_source: final_is_source,
            },
        );
    }
}

fn is_derivation_equal(a: &DerivationMetadata<'_>, b: &DerivationMetadata<'_>) -> bool {
    if a.type_of_value != b.type_of_value {
        return false;
    }
    if a.source_ids.len() != b.source_ids.len() {
        return false;
    }
    for id in &a.source_ids {
        if !b.source_ids.contains(id) {
            return false;
        }
    }
    true
}

fn join_value(lvalue_type: TypeOfValue, value_type: TypeOfValue) -> TypeOfValue {
    if lvalue_type == TypeOfValue::Ignored {
        return value_type;
    }
    if value_type == TypeOfValue::Ignored {
        return lvalue_type;
    }
    if lvalue_type == value_type {
        return lvalue_type;
    }
    TypeOfValue::FromPropsAndState
}

fn get_root_set_state(
    key: IdentifierId,
    loads: &FxHashMap<IdentifierId, Option<IdentifierId>>,
    visited: &mut FxHashSet<IdentifierId>,
) -> Option<IdentifierId> {
    if visited.contains(&key) {
        return None;
    }
    visited.insert(key);

    match loads.get(&key) {
        None => None,
        Some(None) => Some(key),
        Some(Some(parent_id)) => get_root_set_state(*parent_id, loads, visited),
    }
}

fn maybe_record_set_state_for_instr(
    instr: &Instruction,
    env: &Environment,
    set_state_loads: &mut FxHashMap<IdentifierId, Option<IdentifierId>>,
    set_state_usages: &mut FxHashMap<IdentifierId, FxHashSet<Span>>,
) {
    let identifiers = &env.identifiers;
    let types = &env.types;

    let all_lvalues = each_instruction_lvalue_ids(instr);
    for &lvalue_id in &all_lvalues {
        // Check if this is a LoadLocal from a known setState
        if let InstructionValue::LoadLocal { place, .. } = &instr.value {
            if set_state_loads.contains_key(&place.identifier) {
                set_state_loads.insert(lvalue_id, Some(place.identifier));
            } else {
                // Only check root setState if not a LoadLocal from a known chain
                let lvalue_ident = &identifiers[lvalue_id];
                let lvalue_ty = &types[lvalue_ident.type_];
                if is_set_state_type(lvalue_ty) {
                    set_state_loads.insert(lvalue_id, None);
                }
            }
        } else {
            // Check if lvalue is a setState type (root setState)
            let lvalue_ident = &identifiers[lvalue_id];
            let lvalue_ty = &types[lvalue_ident.type_];
            if is_set_state_type(lvalue_ty) {
                set_state_loads.insert(lvalue_id, None);
            }
        }

        let root = get_root_set_state(lvalue_id, set_state_loads, &mut FxHashSet::default());
        if let Some(root_id) = root {
            set_state_usages.entry(root_id).or_insert_with(|| {
                let mut set = FxHashSet::default();
                set.insert(instr.lvalue.span.unwrap_or_default());
                set
            });
        }
    }
}

fn is_mutable_at(
    env: &Environment,
    eval_order: EvaluationOrder,
    identifier_id: IdentifierId,
) -> bool {
    env.identifiers[identifier_id].mutable_range.contains(eval_order)
}

pub fn validate_no_derived_computations_in_effects_exp(
    func: &HirFunction<'_>,
    env: &Environment<'_>,
) -> Result<Diagnostics, OxcDiagnostic> {
    let identifiers = &env.identifiers;

    let mut context = ValidationContext {
        functions: FxHashMap::default(),
        candidate_dependencies: FxHashMap::default(),
        derivation_cache: DerivationCache::new(),
        effects_cache: FxHashMap::default(),
        set_state_loads: FxHashMap::default(),
        set_state_usages: FxHashMap::default(),
    };

    // Initialize derivation cache based on function type
    if func.fn_type == ReactFunctionType::Hook {
        for param in &func.params {
            if let ParamPattern::Place(place) = param {
                let name = identifiers[place.identifier].name;
                context.derivation_cache.cache.insert(
                    place.identifier,
                    DerivationMetadata {
                        place_identifier: place.identifier,
                        place_name: name,
                        source_ids: FxIndexSet::default(),
                        type_of_value: TypeOfValue::FromProps,
                        is_state_source: true,
                    },
                );
            }
        }
    } else if func.fn_type == ReactFunctionType::Component {
        if let Some(ParamPattern::Place(place)) = func.params.first() {
            let name = identifiers[place.identifier].name;
            context.derivation_cache.cache.insert(
                place.identifier,
                DerivationMetadata {
                    place_identifier: place.identifier,
                    place_name: name,
                    source_ids: FxIndexSet::default(),
                    type_of_value: TypeOfValue::FromProps,
                    is_state_source: true,
                },
            );
        }
    }

    // Fixpoint iteration
    let mut is_first_pass = true;
    let mut iteration_count = 0;
    loop {
        context.derivation_cache.take_snapshot();

        for (_block_id, block) in &func.body.blocks {
            record_phi_derivations(block, &mut context, env);
            for &instr_id in &block.instructions {
                let instr = &func.instructions[instr_id.index()];
                record_instruction_derivations(instr, &mut context, is_first_pass, env)?;
            }
        }

        context.derivation_cache.check_for_changes();
        is_first_pass = false;
        iteration_count += 1;
        assert!(
            iteration_count < MAX_FIXPOINT_ITERATIONS,
            "[ValidateNoDerivedComputationsInEffects] Fixpoint iteration failed to converge."
        );

        if !context.derivation_cache.snapshot() {
            break;
        }
    }

    // Validate all effect sites
    let mut errors = Diagnostics::new();
    let effects_cache: Vec<(IdentifierId, FunctionId, Vec<DepElement>)> = context
        .effects_cache
        .iter()
        .map(|(k, v)| (*k, v.effect_func_id, v.dep_elements.clone()))
        .collect();

    for (_key, effect_func_id, dep_elements) in &effects_cache {
        validate_effect(*effect_func_id, dep_elements, &mut context, env, &mut errors);
    }

    Ok(errors)
}

fn record_phi_derivations<'a>(
    block: &BasicBlock,
    context: &mut ValidationContext<'a>,
    env: &Environment<'a>,
) {
    let identifiers = &env.identifiers;
    for phi in &block.phis {
        let mut type_of_value = TypeOfValue::Ignored;
        let mut source_ids: FxIndexSet<IdentifierId> = FxIndexSet::default();

        for (_block_id, operand) in &phi.operands {
            if let Some(operand_metadata) = context.derivation_cache.cache.get(&operand.identifier)
            {
                type_of_value = join_value(type_of_value, operand_metadata.type_of_value);
                source_ids.insert(operand.identifier);
            }
        }

        if type_of_value != TypeOfValue::Ignored {
            let name = identifiers[phi.place.identifier].name;
            context.derivation_cache.add_derivation_entry(
                phi.place.identifier,
                name,
                source_ids,
                type_of_value,
                false,
            );
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn record_instruction_derivations<'a>(
    instr: &Instruction,
    context: &mut ValidationContext<'a>,
    is_first_pass: bool,
    env: &Environment<'a>,
) -> Result<(), OxcDiagnostic> {
    let identifiers = &env.identifiers;
    let types = &env.types;
    let functions = &env.functions;
    let lvalue_id = instr.lvalue.identifier;

    // maybeRecordSetState
    maybe_record_set_state_for_instr(
        instr,
        env,
        &mut context.set_state_loads,
        &mut context.set_state_usages,
    );

    let mut type_of_value = TypeOfValue::Ignored;
    let is_source = false;
    let mut sources: FxIndexSet<IdentifierId> = FxIndexSet::default();

    match &instr.value {
        InstructionValue::FunctionExpression { lowered_func, .. } => {
            context.functions.insert(lvalue_id, lowered_func.func);
            // Recurse into the inner function
            let inner_func = &functions[lowered_func.func];
            for (_block_id, block) in &inner_func.body.blocks {
                record_phi_derivations(block, context, env);
                for &inner_instr_id in &block.instructions {
                    let inner_instr = &inner_func.instructions[inner_instr_id.index()];
                    record_instruction_derivations(inner_instr, context, is_first_pass, env)?;
                }
            }
        }
        InstructionValue::CallExpression { callee, args, .. } => {
            let callee_type = &types[identifiers[callee.identifier].type_];
            if is_use_effect_hook_type(callee_type) && args.len() == 2 {
                if let (PlaceOrSpread::Place(arg0), PlaceOrSpread::Place(arg1)) =
                    (&args[0], &args[1])
                {
                    let effect_function = context.functions.get(&arg0.identifier).copied();
                    let deps = context.candidate_dependencies.get(&arg1.identifier).cloned();
                    if let (Some(effect_func_id), Some(dep_elements)) = (effect_function, deps) {
                        context.effects_cache.insert(
                            arg0.identifier,
                            EffectMetadata { effect_func_id, dep_elements },
                        );
                    }
                }
            }

            // Check if lvalue is useState type
            let lvalue_type = &types[identifiers[lvalue_id].type_];
            if is_use_state_type(lvalue_type) {
                let name = identifiers[lvalue_id].name;
                context.derivation_cache.add_derivation_entry(
                    lvalue_id,
                    name,
                    FxIndexSet::default(),
                    TypeOfValue::FromState,
                    true,
                );
                return Ok(());
            }
        }
        InstructionValue::MethodCall { property, args, .. } => {
            let prop_type = &types[identifiers[property.identifier].type_];
            if is_use_effect_hook_type(prop_type) && args.len() == 2 {
                if let (PlaceOrSpread::Place(arg0), PlaceOrSpread::Place(arg1)) =
                    (&args[0], &args[1])
                {
                    let effect_function = context.functions.get(&arg0.identifier).copied();
                    let deps = context.candidate_dependencies.get(&arg1.identifier).cloned();
                    if let (Some(effect_func_id), Some(dep_elements)) = (effect_function, deps) {
                        context.effects_cache.insert(
                            arg0.identifier,
                            EffectMetadata { effect_func_id, dep_elements },
                        );
                    }
                }
            }

            // Check if lvalue is useState type
            let lvalue_type = &types[identifiers[lvalue_id].type_];
            if is_use_state_type(lvalue_type) {
                let name = identifiers[lvalue_id].name;
                context.derivation_cache.add_derivation_entry(
                    lvalue_id,
                    name,
                    FxIndexSet::default(),
                    TypeOfValue::FromState,
                    true,
                );
                return Ok(());
            }
        }
        InstructionValue::ArrayExpression { elements, .. } => {
            let dep_elements: Vec<DepElement> = elements
                .iter()
                .filter_map(|el| match el {
                    ArrayElement::Place(p) => {
                        Some(DepElement { identifier: p.identifier, span: p.span })
                    }
                    _ => None,
                })
                .collect();
            context.candidate_dependencies.insert(lvalue_id, dep_elements);
        }
        _ => {}
    }

    // Collect operand derivations
    for (operand_id, operand_span) in each_instruction_operand(instr, env) {
        // Track setState usages
        if context.set_state_loads.contains_key(&operand_id) {
            let root =
                get_root_set_state(operand_id, &context.set_state_loads, &mut FxHashSet::default());
            if let Some(root_id) = root {
                if let Some(usages) = context.set_state_usages.get_mut(&root_id) {
                    usages.insert(operand_span.unwrap_or_default());
                }
            }
        }

        if let Some(operand_metadata) = context.derivation_cache.cache.get(&operand_id) {
            type_of_value = join_value(type_of_value, operand_metadata.type_of_value);
            sources.insert(operand_id);
        }
    }

    if type_of_value == TypeOfValue::Ignored {
        return Ok(());
    }

    // Record derivation for ALL lvalue places (including destructured variables)
    for &lv_id in &each_instruction_lvalue_ids(instr) {
        let name = identifiers[lv_id].name;
        context.derivation_cache.add_derivation_entry(
            lv_id,
            name,
            sources.clone(),
            type_of_value,
            is_source,
        );
    }

    if matches!(&instr.value, InstructionValue::FunctionExpression { .. }) {
        // Don't record mutation effects for FunctionExpressions
        return Ok(());
    }

    // Handle mutable operands
    for operand in each_instruction_operand_with_effect(instr, env) {
        if operand.effect.is_mutable() {
            if is_mutable_at(env, instr.id, operand.id) {
                if let Some(existing) = context.derivation_cache.cache.get_mut(&operand.id) {
                    existing.type_of_value = join_value(type_of_value, existing.type_of_value);
                } else {
                    let name = identifiers[operand.id].name;
                    context.derivation_cache.add_derivation_entry(
                        operand.id,
                        name,
                        sources.clone(),
                        type_of_value,
                        false,
                    );
                }
            }
        } else if matches!(operand.effect, Effect::Unknown) {
            return Err(ErrorCategory::Invariant.diagnostic("Unexpected unknown effect"));
        }
        // Freeze | Read => no-op
    }
    Ok(())
}

struct OperandWithEffect {
    id: IdentifierId,
    effect: Effect,
}

/// Collects operand (IdentifierId, span) pairs from an instruction.
/// Thin wrapper around canonical `each_instruction_operand` that maps Places to (id, span) pairs.
fn each_instruction_operand(
    instr: &Instruction,
    env: &Environment,
) -> Vec<(IdentifierId, Option<Span>)> {
    canonical_each_instruction_operand(instr, env)
        .into_iter()
        .map(|place| (place.identifier, place.span))
        .collect()
}

/// Collects operands with their effects.
/// Thin wrapper around canonical `each_instruction_operand` that maps Places to OperandWithEffect.
fn each_instruction_operand_with_effect(
    instr: &Instruction,
    env: &Environment,
) -> Vec<OperandWithEffect> {
    canonical_each_instruction_operand(instr, env)
        .into_iter()
        .map(|place| OperandWithEffect { id: place.identifier, effect: place.effect })
        .collect()
}

// =============================================================================
// Tree building and rendering (for error messages)
// =============================================================================

struct TreeNode<'a> {
    name: Ident<'a>,
    type_of_value: TypeOfValue,
    is_source: bool,
    children: Vec<TreeNode<'a>>,
}

fn build_tree_node<'a>(
    source_id: IdentifierId,
    context: &ValidationContext<'a>,
    visited: &IdentHashSet<'a>,
) -> Vec<TreeNode<'a>> {
    let source_metadata = match context.derivation_cache.cache.get(&source_id) {
        Some(m) => m,
        None => return Vec::new(),
    };

    if source_metadata.is_state_source {
        if let Some(IdentifierName::Named(name)) = &source_metadata.place_name {
            return vec![TreeNode {
                name: *name,
                type_of_value: source_metadata.type_of_value,
                is_source: true,
                children: Vec::new(),
            }];
        }
    }

    let mut children: Vec<TreeNode> = Vec::new();
    let mut named_siblings: IdentIndexSet = IdentIndexSet::default();

    for child_id in &source_metadata.source_ids {
        assert_ne!(
            *child_id, source_id,
            "Unexpected self-reference: a value should not have itself as a source"
        );

        let mut new_visited = visited.clone();
        if let Some(IdentifierName::Named(name)) = &source_metadata.place_name {
            new_visited.insert(*name);
        }

        let child_nodes = build_tree_node(*child_id, context, &new_visited);
        for child_node in child_nodes {
            if !named_siblings.contains(&child_node.name) {
                named_siblings.insert(child_node.name);
                children.push(child_node);
            }
        }
    }

    if let Some(IdentifierName::Named(name)) = &source_metadata.place_name {
        if !visited.contains(name) {
            return vec![TreeNode {
                name: *name,
                type_of_value: source_metadata.type_of_value,
                is_source: source_metadata.is_state_source,
                children,
            }];
        }
    }

    children
}

fn render_tree<'a>(
    node: &TreeNode<'a>,
    indent: &str,
    is_last: bool,
    props_set: &mut IdentIndexSet<'a>,
    state_set: &mut IdentIndexSet<'a>,
) -> String {
    let prefix = format!(
        "{}{}",
        indent,
        if is_last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " }
    );
    let child_indent = format!("{}{}", indent, if is_last { "    " } else { "\u{2502}   " });

    let mut result = format!("{}{}", prefix, node.name);

    if node.is_source {
        let type_label = match node.type_of_value {
            TypeOfValue::FromProps => {
                props_set.insert(node.name);
                "Prop"
            }
            TypeOfValue::FromState => {
                state_set.insert(node.name);
                "State"
            }
            _ => {
                props_set.insert(node.name);
                state_set.insert(node.name);
                "Prop and State"
            }
        };
        result += &format!(" ({})", type_label);
    }

    if !node.children.is_empty() {
        result += "\n";
        for (index, child) in node.children.iter().enumerate() {
            let is_last_child = index == node.children.len() - 1;
            result += &render_tree(child, &child_indent, is_last_child, props_set, state_set);
            if index < node.children.len() - 1 {
                result += "\n";
            }
        }
    }

    result
}

fn get_fn_local_deps(
    func_id: Option<FunctionId>,
    env: &Environment,
) -> Option<FxHashSet<IdentifierId>> {
    let func_id = func_id?;
    let inner = &env.functions[func_id];
    let mut deps: FxHashSet<IdentifierId> = FxHashSet::default();

    for (_block_id, block) in &inner.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &inner.instructions[instr_id.index()];
            if let InstructionValue::LoadLocal { place, .. } = &instr.value {
                deps.insert(place.identifier);
            }
        }
    }

    Some(deps)
}

fn validate_effect<'a>(
    effect_func_id: FunctionId,
    dependencies: &[DepElement],
    context: &mut ValidationContext<'a>,
    env: &Environment<'a>,
    errors: &mut Diagnostics,
) {
    let identifiers = &env.identifiers;
    let types = &env.types;
    let functions = &env.functions;
    let effect_function = &functions[effect_func_id];
    let mut seen_blocks: FxHashSet<BlockId> = FxHashSet::default();

    struct DerivedSetStateCall {
        callee_span: Option<Span>,
        callee_id: IdentifierId,
        source_ids: FxIndexSet<IdentifierId>,
    }

    let mut effect_derived_set_state_calls: Vec<DerivedSetStateCall> = Vec::new();
    let mut effect_set_state_usages: FxHashMap<IdentifierId, FxHashSet<Span>> =
        FxHashMap::default();

    // Consider setStates in the effect's dependency array as being part of effectSetStateUsages
    for dep in dependencies {
        let root =
            get_root_set_state(dep.identifier, &context.set_state_loads, &mut FxHashSet::default());
        if let Some(root_id) = root {
            let mut set = FxHashSet::default();
            set.insert(dep.span.unwrap_or_default());
            effect_set_state_usages.insert(root_id, set);
        }
    }

    let mut cleanup_function_deps: Option<FxHashSet<IdentifierId>> = None;
    let mut globals: FxHashSet<IdentifierId> = FxHashSet::default();

    for (_block_id, block) in &effect_function.body.blocks {
        // Check for return -> cleanup function
        if let Terminal::Return { value, return_variant: ReturnVariant::Explicit, .. } =
            &block.terminal
        {
            let func_id = context.functions.get(&value.identifier).copied();
            cleanup_function_deps = get_fn_local_deps(func_id, env);
        }

        // Skip if block has a back edge (pred not yet seen)
        let has_back_edge = block.preds.iter().any(|pred| !seen_blocks.contains(pred));
        if has_back_edge {
            return;
        }

        for &instr_id in &block.instructions {
            let instr = &effect_function.instructions[instr_id.index()];

            // Early return if any instruction derives from a ref
            let lvalue_type = &types[identifiers[instr.lvalue.identifier].type_];
            if is_use_ref_type(lvalue_type) {
                return;
            }

            // maybeRecordSetState for effect instructions
            maybe_record_set_state_for_instr(
                instr,
                env,
                &mut context.set_state_loads,
                &mut effect_set_state_usages,
            );

            // Track setState usages for operands
            for (operand_id, operand_span) in each_instruction_operand(instr, env) {
                if context.set_state_loads.contains_key(&operand_id) {
                    let root = get_root_set_state(
                        operand_id,
                        &context.set_state_loads,
                        &mut FxHashSet::default(),
                    );
                    if let Some(root_id) = root {
                        if let Some(usages) = effect_set_state_usages.get_mut(&root_id) {
                            usages.insert(operand_span.unwrap_or_default());
                        }
                    }
                }
            }

            match &instr.value {
                InstructionValue::CallExpression { callee, args, .. } => {
                    let callee_type = &types[identifiers[callee.identifier].type_];
                    if is_set_state_type(callee_type) && args.len() == 1 {
                        if let PlaceOrSpread::Place(arg0) = &args[0] {
                            let callee_metadata =
                                context.derivation_cache.cache.get(&callee.identifier);

                            // If the setState comes from a source other than local state, skip
                            if let Some(cm) = callee_metadata {
                                if cm.type_of_value != TypeOfValue::FromState {
                                    continue;
                                }
                            } else {
                                continue;
                            }

                            let arg_metadata = context.derivation_cache.cache.get(&arg0.identifier);
                            if let Some(am) = arg_metadata {
                                effect_derived_set_state_calls.push(DerivedSetStateCall {
                                    callee_span: callee.span,
                                    callee_id: callee.identifier,
                                    source_ids: am.source_ids.clone(),
                                });
                            }
                        }
                    } else {
                        // Check if callee is from props/propsAndState -> bail
                        let callee_metadata =
                            context.derivation_cache.cache.get(&callee.identifier);
                        if let Some(cm) = callee_metadata {
                            if cm.type_of_value == TypeOfValue::FromProps
                                || cm.type_of_value == TypeOfValue::FromPropsAndState
                            {
                                return;
                            }
                        }

                        if globals.contains(&callee.identifier) {
                            return;
                        }
                    }
                }
                InstructionValue::LoadGlobal { .. } => {
                    globals.insert(instr.lvalue.identifier);
                    for (operand_id, _) in each_instruction_operand(instr, env) {
                        globals.insert(operand_id);
                    }
                }
                _ => {}
            }
        }
        seen_blocks.insert(block.id);
    }

    // Emit errors for derived setState calls
    for derived in &effect_derived_set_state_calls {
        let root_set_state_call = get_root_set_state(
            derived.callee_id,
            &context.set_state_loads,
            &mut FxHashSet::default(),
        );
        if let Some(root_id) = root_set_state_call {
            let effect_usage_count =
                effect_set_state_usages.get(&root_id).map(|s| s.len()).unwrap_or(0);
            let total_usage_count =
                context.set_state_usages.get(&root_id).map(|s| s.len()).unwrap_or(0);
            if effect_set_state_usages.contains_key(&root_id)
                && context.set_state_usages.contains_key(&root_id)
                && effect_usage_count == total_usage_count - 1
            {
                let mut props_set: IdentIndexSet = IdentIndexSet::default();
                let mut state_set: IdentIndexSet = IdentIndexSet::default();

                let mut root_nodes_map: IdentIndexMap<TreeNode> = IdentIndexMap::default();
                for id in &derived.source_ids {
                    let nodes = build_tree_node(*id, context, &IdentHashSet::default());
                    for node in nodes {
                        if !root_nodes_map.contains_key(&node.name) {
                            root_nodes_map.insert(node.name, node);
                        }
                    }
                }
                let root_nodes: Vec<&TreeNode> = root_nodes_map.values().collect();

                let trees: Vec<String> = root_nodes
                    .iter()
                    .enumerate()
                    .map(|(index, node)| {
                        render_tree(
                            node,
                            "",
                            index == root_nodes.len() - 1,
                            &mut props_set,
                            &mut state_set,
                        )
                    })
                    .collect();

                // Check cleanup function dependencies
                let should_skip = if let Some(ref cleanup_deps) = cleanup_function_deps {
                    derived.source_ids.iter().any(|dep| cleanup_deps.contains(dep))
                } else {
                    false
                };
                if should_skip {
                    return;
                }

                let mut root_sources = String::new();
                if !props_set.is_empty() {
                    let props_list: Vec<&str> = props_set.iter().map(|s| s.as_str()).collect();
                    root_sources += &format!("Props: [{}]", props_list.join(", "));
                }
                if !state_set.is_empty() {
                    if !root_sources.is_empty() {
                        root_sources += "\n";
                    }
                    let state_list: Vec<&str> = state_set.iter().map(|s| s.as_str()).collect();
                    root_sources += &format!("State: [{}]", state_list.join(", "));
                }

                let description = format!(
                    "Using an effect triggers an additional render which can hurt performance and user experience, potentially briefly showing stale values to the user\n\n\
                     This setState call is setting a derived value that depends on the following reactive sources:\n\n\
                     {}\n\n\
                     Data Flow Tree:\n\
                     {}\n\n\
                     See: https://react.dev/learn/you-might-not-need-an-effect#updating-state-based-on-props-or-state",
                    root_sources,
                    trees.join("\n"),
                );

                errors.push(
                    ErrorCategory::EffectDerivationsOfState
                        .diagnostic(
                            "You might not need an effect. Derive values in render, not effects.",
                        )
                        .with_help(description)
                        .with_labels(derived.callee_span.map(|s| {
                            s.label("This should be computed during render, not in an effect")
                        })),
                );
            }
        }
    }
}

// =============================================================================
// Non-exp version: ValidateNoDerivedComputationsInEffects
// Port of ValidateNoDerivedComputationsInEffects.ts
// =============================================================================

/// Non-experimental version of the derived-computations-in-effects validation.
/// Records errors directly on the Environment (matching TS `env.recordError()` behavior).
pub fn validate_no_derived_computations_in_effects(
    func: &HirFunction,
    env: &mut Environment,
) -> Result<(), OxcDiagnostic> {
    // Phase 1: Collect effect call sites (func_id + resolved deps).
    // Done with only immutable borrows of env fields.
    let effects_to_validate: Vec<(FunctionId, Vec<IdentifierId>)> = {
        let ids = &env.identifiers;
        let tys = &env.types;
        let mut candidate_deps: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
        let mut functions_map: FxHashMap<IdentifierId, FunctionId> = FxHashMap::default();
        let mut locals_map: FxHashMap<IdentifierId, IdentifierId> = FxHashMap::default();
        let mut result = Vec::new();

        for (_, block) in &func.body.blocks {
            for &iid in &block.instructions {
                let instr = &func.instructions[iid.index()];
                match &instr.value {
                    InstructionValue::LoadLocal { place, .. } => {
                        locals_map.insert(instr.lvalue.identifier, place.identifier);
                    }
                    InstructionValue::ArrayExpression { elements, .. } => {
                        let elem_ids: Vec<IdentifierId> = elements
                            .iter()
                            .filter_map(|e| match e {
                                ArrayElement::Place(p) => Some(p.identifier),
                                _ => None,
                            })
                            .collect();
                        if elem_ids.len() == elements.len() {
                            candidate_deps.insert(instr.lvalue.identifier, elem_ids);
                        }
                    }
                    InstructionValue::FunctionExpression { lowered_func, .. } => {
                        functions_map.insert(instr.lvalue.identifier, lowered_func.func);
                    }
                    InstructionValue::CallExpression { callee, args, .. } => {
                        let callee_ty = &tys[ids[callee.identifier].type_];
                        if is_use_effect_hook_type(callee_ty) && args.len() == 2 {
                            if let (PlaceOrSpread::Place(arg0), PlaceOrSpread::Place(arg1)) =
                                (&args[0], &args[1])
                            {
                                if let (Some(&func_id), Some(dep_elements)) = (
                                    functions_map.get(&arg0.identifier),
                                    candidate_deps.get(&arg1.identifier),
                                ) {
                                    if !dep_elements.is_empty() {
                                        let resolved: Vec<IdentifierId> = dep_elements
                                            .iter()
                                            .map(|d| locals_map.get(d).copied().unwrap_or(*d))
                                            .collect();
                                        result.push((func_id, resolved));
                                    }
                                }
                            }
                        }
                    }
                    InstructionValue::MethodCall { property, args, .. } => {
                        let callee_ty = &tys[ids[property.identifier].type_];
                        if is_use_effect_hook_type(callee_ty) && args.len() == 2 {
                            if let (PlaceOrSpread::Place(arg0), PlaceOrSpread::Place(arg1)) =
                                (&args[0], &args[1])
                            {
                                if let (Some(&func_id), Some(dep_elements)) = (
                                    functions_map.get(&arg0.identifier),
                                    candidate_deps.get(&arg1.identifier),
                                ) {
                                    if !dep_elements.is_empty() {
                                        let resolved: Vec<IdentifierId> = dep_elements
                                            .iter()
                                            .map(|d| locals_map.get(d).copied().unwrap_or(*d))
                                            .collect();
                                        result.push((func_id, resolved));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        result
    };

    // Phase 2: Validate each collected effect and record error details.
    // Matches TS behavior where env.recordError(new CompilerErrorDetail({...})) is used.
    for (func_id, resolved_deps) in effects_to_validate {
        let details = validate_effect_non_exp(
            &env.functions[func_id],
            &resolved_deps,
            &env.identifiers,
            &env.types,
        );
        for detail in details {
            env.record_error(detail)?;
        }
    }
    Ok(())
}

fn validate_effect_non_exp(
    effect_func: &HirFunction,
    effect_deps: &[IdentifierId],
    ids: &IndexSlice<IdentifierId, [Identifier]>,
    tys: &IndexSlice<TypeId, [Type]>,
) -> Vec<OxcDiagnostic> {
    // Check that the effect function only captures effect deps and setState
    for ctx in &effect_func.context {
        let ctx_ty = &tys[ids[ctx.identifier].type_];
        if is_set_state_type(ctx_ty) || effect_deps.contains(&ctx.identifier) {
            continue;
        } else {
            return Vec::new();
        }
    }

    // Check that all effect deps are actually used in the function
    for dep in effect_deps {
        if !effect_func.context.iter().any(|c| c.identifier == *dep) {
            return Vec::new();
        }
    }

    let mut seen_blocks: FxHashSet<BlockId> = FxHashSet::default();
    let mut dep_values: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
    for dep in effect_deps {
        dep_values.insert(*dep, vec![*dep]);
    }

    let mut set_state_spans: Vec<Span> = Vec::new();

    for (_, block) in &effect_func.body.blocks {
        for &pred in &block.preds {
            if !seen_blocks.contains(&pred) {
                return Vec::new();
            }
        }

        for phi in &block.phis {
            let mut aggregate: FxHashSet<IdentifierId> = FxHashSet::default();
            for operand in phi.operands.values() {
                if let Some(deps) = dep_values.get(&operand.identifier) {
                    for d in deps {
                        aggregate.insert(*d);
                    }
                }
            }
            if !aggregate.is_empty() {
                dep_values.insert(phi.place.identifier, aggregate.into_iter().collect());
            }
        }

        for &iid in &block.instructions {
            let instr = &effect_func.instructions[iid.index()];
            match &instr.value {
                InstructionValue::Primitive { .. }
                | InstructionValue::JSXText { .. }
                | InstructionValue::LoadGlobal { .. } => {}
                InstructionValue::LoadLocal { place, .. } => {
                    if let Some(deps) = dep_values.get(&place.identifier) {
                        dep_values.insert(instr.lvalue.identifier, deps.clone());
                    }
                }
                InstructionValue::ComputedLoad { .. }
                | InstructionValue::PropertyLoad { .. }
                | InstructionValue::BinaryExpression { .. }
                | InstructionValue::TemplateLiteral { .. }
                | InstructionValue::CallExpression { .. }
                | InstructionValue::MethodCall { .. } => {
                    let mut aggregate: FxHashSet<IdentifierId> = FxHashSet::default();
                    for operand in non_exp_value_operands(&instr.value) {
                        if let Some(deps) = dep_values.get(&operand) {
                            for d in deps {
                                aggregate.insert(*d);
                            }
                        }
                    }
                    if !aggregate.is_empty() {
                        dep_values.insert(instr.lvalue.identifier, aggregate.into_iter().collect());
                    }

                    if let InstructionValue::CallExpression { callee, args, .. } = &instr.value {
                        let callee_ty = &tys[ids[callee.identifier].type_];
                        if is_set_state_type(callee_ty) && args.len() == 1 {
                            if let PlaceOrSpread::Place(arg) = &args[0] {
                                if let Some(deps) = dep_values.get(&arg.identifier) {
                                    let dep_set: FxHashSet<_> = deps.iter().collect();
                                    if dep_set.len() == effect_deps.len() {
                                        if let Some(span) = callee.span {
                                            set_state_spans.push(span);
                                        }
                                    } else {
                                        return Vec::new();
                                    }
                                } else {
                                    return Vec::new();
                                }
                            }
                        }
                    }
                }
                _ => {
                    return Vec::new();
                }
            }
        }

        match &block.terminal {
            Terminal::Return { value, .. } | Terminal::Throw { value, .. } => {
                if dep_values.contains_key(&value.identifier) {
                    return Vec::new();
                }
            }
            Terminal::If { test, .. } | Terminal::Branch { test, .. } => {
                if dep_values.contains_key(&test.identifier) {
                    return Vec::new();
                }
            }
            Terminal::Switch { test, .. } if dep_values.contains_key(&test.identifier) => {
                return Vec::new();
            }
            _ => {}
        }

        seen_blocks.insert(block.id);
    }

    set_state_spans
        .into_iter()
        .map(|span| {
            ErrorCategory::EffectDerivationsOfState
                .diagnostic(
                    "Values derived from props and state should be calculated during render, not in an effect. (https://react.dev/learn/you-might-not-need-an-effect#updating-state-based-on-props-or-state)",
                )
                .with_label(span)
        })
        .collect()
}

/// Collects operand IdentifierIds for a subset of instruction variants used
/// by `validate_effect_non_exp`.
///
/// NOTE: This intentionally does NOT use the canonical `each_instruction_value_operand`
/// because: (1) `validate_effect_non_exp` only matches specific variants
/// (ComputedLoad, PropertyLoad, BinaryExpression, TemplateLiteral, CallExpression,
/// MethodCall), so FunctionExpression/ObjectMethod context handling is unnecessary;
/// and (2) the caller does not have access to `env` which the canonical function requires
/// for resolving function expression context captures.
fn non_exp_value_operands(value: &InstructionValue) -> Vec<IdentifierId> {
    match value {
        InstructionValue::ComputedLoad { object, property, .. } => {
            vec![object.identifier, property.identifier]
        }
        InstructionValue::PropertyLoad { object, .. } => vec![object.identifier],
        InstructionValue::BinaryExpression { left, right, .. } => {
            vec![left.identifier, right.identifier]
        }
        InstructionValue::TemplateLiteral { subexprs, .. } => {
            subexprs.iter().map(|s| s.identifier).collect()
        }
        InstructionValue::CallExpression { callee, args, .. } => {
            let mut op_ids = vec![callee.identifier];
            for a in args {
                match a {
                    PlaceOrSpread::Place(p) => op_ids.push(p.identifier),
                    PlaceOrSpread::Spread(s) => op_ids.push(s.place.identifier),
                }
            }
            op_ids
        }
        InstructionValue::MethodCall { receiver, property, args, .. } => {
            let mut op_ids = vec![receiver.identifier, property.identifier];
            for a in args {
                match a {
                    PlaceOrSpread::Place(p) => op_ids.push(p.identifier),
                    PlaceOrSpread::Spread(s) => op_ids.push(s.place.identifier),
                }
            }
            op_ids
        }
        _ => Vec::new(),
    }
}
