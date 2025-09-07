use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression, Statement, UnaryOperator},
};
use oxc_cfg::{
    EdgeType, ErrorEdgeKind, InstructionKind,
    graph::{
        Direction,
        visit::{Control, DfsEvent, set_depth_first_search},
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn always_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Each then() should return a value or throw").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AlwaysReturn(Box<AlwaysReturnConfig>);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlwaysReturnConfig {
    #[serde(default)]
    ignore_last_callback: bool,
    #[serde(default)]
    ignore_assignment_variable: FxHashSet<Cow<'static, str>>,
}

impl Default for AlwaysReturnConfig {
    fn default() -> Self {
        Self {
            ignore_last_callback: false,
            ignore_assignment_variable: FxHashSet::from_iter([Cow::Borrowed("globalThis")]),
        }
    }
}

impl std::ops::Deref for AlwaysReturn {
    type Target = AlwaysReturnConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require returning inside each `then()` to create readable and reusable Promise chains.
    /// We also allow someone to throw inside a `then()` which is essentially the same as return `Promise.reject()`.
    ///
    /// ### Why is this bad?
    ///
    /// Broken Promise Chain.
    /// Inside the first `then()` callback, a function is called but not returned.
    /// This causes the next `then()` in the chain to execute immediately without waiting for the called function to complete.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// myPromise.then(function (val) {})
    /// myPromise.then(() => {
    ///     doSomething()
    /// })
    /// myPromise.then((b) => {
    ///     if (b) {
    ///         return 'yes'
    ///     } else {
    ///         forgotToReturn()
    ///     }
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// myPromise.then((val) => val * 2)
    /// myPromise.then(function (val) {
    ///     return val * 2
    ///})
    /// myPromise.then(doSomething) // could be either
    /// myPromise.then((b) => {
    ///     if (b) {
    ///         return 'yes'
    ///     } else {
    ///         return 'no'
    ///     }
    /// })
    /// ```
    ///
    /// ### Options
    ///
    /// #### `ignoreLastCallback`
    ///
    /// You can pass an `{ ignoreLastCallback: true }` as an option to this rule so that
    /// the last `then()` callback in a promise chain does not warn if it does not have
    /// a `return`. Default is `false`.
    ///
    /// ```javascript
    /// // OK
    /// promise.then((x) => {
    ///     console.log(x)
    /// })
    /// // OK
    /// void promise.then((x) => {
    ///     console.log(x)
    /// })
    /// // OK
    /// await promise.then((x) => {
    ///     console.log(x)
    /// })
    ///
    /// promise
    ///     // NG
    ///     .then((x) => {
    ///         console.log(x)
    ///     })
    ///     // OK
    ///     .then((x) => {
    ///         console.log(x)
    /// })
    ///
    /// // NG
    /// const v = promise.then((x) => {
    ///     console.log(x)
    /// })
    /// // NG
    /// const v = await promise.then((x) => {
    ///     console.log(x)
    /// })
    /// function foo() {
    ///     // NG
    ///     return promise.then((x) => {
    ///         console.log(x)
    ///     })
    /// }
    /// ```
    ///
    /// #### `ignoreAssignmentVariable`
    ///
    /// You can pass an `{ ignoreAssignmentVariable: [] }` as an option to this rule
    /// with a list of variable names so that the last `then()` callback in a promise
    /// chain does not warn if it does an assignment to a global variable. Default is
    /// `["globalThis"]`.
    ///
    /// ```javascript
    /// /* eslint promise/always-return: ["error", { ignoreAssignmentVariable: ["globalThis"] }] */
    ///
    /// // OK
    /// promise.then((x) => {
    ///     globalThis = x
    /// })
    ///
    /// promise.then((x) => {
    ///     globalThis.x = x
    /// })
    ///
    /// // OK
    /// promise.then((x) => {
    ///     globalThis.x.y = x
    /// })
    ///
    /// // NG
    /// promise.then((x) => {
    ///     anyOtherVariable = x
    /// })
    ///
    /// // NG
    /// promise.then((x) => {
    ///     anyOtherVariable.x = x
    /// })
    ///
    /// // NG
    /// promise.then((x) => {
    ///     x()
    /// })
    /// ```

    AlwaysReturn,
    promise,
    suspicious,
);

const PROCESS_METHODS: [&str; 2] = ["exit", "abort"];

impl Rule for AlwaysReturn {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(
            value
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or_default(),
        ))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_inline_then_function_expression(node, ctx) {
            return;
        }
        // want CallExpression (direct parent after AST changes)
        let parent = ctx.nodes().parent_node(node.id());
        if self.ignore_last_callback && is_last_callback(parent, ctx) {
            return;
        }
        if !self.ignore_assignment_variable.is_empty()
            && is_last_callback(parent, ctx)
            && has_ignored_assignment(node, &self.ignore_assignment_variable)
        {
            return;
        }
        if has_no_return_code_path(node, ctx) {
            ctx.diagnostic(always_return_diagnostic(node.span()));
        }
    }
}

fn is_function_with_block_statement(node: &AstNode) -> bool {
    matches!(node.kind(), AstKind::Function(function_expr) if !function_expr.declare) // e.g. function () {}
        || matches!(node.kind(), AstKind::ArrowFunctionExpression(arrow_func_expr) if !arrow_func_expr.expression) // e.g. () => {}
}

fn is_member_call(node: &AstNode, member_name: &str) -> bool {
    if let AstKind::CallExpression(call_expr) = node.kind() {
        return matches!(&call_expr.callee, Expression::StaticMemberExpression(member_expr) if member_expr.property.name == member_name);
    }
    false
}

fn is_first_argument(node: &AstNode, call_node: &AstNode) -> bool {
    match call_node.kind() {
        AstKind::CallExpression(call_exp) => {
            call_exp.arguments.first().is_some_and(|arg| arg.span() == node.span())
        }
        _ => false,
    }
}

fn is_inline_then_function_expression(node: &AstNode, ctx: &LintContext) -> bool {
    // After AST changes, the parent is directly the CallExpression
    let parent = ctx.nodes().parent_node(node.id());

    is_function_with_block_statement(node)
        && is_member_call(parent, "then")
        && is_first_argument(node, parent)
}

fn is_last_callback(node: &AstNode, ctx: &LintContext) -> bool {
    let get_parent_node = |n: &AstNode| ctx.nodes().parent_node(n.id());
    let mut target = node;
    let mut parent = get_parent_node(node);
    while parent.id() != NodeId::ROOT {
        match parent.kind() {
            AstKind::ExpressionStatement(_) => {
                // e.g. { promise.then(() => value) }
                return true;
            }
            AstKind::UnaryExpression(unary_expr) => {
                // e.g. void promise.then(() => value)
                return unary_expr.operator == UnaryOperator::Void;
            }
            AstKind::SequenceExpression(sequence_expr) => {
                // e.g. (promise.then(() => value), expr)
                if let Some(last_expr) = sequence_expr.expressions.last()
                    && target.kind().span() != last_expr.span()
                {
                    return true;
                }
                target = parent;
                parent = get_parent_node(parent);
            }
            AstKind::ParenthesizedExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::AwaitExpression(_) => {
                // e.g. promise?.then(() => value) | await promise.then(() => value)
                target = parent;
                parent = get_parent_node(parent);
            }
            AstKind::StaticMemberExpression(_) => {
                // e.g. promise.then(() => value).catch(e => {})
                // want CallExpression
                let parent1 = get_parent_node(parent);
                if is_member_call(parent1, "catch") || is_member_call(parent1, "finally") {
                    target = parent1;
                    parent = get_parent_node(parent1);
                } else {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }
    }
    false
}

fn is_nodejs_terminal_statement(node: &AstNode) -> bool {
    node.kind().as_expression_statement().is_some_and(|exp| {
        match &exp.expression {
            Expression::CallExpression(call_expr) => {
                match &call_expr.callee {
                    Expression::StaticMemberExpression(member_expr) if PROCESS_METHODS.contains(&member_expr.property.name.as_str()) => {
                        matches!(&member_expr.object, Expression::Identifier(identifier) if identifier.name == "process")
                    },
                    _ => {
                        false
                    }

                }
            },
            _ => false,
        }
    })
}

fn has_no_return_code_path(node: &AstNode, ctx: &LintContext) -> bool {
    let cfg = ctx.cfg();
    let graph = cfg.graph();
    let output = set_depth_first_search(graph, Some(ctx.nodes().cfg_id(node.id())), |event| {
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
                let return_instruction =
                    cfg.basic_block(basic_block_id).instructions().iter().find(|it| {
                        match it.kind {
                            // return or throw
                            InstructionKind::Return(_) | InstructionKind::Throw => true,
                            InstructionKind::Statement => it.node_id.is_some_and(|node_id| {
                                // process.exit(0) | process.abort()
                                let node = ctx.nodes().get_node(node_id);
                                is_nodejs_terminal_statement(node)
                            }),
                            _ => false,
                        }
                    });

                let does_return = return_instruction.is_some();

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
        }
    });
    output.break_value().is_some()
}

fn has_ignored_assignment(
    node: &AstNode,
    ignore_assignment_variable: &FxHashSet<Cow<str>>,
) -> bool {
    let body_statements = match node.kind() {
        AstKind::Function(func_expr) => func_expr.body.as_ref().map(|body| &body.statements),
        AstKind::ArrowFunctionExpression(arrow_func_expr) => Some(&arrow_func_expr.body.statements),
        _ => None,
    };
    body_statements.is_some_and(|statements| {
        statements.iter().any(|it| match it {
            Statement::ExpressionStatement(expression) => match &expression.expression {
                Expression::AssignmentExpression(assignment_expr) => {
                    let object_name = get_root_object_name(&assignment_expr.left);
                    object_name.is_some_and(|name| ignore_assignment_variable.contains(name))
                }
                _ => false,
            },
            _ => false,
        })
    })
}

fn get_root_object_name<'a>(assignment_target: &AssignmentTarget<'a>) -> Option<&'a str> {
    match assignment_target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => Some(id.name.as_str()),
        AssignmentTarget::StaticMemberExpression(member_expr) => {
            get_member_expr_root_object_name(&member_expr.object)
        }
        AssignmentTarget::ComputedMemberExpression(member_expr) => {
            get_member_expr_root_object_name(&member_expr.object)
        }
        _ => None,
    }
}

fn get_member_expr_root_object_name<'a>(member_expr: &Expression<'a>) -> Option<&'a str> {
    match member_expr {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member_expr) => {
            get_member_expr_root_object_name(&member_expr.object)
        }
        Expression::ComputedMemberExpression(member_expr) => {
            get_member_expr_root_object_name(&member_expr.object)
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("hey.then(x => x)", None),
        ("hey.then(x => ({}))", None),
        ("hey.then(x => { return; })", None),
        ("hey.then(x => { return x ? x.id : null })", None),
        ("hey.then(x => { return x * 10 })", None),
        ("hey.then(x => { process.exit(0); })", None),
        ("hey.then(x => { process.abort(); })", None),
        ("hey.then(function() { return 42; })", None),
        ("hey.then(function() { return new Promise(); })", None),
        (r#"hey.then(function() { return "x"; }).then(doSomethingWicked)"#, None),
        (r#"hey.then(x => x).then(function() { return "3" })"#, None),
        (r#"hey.then(function() { throw new Error("msg"); })"#, None),
        (r#"hey.then(function(x) { if (!x) { throw new Error("no x"); } return x; })"#, None),
        (r#"hey.then(function(x) { if (x) { return x; } throw new Error("no x"); })"#, None),
        (r#"hey.then(function(x) { if (x) { process.exit(0); } throw new Error("no x"); })"#, None),
        (r#"hey.then(function(x) { if (x) { process.abort(); } throw new Error("no x"); })"#, None),
        (r#"hey.then(x => { throw new Error("msg"); })"#, None),
        (r#"hey.then(x => { if (!x) { throw new Error("no x"); } return x; })"#, None),
        (r#"hey.then(x => { if (x) { return x; } throw new Error("no x"); })"#, None),
        ("hey.then(x => { var f = function() { }; return f; })", None),
        ("hey.then(x => { if (x) { return x; } else { return x; } })", None),
        (r#"hey.then(x => { return x; var y = "unreachable"; })"#, None),
        (r#"hey.then(x => { return x; return "unreachable"; })"#, None),
        ("hey.then(x => { return; }, err=>{ log(err); })", None),
        ("hey.then(x => { return x && x(); }, err=>{ log(err); })", None),
        ("hey.then(x => { return x.y || x(); }, err=>{ log(err); })", None),
        (
            "hey.then(x => {
    return anotherFunc({
        nested: {
            one: x === 1 ? 1 : 0,
            two: x === 2 ? 1 : 0
        }
    })
})",
            None,
        ),
        (
            "hey.then(({x, y}) => {
    if (y) {
        throw new Error(x || y)
    }
    return x
})",
            None,
        ),
        (
            "hey.then(x => { console.log(x) })",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "if(foo) { hey.then(x => { console.log(x) }) }",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "void hey.then(x => { console.log(x) })",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "async function foo() {
                await hey.then(x => { console.log(x) })
            }",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "hey?.then(x => { console.log(x) })",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "foo = (hey.then(x => { console.log(x) }), 42)",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "(42, hey.then(x => { console.log(x) }))",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "hey
    .then(x => { console.log(x) })
    .catch(e => console.error(e))",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "hey
    .then(x => { console.log(x) })
    .catch(e => console.error(e))
    .finally(() => console.error('end'))",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "hey
    .then(x => { console.log(x) })
    .finally(() => console.error('end'))",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        ("hey.then(x => { globalThis = x })", None),
        ("hey.then(x => { globalThis[a] = x })", None),
        ("hey.then(x => { globalThis.a = x })", None),
        ("hey.then(x => { globalThis.a.n = x })", None),
        ("hey.then(x => { globalThis[12] = x })", None),
        (r#"hey.then(x => { globalThis['12']["test"] = x })"#, None),
        (
            "hey.then(x => { window['x'] = x })",
            Some(serde_json::json!([{ "ignoreAssignmentVariable": ["globalThis", "window"] }])),
        ),
    ];

    let fail = vec![
        ("hey.then(x => {})", None),
        ("hey.then(function() { })", None),
        ("hey.then(function() { }).then(x)", None),
        ("hey.then(function() { }).then(function() { })", None),
        ("hey.then(function() { return; }).then(function() { })", None),
        ("hey.then(function() { doSomethingWicked(); })", None),
        ("hey.then(function() { if (x) { return x; } })", None),
        ("hey.then(function() { if (x) { return x; } else { }})", None),
        ("hey.then(function() { if (x) { } else { return x; }})", None),
        ("hey.then(function() { if (x) { process.chdir(); } else { return x; }})", None),
        ("hey.then(function() { if (x) { return you.then(function() { return x; }); } })", None),
        ("hey.then( x => { x ? x.id : null })", None),
        ("hey.then(function(x) { x ? x.id : null })", None),
        (
            "(function() {
    return hey.then(x => {
        anotherFunc({
            nested: {
                one: x === 1 ? 1 : 0,
                two: x === 2 ? 1 : 0
            }
        })
    })
})()",
            None,
        ),
        (
            "hey.then(({x, y}) => {
    if (y) {
        throw new Error(x || y)
    }
})",
            None,
        ),
        (
            "hey.then(({x, y}) => {
    if (y) {
        return x
    }
})",
            None,
        ),
        (
            "hey
    .then(function(x) { console.log(x) /* missing return here */ })
    .then(function(y) { console.log(y) /* no error here */ })",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "const foo = hey.then(function(x) {});",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "function foo() {
    return hey.then(function(x) {});
}",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "async function foo() {
    return await hey.then(x => { console.log(x) })
}",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        (
            "const foo = hey?.then(x => { console.log(x) })",
            Some(serde_json::json!([{ "ignoreLastCallback": true }])),
        ),
        ("hey.then(x => { invalid = x })", None),
        ("hey.then(x => { invalid['x'] = x })", None),
        (
            "hey.then(x => { wind[x] = x })",
            Some(serde_json::json!([{ "ignoreAssignmentVariable": ["window"] }])),
        ),
        (
            "hey.then(x => { wind['x'] = x })",
            Some(serde_json::json!([{ "ignoreAssignmentVariable": ["window"] }])),
        ),
        (
            "hey.then(x => { windows['x'] = x })",
            Some(serde_json::json!([{ "ignoreAssignmentVariable": ["window"] }])),
        ),
        (
            "hey.then(x => { x() })",
            Some(serde_json::json!([{ "ignoreAssignmentVariable": ["window"] }])),
        ),
    ];

    Tester::new(AlwaysReturn::NAME, AlwaysReturn::PLUGIN, pass, fail).test_and_snapshot();
}
