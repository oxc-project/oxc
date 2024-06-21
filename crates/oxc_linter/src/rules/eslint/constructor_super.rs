use itertools::Itertools;
use oxc_ast::{
    ast::{AssignmentTarget, Expression, ExpressionStatement, MethodDefinitionKind},
    match_member_expression, AstKind,
};
use oxc_cfg::{
    graph::{
        visit::{Control, DfsEvent},
        Direction,
    },
    visit::set_depth_first_search,
    EdgeType, ErrorEdgeKind, InstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_span::{GetSpan, Span};

fn missing_some_super_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "eslint(constructor-super): Lacked a call of 'super()' in some code paths.",
    )
    .with_help("Ensure 'super()' is called from constructor")
    .with_labels([span.into()])
}

fn missing_all_super_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("eslint(constructor-super): Expected to call 'super()'.")
        .with_help("Ensure 'super()' is called from constructor")
        .with_labels([span.into()])
}

// fn duplicate_super_diagnostic(span: Span) -> OxcDiagnostic {
//     OxcDiagnostic::error("eslint(constructor-super): Unexpected duplicate 'super()'.")
//         .with_help("'super()' should only be called once.")
//         .with_labels([span.into()])
// }

fn bad_super_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "eslint(constructor-super): Unexpected 'super()' because 'super' is not a constructor.",
    )
    .with_help("Do not call 'super()' from constructor.")
    .with_labels([span.into()])
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

declare_oxc_lint!(
    /// ### What it does
    /// Require 'super()' calls in constructors.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// class A extends B {
    ///   constructor() {}
    /// }
    /// ```
    ConstructorSuper,
    nursery // This rule should be implemented with CFG, the current implementation has a lot of
            // false positives.
);

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(super_) = Self::is_wanted_node(node, ctx) else {
            return;
        };

        let span = node.kind().span();
        if !Self::is_super_constructor(super_) {
            ctx.diagnostic(bad_super_diagnostic(span));
        }

        let cfg = ctx.cfg();

        let graph = cfg.graph();
        let mut met_any_super = false;
        let output = set_depth_first_search(graph, Some(node.cfg_id()), |event| {
            match event {
                // We only need to check paths that are normal or jump.
                DfsEvent::TreeEdge(a, b) => {
                    let edges = graph.edges_connecting(a, b).collect::<Vec<_>>();
                    if edges.iter().any(|e| {
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
                    let has_super = cfg
                        .basic_block(basic_block_id)
                        .instructions()
                        .iter()
                        .rev()
                        .filter(|it| {
                            matches!(
                                it.kind,
                                InstructionKind::Statement | InstructionKind::Condition
                            )
                        })
                        .filter_map(|it| {
                            it.node_id
                                .map(|id| ctx.nodes().get_node(id))
                                .map(|node| (&it.kind, node))
                        })
                        .find(|(ins, node)| {
                            if matches!(ins, InstructionKind::Condition)
                                && ctx.nodes().parent_kind(node.id()).is_some_and(|it| {
                                    matches!(it, AstKind::ConditionalExpression(_))
                                })
                            {
                                let edges = graph
                                    .edges_directed(basic_block_id, Direction::Outgoing)
                                    .filter(|e| {
                                        matches!(e.weight(), EdgeType::Jump | EdgeType::Normal)
                                    })
                                    .collect_vec();
                                dbg!(edges);
                                false
                            } else {
                                matches!(
                                    node.kind(),
                                    AstKind::ExpressionStatement(ExpressionStatement {
                                        expression: expr,
                                        ..
                                    }) if expr.is_super_call_expression()
                                )
                            }
                        });
                    if has_super.is_some() {
                        met_any_super = true;
                        Control::Prune
                    }
                    // Return true as the second argument to signify we should
                    // continue walking this branch, as we haven't seen anything
                    // that will signify to us that this path of the program will
                    // definitely return or throw.
                    else if graph.edges_directed(basic_block_id, Direction::Outgoing).any(|e| {
                        matches!(
                            e.weight(),
                            EdgeType::Jump
                                | EdgeType::Normal
                                | EdgeType::Error(ErrorEdgeKind::Explicit)
                        )
                    }) {
                        Control::Continue
                    } else {
                        Control::Break(())
                    }
                }
                _ => Control::Continue,
            }
        });

        let definitely_calls_super_in_all_codepaths = output.break_value().is_none();

        if !met_any_super {
            ctx.diagnostic(missing_all_super_diagnostic(span));
        } else if !definitely_calls_super_in_all_codepaths {
            ctx.diagnostic(missing_some_super_diagnostic(span));
        }
    }
}

impl ConstructorSuper {
    fn is_wanted_node<'a, 'b>(
        node: &'b AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<&'b Expression<'a>> {
        let nodes = ctx.nodes();

        if !matches!(node.kind(), AstKind::Function(_)) {
            return None;
        }

        let parent = nodes.parent_node(node.id())?;

        let AstKind::MethodDefinition(mdef) = parent.kind() else {
            return None;
        };

        if !matches!(mdef.kind, MethodDefinitionKind::Constructor) {
            return None;
        }

        let parent_2 = ctx.nodes().parent_node(parent.id())?;

        let parent_3 = ctx.nodes().parent_node(parent_2.id())?;

        let AstKind::Class(c) = parent_3.kind() else {
            return None;
        };

        c.super_class.as_ref()
    }

    fn is_super_constructor(expr: &Expression) -> bool {
        #[allow(clippy::unnested_or_patterns)]
        match expr {
            Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ThisExpression(_)
            | Expression::CallExpression(_)
            | Expression::NewExpression(_)
            | Expression::ChainExpression(_)
            | Expression::YieldExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::MetaProperty(_)
            | match_member_expression!(Expression) => true,

            Expression::ParenthesizedExpression(expr) => {
                Self::is_super_constructor(&expr.expression)
            }

            Expression::Identifier(ident) => ident.name != "undefined",

            Expression::AssignmentExpression(assign) => match assign.operator {
                AssignmentOperator::Assign | AssignmentOperator::LogicalAnd => {
                    Self::is_super_constructor(&assign.right)
                }
                AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish => {
                    let left = match &assign.left {
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            ident.name != "undefined"
                        }
                        match_member_expression!(AssignmentTarget) => true,
                        _ => false,
                    };
                    left || Self::is_super_constructor(&assign.right)
                }
                _ => false,
            },

            Expression::LogicalExpression(logical) => {
                if matches!(logical.operator, LogicalOperator::And) {
                    Self::is_super_constructor(&logical.right)
                } else {
                    Self::is_super_constructor(&logical.left)
                        || Self::is_super_constructor(&logical.right)
                }
            }

            Expression::ConditionalExpression(cond) => {
                Self::is_super_constructor(&cond.alternate)
                    || Self::is_super_constructor(&cond.consequent)
            }

            Expression::SequenceExpression(seq) => {
                seq.expressions.last().is_some_and(Self::is_super_constructor)
            }

            _ => false,
        }
    }
    //
    // fn has_super_call(node: &AstNode) -> bool {
    // }
}

// struct

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // non derived classes.
        "class A { }",
        "class A { constructor() { } }",

        // inherit from non constructors.
        // those are valid if we don't define the constructor.
        "class A extends null { }",

        // derived classes.
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
        // A future improvement could detect the left side as statically falsy, making this invalid.
        "class A extends (false && B) { constructor() { super(); } }",
        "class A extends (B || 5) { constructor() { super(); } }",
        "class A extends (B ?? 5) { constructor() { super(); } }",

        "class A extends (a ? B : C) { constructor() { super(); } }",
        "class A extends (B, C) { constructor() { super(); } }",

        // nested.
        "class A { constructor() { class B extends C { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
        "class A extends B { constructor() { super(); class C { constructor() { } } } }",

        // multi code path.
        "class A extends B { constructor() { a ? super() : super(); } }",
        "class A extends B { constructor() { if (a) super(); else super(); } }",
        "class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
        "class A extends B { constructor() { try {} finally { super(); } } }",
        "class A extends B { constructor() { if (a) throw Error(); super(); } }",

        // returning value is a substitute of 'super()'.
        "class A extends B { constructor() { if (true) return a; super(); } }",
        "class A extends null { constructor() { return a; } }",
        "class A { constructor() { return a; } }",

        // https://github.com/eslint/eslint/issues/5261
        "class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
        "class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",

        // https://github.com/eslint/eslint/issues/5319
        "class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",

        // https://github.com/eslint/eslint/issues/5394
        "class A extends Object {\n\
             constructor() {\n\
                 super();\n\
                 for (let i = 0; i < 0; i++);\n\
             }\n\
        }",
        "class A extends Object {\n\
             constructor() {\n\
                 super();\n\
                 for (; i < 0; i++);\n\
             }\n\
        }",
        "class A extends Object {\n\
             constructor() {\n\
                 super();\n\
                 for (let i = 0;; i++) {\n\
                     if (foo) break;\n\
                 }\n\
             }\n\
        }",
        "class A extends Object {\n\
             constructor() {\n\
                 super();\n\
                 for (let i = 0; i < 0;);\n\
             }\n\
        }",
        "class A extends Object {\n\
             constructor() {\n\
                 super();\n\
                 for (let i = 0;;) {\n\
                     if (foo) break;\n\
                 }\n\
             }\n\
        }",
        // https://github.com/eslint/eslint/issues/8848
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
        // Optional chaining
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
        "class A extends B { constructor() {} }",
        "class A extends null { constructor() { super(); } }",
        "class A extends null { constructor() { } }",
        "class A extends 100 { constructor() { super(); } }",
        "class A extends 'test' { constructor() { super(); } }",
    ];

    Tester::new(ConstructorSuper::NAME, pass, fail).test_and_snapshot();
}
