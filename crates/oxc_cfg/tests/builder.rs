use oxc_cfg::{ControlFlowGraphBuilder, CtxCursor};
use oxc_syntax::node::NodeId;
/// same as but just the skeleton
/// ```js
/// A: {
///   do {} while (a);
///   do {} while (b);
///   break A;
/// }
/// ```
#[test]
fn labeled_statement_with_multiple_loops_continue_and_break() {
    const A: Option<&str> = Some("A");

    let mut cfg = ControlFlowGraphBuilder::default();
    cfg.attach_error_harness(oxc_cfg::ErrorEdgeKind::Implicit);

    // labeled block start
    let labeled = cfg.new_basic_block_normal();
    cfg.ctx(A).default().allow_break().allow_continue();

    // loop context 1
    let c1 = cfg.new_basic_block_normal();
    cfg.ctx(None).default().allow_break().allow_continue();
    cfg.ctx(None).mark_break(c1).mark_continue(c1).resolve_with_upper_label();

    // loop context 2
    let c2 = cfg.new_basic_block_normal();
    cfg.ctx(None).default().allow_break().allow_continue();
    cfg.ctx(None).mark_break(c2).mark_continue(c2).resolve_with_upper_label();

    cfg.append_break(NodeId::DUMMY, A);

    // labeled block end
    cfg.ctx(A).mark_break(labeled).resolve();

    cfg.build();
}

#[test]
fn unlabeled_break_skips_labeled_context() {
    let mut cfg = ControlFlowGraphBuilder::default();
    cfg.attach_error_harness(oxc_cfg::ErrorEdgeKind::Implicit);
    cfg.ctx(None).default().allow_break();
    cfg.ctx(Some("B")).default().allow_break();
    let before_break = cfg.new_basic_block_normal();
    cfg.append_break(NodeId::DUMMY, None);
    let after_label = cfg.new_basic_block_normal();
    cfg.ctx(Some("B")).mark_break(after_label).resolve();
    let after_loop = cfg.new_basic_block_normal();
    cfg.ctx(None).mark_break(after_loop).resolve();
    let cfg = cfg.build();
    assert!(cfg.is_reachable(before_break, after_loop));
    assert!(!cfg.is_reachable(before_break, after_label));
}

#[test]
fn stacked_loop_labels_share_continue_target() {
    let mut cfg = ControlFlowGraphBuilder::default();
    cfg.attach_error_harness(oxc_cfg::ErrorEdgeKind::Implicit);
    let target = cfg.new_basic_block_normal();
    for label in ["a", "b", "c"] {
        cfg.ctx(Some(label)).default().allow_break();
    }
    cfg.ctx(None).default().allow_break().allow_continue();
    let mut sources = Vec::new();
    for label in ["a", "b", "c"] {
        let source = cfg.new_basic_block_normal();
        cfg.append_continue(NodeId::DUMMY, Some(label));
        sources.push(source);
    }
    cfg.ctx(None).mark_continue(target).resolve_with_upper_label();
    for _ in 0..3 {
        cfg.ctx(None).resolve();
    }
    let cfg = cfg.build();
    for source in sources {
        assert!(
            cfg.graph
                .edges_connecting(source, target)
                .any(|edge| matches!(edge.weight(), oxc_cfg::EdgeType::Jump))
        );
    }
}
