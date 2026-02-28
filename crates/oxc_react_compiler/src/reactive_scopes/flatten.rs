/// Flatten reactive scopes in loops and hooks.
///
/// Ports of:
/// - `ReactiveScopes/FlattenReactiveLoopsHIR.ts`
/// - `ReactiveScopes/FlattenScopesWithHooksOrUseHIR.ts`
///
/// These passes flatten (remove) reactive scopes that cannot be correctly
/// memoized due to being inside loops or containing hook calls.
use crate::hir::{
    BlockId, HIRFunction, Identifier, InstructionValue, LabelTerminal, PrunedScopeTerminal,
    Terminal, environment::get_hook_kind_for_type, object_shape::BUILT_IN_USE_OPERATOR_ID,
};
use rustc_hash::FxHashSet;

/// Flatten reactive loops -- removes reactive scopes inside loops.
///
/// Loops may execute their body multiple times, so reactive scopes inside loops
/// cannot be correctly memoized with a single cache slot. Any `Scope` terminal
/// found while a loop is active is converted to a `PrunedScope` terminal.
///
/// This matches the TypeScript reference (`FlattenReactiveLoopsHIR.ts`) which iterates
/// `fn.body.blocks` in insertion order (JavaScript Map preserves insertion order).
/// After `BuildReactiveScopeTerminalsHIR` calls `reversePostorderBlocks` at the end
/// of its own pass, blocks are in TS RPO order.
///
/// IMPORTANT: The Rust `reverse_postorder_blocks` produces different ordering than
/// the TypeScript `reversePostorderBlocks` for graphs containing `Try` terminals
/// (specifically for catch/handler blocks vs loop fallthroughs). To avoid incorrect
/// pruning of reactive scopes in catch blocks, we compute the TS-compatible RPO
/// order internally and iterate in that order, without modifying the block map.
pub fn flatten_reactive_loops_hir(func: &mut HIRFunction) {
    // Compute TS-compatible RPO order for iteration.
    // The TS reference iterates fn.body.blocks in Map insertion order, which after
    // BuildReactiveScopeTerminalsHIR's reversePostorderBlocks call is TS RPO order.
    //
    // We compute TS-compatible RPO here (using successor ordering that matches TS's
    // recursive DFS) rather than using the current insertion order. This is necessary
    // because the Rust reverse_postorder_blocks uses a different successor visit order
    // that can place catch/handler blocks BEFORE their loop's fallthrough block,
    // causing incorrect pruning of reactive scopes in catch blocks.
    let block_ids = compute_ts_compatible_rpo(func);

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

/// Compute block IDs in TypeScript-compatible reverse postorder.
///
/// This produces the same block ordering as the TypeScript `reversePostorderBlocks`
/// function's DFS traversal. The key difference from Rust's `reverse_postorder_blocks`
/// is the order in which successors are pushed to the DFS stack:
///
/// - TS recursive DFS: reverses successors array, then visits in order
///   (e.g., for `[A, B]`: reversed = `[B, A]`, visit B first)
/// - This function: pushes successors in ORIGINAL order to LIFO stack
///   (e.g., for `[A, B]`: push A, push B; pop B first → same visit order as TS)
///
/// This matters for `Try` terminals where handler blocks must appear AFTER loop
/// fallthroughs in RPO to avoid incorrect pruning by `flatten_reactive_loops_hir`.
fn compute_ts_compatible_rpo(func: &HIRFunction) -> Vec<BlockId> {
    use crate::hir::hir_builder::each_terminal_successor;

    enum Phase {
        PreVisit,
        PostVisit,
    }

    let mut visited: FxHashSet<BlockId> = FxHashSet::default();
    let mut used: FxHashSet<BlockId> = FxHashSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();
    let mut stack: Vec<(BlockId, bool, Phase)> = Vec::new();
    stack.push((func.body.entry, true, Phase::PreVisit));

    while let Some((block_id, is_used, phase)) = stack.pop() {
        match phase {
            Phase::PreVisit => {
                let was_used = used.contains(&block_id);
                let was_visited = visited.contains(&block_id);
                visited.insert(block_id);
                if is_used {
                    used.insert(block_id);
                }
                if was_visited && (was_used || !is_used) {
                    continue;
                }

                let Some(block) = func.body.blocks.get(&block_id) else {
                    continue;
                };

                // Push successors in ORIGINAL order (without reversing) so that LIFO
                // pops them in the same visit order as TypeScript's recursive DFS.
                // TS: reverses successors to [B, A], then visits B first.
                // Rust LIFO: pushes [A, B], then pops B first. Same result.
                let successors: Vec<BlockId> =
                    each_terminal_successor(&block.terminal).into_iter().collect();
                let fallthrough = block.terminal.fallthrough();

                if !was_visited {
                    stack.push((block_id, is_used, Phase::PostVisit));
                }

                for &successor in &successors {
                    stack.push((successor, is_used, Phase::PreVisit));
                }

                if let Some(ft) = fallthrough {
                    stack.push((ft, false, Phase::PreVisit));
                }
            }
            Phase::PostVisit => {
                // Unconditionally push to postorder — the same block may have been
                // visited first as not-used (fallthrough) and later upgraded to used
                // (when reachable via a "used" path). We filter by used at the end.
                postorder.push(block_id);
            }
        }
    }

    postorder.reverse();
    // Only include blocks that were actually "used" (reachable from entry via
    // non-fallthrough paths). This matches the TypeScript reference which only
    // includes "used" blocks in the final block map.
    postorder.retain(|id| used.contains(id));
    postorder
}

/// Flatten scopes with hooks or use -- removes reactive scopes that contain
/// hook calls or `use()` calls.
///
/// Hooks must be called unconditionally and in consistent order, so they
/// cannot be inside conditionally-executed reactive scopes. This pass finds
/// and removes any scopes that transitively contain a hook or use call.
///
/// Port of `flattenScopesWithHooksOrUseHIR` from
/// `ReactiveScopes/FlattenScopesWithHooksOrUseHIR.ts`.
///
/// The TS version relies entirely on the type system (`getHookKind` and
/// `isUseOperator`) to identify hook calls. After InferTypes runs, each
/// identifier that refers to a hook has a `Function` type whose `shapeId`
/// points to a shape with `hookKind` set. This works for:
/// - Direct hook calls: `useState(0)` — the `useState` global is typed
/// - Method hook calls: `React.useState(0)` — `PropertyLoad` resolves the
///   property type via the shapes or the `isHookName` fallback in
///   `getPropertyType`, which returns a type with `hookKind: Custom`.
pub fn flatten_scopes_with_hooks_or_use_hir(func: &mut HIRFunction) {
    // Iterate in TypeScript-compatible RPO order.
    // This matches the TypeScript reference which iterates fn.body.blocks in Map order
    // (which is TS RPO order after BuildReactiveScopeTerminals' reversePostorderBlocks call).
    //
    // IMPORTANT: We must use compute_ts_compatible_rpo (not Rust's reverse_postorder_blocks)
    // because the Rust and TS RPO differ for certain control flow graphs. Specifically,
    // when a scope's fallthrough block contains only sequential code (like `useIdentity(null)`),
    // the TS RPO visits that fallthrough BEFORE the scope body's inner hook-containing blocks.
    // Using Rust's RPO order can cause incorrect pruning of outer scopes because hook-call
    // blocks appear before the outer scope's fallthrough in the iteration order.
    let block_ids = compute_ts_compatible_rpo(func);

    // Find scopes that contain hook calls
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
                    is_hook_or_use(&func.env, &v.callee.identifier)
                }
                InstructionValue::MethodCall(v) => {
                    is_hook_or_use(&func.env, &v.property.identifier)
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

    // Prune the identified scopes
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

/// Check if an identifier is a hook call or `use` operator.
///
/// Port of the inline check in the TS version:
/// ```ts
/// getHookKind(fn.env, callee.identifier) != null || isUseOperator(callee.identifier)
/// ```
///
/// This checks the identifier's type: if it has a `Function` type with a shape
/// that has `hookKind` set, it's a hook. If its shape_id is `BuiltInUseOperator`,
/// it's a `use()` call.
fn is_hook_or_use(env: &crate::hir::environment::Environment, identifier: &Identifier) -> bool {
    // Check getHookKind: type-based hook detection
    if get_hook_kind_for_type(env, &identifier.type_).is_some() {
        return true;
    }
    // Check isUseOperator: type-based use() detection
    if is_use_operator(identifier) {
        return true;
    }
    false
}

/// Check if an identifier is the `use` operator.
///
/// Port of `isUseOperator` from `HIR/HIR.ts`:
/// ```ts
/// id.type.kind === 'Function' && id.type.shapeId === 'BuiltInUseOperator'
/// ```
fn is_use_operator(id: &Identifier) -> bool {
    matches!(
        &id.type_,
        crate::hir::types::Type::Function(f)
            if f.shape_id.as_deref() == Some(BUILT_IN_USE_OPERATOR_ID)
    )
}
