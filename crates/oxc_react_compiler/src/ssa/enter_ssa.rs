/// SSA conversion pass.
///
/// Port of `SSA/EnterSSA.ts` from the React Compiler.
///
/// Converts the HIR into Static Single Assignment (SSA) form using the
/// algorithm from Braun et al. "Simple and Efficient Construction of
/// Static Single Assignment Form" (2013).
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{
        BasicBlock, BlockId, HIRFunction, Identifier, IdentifierId, InstructionId,
        InstructionValue, LoweredFunction, MutableRange, Phi, Place, ReactiveParam, SpreadPattern,
        environment::Environment,
        hir_builder::each_terminal_successor,
        types::make_type,
        visitors::{map_instruction_lvalues, map_instruction_operands, map_terminal_operands},
    },
};

struct IncompletePhi {
    old_place: Place,
    new_place: Place,
}

struct State {
    defs: FxHashMap<IdentifierId, Identifier>,
    incomplete_phis: Vec<IncompletePhi>,
}

struct SsaBuilder {
    states: FxHashMap<BlockId, State>,
    current_block_id: Option<BlockId>,
    unsealed_preds: FxHashMap<BlockId, usize>,
    blocks: FxHashMap<BlockId, BasicBlock>,
    env: Environment,
    unknown: FxHashSet<IdentifierId>,
    context: FxHashSet<IdentifierId>,
}

impl SsaBuilder {
    fn new(env: Environment, blocks: &FxHashMap<BlockId, BasicBlock>) -> Self {
        Self {
            states: FxHashMap::default(),
            current_block_id: None,
            unsealed_preds: FxHashMap::default(),
            blocks: blocks.clone(),
            env,
            unknown: FxHashSet::default(),
            context: FxHashSet::default(),
        }
    }

    fn make_id(&mut self, old_id: &Identifier) -> Identifier {
        let id = self.env.next_identifier_id();
        Identifier {
            id,
            declaration_id: old_id.declaration_id,
            name: old_id.name.clone(),
            mutable_range: MutableRange { start: InstructionId(0), end: InstructionId(0) },
            scope: None,
            type_: make_type(),
            loc: old_id.loc,
        }
    }

    fn register_blocks_from(&mut self, func: &HIRFunction) {
        for (id, block) in &func.body.blocks {
            self.blocks.insert(*id, block.clone());
        }
    }

    /// Save/restore current_block_id around a closure, matching TS `enter()`.
    fn enter(&mut self, f: impl FnOnce(&mut Self)) {
        let saved_block = self.current_block_id;
        f(self);
        self.current_block_id = saved_block;
    }

    fn state_mut(&mut self) -> &mut State {
        let block_id = self.current_block_id.expect("must be in a block to access state");
        self.states.get_mut(&block_id).expect("state must exist for current block")
    }

    fn define_context(&mut self, old_place: Place) -> Place {
        let old_id = old_place.identifier.id;
        let new_place = self.define_place(old_place);
        self.context.insert(old_id);
        new_place
    }

    fn define_place(&mut self, old_place: Place) -> Place {
        let old_id = old_place.identifier.id;

        if self.unknown.contains(&old_id) {
            // In TS this throws a Todo error; we return the original place
            // to avoid panicking in the port
            return old_place;
        }

        // Do not redefine context references
        if self.context.contains(&old_id) {
            return self.get_place(&old_place);
        }

        let new_id = self.make_id(&old_place.identifier);
        self.state_mut().defs.insert(old_id, new_id.clone());
        Place { identifier: new_id, ..old_place }
    }

    fn get_place(&mut self, old_place: &Place) -> Place {
        let block_id = self.current_block_id.expect("must be in a block");
        let new_id = self.get_id_at(old_place, block_id);
        Place {
            identifier: new_id,
            effect: old_place.effect,
            reactive: old_place.reactive,
            loc: old_place.loc,
        }
    }

    fn get_id_at(&mut self, old_place: &Place, block_id: BlockId) -> Identifier {
        let old_id = old_place.identifier.id;

        // Check if Place is defined locally
        if let Some(state) = self.states.get(&block_id)
            && let Some(id) = state.defs.get(&old_id)
        {
            return id.clone();
        }

        let Some(block) = self.blocks.get(&block_id).cloned() else {
            return old_place.identifier.clone();
        };

        if block.preds.is_empty() {
            // Entry block, definition not found — assume global
            self.unknown.insert(old_id);
            return old_place.identifier.clone();
        }

        let unsealed = self.unsealed_preds.get(&block_id).copied().unwrap_or(0);
        if unsealed > 0 {
            // Haven't visited all predecessors; place an incomplete phi
            let new_id = self.make_id(&old_place.identifier);
            let new_place = Place { identifier: new_id.clone(), ..old_place.clone() };
            let state = self.states.get_mut(&block_id).expect("state must exist");
            state.incomplete_phis.push(IncompletePhi { old_place: old_place.clone(), new_place });
            state.defs.insert(old_id, new_id.clone());
            return new_id;
        }

        // Only one predecessor
        if block.preds.len() == 1 {
            let pred = *block.preds.iter().next().expect("preds is non-empty");
            let new_id = self.get_id_at(old_place, pred);
            if let Some(state) = self.states.get_mut(&block_id) {
                state.defs.insert(old_id, new_id.clone());
            }
            return new_id;
        }

        // Multiple predecessors — may need a phi
        let new_id = self.make_id(&old_place.identifier);
        // Update defs before adding phi to terminate recursion for loops
        if let Some(state) = self.states.get_mut(&block_id) {
            state.defs.insert(old_id, new_id.clone());
        }
        let new_place = Place { identifier: new_id, ..old_place.clone() };
        self.add_phi(&block, old_place, &new_place)
    }

    fn add_phi(&mut self, block: &BasicBlock, old_place: &Place, new_place: &Place) -> Identifier {
        let mut pred_defs: FxHashMap<BlockId, Place> = FxHashMap::default();
        for &pred_block_id in &block.preds {
            let pred_id = self.get_id_at(old_place, pred_block_id);
            pred_defs.insert(pred_block_id, Place { identifier: pred_id, ..old_place.clone() });
        }

        let phi =
            Phi { id: new_place.identifier.id.0, place: new_place.clone(), operands: pred_defs };

        // Insert the phi into the actual block in our blocks map
        if let Some(actual_block) = self.blocks.get_mut(&block.id) {
            actual_block.phis.push(phi);
        }

        new_place.identifier.clone()
    }

    fn fix_incomplete_phis(&mut self, block_id: BlockId) {
        let incomplete_phis: Vec<IncompletePhi> = {
            let state = self.states.get_mut(&block_id).expect("state must exist");
            std::mem::take(&mut state.incomplete_phis)
        };
        let block = self.blocks.get(&block_id).cloned();
        if let Some(block) = block {
            for phi in &incomplete_phis {
                self.add_phi(&block, &phi.old_place, &phi.new_place);
            }
        }
    }

    fn start_block(&mut self, block_id: BlockId) {
        self.current_block_id = Some(block_id);
        self.states
            .insert(block_id, State { defs: FxHashMap::default(), incomplete_phis: Vec::new() });
    }
}

/// Convert a function's HIR to SSA form.
///
/// # Errors
/// Returns a `CompilerError` if the function has invalid structure.
pub fn enter_ssa(func: &mut HIRFunction, env: &Environment) -> Result<(), CompilerError> {
    let entry = func.body.entry;
    let mut builder = SsaBuilder::new(env.clone(), &func.body.blocks);
    enter_ssa_impl(func, &mut builder, entry)?;

    // Write back the blocks from the builder
    func.body.blocks = builder.blocks;
    Ok(())
}

fn enter_ssa_impl(
    func: &mut HIRFunction,
    builder: &mut SsaBuilder,
    root_entry: BlockId,
) -> Result<(), CompilerError> {
    let mut visited_blocks: FxHashSet<BlockId> = FxHashSet::default();
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    for block_id in block_ids {
        if !visited_blocks.insert(block_id) {
            return Err(CompilerError::invariant(
                &format!("found a cycle! visiting bb{} again", block_id.0),
                None,
                GENERATED_SOURCE,
            ));
        }

        builder.start_block(block_id);

        if block_id == root_entry {
            // func.context should be empty for the root function
            if !func.context.is_empty() {
                return Err(CompilerError::invariant(
                    "Expected function context to be empty for outer function declarations",
                    None,
                    func.loc,
                ));
            }

            func.params = func
                .params
                .iter()
                .map(|param| match param {
                    ReactiveParam::Place(p) => {
                        ReactiveParam::Place(builder.define_place(p.clone()))
                    }
                    ReactiveParam::Spread(s) => ReactiveParam::Spread(SpreadPattern {
                        place: builder.define_place(s.place.clone()),
                    }),
                })
                .collect();
        }

        // Process instructions in this block
        let block = builder.blocks.get(&block_id).cloned();
        if let Some(mut block) = block {
            for instr in &mut block.instructions {
                map_instruction_operands(instr, &mut |place| builder.get_place(&place));
                map_instruction_lvalues(instr, &mut |place| builder.define_place(place));

                // Handle nested function expressions: take the lowered function
                // out of the instruction, perform recursive SSA conversion, then
                // put it back. This mirrors the TS EnterSSA lines 283-310.
                let nested_func = take_lowered_function(&mut instr.value);
                if let Some(mut lowered_func) = nested_func {
                    let entry_id = lowered_func.func.body.entry;

                    // The entry block should have zero predecessors; temporarily
                    // add the current block_id so SSA lookups can traverse into
                    // the outer scope.
                    if let Some(entry_block) = lowered_func.func.body.blocks.get_mut(&entry_id) {
                        entry_block.preds.insert(block_id);
                    }

                    // Register nested function's blocks in the builder
                    // (TS: builder.defineFunction)
                    builder.register_blocks_from(&lowered_func.func);

                    // Enter a new scope: define context, params, and recurse
                    builder.enter(|builder| {
                        // Define context variables (captured from outer scope)
                        lowered_func.func.context = lowered_func
                            .func
                            .context
                            .iter()
                            .map(|place| builder.define_context(place.clone()))
                            .collect();

                        lowered_func.func.params = lowered_func
                            .func
                            .params
                            .iter()
                            .map(|param| match param {
                                ReactiveParam::Place(p) => {
                                    ReactiveParam::Place(builder.define_place(p.clone()))
                                }
                                ReactiveParam::Spread(s) => ReactiveParam::Spread(SpreadPattern {
                                    place: builder.define_place(s.place.clone()),
                                }),
                            })
                            .collect();

                        // Recursively apply SSA to the nested function.
                        let _ = enter_ssa_impl(&mut lowered_func.func, builder, root_entry);
                    });

                    // Remove the artificial predecessor
                    if let Some(entry_block) = lowered_func.func.body.blocks.get_mut(&entry_id) {
                        entry_block.preds.remove(&block_id);
                    }

                    // Also update the builder's copy of the entry block
                    if let Some(builder_entry) = builder.blocks.get_mut(&entry_id) {
                        builder_entry.preds.remove(&block_id);
                    }

                    // Put the lowered function back into the instruction
                    put_lowered_function(&mut instr.value, lowered_func);
                }
            }

            // Map terminal operands
            map_terminal_operands(&mut block.terminal, &mut |place| builder.get_place(&place));

            // Process successors
            let successors = each_terminal_successor(&block.terminal);
            for output_id in &successors {
                let output_preds_size = builder.blocks.get(output_id).map_or(0, |b| b.preds.len());

                let count = if let Some(&existing) = builder.unsealed_preds.get(output_id) {
                    existing.saturating_sub(1)
                } else {
                    output_preds_size.saturating_sub(1)
                };
                builder.unsealed_preds.insert(*output_id, count);

                if count == 0 && visited_blocks.contains(output_id) {
                    builder.fix_incomplete_phis(*output_id);
                }
            }

            // Write back the modified block
            builder.blocks.insert(block_id, block);
        }
    }

    Ok(())
}

/// Create a minimal placeholder `LoweredFunction` for use in take/replace patterns.
fn placeholder_lowered_function() -> LoweredFunction {
    LoweredFunction {
        func: Box::new(HIRFunction {
            loc: GENERATED_SOURCE,
            id: None,
            name_hint: None,
            fn_type: crate::hir::ReactFunctionType::Other,
            env: Environment::new(
                crate::hir::ReactFunctionType::Other,
                crate::hir::environment::CompilerOutputMode::Client,
                crate::hir::environment::EnvironmentConfig::default(),
            ),
            params: Vec::new(),
            returns: Place {
                identifier: Identifier {
                    id: IdentifierId(0),
                    declaration_id: crate::hir::DeclarationId(0),
                    name: None,
                    mutable_range: MutableRange::default(),
                    scope: None,
                    type_: make_type(),
                    loc: GENERATED_SOURCE,
                },
                effect: crate::hir::Effect::Unknown,
                reactive: false,
                loc: GENERATED_SOURCE,
            },
            context: Vec::new(),
            body: crate::hir::Hir { entry: BlockId(0), blocks: FxHashMap::default() },
            generator: false,
            is_async: false,
            directives: Vec::new(),
            aliasing_effects: None,
        }),
    }
}

/// Extract the `LoweredFunction` from a `FunctionExpression` or `ObjectMethod`
/// instruction value, replacing it with a temporary placeholder. Returns `None`
/// if the instruction is not a function expression or object method.
fn take_lowered_function(value: &mut InstructionValue) -> Option<LoweredFunction> {
    match value {
        InstructionValue::FunctionExpression(v) => {
            Some(std::mem::replace(&mut v.lowered_func, placeholder_lowered_function()))
        }
        InstructionValue::ObjectMethod(v) => {
            Some(std::mem::replace(&mut v.lowered_func, placeholder_lowered_function()))
        }
        _ => None,
    }
}

/// Put a `LoweredFunction` back into a `FunctionExpression` or `ObjectMethod`.
fn put_lowered_function(value: &mut InstructionValue, lowered_func: LoweredFunction) {
    match value {
        InstructionValue::FunctionExpression(v) => {
            v.lowered_func = lowered_func;
        }
        InstructionValue::ObjectMethod(v) => {
            v.lowered_func = lowered_func;
        }
        _ => {}
    }
}
