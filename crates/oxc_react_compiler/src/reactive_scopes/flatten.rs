/// Flatten reactive scopes in loops and hooks.
///
/// Ports of:
/// - `ReactiveScopes/FlattenReactiveLoopsHIR.ts`
/// - `ReactiveScopes/FlattenScopesWithHooksOrUseHIR.ts`
///
/// These passes flatten (remove) reactive scopes that cannot be correctly
/// memoized due to being inside loops or containing hook calls.
use rustc_hash::FxHashSet;

use crate::hir::{
    BlockId, HIRFunction, IdentifierId, InstructionValue, LabelTerminal, PrunedScopeTerminal,
    Terminal,
};
use crate::utils::hook_declaration::is_hook_name;

/// Flatten reactive loops -- removes reactive scopes inside loops.
///
/// Loops may execute their body multiple times, so reactive scopes inside loops
/// cannot be correctly memoized with a single cache slot. Any `Scope` terminal
/// found while a loop is active is converted to a `PrunedScope` terminal.
pub fn flatten_reactive_loops_hir(func: &mut HIRFunction) {
    let mut block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    block_ids.sort();

    let mut active_loops: Vec<BlockId> = Vec::new();

    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        // Remove any loop entries whose fallthrough matches the current block
        active_loops.retain(|id| *id != block.id);

        match &block.terminal {
            Terminal::DoWhile(t) => {
                active_loops.push(t.fallthrough);
            }
            Terminal::For(t) => {
                active_loops.push(t.fallthrough);
            }
            Terminal::ForOf(t) => {
                active_loops.push(t.fallthrough);
            }
            Terminal::ForIn(t) => {
                active_loops.push(t.fallthrough);
            }
            Terminal::While(t) => {
                active_loops.push(t.fallthrough);
            }
            Terminal::Scope(t) if !active_loops.is_empty() => {
                block.terminal = Terminal::PrunedScope(PrunedScopeTerminal {
                    id: t.id,
                    block: t.block,
                    scope: t.scope.clone(),
                    fallthrough: t.fallthrough,
                    loc: t.loc,
                });
            }
            // All other terminals: no-op
            Terminal::Branch(_)
            | Terminal::Goto(_)
            | Terminal::If(_)
            | Terminal::Label(_)
            | Terminal::Logical(_)
            | Terminal::MaybeThrow(_)
            | Terminal::Optional(_)
            | Terminal::PrunedScope(_)
            | Terminal::Return(_)
            | Terminal::Scope(_)
            | Terminal::Sequence(_)
            | Terminal::Switch(_)
            | Terminal::Ternary(_)
            | Terminal::Throw(_)
            | Terminal::Try(_)
            | Terminal::Unreachable(_)
            | Terminal::Unsupported(_) => {}
        }
    }
}

/// Flatten scopes with hooks or use -- removes reactive scopes that contain
/// hook calls or `use()` calls.
///
/// Hooks must be called unconditionally and in consistent order, so they
/// cannot be inside conditionally-executed reactive scopes. This pass finds
/// and removes any scopes that transitively contain a hook or use call.
pub fn flatten_scopes_with_hooks_or_use_hir(func: &mut HIRFunction) {
    let mut block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    block_ids.sort();

    // Phase 1: Forward pass to identify hook identifiers by name
    let mut hook_identifiers: FxHashSet<IdentifierId> = FxHashSet::default();
    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else {
            continue;
        };
        for instr in &block.instructions {
            if let InstructionValue::LoadGlobal(v) = &instr.value
                && is_hook_name(v.binding.name())
            {
                hook_identifiers.insert(instr.lvalue.identifier.id);
            }
        }
    }

    // Phase 2: Find scopes that contain hook calls
    let mut active_scopes: Vec<(BlockId, BlockId)> = Vec::new(); // (block_id, fallthrough)
    let mut prune: Vec<BlockId> = Vec::new();

    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else {
            continue;
        };

        // Remove scopes whose fallthrough matches current block
        active_scopes.retain(|entry| entry.1 != block.id);

        // Check instructions for hook/use calls
        for instr in &block.instructions {
            let is_hook_call = match &instr.value {
                InstructionValue::CallExpression(v) => {
                    hook_identifiers.contains(&v.callee.identifier.id)
                        || is_use_operator_by_name(v.callee.identifier.name.as_ref())
                }
                InstructionValue::MethodCall(v) => {
                    hook_identifiers.contains(&v.property.identifier.id)
                        || is_use_operator_by_name(v.property.identifier.name.as_ref())
                }
                _ => false,
            };

            if is_hook_call {
                prune.extend(active_scopes.iter().map(|entry| entry.0));
                active_scopes.clear();
            }
        }

        // Track scope terminals
        if let Terminal::Scope(t) = &block.terminal {
            active_scopes.push((block.id, t.fallthrough));
        }
    }

    // Phase 3: Prune the identified scopes
    for id in prune {
        let Some(block) = func.body.blocks.get(&id) else {
            continue;
        };

        let Terminal::Scope(terminal) = &block.terminal else {
            continue;
        };

        // Check if the scope body is a single hook-only instruction followed by goto
        // to the fallthrough. In that case, convert to a Label instead of PrunedScope.
        let body_block_id = terminal.block;
        let fallthrough = terminal.fallthrough;

        let is_single_hook_scope = func.body.blocks.get(&body_block_id).is_some_and(|body| {
            body.instructions.len() == 1
                && matches!(&body.terminal, Terminal::Goto(g) if g.block == fallthrough)
        });

        let block = func.body.blocks.get_mut(&id);
        let Some(block) = block else {
            continue;
        };

        let Terminal::Scope(terminal) = &block.terminal else {
            continue;
        };

        if is_single_hook_scope {
            block.terminal = Terminal::Label(LabelTerminal {
                id: terminal.id,
                block: terminal.block,
                fallthrough: terminal.fallthrough,
                loc: terminal.loc,
            });
        } else {
            block.terminal = Terminal::PrunedScope(PrunedScopeTerminal {
                id: terminal.id,
                block: terminal.block,
                scope: terminal.scope.clone(),
                fallthrough: terminal.fallthrough,
                loc: terminal.loc,
            });
        }
    }
}

/// Check if an identifier name matches the `use` operator pattern.
///
/// In the TS version, `isUseOperator` checks the type system for `BuiltInUseOperator`.
/// Since we don't have the full type system, we check by name: exactly "use".
fn is_use_operator_by_name(name: Option<&crate::hir::IdentifierName>) -> bool {
    name.is_some_and(|n| n.value() == "use")
}
