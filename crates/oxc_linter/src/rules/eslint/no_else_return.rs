use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoElseReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoElseReturn,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

impl Rule for NoElseReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { if (true) { if (false) { return x; } } else { return y; } }", None),
("function foo() { if (true) { return x; } return y; }", None),
("function foo() { if (true) { for (;;) { return x; } } else { return y; } }", None),
("function foo() { var x = true; if (x) { return x; } else if (x === false) { return false; } }", None),
("function foo() { if (true) notAReturn(); else return y; }", None),
("function foo() {if (x) { notAReturn(); } else if (y) { return true; } else { notAReturn(); } }", None),
("function foo() {if (x) { return true; } else if (y) { notAReturn() } else { notAReturn(); } }", None),
("if (0) { if (0) {} else {} } else {}", None),
("
			            function foo() {
			                if (foo)
			                    if (bar) return;
			                    else baz;
			                else qux;
			            }
			        ", None),
("
			            function foo() {
			                while (foo)
			                    if (bar) return;
			                    else baz;
			            }
			        ", None),
("function foo19() { if (true) { return x; } else if (false) { return y; } }", Some(serde_json::json!([{ "allowElseIf": true }]))),
("function foo20() {if (x) { return true; } else if (y) { notAReturn() } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": true }]))),
("function foo21() { var x = true; if (x) { return x; } else if (x === false) { return false; } }", Some(serde_json::json!([{ "allowElseIf": true }])))
    ];

    let fail = vec![
        ("function foo1() { if (true) { return x; } else { return y; } }", None),
("function foo2() { if (true) { var x = bar; return x; } else { var y = baz; return y; } }", None),
("function foo3() { if (true) return x; else return y; }", None),
("function foo4() { if (true) { if (false) return x; else return y; } else { return z; } }", None),
("function foo5() { if (true) { if (false) { if (true) return x; else { w = y; } } else { w = x; } } else { return z; } }", None),
("function foo6() { if (true) { if (false) { if (true) return x; else return y; } } else { return z; } }", None),
("function foo7() { if (true) { if (false) { if (true) return x; else return y; } return w; } else { return z; } }", None),
("function foo8() { if (true) { if (false) { if (true) return x; else return y; } else { w = x; } } else { return z; } }", None),
("function foo9() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", None),
("function foo9a() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo9b() {if (x) { return true; } if (y) { return true; } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo10() { if (foo) return bar; else (foo).bar(); }", None),
("function foo11() { if (foo) return bar 
			else { [1, 2, 3].map(foo) } }", None),
("function foo12() { if (foo) return bar 
			else { baz() } 
			[1, 2, 3].map(foo) }", None),
("function foo13() { if (foo) return bar; 
			else { [1, 2, 3].map(foo) } }", None),
("function foo14() { if (foo) return bar 
			else { baz(); } 
			[1, 2, 3].map(foo) }", None),
("function foo15() { if (foo) return bar; else { baz() } qaz() }", None),
("function foo16() { if (foo) return bar 
			else { baz() } qaz() }", None),
("function foo17() { if (foo) return bar 
			else { baz() } 
			qaz() }", None),
("function foo18() { if (foo) return function() {} 
			else [1, 2, 3].map(bar) }", None),
("function foo19() { if (true) { return x; } else if (false) { return y; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo20() {if (x) { return true; } else if (y) { notAReturn() } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo21() { var x = true; if (x) { return x; } else if (x === false) { return false; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo() { var a; if (bar) { return true; } else { var a; } }", None),
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", None),
("function foo() { var a; if (bar) { return true; } else { var a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { let a; if (bar) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("class foo { bar() { let a; if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { let a; if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() {let a; if (bar) { if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { const a = 1; if (bar) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { const a = 1; if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { let a; if (bar) { return true; } else { const a = 1 } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { let a; if (baz) { return true; } else { const a = 1; } } }", None), // { "ecmaVersion": 6 },
("function foo() { class a {}; if (bar) { return true; } else { const a = 1; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { class a {}; if (baz) { return true; } else { const a = 1; } } }", None), // { "ecmaVersion": 6 },
("function foo() { const a = 1; if (bar) { return true; } else { class a {} } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { const a = 1; if (baz) { return true; } else { class a {} } } }", None), // { "ecmaVersion": 6 },
("function foo() { var a; if (bar) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { var a; return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; }  while (baz) { var a; } }", None), // { "ecmaVersion": 6 },
("function foo(a) { if (bar) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo(a = 1) { if (bar) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo(a, b = a) { if (bar) { return true; } else { let a; }  if (bar) { return true; } else { let b; }}", None), // { "ecmaVersion": 6 },
("function foo(...args) { if (bar) { return true; } else { let args; } }", None), // { "ecmaVersion": 6 },
("function foo() { try {} catch (a) { if (bar) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { try {} catch (a) { if (bar) { if (baz) { return true; } else { let a; } } } }", None), // { "ecmaVersion": 6 },
("function foo() { try {} catch ({bar, a = 1}) { if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let arguments; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let arguments; } return arguments[0]; }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let arguments; } if (baz) { return arguments[0]; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let arguments; } } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; } a; }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; } if (baz) { a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } a; }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } if (quux) { a; } } }", None), // { "ecmaVersion": 6 },
("function a() { if (foo) { return true; } else { let a; } a(); }", None), // { "ecmaVersion": 6 },
("function a() { if (a) { return true; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function a() { if (foo) { return a; } else { let a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; } function baz() { a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } (() => a) } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; } var a; }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } var a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } var { a } = {}; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } if (quux) { var a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { var a; } }", None), // { "ecmaVersion": 6 },
("function foo() { if (quux) { var a; } if (bar) { if (baz) { return true; } else { let a; } } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else { let a; } function a(){} }", None), // { "ecmaVersion": 6 },
("function foo() { if (baz) { if (bar) { return true; } else { let a; } function a(){} } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { function a(){}  } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } function a(){} }", None), // { "ecmaVersion": 6 },
("function foo() { let a; if (bar) { return true; } else { function a(){} } }", None), // { "ecmaVersion": 6 },
("function foo() { var a; if (bar) { return true; } else { function a(){} } }", None), // { "ecmaVersion": 6 },
("function foo() { if (bar) { return true; } else function baz() {} };", None),
("if (foo) { return true; } else { let a; }", None), // { "ecmaVersion": 6, "sourceType": "commonjs" },
("let a; if (foo) { return true; } else { let a; }", None), // { "ecmaVersion": 6, "sourceType": "commonjs" }
    ];

    let fix = vec![
        ("function foo1() { if (true) { return x; } else { return y; } }", "function foo1() { if (true) { return x; }  return y;  }", None),
("function foo2() { if (true) { var x = bar; return x; } else { var y = baz; return y; } }", "function foo2() { if (true) { var x = bar; return x; }  var y = baz; return y;  }", None),
("function foo3() { if (true) return x; else return y; }", "function foo3() { if (true) return x; return y; }", None),
("function foo4() { if (true) { if (false) return x; else return y; } else { return z; } }", "function foo4() { if (true) { if (false) return x; return y; } else { return z; } }", None),
("function foo5() { if (true) { if (false) { if (true) return x; else { w = y; } } else { w = x; } } else { return z; } }", "function foo5() { if (true) { if (false) { if (true) return x;  w = y;  } else { w = x; } } else { return z; } }", None),
("function foo6() { if (true) { if (false) { if (true) return x; else return y; } } else { return z; } }", "function foo6() { if (true) { if (false) { if (true) return x; return y; } } else { return z; } }", None),
("function foo7() { if (true) { if (false) { if (true) return x; else return y; } return w; } else { return z; } }", "function foo7() { if (true) { if (false) { if (true) return x; return y; } return w; } else { return z; } }", None),
("function foo8() { if (true) { if (false) { if (true) return x; else return y; } else { w = x; } } else { return z; } }", "function foo8() { if (true) { if (false) { if (true) return x; return y; } else { w = x; } } else { return z; } }", None),
("function foo9() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", "function foo9() {if (x) { return true; } else if (y) { return true; }  notAReturn();  }", None),
("function foo9a() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", "function foo9a() {if (x) { return true; } if (y) { return true; } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo9b() {if (x) { return true; } if (y) { return true; } else { notAReturn(); } }", "function foo9b() {if (x) { return true; } if (y) { return true; }  notAReturn();  }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo10() { if (foo) return bar; else (foo).bar(); }", "function foo10() { if (foo) return bar; (foo).bar(); }", None),
("function foo13() { if (foo) return bar; 
			else { [1, 2, 3].map(foo) } }", "function foo13() { if (foo) return bar; 
			 [1, 2, 3].map(foo)  }", None),
("function foo14() { if (foo) return bar 
			else { baz(); } 
			[1, 2, 3].map(foo) }", "function foo14() { if (foo) return bar 
			 baz();  
			[1, 2, 3].map(foo) }", None),
("function foo17() { if (foo) return bar 
			else { baz() } 
			qaz() }", "function foo17() { if (foo) return bar 
			 baz()  
			qaz() }", None),
("function foo19() { if (true) { return x; } else if (false) { return y; } }", "function foo19() { if (true) { return x; } if (false) { return y; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo20() {if (x) { return true; } else if (y) { notAReturn() } else { notAReturn(); } }", "function foo20() {if (x) { return true; } if (y) { notAReturn() } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo21() { var x = true; if (x) { return x; } else if (x === false) { return false; } }", "function foo21() { var x = true; if (x) { return x; } if (x === false) { return false; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo() { var a; if (bar) { return true; } else { var a; } }", "function foo() { var a; if (bar) { return true; }  var a;  }", None),
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", "function foo() { if (bar) { var a; if (baz) { return true; }  var a;  } }", None),
("function foo() { var a; if (bar) { return true; } else { var a; } }", "function foo() { var a; if (bar) { return true; }  var a;  }", None),
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", "function foo() { if (bar) { var a; if (baz) { return true; }  var a;  } }", None),
("function foo() {let a; if (bar) { if (baz) { return true; } else { let a; } } }", "function foo() {let a; if (bar) { if (baz) { return true; }  let a;  } }", None),
("function foo() { try {} catch (a) { if (bar) { if (baz) { return true; } else { let a; } } } }", "function foo() { try {} catch (a) { if (bar) { if (baz) { return true; }  let a;  } } }", None),
("function foo() { if (bar) { return true; } else { let arguments; } }", "function foo() { if (bar) { return true; }  let arguments;  }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let arguments; } } }", "function foo() { if (bar) { if (baz) { return true; }  let arguments;  } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } a; }", "function foo() { if (bar) { if (baz) { return true; }  let a;  } a; }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { var a; } }", "function foo() { if (bar) { if (baz) { return true; }  let a;  } if (quux) { var a; } }", None),
("function foo() { if (quux) { var a; } if (bar) { if (baz) { return true; } else { let a; } } }", "function foo() { if (quux) { var a; } if (bar) { if (baz) { return true; }  let a;  } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { function a(){}  } }", "function foo() { if (bar) { if (baz) { return true; }  let a;  } if (quux) { function a(){}  } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } function a(){} }", "function foo() { if (bar) { if (baz) { return true; }  let a;  } function a(){} }", None),
("if (foo) { return true; } else { let a; }", "if (foo) { return true; }  let a; ", None)
    ];
    Tester::new(NoElseReturn::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
