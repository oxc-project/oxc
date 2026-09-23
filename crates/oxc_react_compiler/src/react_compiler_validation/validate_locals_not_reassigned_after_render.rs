/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::VecDeque;

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexSlice;
use smallvec::smallvec;

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::type_config::AliasingEffectConfig;
use crate::react_compiler_hir::visitors::{
    PlaceList, each_instruction_lvalue_ids, each_instruction_value_operand, each_terminal_operand,
};
use crate::react_compiler_hir::{
    ArrayElement, BlockId, Effect, FunctionId, HirFunction, Identifier, IdentifierId,
    IdentifierName, InstructionKind, InstructionValue, ObjectPropertyKey, ObjectPropertyOrSpread,
    ParamPattern, Place, PlaceOrSpread, PrimitiveValue, Terminal,
};

type ContextValues = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type CapturedContexts = FxHashMap<IdentifierId, FxHashSet<IdentifierId>>;
type HoistedFunctionValues = FxHashMap<IdentifierId, IdentifierId>;
type CorrelatedValues = FxHashMap<IdentifierId, Vec<(Option<BlockId>, IdentifierId)>>;

struct ReassignmentSources<'a> {
    context_values_by_block: &'a FxHashMap<BlockId, ContextValues>,
    reassigning_functions: &'a FxHashMap<IdentifierId, Place>,
    captured_contexts: &'a CapturedContexts,
    correlated_values: &'a CorrelatedValues,
}

#[derive(Clone, Default)]
struct ReassignmentResult {
    reassignment: Option<Place>,
    returned: Option<Place>,
    returned_contexts: FxHashSet<IdentifierId>,
    returned_context_loads: FxHashSet<IdentifierId>,
}

/// Validates that local variables cannot be reassigned after render.
/// This prevents a category of bugs in which a closure captures a
/// binding from one render but does not update.
pub fn validate_locals_not_reassigned_after_render(func: &HirFunction, env: &mut Environment) {
    let mut context_variables: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut diagnostics: Vec<OxcDiagnostic> = Vec::new();

    for param in &func.params {
        let place = match param {
            ParamPattern::Place(place) => place,
            ParamPattern::Spread(spread) => &spread.place,
        };
        context_variables.insert(place.identifier);
    }
    collect_outer_context_variables(func, &mut context_variables);

    let reassignment = get_context_reassignment(
        func,
        &env.identifiers,
        &env.functions,
        env,
        &mut context_variables,
        false,
        false,
        &mut diagnostics,
    )
    .reassignment;

    // Record accumulated errors (from async function checks in inner functions) first
    for diagnostic in diagnostics {
        env.record_diagnostic(diagnostic);
    }

    // Then record the top-level reassignment error if any
    if let Some(reassignment_place) = reassignment {
        let variable_name = format_variable_name(&reassignment_place, &env.identifiers);
        let declaration_span = env.identifiers[reassignment_place.identifier].span;
        env.record_diagnostic(diagnostics::reassigned_after_render(
            &variable_name,
            reassignment_place.span,
            declaration_span,
        ));
    }
}

/// Parameter patterns are lowered to temporary parameters followed by context stores.
/// Collect those stores before visiting initializer closures so later named parameters
/// are visible to closures created by earlier parameter initializers.
fn collect_outer_context_variables(
    func: &HirFunction,
    context_variables: &mut FxHashSet<IdentifierId>,
) {
    for (_, block) in &func.body.blocks {
        for &instruction_id in &block.instructions {
            match &func.instructions[instruction_id.index()].value {
                InstructionValue::DeclareContext { lvalue, .. }
                | InstructionValue::StoreContext { lvalue, .. } => {
                    context_variables.insert(lvalue.place.identifier);
                }
                InstructionValue::PostfixUpdateContext { lvalue, value, .. }
                | InstructionValue::PrefixUpdateContext { lvalue, value, .. } => {
                    context_variables.insert(lvalue.identifier);
                    context_variables.insert(value.identifier);
                }
                _ => {}
            }
        }
    }
}

/// Format a variable name for error messages. Uses the named identifier if
/// available, otherwise falls back to "variable".
fn format_variable_name(
    place: &Place,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
) -> String {
    let identifier = &identifiers[place.identifier];
    match &identifier.name {
        Some(IdentifierName::Named(name)) => format!("`{}`", name),
        _ => "variable".to_string(),
    }
}

fn record_async_reassignment(
    reassignment_place: Place,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    diagnostics: &mut Vec<OxcDiagnostic>,
) {
    let variable_name = format_variable_name(&reassignment_place, identifiers);
    diagnostics.push(diagnostics::reassigned_in_async_function(
        &variable_name,
        reassignment_place.span,
        identifiers[reassignment_place.identifier].span,
    ));
}

/// Operands through which a reassigning function can propagate. Calls with a
/// no-alias signature only propagate through their callee or receiver.
fn each_reassigning_operand(value: &InstructionValue<'_>, env: &Environment<'_>) -> PlaceList {
    match value {
        InstructionValue::CallExpression { callee, .. }
            if env.has_no_alias_signature(callee.identifier) =>
        {
            smallvec![*callee]
        }
        InstructionValue::MethodCall { receiver, property, .. }
            if env.has_no_alias_signature(property.identifier) =>
        {
            smallvec![*receiver, *property]
        }
        InstructionValue::TaggedTemplateExpression { tag, .. }
            if env.has_no_alias_signature(tag.identifier) =>
        {
            smallvec![*tag]
        }
        _ => each_instruction_value_operand(value, env),
    }
}

/// Callback operands excluded by a no-alias signature. Mutations performed by
/// these callbacks are synchronous, but a value returned from one can still
/// contain an escaping closure.
fn each_no_alias_callback_operand(
    value: &InstructionValue<'_>,
    env: &Environment<'_>,
    require_result_flow: bool,
) -> PlaceList {
    let (signature_identifier, args) = match value {
        InstructionValue::CallExpression { callee, args, .. } => {
            (callee.identifier, args.as_slice())
        }
        InstructionValue::MethodCall { property, args, .. } => {
            (property.identifier, args.as_slice())
        }
        _ => return PlaceList::new(),
    };

    let ty = &env.types[env.identifiers[signature_identifier].type_];
    let Some(signature) = env.get_function_signature(ty).ok().flatten() else {
        return PlaceList::new();
    };
    if !signature.no_alias {
        return PlaceList::new();
    }
    let Some(aliasing) = signature.aliasing else {
        if require_result_flow {
            return PlaceList::new();
        }
        // Legacy no-alias signatures can invoke callbacks without retaining
        // their results. Read-only arguments cannot invoke a mutable callback.
        let only_first_callback = matches!(
            signature.canonical_name.as_deref(),
            Some(
                "Array.filter"
                    | "Array.every"
                    | "Array.some"
                    | "Array.find"
                    | "Array.findIndex"
                    | "Set.forEach"
                    | "Map.forEach"
            )
        );
        return args
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| {
                // These built-ins invoke only the first argument; thisArg can
                // itself be a function without ever being called.
                if only_first_callback && index != 0 {
                    return None;
                }
                let effect =
                    signature.positional_params.get(index).copied().or(signature.rest_param);
                if !matches!(
                    effect,
                    Some(Effect::ConditionallyMutate | Effect::Capture | Effect::Store)
                ) {
                    return None;
                }
                Some(match argument {
                    PlaceOrSpread::Place(place) => *place,
                    PlaceOrSpread::Spread(spread) => spread.place,
                })
            })
            .collect();
    };

    let mut operands = PlaceList::new();
    for effect in aliasing.effects {
        let AliasingEffectConfig::Apply { function, into, .. } = effect else { continue };
        if require_result_flow && !aliasing_value_flows_to_return(&aliasing, into) {
            continue;
        }
        for (index, parameter) in aliasing.params.iter().enumerate() {
            if parameter != function {
                continue;
            }
            if let Some(argument) = args.get(index) {
                operands.push(match argument {
                    PlaceOrSpread::Place(place) => *place,
                    PlaceOrSpread::Spread(spread) => spread.place,
                });
            }
        }
    }
    operands
}

fn aliasing_value_flows_to_return(
    aliasing: &crate::react_compiler_hir::type_config::AliasingSignatureConfig,
    source: &'static str,
) -> bool {
    let mut values = FxHashSet::default();
    let mut pending = VecDeque::from([source]);
    while let Some(value) = pending.pop_front() {
        if value == aliasing.returns {
            return true;
        }
        if !values.insert(value) {
            continue;
        }
        for effect in aliasing.effects {
            let edge = match effect {
                AliasingEffectConfig::CreateFrom { from, into }
                | AliasingEffectConfig::Assign { from, into }
                | AliasingEffectConfig::Alias { from, into }
                | AliasingEffectConfig::Capture { from, into }
                | AliasingEffectConfig::ImmutableCapture { from, into } => Some((*from, *into)),
                _ => None,
            };
            if let Some((from, into)) = edge
                && from == value
            {
                pending.push_back(into);
            }
        }
    }
    false
}

/// Return the function operand for instructions that invoke a callable value.
fn invoked_function_operand(value: &InstructionValue<'_>) -> Option<Place> {
    match value {
        InstructionValue::CallExpression { callee, .. } => Some(*callee),
        InstructionValue::MethodCall { property, .. } => Some(*property),
        InstructionValue::TaggedTemplateExpression { tag, .. } => Some(*tag),
        _ => None,
    }
}

fn each_invoked_async_operand(value: &InstructionValue<'_>, env: &Environment<'_>) -> PlaceList {
    let mut operands = each_no_alias_callback_operand(value, env, false);
    if let Some(function) = invoked_function_operand(value) {
        operands.push(function);
    }
    operands
}

fn add_invoked_async_contexts(
    value: &InstructionValue<'_>,
    env: &Environment<'_>,
    async_function_values: &FxHashSet<IdentifierId>,
    captured_contexts: &CapturedContexts,
    context_values: &ContextValues,
    active_contexts: &mut ContextValues,
) {
    for operand in each_invoked_async_operand(value, env) {
        if async_function_values.contains(&operand.identifier)
            && let Some(contexts) = captured_contexts.get(&operand.identifier)
        {
            for &context in contexts {
                if let Some(values) = context_values.get(&context) {
                    active_contexts.entry(context).or_default().extend(values);
                } else {
                    insert_context_value(active_contexts, context, context);
                }
            }
        }
    }
}

fn insert_context_value(
    context_values: &mut ContextValues,
    context_identifier: IdentifierId,
    value: IdentifierId,
) {
    context_values.entry(context_identifier).or_default().insert(value);
}

fn replace_context_value(
    context_values: &mut ContextValues,
    context_identifier: IdentifierId,
    value: IdentifierId,
) {
    let mut values = FxHashSet::default();
    values.insert(value);
    context_values.insert(context_identifier, values);
}

fn clear_context_value(context_values: &mut ContextValues, context_identifier: IdentifierId) {
    context_values.insert(context_identifier, FxHashSet::default());
}

fn replace_correlated_value(
    correlated_values: &mut CorrelatedValues,
    context_identifier: IdentifierId,
    value: IdentifierId,
) {
    correlated_values.insert(context_identifier, vec![(None, value)]);
}

fn clear_correlated_value(
    correlated_values: &mut CorrelatedValues,
    context_identifier: IdentifierId,
) {
    correlated_values.insert(context_identifier, Vec::new());
}

fn merge_context_values(target: &mut ContextValues, source: &ContextValues) {
    for (&context_identifier, values) in source {
        target.entry(context_identifier).or_default().extend(values);
    }
}

fn connect_context_value(
    context_values: &ContextValues,
    context_identifier: IdentifierId,
    target: IdentifierId,
    propagation_edges: &mut FxHashMap<IdentifierId, Vec<IdentifierId>>,
) {
    if let Some(values) = context_values.get(&context_identifier) {
        for &value in values {
            propagation_edges.entry(value).or_default().push(target);
        }
    } else {
        propagation_edges.entry(context_identifier).or_default().push(target);
    }
}

fn find_reassignment_for_operand(
    identifier: IdentifierId,
    context_values: &ContextValues,
    context_values_by_block: &FxHashMap<BlockId, ContextValues>,
    reassigning_functions: &FxHashMap<IdentifierId, Place>,
    captured_contexts: &CapturedContexts,
    correlated_values: &CorrelatedValues,
) -> Option<Place> {
    fn visit_value(
        identifier: IdentifierId,
        context_values: &ContextValues,
        context_block: Option<BlockId>,
        sources: &ReassignmentSources<'_>,
        visited: &mut FxHashSet<(IdentifierId, Option<BlockId>)>,
    ) -> Option<Place> {
        if !visited.insert((identifier, context_block)) {
            return None;
        }
        if let Some(place) = sources.reassigning_functions.get(&identifier) {
            return Some(*place);
        }
        if let Some(correlated_sources) = sources.correlated_values.get(&identifier) {
            return correlated_sources.iter().find_map(|&(predecessor, source)| {
                let predecessor_values = predecessor
                    .and_then(|block_id| sources.context_values_by_block.get(&block_id))
                    .unwrap_or(context_values);
                visit_value(source, predecessor_values, predecessor, sources, visited)
            });
        }
        sources.captured_contexts.get(&identifier).and_then(|contexts| {
            contexts.iter().find_map(|&context_identifier| {
                if let Some(values) = context_values.get(&context_identifier) {
                    values.iter().find_map(|&value| {
                        visit_value(value, context_values, context_block, sources, visited)
                    })
                } else {
                    visit_value(context_identifier, context_values, context_block, sources, visited)
                }
            })
        })
    }

    let mut visited = FxHashSet::default();
    let sources = ReassignmentSources {
        context_values_by_block,
        reassigning_functions,
        captured_contexts,
        correlated_values,
    };
    if let Some(values) = context_values.get(&identifier) {
        values
            .iter()
            .find_map(|&value| visit_value(value, context_values, None, &sources, &mut visited))
    } else {
        visit_value(identifier, context_values, None, &sources, &mut visited)
    }
}

fn propagate_captured_contexts(
    captured_contexts: &mut CapturedContexts,
    propagation_edges: &FxHashMap<IdentifierId, Vec<IdentifierId>>,
) {
    let mut pending: VecDeque<IdentifierId> = captured_contexts.keys().copied().collect();
    while let Some(identifier) = pending.pop_front() {
        let Some(contexts) = captured_contexts.get(&identifier).cloned() else { continue };
        let Some(targets) = propagation_edges.get(&identifier) else { continue };
        for &target in targets {
            let target_contexts = captured_contexts.entry(target).or_default();
            let previous_len = target_contexts.len();
            target_contexts.extend(contexts.iter().copied());
            if target_contexts.len() != previous_len {
                pending.push_back(target);
            }
        }
    }
}

fn propagate_correlated_values(
    correlated_values: &mut CorrelatedValues,
    propagation_edges: &FxHashMap<IdentifierId, Vec<IdentifierId>>,
) {
    let mut pending: VecDeque<IdentifierId> = correlated_values.keys().copied().collect();
    while let Some(identifier) = pending.pop_front() {
        let Some(sources) = correlated_values.get(&identifier).cloned() else { continue };
        let Some(targets) = propagation_edges.get(&identifier) else { continue };
        for &target in targets {
            let target_sources = correlated_values.entry(target).or_default();
            let previous_len = target_sources.len();
            for source in &sources {
                if !target_sources.contains(source) {
                    target_sources.push(*source);
                }
            }
            if target_sources.len() != previous_len {
                pending.push_back(target);
            }
        }
    }
}

fn propagate_reassignments(
    seeds: &[(IdentifierId, Place)],
    propagation_edges: &FxHashMap<IdentifierId, Vec<IdentifierId>>,
) -> FxHashMap<IdentifierId, Place> {
    let mut reassignments = FxHashMap::default();
    let mut pending = VecDeque::new();
    for &(identifier, reassignment_place) in seeds {
        if let std::collections::hash_map::Entry::Vacant(entry) = reassignments.entry(identifier) {
            entry.insert(reassignment_place);
            pending.push_back(identifier);
        }
    }
    while let Some(identifier) = pending.pop_front() {
        let reassignment_place = reassignments[&identifier];
        let Some(targets) = propagation_edges.get(&identifier) else { continue };
        for &target in targets {
            if let std::collections::hash_map::Entry::Vacant(entry) = reassignments.entry(target) {
                entry.insert(reassignment_place);
                pending.push_back(target);
            }
        }
    }
    reassignments
}

/// Track callable identity through aliases and selected properties. Generic taint
/// edges also represent containment, so an async object member must not make the
/// object itself (or another member) an async function.
fn infer_async_callable_contexts(
    func: &HirFunction,
    alias_edges: &FxHashMap<IdentifierId, Vec<IdentifierId>>,
    async_functions: &FxHashSet<IdentifierId>,
    captured_contexts: &CapturedContexts,
) -> CapturedContexts {
    if async_functions.is_empty() {
        return CapturedContexts::default();
    }
    type Property = (Option<String>, IdentifierId);
    let mut sources = CapturedContexts::default();
    let mut properties: FxHashMap<IdentifierId, Vec<Property>> = FxHashMap::default();
    let mut constants = FxHashMap::default();
    for instruction in &func.instructions {
        if let InstructionValue::Primitive { value, .. } = &instruction.value {
            let key = match value {
                PrimitiveValue::String(value) => Some(value.to_string()),
                PrimitiveValue::Number(value) => Some(value.to_string()),
                _ => None,
            };
            if let Some(key) = key {
                constants.insert(instruction.lvalue.identifier, key);
            }
        }
    }
    let mut loads = Vec::new();
    let mut stores = Vec::new();
    let mut spreads = Vec::new();
    for instruction in &func.instructions {
        let id = instruction.lvalue.identifier;
        if async_functions.contains(&id) {
            sources.entry(id).or_default().insert(id);
        }
        match &instruction.value {
            InstructionValue::ObjectExpression { properties: entries, .. } => {
                sources.entry(id).or_default().insert(id);
                for entry in entries {
                    match entry {
                        ObjectPropertyOrSpread::Property(property) => {
                            let key = match &property.key {
                                ObjectPropertyKey::Identifier { name, .. }
                                | ObjectPropertyKey::String { name, .. } => Some(name.to_string()),
                                ObjectPropertyKey::Computed { name, .. } => {
                                    constants.get(&name.identifier).cloned()
                                }
                            };
                            properties
                                .entry(id)
                                .or_default()
                                .push((key, property.place.identifier));
                        }
                        ObjectPropertyOrSpread::Spread(spread) => {
                            spreads.push((spread.place.identifier, id, false))
                        }
                    }
                }
            }
            InstructionValue::ArrayExpression { elements, .. } => {
                sources.entry(id).or_default().insert(id);
                let mut fixed_index = true;
                for (index, element) in elements.iter().enumerate() {
                    match element {
                        ArrayElement::Place(place) => properties
                            .entry(id)
                            .or_default()
                            .push((fixed_index.then(|| index.to_string()), place.identifier)),
                        ArrayElement::Spread(spread) => {
                            spreads.push((spread.place.identifier, id, true));
                            fixed_index = false;
                        }
                        ArrayElement::Hole => {}
                    }
                }
            }
            InstructionValue::PropertyLoad { object, property, .. } => {
                loads.push((object.identifier, Some(property.to_string()), id))
            }
            InstructionValue::ComputedLoad { object, property, .. } => {
                loads.push((object.identifier, constants.get(&property.identifier).cloned(), id))
            }
            InstructionValue::PropertyStore { object, property, value, .. } => {
                stores.push((object.identifier, Some(property.to_string()), value.identifier))
            }
            InstructionValue::ComputedStore { object, property, value, .. } => stores.push((
                object.identifier,
                constants.get(&property.identifier).cloned(),
                value.identifier,
            )),
            _ => {}
        }
    }
    loop {
        let previous_count: usize = sources.values().map(FxHashSet::len).sum::<usize>()
            + properties.values().map(Vec::len).sum::<usize>();
        propagate_captured_contexts(&mut sources, alias_edges);
        for (object, key, value) in &stores {
            if let Some(objects) = sources.get(object) {
                for &object in objects {
                    let entries = properties.entry(object).or_default();
                    let entry = (key.clone(), *value);
                    if !entries.contains(&entry) {
                        entries.push(entry);
                    }
                }
            }
        }
        for &(from, into, array) in &spreads {
            let entries: Vec<_> = sources
                .get(&from)
                .into_iter()
                .flatten()
                .filter_map(|source| properties.get(source))
                .flatten()
                .map(|(key, value)| (if array { None } else { key.clone() }, *value))
                .collect();
            let into = properties.entry(into).or_default();
            for entry in entries {
                if !into.contains(&entry) {
                    into.push(entry);
                }
            }
        }
        for (object, key, into) in &loads {
            let values: Vec<_> = sources
                .get(object)
                .into_iter()
                .flatten()
                .filter_map(|source| properties.get(source))
                .flatten()
                .filter(|(property, _)| key.is_none() || property.is_none() || key == property)
                .flat_map(|(_, value)| sources.get(value).into_iter().flatten().copied())
                .collect();
            sources.entry(*into).or_default().extend(values);
        }
        let count: usize = sources.values().map(FxHashSet::len).sum::<usize>()
            + properties.values().map(Vec::len).sum::<usize>();
        if count == previous_count {
            break;
        }
    }
    let mut result = CapturedContexts::default();
    for (identifier, values) in sources {
        for value in values {
            if async_functions.contains(&value) {
                result
                    .entry(identifier)
                    .or_default()
                    .extend(captured_contexts.get(&value).into_iter().flatten().copied());
            }
        }
    }
    result
}

fn context_values_at_block_entry(
    func: &HirFunction,
    block_id: BlockId,
    initial: &ContextValues,
    values_by_block: &FxHashMap<BlockId, ContextValues>,
) -> ContextValues {
    let block = &func.body.blocks[&block_id];
    let mut values = ContextValues::default();
    if block_id == func.body.entry {
        merge_context_values(&mut values, initial);
    }
    for predecessor in &block.preds {
        if let Some(predecessor_values) = values_by_block.get(predecessor) {
            merge_context_values(&mut values, predecessor_values);
        }
    }
    values
}

fn correlated_context_values_at_block_entry(
    func: &HirFunction,
    block_id: BlockId,
    initial: &ContextValues,
    values_by_block: &FxHashMap<BlockId, ContextValues>,
) -> CorrelatedValues {
    let block = &func.body.blocks[&block_id];
    let mut sources = CorrelatedValues::default();
    if block_id == func.body.entry {
        for (&context_identifier, values) in initial {
            sources
                .entry(context_identifier)
                .or_default()
                .extend(values.iter().map(|&value| (None, value)));
        }
    }
    for predecessor in &block.preds {
        if let Some(predecessor_values) = values_by_block.get(predecessor) {
            for (&context_identifier, values) in predecessor_values {
                let context_sources = sources.entry(context_identifier).or_default();
                for &value in values {
                    let source = (Some(*predecessor), value);
                    if !context_sources.contains(&source) {
                        context_sources.push(source);
                    }
                }
            }
        }
    }
    sources
}

fn is_hoisted_function_store(
    value: &InstructionValue<'_>,
    hoisted_function_values: &HoistedFunctionValues,
) -> bool {
    matches!(value, InstructionValue::StoreContext { lvalue, .. }
        if lvalue.kind == InstructionKind::Function
            && hoisted_function_values.contains_key(&lvalue.place.identifier))
}

fn update_context_values(
    value: &InstructionValue<'_>,
    values: &mut ContextValues,
    hoisted_function_values: &HoistedFunctionValues,
    only_tracked: bool,
) {
    if is_hoisted_function_store(value, hoisted_function_values) {
        return;
    }
    let (identifier, replacement) = match value {
        InstructionValue::DeclareContext { lvalue, .. } => {
            let replacement = if lvalue.kind == InstructionKind::HoistedFunction {
                hoisted_function_values
                    .get(&lvalue.place.identifier)
                    .copied()
                    .unwrap_or(lvalue.place.identifier)
            } else {
                lvalue.place.identifier
            };
            (lvalue.place.identifier, Some(replacement))
        }
        InstructionValue::StoreContext { lvalue, value, .. } => {
            (lvalue.place.identifier, Some(value.identifier))
        }
        InstructionValue::PostfixUpdateContext { lvalue, .. }
        | InstructionValue::PrefixUpdateContext { lvalue, .. } => (lvalue.identifier, None),
        _ => return,
    };
    if only_tracked && !values.contains_key(&identifier) {
        return;
    }
    if let Some(replacement) = replacement {
        replace_context_value(values, identifier, replacement);
    } else {
        clear_context_value(values, identifier);
    }
}

/// Keep reaching values only on paths where an async invocation is pending.
/// Merging an invocation from one branch with a store from another would invent
/// a continuation that cannot occur at runtime.
fn infer_active_async_contexts_by_block(
    func: &HirFunction,
    env: &Environment<'_>,
    async_function_values: &FxHashSet<IdentifierId>,
    captured_contexts: &CapturedContexts,
    initial: &ContextValues,
    values_by_block: &FxHashMap<BlockId, ContextValues>,
    hoisted_function_values: &HoistedFunctionValues,
) -> FxHashMap<BlockId, ContextValues> {
    let mut contexts_by_block = FxHashMap::default();
    let empty = ContextValues::default();
    loop {
        let mut changed = false;
        for (&block_id, block) in &func.body.blocks {
            let mut contexts =
                context_values_at_block_entry(func, block_id, &empty, &contexts_by_block);
            let mut values =
                context_values_at_block_entry(func, block_id, initial, values_by_block);
            for &instruction_id in &block.instructions {
                let value = &func.instructions[instruction_id.index()].value;
                add_invoked_async_contexts(
                    value,
                    env,
                    async_function_values,
                    captured_contexts,
                    &values,
                    &mut contexts,
                );
                update_context_values(value, &mut values, hoisted_function_values, false);
                update_context_values(value, &mut contexts, hoisted_function_values, true);
            }
            if contexts_by_block.get(&block_id) != Some(&contexts) {
                contexts_by_block.insert(block_id, contexts);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    contexts_by_block
}

/// Compute the possible current value of each context binding at every block exit.
/// Context bindings are not SSA-renamed, so preserving these reaching values keeps a
/// later store from tainting an earlier load while still accounting for loop backedges.
fn infer_context_values_by_block(
    func: &HirFunction,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
) -> (ContextValues, FxHashMap<BlockId, ContextValues>, HoistedFunctionValues) {
    let mut initial = ContextValues::default();
    for context_place in &func.context {
        insert_context_value(&mut initial, context_place.identifier, context_place.identifier);
    }
    for (_, block) in &func.body.blocks {
        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];
            match &instr.value {
                InstructionValue::DeclareContext { lvalue, .. }
                | InstructionValue::StoreContext { lvalue, .. } => {
                    insert_context_value(
                        &mut initial,
                        lvalue.place.identifier,
                        lvalue.place.identifier,
                    );
                }
                InstructionValue::LoadContext { place, .. } => {
                    insert_context_value(&mut initial, place.identifier, place.identifier);
                }
                InstructionValue::PostfixUpdateContext { lvalue, value, .. }
                | InstructionValue::PrefixUpdateContext { lvalue, value, .. } => {
                    insert_context_value(&mut initial, lvalue.identifier, lvalue.identifier);
                    insert_context_value(&mut initial, value.identifier, value.identifier);
                }
                InstructionValue::FunctionExpression { lowered_func, .. }
                | InstructionValue::ObjectMethod { lowered_func, .. } => {
                    for context_place in &functions[lowered_func.func].context {
                        insert_context_value(
                            &mut initial,
                            context_place.identifier,
                            context_place.identifier,
                        );
                    }
                }
                _ => {}
            }
        }
    }
    // Function declarations are initialized before the body executes, even though
    // their lexical StoreContext instruction appears later in HIR source order.
    // The last hoisted declaration wins, matching function instantiation.
    let mut hoisted_function_contexts = FxHashSet::default();
    for (_, block) in &func.body.blocks {
        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];
            if let InstructionValue::DeclareContext { lvalue, .. } = &instr.value
                && lvalue.kind == InstructionKind::HoistedFunction
            {
                hoisted_function_contexts.insert(lvalue.place.identifier);
            }
        }
    }
    let mut hoisted_function_values = HoistedFunctionValues::default();
    for (_, block) in &func.body.blocks {
        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];
            if let InstructionValue::StoreContext { lvalue, value, .. } = &instr.value
                && lvalue.kind == InstructionKind::Function
                && hoisted_function_contexts.contains(&lvalue.place.identifier)
            {
                hoisted_function_values.insert(lvalue.place.identifier, value.identifier);
            }
        }
    }

    let mut values_by_block: FxHashMap<BlockId, ContextValues> = FxHashMap::default();
    loop {
        let mut changed = false;
        for (&block_id, block) in &func.body.blocks {
            let mut values =
                context_values_at_block_entry(func, block_id, &initial, &values_by_block);
            for &instruction_id in &block.instructions {
                let instr = &func.instructions[instruction_id.index()];
                update_context_values(&instr.value, &mut values, &hoisted_function_values, false);
            }
            if values_by_block.get(&block_id) != Some(&values) {
                values_by_block.insert(block_id, values);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    (initial, values_by_block, hoisted_function_values)
}

/// Recursively checks whether a function (or its dependencies) reassigns a
/// context variable.
///
/// Side effects: accumulates async-function reassignment diagnostics into `diagnostics`.
#[allow(clippy::too_many_arguments)]
fn get_context_reassignment(
    func: &HirFunction,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    env: &Environment,
    context_variables: &mut FxHashSet<IdentifierId>,
    is_function_expression: bool,
    is_async: bool,
    diagnostics: &mut Vec<OxcDiagnostic>,
) -> ReassignmentResult {
    // Reassignment taint propagates through aliases, instruction results, and
    // phis. Build the complete graph before validating uses so loop backedges
    // and other cycles reach a fixpoint independent of block order.
    let mut propagation_edges: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
    let mut callable_edges: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
    let mut correlated_values = CorrelatedValues::default();
    let mut seeds: Vec<(IdentifierId, Place)> = Vec::new();
    let mut returning_seeds: Vec<(IdentifierId, Place)> = Vec::new();
    let mut no_alias_calls: Vec<(Vec<IdentifierId>, Vec<IdentifierId>, ContextValues)> = Vec::new();
    let mut async_function_values = FxHashSet::default();
    // Recursive results are computed once. They are needed again when an async
    // function is validated after the propagation graph reaches a fixpoint.
    let mut nested_reassignments: FxHashMap<IdentifierId, (ReassignmentResult, bool)> =
        FxHashMap::default();
    // Function values retain the context bindings they close over. Resolve those
    // bindings at each escape point so a definite overwrite can kill stale taint.
    let mut captured_contexts = CapturedContexts::default();
    let mut returned_captured_contexts = CapturedContexts::default();
    let mut returned_loaded_contexts = CapturedContexts::default();
    let (initial_context_values, context_values_by_block, hoisted_function_values) =
        infer_context_values_by_block(func, functions);
    // Bindings owned by this function are ordinary locals while it executes,
    // but become context variables for child closures.
    let mut child_context_variables = context_variables.clone();
    for param in &func.params {
        let place = match param {
            ParamPattern::Place(place) => place,
            ParamPattern::Spread(spread) => &spread.place,
        };
        child_context_variables.insert(place.identifier);
    }
    collect_outer_context_variables(func, &mut child_context_variables);

    for (&block_id, block) in &func.body.blocks {
        let mut context_values = context_values_at_block_entry(
            func,
            block_id,
            &initial_context_values,
            &context_values_by_block,
        );
        let mut correlated_context_values = correlated_context_values_at_block_entry(
            func,
            block_id,
            &initial_context_values,
            &context_values_by_block,
        );
        for phi in &block.phis {
            correlated_values.insert(
                phi.place.identifier,
                phi.operands
                    .iter()
                    .map(|(predecessor, operand)| (Some(*predecessor), operand.identifier))
                    .collect(),
            );
            for operand in phi.operands.values() {
                propagation_edges.entry(operand.identifier).or_default().push(phi.place.identifier);
                callable_edges.entry(operand.identifier).or_default().push(phi.place.identifier);
            }
        }

        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];
            if is_hoisted_function_store(&instr.value, &hoisted_function_values) {
                continue;
            }

            match &instr.value {
                InstructionValue::FunctionExpression { lowered_func, .. }
                | InstructionValue::ObjectMethod { lowered_func, .. } => {
                    let inner_function = &functions[lowered_func.func];
                    let inner_is_async = is_async || inner_function.is_async;

                    // Recursively check the inner function
                    let reassignment = get_context_reassignment(
                        inner_function,
                        identifiers,
                        functions,
                        env,
                        &mut child_context_variables,
                        true,
                        inner_is_async,
                        diagnostics,
                    );
                    nested_reassignments
                        .insert(instr.lvalue.identifier, (reassignment.clone(), inner_is_async));

                    // Async reassignments produce their own diagnostic and do
                    // not taint the function value, matching the TS behavior.
                    if inner_is_async {
                        async_function_values.insert(instr.lvalue.identifier);
                    } else if let Some(reassignment_place) = reassignment.reassignment {
                        seeds.push((instr.lvalue.identifier, reassignment_place));
                    }
                    if !inner_is_async && let Some(reassignment_place) = reassignment.returned {
                        returning_seeds.push((instr.lvalue.identifier, reassignment_place));
                    }
                    if !inner_is_async && !reassignment.returned_contexts.is_empty() {
                        returned_captured_contexts.insert(
                            instr.lvalue.identifier,
                            reassignment.returned_contexts.clone(),
                        );
                    }
                    if !inner_is_async && !reassignment.returned_context_loads.is_empty() {
                        returned_loaded_contexts.insert(
                            instr.lvalue.identifier,
                            reassignment.returned_context_loads.clone(),
                        );
                    }
                    // Captures are live bindings even for async functions. Keep
                    // their dependencies so a later context store is observed
                    // when the function value eventually escapes.
                    for context_place in &inner_function.context {
                        captured_contexts
                            .entry(instr.lvalue.identifier)
                            .or_default()
                            .insert(context_place.identifier);
                    }
                }

                InstructionValue::StoreLocal { lvalue, value, .. } => {
                    callable_edges
                        .entry(value.identifier)
                        .or_default()
                        .extend([lvalue.place.identifier, instr.lvalue.identifier]);
                    propagation_edges
                        .entry(value.identifier)
                        .or_default()
                        .extend([lvalue.place.identifier, instr.lvalue.identifier]);
                }

                InstructionValue::PostfixUpdateLocal { .. }
                | InstructionValue::PrefixUpdateLocal { .. } => {}

                InstructionValue::LoadLocal { place, .. } => {
                    callable_edges
                        .entry(place.identifier)
                        .or_default()
                        .push(instr.lvalue.identifier);
                    propagation_edges
                        .entry(place.identifier)
                        .or_default()
                        .push(instr.lvalue.identifier);
                }

                InstructionValue::LoadContext { place, .. } => {
                    correlated_values.insert(
                        instr.lvalue.identifier,
                        correlated_context_values
                            .get(&place.identifier)
                            .cloned()
                            .unwrap_or_else(|| vec![(None, place.identifier)]),
                    );
                    connect_context_value(
                        &context_values,
                        place.identifier,
                        instr.lvalue.identifier,
                        &mut propagation_edges,
                    );
                    connect_context_value(
                        &context_values,
                        place.identifier,
                        instr.lvalue.identifier,
                        &mut callable_edges,
                    );
                }

                InstructionValue::DeclareContext { lvalue, .. } => {
                    if !is_function_expression {
                        context_variables.insert(lvalue.place.identifier);
                    }
                    let value = if lvalue.kind == InstructionKind::HoistedFunction {
                        hoisted_function_values
                            .get(&lvalue.place.identifier)
                            .copied()
                            .unwrap_or(lvalue.place.identifier)
                    } else {
                        lvalue.place.identifier
                    };
                    replace_context_value(&mut context_values, lvalue.place.identifier, value);
                    replace_correlated_value(
                        &mut correlated_context_values,
                        lvalue.place.identifier,
                        value,
                    );
                }

                InstructionValue::StoreContext { lvalue, value, .. } => {
                    callable_edges
                        .entry(value.identifier)
                        .or_default()
                        .push(instr.lvalue.identifier);
                    if !is_function_expression {
                        context_variables.insert(lvalue.place.identifier);
                    }
                    propagation_edges
                        .entry(value.identifier)
                        .or_default()
                        .push(instr.lvalue.identifier);
                    replace_context_value(
                        &mut context_values,
                        lvalue.place.identifier,
                        value.identifier,
                    );
                    replace_correlated_value(
                        &mut correlated_context_values,
                        lvalue.place.identifier,
                        value.identifier,
                    );
                }

                InstructionValue::PostfixUpdateContext { lvalue, value, .. }
                | InstructionValue::PrefixUpdateContext { lvalue, value, .. } => {
                    if !is_function_expression {
                        context_variables.insert(lvalue.identifier);
                        context_variables.insert(value.identifier);
                    }
                    clear_context_value(&mut context_values, lvalue.identifier);
                    clear_correlated_value(&mut correlated_context_values, lvalue.identifier);
                }

                _ => {
                    for operand in each_reassigning_operand(&instr.value, env) {
                        // Invariant: effects must be inferred before this pass runs
                        assert!(
                            operand.effect != Effect::Unknown,
                            "Expected effects to be inferred prior to \
                             ValidateLocalsNotReassignedAfterRender"
                        );
                        if operand.effect != Effect::Freeze {
                            propagation_edges
                                .entry(operand.identifier)
                                .or_default()
                                .extend(each_instruction_lvalue_ids(instr));
                        }
                    }
                }
            }

            let callback_operands = each_no_alias_callback_operand(&instr.value, env, true);
            if !callback_operands.is_empty() {
                no_alias_calls.push((
                    callback_operands.iter().map(|place| place.identifier).collect(),
                    each_instruction_lvalue_ids(instr),
                    context_values.clone(),
                ));
            }
        }
    }

    let async_captured_contexts = infer_async_callable_contexts(
        func,
        &callable_edges,
        &async_function_values,
        &captured_contexts,
    );
    async_function_values = async_captured_contexts.keys().copied().collect();
    propagate_captured_contexts(&mut captured_contexts, &propagation_edges);
    propagate_captured_contexts(&mut returned_captured_contexts, &propagation_edges);
    propagate_captured_contexts(&mut returned_loaded_contexts, &propagation_edges);
    for (callback_identifiers, result_identifiers, call_context_values) in &no_alias_calls {
        let mut contexts = FxHashSet::default();
        for callback_identifier in callback_identifiers {
            // A direct return of a context load snapshots its current value.
            // Only a returned closure keeps observing later binding stores.
            if let Some(loaded_contexts) = returned_loaded_contexts.get(callback_identifier) {
                for &context in loaded_contexts {
                    for &result in result_identifiers {
                        connect_context_value(
                            call_context_values,
                            context,
                            result,
                            &mut propagation_edges,
                        );
                    }
                }
            }
            if let Some(callback_contexts) = returned_captured_contexts.get(callback_identifier) {
                contexts.extend(callback_contexts.iter().copied());
            }
        }
        for &result_identifier in result_identifiers {
            captured_contexts
                .entry(result_identifier)
                .or_default()
                .extend(contexts.iter().copied());
        }
    }
    propagate_captured_contexts(&mut captured_contexts, &propagation_edges);
    propagate_correlated_values(&mut correlated_values, &propagation_edges);

    let active_async_contexts_by_block = infer_active_async_contexts_by_block(
        func,
        env,
        &async_function_values,
        &async_captured_contexts,
        &initial_context_values,
        &context_values_by_block,
        &hoisted_function_values,
    );

    let returning_functions = propagate_reassignments(&returning_seeds, &propagation_edges);
    let mut reassigning_functions = propagate_reassignments(&seeds, &propagation_edges);
    loop {
        let mut added_seed = false;
        for (callback_identifiers, result_identifiers, call_context_values) in &no_alias_calls {
            let reassignment_place = callback_identifiers.iter().find_map(|id| {
                returning_functions.get(id).copied().or_else(|| {
                    returned_captured_contexts.get(id).and_then(|contexts| {
                        contexts.iter().find_map(|context| {
                            find_reassignment_for_operand(
                                *context,
                                call_context_values,
                                &context_values_by_block,
                                &reassigning_functions,
                                &captured_contexts,
                                &correlated_values,
                            )
                        })
                    })
                })
            });
            if let Some(reassignment_place) = reassignment_place {
                for &result_identifier in result_identifiers {
                    if !reassigning_functions.contains_key(&result_identifier) {
                        seeds.push((result_identifier, reassignment_place));
                        added_seed = true;
                    }
                }
            }
        }
        if !added_seed {
            break;
        }
        reassigning_functions = propagate_reassignments(&seeds, &propagation_edges);
    }

    let mut returned_value_loads = CapturedContexts::default();
    for context_place in &func.context {
        returned_value_loads
            .entry(context_place.identifier)
            .or_default()
            .insert(context_place.identifier);
    }
    propagate_captured_contexts(&mut returned_value_loads, &propagation_edges);

    // With the complete fixed-point map, validate uses in source CFG order so
    // diagnostics retain their existing ordering.
    let mut result = ReassignmentResult::default();
    for (&block_id, block) in &func.body.blocks {
        let mut context_values = context_values_at_block_entry(
            func,
            block_id,
            &initial_context_values,
            &context_values_by_block,
        );
        let mut active_async_contexts = context_values_at_block_entry(
            func,
            block_id,
            &ContextValues::default(),
            &active_async_contexts_by_block,
        );
        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];
            if is_hoisted_function_store(&instr.value, &hoisted_function_values) {
                continue;
            }

            match &instr.value {
                InstructionValue::FunctionExpression { .. }
                | InstructionValue::ObjectMethod { .. } => {
                    let (reassignment, inner_is_async) = nested_reassignments
                        .get(&instr.lvalue.identifier)
                        .cloned()
                        .expect("nested function was analyzed during graph construction");
                    if inner_is_async && let Some(reassignment_place) = reassignment.reassignment {
                        record_async_reassignment(reassignment_place, identifiers, diagnostics);
                        // Direct async reassignments are diagnosed immediately and
                        // do not propagate through the function value.
                        return ReassignmentResult::default();
                    }
                }

                InstructionValue::StoreContext { lvalue, .. } => {
                    if is_function_expression
                        && context_variables.contains(&lvalue.place.identifier)
                    {
                        result.reassignment.get_or_insert(lvalue.place);
                    }
                }

                InstructionValue::PostfixUpdateContext { lvalue, value, .. }
                | InstructionValue::PrefixUpdateContext { lvalue, value, .. } => {
                    if is_function_expression
                        && (context_variables.contains(&lvalue.identifier)
                            || context_variables.contains(&value.identifier))
                    {
                        result.reassignment.get_or_insert(*lvalue);
                    }
                }

                InstructionValue::StoreLocal { .. }
                | InstructionValue::LoadLocal { .. }
                | InstructionValue::DeclareContext { .. } => {}

                _ => {
                    for operand in each_reassigning_operand(&instr.value, env) {
                        assert!(
                            operand.effect != Effect::Unknown,
                            "Expected effects to be inferred prior to \
                             ValidateLocalsNotReassignedAfterRender"
                        );
                        if operand.effect == Effect::Freeze
                            && let Some(reassignment_place) = find_reassignment_for_operand(
                                operand.identifier,
                                &context_values,
                                &context_values_by_block,
                                &reassigning_functions,
                                &captured_contexts,
                                &correlated_values,
                            )
                        {
                            if async_function_values.contains(&operand.identifier) {
                                record_async_reassignment(
                                    reassignment_place,
                                    identifiers,
                                    diagnostics,
                                );
                                return ReassignmentResult::default();
                            }
                            result.reassignment.get_or_insert(reassignment_place);
                        }
                    }
                }
            }

            // An async continuation observes the live binding when execution
            // yields, not every intermediate store in the current synchronous job.
            if matches!(instr.value, InstructionValue::Await { .. }) {
                for &context in active_async_contexts.keys() {
                    if let Some(reassignment_place) = find_reassignment_for_operand(
                        context,
                        &active_async_contexts,
                        &context_values_by_block,
                        &reassigning_functions,
                        &captured_contexts,
                        &correlated_values,
                    ) {
                        record_async_reassignment(reassignment_place, identifiers, diagnostics);
                        return ReassignmentResult::default();
                    }
                }
            }

            for operand in each_invoked_async_operand(&instr.value, env) {
                let Some(contexts) = async_captured_contexts.get(&operand.identifier) else {
                    continue;
                };
                for &context in contexts {
                    if let Some(reassignment_place) = find_reassignment_for_operand(
                        context,
                        &context_values,
                        &context_values_by_block,
                        &reassigning_functions,
                        &captured_contexts,
                        &correlated_values,
                    ) {
                        record_async_reassignment(reassignment_place, identifiers, diagnostics);
                        return ReassignmentResult::default();
                    }
                }
            }

            add_invoked_async_contexts(
                &instr.value,
                env,
                &async_function_values,
                &async_captured_contexts,
                &context_values,
                &mut active_async_contexts,
            );
            update_context_values(
                &instr.value,
                &mut context_values,
                &hoisted_function_values,
                false,
            );
            update_context_values(
                &instr.value,
                &mut active_async_contexts,
                &hoisted_function_values,
                true,
            );
        }

        // Leaving the function also allows pending async continuations to resume.
        if matches!(block.terminal, Terminal::Return { .. } | Terminal::Throw { .. }) {
            for &context in active_async_contexts.keys() {
                if let Some(reassignment_place) = find_reassignment_for_operand(
                    context,
                    &active_async_contexts,
                    &context_values_by_block,
                    &reassigning_functions,
                    &captured_contexts,
                    &correlated_values,
                ) {
                    record_async_reassignment(reassignment_place, identifiers, diagnostics);
                    return ReassignmentResult::default();
                }
            }
        }

        // Check terminal operands for reassigning functions.
        let is_return = matches!(block.terminal, Terminal::Return { .. });
        for operand in each_terminal_operand(&block.terminal) {
            if is_return && let Some(contexts) = captured_contexts.get(&operand.identifier) {
                result.returned_contexts.extend(contexts.iter().copied());
            }
            if is_return && let Some(contexts) = returned_value_loads.get(&operand.identifier) {
                result.returned_context_loads.extend(contexts.iter().copied());
            }
            if let Some(reassignment_place) = find_reassignment_for_operand(
                operand.identifier,
                &context_values,
                &context_values_by_block,
                &reassigning_functions,
                &captured_contexts,
                &correlated_values,
            ) {
                if async_function_values.contains(&operand.identifier) {
                    record_async_reassignment(reassignment_place, identifiers, diagnostics);
                    return ReassignmentResult::default();
                }
                result.reassignment.get_or_insert(reassignment_place);
                if is_return {
                    result.returned.get_or_insert(reassignment_place);
                }
            }
        }
    }

    result
}
