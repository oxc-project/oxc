/// Collect optional chain dependencies.
///
/// Port of `HIR/CollectOptionalChainDependencies.ts` from the React Compiler.
///
/// Analyzes optional chains (e.g., `a?.b?.c`) to determine which parts of the
/// chain are dependencies for reactive scope tracking. Optional chains are
/// lowered into conditional blocks in the HIR, and this pass reconstructs
/// the dependency path from those blocks.
use rustc_hash::FxHashMap;

use super::hir_types::{
    HIRFunction, IdentifierId, InstructionValue, Terminal,
    DependencyPathEntry, ReactiveScopeDependency,
};

/// Result of optional chain dependency collection.
pub struct OptionalChainDependencies {
    /// Map from the result identifier of an optional chain to the full dependency path.
    pub dependencies: FxHashMap<IdentifierId, ReactiveScopeDependency>,
}

/// Collect optional chain dependencies from the function.
pub fn collect_optional_chain_dependencies(func: &HIRFunction) -> OptionalChainDependencies {
    let mut dependencies = FxHashMap::default();

    // Find optional terminals in the CFG
    for block in func.body.blocks.values() {
        if let Terminal::Optional(opt) = &block.terminal {
            // An optional terminal represents `a?.b` — the test block checks
            // if the value is nullish, and either continues the chain or
            // short-circuits to the fallthrough.

            // Collect the property path from the test block
            if let Some(test_block) = func.body.blocks.get(&opt.test) {
                for instr in &test_block.instructions {
                    match &instr.value {
                        InstructionValue::PropertyLoad(v) => {
                            // This is part of the optional chain path
                            let dep = ReactiveScopeDependency {
                                identifier_id: v.object.identifier.id,
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
                        _ => {}
                    }
                }
            }
        }
    }

    OptionalChainDependencies { dependencies }
}
