use itertools::Itertools;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeId, ScopeTree};
use oxc_span::{Atom, GetSpan, Span};
use oxc_ast::{
  ast::{BlockStatement, IfStatement, Statement}, AstKind
};
use crate::{context::LintContext, fixer::{Fix, RuleFixer}, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoElseReturn {
  allow_else_if: bool
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
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

fn is_safe_from_name_collisions<'a>(ctx: &LintContext<'a>, stmt: &Statement, parent_scope_id: ScopeId) -> bool {
  let scopes: &ScopeTree = ctx.scopes();

  
  match stmt {
    Statement::BlockStatement(block) => {
      let block_scope_id = block.scope_id.get().unwrap();

      let bindings = scopes.get_bindings(block_scope_id);
      let parent_bindings: Vec<_> = scopes.get_bindings(parent_scope_id)
        .iter()
        .filter(|(_, symbol_id)| {
          bindings.iter().find(|(_, bindings_symbol_id)| symbol_id == bindings_symbol_id).is_none()
        })
        .collect();
        
      if bindings.iter().any(|(name, _)| {
        parent_bindings.iter().any(|(parent_name, _)| name == parent_name)
      }) {
        return false;
      }
      
      true
    },
    Statement::FunctionDeclaration(_) => false,
    _ => true
  }
}

fn no_else_return_diagnostic(stmt: &Statement) -> OxcDiagnostic{
  OxcDiagnostic::warn("eslint(no-else-return): Disallow `else` blocks after `return` statements in `if` statements")
      .with_label(stmt.span())
}

fn no_else_return_diagnostic_fix<'a>(ctx: &LintContext<'a>, stmt: &Statement, parent_scope_id: ScopeId) {
  if !is_safe_from_name_collisions(ctx, stmt, parent_scope_id) {
    return ctx.diagnostic(no_else_return_diagnostic(stmt));
  };
  ctx.diagnostic_with_fix(
    no_else_return_diagnostic(stmt),
    |fixer| {      
      fixer.replace(Span::new(0,0), "")
  })
}

fn check_for_return(node: &Statement) -> bool {
  match node {
    Statement::ReturnStatement(_) => true,
    _ => false
  }
}

fn naive_has_return(node: &Statement) -> bool {
  match node {
    Statement::BlockStatement(block) => {
      let last_child = block.body.last();
      match last_child {
        Some(node) => check_for_return(node),
        None => false
      }
    },
    node => check_for_return(node)
  }
}

fn check_for_return_or_if(node: &Statement) -> bool {
  match node {
    Statement::ReturnStatement(_) => true,
    Statement::IfStatement(if_stmt) => {
      let Some(alternate) = &if_stmt.alternate else {
        return false;
      };
      naive_has_return(&alternate) && naive_has_return(&if_stmt.consequent)
    }
    _ => false
  }

}

fn always_returns(stmt: &Statement) -> bool {
  match stmt {
    Statement::BlockStatement(block) => block.body.iter().any(check_for_return_or_if),
    node => check_for_return_or_if(node)
  }
}

fn check_if_with_else<'a>(ctx: &LintContext<'a>, if_stmt: &IfStatement<'a>,  parent_scope_id: ScopeId) {
  let Some(alternate) = &if_stmt.alternate else {
    return;
  };

  if always_returns(&if_stmt.consequent) {
    no_else_return_diagnostic_fix(ctx, alternate, parent_scope_id);
  }
}

fn check_if_without_else<'a>(ctx: &LintContext<'a>, if_stmt: &IfStatement<'a>, parent_scope_id: ScopeId) {
  let mut consequents: Vec<&Statement> = Vec::new();
  let mut current_node = if_stmt;
  let mut last_alternate: &Statement;

  loop {
    let Some(alternate) = &current_node.alternate else {
      return;
    };
    consequents.push(&current_node.consequent);
    last_alternate = alternate;
    match alternate {
      Statement::IfStatement(if_stmt) => {
        current_node = if_stmt;
      },
      _ => break,
    }
  }

  
  if consequents.iter().all(|stmt| always_returns(stmt)) {
    no_else_return_diagnostic_fix(ctx, last_alternate, parent_scope_id);
  }
}

fn is_in_statement_list_parents(node: &AstNode) -> bool{
  match node.kind() {
    AstKind::Program(_) => true,
    AstKind::BlockStatement(_) => true,
    AstKind::StaticBlock(_) => true,
    AstKind::SwitchCase(_) => true,
    AstKind::FunctionBody(_) => true,
    _ => false
  }
}

impl Rule for NoElseReturn {
  fn from_configuration(value: serde_json::Value) -> Self {
    let Some(value) = value.get(0) else { 
      return Self {
        allow_else_if: true
      } 
    };
    Self {
      allow_else_if: value.get("allowElseIf").and_then(serde_json::Value::as_bool).unwrap_or(true),
    }
  }

  fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::IfStatement(if_stmt) = node.kind() else {
      return;
    };

    let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
      return;
    };

    let parent_scope_id = parent_node.scope_id();
    // println!("{:?}", ctx.source_range(parent_node.kind().span()));

    if !is_in_statement_list_parents(parent_node) {
      return;
    }

    if self.allow_else_if { 
      check_if_without_else(ctx, if_stmt, parent_scope_id);
    } else {
      check_if_with_else(ctx, if_stmt, parent_scope_id);
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
