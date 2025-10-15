use oxc_ast::{
    AstKind,
    ast::{self, MethodDefinitionKind},
};
use oxc_cfg::{
    BlockNodeId, EdgeType, ErrorEdgeKind, InstructionKind, ReturnInstructionKind,
    graph::{
        Direction,
        visit::{EdgeRef, NodeRef},
    },
    visit::neighbors_filtered_by_edge_weight,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

fn constructor_super_diagnostic_unexpected_super(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Unexpected 'super()' because 'super' is not a constructor.")
        .with_help("Remove the 'super()' call from the constructor")
        .with_label(span)
}

fn constructor_super_diagnostic_missing_some_super(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Missing a call of 'super()' in some code paths.")
        .with_help("Add a 'super()' call to some code paths")
        .with_label(span)
}

fn constructor_super_diagnostic_missing_super(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Missing a call of 'super()' in the constructor.")
        .with_help("Add a 'super()' call to the constructor")
        .with_label(span)
}

fn constructor_super_diagnostic_duplicate_super(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Duplicate 'super()' call in the constructor.")
        .with_help("Remove the duplicate 'super()' call from the constructor")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks whether or not there is a valid super() call.
    ///
    /// ### Why is this bad?
    ///
    /// Constructors of derived classes must call super(). Constructors of non derived classes must not call super(). If this is not observed, the JavaScript engine will raise a runtime error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class A extends B {
    ///     constructor() { }  // Would throw a ReferenceError.
    /// }

    /// // Classes which inherits from a non constructor are always problems.
    /// class C extends null {
    ///     constructor() {
    ///         super();  // Would throw a TypeError.
    ///     }
    /// }

    /// class D extends null {
    ///     constructor() { }  // Would throw a ReferenceError.
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class A {
    ///     constructor() { }
    /// }

    /// class B extends C {
    ///     constructor() {
    ///         super();
    ///     }
    /// }
    /// ```
    ConstructorSuper,
    eslint,
    correctness,
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

#[derive(Default, Clone, Debug)]
struct TraverseState {
    super_called: bool,
    spans_in_loop: Option<Vec<Span>>,
    loop_ends_at: Option<BlockNodeId>,
}
impl Rule for ConstructorSuper {
    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        let mut basic_blocks_with_super_called = FxHashMap::<BlockNodeId, Span>::default();

        let graph = ctx.cfg().graph();

        // First pass: collect all basic blocks that contain a `super()` call
        for node in ctx.nodes() {
            if let AstKind::Super(_) = node.kind()
                && let AstKind::CallExpression(_) = ctx.nodes().parent_kind(node.id())
            {
                let cfg_id = ctx.nodes().cfg_id(node.id());
                match basic_blocks_with_super_called.entry(cfg_id) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        ctx.diagnostic(constructor_super_diagnostic_duplicate_super(node.span()));
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(node.span());
                    }
                }
            }
        }

        // Second pass: for each constructor, check the rules regarding `super()` calls.
        // Note: only constructors of classes that have a `super_class` are examined.
        // We are skipping class with or without valid `super_class` here because
        // 1. If a class has a valid superclass and no explicit constructor, it implicitly has a constructor that calls `super()`.
        // 2. If a class has an invalid superclass (e.g. a non-constructor), omitting a constructor is valid because no `super()` call is required.
        for node in ctx.nodes() {
            if let AstKind::MethodDefinition(method) = node.kind()
                && method.kind == MethodDefinitionKind::Constructor
            {
                let class_node: Option<&'a ast::Class<'a>> =
                    ctx.nodes().ancestors(node.id()).find_map(|parent| parent.kind().as_class());

                if let Some(class) = class_node
                    && let Some(super_class) = &class.super_class
                {
                    let is_constructor_like = Self::is_constructor_like(super_class);
                    let is_valid_superclass = super_class.is_null() || is_constructor_like;

                    let mut has_super_path = false;
                    let mut has_none_super_path = false;

                    // finding the entry block of the constructor function
                    let function_entry_block = graph
                        .edges_directed(ctx.nodes().cfg_id(node.id()), Direction::Outgoing)
                        .find(|edge| matches!(edge.weight(), EdgeType::NewFunction))
                        .map(|edge| edge.target())
                        .expect("Constructor should have a NewFunction edge");

                    neighbors_filtered_by_edge_weight(
                        graph,
                        function_entry_block,
                        &|edge| match edge {
                            EdgeType::Jump
                            | EdgeType::Normal
                            | EdgeType::Finalize
                            | EdgeType::Error(ErrorEdgeKind::Explicit) => None,
                            _ => Some(TraverseState::default()),
                        },
                        &mut |basic_block_id, mut state| {
                            let has_super =
                                basic_blocks_with_super_called.contains_key(basic_block_id);

                            // Mark the start of a loop by its end point.
                            if let Some(backage) = graph
                                .edges_directed(*basic_block_id, Direction::Incoming)
                                .find(|e| matches!(e.weight(), EdgeType::Backedge))
                            {
                                state.loop_ends_at = Some(backage.source().id());
                            }

                            // If we are in a loop and see a `super()` call, record it.
                            if has_super
                                && state.loop_ends_at.is_some()
                                && let Some(span) =
                                    basic_blocks_with_super_called.get(basic_block_id).copied()
                            {
                                match &mut state.spans_in_loop {
                                    Some(spans) => spans.push(span),
                                    None => state.spans_in_loop = Some(vec![span]),
                                }
                            }

                            // If we reach the end of a loop, report any duplicate `super()` calls in the loop.
                            if graph
                                .edges_directed(*basic_block_id, Direction::Outgoing)
                                .any(|e| matches!(e.weight(), EdgeType::Backedge))
                            {
                                state.loop_ends_at = None;

                                for span in state.spans_in_loop.take().unwrap_or_default() {
                                    ctx.diagnostic(constructor_super_diagnostic_duplicate_super(
                                        span,
                                    ));
                                }
                            }

                            // If the current basic block ends with `throw` or `return <expr>`, then it does not lead to any other blocks.
                            let terminates_abnormally =
                                ctx.cfg().basic_block(*basic_block_id).instructions().iter().any(
                                    |inst| {
                                        matches!(
                                            inst.kind,
                                            InstructionKind::Throw
                                                | InstructionKind::Return(
                                                    ReturnInstructionKind::NotImplicitUndefined
                                                )
                                        )
                                    },
                                );

                            if terminates_abnormally {
                                return (state, false);
                            }

                            if has_super {
                                let has_catch_edge = ctx
                                    .cfg()
                                    .graph()
                                    .edges_directed(*basic_block_id, Direction::Outgoing)
                                    .any(|e| {
                                        matches!(
                                            e.weight(),
                                            EdgeType::Error(ErrorEdgeKind::Explicit)
                                        )
                                    });

                                // If there is an outgoing edge to a catch block, the `super()` call may not be executed.
                                if !has_catch_edge {
                                    if state.super_called {
                                        ctx.diagnostic(
                                            constructor_super_diagnostic_duplicate_super(
                                                basic_blocks_with_super_called
                                                    .get(basic_block_id)
                                                    .copied()
                                                    .unwrap(),
                                            ),
                                        );
                                    }

                                    state.super_called = true;
                                }

                                // special handling for null superclass
                                if !is_valid_superclass {
                                    ctx.diagnostic(constructor_super_diagnostic_unexpected_super(
                                        basic_blocks_with_super_called
                                            .get(basic_block_id)
                                            .copied()
                                            .unwrap(),
                                    ));
                                }
                            }

                            let is_end_of_path = graph
                                .edges_directed(*basic_block_id, Direction::Outgoing)
                                .all(|e| {
                                    !matches!(
                                        e.weight(),
                                        EdgeType::Normal
                                            | EdgeType::Jump
                                            | EdgeType::Backedge
                                            | EdgeType::Finalize
                                            | EdgeType::Error(ErrorEdgeKind::Explicit)
                                    )
                                });

                            if is_end_of_path {
                                if state.super_called {
                                    has_super_path = true;
                                } else {
                                    has_none_super_path = true;
                                }
                            }

                            (state, true)
                        },
                        true,
                    );

                    if is_valid_superclass && has_super_path && !is_constructor_like {
                        ctx.diagnostic(constructor_super_diagnostic_unexpected_super(method.span));
                    }

                    if is_valid_superclass && !has_super_path && has_none_super_path {
                        ctx.diagnostic(constructor_super_diagnostic_missing_super(method.span));
                    }

                    if is_valid_superclass && has_super_path && has_none_super_path {
                        ctx.diagnostic(constructor_super_diagnostic_missing_some_super(
                            method.span,
                        ));
                    }
                }
            }
        }
    }
}

impl ConstructorSuper {
    fn is_constructor_like(expr: &ast::Expression) -> bool {
        match expr {
            ast::Expression::Identifier(_)
            | ast::Expression::FunctionExpression(_)
            | ast::Expression::NewExpression(_)
            | ast::Expression::ThisExpression(_)
            | ast::Expression::YieldExpression(_)
            | ast::Expression::TaggedTemplateExpression(_)
            | ast::Expression::MetaProperty(_)
            | ast::Expression::ChainExpression(_)
            | ast::Expression::StaticMemberExpression(_)
            | ast::Expression::ComputedMemberExpression(_)
            | ast::Expression::PrivateFieldExpression(_)
            | ast::Expression::ClassExpression(_) => true,
            ast::Expression::ParenthesizedExpression(paren) => {
                Self::is_constructor_like(&paren.expression)
            }
            ast::Expression::AssignmentExpression(assign) => match assign.operator {
                ast::AssignmentOperator::Assign | ast::AssignmentOperator::LogicalAnd => {
                    Self::is_constructor_like(&assign.right)
                }
                ast::AssignmentOperator::LogicalOr | ast::AssignmentOperator::LogicalNullish => {
                    !matches!(
                        assign.left,
                        ast::AssignmentTarget::ArrayAssignmentTarget(_)
                            | ast::AssignmentTarget::ObjectAssignmentTarget(_)
                    ) || Self::is_constructor_like(&assign.right)
                }
                _ => false,
            },
            ast::Expression::LogicalExpression(logical) => match logical.operator {
                ast::LogicalOperator::And => Self::is_constructor_like(&logical.right),
                _ => {
                    Self::is_constructor_like(&logical.left)
                        || Self::is_constructor_like(&logical.right)
                }
            },
            ast::Expression::ConditionalExpression(cond) => {
                Self::is_constructor_like(&cond.alternate)
                    || Self::is_constructor_like(&cond.consequent)
            }
            ast::Expression::SequenceExpression(seq) => {
                seq.expressions.last().is_some_and(|expr| Self::is_constructor_like(expr))
            }
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass: Vec<&'static str> = vec![
        "class A { }",
        "class A { constructor() { } }",
        "class A extends null { }",
        "class A extends B { }",
        "class A extends B { constructor() { super(); } }",
        "class A extends B { constructor() { if (true) { super(); } else { super(); } } }",
        "class A extends (class B {}) { constructor() { super(); } }",
        "class A extends (B = C) { constructor() { super(); } }",
        "class A extends (B &&= C) { constructor() { super(); } }",
        "class A extends (B ||= C) { constructor() { super(); } }",
        "class A extends (B ??= C) { constructor() { super(); } }",
        "class A extends (B ||= 5) { constructor() { super(); } }",
        "class A extends (B ??= 5) { constructor() { super(); } }",
        "class A extends (B || C) { constructor() { super(); } }",
        "class A extends (5 && B) { constructor() { super(); } }",
        "class A extends (false && B) { constructor() { super(); } }",
        "class A extends (B || 5) { constructor() { super(); } }",
        "class A extends (B ?? 5) { constructor() { super(); } }",
        "class A extends (a ? B : C) { constructor() { super(); } }",
        "class A extends (B, C) { constructor() { super(); } }",
        "class A { constructor() { class B extends C { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C { constructor() { } } } }",
        "class A extends B { constructor() { a ? super() : super(); } }",
        "class A extends B { constructor() { if (a) super(); else super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
        "class A extends B { constructor() { try {} finally { super(); } } }",
        "class A extends B { constructor() { if (a) throw Error(); super(); } }",
        "class A extends B { constructor() { if (true) return a; super(); } }",
        "class A extends null { constructor() { return a; } }",
        "class A { constructor() { return a; } }",
        "class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
        "class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",
        "class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",
        "class A extends Object {
        	    constructor() {
        	        super();
        	        for (let i = 0; i < 0; i++);
        	    }
        	}
        	",
        "class A extends Object {
        	    constructor() {
        	        super();
        	        for (; i < 0; i++);
        	    }
        	}
        	",
        "class A extends Object {
        	    constructor() {
        	        super();
        	        for (let i = 0;; i++) {
        	            if (foo) break;
        	        }
        	    }
        	}
        	",
        "class A extends Object {
        	    constructor() {
        	        super();
        	        for (let i = 0; i < 0;);
        	    }
        	}
        	",
        "class A extends Object {
        	    constructor() {
        	        super();
        	        for (let i = 0;;) {
        	            if (foo) break;
        	        }
        	    }
        	}
        	",
        "
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
        "class A extends obj?.prop { constructor() { super(); } }",
        "
        	            class A extends Base {
        	                constructor(list) {
        	                    for (const a of list) {
        	                        if (a.foo) {
        	                            super(a);
        	                            return;
        	                        }
        	                    }
        	                    super();
        	                }
        	            }
        	        ",
    ];

    let fail = vec![
        "class A extends null { constructor() { super(); } }",
        "class A extends null { constructor() { } }",
        "class A extends 100 { constructor() { super(); } }",
        "class A extends 'test' { constructor() { super(); } }",
        "class A extends (B = 5) { constructor() { super(); } }",
        "class A extends (B && 5) { constructor() { super(); } }",
        "class A extends (B &&= 5) { constructor() { super(); } }",
        "class A extends (B += C) { constructor() { super(); } }",
        "class A extends (B -= C) { constructor() { super(); } }",
        "class A extends (B **= C) { constructor() { super(); } }",
        "class A extends (B |= C) { constructor() { super(); } }",
        "class A extends (B &= C) { constructor() { super(); } }",
        "class A extends B { constructor() { } }",
        "class A extends B { constructor() { for (var a of b) super.foo(); } }",
        "class A extends B { constructor() { for (var i = 1; i < 10; i++) super.foo(); } }",
        "class A extends B { constructor() { var c = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var c = () => super(); } }",
        "class A extends B { constructor() { class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { var C = class extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { } } } }",
        "class A extends B { constructor() { super(); var C = class extends D { constructor() { } } } }",
        "class A extends B { constructor() { if (a) super(); } }",
        "class A extends B { constructor() { if (a); else super(); } }",
        "class A extends B { constructor() { a && super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); } } }",
        "class A extends B { constructor() { switch (a) { case 0: break; default: super(); } } }",
        "class A extends B { constructor() { try { super(); } catch (err) {} } }",
        "class A extends B { constructor() { try { a; } catch (err) { super(); } } }",
        "class A extends B { constructor() { if (a) return; super(); } }",
        "class A extends B { constructor() { super(); super(); } }",
        "class A extends B { constructor() { super() || super(); } }",
        "class A extends B { constructor() { if (a) super(); super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); default: super(); } } }",
        "class A extends B { constructor(a) { while (a) super(); } }",
        "class A extends B { constructor() { return; super(); } }",
        "class Foo extends Bar {
        	                constructor() {
        	                    for (a in b) for (c in d);
        	                }
        	            }",
        "class C extends D {

        	                constructor() {
        	                    do {
        	                        something();
        	                    } while (foo);
        	                }

        	            }",
        "class C extends D {

        	                constructor() {
        	                    for (let i = 1;;i++) {
        	                        if (bar) {
        	                            break;
        	                        }
        	                    }
        	                }

        	            }",
        "class C extends D {

        	                constructor() {
        	                    do {
        	                        super();
        	                    } while (foo);
        	                }

        	            }",
        "class C extends D {

        	                constructor() {
        	                    while (foo) {
        	                        if (bar) {
        	                            super();
        	                            break;
        	                        }
        	                    }
        	                }

        	            }",
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::PLUGIN, pass, fail).test_and_snapshot();
}
