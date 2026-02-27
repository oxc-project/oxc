/// Collect optional chain dependencies.
///
/// Port of `HIR/CollectOptionalChainDependencies.ts` from the React Compiler.
///
/// Analyzes optional chains (e.g., `a?.b?.c`) to determine which parts of the
/// chain are dependencies for reactive scope tracking. Optional chains are
/// lowered into conditional blocks in the HIR, and this pass reconstructs
/// the dependency path from those blocks.
///
/// The key function is `traverse_optional_block` which recursively descends into
/// nested optional block sequences to build cumulative dependency paths.
use rustc_hash::{FxHashMap, FxHashSet};

use super::hir_types::{
    BasicBlock, BlockId, BranchTerminal, DependencyPathEntry, GotoVariant, HIRFunction,
    IdentifierId, InstructionId, InstructionValue, OptionalTerminal, ReactiveScopeDependency,
    Terminal,
};
use crate::hir::types::PropertyLiteral;

/// Result of optional chain dependency collection.
///
/// Port of `OptionalChainSidemap` from the TS reference.
pub struct OptionalChainSidemap {
    /// Stores the correct property mapping (e.g. `a?.b` instead of `a.b`) for
    /// dependency calculation.
    /// Equivalent to TS `temporariesReadInOptional`.
    pub temporaries_read_in_optional: FxHashMap<IdentifierId, ReactiveScopeDependency>,

    /// Records instructions (PropertyLoads, StoreLocals, and test terminals)
    /// processed in this pass. When extracting dependencies in
    /// PropagateScopeDependencies, these instructions are skipped.
    /// Equivalent to TS `processedInstrsInOptional`.
    pub processed_instrs_in_optional: FxHashSet<InstructionId>,

    /// Records optional chains for which we can safely evaluate non-optional
    /// PropertyLoads. e.g. given `a?.b.c`, we can evaluate any load from `a?.b`
    /// at the optional terminal.
    /// Equivalent to TS `hoistableObjects`.
    pub hoistable_objects: FxHashMap<BlockId, ReactiveScopeDependency>,
}

/// Block map type alias.
type BlockMap = indexmap::IndexMap<BlockId, BasicBlock, rustc_hash::FxBuildHasher>;

/// Internal traversal context, matching TS `OptionalTraversalContext`.
struct OptionalTraversalContext {
    /// Track optional blocks to avoid outer calls into nested optionals.
    seen_optionals: FxHashSet<BlockId>,

    processed_instrs_in_optional: FxHashSet<InstructionId>,
    temporaries_read_in_optional: FxHashMap<IdentifierId, ReactiveScopeDependency>,
    hoistable_objects: FxHashMap<BlockId, ReactiveScopeDependency>,
}

/// Collect optional chain sidemap from the function.
///
/// Port of `collectOptionalChainSidemap` from the TS reference.
pub fn collect_optional_chain_sidemap(func: &HIRFunction) -> OptionalChainSidemap {
    let mut context = OptionalTraversalContext {
        seen_optionals: FxHashSet::default(),
        processed_instrs_in_optional: FxHashSet::default(),
        temporaries_read_in_optional: FxHashMap::default(),
        hoistable_objects: FxHashMap::default(),
    };
    traverse_function(func, &mut context);
    OptionalChainSidemap {
        temporaries_read_in_optional: context.temporaries_read_in_optional,
        processed_instrs_in_optional: context.processed_instrs_in_optional,
        hoistable_objects: context.hoistable_objects,
    }
}

/// Traverse a function and all its inner functions to collect optional chain
/// dependencies.
///
/// Port of `traverseFunction` from the TS reference.
fn traverse_function(func: &HIRFunction, context: &mut OptionalTraversalContext) {
    let blocks = &func.body.blocks;
    for block in blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    // Save and restore seen_optionals around inner function traversal.
                    // BlockIds can overlap between inner and outer functions in the Rust
                    // implementation (unlike the TS reference which uses a shared
                    // Environment counter), so inner function processing must not pollute
                    // the outer function's seen set.
                    let saved = std::mem::take(&mut context.seen_optionals);
                    traverse_function(&v.lowered_func.func, context);
                    context.seen_optionals = saved;
                }
                InstructionValue::ObjectMethod(v) => {
                    let saved = std::mem::take(&mut context.seen_optionals);
                    traverse_function(&v.lowered_func.func, context);
                    context.seen_optionals = saved;
                }
                _ => {}
            }
        }
        if let Terminal::Optional(opt) = &block.terminal {
            if !context.seen_optionals.contains(&block.id) {
                traverse_optional_block(block, opt, context, None, blocks);
            }
        }
    }
}

/// Result of matching an optional test block's consequent.
struct MatchConsequentResult {
    consequent_id: IdentifierId,
    property: PropertyLiteral,
    property_id: IdentifierId,
    store_local_instr_id: InstructionId,
    consequent_goto: BlockId,
    property_load_loc: crate::compiler_error::SourceLocation,
}

/// Match the consequent and alternate blocks of an optional's branch terminal.
///
/// Returns the property load computed by the consequent block, or None if the
/// consequent block is not a simple PropertyLoad + StoreLocal pattern.
///
/// Port of `matchOptionalTestBlock` from the TS reference.
fn match_optional_test_block(
    terminal: &BranchTerminal,
    blocks: &BlockMap,
) -> Option<MatchConsequentResult> {
    let consequent_block = blocks.get(&terminal.consequent)?;

    if consequent_block.instructions.len() == 2
        && matches!(consequent_block.instructions[0].value, InstructionValue::PropertyLoad(_))
        && matches!(consequent_block.instructions[1].value, InstructionValue::StoreLocal(_))
    {
        let property_load = match &consequent_block.instructions[0].value {
            InstructionValue::PropertyLoad(v) => v,
            _ => return None,
        };
        let store_local = match &consequent_block.instructions[1].value {
            InstructionValue::StoreLocal(v) => v,
            _ => return None,
        };
        let store_local_instr = &consequent_block.instructions[1];

        // Invariant: PropertyLoad object matches the branch test
        if property_load.object.identifier.id != terminal.test.identifier.id {
            // In TS this is a CompilerError.invariant, but we return None for safety
            return None;
        }

        // Invariant: StoreLocal value matches PropertyLoad lvalue
        if store_local.value.identifier.id != consequent_block.instructions[0].lvalue.identifier.id
        {
            return None;
        }

        // Check terminal is goto with Break variant
        let goto_block = match &consequent_block.terminal {
            Terminal::Goto(g) if g.variant == GotoVariant::Break => g.block,
            _ => return None,
        };

        // Validate alternate block structure: [Primitive, StoreLocal]
        let alternate = blocks.get(&terminal.alternate)?;
        if !(alternate.instructions.len() == 2
            && matches!(alternate.instructions[0].value, InstructionValue::Primitive(_))
            && matches!(alternate.instructions[1].value, InstructionValue::StoreLocal(_)))
        {
            // In TS this is a CompilerError.invariant
            return None;
        }

        Some(MatchConsequentResult {
            consequent_id: store_local.lvalue.place.identifier.id,
            property: property_load.property.clone(),
            property_id: consequent_block.instructions[0].lvalue.identifier.id,
            store_local_instr_id: store_local_instr.id,
            consequent_goto: goto_block,
            property_load_loc: property_load.loc,
        })
    } else {
        None
    }
}

/// Traverse into the optional block and all transitively referenced blocks to
/// collect sidemaps of optional chain dependencies.
///
/// Returns the IdentifierId representing the optional block if the block and
/// all transitively referenced optional blocks precisely represent a chain of
/// property loads. If any part of the optional chain is not hoistable, returns
/// None.
///
/// Port of `traverseOptionalBlock` from the TS reference.
fn traverse_optional_block(
    optional_block: &BasicBlock,
    optional: &OptionalTerminal,
    context: &mut OptionalTraversalContext,
    outer_alternate: Option<BlockId>,
    blocks: &BlockMap,
) -> Option<IdentifierId> {
    context.seen_optionals.insert(optional_block.id);

    let maybe_test = blocks.get(&optional.test)?;
    let test: &BranchTerminal;
    let base_object: ReactiveScopeDependency;

    match &maybe_test.terminal {
        Terminal::Branch(branch) => {
            // Base case: the test block terminates with a Branch.
            // This means we're at the root of an optional chain.
            //
            // Note: The TS reference has an invariant that optional.terminal.optional
            // must be true here, because in the TS HIR, non-optional accesses in an
            // optional chain are just instructions in a test block (not separate Optional
            // terminals). In the Rust HIR, however, every property access in an optional
            // chain (including non-optional ones like `.post` in `a?.b.post`) is wrapped
            // in its own Optional terminal with optional=false. So the base case can be
            // reached through an optional=false terminal, and we must not bail out.

            // Only match base expressions that are straightforward PropertyLoad chains.
            // Accept both LoadLocal and LoadContext as the base instruction. In the TS
            // reference, captured variables use LoadLocal after IIFE inlining, but in
            // Rust they may remain as LoadContext due to the broader context_identifiers
            // propagation in build_hir. Both are semantically equivalent for dependency
            // tracking.
            if maybe_test.instructions.is_empty()
                || !matches!(
                    maybe_test.instructions[0].value,
                    InstructionValue::LoadLocal(_) | InstructionValue::LoadContext(_)
                )
            {
                return None;
            }

            let mut path: Vec<DependencyPathEntry> = Vec::new();
            for i in 1..maybe_test.instructions.len() {
                let instr_val = &maybe_test.instructions[i].value;
                let prev_instr = &maybe_test.instructions[i - 1];
                if let InstructionValue::PropertyLoad(prop_load) = instr_val {
                    if prop_load.object.identifier.id == prev_instr.lvalue.identifier.id {
                        path.push(DependencyPathEntry {
                            property: prop_load.property.clone(),
                            optional: false,
                            loc: prop_load.loc,
                        });
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }

            // Invariant: terminal test matches last instruction's lvalue
            let last_instr = maybe_test.instructions.last()?;
            if branch.test.identifier.id != last_instr.lvalue.identifier.id {
                // TS: CompilerError.invariant
                return None;
            }

            // Extract the place from the base instruction (LoadLocal or LoadContext).
            let base_place = match &maybe_test.instructions[0].value {
                InstructionValue::LoadLocal(v) => &v.place,
                InstructionValue::LoadContext(v) => &v.place,
                _ => return None,
            };

            base_object = ReactiveScopeDependency {
                identifier: base_place.identifier.clone(),
                reactive: base_place.reactive,
                path,
                loc: base_place.loc,
            };
            test = branch;
        }
        Terminal::Optional(inner_opt) => {
            // This is either:
            // - <inner_optional>?.property (optional=true)
            // - <inner_optional>.property  (optional=false)
            // - <inner_optional> <other operation>
            // - an optional base block with a separate nested optional-chain

            let test_block = blocks.get(&inner_opt.fallthrough)?;
            let inner_branch = match &test_block.terminal {
                Terminal::Branch(b) => b,
                _ => {
                    // TS: CompilerError.throwTodo
                    // Fallthrough of the inner optional should be a branch block
                    return None;
                }
            };

            // Recurse into inner optional blocks to collect inner optional-chain
            // expressions, regardless of whether we can match the outer one to a
            // PropertyLoad.
            let inner_optional = traverse_optional_block(
                maybe_test,
                inner_opt,
                context,
                Some(inner_branch.alternate),
                blocks,
            )?;

            // Check that the inner optional is part of the same optional-chain as
            // the outer one.
            if inner_branch.test.identifier.id != inner_optional {
                return None;
            }

            if !optional.optional {
                // If this is a non-optional load participating in an optional chain
                // (e.g. loading the `c` property in `a?.b.c`), record that PropertyLoads
                // from the inner optional value are hoistable.
                if let Some(dep) = context.temporaries_read_in_optional.get(&inner_optional) {
                    context.hoistable_objects.insert(optional_block.id, dep.clone());
                }
            }

            base_object = context.temporaries_read_in_optional.get(&inner_optional)?.clone();
            test = inner_branch;
        }
        _ => {
            return None;
        }
    }

    if outer_alternate.is_some_and(|alt| test.alternate == alt) {
        if !optional_block.instructions.is_empty() {
            // TS: CompilerError.invariant(optional.instructions.length === 0, ...)
            // Unexpected instructions in an inner optional block
            return None;
        }
    }

    let match_consequent_result = match_optional_test_block(test, blocks)?;

    // Invariant: consequent goto matches optional fallthrough
    if match_consequent_result.consequent_goto != optional.fallthrough {
        // TS: CompilerError.invariant
        return None;
    }

    let load = ReactiveScopeDependency {
        identifier: base_object.identifier.clone(),
        reactive: base_object.reactive,
        path: {
            let mut path = base_object.path.clone();
            path.push(DependencyPathEntry {
                property: match_consequent_result.property,
                optional: optional.optional,
                loc: match_consequent_result.property_load_loc,
            });
            path
        },
        loc: match_consequent_result.property_load_loc,
    };

    context.processed_instrs_in_optional.insert(match_consequent_result.store_local_instr_id);
    context.processed_instrs_in_optional.insert(test.id);
    context
        .temporaries_read_in_optional
        .insert(match_consequent_result.consequent_id, load.clone());
    context.temporaries_read_in_optional.insert(match_consequent_result.property_id, load);

    Some(match_consequent_result.consequent_id)
}
