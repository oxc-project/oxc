use oxc_ast::{
    ast::{
        match_member_expression, ChainElement, Expression, MemberExpression, MethodDefinitionKind,
        ObjectProperty, PropertyKind,
    },
    AstKind,
};
use oxc_cfg::{
    graph::{
        visit::{set_depth_first_search, Control, DfsEvent},
        Direction,
    },
    EdgeType, ErrorEdgeKind, InstructionKind, ReturnInstructionKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn getter_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to always return a value in getter.")
        .with_help("Return a value from all code paths in getter.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct GetterReturn {
    pub allow_implicit: bool,
}

const METHODS_TO_WATCH_FOR: [(&str, &str); 4] = [
    ("Object", "defineProperty"),
    ("Reflect", "defineProperty"),
    ("Object", "create"),
    ("Object", "defineProperties"),
];

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires all getters to have a `return` statement.
    ///
    /// ### Why is this bad?
    /// Getters should always return a value. If they don't, it's probably a mistake.
    ///
    /// This rule does not run on TypeScript files, since type checking will
    /// catch getters that do not return a value.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class Person {
    ///     get name() {
    ///         // no return
    ///     }
    /// }
    ///
    /// const obj = {
    ///     get foo() {
    ///         // object getter are also checked
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class Person {
    ///     get name() {
    ///         return this._name;
    ///     }
    /// }
    /// ```
    GetterReturn,
    eslint,
    nursery
);

impl Rule for GetterReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) if !func.is_typescript_syntax() => {
                self.run_diagnostic(node, ctx, func.span);
            }
            AstKind::ArrowFunctionExpression(expr) => {
                self.run_diagnostic(node, ctx, expr.span);
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_implicit = value
            .get(0)
            .and_then(|config| config.get("allowImplicit"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_implicit }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // https://eslint.org/docs/latest/rules/getter-return#handled_by_typescript
        !ctx.source_type().is_typescript()
    }
}

impl GetterReturn {
    fn handle_member_expression<'a>(member_expression: &'a MemberExpression<'a>) -> bool {
        for (a, b) in METHODS_TO_WATCH_FOR {
            if member_expression.through_optional_is_specific_member_access(a, b) {
                return true;
            }
        }

        false
    }

    fn handle_actual_expression<'a>(callee: &'a Expression<'a>) -> bool {
        match callee.without_parentheses() {
            expr @ match_member_expression!(Expression) => {
                Self::handle_member_expression(expr.to_member_expression())
            }
            Expression::ChainExpression(ce) => match &ce.expression {
                match_member_expression!(ChainElement) => {
                    Self::handle_member_expression(ce.expression.to_member_expression())
                }
                ChainElement::CallExpression(_) | ChainElement::TSNonNullExpression(_) => {
                    false // todo: make a test for this
                }
            },
            _ => false,
        }
    }

    fn handle_paren_expr<'a>(expr: &'a Expression<'a>) -> bool {
        match expr.without_parentheses() {
            Expression::CallExpression(ce) => Self::handle_actual_expression(&ce.callee),
            _ => false,
        }
    }

    /// Checks whether it is necessary to check the node
    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> Option<bool> {
        let parent = ctx.nodes().parent_node(node.id())?;
        match parent.kind() {
            AstKind::MethodDefinition(mdef) => {
                if matches!(mdef.kind, MethodDefinitionKind::Get) {
                    return Some(true);
                }
            }
            AstKind::ObjectProperty(ObjectProperty { kind, key: prop_key, .. }) => {
                if matches!(kind, PropertyKind::Get) {
                    return Some(true);
                }
                if prop_key.name().is_some_and(|key| key != "get") {
                    return Some(false);
                }

                let parent_2 = ctx.nodes().parent_node(parent.id())?;
                let parent_3 = ctx.nodes().parent_node(parent_2.id())?;
                let parent_4 = ctx.nodes().parent_node(parent_3.id())?;
                // handle (X())
                match parent_4.kind() {
                    AstKind::ParenthesizedExpression(p) => {
                        if Self::handle_paren_expr(&p.expression) {
                            return Some(true);
                        }
                    }
                    AstKind::CallExpression(ce) => {
                        if Self::handle_actual_expression(&ce.callee) {
                            return Some(true);
                        }
                    }
                    _ => {}
                }

                let parent_5 = ctx.nodes().parent_node(parent_4.id())?;
                let parent_6 = ctx.nodes().parent_node(parent_5.id())?;
                match parent_6.kind() {
                    AstKind::ParenthesizedExpression(p) => {
                        if Self::handle_paren_expr(&p.expression) {
                            return Some(true);
                        }
                    }
                    AstKind::CallExpression(ce) => {
                        if Self::handle_actual_expression(&ce.callee) {
                            return Some(true);
                        }
                    }
                    _ => {
                        return Some(false);
                    }
                };
            }
            _ => {}
        }

        Some(false)
    }

    fn run_diagnostic<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>, span: Span) {
        if !Self::is_wanted_node(node, ctx).unwrap_or_default() {
            return;
        }

        let cfg = ctx.cfg();

        let graph = cfg.graph();
        let definitely_returns_in_all_codepaths = 'returns: {
            // The expression is the equivalent of return.
            // Therefore, if a function is an expression, it always returns its value.
            //
            // Example expression:
            // ```js
            // const fn = () => 1;
            // ```
            if let AstKind::ArrowFunctionExpression(arrow_expr) = node.kind() {
                if arrow_expr.expression {
                    break 'returns true;
                }
                // If the signature of function supports the return of the `undefined` value,
                // you do not need to check this rule
                if let AstKind::Function(func) = node.kind() {
                    if let Some(ret) = &func.return_type {
                        if ret.type_annotation.is_maybe_undefined() {
                            break 'returns true;
                        }
                    }
                }
            }
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
                        let return_instruction =
                            cfg.basic_block(basic_block_id).instructions().iter().find(|it| {
                                match it.kind {
                                    // Throws are classified as returning.
                                    InstructionKind::Return(_) | InstructionKind::Throw => true,

                                    // Ignore irrelevant elements.
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
                            !matches! { ret.kind,
                            InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined)
                                if !self.allow_implicit
                            }
                        });

                        // Return true as the second argument to signify we should
                        // continue walking this branch, as we haven't seen anything
                        // that will signify to us that this path of the program will
                        // definitely return or throw.
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

            output.break_value().is_none()
        };

        if !definitely_returns_in_all_codepaths {
            ctx.diagnostic(getter_return_diagnostic(span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        ("var foo = { get bar(){return true;} };", None),
        ("var foo = { get bar() {return;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var foo = { get bar(){return true;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var foo = { get bar(){if(bar) {return;} return true;} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("class foo { get bar(){return true;} }", None),
        ("class foo { get bar(){if(baz){return true;} else {return false;} } }", None),
        ("class foo { get(){return true;} }", None),
        ("class foo { get bar(){return true;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("class foo { get bar(){return;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        ("Object.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});", None),
        ("Object.defineProperty(foo, \"bar\", { set: function () {}});", None),
        ("Object.defineProperty(foo, \"bar\", { set: () => {}});", None),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", None),
        ("Object.defineProperties(foo, { bar: { get: function () { ~function (){ return true; }(); return true;}} });", None),
        ("Object.defineProperties(foo, { bar: { set: function () {}} });", None),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});", None),
        ("Object.create(foo, { bar: { get() {return true;} } });", None),
        ("Object.create(foo, { bar: { get: function () {return true;} } });", None),
        ("Object.create(foo, { bar: { get: () => {return true;} } });", None),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function (){return;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperties(foo, { bar: { get: function () {return;}} });", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("var get = function(){};", None),
        ("var get = function(){ return true; };", None),
        ("var foo = { bar(){} };", None),
        ("var foo = { bar(){ return true; } };", None),
        ("var foo = { bar: function(){} };", None),
        ("var foo = { bar: function(){return;} };", None),
        ("var foo = { bar: function(){return true;} };", None),
        ("var foo = { get: function () {} }", None),
        ("var foo = { get: () => {}};", None),
        ("class C { get; foo() {} }", None),
        ("foo.defineProperty(null, { get() {} });", None),
        ("foo.defineProperties(null, { bar: { get() {} } });", None),
        ("foo.create(null, { bar: { get() {} } });", None),
        ("var foo = { get willThrowSoValid() { throw MyException() } };", None),
        (
            "const originalClearTimeout = targetWindow.clearTimeout;
        Object.defineProperty(targetWindow, 'vscodeOriginalClearTimeout', { get: () => originalClearTimeout });
        ",
            None,
        ),
        (r"
        var foo = {
                get bar() {
                        let name = ([] || [])[1];
                        return name;
                },
        };
        ", None),
        ("var foo = { get bar() { try { return a(); } finally {  } } };", None),
        ("
        var foo = {
            get bar() {
                switch (baz) {
                    case VS_LIGHT_THEME: return a;
                    case VS_HC_THEME: return b;
                    case VS_HC_LIGHT_THEME: return c;
                    default: return d;
                }
            }
        };
        ", None),
        // adapted from: https://github.com/1024pix/pix/blob/1352bd8d7f6070f1ff8da79867f543c1c1926e59/mon-pix/app/components/progress-bar.js#L29-L43
        ("
        export default class ProgressBar extends Component {
            get steps() {
                const steps = [];

                for (let i = 0; i < this.maxStepsNumber; i++) {
                    steps.push({
                        stepnum: i + 1,
                    });
                }

                return steps;
            }
        }", None),
        ("
        var foo = {
            get bar() {
                for (let i = 0; i<10; i++) {
                    if (i === 5) {
                        return i;
                    }
                }
                return 0;
            }
        }", None),
        ("
        var foo = {
            get bar() {
                let i = 0;
                while (i < 10) {
                    if (i === 5) {
                        return i;
                    }
                    i++;
                }
                return 0;
            }
        }", None),
    ];

    let fail = vec![
        ("var foo = { get bar() {} };", None),
        ("var foo = { get\n bar () {} };", None),
        ("var foo = { get bar(){if(baz) {return true;}} };", None),
        ("var foo = { get bar() { ~function () {return true;}} };", None),
        ("var foo = { get bar() { return; } };", None),
        ("var foo = { get bar() {} };", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "var foo = { get bar() {if (baz) {return;}} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){} }", None),
        ("var foo = class {\n  static get\nbar(){} }", None),
        ("class foo { get bar(){ if (baz) { return true; }}}", None),
        ("class foo { get bar(){ ~function () { return true; }()}}", None),
        ("class foo { get bar(){} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        (
            "class foo { get bar(){if (baz) {return true;} } }",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get: function getfoo (){}});", None),
        ("Object.defineProperty(foo, 'bar', { get(){} });", None),
        ("Object.defineProperty(foo, 'bar', { get: () => {}});", None),
        ("Object.defineProperty(foo, \"bar\", { get: function (){if(bar) {return true;}}});", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){ ~function () { return true; }()}});",
            None,
        ),
        ("Reflect.defineProperty(foo, 'bar', { get: function (){}});", None),
        ("Object.create(foo, { bar: { get: function() {} } })", None),
        ("Object.create(foo, { bar: { get() {} } })", None),
        ("Object.create(foo, { bar: { get: () => {} } })", None),
        (
            "Object.defineProperties(foo, { bar: { get: function () {}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function (){if(bar) {return true;}}}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {~function () { return true; }()}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.create(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function (){}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("Object?.defineProperty(foo, 'bar', { get: function (){} });", None),
        ("(Object?.defineProperty)(foo, 'bar', { get: function (){} });", None),
        (
            "Object?.defineProperty(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.defineProperty)(foo, 'bar', { get: function (){} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "(Object?.create)(foo, { bar: { get: function (){} } });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("var foo = { get bar() { try { return a(); } catch {} } };", None),
        ("var foo = { get bar() { try { return a(); } catch {  } finally {  } } };", None),
        (
            "
        var foo = {
            get bar() {
                for (let i = 0; i<10; i++) {
                    return i;
                }
            }
        }",
            None,
        ),
        (
            "
        var foo = {
            get bar() {
                let i = 0;
                while (i < 10) {
                    return i;
                }
            }
        }",
            None,
        ),
    ];

    Tester::new(GetterReturn::NAME, GetterReturn::PLUGIN, pass, fail)
        .change_rule_path_extension("js")
        .test_and_snapshot();

    // TypeScript tests
    let pass = vec![(
        "var foo = {
            get bar(): boolean | undefined {
                if (Math.random() > 0.5) {
                    return true;
                }
            }
        };",
        None,
    )];

    let fail = vec![];

    Tester::new(GetterReturn::NAME, GetterReturn::PLUGIN, pass, fail).test();
}
