/// Optimize for server-side rendering (SSR).
///
/// Port of `Optimization/OptimizeForSSR.ts` from the React Compiler.
///
/// Optimizes the code for running specifically in an SSR environment. This
/// optimization assumes that setState will not be called during render during
/// initial mount, which allows inlining useState/useReducer.
///
/// Optimizations:
/// - Inline useState/useReducer
/// - Remove effects (useEffect/useLayoutEffect/useInsertionEffect)
/// - Remove refs where known to be unused during render (e.g., directly passed to a DOM node)
/// - Remove event handlers
///
/// Note that an earlier pass already inlines useMemo/useCallback.
use rustc_hash::FxHashMap;

use crate::hir::{
    ArrayPatternElement, BlockId, CallArg, CallExpression, HIRFunction, IdentifierId,
    InstructionValue, JsxAttribute, JsxExpression, JsxTag, LValue, LoadLocal, Pattern, Place,
    PrimitiveValue, PrimitiveValueKind, StoreLocal,
    object_shape::{
        BUILT_IN_SET_STATE_ID, BUILT_IN_START_TRANSITION_ID, BUILT_IN_USE_EFFECT_HOOK_ID,
        BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        BUILT_IN_USE_STATE_ID,
    },
    types::{FunctionType, Type},
    visitors::{each_instruction_value_operand, each_terminal_operand},
};

/// Optimize the function for SSR output.
pub fn optimize_for_ssr(func: &mut HIRFunction) {
    let inlined_state = collect_inlinable_state(func);
    rewrite_for_ssr(func, &inlined_state);
}

/// Hook kind determined by shape_id matching.
enum HookKind {
    State,
    Reducer,
    Effect,
    LayoutEffect,
    InsertionEffect,
    EffectEvent,
}

/// Get the hook kind for a callee identifier based on its type's shape_id.
fn get_hook_kind_from_type(ty: &Type) -> Option<HookKind> {
    match ty {
        Type::Function(FunctionType { shape_id: Some(id), .. }) => match id.as_str() {
            BUILT_IN_USE_STATE_ID => Some(HookKind::State),
            "BuiltInUseReducer" => Some(HookKind::Reducer),
            BUILT_IN_USE_EFFECT_HOOK_ID => Some(HookKind::Effect),
            BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID => Some(HookKind::LayoutEffect),
            BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID => Some(HookKind::InsertionEffect),
            "BuiltInUseEffectEvent" => Some(HookKind::EffectEvent),
            _ => None,
        },
        _ => None,
    }
}

fn is_primitive_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive)
}

fn is_plain_object_type(ty: &Type) -> bool {
    matches!(ty, Type::Object(_))
}

fn is_array_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Object(crate::hir::types::ObjectType { shape_id: Some(id) })
            if id == "BuiltInArray"
    )
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
            if id == BUILT_IN_SET_STATE_ID
    )
}

fn is_start_transition_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
            if id == BUILT_IN_START_TRANSITION_ID
    )
}

/// First pass: identify useState/useReducer calls that can be safely inlined.
///
/// Supported cases:
/// - `const [state, ] = useState( <primitive-array-or-object> )`
/// - `const [state, ] = useReducer(..., <value>)`
/// - `const [state, ] = useReducer(..., <value>, <init>)`
fn collect_inlinable_state(func: &HIRFunction) -> FxHashMap<IdentifierId, InlineReplacement> {
    let mut inlined_state: FxHashMap<IdentifierId, InlineReplacement> = FxHashMap::default();

    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get(block_id) else { continue };
        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            match &instr.value {
                InstructionValue::Destructure(d) => {
                    // Allow destructuring of inlined states
                    if inlined_state.contains_key(&d.value.identifier.id)
                        && matches!(&d.lvalue.pattern, Pattern::Array(arr) if !arr.items.is_empty())
                    {
                        continue;
                    }
                }
                InstructionValue::CallExpression(call) => {
                    match get_hook_kind_from_type(&call.callee.identifier.type_) {
                        Some(HookKind::Reducer) => {
                            if call.args.len() == 2 {
                                if let Some(CallArg::Place(arg)) = call.args.get(1) {
                                    inlined_state.insert(
                                        lvalue_id,
                                        InlineReplacement::LoadLocal(arg.clone()),
                                    );
                                }
                            } else if call.args.len() == 3
                                && let (
                                    Some(CallArg::Place(arg)),
                                    Some(CallArg::Place(initializer)),
                                ) = (call.args.get(1), call.args.get(2))
                            {
                                inlined_state.insert(
                                    lvalue_id,
                                    InlineReplacement::CallExpression {
                                        callee: initializer.clone(),
                                        arg: arg.clone(),
                                        loc: instr.value.loc(),
                                    },
                                );
                            }
                        }
                        Some(HookKind::State) => {
                            if call.args.len() == 1
                                && let Some(CallArg::Place(arg)) = call.args.first()
                                && (is_primitive_type(&arg.identifier.type_)
                                    || is_plain_object_type(&arg.identifier.type_)
                                    || is_array_type(&arg.identifier.type_))
                            {
                                inlined_state
                                    .insert(lvalue_id, InlineReplacement::LoadLocal(arg.clone()));
                            }
                        }
                        _ => {}
                    }
                }
                InstructionValue::MethodCall(call) => {
                    match get_hook_kind_from_type(&call.property.identifier.type_) {
                        Some(HookKind::Reducer) => {
                            if call.args.len() == 2 {
                                if let Some(CallArg::Place(arg)) = call.args.get(1) {
                                    inlined_state.insert(
                                        lvalue_id,
                                        InlineReplacement::LoadLocal(arg.clone()),
                                    );
                                }
                            } else if call.args.len() == 3
                                && let (
                                    Some(CallArg::Place(arg)),
                                    Some(CallArg::Place(initializer)),
                                ) = (call.args.get(1), call.args.get(2))
                            {
                                inlined_state.insert(
                                    lvalue_id,
                                    InlineReplacement::CallExpression {
                                        callee: initializer.clone(),
                                        arg: arg.clone(),
                                        loc: instr.value.loc(),
                                    },
                                );
                            }
                        }
                        Some(HookKind::State) => {
                            if call.args.len() == 1
                                && let Some(CallArg::Place(arg)) = call.args.first()
                                && (is_primitive_type(&arg.identifier.type_)
                                    || is_plain_object_type(&arg.identifier.type_)
                                    || is_array_type(&arg.identifier.type_))
                            {
                                inlined_state
                                    .insert(lvalue_id, InlineReplacement::LoadLocal(arg.clone()));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            // Any use of useState/useReducer return besides destructuring prevents inlining
            if !inlined_state.is_empty() {
                for operand in each_instruction_value_operand(&instr.value) {
                    inlined_state.remove(&operand.identifier.id);
                }
            }
        }
        if !inlined_state.is_empty() {
            for operand in each_terminal_operand(&block.terminal) {
                inlined_state.remove(&operand.identifier.id);
            }
        }
    }

    inlined_state
}

/// What to replace a useState/useReducer call with.
#[derive(Debug, Clone)]
enum InlineReplacement {
    /// Replace with a LoadLocal of the initial value.
    LoadLocal(Place),
    /// Replace with a CallExpression (for useReducer with init function).
    CallExpression { callee: Place, arg: Place, loc: crate::compiler_error::SourceLocation },
}

/// Check if a function contains a known non-render call (setState or startTransition).
fn has_known_non_render_call(func: &HIRFunction) -> bool {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::CallExpression(call) = &instr.value
                && (is_set_state_type(&call.callee.identifier.type_)
                    || is_start_transition_type(&call.callee.identifier.type_))
            {
                return true;
            }
        }
    }
    false
}

/// Check if a prop name is a known event handler (starts with "on" + uppercase letter).
fn is_known_event_handler(prop: &str) -> bool {
    if prop.len() >= 3 && prop.starts_with("on") {
        let third_char = prop.as_bytes()[2];
        third_char.is_ascii_uppercase()
    } else {
        false
    }
}

/// Second pass: rewrite instructions for SSR.
fn rewrite_for_ssr(
    func: &mut HIRFunction,
    inlined_state: &FxHashMap<IdentifierId, InlineReplacement>,
) {
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
        for instr in &mut block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    if has_known_non_render_call(&v.lowered_func.func) {
                        let loc = instr.value.loc();
                        instr.value = InstructionValue::Primitive(PrimitiveValue {
                            value: PrimitiveValueKind::Undefined,
                            loc,
                        });
                    }
                }
                InstructionValue::JsxExpression(jsx) => {
                    if let JsxTag::BuiltIn(tag) = &jsx.tag
                        && !tag.name.contains('-')
                    {
                        let loc = jsx.loc;
                        let tag_clone = jsx.tag.clone();
                        let children = jsx.children.clone();
                        let opening_loc = jsx.opening_loc;
                        let closing_loc = jsx.closing_loc;
                        // Filter out event handlers and ref props
                        let filtered_props: Vec<JsxAttribute> = jsx
                            .props
                            .iter()
                            .filter(|prop| match prop {
                                JsxAttribute::Spread { .. } => true,
                                JsxAttribute::Attribute { name, .. } => {
                                    !is_known_event_handler(name) && name != "ref"
                                }
                            })
                            .cloned()
                            .collect();
                        instr.value = InstructionValue::JsxExpression(JsxExpression {
                            tag: tag_clone,
                            props: filtered_props,
                            children,
                            loc,
                            opening_loc,
                            closing_loc,
                        });
                    }
                }
                InstructionValue::Destructure(d) => {
                    if inlined_state.contains_key(&d.value.identifier.id) {
                        // Verify the pattern is an array destructure with at least one item
                        if let Pattern::Array(arr) = &d.lvalue.pattern
                            && let Some(ArrayPatternElement::Place(item)) = arr.items.first()
                        {
                            let loc = d.loc;
                            let kind = d.lvalue.kind;
                            let value = d.value.clone();
                            let item_place = item.clone();
                            instr.value = InstructionValue::StoreLocal(StoreLocal {
                                lvalue: LValue { place: item_place, kind },
                                value,
                                loc,
                            });
                        }
                    }
                }
                InstructionValue::CallExpression(call) => {
                    let callee_type = &call.callee.identifier.type_;
                    match get_hook_kind_from_type(callee_type) {
                        Some(HookKind::EffectEvent) => {
                            if call.args.len() == 1
                                && let Some(CallArg::Place(arg)) = call.args.first()
                            {
                                let loc = instr.value.loc();
                                instr.value = InstructionValue::LoadLocal(LoadLocal {
                                    place: arg.clone(),
                                    loc,
                                });
                            }
                        }
                        Some(
                            HookKind::Effect | HookKind::LayoutEffect | HookKind::InsertionEffect,
                        ) => {
                            let loc = instr.value.loc();
                            instr.value = InstructionValue::Primitive(PrimitiveValue {
                                value: PrimitiveValueKind::Undefined,
                                loc,
                            });
                        }
                        Some(HookKind::Reducer | HookKind::State) => {
                            if let Some(replacement) =
                                inlined_state.get(&instr.lvalue.identifier.id)
                            {
                                let loc = instr.value.loc();
                                instr.value = match replacement {
                                    InlineReplacement::LoadLocal(place) => {
                                        InstructionValue::LoadLocal(LoadLocal {
                                            place: place.clone(),
                                            loc,
                                        })
                                    }
                                    InlineReplacement::CallExpression {
                                        callee,
                                        arg,
                                        loc: call_loc,
                                    } => InstructionValue::CallExpression(CallExpression {
                                        callee: callee.clone(),
                                        args: vec![CallArg::Place(arg.clone())],
                                        loc: *call_loc,
                                    }),
                                };
                            }
                        }
                        _ => {}
                    }
                }
                InstructionValue::MethodCall(call) => {
                    let callee_type = &call.property.identifier.type_;
                    match get_hook_kind_from_type(callee_type) {
                        Some(HookKind::EffectEvent) => {
                            if call.args.len() == 1
                                && let Some(CallArg::Place(arg)) = call.args.first()
                            {
                                let loc = instr.value.loc();
                                instr.value = InstructionValue::LoadLocal(LoadLocal {
                                    place: arg.clone(),
                                    loc,
                                });
                            }
                        }
                        Some(
                            HookKind::Effect | HookKind::LayoutEffect | HookKind::InsertionEffect,
                        ) => {
                            let loc = instr.value.loc();
                            instr.value = InstructionValue::Primitive(PrimitiveValue {
                                value: PrimitiveValueKind::Undefined,
                                loc,
                            });
                        }
                        Some(HookKind::Reducer | HookKind::State) => {
                            if let Some(replacement) =
                                inlined_state.get(&instr.lvalue.identifier.id)
                            {
                                let loc = instr.value.loc();
                                instr.value = match replacement {
                                    InlineReplacement::LoadLocal(place) => {
                                        InstructionValue::LoadLocal(LoadLocal {
                                            place: place.clone(),
                                            loc,
                                        })
                                    }
                                    InlineReplacement::CallExpression {
                                        callee,
                                        arg,
                                        loc: call_loc,
                                    } => InstructionValue::CallExpression(CallExpression {
                                        callee: callee.clone(),
                                        args: vec![CallArg::Place(arg.clone())],
                                        loc: *call_loc,
                                    }),
                                };
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
