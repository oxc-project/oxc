use oxc_cfg::{
    EdgeType, ErrorEdgeKind, InstructionKind, ReturnInstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, set_depth_first_search},
    },
};

use crate::{AstNode, context::LintContext};

/// Whether every code path inside the given function definitely reaches a
/// `return` (or `throw`) before falling off the end.
///
/// When `treat_undefined_as_unspecified` is `true`, a bare `return;`
/// (which implicitly returns `undefined`) is treated as a missing return.
///
/// `node` should be a `Function` or `ArrowFunctionExpression` (block-body).
/// Concise-body arrows (`() => expr`) always return, so callers can short-
/// circuit before invoking this.
pub fn definitely_returns_in_all_codepaths(
    node: &AstNode<'_>,
    ctx: &LintContext<'_>,
    treat_undefined_as_unspecified: bool,
) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();

    let output =
        set_depth_first_search(graph, Some(ctx.nodes().cfg_id(node.id())), |event| match event {
            DfsEvent::TreeEdge(a, b) => {
                if graph.edges_connecting(a, b).any(|e| {
                    matches!(
                        e.weight(),
                        EdgeType::Normal
                            | EdgeType::Jump
                            | EdgeType::Error(ErrorEdgeKind::Explicit)
                    )
                }) {
                    Control::Continue
                } else {
                    Control::Prune
                }
            }
            DfsEvent::Discover(basic_block_id, _) => {
                let return_instruction =
                    cfg.basic_block(basic_block_id).instructions().iter().find(|it| {
                        match it.kind {
                            InstructionKind::Return(_) | InstructionKind::Throw => true,
                            InstructionKind::ImplicitReturn
                            | InstructionKind::Break(_)
                            | InstructionKind::Continue(_)
                            | InstructionKind::Iteration(_)
                            | InstructionKind::Unreachable
                            | InstructionKind::Condition
                            | InstructionKind::Statement => false,
                        }
                    });

                let does_return = return_instruction.is_some_and(|ret| {
                    !matches!(
                        ret.kind,
                        InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
                            if treat_undefined_as_unspecified
                    )
                });

                if graph.edges_directed(basic_block_id, Direction::Outgoing).any(|e| {
                    matches!(
                        e.weight(),
                        EdgeType::Jump
                            | EdgeType::Normal
                            | EdgeType::Backedge
                            | EdgeType::Error(ErrorEdgeKind::Explicit)
                    )
                }) {
                    Control::Continue
                } else if does_return {
                    Control::Prune
                } else {
                    Control::Break(())
                }
            }
            _ => Control::Continue,
        });

    output.break_value().is_none()
}
