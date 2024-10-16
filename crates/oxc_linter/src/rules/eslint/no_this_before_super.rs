use oxc_ast::{
    ast::{Argument, Expression, MethodDefinitionKind},
    AstKind,
};
use oxc_cfg::{
    graph::visit::{neighbors_filtered_by_edge_weight, EdgeRef},
    BlockNodeId, ControlFlowGraph, EdgeType, ErrorEdgeKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_this_before_super_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to always call super() before this/super property access.")
        .with_help("Call super() before this/super property access.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisBeforeSuper;

declare_oxc_lint!(
    /// ### What it does
    /// Requires calling `super()` before using `this` or `super`.
    ///
    /// ### Why is this bad?
    /// Getters should always return a value.
    /// If they don't, it's probably a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// class A1 extends B {
    ///     constructor() {
    ///         // super() needs to be called first
    ///         this.a = 0;
    ///         super();
    ///     }
    /// }
    /// ```
    NoThisBeforeSuper,
    correctness
);

#[derive(Default, Copy, Clone, Debug)]
enum DefinitelyCallsThisBeforeSuper {
    #[default]
    No,
    Yes,
    Maybe(BlockNodeId),
}

impl Rule for NoThisBeforeSuper {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        let semantic = ctx.semantic();

        // first pass -> find super calls and local violations
        let mut wanted_nodes = Vec::new();
        let mut basic_blocks_with_super_called = FxHashSet::<BlockNodeId>::default();
        let mut basic_blocks_with_local_violations =
            FxHashMap::<BlockNodeId, Vec<NodeId>>::default();
        for node in semantic.nodes() {
            match node.kind() {
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                    if Self::is_wanted_node(node, ctx).unwrap_or_default() {
                        wanted_nodes.push(node);
                    }
                }
                AstKind::Super(_) => {
                    let basic_block_id = node.cfg_id();
                    if let Some(parent) = semantic.nodes().parent_node(node.id()) {
                        if let AstKind::CallExpression(call_expr) = parent.kind() {
                            let has_this_or_super_in_args =
                                Self::contains_this_or_super_in_args(&call_expr.arguments);

                            if !has_this_or_super_in_args {
                                basic_blocks_with_super_called.insert(basic_block_id);
                            }
                        }
                    }
                    if !basic_blocks_with_super_called.contains(&basic_block_id) {
                        basic_blocks_with_local_violations
                            .entry(basic_block_id)
                            .or_default()
                            .push(node.id());
                    }
                }
                AstKind::ThisExpression(_) => {
                    let basic_block_id = node.cfg_id();
                    if !basic_blocks_with_super_called.contains(&basic_block_id) {
                        basic_blocks_with_local_violations
                            .entry(basic_block_id)
                            .or_default()
                            .push(node.id());
                    }
                }
                _ => {}
            }
        }

        // second pass, walk cfg for wanted nodes and propagate
        // cross-block super calls:
        for node in wanted_nodes {
            let output = Self::analyze(
                cfg,
                node.cfg_id(),
                &basic_blocks_with_super_called,
                &basic_blocks_with_local_violations,
                false,
            );

            let violation_in_any_codepath = Self::check_for_violation(
                cfg,
                output,
                &basic_blocks_with_super_called,
                &basic_blocks_with_local_violations,
            );

            // If not, flag it as a diagnostic.
            if violation_in_any_codepath {
                // the parent must exist, because of Self::is_wanted_node
                // so the unwrap() is safe here. The parent node is the
                // AstKind::MethodDefinition for `constructor`.
                let parent_span = ctx.nodes().parent_node(node.id()).unwrap().kind().span();
                ctx.diagnostic(no_this_before_super_diagnostic(parent_span));
            }
        }
    }
}

impl NoThisBeforeSuper {
    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> Option<bool> {
        let parent = ctx.nodes().parent_node(node.id())?;
        let method_def = parent.kind().as_method_definition()?;

        if matches!(method_def.kind, MethodDefinitionKind::Constructor) {
            let parent_2 = ctx.nodes().parent_node(parent.id())?;
            let parent_3 = ctx.nodes().parent_node(parent_2.id())?;

            let class = parent_3.kind().as_class()?;
            let super_class = class.super_class.as_ref()?;
            return Some(!matches!(super_class, Expression::NullLiteral(_)));
        }

        Some(false)
    }

    fn analyze(
        cfg: &ControlFlowGraph,
        id: BlockNodeId,
        basic_blocks_with_super_called: &FxHashSet<BlockNodeId>,
        basic_blocks_with_local_violations: &FxHashMap<BlockNodeId, Vec<NodeId>>,
        follow_join: bool,
    ) -> Vec<DefinitelyCallsThisBeforeSuper> {
        neighbors_filtered_by_edge_weight(
            &cfg.graph,
            id,
            &|edge| match edge {
                EdgeType::Jump | EdgeType::Normal => None,
                EdgeType::Join if follow_join => None,
                EdgeType::Unreachable
                | EdgeType::Join
                | EdgeType::Error(_)
                | EdgeType::Finalize
                | EdgeType::Backedge
                | EdgeType::NewFunction => Some(DefinitelyCallsThisBeforeSuper::No),
            },
            &mut |basic_block_id, _| {
                let super_called = basic_blocks_with_super_called.contains(basic_block_id);
                if basic_blocks_with_local_violations.contains_key(basic_block_id) {
                    // super was not called before this in the current code path:
                    return (DefinitelyCallsThisBeforeSuper::Yes, false);
                }

                if super_called {
                    // If super is called but we are in a try-catch(-finally) block mark it as a
                    // maybe, since we might throw on super call and still call this in
                    // `catch`/`finally` block(s).
                    if cfg.graph.edges(*basic_block_id).any(|it| {
                        matches!(
                            it.weight(),
                            EdgeType::Error(ErrorEdgeKind::Explicit) | EdgeType::Finalize
                        )
                    }) {
                        (DefinitelyCallsThisBeforeSuper::Maybe(*basic_block_id), false)
                    // Otherwise we know for sure that super is called in this branch before
                    // reaching a this expression.
                    } else {
                        (DefinitelyCallsThisBeforeSuper::No, false)
                    }
                // If we haven't visited a super call and we have a non-error/finalize path
                // forward, continue visiting this branch.
                } else if cfg
                    .graph
                    .edges(*basic_block_id)
                    .any(|it| !matches!(it.weight(), EdgeType::Error(_) | EdgeType::Finalize))
                {
                    (DefinitelyCallsThisBeforeSuper::No, true)
                // Otherwise we mark it as a `Maybe` so we can analyze error/finalize paths separately.
                } else {
                    (DefinitelyCallsThisBeforeSuper::Maybe(*basic_block_id), false)
                }
            },
        )
    }

    fn check_for_violation(
        cfg: &ControlFlowGraph,
        output: Vec<DefinitelyCallsThisBeforeSuper>,
        basic_blocks_with_super_called: &FxHashSet<BlockNodeId>,
        basic_blocks_with_local_violations: &FxHashMap<BlockNodeId, Vec<NodeId>>,
    ) -> bool {
        // Deciding whether we definitely call this before super in all
        // codepaths is as simple as seeing if any individual codepath
        // definitely calls this before super.
        output.into_iter().any(|y| match y {
            DefinitelyCallsThisBeforeSuper::Yes => true,
            DefinitelyCallsThisBeforeSuper::No => false,
            DefinitelyCallsThisBeforeSuper::Maybe(id) => cfg.graph.edges(id).any(|edge| {
                let weight = edge.weight();
                let is_explicit_error = matches!(weight, EdgeType::Error(ErrorEdgeKind::Explicit));
                if is_explicit_error || matches!(weight, EdgeType::Finalize) {
                    Self::check_for_violation(
                        cfg,
                        Self::analyze(
                            cfg,
                            edge.target(),
                            basic_blocks_with_super_called,
                            basic_blocks_with_local_violations,
                            is_explicit_error,
                        ),
                        basic_blocks_with_super_called,
                        basic_blocks_with_local_violations,
                    )
                } else {
                    false
                }
            }),
        })
    }

    fn contains_this_or_super(arg: &Argument) -> bool {
        match arg {
            Argument::Super(_) | Argument::ThisExpression(_) => true,
            Argument::CallExpression(call_expr) => {
                matches!(&call_expr.callee, Expression::Super(_) | Expression::ThisExpression(_))
                    || matches!(&call_expr.callee,
                    Expression::StaticMemberExpression(static_member) if
                    matches!(static_member.object, Expression::Super(_) | Expression::ThisExpression(_)))
                    || Self::contains_this_or_super_in_args(&call_expr.arguments)
            }
            Argument::StaticMemberExpression(call_expr) => {
                matches!(&call_expr.object, Expression::Super(_) | Expression::ThisExpression(_))
            }
            _ => false,
        }
    }

    fn contains_this_or_super_in_args(args: &[Argument]) -> bool {
        args.iter().any(|arg| Self::contains_this_or_super(arg))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        /*
         * if the class has no extends or `extends null`, just ignore.
         * those classes cannot call `super()`.
         */
        ("class A { }", None),
        ("class A { constructor() { } }", None),
        ("class A { constructor() { this.b = 0; } }", None),
        ("class A { constructor() { this.b(); } }", None),
        ("class A extends null { }", None),
        ("class A extends null { constructor() { } }", None),
        // allows `this`/`super` after `super()`.
        ("class A extends B { }", None),
        ("class A extends B { constructor() { super(); } }", None),
        (
            "
        function f() {
            try {
                return a();
            }
            catch (err) {
                throw new class CustomError extends Error {
                    constructor() {
                        super(err);
                    }
                };
            }
            finally {
                this.b();
            }
        }
        ",
            None,
        ),
        ("class A extends B { constructor() { super(); this.c = this.d; } }", None),
        ("class A extends B { constructor() { super(); this.c(); } }", None),
        ("class A extends B { constructor() { super(); super.c(); } }", None),
        (
            "class A extends B { constructor() { if (true) { super(); } else { super(); } this.c(); } }",
            None,
        ),
        ("class A extends B { constructor() { foo = super(); this.c(); } }", None),
        ("class A extends B { constructor() { foo += super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo |= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo &= super().a; this.c(); } }", None),
        // allows `this`/`super` in nested executable scopes, even if before `super()`.
        (
            "class A extends B { constructor() { class B extends C { constructor() { super(); this.d = 0; } } super(); } }",
            None,
        ),
        (
            "class A extends B { constructor() { var B = class extends C { constructor() { super(); this.d = 0; } }; super(); } }",
            None,
        ),
        ("class A extends B { constructor() { function c() { this.d(); } super(); } }", None),
        (
            "class A extends B { constructor() { var c = function c() { this.d(); }; super(); } }",
            None,
        ),
        ("class A extends B { constructor() { var c = () => this.d(); super(); } }", None),
        // ignores out of constructors.
        ("class A { b() { this.c = 0; } }", None),
        ("class A extends B { c() { this.d = 0; } }", None),
        ("function a() { this.b = 0; }", None),
        // multi code path.
        (
            "class A extends B { constructor() { if (a) { super(); this.a(); } else { super(); this.b(); } } }",
            None,
        ),
        ("class A extends B { constructor() { if (a) super(); else super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally {} this.a(); } }", None),
        // https://github.com/eslint/eslint/issues/5261
        (
            "class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
            None,
        ),
        ("class A extends B { constructor(a) { for (const b of a) { foo(b); } super(); } }", None),
        // https://github.com/eslint/eslint/issues/5319
        (
            "class A extends B { constructor(a) { super(); this.a = a && function(){} && this.foo; } }",
            None,
        ),
        // https://github.com/eslint/eslint/issues/5394
        (
            r"class A extends Object {
                constructor() {
                    super();
                    for (let i = 0; i < 0; i++);
                    this;
                }
            }",
            None,
        ),
        // https://github.com/eslint/eslint/issues/5894
        ("class A { constructor() { return; this; } }", None),
        ("class A extends B { constructor() { return; this; } }", None),
        // https://github.com/eslint/eslint/issues/8848
        (
            r"
            class A extends B {
                constructor(props) {
                    super(props);

                    try {
                        let arr = [];
                        for (let a of arr) {
                        }
                    } catch (err) {
                    }
                }
            }
        ",
            None,
        ),
        // Class field initializers are always evaluated after `super()`.
        ("class C { field = this.toString(); }", None),
        ("class C extends B { field = this.foo(); }", None),
        ("class C extends B { field = this.foo(); constructor() { super(); } }", None),
        ("class C extends B { field = this.foo(); constructor() { } }", None), // < in this case, initializers are never evaluated.
    ];

    let fail = vec![
        // disallows all `this`/`super` if `super()` is missing.
        ("class A extends B { constructor() { this.c = 0; } }", None),
        ("class A extends B { constructor() { this.c(); } }", None),
        ("class A extends B { constructor() { super.c(); } }", None),
        // disallows `this`/`super` before `super()`.
        ("class A extends B { constructor() { this.c = 0; super(); } }", None),
        ("class A extends B { constructor() { this.c(); super(); } }", None),
        ("class A extends B { constructor() { super.c(); super(); } }", None),
        // disallows `this`/`super` in arguments of `super()`.
        ("class A extends B { constructor() { super(this); } }", None),
        ("class A extends B { constructor() { super(this.c); } }", None),
        ("class A extends B { constructor() { super(a(b(this.c))); } }", None),
        ("class A extends B { constructor() { super(this.c()); } }", None),
        ("class A extends B { constructor() { super(super.c); } }", None),
        ("class A extends B { constructor() { super(super.c()); } }", None),
        // // even if is nested, reports correctly.
        (
            "class A extends B { constructor() { class C extends D { constructor() { super(); this.e(); } } this.f(); super(); } }",
            None,
        ),
        (
            "class A extends B { constructor() { class C extends D { constructor() { this.e(); super(); } } super(); this.f(); } }",
            None,
        ),
        // multi code path.
        ("class A extends B { constructor() { if (a) super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally { this.a; } } }", None),
        ("class A extends B { constructor() { try { super(); } catch (err) { } this.a; } }", None),
        ("class A extends B { constructor() { foo &&= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ||= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ??= super().a; this.c(); } }", None),
        (
            "class A extends B { constructor() { if (foo) { if (bar) { } super(); } this.a(); }}",
            None,
        ),
        (
            "class A extends B {
                constructor() {
                    if (foo) {
                    } else {
                      super();
                    }
                    this.a();
                }
            }",
            None,
        ),
        (
            "class A extends B {
                constructor() {
                    try {
                        call();
                    } finally {
                        this.a();
                    }
                }
            }",
            None,
        ),
        (
            "class A extends B {
                constructor() {
                    while (foo) {
                        super();
                    }
                    this.a();
                }
            }",
            None,
        ),
        (
            "class A extends B {
                constructor() {
                    while (foo) {
                        this.a();
                        super();
                    }
                }
            }",
            None,
        ),
        (
            "class A extends B {
                constructor() {
                    while (foo) {
                        if (init) {
                            this.a();
                            super();
                        }
                    }
                }
            }",
            None,
        ),
    ];

    Tester::new(NoThisBeforeSuper::NAME, pass, fail).test_and_snapshot();
}
