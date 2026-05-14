//! `InlineJsxTransform` optimization pass.
//!
//! Port of `Optimization/InlineJsxTransform.ts` (~797 LoC) from the React
//! Compiler. Gated by `Environment.config().inline_jsx_transform` (a non-null
//! `InlineJsxTransformConfig` carrying `{ element_symbol, global_dev_var }`).
//!
//! # Algorithm (mirrors upstream)
//!
//! For every `JsxExpression`/`JsxFragment` instruction in a *statement* block
//! (i.e. `BlockKind::Block` or `BlockKind::Catch`; value-kind blocks are
//! skipped because upstream cannot handle them either — see the upstream
//! `TODO: Support value blocks` branch), the pass splits the block in four:
//!
//! ```text
//! current  →  (if globalDevVar)  →  then  →  fallthrough
//!                                  else  →
//! ```
//!
//! * `current` ends with a new `if (globalDevVar)` terminal. Its instructions
//!   are everything that came *before* the JSX, plus a fresh `DeclareLocal t`
//!   declaration (`let`) and a `LoadGlobal globalDevVar` for the test.
//! * `then` contains the original JSX instruction, then a `StoreLocal
//!   t <- jsxResult` reassign, and goto-breaks to `fallthrough`.
//! * `else` builds a `ReactElement` object literal by emitting
//!   `Symbol.for("react.transitional.element")` for `$$typeof`, the JSX
//!   tag (`Primitive` for built-in tags, `Place` for component references,
//!   `Symbol.for("react.fragment")` for fragments), the `ref` and `key`
//!   properties (filled with `null` primitives when absent), and a
//!   `props` object with all remaining JSX attributes plus children.
//!   The literal is bound to `elseVarPlace` via a `StoreLocal Reassign`,
//!   then goto-breaks to `fallthrough`.
//! * `fallthrough` is everything that was after the JSX. The original `t`
//!   declaration is merged through a phi over the `then`/`else` reassigns.
//!
//! All places to the JSX result (everything reading the original `instr.lvalue`)
//! are rewritten in a second pass to point at the new phi identifier, except
//! inside `then` and `else` themselves (the original JSX still lives in `then`
//! and produces the same lvalue).
//!
//! Nested `FunctionExpression` / `ObjectMethod` bodies recurse so JSX inside
//! lambdas is inlined too.
//!
//! # Pipeline placement
//!
//! Runs immediately after `InferEffectDependencies` and before
//! `BuildReactiveFunction`, mirroring upstream `Pipeline.ts:418-425`.
//!
//! # Output shape
//!
//! See the `.expect.md` snapshot of
//! `__tests__/fixtures/compiler/inline-jsx-transform.js` for the canonical
//! production-branch shape:
//!
//! ```text
//! t1 = {
//!   $$typeof: Symbol.for("react.transitional.element"),
//!   type: <tag>,
//!   ref: <ref or null>,
//!   key: <key or null>,
//!   props: <props object or spread-source>,
//! };
//! ```

use rustc_hash::FxHashSet;

use crate::hir::{
    ArrayExpression, ArrayExpressionElement, BasicBlock, BlockId, BlockKind, DeclarationId,
    DeclareLocal, Effect, GotoTerminal, GotoVariant, HIRFunction, Identifier, IdentifierId,
    IdentifierName, IfTerminal, Instruction, InstructionId, InstructionKind, InstructionValue,
    JsxAttribute, JsxTag, LValue, LoadGlobal, MethodCall, MutableRange, NonLocalBinding,
    ObjectExpression, ObjectPatternProperty, ObjectProperty, ObjectPropertyKey, ObjectPropertyType,
    Phi, Place, PrimitiveValue, PrimitiveValueKind, PropertyLoad, ReactiveScope, SpreadPattern,
    StoreLocal, Terminal,
    build_reactive_scope_terminals_hir::fix_scope_and_identifier_ranges,
    environment::{Environment, InlineJsxTransformConfig},
    hir_builder::{
        create_temporary_place, mark_instruction_ids, mark_predecessors, reverse_postorder_blocks,
    },
    types::PropertyLiteral,
    visitors::{
        map_instruction_lvalues, map_instruction_operands, map_instruction_value_operands,
        map_terminal_operands,
    },
};

/// Tracking entry for a JSX instruction we've inlined. The original
/// `DeclarationId` of the JSX lvalue maps to the phi identifier that replaced
/// it, plus the set of block ids inside which we MUST NOT rewrite uses (those
/// being `then` and `else` — the original JSX still produces the same
/// declaration id inside `then`, and the production object literal stores
/// into a fork of the same declaration id inside `else`).
struct InlinedJsxDeclaration {
    identifier: Identifier,
    block_ids_to_ignore: FxHashSet<BlockId>,
}

/// Run the `InlineJsxTransform` optimization on a HIR function.
///
/// Mutates `func` in place. Recurses into nested function expressions and
/// object methods so JSX inside lambdas is inlined too.
pub fn inline_jsx_transform(func: &mut HIRFunction, config: &InlineJsxTransformConfig) {
    let mut inlined: rustc_hash::FxHashMap<DeclarationId, InlinedJsxDeclaration> =
        rustc_hash::FxHashMap::default();

    // -----------------------------------------------------------------------
    // Step 1: split blocks at each JsxExpression / JsxFragment and codegen
    // the `if (DEV) <jsx> else <object literal>` conditional.
    // -----------------------------------------------------------------------
    //
    // Iterate over a snapshot of block ids so we don't observe new blocks we
    // insert during this loop (TS: `for (const [_, currentBlock] of
    // [...fn.body.blocks])`).
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for current_block_id in block_ids {
        // A given source block may contain MULTIPLE JSX expressions. Upstream
        // handles this by initialising `fallthroughBlockInstructions` once
        // (the tail after the FIRST JSX in this block) and reusing it for
        // every subsequent split — every new fallthrough block becomes the
        // start of the next iteration's tail. The Rust port adopts the same
        // strategy: walk instructions, on every JSX split mutate
        // `current_block` to end with an If, push a new fallthrough block,
        // and continue iteration in the fallthrough.
        let mut working_block_id = current_block_id;
        // The body re-borrows `func.body.blocks` mutably inside the loop
        // (insert/get_mut). `clippy::while_let_loop` would suggest rewriting
        // as `while let Some(block_ref) = func.body.blocks.get(&working_block_id) { ... }`
        // but that holds an immutable borrow across the body, which would
        // forbid the later `func.body.blocks.get_mut` / `.insert` calls.
        #[expect(clippy::while_let_loop)]
        loop {
            // Snapshot the current block's contents.
            let Some(block_ref) = func.body.blocks.get(&working_block_id) else { break };
            let block_kind = block_ref.kind;
            let instructions = block_ref.instructions.clone();

            // Recurse into FunctionExpression / ObjectMethod bodies. This is
            // separate from the JSX-split logic because nested function
            // bodies don't participate in the parent's CFG split.
            // We do this in-place via a second walk after we identify a
            // possible JSX split — but if there's no JSX in this block,
            // we still must recurse.
            //
            // Find the first JSX instruction in this block.
            let mut jsx_index: Option<usize> = None;
            for (i, instr) in instructions.iter().enumerate() {
                if matches!(
                    &instr.value,
                    InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_)
                ) {
                    // Skip JSX in value blocks (upstream TODO).
                    if matches!(block_kind, BlockKind::Value) {
                        continue;
                    }
                    jsx_index = Some(i);
                    break;
                }
            }

            // Recurse into nested function bodies of THIS block's
            // instructions regardless of whether we found JSX, so that
            // child closures are processed even when their outer block has
            // no top-level JSX.
            {
                let Some(block_mut) = func.body.blocks.get_mut(&working_block_id) else { break };
                for instr in &mut block_mut.instructions {
                    match &mut instr.value {
                        InstructionValue::FunctionExpression(fe) => {
                            inline_jsx_transform(&mut fe.lowered_func.func, config);
                        }
                        InstructionValue::ObjectMethod(om) => {
                            inline_jsx_transform(&mut om.lowered_func.func, config);
                        }
                        _ => {}
                    }
                }
            }

            let Some(jsx_idx) = jsx_index else { break };

            // Re-snapshot instructions because we may have rewritten nested
            // lambdas above.
            let Some(block_ref) = func.body.blocks.get(&working_block_id) else { break };
            let instructions = block_ref.instructions.clone();
            let block_kind = block_ref.kind;
            let block_terminal = block_ref.terminal.clone();

            // Compute the slices: instructions BEFORE the JSX, the JSX
            // itself, and everything AFTER.
            let mut before_instructions: Vec<Instruction> = instructions[..jsx_idx].to_vec();
            let jsx_instr = instructions[jsx_idx].clone();
            let after_instructions: Vec<Instruction> = instructions[jsx_idx + 1..].to_vec();

            // Pull what we need from the JSX instruction.
            let jsx_loc = jsx_instr.loc;
            let jsx_value_loc = jsx_instr.value.loc();
            let original_lvalue = jsx_instr.lvalue.clone();

            // -- Allocate the four block ids and the four key Places. --
            //
            // Upstream allocates a `varPlace` (the "merged" t1 identifier),
            // forks it into `thenVarPlace` and `elseVarPlace` (separate
            // IdentifierId-s sharing the same DeclarationId), and a fresh
            // `phiIdentifier`. The phi merges then/else into one declaration.
            let mut var_place = create_temporary_place(&mut func.env, jsx_value_loc);
            promote_temporary(&mut var_place);
            let var_lvalue_place = create_temporary_place(&mut func.env, jsx_value_loc);

            let then_var_place = fork_temporary(&mut func.env, &var_place);
            let else_var_place = fork_temporary(&mut func.env, &var_place);

            // DeclareLocal instruction: `let t1;`
            let var_instr = Instruction {
                id: InstructionId(0),
                lvalue: var_lvalue_place,
                value: InstructionValue::DeclareLocal(DeclareLocal {
                    lvalue: LValue { place: var_place.clone(), kind: InstructionKind::Let },
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            before_instructions.push(var_instr);

            // LoadGlobal devVar -> test for the new If terminal.
            let mut dev_global_place = create_temporary_place(&mut func.env, jsx_value_loc);
            dev_global_place.effect = Effect::Mutate;
            let dev_global_instr = Instruction {
                id: InstructionId(0),
                lvalue: dev_global_place.clone(),
                value: InstructionValue::LoadGlobal(LoadGlobal {
                    binding: NonLocalBinding::Global { name: config.global_dev_var.clone() },
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            before_instructions.push(dev_global_instr);

            let then_block_id = func.env.next_block_id();
            let else_block_id = func.env.next_block_id();
            let fallthrough_block_id = func.env.next_block_id();

            let mut if_test = dev_global_place;
            if_test.effect = Effect::Read;
            let if_terminal = Terminal::If(IfTerminal {
                id: InstructionId(0),
                test: if_test,
                consequent: then_block_id,
                alternate: else_block_id,
                fallthrough: fallthrough_block_id,
                loc: jsx_loc,
            });

            // -- Build the `then` block: original JSX + StoreLocal reassign. --
            let mut then_instructions: Vec<Instruction> = vec![jsx_instr.clone()];
            let then_reassign_lvalue = create_temporary_place(&mut func.env, jsx_value_loc);
            let then_reassign_instr = Instruction {
                id: InstructionId(0),
                lvalue: then_reassign_lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue {
                        place: then_var_place.clone(),
                        kind: InstructionKind::Reassign,
                    },
                    value: original_lvalue.clone(),
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            then_instructions.push(then_reassign_instr);

            let then_block = BasicBlock {
                kind: BlockKind::Block,
                id: then_block_id,
                instructions: then_instructions,
                terminal: Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: fallthrough_block_id,
                    variant: GotoVariant::Break,
                    loc: jsx_loc,
                }),
                preds: FxHashSet::default(),
                phis: Vec::new(),
            };

            // -- Build the `else` block: ReactElement object literal. --
            let mut else_instructions: Vec<Instruction> = Vec::new();
            let (jsx_kind_is_fragment, props_attributes, children) = match &jsx_instr.value {
                InstructionValue::JsxExpression(e) => (false, e.props.clone(), e.children.clone()),
                InstructionValue::JsxFragment(f) => (true, Vec::new(), Some(f.children.clone())),
                _ => unreachable!("jsx_index only matches Jsx* instruction values"),
            };

            let PropsProperties { ref_property, key_property, props_property } =
                create_props_properties(
                    &mut func.env,
                    &jsx_instr,
                    &mut else_instructions,
                    &props_attributes,
                    children.as_deref(),
                );

            // `$$typeof: Symbol.for("react.transitional.element")`.
            let typeof_property = create_symbol_property(
                &mut func.env,
                &jsx_instr,
                &mut else_instructions,
                "$$typeof",
                &config.element_symbol,
            );

            // `type` property:
            //  - JsxFragment        => Symbol.for("react.fragment")
            //  - JsxExpression+Builtin => Primitive("tag-name")
            //  - JsxExpression+Place   => the Place itself
            let type_property = if jsx_kind_is_fragment {
                create_symbol_property(
                    &mut func.env,
                    &jsx_instr,
                    &mut else_instructions,
                    "type",
                    "react.fragment",
                )
            } else {
                let tag = match &jsx_instr.value {
                    InstructionValue::JsxExpression(e) => e.tag.clone(),
                    _ => unreachable!(),
                };
                create_tag_property(&mut func.env, &jsx_instr, &mut else_instructions, &tag)
            };

            let mut react_element_lvalue = create_temporary_place(&mut func.env, jsx_value_loc);
            react_element_lvalue.effect = Effect::Store;
            let react_element_instr = Instruction {
                id: InstructionId(0),
                lvalue: react_element_lvalue.clone(),
                value: InstructionValue::ObjectExpression(ObjectExpression {
                    properties: vec![
                        ObjectPatternProperty::Property(typeof_property),
                        ObjectPatternProperty::Property(type_property),
                        ObjectPatternProperty::Property(ref_property),
                        ObjectPatternProperty::Property(key_property),
                        ObjectPatternProperty::Property(props_property),
                    ],
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            else_instructions.push(react_element_instr);

            // StoreLocal Reassign elseVarPlace <- reactElement.lvalue
            let else_reassign_lvalue = create_temporary_place(&mut func.env, jsx_value_loc);
            let else_reassign_instr = Instruction {
                id: InstructionId(0),
                lvalue: else_reassign_lvalue,
                value: InstructionValue::StoreLocal(StoreLocal {
                    lvalue: LValue {
                        place: else_var_place.clone(),
                        kind: InstructionKind::Reassign,
                    },
                    value: react_element_lvalue,
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            else_instructions.push(else_reassign_instr);

            let else_block = BasicBlock {
                kind: BlockKind::Block,
                id: else_block_id,
                instructions: else_instructions,
                terminal: Terminal::Goto(GotoTerminal {
                    id: InstructionId(0),
                    block: fallthrough_block_id,
                    variant: GotoVariant::Break,
                    loc: jsx_loc,
                }),
                preds: FxHashSet::default(),
                phis: Vec::new(),
            };

            // -- Build the fallthrough block: everything that was after JSX. --
            //
            // Phi: in `then` we wrote into `then_var_place`, in `else` we
            // wrote into `else_var_place` — both forks of the same
            // declaration. The phi unifies them. Mirrors upstream lines
            // 326-348 exactly: operand at `then_block_id` is `elseVarPlace`
            // (TS swaps these — see comment "Create phis to reassign the var"
            // in `InlineJsxTransform.ts` line 325 onward; following that
            // exactly here).
            let phi_identifier = fork_temporary_identifier(&mut func.env, &var_place.identifier);
            let mut phi_place = create_temporary_place(&mut func.env, jsx_value_loc);
            phi_place.identifier = phi_identifier.clone();

            let mut phi_operands: rustc_hash::FxHashMap<BlockId, Place> =
                rustc_hash::FxHashMap::default();
            // Upstream sets the operand at `thenBlockId` to a clone of
            // `elseVarPlace` and the one at `elseBlockId` to a clone of
            // `thenVarPlace`. This is the precise swap upstream performs
            // (lines 327-332). Preserve it bit-for-bit.
            phi_operands.insert(then_block_id, clone_place(&else_var_place));
            phi_operands.insert(else_block_id, clone_place(&then_var_place));

            let phi = Phi { id: 0, place: phi_place, operands: phi_operands };

            let fallthrough_block = BasicBlock {
                kind: block_kind,
                id: fallthrough_block_id,
                instructions: after_instructions,
                terminal: block_terminal,
                preds: FxHashSet::default(),
                phis: vec![phi],
            };

            // -- Commit the new shape into the function body. --
            if let Some(working_block) = func.body.blocks.get_mut(&working_block_id) {
                working_block.instructions = before_instructions;
                working_block.terminal = if_terminal;
            }
            func.body.blocks.insert(then_block_id, then_block);
            func.body.blocks.insert(else_block_id, else_block);
            func.body.blocks.insert(fallthrough_block_id, fallthrough_block);

            // Remember this rewrite for Step 2 use-site substitution.
            inlined.insert(
                original_lvalue.identifier.declaration_id,
                InlinedJsxDeclaration {
                    identifier: phi_identifier,
                    block_ids_to_ignore: {
                        let mut s: FxHashSet<BlockId> = FxHashSet::default();
                        s.insert(then_block_id);
                        s.insert(else_block_id);
                        s
                    },
                },
            );

            // Continue processing in the new fallthrough — additional JSX
            // expressions originally in this same block now live in the
            // fallthrough's instruction list.
            working_block_id = fallthrough_block_id;
        }
    }

    if inlined.is_empty() {
        return;
    }

    // -----------------------------------------------------------------------
    // Step 2: rewrite all uses of the JSX result lvalues to use the phi
    // identifier produced in Step 1. Skip the then/else blocks for each
    // rewrite — those still hold the source/literal that becomes the value
    // of the phi.
    // -----------------------------------------------------------------------
    let block_ids_iter: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids_iter {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
        for instr in &mut block.instructions {
            map_instruction_operands(instr, &mut |p: Place| handle_place(p, block_id, &inlined));
            map_instruction_lvalues(instr, &mut |p: Place| handle_lvalue(p, block_id, &inlined));
            map_instruction_value_operands(&mut instr.value, &mut |p: Place| {
                handle_place(p, block_id, &inlined)
            });
        }
        map_terminal_operands(&mut block.terminal, &mut |p: Place| {
            handle_place(p, block_id, &inlined)
        });

        // Upstream additionally rewrites `terminal.scope.dependencies` and
        // `terminal.scope.declarations` when the terminal is a scope
        // terminal. The Rust port runs `InlineJsxTransform` AFTER
        // `BuildReactiveScopeTerminalsHIR` (mirroring upstream's
        // `Pipeline.ts` order: lines 378-425), so scope terminals already
        // reference the OLD JSX lvalue ids. Without the fixup, the reactive
        // function builder + codegen think the cached value is the JSX's
        // original lvalue (e.g. `t_jsx`), but the function actually returns
        // the phi (`t_phi`), producing the divergent
        // `cache[i] = t_jsx; return t_phi` shape.
        match &mut block.terminal {
            Terminal::Scope(t) => {
                rewrite_scope(&mut t.scope, &inlined);
            }
            Terminal::PrunedScope(t) => {
                rewrite_scope(&mut t.scope, &inlined);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Step 3: HIR fixup — RPO, predecessors, instruction ids, scope ranges.
    // -----------------------------------------------------------------------
    reverse_postorder_blocks(&mut func.body);
    mark_predecessors(&mut func.body);
    mark_instruction_ids(&mut func.body);
    fix_scope_and_identifier_ranges(&mut func.body);
}

// =============================================================================
// Helpers
// =============================================================================

fn clone_place(p: &Place) -> Place {
    p.clone()
}

/// Promote a temporary identifier so codegen prints it as `t0`, `t1`, ...
/// instead of inlining the expression. Mirrors upstream `promoteTemporary`
/// (`HIR/HIR.ts`).
fn promote_temporary(place: &mut Place) {
    let decl_id = place.identifier.declaration_id.0;
    place.identifier.name = Some(IdentifierName::Promoted(format!("#t{decl_id}")));
}

/// Fork a `Place` by allocating a fresh `IdentifierId` but keeping the
/// `DeclarationId`. Mirrors upstream `{ ...varPlace, identifier:
/// forkTemporaryIdentifier(...) }`.
fn fork_temporary(env: &mut Environment, source: &Place) -> Place {
    let mut p = source.clone();
    p.identifier = fork_temporary_identifier(env, &source.identifier);
    p
}

/// Fork an `Identifier`: allocate a new `IdentifierId`, copy everything else.
/// `mutable_range` is reset to default per upstream `forkTemporaryIdentifier`.
fn fork_temporary_identifier(env: &mut Environment, source: &Identifier) -> Identifier {
    let id = env.next_identifier_id();
    Identifier {
        id,
        declaration_id: source.declaration_id,
        name: source.name.clone(),
        mutable_range: MutableRange::default(),
        scope: source.scope.clone(),
        type_: source.type_.clone(),
        loc: source.loc,
    }
}

/// Produce `Symbol.for(<symbol_name>)` as four instructions appended to
/// `next_instructions`, returning the `ObjectProperty` that wraps the final
/// call's lvalue. Mirrors upstream `createSymbolProperty`.
fn create_symbol_property(
    env: &mut Environment,
    jsx_instr: &Instruction,
    next_instructions: &mut Vec<Instruction>,
    property_name: &str,
    symbol_name: &str,
) -> ObjectProperty {
    let jsx_loc = jsx_instr.loc;
    let jsx_value_loc = jsx_instr.value.loc();

    // LoadGlobal Symbol
    let mut symbol_place = create_temporary_place(env, jsx_value_loc);
    symbol_place.effect = Effect::Mutate;
    let symbol_instr = Instruction {
        id: InstructionId(0),
        lvalue: symbol_place.clone(),
        value: InstructionValue::LoadGlobal(LoadGlobal {
            binding: NonLocalBinding::Global { name: "Symbol".to_string() },
            loc: jsx_value_loc,
        }),
        effects: None,
        loc: jsx_loc,
    };
    next_instructions.push(symbol_instr);

    // PropertyLoad Symbol.for
    let mut symbol_for_place = create_temporary_place(env, jsx_value_loc);
    symbol_for_place.effect = Effect::Read;
    let symbol_for_instr = Instruction {
        id: InstructionId(0),
        lvalue: symbol_for_place.clone(),
        value: InstructionValue::PropertyLoad(PropertyLoad {
            object: symbol_place.clone(),
            property: PropertyLiteral::String("for".to_string()),
            optional: false,
            loc: jsx_value_loc,
        }),
        effects: None,
        loc: jsx_loc,
    };
    next_instructions.push(symbol_for_instr);

    // Primitive "<symbol_name>"
    let mut symbol_value_place = create_temporary_place(env, jsx_value_loc);
    symbol_value_place.effect = Effect::Mutate;
    let symbol_value_instr = Instruction {
        id: InstructionId(0),
        lvalue: symbol_value_place.clone(),
        value: InstructionValue::Primitive(PrimitiveValue {
            value: PrimitiveValueKind::String(symbol_name.to_string()),
            loc: jsx_value_loc,
        }),
        effects: None,
        loc: jsx_loc,
    };
    next_instructions.push(symbol_value_instr);

    // MethodCall Symbol.for("<symbol_name>")
    let mut method_call_place = create_temporary_place(env, jsx_value_loc);
    method_call_place.effect = Effect::Mutate;
    let method_call_instr = Instruction {
        id: InstructionId(0),
        lvalue: method_call_place.clone(),
        value: InstructionValue::MethodCall(MethodCall {
            receiver: symbol_place,
            property: symbol_for_place,
            args: vec![crate::hir::CallArg::Place(symbol_value_place)],
            loc: jsx_value_loc,
        }),
        effects: None,
        loc: jsx_loc,
    };
    next_instructions.push(method_call_instr);

    let mut prop_place = method_call_place;
    prop_place.effect = Effect::Capture;
    ObjectProperty {
        key: ObjectPropertyKey::String(property_name.to_string()),
        property_type: ObjectPropertyType::Property,
        place: prop_place,
    }
}

/// Produce a `type` property carrying either the built-in tag string or the
/// component identifier place. Mirrors upstream `createTagProperty`.
fn create_tag_property(
    env: &mut Environment,
    jsx_instr: &Instruction,
    next_instructions: &mut Vec<Instruction>,
    tag: &JsxTag,
) -> ObjectProperty {
    let jsx_loc = jsx_instr.loc;
    let jsx_value_loc = jsx_instr.value.loc();
    match tag {
        JsxTag::BuiltIn(builtin) => {
            let mut tag_place = create_temporary_place(env, jsx_value_loc);
            tag_place.effect = Effect::Mutate;
            let tag_instr = Instruction {
                id: InstructionId(0),
                lvalue: tag_place.clone(),
                value: InstructionValue::Primitive(PrimitiveValue {
                    value: PrimitiveValueKind::String(builtin.name.clone()),
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            next_instructions.push(tag_instr);
            let mut prop_place = tag_place;
            prop_place.effect = Effect::Capture;
            ObjectProperty {
                key: ObjectPropertyKey::String("type".to_string()),
                property_type: ObjectPropertyType::Property,
                place: prop_place,
            }
        }
        JsxTag::Place(p) => {
            let mut prop_place = p.clone();
            prop_place.effect = Effect::Capture;
            ObjectProperty {
                key: ObjectPropertyKey::String("type".to_string()),
                property_type: ObjectPropertyType::Property,
                place: prop_place,
            }
        }
    }
}

#[expect(clippy::struct_field_names)]
struct PropsProperties {
    ref_property: ObjectProperty,
    key_property: ObjectProperty,
    props_property: ObjectProperty,
}

/// Build the `ref`, `key`, and `props` object properties from a JSX
/// expression's attributes + children. Mirrors upstream `createPropsProperties`.
fn create_props_properties(
    env: &mut Environment,
    jsx_instr: &Instruction,
    next_instructions: &mut Vec<Instruction>,
    prop_attributes: &[JsxAttribute],
    children: Option<&[Place]>,
) -> PropsProperties {
    let jsx_loc = jsx_instr.loc;
    let jsx_value_loc = jsx_instr.value.loc();

    let mut ref_property: Option<ObjectProperty> = None;
    let mut key_property: Option<ObjectProperty> = None;
    let mut props: Vec<ObjectPatternProperty> = Vec::new();

    // Detect "spread-only" props: a single `{...spread}` and no other
    // attributes (excluding `key`). Upstream emits `props: spread` in that
    // case rather than wrapping in an object literal.
    let mut jsx_attrs_without_key = 0usize;
    let mut spread_count = 0usize;
    let mut spread_only_arg: Option<Place> = None;
    for prop in prop_attributes {
        match prop {
            JsxAttribute::Attribute { name, .. } if name != "key" => jsx_attrs_without_key += 1,
            JsxAttribute::Attribute { .. } => {}
            JsxAttribute::Spread { argument } => {
                spread_count += 1;
                spread_only_arg = Some(argument.clone());
            }
        }
    }
    let spread_props_only = jsx_attrs_without_key == 0 && spread_count == 1;

    for prop in prop_attributes {
        match prop {
            JsxAttribute::Attribute { name, place } => {
                if name == "key" {
                    key_property = Some(ObjectProperty {
                        key: ObjectPropertyKey::String("key".to_string()),
                        property_type: ObjectPropertyType::Property,
                        place: place.clone(),
                    });
                } else if name == "ref" {
                    // ref appears twice: once as the element's `ref` slot and
                    // once inside `props` for backwards compatibility.
                    ref_property = Some(ObjectProperty {
                        key: ObjectPropertyKey::String("ref".to_string()),
                        property_type: ObjectPropertyType::Property,
                        place: place.clone(),
                    });
                    props.push(ObjectPatternProperty::Property(ObjectProperty {
                        key: ObjectPropertyKey::String("ref".to_string()),
                        property_type: ObjectPropertyType::Property,
                        place: place.clone(),
                    }));
                } else {
                    props.push(ObjectPatternProperty::Property(ObjectProperty {
                        key: ObjectPropertyKey::String(name.clone()),
                        property_type: ObjectPropertyType::Property,
                        place: place.clone(),
                    }));
                }
            }
            JsxAttribute::Spread { argument } => {
                props
                    .push(ObjectPatternProperty::Spread(SpreadPattern { place: argument.clone() }));
            }
        }
    }

    // Children: single child becomes `children: <place>`, multiple become
    // `children: [<place>, <place>, ...]` via an emitted ArrayExpression.
    if let Some(children) = children {
        let child_property = if children.len() == 1 {
            let mut child_place = children[0].clone();
            child_place.effect = Effect::Capture;
            ObjectProperty {
                key: ObjectPropertyKey::String("children".to_string()),
                property_type: ObjectPropertyType::Property,
                place: child_place,
            }
        } else {
            let mut array_lvalue = create_temporary_place(env, jsx_value_loc);
            array_lvalue.effect = Effect::Mutate;
            let array_instr = Instruction {
                id: InstructionId(0),
                lvalue: array_lvalue.clone(),
                value: InstructionValue::ArrayExpression(ArrayExpression {
                    elements: children.iter().cloned().map(ArrayExpressionElement::Place).collect(),
                    loc: jsx_value_loc,
                }),
                effects: None,
                loc: jsx_loc,
            };
            next_instructions.push(array_instr);
            let mut prop_place = array_lvalue;
            prop_place.effect = Effect::Capture;
            ObjectProperty {
                key: ObjectPropertyKey::String("children".to_string()),
                property_type: ObjectPropertyType::Property,
                place: prop_place,
            }
        };
        props.push(ObjectPatternProperty::Property(child_property));
    }

    // Fill in `ref` / `key` with primitive `null` when absent.
    let ref_property = ref_property.unwrap_or_else(|| {
        let mut p = create_temporary_place(env, jsx_value_loc);
        p.effect = Effect::Mutate;
        let instr = Instruction {
            id: InstructionId(0),
            lvalue: p.clone(),
            value: InstructionValue::Primitive(PrimitiveValue {
                value: PrimitiveValueKind::Null,
                loc: jsx_value_loc,
            }),
            effects: None,
            loc: jsx_loc,
        };
        next_instructions.push(instr);
        let mut prop_place = p;
        prop_place.effect = Effect::Capture;
        ObjectProperty {
            key: ObjectPropertyKey::String("ref".to_string()),
            property_type: ObjectPropertyType::Property,
            place: prop_place,
        }
    });
    let key_property = key_property.unwrap_or_else(|| {
        let mut p = create_temporary_place(env, jsx_value_loc);
        p.effect = Effect::Mutate;
        let instr = Instruction {
            id: InstructionId(0),
            lvalue: p.clone(),
            value: InstructionValue::Primitive(PrimitiveValue {
                value: PrimitiveValueKind::Null,
                loc: jsx_value_loc,
            }),
            effects: None,
            loc: jsx_loc,
        };
        next_instructions.push(instr);
        let mut prop_place = p;
        prop_place.effect = Effect::Capture;
        ObjectProperty {
            key: ObjectPropertyKey::String("key".to_string()),
            property_type: ObjectPropertyType::Property,
            place: prop_place,
        }
    });

    let props_property = if spread_props_only {
        let mut spread = spread_only_arg.expect("spread_props_only requires a spread argument");
        spread.effect = Effect::Mutate;
        ObjectProperty {
            key: ObjectPropertyKey::String("props".to_string()),
            property_type: ObjectPropertyType::Property,
            place: spread,
        }
    } else {
        let mut props_lvalue = create_temporary_place(env, jsx_value_loc);
        props_lvalue.effect = Effect::Mutate;
        let props_instr = Instruction {
            id: InstructionId(0),
            lvalue: props_lvalue.clone(),
            value: InstructionValue::ObjectExpression(ObjectExpression {
                properties: props,
                loc: jsx_value_loc,
            }),
            effects: None,
            loc: jsx_loc,
        };
        next_instructions.push(props_instr);
        let mut prop_place = props_lvalue;
        prop_place.effect = Effect::Capture;
        ObjectProperty {
            key: ObjectPropertyKey::String("props".to_string()),
            property_type: ObjectPropertyType::Property,
            place: prop_place,
        }
    };

    PropsProperties { ref_property, key_property, props_property }
}

fn handle_place(
    place: Place,
    block_id: BlockId,
    inlined: &rustc_hash::FxHashMap<DeclarationId, InlinedJsxDeclaration>,
) -> Place {
    let Some(entry) = inlined.get(&place.identifier.declaration_id) else {
        return place;
    };
    if entry.block_ids_to_ignore.contains(&block_id) {
        return place;
    }
    let mut next = place;
    next.identifier = entry.identifier.clone();
    next
}

fn handle_lvalue(
    place: Place,
    block_id: BlockId,
    inlined: &rustc_hash::FxHashMap<DeclarationId, InlinedJsxDeclaration>,
) -> Place {
    handle_place(place, block_id, inlined)
}

/// Look up an identifier in the inlined map (no block-ignore semantics —
/// scope structures are global to a block, not nested within then/else).
fn handle_identifier(
    identifier: &Identifier,
    inlined: &rustc_hash::FxHashMap<DeclarationId, InlinedJsxDeclaration>,
) -> Option<Identifier> {
    inlined.get(&identifier.declaration_id).map(|entry| entry.identifier.clone())
}

/// Rewrite a `ReactiveScope` to replace any inlined JSX identifiers with
/// their phi-merged equivalents. Mirrors upstream lines 395-417.
///
/// `scope.dependencies` are updated in place; entries keep their `path` /
/// `reactive` flags but their `identifier` field is replaced. Duplicate
/// identifiers after rewriting are deduplicated implicitly by the hash set.
///
/// `scope.declarations` are rekeyed: when an entry's identifier maps to a
/// phi, the entry is removed and re-inserted under the phi identifier's
/// `IdentifierId`. The order of insertion is preserved as best-effort by
/// re-using the underlying `IndexMap`'s `shift_remove` semantics, mirroring
/// the upstream `Map.delete` + `Map.set` sequence.
fn rewrite_scope(
    scope: &mut ReactiveScope,
    inlined: &rustc_hash::FxHashMap<DeclarationId, InlinedJsxDeclaration>,
) {
    // Dependencies: clone, mutate, re-insert. `FxHashSet<ReactiveScopeDependency>`
    // hashes on `(identifier.id, reactive, path)`, so changing the identifier
    // necessitates a re-insertion.
    let dependencies: Vec<_> = std::mem::take(&mut scope.dependencies).into_iter().collect();
    for mut dep in dependencies {
        if let Some(new_ident) = handle_identifier(&dep.identifier, inlined) {
            dep.identifier = new_ident;
        }
        scope.dependencies.insert(dep);
    }

    // Declarations: rekey when the identifier changes. Upstream does
    // `delete(origId); set(decl.identifier.id, {...})`. Since the IndexMap
    // preserves insertion order, we drain into a Vec, then re-insert.
    let entries: Vec<(IdentifierId, crate::hir::ReactiveScopeDeclaration)> =
        std::mem::take(&mut scope.declarations).into_iter().collect();
    for (orig_id, mut decl) in entries {
        if let Some(new_ident) = handle_identifier(&decl.identifier, inlined) {
            let new_id = new_ident.id;
            decl.identifier = new_ident;
            scope.declarations.insert(new_id, decl);
        } else {
            scope.declarations.insert(orig_id, decl);
        }
    }
}
