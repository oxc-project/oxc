use crate::{AstNode, context::LintContext};
use oxc_ast::AstKind;
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, EvalConstConditionResult, Instruction, InstructionKind,
    ReturnInstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, EdgeRef, set_depth_first_search},
    },
};

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

pub fn effective_unreachable_blocks(ctx: &LintContext<'_>) -> Vec<bool> {
    let nodes = ctx.nodes();
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let mut unreachable = vec![true; cfg.basic_blocks.len()];
    let mut infinite_loops = Vec::new();

    for node in graph.node_indices() {
        let is_unreachable = cfg.basic_block(node).is_unreachable();
        unreachable[node.index()] = is_unreachable;

        if !is_unreachable
            && let Some(loop_) = cfg.is_infinite_loop_start(node, |instruction| match instruction {
                Instruction { kind: InstructionKind::Condition, node_id: Some(id) } => {
                    match nodes.kind(*id) {
                        AstKind::BooleanLiteral(lit) => EvalConstConditionResult::Eval(lit.value),
                        _ => EvalConstConditionResult::Fail,
                    }
                }
                _ => EvalConstConditionResult::NotFound,
            })
        {
            infinite_loops.push(loop_);
        }
    }

    for loop_ in infinite_loops {
        let starts: Vec<_> = graph
            .edges_directed(loop_.1, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), EdgeType::Normal))
            .map(|edge| edge.target())
            .collect();

        let _: Control<()> = set_depth_first_search(graph, starts, |event| match event {
            DfsEvent::Discover(node, _) => {
                let mut incoming = graph.edges_directed(node, Direction::Incoming);
                if incoming.any(|edge| match edge.weight() {
                    EdgeType::NewFunction
                    | EdgeType::Finalize
                    | EdgeType::Error(ErrorEdgeKind::Explicit) => true,
                    EdgeType::Normal | EdgeType::Jump
                        if edge.source() != loop_.1 && !unreachable[edge.source().index()] =>
                    {
                        true
                    }
                    EdgeType::Jump
                        if cfg.basic_block(edge.source()).instructions().iter().any(
                            |instruction| matches!(instruction.kind, InstructionKind::Break(_)),
                        ) =>
                    {
                        true
                    }
                    _ => false,
                }) {
                    Control::Prune
                } else {
                    unreachable[node.index()] = true;
                    Control::Continue
                }
            }
            _ => Control::Continue,
        });
    }

    unreachable
}
