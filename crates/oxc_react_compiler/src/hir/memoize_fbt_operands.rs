/// Memoize fbt and macro operands in the same scope.
///
/// Port of `ReactiveScopes/MemoizeFbtAndMacroOperandsInSameScope.ts` from the React Compiler.
///
/// Ensures that fbt (Facebook's internationalization library) operands
/// are memoized together with their containing fbt call, preventing
/// unnecessary re-translations.
///
/// FBT provides the `<fbt>` JSX element and `fbt()` calls (which take params in the
/// form of `<fbt:param>` children or `fbt.param()` arguments, respectively). These
/// tags/functions have restrictions on what types of syntax may appear as props/children/
/// arguments, notably that variable references may not appear directly -- variables
/// must always be wrapped in a `<fbt:param>` or `fbt.param()`.
///
/// To ensure the compiler doesn't rewrite code to violate this restriction, we force
/// operands to fbt tags/calls to have the same scope as the tag/call itself.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    HIRFunction, IdentifierId, InstructionId, InstructionValue, JsxTag, MutableRange, ReactiveScope,
};

use super::types::PropertyLiteral;
use super::visitors::each_instruction_value_operand;

/// Whether a macro requires its arguments to be transitively inlined (eg fbt)
/// or just avoid having the top-level values be converted to variables (eg fbt.param).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineLevel {
    Transitive,
    Shallow,
}

/// Definition of a macro tag with its inline level and optional property sub-definitions.
#[derive(Debug, Clone)]
struct MacroDefinition {
    level: InlineLevel,
    /// Map from property name to sub-definition. `"*"` is a wildcard.
    properties: Option<FxHashMap<String, MacroDefinition>>,
}

fn shallow_macro() -> MacroDefinition {
    MacroDefinition { level: InlineLevel::Shallow, properties: None }
}

fn transitive_macro() -> MacroDefinition {
    MacroDefinition { level: InlineLevel::Transitive, properties: None }
}

fn fbt_macro() -> MacroDefinition {
    let mut props = FxHashMap::default();
    props.insert("*".to_string(), shallow_macro());
    // fbt.enum and fbs.enum recurse back to fbt_macro level
    props.insert("enum".to_string(), fbt_macro_no_recurse());
    MacroDefinition { level: InlineLevel::Transitive, properties: Some(props) }
}

/// Same as `fbt_macro` but without nested `enum` recursion (to avoid infinite recursion
/// in the builder). The TS version uses mutation (`FBT_MACRO.properties!.set('enum', FBT_MACRO)`)
/// which creates a cycle. We approximate with one level of nesting.
fn fbt_macro_no_recurse() -> MacroDefinition {
    let mut props = FxHashMap::default();
    props.insert("*".to_string(), shallow_macro());
    MacroDefinition { level: InlineLevel::Transitive, properties: Some(props) }
}

/// Build the FBT_TAGS map.
fn fbt_tags() -> FxHashMap<String, MacroDefinition> {
    let mut tags = FxHashMap::default();
    tags.insert("fbt".to_string(), fbt_macro());
    tags.insert("fbt:param".to_string(), shallow_macro());
    tags.insert("fbt:enum".to_string(), fbt_macro());
    tags.insert("fbt:plural".to_string(), shallow_macro());
    tags.insert("fbs".to_string(), fbt_macro());
    tags.insert("fbs:param".to_string(), shallow_macro());
    tags.insert("fbs:enum".to_string(), fbt_macro());
    tags.insert("fbs:plural".to_string(), shallow_macro());
    tags
}

/// Memoize fbt/macro operands in the same scope as the fbt call.
///
/// Returns the set of identifiers that are fbt/macro operands (needed by OutlineFunctions).
pub fn memoize_fbt_and_macro_operands_in_same_scope(
    func: &mut HIRFunction,
) -> FxHashSet<IdentifierId> {
    // Build the macro kinds map from FBT_TAGS + customMacros
    let mut macro_kinds = fbt_tags();
    if let Some(custom_macros) = &func.env.config.custom_macros {
        for name in custom_macros {
            macro_kinds.insert(name.clone(), transitive_macro());
        }
    }

    // Forward data-flow analysis to identify all macro tags
    let macro_tags = populate_macro_tags(func, &macro_kinds);

    // Reverse data-flow analysis to merge arguments to macro invocations
    merge_macro_arguments(func, macro_tags, &macro_kinds)
}

/// Forward data-flow analysis to identify all macro tags, including
/// things like `fbt.foo.bar(...)`.
fn populate_macro_tags(
    func: &HIRFunction,
    macro_kinds: &FxHashMap<String, MacroDefinition>,
) -> FxHashMap<IdentifierId, MacroDefinition> {
    let mut macro_tags: FxHashMap<IdentifierId, MacroDefinition> = FxHashMap::default();

    let mut block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    block_ids.sort();

    for block_id in block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else {
            continue;
        };

        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::Primitive(v) => {
                    if let crate::hir::PrimitiveValueKind::String(s) = &v.value
                        && let Some(macro_def) = macro_kinds.get(s.as_str())
                    {
                        macro_tags.insert(instr.lvalue.identifier.id, macro_def.clone());
                    }
                }
                InstructionValue::LoadGlobal(v) => {
                    if let Some(macro_def) = macro_kinds.get(v.binding.name()) {
                        macro_tags.insert(instr.lvalue.identifier.id, macro_def.clone());
                    }
                }
                InstructionValue::PropertyLoad(v) => {
                    if let PropertyLiteral::String(prop_name) = &v.property
                        && let Some(obj_def) = macro_tags.get(&v.object.identifier.id)
                    {
                        // Look up the property in the parent macro definition
                        let property_def = obj_def.properties.as_ref().and_then(|props| {
                            props.get(prop_name.as_str()).or_else(|| props.get("*"))
                        });
                        // If found, use the property definition; otherwise propagate
                        // the parent definition
                        let resolved = match property_def {
                            Some(def) => def.clone(),
                            None => obj_def.clone(),
                        };
                        macro_tags.insert(instr.lvalue.identifier.id, resolved);
                    }
                }
                _ => {}
            }
        }
    }

    macro_tags
}

/// Reverse data-flow analysis to merge arguments to macro invocations
/// based on the kind of the macro.
fn merge_macro_arguments(
    func: &mut HIRFunction,
    mut macro_tags: FxHashMap<IdentifierId, MacroDefinition>,
    macro_kinds: &FxHashMap<String, MacroDefinition>,
) -> FxHashSet<IdentifierId> {
    let mut macro_values = FxHashSet::default();
    // Seed with all known macro tag identifiers
    for &id in macro_tags.keys() {
        macro_values.insert(id);
    }

    let mut block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    block_ids.sort();
    block_ids.reverse();

    for block_id in block_ids {
        // Process instructions in reverse order.
        // We need to collect mutation info first, then apply, to satisfy the borrow checker.
        let Some(block) = func.body.blocks.get(&block_id) else {
            continue;
        };

        let num_instructions = block.instructions.len();
        for i in (0..num_instructions).rev() {
            let Some(block) = func.body.blocks.get(&block_id) else {
                break;
            };
            let instr = &block.instructions[i];
            let lvalue_id = instr.lvalue.identifier.id;
            let lvalue_scope = instr.lvalue.identifier.scope.clone();

            match &instr.value {
                // Instructions that never need to be merged
                InstructionValue::DeclareContext(_)
                | InstructionValue::DeclareLocal(_)
                | InstructionValue::Destructure(_)
                | InstructionValue::LoadContext(_)
                | InstructionValue::LoadLocal(_)
                | InstructionValue::PostfixUpdate(_)
                | InstructionValue::PrefixUpdate(_)
                | InstructionValue::StoreContext(_)
                | InstructionValue::StoreLocal(_) => {}
                InstructionValue::CallExpression(v) => {
                    let Some(scope) = &lvalue_scope else {
                        continue;
                    };
                    let callee_id = v.callee.identifier.id;
                    let macro_def =
                        macro_tags.get(&callee_id).or_else(|| macro_tags.get(&lvalue_id)).cloned();
                    if let Some(macro_def) = macro_def {
                        let operand_ids =
                            collect_operand_info(&each_instruction_value_operand(&instr.value));
                        apply_operand_merges(
                            func,
                            block_id,
                            i,
                            &macro_def,
                            scope,
                            &operand_ids,
                            &mut macro_values,
                            &mut macro_tags,
                        );
                    }
                }
                InstructionValue::MethodCall(v) => {
                    let Some(scope) = &lvalue_scope else {
                        continue;
                    };
                    let property_id = v.property.identifier.id;
                    let macro_def = macro_tags
                        .get(&property_id)
                        .or_else(|| macro_tags.get(&lvalue_id))
                        .cloned();
                    if let Some(macro_def) = macro_def {
                        let operand_ids =
                            collect_operand_info(&each_instruction_value_operand(&instr.value));
                        apply_operand_merges(
                            func,
                            block_id,
                            i,
                            &macro_def,
                            scope,
                            &operand_ids,
                            &mut macro_values,
                            &mut macro_tags,
                        );
                    }
                }
                InstructionValue::JsxExpression(v) => {
                    let Some(scope) = &lvalue_scope else {
                        continue;
                    };
                    let macro_def = match &v.tag {
                        JsxTag::Place(p) => macro_tags.get(&p.identifier.id).cloned(),
                        JsxTag::BuiltIn(tag) => macro_kinds.get(&tag.name).cloned(),
                    };
                    let macro_def = macro_def.or_else(|| macro_tags.get(&lvalue_id).cloned());
                    if let Some(macro_def) = macro_def {
                        let operand_ids =
                            collect_operand_info(&each_instruction_value_operand(&instr.value));
                        apply_operand_merges(
                            func,
                            block_id,
                            i,
                            &macro_def,
                            scope,
                            &operand_ids,
                            &mut macro_values,
                            &mut macro_tags,
                        );
                    }
                }
                _ => {
                    // Default case: check if lvalue is a macro tag
                    let Some(scope) = &lvalue_scope else {
                        continue;
                    };
                    let macro_def = macro_tags.get(&lvalue_id).cloned();
                    if let Some(macro_def) = macro_def {
                        let operand_ids =
                            collect_operand_info(&each_instruction_value_operand(&instr.value));
                        apply_operand_merges(
                            func,
                            block_id,
                            i,
                            &macro_def,
                            scope,
                            &operand_ids,
                            &mut macro_values,
                            &mut macro_tags,
                        );
                    }
                }
            }
        }

        // Process phis
        let Some(block) = func.body.blocks.get(&block_id) else {
            continue;
        };

        // Collect phi data that needs mutation
        let phi_merges: Vec<_> = block
            .phis
            .iter()
            .filter_map(|phi| {
                let phi_id = phi.place.identifier.id;
                let macro_def = macro_tags.get(&phi_id)?;
                if macro_def.level == InlineLevel::Shallow {
                    return None;
                }
                let scope = phi.place.identifier.scope.as_ref()?;
                let operand_data: Vec<_> = phi
                    .operands
                    .values()
                    .map(|operand| (operand.identifier.id, operand.identifier.mutable_range))
                    .collect();
                Some((phi_id, scope.as_ref().clone(), macro_def.clone(), operand_data))
            })
            .collect();

        // Apply phi mutations
        for (phi_id, scope, macro_def, operand_data) in phi_merges {
            macro_values.insert(phi_id);

            for (operand_id, operand_mutable_range) in &operand_data {
                macro_tags.insert(*operand_id, macro_def.clone());
                macro_values.insert(*operand_id);

                // We need to update the scope range on the phi's scope
                // and assign the scope to the operand's identifier
                // This requires finding the phi and mutating it
                let Some(block) = func.body.blocks.get_mut(&block_id) else {
                    break;
                };
                for phi in &mut block.phis {
                    if phi.place.identifier.id == phi_id {
                        // Expand the scope range
                        if let Some(ref mut phi_scope) = phi.place.identifier.scope {
                            expand_fbt_scope_range(&mut phi_scope.range, *operand_mutable_range);
                        }
                        // Set operand scopes
                        for operand in phi.operands.values_mut() {
                            if operand.identifier.id == *operand_id {
                                operand.identifier.scope = Some(Box::new(scope.clone()));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    macro_values
}

/// Information about an operand needed for scope merging.
struct OperandInfo {
    id: IdentifierId,
    mutable_range: MutableRange,
}

/// Collect operand identifiers and their mutable ranges from a set of places.
fn collect_operand_info(operands: &[&crate::hir::Place]) -> Vec<OperandInfo> {
    operands
        .iter()
        .map(|place| OperandInfo {
            id: place.identifier.id,
            mutable_range: place.identifier.mutable_range,
        })
        .collect()
}

/// Apply operand merges: mark lvalue as macro value, and for each operand,
/// set its scope and extend the scope range.
///
/// In the TS version, all operands share a single scope reference, so expanding
/// the scope range in one place affects all. In Rust, we first expand the scope
/// range with all operand mutable ranges, then assign the expanded scope to each.
fn apply_operand_merges(
    func: &mut HIRFunction,
    block_id: crate::hir::BlockId,
    instr_idx: usize,
    macro_def: &MacroDefinition,
    scope: &ReactiveScope,
    operand_infos: &[OperandInfo],
    macro_values: &mut FxHashSet<IdentifierId>,
    macro_tags: &mut FxHashMap<IdentifierId, MacroDefinition>,
) {
    let Some(block) = func.body.blocks.get_mut(&block_id) else {
        return;
    };
    let instr = &mut block.instructions[instr_idx];

    // Mark the lvalue as a macro value
    macro_values.insert(instr.lvalue.identifier.id);

    // For each operand, record in macro_tags and macro_values
    for operand_info in operand_infos {
        if macro_def.level == InlineLevel::Transitive {
            macro_tags.insert(operand_info.id, macro_def.clone());
        }
        macro_values.insert(operand_info.id);
    }

    // For transitive macros, expand the scope range with all operand mutable ranges
    // then assign the expanded scope to each operand
    if macro_def.level == InlineLevel::Transitive {
        let mut expanded_scope = scope.clone();
        for operand_info in operand_infos {
            expand_fbt_scope_range(&mut expanded_scope.range, operand_info.mutable_range);
        }
        // Apply the expanded scope to all instruction operands
        crate::hir::visitors::map_instruction_value_operands(
            &mut block.instructions[instr_idx].value,
            &mut |mut place| {
                place.identifier.scope = Some(Box::new(expanded_scope.clone()));
                place
            },
        );
    }
}

/// Expand the fbt scope range to include the given mutable range.
fn expand_fbt_scope_range(fbt_range: &mut MutableRange, extend_with: MutableRange) {
    if extend_with.start != InstructionId::ZERO {
        fbt_range.start = InstructionId(fbt_range.start.0.min(extend_with.start.0));
    }
}
