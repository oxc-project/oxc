/// Collect optional chain dependencies.
///
/// Port of `HIR/CollectOptionalChainDependencies.ts` from the React Compiler.
///
/// Analyzes optional chains (e.g., `a?.b?.c`) to determine which parts of the
/// chain are dependencies for reactive scope tracking. Optional chains are
/// lowered into conditional blocks in the HIR, and this pass reconstructs
/// the dependency path from those blocks.
use rustc_hash::{FxHashMap, FxHashSet};

use super::hir_types::{
    DependencyPathEntry, HIRFunction, IdentifierId, InstructionId, InstructionValue,
    ReactiveScopeDependency, Terminal,
};

/// Result of optional chain dependency collection.
pub struct OptionalChainDependencies {
    /// Map from the result identifier of an optional chain to the full dependency path.
    /// Equivalent to TS `temporariesReadInOptional`.
    pub dependencies: FxHashMap<IdentifierId, ReactiveScopeDependency>,

    /// Set of instruction/terminal IDs already processed during optional chain analysis.
    /// These should be skipped by `collect_dependencies` to avoid duplicate dependencies.
    /// Equivalent to TS `processedInstrsInOptional`.
    pub processed_instrs_in_optional: FxHashSet<InstructionId>,
}

/// Collect optional chain dependencies from the function.
pub fn collect_optional_chain_dependencies(func: &HIRFunction) -> OptionalChainDependencies {
    let mut dependencies = FxHashMap::default();
    let mut processed_instrs = FxHashSet::default();

    // Find optional terminals in the CFG
    for block in func.body.blocks.values() {
        if let Terminal::Optional(opt) = &block.terminal {
            // An optional terminal represents `a?.b` — the test block checks
            // if the value is nullish, and either continues the chain or
            // short-circuits to the fallthrough.

            // Collect the property path from the test block
            if let Some(test_block) = func.body.blocks.get(&opt.test) {
                for instr in &test_block.instructions {
                    if let InstructionValue::PropertyLoad(v) = &instr.value {
                        // This is part of the optional chain path
                        let dep = ReactiveScopeDependency {
                            identifier: v.object.identifier.clone(),
                            reactive: false,
                            path: vec![DependencyPathEntry {
                                property: v.property.clone(),
                                optional: opt.optional,
                                loc: v.loc,
                            }],
                            loc: v.loc,
                        };
                        dependencies.insert(instr.lvalue.identifier.id, dep);
                    }
                }

                // Mark the test block's branch terminal as processed so its operand
                // (the base of the optional chain) is not visited again by collect_dependencies
                processed_instrs.insert(test_block.terminal.id());

                // Find and mark the StoreLocal instruction in the consequent block.
                // The consequent block pattern is [PropertyLoad, StoreLocal].
                // The StoreLocal copies the PropertyLoad result into the chain's output temp.
                if let Terminal::Branch(branch) = &test_block.terminal {
                    if let Some(consequent_block) = func.body.blocks.get(&branch.consequent) {
                        for instr in &consequent_block.instructions {
                            if let InstructionValue::StoreLocal(_) = &instr.value {
                                processed_instrs.insert(instr.id);
                            }
                        }
                    }
                }
            }
        }
    }

    OptionalChainDependencies { dependencies, processed_instrs_in_optional: processed_instrs }
}
