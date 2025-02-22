use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{self},
    config::GlobalValue,
    context::LintContext,
    rule::Rule,
};

fn no_eval_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eval can be harmful.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEval {
    allow_indirect: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows referencing the `eval` function. This rule is aimed at preventing
    /// potentially dangerous, unnecessary, and slow code by disallowing the use of
    /// the `eval()` function.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript’s `eval()` function is potentially dangerous and is often misused.
    /// Using `eval()` on untrusted code can open a program up to several different
    /// injection attacks. The use of `eval()` in most contexts can be substituted for
    /// a better, safer alternative approach to solving the problem, such as using
    /// `JSON.parse()` or `Function` constructors in safer ways.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const obj = { x: "foo" },
    ///   key = "x",
    ///   value = eval("obj." + key);
    ///
    /// (0, eval)("const a = 0");
    ///
    /// const foo = eval;
    /// foo("const a = 0");
    ///
    /// this.eval("const a = 0");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const obj = { x: "foo" },
    ///   key = "x",
    ///   value = obj[key];
    ///
    /// class A {
    ///   foo() {
    ///     this.eval("const a = 0");
    ///   }
    ///
    ///   eval() { }
    ///
    ///   static {
    ///     this.eval("const a = 0");
    ///   }
    ///
    ///   static eval() { }
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### allowIndirect
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// This `allowIndirect` option allows indirect `eval()` calls.
    ///
    /// Indirect calls to `eval`(e.g., `window['eval']`) are less dangerous
    /// than direct calls because they cannot dynamically change the scope.
    /// Indirect `eval()` calls also typically have less impact on performance
    /// compared to direct calls, as they do not invoke JavaScript's scope chain.
    ///
    /// Example:
    /// ```json
    /// "eslint/no-eval": [
    ///   "error",
    ///   { "allowIndirect": true }
    /// ]
    /// ```
    NoEval,
    eslint,
    correctness
);

impl Rule for NoEval {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_indirect = value.get(0).is_some_and(|config| {
            config.get("allowIndirect").and_then(serde_json::Value::as_bool).unwrap_or(false)
        });

        Self { allow_indirect }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_) if !self.allow_indirect => {
                let globals =
                    ["eval", "global", "window", "globalThis"].into_iter().filter(|name| {
                        ctx.get_global_variable_value(name)
                            .is_some_and(|var| var != GlobalValue::Off)
                    });

                for name in globals {
                    let Some(references) = ctx.scopes().root_unresolved_references().get(name)
                    else {
                        continue;
                    };

                    for reference_id in references {
                        let reference = ctx.symbols().get_reference(*reference_id);
                        let node = ctx.nodes().get_node(reference.node_id());
                        let mut parent = Self::outermost_mem_expr(node, ctx).unwrap();

                        if name == "eval" {
                            if !matches!(parent.kind(), AstKind::CallExpression(_)) {
                                ctx.diagnostic(no_eval_diagnostic(node.span()));
                            }
                        } else {
                            while let Some(mem_expr) = parent.kind().as_member_expression() {
                                if mem_expr.static_property_name().is_some_and(|p| p == name) {
                                    parent = Self::outermost_mem_expr(parent, ctx).unwrap();
                                } else {
                                    break;
                                }
                            }

                            let Some(mem_expr) = parent.kind().as_member_expression() else {
                                continue;
                            };
                            let Some((span, name)) = mem_expr.static_property_info() else {
                                continue;
                            };
                            if name == "eval" {
                                ctx.diagnostic(no_eval_diagnostic(span));
                            }
                        }
                    }
                }
            }
            AstKind::ThisExpression(_) if !self.allow_indirect => {
                let parent = ctx.nodes().parent_node(node.id()).unwrap();
                let Some(mem_expr) = parent.kind().as_member_expression() else { return };
                let Some((span, name)) = mem_expr.static_property_info() else { return };

                if name == "eval" {
                    let scope_id = ctx.scopes().ancestors(parent.scope_id()).find(|scope_id| {
                        let scope_flags = ctx.scopes().get_flags(*scope_id);
                        scope_flags.is_var() && !scope_flags.is_arrow()
                    });

                    let scope_id = scope_id.unwrap();
                    let scope_flags = ctx.scopes().get_flags(scope_id);

                    // The `TsModuleBlock` shouldn't be considered
                    if scope_flags.is_ts_module_block() {
                        return;
                    }

                    let root = ctx.nodes().get_node(ctx.nodes().root().unwrap());
                    let program = root.kind().as_program().unwrap();

                    let is_valid = if scope_flags.is_top() {
                        program.source_type.is_script()
                    } else {
                        let node = ctx.nodes().get_node(ctx.scopes().get_node_id(scope_id));
                        ast_util::is_default_this_binding(ctx, node, true)
                            && (!scope_flags.is_strict_mode() || scope_flags.is_arrow())
                    };

                    if is_valid {
                        ctx.diagnostic(no_eval_diagnostic(span));
                    }
                }
            }
            AstKind::CallExpression(call_expr) => {
                let is_valid = !self.allow_indirect || !call_expr.optional;
                if is_valid && call_expr.callee.is_specific_id("eval") {
                    ctx.diagnostic(no_eval_diagnostic(call_expr.callee.span()));
                }
            }
            _ => {}
        }
    }
}

impl NoEval {
    fn outermost_mem_expr<'a, 'b>(
        node: &'a AstNode<'b>,
        semantic: &'a LintContext<'b>,
    ) -> Option<&'a AstNode<'b>> {
        semantic.nodes().ancestors(node.id()).skip(1).find(|parent| {
            !matches!(
                parent.kind(),
                AstKind::ParenthesizedExpression(_) | AstKind::ChainExpression(_)
            )
        })
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    #[expect(clippy::unnecessary_wraps)]
    fn allow_indirect_with_true() -> Option<serde_json::Value> {
        Some(serde_json::json!([{ "allowIndirect": true }]))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn env_with_node() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "env": {
                "node": true
            }
        }))
    }

    #[expect(clippy::unnecessary_wraps)]
    fn env_with_browser() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "env": {
                "browser": true
            }
        }))
    }

    let pass = vec![
        ("Eval(foo)", None, None, None),
        ("setTimeout('foo')", None, None, None),
        ("setInterval('foo')", None, None, None),
        ("window.setTimeout('foo')", None, env_with_browser(), None),
        ("window.setInterval('foo')", None, env_with_browser(), None),
        ("window.eval('foo')", None, None, None),
        ("window.eval('foo')", None, None, Some(PathBuf::from("foo.cjs"))),
        ("window.noeval('foo')", None, env_with_browser(), None),
        (
            "function foo() { var eval = 'foo'; window[eval]('foo') }",
            None,
            env_with_browser(),
            None,
        ),
        ("global.eval('foo')", None, None, None),
        ("global.noeval('foo')", None, None, Some(PathBuf::from("foo.cjs"))),
        (
            "function foo() { var eval = 'foo'; global[eval]('foo') }",
            None,
            env_with_node(),
            Some(PathBuf::from("foo.cjs")),
        ),
        // ("globalThis.eval('foo')", None, None, None), // { "ecmaVersion": 2017 }, globalThis is not supported
        ("globalThis.noneval('foo')", None, None, None),
        (
            "function foo() { var eval = 'foo'; globalThis[eval]('foo') }",
            None,
            env_with_browser(),
            None,
        ),
        ("this.noeval('foo');", None, None, None),
        ("function foo() { 'use strict'; this.eval('foo'); }", None, None, None),
        ("'use strict'; this.eval('foo');", None, None, None), // { "parserOptions": { "ecmaFeatures": { "globalReturn": true } } },
        ("this.eval('foo');", None, None, None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function foo() { this.eval('foo'); }", None, None, None),
        ("var obj = {foo: function() { this.eval('foo'); }}", None, None, None),
        ("var obj = {}; obj.foo = function() { this.eval('foo'); }", None, None, None),
        ("() => { this.eval('foo') }", None, None, None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("function f() { 'use strict'; () => { this.eval('foo') } }", None, None, None), // { "ecmaVersion": 6 },
        ("(function f() { 'use strict'; () => { this.eval('foo') } })", None, None, None), // { "ecmaVersion": 6 },
        ("class C extends function () { this.eval('foo'); } {}", None, None, None), // { "ecmaVersion": 6 },
        ("class A { foo() { this.eval(); } }", None, None, None), // { "ecmaVersion": 6 },
        ("class A { static foo() { this.eval(); } }", None, None, None), // { "ecmaVersion": 6 },
        ("class A { field = this.eval(); }", None, None, None),   // { "ecmaVersion": 2022 },
        ("class A { field = () => this.eval(); }", None, None, None), // { "ecmaVersion": 2022 },
        ("class A { static { this.eval(); } }", None, None, None), // { "ecmaVersion": 2022 },
        (
            "array.findLast(function (x) { return this.eval.includes(x); }, { eval: ['foo', 'bar'] });",
            None,
            None,
            None,
        ),
        (
            "callbacks.findLastIndex(function (cb) { return cb(this.eval); }, this);",
            None,
            None,
            None,
        ),
        (
            "['1+1'].flatMap(function (str) { return this.eval(str); }, new Evaluator);",
            None,
            None,
            None,
        ),
        ("(0, eval)('foo')", allow_indirect_with_true(), None, None),
        ("(0, window.eval)('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("(0, window['eval'])('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("var EVAL = eval; EVAL('foo')", allow_indirect_with_true(), None, None),
        ("var EVAL = this.eval; EVAL('foo')", allow_indirect_with_true(), None, None),
        ("(function(exe){ exe('foo') })(eval);", allow_indirect_with_true(), None, None),
        ("window.eval('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("window.window.eval('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("window.window['eval']('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("global.eval('foo')", allow_indirect_with_true(), env_with_node(), None),
        ("global.global.eval('foo')", allow_indirect_with_true(), env_with_node(), None),
        ("this.eval('foo')", allow_indirect_with_true(), None, None),
        ("function foo() { this.eval('foo') }", allow_indirect_with_true(), None, None),
        ("(0, globalThis.eval)('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("(0, globalThis['eval'])('foo')", allow_indirect_with_true(), env_with_browser(), None),
        (
            "var EVAL = globalThis.eval; EVAL('foo')",
            allow_indirect_with_true(),
            env_with_browser(),
            None,
        ),
        (
            "function foo() { globalThis.eval('foo') }",
            allow_indirect_with_true(),
            env_with_browser(),
            None,
        ),
        (
            "globalThis.globalThis.eval('foo');",
            allow_indirect_with_true(),
            env_with_browser(),
            None,
        ),
        ("eval?.('foo')", allow_indirect_with_true(), None, None),
        ("window?.eval('foo')", allow_indirect_with_true(), env_with_browser(), None),
        ("(window?.eval)('foo')", allow_indirect_with_true(), env_with_browser(), None),
        (
            "sinon.test(/** @this sinon.Sandbox */function() { this.eval(); });",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ),
    ];

    let fail = vec![
        ("eval(foo)", None, None, None),
        ("eval('foo')", None, None, None),
        ("function foo(eval) { eval('foo') }", None, None, None),
        ("eval(foo)", allow_indirect_with_true(), None, None),
        ("eval('foo')", allow_indirect_with_true(), None, None),
        ("function foo(eval) { eval('foo') }", allow_indirect_with_true(), None, None),
        ("(0, eval)('foo')", None, None, None),
        ("(0, window.eval)('foo')", None, env_with_browser(), None),
        ("(0, window['eval'])('foo')", None, env_with_browser(), None),
        ("var EVAL = eval; EVAL('foo')", None, None, None),
        ("var EVAL = this.eval; EVAL('foo')", None, None, Some(PathBuf::from("foo.cjs"))),
        (
            "'use strict'; var EVAL = this.eval; EVAL('foo')",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ),
        (
            "function foo() { ('use strict'); this.eval; }",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ),
        ("() => { this.eval('foo'); }", None, None, Some(PathBuf::from("foo.cjs"))), // { "ecmaVersion": 6 },
        ("() => { 'use strict'; this.eval('foo'); }", None, None, Some(PathBuf::from("foo.cjs"))), // { "ecmaVersion": 6 },
        ("'use strict'; () => { this.eval('foo'); }", None, None, Some(PathBuf::from("foo.cjs"))), // { "ecmaVersion": 6 },
        (
            "() => { 'use strict'; () => { this.eval('foo'); } }",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ), // { "ecmaVersion": 6 },
        ("(function(exe){ exe('foo') })(eval);", None, None, None),
        ("window.eval('foo')", None, env_with_browser(), None),
        ("window.window.eval('foo')", None, env_with_browser(), None),
        ("window.window['eval']('foo')", None, env_with_browser(), None),
        ("global.eval('foo')", None, env_with_node(), None),
        ("global.global.eval('foo')", None, env_with_node(), None),
        ("global.global[`eval`]('foo')", None, env_with_node(), None), // { "ecmaVersion": 6, "sourceType": "commonjs" },
        ("this.eval('foo')", None, None, Some(PathBuf::from("foo.cjs"))),
        ("'use strict'; this.eval('foo')", None, None, Some(PathBuf::from("foo.cjs"))),
        ("function foo() { this.eval('foo') }", None, None, Some(PathBuf::from("foo.cjs"))),
        ("var EVAL = globalThis.eval; EVAL('foo')", None, env_with_browser(), None),
        ("globalThis.eval('foo')", None, env_with_browser(), None),
        ("globalThis.globalThis.eval('foo')", None, env_with_browser(), None),
        ("globalThis.globalThis['eval']('foo')", None, env_with_browser(), None),
        ("(0, globalThis.eval)('foo')", None, env_with_browser(), None),
        ("(0, globalThis['eval'])('foo')", None, env_with_browser(), None),
        ("window?.eval('foo')", None, env_with_browser(), None),
        ("(window?.eval)('foo')", None, env_with_browser(), None),
        ("(window?.window).eval('foo')", None, env_with_browser(), None),
        ("class C { [this.eval('foo')] }", None, None, Some(PathBuf::from("foo.cjs"))), // { "ecmaVersion": 2022 },
        (
            "'use strict'; class C { [this.eval('foo')] }",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ), // { "ecmaVersion": 2022 },
        ("class A { static {} [this.eval()]; }", None, None, Some(PathBuf::from("foo.cjs"))), // { "ecmaVersion": 2022 },
        // (
        //     "function foo() { 'use strict'; this.eval(); }",
        //     None,
        //     None,
        //     Some(PathBuf::from("foo.cjs")),
        // ), // { "ecmaVersion": 3 }, in es3, "use strict" directives do not apply
        (
            "array.findLast(x => this.eval.includes(x), { eval: 'abc' });",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ), // { "ecmaVersion": 2023 },
        ("callbacks.findLastIndex(function (cb) { return cb(eval); }, this);", None, None, None),
        (
            "['1+1'].flatMap(function (str) { return this.eval(str); });",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ),
        (
            "['1'].reduce(function (a, b) { return this.eval(a) ? a : b; }, '0');",
            None,
            None,
            Some(PathBuf::from("foo.cjs")),
        ),
    ];

    Tester::new(NoEval::NAME, NoEval::PLUGIN, pass, fail).test_and_snapshot();
}
