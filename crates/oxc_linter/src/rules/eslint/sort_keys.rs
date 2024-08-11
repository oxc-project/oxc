use oxc_ast::{ast::VariableDeclarationKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

#[derive(Debug, Default, Clone)]
pub struct SortKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    SortKeys,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for SortKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(dec) = node.kind() {
            if dec.kind == VariableDeclarationKind::Var {
                ctx.diagnostic(no_var_diagnostic(Span::new(dec.span.start, dec.span.start + 3)));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var obj = {'':1, [``]:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {[``]:1, '':2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {'':1, a:2}", Some(serde_json::json!([]))),
        ("var obj = {[``]:1, a:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {_:2, a:1, b:3} // default", Some(serde_json::json!([]))),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!([]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!([]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!([]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!([]))),
        ("var obj = {1:1, '11':2, 2:4, A:3}", Some(serde_json::json!([]))),
        ("var obj = {'#':1, 'Z':2, À:3, è:4}", Some(serde_json::json!([]))),
        ("var obj = { [/(?<zero>0)/]: 1, '/(?<zero>0)/': 2 }", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, b:3, [a + b]: -1, c:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {'':1, [f()]:2, a:3}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, [b++]:2, '':3}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, ...z, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {b:1, ...z, a:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...a, b:1, ...c, d:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...a, b:1, ...d, ...c, e:2, z:5}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {b:1, ...c, ...d, e:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, ...z, '':2}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {'':1, ...z, 'a':2}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...c, a:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, b:1, ...z}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...x, a:1, ...c, ...d, f:5, e:4}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("function fn(...args) { return [...args].length; }", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        (
            "function g() {}; function f(...args) { return g(...args); }",
            Some(serde_json::json!([])),
        ), // { "ecmaVersion": 2018 },
        ("let {a, b} = {}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, b:{x:1, y:1}, c:1}", Some(serde_json::json!([]))),
        ("var obj = {_:2, a:1, b:3} // asc", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!(["asc"]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {1:1, '11':2, 2:4, A:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {'#':1, 'Z':2, À:3, è:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "minKeys": 4 }]))),
        (
            "var obj = {_:2, a:1, b:3} // asc, insensitive",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc", { "caseSensitive": false }]))),
        (
            "var obj = {a:2, b:3, b_:1}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, C:3, c:2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:3, C:2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, '11':2, 2:4, A:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false, "minKeys": 5 }])),
        ),
        (
            "var obj = {_:2, a:1, b:3} // asc, natural",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "minKeys": 4 }])),
        ),
        (
            "var obj = {_:2, a:1, b:3} // asc, natural, insensitive",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, b:3, c:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:2, b:3, b_:1}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, C:3, c:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:3, C:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["asc", { "natural": true, "caseSensitive": false, "minKeys": 4 }]),
            ),
        ),
        ("var obj = {b:3, a:1, _:2} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {c:2, b:3, a:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, b:3, a:2}", Some(serde_json::json!(["desc"]))),
        ("var obj = {c:2, b_:1, C:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:4, _:2, A:3, $:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {A:3, 2:4, '11':2, 1:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {è:4, À:3, 'Z':2, '#':1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc", { "minKeys": 4 }]))),
        (
            "var obj = {b:3, a:1, _:2} // desc, insensitive",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, b:3, a:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, b:3, a:2}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, C:3, b_:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {C:2, c:3, b_:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:4, A:3, _:2, $:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {A:3, 2:4, '11':2, 1:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false, "minKeys": 5 }])),
        ),
        (
            "var obj = {b:3, a:1, _:2} // desc, natural",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        ("var obj = {c:2, b:3, a:1}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, b:3, a:2}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {c:2, b_:1, C:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {a:4, A:3, _:2, $:1}", Some(serde_json::json!(["desc", { "natural": true }]))),
        (
            "var obj = {A:3, '11':2, 2:4, 1:1}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "minKeys": 4 }])),
        ),
        (
            "var obj = {b:3, a:1, _:2} // desc, natural, insensitive",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, b:3, a:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, b:3, a:2}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, C:3, b_:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {C:2, c:3, b_:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:4, A:3, _:2, $:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {A:3, '11':2, 2:4, 1:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["desc", { "natural": true, "caseSensitive": false, "minKeys": 4 }]),
            ),
        ),
        (
            "
			                var obj = {
			                    e: 1,
			                    f: 2,
			                    g: 3,
			
			                    a: 4,
			                    b: 5,
			                    c: 6
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
			                var obj = {
			                    b: 1,
			
			                    // comment
			                    a: 2,
			                    c: 3
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
			                var obj = {
			                    b: 1
			
			                    ,
			
			                    // comment
			                    a: 2,
			                    c: 3
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
			                var obj = {
			                    c: 1,
			                    d: 2,
			
			                    b() {
			                    },
			                    e: 4
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    c: 1,
			                    d: 2,
			                    // comment
			
			                    // comment
			                    b() {
			                    },
			                    e: 4
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                  b,
			
			                  [a+b]: 1,
			                  a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    c: 1,
			                    d: 2,
			
			                    a() {
			
			                    },
			
			                    // abce
			                    f: 3,
			
			                    /*
			
			                    */
			                    [a+b]: 1,
			                    cc: 1,
			                    e: 2
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"
			                var obj = {
			                    b: "/*",
			
			                    a: "*/",
			                }
			            "#,
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
			                var obj = {
			                    b,
			                    /*
			                    */ //
			
			                    a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    b,
			
			                    /*
			                    */ //
			                    a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    b: 1
			
			                    ,a: 2
			                };
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    b: 1
			                // comment before comma
			
			                ,
			                a: 2
			                };
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                  b,
			
			                  a,
			                  ...z,
			                  c
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "
			                var obj = {
			                  b,
			
			                  [foo()]: [
			
			                  ],
			                  a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 }
    ];

    let fail = vec![
        ("var obj = {a:1, '':2} // default", None),
        ("var obj = {a:1, [``]:2} // default", None), // { "ecmaVersion": 6 },
        ("var obj = {a:1, _:2, b:3} // default", None),
        ("var obj = {a:1, c:2, b:3}", None),
        ("var obj = {b_:1, a:2, b:3}", None),
        ("var obj = {b_:1, c:2, C:3}", None),
        ("var obj = {$:1, _:2, A:3, a:4}", None),
        ("var obj = {1:1, 2:4, A:3, '11':2}", None),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", None),
        ("var obj = { null: 1, [/(?<zero>0)/]: 2 }", None), // { "ecmaVersion": 2018 },
        ("var obj = {...z, c:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...c, d:4, b:1, ...y, ...f, e:2, a:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {c:1, b:1, ...a}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...a, c:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, b:1, a:1, ...d, ...c}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:2, b:0, ...x, ...c}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:2, b:0, ...x}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, '':1, a:2}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, [b+c]:2, '':3}", None),                           // { "ecmaVersion": 6 },
        ("var obj = {'':1, [b+c]:2, a:3}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {b:1, [f()]:2, '':3, a:4}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, b:3, [a]: -1, c:2}", None), // { "ecmaVersion": 6 },
        ("var obj = {a:1, c:{y:1, x:1}, b:1}", None),
        ("var obj = {a:1, _:2, b:3} // asc", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {1:1, 2:4, A:3, '11':2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, _:2, b:3}", Some(serde_json::json!(["asc", { "minKeys": 3 }]))),
        (
            "var obj = {a:1, _:2, b:3} // asc, insensitive",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "caseSensitive": false }]))),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false, "minKeys": 3 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // asc, natural",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "minKeys": 2 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // asc, natural, insensitive",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, '11':2, 2:4, A:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["asc", { "natural": true, "caseSensitive": false, "minKeys": 3 }]),
            ),
        ),
        ("var obj = {'':1, a:'2'} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {[``]:1, a:'2'} // desc", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, _:2, b:3} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["desc"]))),
        ("var obj = {1:1, 2:4, A:3, '11':2}", Some(serde_json::json!(["desc"]))),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, _:2, b:3}", Some(serde_json::json!(["desc", { "minKeys": 3 }]))),
        (
            "var obj = {a:1, _:2, b:3} // desc, insensitive",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:2, C:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false, "minKeys": 2 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // desc, natural",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["desc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "minKeys": 3 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // desc, natural, insensitive",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:2, C:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["desc", { "natural": true, "caseSensitive": false, "minKeys": 2 }]),
            ),
        ),
        (
            "
			                var obj = {
			                    b: 1,
			                    c: 2,
			                    a: 3
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": false }])),
        ),
        (
            "
			                let obj = {
			                    b
			
			                    ,a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                 var obj = {
			                    b: 1,
			                    c () {
			
			                    },
			                    a: 3
			                  }
			             ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                 var obj = {
			                    a: 1,
			                    b: 2,
			
			                    z () {
			
			                    },
			                    y: 3
			                  }
			             ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                 var obj = {
			                    b: 1,
			                    c () {
			                    },
			                    // comment
			                    a: 3
			                  }
			             ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                  b,
			                  [a+b]: 1,
			                  a // sort-keys: 'a' should be before 'b'
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    c: 1,
			                    d: 2,
			                    // comment
			                    // comment
			                    b() {
			                    },
			                    e: 4
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                var obj = {
			                    c: 1,
			                    d: 2,
			
			                    z() {
			
			                    },
			                    f: 3,
			                    /*
			
			
			                    */
			                    [a+b]: 1,
			                    b: 1,
			                    e: 2
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"
			                var obj = {
			                    b: "/*",
			                    a: "*/",
			                }
			            "#,
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
			                var obj = {
			                    b: 1
			                    // comment before comma
			                    , a: 2
			                };
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
			                let obj = {
			                  b,
			                  [foo()]: [
			                  // ↓ this blank is inside a property and therefore should not count
			
			                  ],
			                  a
			                }
			            ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 }
    ];

    Tester::new(SortKeys::NAME, pass, fail).test_and_snapshot();
}
