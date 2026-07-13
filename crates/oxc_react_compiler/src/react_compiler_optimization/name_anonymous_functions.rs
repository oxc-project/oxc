// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Port of NameAnonymousFunctions from TypeScript.
//!
//! Generates descriptive names for anonymous function expressions based on
//! how they are used (assigned to variables, passed as arguments to hooks/functions,
//! used as JSX props, etc.). These names appear in React DevTools and error stacks.
//!
//! Conditional on `env.config.enable_name_anonymous_functions`.

use std::borrow::Cow;
use std::mem::take;

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_str::{Ident, format_ident};

use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::object_shape::HookKind;
use crate::react_compiler_hir::{
    FunctionId, HirFunction, IdentifierId, IdentifierName, Instruction, InstructionValue,
    JsxAttribute, JsxTag, PlaceOrSpread,
};

/// Assign generated names to anonymous function expressions.
///
/// Ported from TS `nameAnonymousFunctions` in `Transform/NameAnonymousFunctions.ts`.
pub fn name_anonymous_functions<'a>(func: &mut HirFunction<'a>, env: &mut Environment<'a>) {
    let fn_id = match func.id {
        Some(id) => id,
        None => return,
    };

    let nodes = name_anonymous_functions_impl(func, env);

    fn visit<'a>(
        node: &Node<'a>,
        prefix: &str,
        updates: &mut Vec<(FunctionId, Ident<'a>)>,
        allocator: &'a Allocator,
    ) {
        if let Some(generated_name) = &node.generated_name
            && node.existing_name_hint.is_none()
        {
            // Only add the prefix to anonymous functions regardless of nesting depth
            let name = format_ident!(allocator, "{}{}]", prefix, generated_name);
            updates.push((node.function_id, name));
        }
        // Whether or not we generated a name for the function at this node,
        // traverse into its nested functions to assign them names
        let label =
            node.generated_name.as_deref().or(node.fn_name.as_deref()).unwrap_or("<anonymous>");
        let next_prefix = format!("{}{} > ", prefix, label);
        for inner in &node.inner {
            visit(inner, &next_prefix, updates, allocator);
        }
    }

    let mut updates: Vec<(FunctionId, Ident<'a>)> = Vec::new();
    let prefix = format!("{}[", fn_id);
    for node in &nodes {
        visit(node, &prefix, &mut updates, env.allocator);
    }

    if updates.is_empty() {
        return;
    }
    let update_map: FxHashMap<FunctionId, Ident<'a>> = updates.iter().copied().collect();

    // Apply name updates to the inner HirFunction in the arena
    for (function_id, name) in &updates {
        env.functions[*function_id].name_hint = Some(*name);
    }

    // Update name_hint on FunctionExpression instruction values in the outer function
    apply_name_hints_to_instructions(&mut func.instructions, &update_map);

    // Update name_hint on FunctionExpression instruction values in all arena functions
    for i in 0..env.functions.len() {
        // We need to temporarily take the instructions to avoid borrow issues
        let func_id = FunctionId::from_usize(i);
        let mut instructions = take(&mut env.functions[func_id].instructions);
        apply_name_hints_to_instructions(&mut instructions, &update_map);
        env.functions[func_id].instructions = instructions;
    }
}

/// Apply name hints to FunctionExpression instruction values.
fn apply_name_hints_to_instructions<'a>(
    instructions: &mut [Instruction<'a>],
    update_map: &FxHashMap<FunctionId, Ident<'a>>,
) {
    for instr in instructions.iter_mut() {
        if let InstructionValue::FunctionExpression { lowered_func, name_hint, .. } =
            &mut instr.value
            && let Some(new_name) = update_map.get(&lowered_func.func)
        {
            *name_hint = Some(*new_name);
        }
    }
}

struct Node<'a> {
    /// The FunctionId for the inner function (via lowered_func.func)
    function_id: FunctionId,
    /// The generated name for this anonymous function (set based on usage context)
    generated_name: Option<Ident<'a>>,
    /// The existing `name` on the FunctionExpression (non-anonymous functions have this)
    fn_name: Option<Ident<'a>>,
    /// Whether the inner HirFunction already has a name_hint
    existing_name_hint: Option<Ident<'a>>,
    /// Nested function nodes
    inner: Vec<Node<'a>>,
}

fn name_anonymous_functions_impl<'a>(
    func: &HirFunction<'a>,
    env: &Environment<'a>,
) -> Vec<Node<'a>> {
    // Functions that we track to generate names for
    let mut functions: FxHashMap<IdentifierId, usize> = FxHashMap::default();
    // Tracks temporaries that read from variables/globals/properties
    let mut names: FxHashMap<IdentifierId, Ident<'a>> = FxHashMap::default();
    // Tracks all function nodes
    let mut nodes: Vec<Node<'a>> = Vec::new();

    for block in func.body.blocks.values() {
        for instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let lvalue_id = instr.lvalue.identifier;
            match &instr.value {
                InstructionValue::LoadGlobal { binding, .. } => {
                    names.insert(lvalue_id, binding.name());
                }
                InstructionValue::LoadContext { place, .. }
                | InstructionValue::LoadLocal { place, .. } => {
                    let ident = &env.identifiers[place.identifier];
                    if let Some(IdentifierName::Named(name)) = ident.name {
                        names.insert(lvalue_id, name);
                    }
                    // If the loaded place was tracked as a function, propagate
                    if let Some(&node_idx) = functions.get(&place.identifier) {
                        functions.insert(lvalue_id, node_idx);
                    }
                }
                InstructionValue::PropertyLoad { object, property, .. } => {
                    if let Some(object_name) = names.get(&object.identifier) {
                        let name = format_ident!(env.allocator, "{}.{}", object_name, property);
                        names.insert(lvalue_id, name);
                    }
                }
                InstructionValue::FunctionExpression { name, lowered_func, .. } => {
                    let inner_func = &env.functions[lowered_func.func];
                    let inner = name_anonymous_functions_impl(inner_func, env);
                    let node = Node {
                        function_id: lowered_func.func,
                        generated_name: None,
                        fn_name: *name,
                        existing_name_hint: inner_func.name_hint,
                        inner,
                    };
                    let idx = nodes.len();
                    nodes.push(node);
                    if name.is_none() {
                        // Only generate names for anonymous functions
                        functions.insert(lvalue_id, idx);
                    }
                }
                InstructionValue::StoreContext { lvalue: store_lvalue, value, .. }
                | InstructionValue::StoreLocal { lvalue: store_lvalue, value, .. } => {
                    if let Some(&node_idx) = functions.get(&value.identifier) {
                        let node = &mut nodes[node_idx];
                        let var_ident = &env.identifiers[store_lvalue.place.identifier];
                        if node.generated_name.is_none()
                            && let Some(IdentifierName::Named(var_name)) = var_ident.name
                        {
                            node.generated_name = Some(var_name);
                            functions.remove(&value.identifier);
                        }
                    }
                }
                InstructionValue::CallExpression { callee, args, .. } => {
                    handle_call(env, callee.identifier, args, &mut functions, &names, &mut nodes);
                }
                InstructionValue::MethodCall { property, args, .. } => {
                    handle_call(env, property.identifier, args, &mut functions, &names, &mut nodes);
                }
                InstructionValue::JsxExpression { tag, props, .. } => {
                    for attr in props {
                        match attr {
                            JsxAttribute::SpreadAttribute { .. } => continue,
                            JsxAttribute::Attribute { name: attr_name, place } => {
                                if let Some(&node_idx) = functions.get(&place.identifier) {
                                    let node = &mut nodes[node_idx];
                                    if node.generated_name.is_none() {
                                        let element_name = match tag {
                                            JsxTag::Builtin(builtin) => Some(builtin.name),
                                            JsxTag::Place(tag_place) => {
                                                names.get(&tag_place.identifier).copied()
                                            }
                                        };
                                        let prop_name = match element_name {
                                            None => *attr_name,
                                            Some(el_name) => format_ident!(
                                                env.allocator,
                                                "<{}>.{}",
                                                el_name,
                                                attr_name
                                            ),
                                        };
                                        node.generated_name = Some(prop_name);
                                        functions.remove(&place.identifier);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    nodes
}

/// Handle CallExpression / MethodCall to generate names for function arguments.
fn handle_call<'a>(
    env: &Environment<'a>,
    callee_id: IdentifierId,
    args: &[PlaceOrSpread],
    functions: &mut FxHashMap<IdentifierId, usize>,
    names: &FxHashMap<IdentifierId, Ident<'a>>,
    nodes: &mut [Node<'a>],
) {
    let callee_ident = &env.identifiers[callee_id];
    let callee_ty = &env.types[callee_ident.type_];
    let hook_kind = env.get_hook_kind_for_type(callee_ty).ok().flatten();

    let callee_name: Cow<'_, str> = if let Some(hk) = hook_kind {
        if *hk != HookKind::Custom {
            Cow::Owned(hk.to_string())
        } else {
            names
                .get(&callee_id)
                .map_or(Cow::Borrowed("(anonymous)"), |name| Cow::Borrowed(name.as_str()))
        }
    } else {
        names
            .get(&callee_id)
            .map_or(Cow::Borrowed("(anonymous)"), |name| Cow::Borrowed(name.as_str()))
    };

    // Count how many args are tracked functions
    let fn_arg_count = args
        .iter()
        .filter(|arg| {
            if let PlaceOrSpread::Place(p) = arg {
                functions.contains_key(&p.identifier)
            } else {
                false
            }
        })
        .count();

    for (i, arg) in args.iter().enumerate() {
        let place = match arg {
            PlaceOrSpread::Spread(_) => continue,
            PlaceOrSpread::Place(p) => p,
        };
        if let Some(&node_idx) = functions.get(&place.identifier) {
            let node = &mut nodes[node_idx];
            if node.generated_name.is_none() {
                let generated_name = if fn_arg_count > 1 {
                    format_ident!(env.allocator, "{}(arg{})", callee_name, i)
                } else {
                    format_ident!(env.allocator, "{}()", callee_name)
                };
                node.generated_name = Some(generated_name);
                functions.remove(&place.identifier);
            }
        }
    }
}
