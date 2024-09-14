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
