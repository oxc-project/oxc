use crate::{context::LintContext, rule::Rule, AstNode};
use cow_utils::CowUtils;
use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeId, ScopeTree};
use oxc_span::{GetSpan, Span};

#[derive(Debug, Default, Clone)]
pub struct NoElseReturn {
    allow_else_if: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow `else` blocks after `return` statements in `if` statements
    ///
    /// ### Why is this bad?
    /// If an if block contains a return statement, the else block becomes unnecessary. Its contents can be placed outside of the block.
    ///
    /// ### Example
    /// ```javascript
    /// function foo() {
    ///   if (x) {
    ///     return y;
    ///   } else {
    ///     return z;
    ///   }
    /// }
    /// ```
    NoElseReturn,
    suspicious,
    fix
);

fn no_else_return_diagnostic(else_stmt: &Statement) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow `else` blocks after `return` statements in `if` statements")
        .with_help("Unnecessary 'else' after 'return'.")
        .with_label(else_stmt.span())
}

fn is_safe_from_name_collisions(
    ctx: &LintContext,
    stmt: &Statement,
    parent_scope_id: ScopeId,
) -> bool {
    let scopes: &ScopeTree = ctx.scopes();

    match stmt {
        Statement::BlockStatement(block) => {
            let block_scope_id = block.scope_id.get().unwrap();
            let bindings = scopes.get_bindings(block_scope_id);
            let parent_bindings = scopes.get_bindings(parent_scope_id);

            if bindings.iter().any(|(name, symbol_id)| {
                let Some((parent_name, parent_symbol_id)) = parent_bindings.get_key_value(name)
                else {
                    return false;
                };
                parent_name == name && symbol_id != parent_symbol_id
            }) {
                return false;
            }

            true
        }
        Statement::FunctionDeclaration(_) => false,
        _ => true,
    }
}

fn replace_block(s: &str) -> String {
    if s.starts_with('{') && s.ends_with('}') && s.len() > 1 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn no_else_return_diagnostic_fix(
    ctx: &LintContext,
    else_stmt_prev: &Statement,
    else_stmt: &Statement,
    if_block_node: &AstNode,
) {
    let parent_scope_id = if_block_node.scope_id();

    if !is_safe_from_name_collisions(ctx, else_stmt, parent_scope_id) {
        return ctx.diagnostic(no_else_return_diagnostic(else_stmt));
    };

    let prev_span = else_stmt_prev.span();
    let span = else_stmt.span();

    let else_code = ctx.source_range(span);
    let else_code_prev_token = ctx
        .source_range(Span::new(prev_span.end, span.end))
        .cow_replacen("else ", "", 1)
        .cow_replace(else_code, "")
        .to_string();
    let fix_else_code = else_code_prev_token + &replace_block(else_code);

    ctx.diagnostic_with_fix(no_else_return_diagnostic(else_stmt), |fixer| {
        fixer.replace(Span::new(prev_span.end, span.end), fix_else_code)
    });
}

fn naive_has_return(node: &Statement) -> bool {
    match node {
        Statement::BlockStatement(block) => {
            let Some(last_child) = block.body.last() else {
                return false;
            };
            matches!(last_child, Statement::ReturnStatement(_))
        }
        Statement::ReturnStatement(_) => true,
        _ => false,
    }
}

fn check_for_return_or_if(node: &Statement) -> bool {
    match node {
        Statement::ReturnStatement(_) => true,
        Statement::IfStatement(if_stmt) => {
            let Some(alternate) = &if_stmt.alternate else {
                return false;
            };
            naive_has_return(alternate) && naive_has_return(&if_stmt.consequent)
        }
        _ => false,
    }
}

fn always_returns(stmt: &Statement) -> bool {
    match stmt {
        Statement::BlockStatement(block) => block.body.iter().any(check_for_return_or_if),
        node => check_for_return_or_if(node),
    }
}

fn check_if_with_else(ctx: &LintContext, node: &AstNode) {
    let AstKind::IfStatement(if_stmt) = node.kind() else {
        return;
    };
    let Some(alternate) = &if_stmt.alternate else {
        return;
    };

    if always_returns(&if_stmt.consequent) {
        no_else_return_diagnostic_fix(ctx, &if_stmt.consequent, alternate, node);
    }
}

fn check_if_without_else(ctx: &LintContext, node: &AstNode) {
    let AstKind::IfStatement(if_stmt) = node.kind() else {
        return;
    };
    let mut current_node = if_stmt;
    let mut last_alternate;
    let mut last_alternate_prev;

    loop {
        let Some(alternate) = &current_node.alternate else {
            return;
        };
        if !always_returns(&current_node.consequent) {
            return;
        }
        last_alternate_prev = &current_node.consequent;
        last_alternate = alternate;
        match alternate {
            Statement::IfStatement(if_stmt) => {
                current_node = if_stmt;
            }
            _ => break,
        }
    }

    no_else_return_diagnostic_fix(ctx, last_alternate_prev, last_alternate, node);
}

impl Rule for NoElseReturn {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(value) = value.get(0) else { return Self { allow_else_if: true } };
        Self {
            allow_else_if: value
                .get("allowElseIf")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(_) = node.kind() else {
            return;
        };

        let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        if !matches!(
            parent_node.kind(),
            AstKind::Program(_)
                | AstKind::BlockStatement(_)
                | AstKind::StaticBlock(_)
                | AstKind::SwitchCase(_)
                | AstKind::FunctionBody(_)
        ) {
            return;
        }
        if self.allow_else_if {
            check_if_without_else(ctx, node);
        } else {
            check_if_with_else(ctx, node);
        }
    }
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
("function foo4() { if (true) { if (false) return x; else return y; } else { return z; } }", "function foo4() { if (true) { if (false) return x; return y; }  return z;  }", None),
("function foo5() { if (true) { if (false) { if (true) return x; else { w = y; } } else { w = x; } } else { return z; } }", "function foo5() { if (true) { if (false) { if (true) return x;  w = y;  } else { w = x; } } else { return z; } }", None),
("function foo6() { if (true) { if (false) { if (true) return x; else return y; } } else { return z; } }", "function foo6() { if (true) { if (false) { if (true) return x; return y; } } else { return z; } }", None),
("function foo7() { if (true) { if (false) { if (true) return x; else return y; } return w; } else { return z; } }", "function foo7() { if (true) { if (false) { if (true) return x; return y; } return w; }  return z;  }", None),
("function foo8() { if (true) { if (false) { if (true) return x; else return y; } else { w = x; } } else { return z; } }", "function foo8() { if (true) { if (false) { if (true) return x; return y; }  w = x;  } else { return z; } }", None),
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
