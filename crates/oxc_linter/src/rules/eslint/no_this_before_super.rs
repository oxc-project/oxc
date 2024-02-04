use std::collections::{HashMap, HashSet};

use oxc_ast::{
    ast::{Expression, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    petgraph::stable_graph::NodeIndex, pg::neighbors_filtered_by_edge_weight, AstNodeId, EdgeType,
};
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-this-before-super): Expected to always call super() before this/super property access.")]
#[diagnostic(severity(warning), help("Call super() before this/super property access."))]
struct NoThisBeforeSuperDiagnostic(#[label] pub Span);

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
    nursery
);

impl NoThisBeforeSuper {
    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> bool {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::MethodDefinition(mdef) = parent.kind() {
                if matches!(mdef.kind, MethodDefinitionKind::Constructor) {
                    let parent_2 = ctx.nodes().parent_node(parent.id());
                    if let Some(parent_2) = parent_2 {
                        let parent_3 = ctx.nodes().parent_node(parent_2.id());
                        if let Some(parent_3) = parent_3 {
                            if let AstKind::Class(c) = parent_3.kind() {
                                if let Some(super_class) = &c.super_class {
                                    return !matches!(super_class, Expression::NullLiteral(_));
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }
}

impl Rule for NoThisBeforeSuper {
    fn run_once(&self, ctx: &LintContext) {
        let semantic = ctx.semantic();
        let cfg = semantic.cfg();

        // first pass -> find super calls and local violations
        let mut wanted_nodes = Vec::new();
        let mut basic_blocks_with_super_called = HashSet::<NodeIndex>::new();
        let mut basic_blocks_with_local_violations = HashMap::<NodeIndex, Vec<AstNodeId>>::new();
        for node in semantic.nodes().iter() {
            match node.kind() {
                AstKind::Function(_) | AstKind::ArrowExpression(_) => {
                    if Self::is_wanted_node(node, ctx) {
                        wanted_nodes.push(node);
                    }
                }
                AstKind::Super(_) => {
                    let basic_block_id = node.cfg_ix();
                    if let Some(parent) = semantic.nodes().parent_node(node.id()) {
                        if let AstKind::CallExpression(_) = parent.kind() {
                            // Note: we don't need to worry about also having invalid
                            // usage in the same callexpression, because arguments are visited
                            // before the callee in generating the semantic nodes.
                            basic_blocks_with_super_called.insert(basic_block_id);
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
                    let basic_block_id = node.cfg_ix();
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
            let output = neighbors_filtered_by_edge_weight(
                &cfg.graph,
                node.cfg_ix(),
                &|edge| match edge {
                    EdgeType::Normal => None,
                    EdgeType::Backedge | EdgeType::NewFunction => {
                        Some(DefinitelyCallsThisBeforeSuper::No)
                    }
                },
                &mut |basic_block_id, _| {
                    let super_called = basic_blocks_with_super_called.contains(basic_block_id);
                    if basic_blocks_with_local_violations.contains_key(basic_block_id) {
                        // super was not called before this in the current code path:
                        return (DefinitelyCallsThisBeforeSuper::Yes, false);
                    }

                    if super_called {
                        (DefinitelyCallsThisBeforeSuper::No, false)
                    } else {
                        (DefinitelyCallsThisBeforeSuper::No, true)
                    }
                },
            );

            // Deciding whether we definitely call this before super in all
            // codepaths is as simple as seeing if any individual codepath
            // definitely calls this before super.
            let violation_in_any_codepath =
                output.into_iter().any(|y| matches!(y, DefinitelyCallsThisBeforeSuper::Yes));

            // If not, flag it as a diagnostic.
            if violation_in_any_codepath {
                // the parent must exist, because of Self::is_wanted_node
                // so the unwrap() is safe here. The parent node is the
                // AstKind::MethodDefinition for `constructor`.
                let parent_span = ctx.nodes().parent_node(node.id()).unwrap().kind().span();
                ctx.diagnostic(NoThisBeforeSuperDiagnostic(parent_span));
            }
        }
    }

    fn from_configuration(_value: serde_json::Value) -> Self {
        Self
    }
}

#[derive(Default, Copy, Clone, Debug)]
enum DefinitelyCallsThisBeforeSuper {
    #[default]
    No,
    Yes,
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
        ("class A extends B { constructor() { super(); this.c = this.d; } }", None),
        ("class A extends B { constructor() { super(); this.c(); } }", None),
        ("class A extends B { constructor() { super(); super.c(); } }", None),
        ("class A extends B { constructor() { if (true) { super(); } else { super(); } this.c(); } }", None),
        ("class A extends B { constructor() { foo = super(); this.c(); } }", None),
        ("class A extends B { constructor() { foo += super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo |= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo &= super().a; this.c(); } }", None),

        // allows `this`/`super` in nested executable scopes, even if before `super()`.
        ("class A extends B { constructor() { class B extends C { constructor() { super(); this.d = 0; } } super(); } }", None),
        ("class A extends B { constructor() { var B = class extends C { constructor() { super(); this.d = 0; } }; super(); } }", None),
        ("class A extends B { constructor() { function c() { this.d(); } super(); } }", None),
        ("class A extends B { constructor() { var c = function c() { this.d(); }; super(); } }", None),
        ("class A extends B { constructor() { var c = () => this.d(); super(); } }", None),

        // ignores out of constructors.
        ("class A { b() { this.c = 0; } }", None),
        ("class A extends B { c() { this.d = 0; } }", None),
        ("function a() { this.b = 0; }", None),

        // multi code path.
        ("class A extends B { constructor() { if (a) { super(); this.a(); } else { super(); this.b(); } } }", None),
        ("class A extends B { constructor() { if (a) super(); else super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally {} this.a(); } }", None),

        // https://github.com/eslint/eslint/issues/5261
        ("class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }", None),
        ("class A extends B { constructor(a) { for (const b of a) { foo(b); } super(); } }", None),

        // https://github.com/eslint/eslint/issues/5319
        ("class A extends B { constructor(a) { super(); this.a = a && function(){} && this.foo; } }", None),

        // https://github.com/eslint/eslint/issues/5394
        (
            r"class A extends Object {
                constructor() {
                    super();
                    for (let i = 0; i < 0; i++);
                    this;
                }
            }", None),

        // https://github.com/eslint/eslint/issues/5894
        ("class A { constructor() { return; this; } }", None),
        ("class A extends B { constructor() { return; this; } }", None),

        // https://github.com/eslint/eslint/issues/8848
        (r"
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
        ", None),

        // Class field initializers are always evaluated after `super()`.
        ("class C { field = this.toString(); }", None),
        ("class C extends B { field = this.foo(); }", None),
        ("class C extends B { field = this.foo(); constructor() { super(); } }", None),
        ("class C extends B { field = this.foo(); constructor() { } }", None) // < in this case, initializers are never evaluated.
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
        ("class A extends B { constructor() { super(this.c); } }", None),
        ("class A extends B { constructor() { super(this.c()); } }", None),
        ("class A extends B { constructor() { super(super.c()); } }", None),
        // // even if is nested, reports correctly.
        ("class A extends B { constructor() { class C extends D { constructor() { super(); this.e(); } } this.f(); super(); } }", None),
        ("class A extends B { constructor() { class C extends D { constructor() { this.e(); super(); } } super(); this.f(); } }", None),
        // multi code path.
        ("class A extends B { constructor() { if (a) super(); this.a(); } }", None),
        ("class A extends B { constructor() { try { super(); } finally { this.a; } } }", None),
        ("class A extends B { constructor() { try { super(); } catch (err) { } this.a; } }", None),
        ("class A extends B { constructor() { foo &&= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ||= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { foo ??= super().a; this.c(); } }", None),
        ("class A extends B { constructor() { if (foo) { if (bar) { } super(); } this.a(); }}", None),
        ("class A extends B {
                constructor() {
                    if (foo) {
                    } else {
                      super();
                    }
                    this.a();
                }
            }", None),
        ("class A extends B {
                constructor() {
                    try {
                        call();
                    } finally {
                        this.a();
                    }
                }
            }", None),
        ("class A extends B {
                constructor() {
                    while (foo) {
                        super();
                    }
                    this.a();
                }
            }", None),
        ("class A extends B {
                constructor() {
                    while (foo) {
                        this.a();
                        super();
                    }
                }
            }", None),
        ("class A extends B {
                constructor() {
                    while (foo) {
                        if (init) {
                            this.a();
                            super();
                        }
                    }
                }
            }", None),
    ];

    Tester::new(NoThisBeforeSuper::NAME, pass, fail).test_and_snapshot();
}
