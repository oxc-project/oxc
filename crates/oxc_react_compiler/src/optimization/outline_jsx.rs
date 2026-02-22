/// Outline JSX elements from reactive scopes.
///
/// Port of `Optimization/OutlineJsx.ts` from the React Compiler.
///
/// Moves JSX element creation out of reactive scopes when safe, converting
/// them into outlined component functions. This can reduce the amount of
/// work done during re-renders.
///
/// The algorithm iterates blocks in reverse, collecting sibling JSX groups.
/// For each group of >= 2 JSX instructions, it:
/// 1. Collects the props (returning `None` if any spread is found)
/// 2. Emits a replacement JSX element that references the outlined tag
/// 3. Emits a new `HIRFunction` for the outlined component
/// 4. Calls dead code elimination after rewriting
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::GENERATED_SOURCE,
    hir::{
        BasicBlock, BlockId, BlockKind, Destructure, HIRFunction, Hir, IdentifierId,
        IdentifierName, Instruction, InstructionId, InstructionKind, InstructionValue,
        JsxAttribute, JsxExpression, JsxTag, LValuePattern, LoadGlobal, NonLocalBinding,
        ObjectPattern, ObjectPatternProperty, ObjectProperty, ObjectPropertyKey,
        ObjectPropertyType, Pattern, Place, ReactFunctionType, ReactiveParam, ReturnTerminal,
        ReturnVariant, Terminal, hir_builder::create_temporary_place,
    },
};

use super::dead_code_elimination::dead_code_elimination;

/// Outline JSX elements from reactive scopes.
pub fn outline_jsx(func: &mut HIRFunction) {
    let mut outlined_fns: Vec<HIRFunction> = Vec::new();
    outline_jsx_impl(func, &mut outlined_fns);

    for outlined_fn in outlined_fns {
        func.env.outline_function(outlined_fn, Some(ReactFunctionType::Component));
    }
}

/// An attribute collected from JSX for outlining.
#[derive(Debug, Clone)]
struct OutlinedJsxAttribute {
    original_name: String,
    new_name: String,
    place: Place,
}

/// The result of processing a JSX group for outlining.
struct OutlinedResult {
    instrs: Vec<Instruction>,
    func: HIRFunction,
}

/// State for tracking sibling JSX groups during reverse iteration.
struct JsxState {
    /// JSX instructions collected (in reverse order).
    jsx: Vec<usize>,
    /// Identifier IDs of children of collected JSX instructions.
    children: FxHashSet<IdentifierId>,
}

impl JsxState {
    fn new() -> Self {
        Self { jsx: Vec::new(), children: FxHashSet::default() }
    }
}

fn outline_jsx_impl(func: &mut HIRFunction, outlined_fns: &mut Vec<HIRFunction>) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for block_id in block_ids {
        let mut rewrite_instr: FxHashMap<InstructionId, Vec<Instruction>> = FxHashMap::default();
        let mut block_globals: FxHashMap<IdentifierId, Instruction> = FxHashMap::default();

        // Collect all the information we need from the block upfront, so we
        // can drop the immutable borrow before calling functions that need
        // `&mut func`.
        #[derive(Clone)]
        enum InstrInfo {
            LoadGlobal { id: IdentifierId, instr: Instruction },
            FunctionExpression,
            Jsx { lvalue_id: IdentifierId, children_ids: Vec<IdentifierId>, idx: usize },
            Other,
        }

        let instr_infos: Vec<InstrInfo> = {
            let Some(block) = func.body.blocks.get(&block_id) else {
                continue;
            };
            block
                .instructions
                .iter()
                .enumerate()
                .map(|(idx, instr)| match &instr.value {
                    InstructionValue::LoadGlobal(_) => InstrInfo::LoadGlobal {
                        id: instr.lvalue.identifier.id,
                        instr: instr.clone(),
                    },
                    InstructionValue::FunctionExpression(_) => InstrInfo::FunctionExpression,
                    InstructionValue::JsxExpression(jsx_expr) => {
                        let children_ids = jsx_expr
                            .children
                            .as_ref()
                            .map(|children| children.iter().map(|c| c.identifier.id).collect())
                            .unwrap_or_default();
                        InstrInfo::Jsx { lvalue_id: instr.lvalue.identifier.id, children_ids, idx }
                    }
                    _ => InstrInfo::Other,
                })
                .collect()
        };

        // First pass: collect globals from this block
        for info in &instr_infos {
            if let InstrInfo::LoadGlobal { id, instr } = info {
                block_globals.insert(*id, instr.clone());
            }
        }

        // Second pass: reverse iterate to find JSX groups
        let mut state = JsxState::new();
        for info in instr_infos.iter().rev() {
            match info {
                InstrInfo::LoadGlobal { .. } | InstrInfo::FunctionExpression | InstrInfo::Other => {
                }
                InstrInfo::Jsx { lvalue_id, children_ids, idx } => {
                    if !state.children.contains(lvalue_id) {
                        // This JSX is not a child of a previously collected JSX,
                        // so process the current group and start a new one
                        process_and_outline_jsx(
                            func,
                            &state,
                            &mut rewrite_instr,
                            &block_globals,
                            outlined_fns,
                            block_id,
                        );
                        state = JsxState::new();
                    }
                    state.jsx.push(*idx);
                    for child_id in children_ids {
                        state.children.insert(*child_id);
                    }
                }
            }
        }
        // Process final group
        process_and_outline_jsx(
            func,
            &state,
            &mut rewrite_instr,
            &block_globals,
            outlined_fns,
            block_id,
        );

        // Rewrite instructions if needed
        if !rewrite_instr.is_empty() {
            let Some(block) = func.body.blocks.get_mut(&block_id) else {
                continue;
            };
            let mut new_instrs = Vec::new();
            for (i, instr) in block.instructions.iter().enumerate() {
                // InstructionId's are one-indexed, so add one to account for them.
                let id = InstructionId(u32::try_from(i).unwrap_or(0) + 1);
                if let Some(replacement) = rewrite_instr.get(&id) {
                    new_instrs.extend(replacement.iter().cloned());
                } else {
                    new_instrs.push(instr.clone());
                }
            }
            block.instructions = new_instrs;
        }

        // Recurse into inner function expressions
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            if let InstructionValue::FunctionExpression(func_expr) = &mut instr.value {
                outline_jsx_impl(&mut func_expr.lowered_func.func, outlined_fns);
            }
        }

        dead_code_elimination(func);
    }
}

fn process_and_outline_jsx(
    func: &mut HIRFunction,
    state: &JsxState,
    rewrite_instr: &mut FxHashMap<InstructionId, Vec<Instruction>>,
    globals: &FxHashMap<IdentifierId, Instruction>,
    outlined_fns: &mut Vec<HIRFunction>,
    block_id: BlockId,
) {
    if state.jsx.len() <= 1 {
        return;
    }

    // Clone all data we need from the block before calling `process` which
    // needs `&mut func`.
    let (jsx_instrs, first_id) = {
        let Some(block) = func.body.blocks.get(&block_id) else {
            return;
        };

        // Sort JSX indices by instruction id (ascending)
        let mut sorted_indices = state.jsx.clone();
        sorted_indices.sort_by_key(|&idx| block.instructions[idx].id);

        // Collect JSX instructions (cloned so we can drop the block borrow)
        let jsx_instrs: Vec<Instruction> =
            sorted_indices.iter().map(|&idx| block.instructions[idx].clone()).collect();

        // Capture the first instruction's id for the rewrite key
        let first_id = sorted_indices.first().map(|&idx| block.instructions[idx].id);

        (jsx_instrs, first_id)
    };
    // Block borrow is now dropped.

    let result = process(func, &jsx_instrs, globals);
    if let Some(result) = result {
        outlined_fns.push(result.func);
        if let Some(first_id) = first_id {
            rewrite_instr.insert(first_id, result.instrs);
        }
    }
}

fn process(
    func: &mut HIRFunction,
    jsx: &[Instruction],
    globals: &FxHashMap<IdentifierId, Instruction>,
) -> Option<OutlinedResult> {
    // Only outline jsx in callbacks, not in top-level components
    if func.fn_type == ReactFunctionType::Component {
        return None;
    }

    let props = collect_props(&mut func.env, jsx)?;

    let outlined_tag_name = func.env.generate_globally_unique_identifier_name(None);
    let outlined_tag = outlined_tag_name.value().to_string();

    let new_instrs = emit_outlined_jsx(&mut func.env, jsx, &props, &outlined_tag);

    let mut outlined_fn = emit_outlined_fn(&mut func.env, jsx, &props, globals)?;
    outlined_fn.id = Some(outlined_tag);

    Some(OutlinedResult { instrs: new_instrs, func: outlined_fn })
}

fn collect_props(
    env: &mut crate::hir::environment::Environment,
    instructions: &[Instruction],
) -> Option<Vec<OutlinedJsxAttribute>> {
    let mut counter = 1u32;
    let mut seen: FxHashSet<String> = FxHashSet::default();
    let mut attributes: Vec<OutlinedJsxAttribute> = Vec::new();
    let jsx_ids: FxHashSet<IdentifierId> =
        instructions.iter().map(|i| i.lvalue.identifier.id).collect();

    let mut generate_name = |old_name: &str, env: &mut crate::hir::environment::Environment| {
        let mut new_name = old_name.to_string();
        while seen.contains(&new_name) {
            new_name = format!("{old_name}{counter}");
            counter += 1;
        }
        seen.insert(new_name.clone());
        env.add_new_reference(&new_name);
        new_name
    };

    for instr in instructions {
        let jsx_expr = match &instr.value {
            InstructionValue::JsxExpression(expr) => expr,
            _ => continue,
        };

        for attr in &jsx_expr.props {
            match attr {
                JsxAttribute::Spread { .. } => {
                    // Cannot outline if there are spread attributes
                    return None;
                }
                JsxAttribute::Attribute { name, place } => {
                    let new_name = generate_name(name, env);
                    attributes.push(OutlinedJsxAttribute {
                        original_name: name.clone(),
                        new_name,
                        place: place.clone(),
                    });
                }
            }
        }

        if let Some(children) = &jsx_expr.children {
            for child in children {
                if jsx_ids.contains(&child.identifier.id) {
                    continue;
                }

                // Promote temporary
                let mut promoted_child = child.clone();
                if promoted_child.identifier.name.is_none() {
                    promoted_child.identifier.name = Some(IdentifierName::Promoted(format!(
                        "#t{}",
                        promoted_child.identifier.declaration_id.0
                    )));
                }

                let new_name = generate_name("t", env);
                let child_name = match &promoted_child.identifier.name {
                    Some(name) => name.value().to_string(),
                    None => format!("t{}", promoted_child.identifier.id.0),
                };
                attributes.push(OutlinedJsxAttribute {
                    original_name: child_name,
                    new_name,
                    place: promoted_child,
                });
            }
        }
    }

    Some(attributes)
}

fn emit_outlined_jsx(
    env: &mut crate::hir::environment::Environment,
    instructions: &[Instruction],
    outlined_props: &[OutlinedJsxAttribute],
    outlined_tag: &str,
) -> Vec<Instruction> {
    let props: Vec<JsxAttribute> = outlined_props
        .iter()
        .map(|p| JsxAttribute::Attribute { name: p.new_name.clone(), place: p.place.clone() })
        .collect();

    let mut load_lvalue = create_temporary_place(env, GENERATED_SOURCE);
    // Promote the lvalue as a JSX tag (using #T prefix)
    if load_lvalue.identifier.name.is_none() {
        load_lvalue.identifier.name = Some(IdentifierName::Promoted(format!(
            "#T{}",
            load_lvalue.identifier.declaration_id.0
        )));
    }

    let load_jsx = Instruction {
        id: InstructionId::ZERO,
        loc: GENERATED_SOURCE,
        lvalue: load_lvalue.clone(),
        value: InstructionValue::LoadGlobal(LoadGlobal {
            binding: NonLocalBinding::ModuleLocal { name: outlined_tag.to_string() },
            loc: GENERATED_SOURCE,
        }),
        effects: None,
    };

    // Use the last instruction's lvalue for the JSX expression
    let jsx_lvalue = match instructions.last() {
        Some(last) => last.lvalue.clone(),
        None => create_temporary_place(env, GENERATED_SOURCE),
    };

    let jsx_expr = Instruction {
        id: InstructionId::ZERO,
        loc: GENERATED_SOURCE,
        lvalue: jsx_lvalue,
        value: InstructionValue::JsxExpression(JsxExpression {
            tag: JsxTag::Place(load_lvalue),
            props,
            children: None,
            loc: GENERATED_SOURCE,
            opening_loc: GENERATED_SOURCE,
            closing_loc: GENERATED_SOURCE,
        }),
        effects: None,
    };

    vec![load_jsx, jsx_expr]
}

fn emit_outlined_fn(
    env: &mut crate::hir::environment::Environment,
    jsx: &[Instruction],
    old_props: &[OutlinedJsxAttribute],
    globals: &FxHashMap<IdentifierId, Instruction>,
) -> Option<HIRFunction> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let old_to_new_props = create_old_to_new_props_mapping(env, old_props);

    let mut props_obj = create_temporary_place(env, GENERATED_SOURCE);
    // Promote temporary
    if props_obj.identifier.name.is_none() {
        props_obj.identifier.name =
            Some(IdentifierName::Promoted(format!("#t{}", props_obj.identifier.declaration_id.0)));
    }

    let destructure_instr = emit_destructure_props(env, &props_obj, &old_to_new_props);
    instructions.push(destructure_instr);

    let updated_jsx = emit_updated_jsx(jsx, &old_to_new_props);
    let load_global_instrs = emit_load_globals(jsx, globals)?;
    instructions.extend(load_global_instrs);
    instructions.extend(updated_jsx);

    let return_lvalue = match instructions.last() {
        Some(last) => last.lvalue.clone(),
        None => create_temporary_place(env, GENERATED_SOURCE),
    };

    let block = BasicBlock {
        kind: BlockKind::Block,
        id: BlockId(0),
        instructions,
        terminal: Terminal::Return(ReturnTerminal {
            id: InstructionId::ZERO,
            value: return_lvalue,
            return_variant: ReturnVariant::Explicit,
            loc: GENERATED_SOURCE,
        }),
        preds: FxHashSet::default(),
        phis: Vec::new(),
    };

    let mut blocks = FxHashMap::default();
    blocks.insert(block.id, block);

    Some(HIRFunction {
        loc: GENERATED_SOURCE,
        id: None,
        name_hint: None,
        fn_type: ReactFunctionType::Other,
        env: env.clone(),
        params: vec![ReactiveParam::Place(props_obj)],
        returns: create_temporary_place(env, GENERATED_SOURCE),
        context: Vec::new(),
        body: Hir { entry: BlockId(0), blocks },
        generator: false,
        is_async: false,
        directives: Vec::new(),
        aliasing_effects: None,
    })
}

fn emit_load_globals(
    jsx: &[Instruction],
    globals: &FxHashMap<IdentifierId, Instruction>,
) -> Option<Vec<Instruction>> {
    let mut instructions = Vec::new();
    for instr in jsx {
        let jsx_expr = match &instr.value {
            InstructionValue::JsxExpression(expr) => expr,
            _ => continue,
        };
        // Add load globals instructions for JSX tags that are identifiers
        if let JsxTag::Place(tag_place) = &jsx_expr.tag {
            let load_global_instr = globals.get(&tag_place.identifier.id)?;
            instructions.push(load_global_instr.clone());
        }
    }
    Some(instructions)
}

fn emit_updated_jsx(
    jsx: &[Instruction],
    old_to_new_props: &FxHashMap<IdentifierId, OutlinedJsxAttribute>,
) -> Vec<Instruction> {
    let jsx_ids: FxHashSet<IdentifierId> = jsx.iter().map(|i| i.lvalue.identifier.id).collect();
    let mut new_instrs = Vec::new();

    for instr in jsx {
        let jsx_expr = match &instr.value {
            InstructionValue::JsxExpression(expr) => expr,
            _ => continue,
        };

        let mut new_props: Vec<JsxAttribute> = Vec::new();
        for prop in &jsx_expr.props {
            match prop {
                JsxAttribute::Attribute { name, place } => {
                    if name == "key" {
                        continue;
                    }
                    if let Some(new_prop) = old_to_new_props.get(&place.identifier.id) {
                        new_props.push(JsxAttribute::Attribute {
                            name: new_prop.original_name.clone(),
                            place: new_prop.place.clone(),
                        });
                    }
                }
                JsxAttribute::Spread { .. } => {
                    // Spreads should have been caught in collect_props
                }
            }
        }

        let new_children = jsx_expr.children.as_ref().map(|children| {
            children
                .iter()
                .map(|child| {
                    if jsx_ids.contains(&child.identifier.id) {
                        child.clone()
                    } else if let Some(new_child) = old_to_new_props.get(&child.identifier.id) {
                        new_child.place.clone()
                    } else {
                        child.clone()
                    }
                })
                .collect()
        });

        new_instrs.push(Instruction {
            id: instr.id,
            loc: instr.loc,
            lvalue: instr.lvalue.clone(),
            value: InstructionValue::JsxExpression(JsxExpression {
                tag: jsx_expr.tag.clone(),
                props: new_props,
                children: new_children,
                loc: jsx_expr.loc,
                opening_loc: jsx_expr.opening_loc,
                closing_loc: jsx_expr.closing_loc,
            }),
            effects: instr.effects.clone(),
        });
    }

    new_instrs
}

fn create_old_to_new_props_mapping(
    env: &mut crate::hir::environment::Environment,
    old_props: &[OutlinedJsxAttribute],
) -> FxHashMap<IdentifierId, OutlinedJsxAttribute> {
    let mut old_to_new = FxHashMap::default();

    for old_prop in old_props {
        // Do not read key prop in the outlined component
        if old_prop.original_name == "key" {
            continue;
        }

        let mut new_place = create_temporary_place(env, GENERATED_SOURCE);
        new_place.identifier.name = Some(IdentifierName::Named(old_prop.new_name.clone()));

        let new_prop = OutlinedJsxAttribute {
            original_name: old_prop.original_name.clone(),
            new_name: old_prop.new_name.clone(),
            place: new_place,
        };
        old_to_new.insert(old_prop.place.identifier.id, new_prop);
    }

    old_to_new
}

fn emit_destructure_props(
    env: &mut crate::hir::environment::Environment,
    props_obj: &Place,
    old_to_new_props: &FxHashMap<IdentifierId, OutlinedJsxAttribute>,
) -> Instruction {
    let properties: Vec<ObjectPatternProperty> = old_to_new_props
        .values()
        .map(|prop| {
            ObjectPatternProperty::Property(ObjectProperty {
                key: ObjectPropertyKey::String(prop.new_name.clone()),
                property_type: ObjectPropertyType::Property,
                place: prop.place.clone(),
            })
        })
        .collect();

    Instruction {
        id: InstructionId::ZERO,
        lvalue: create_temporary_place(env, GENERATED_SOURCE),
        loc: GENERATED_SOURCE,
        value: InstructionValue::Destructure(Destructure {
            lvalue: LValuePattern {
                pattern: Pattern::Object(ObjectPattern { properties, loc: GENERATED_SOURCE }),
                kind: InstructionKind::Let,
            },
            value: props_obj.clone(),
            loc: GENERATED_SOURCE,
        }),
        effects: None,
    }
}
