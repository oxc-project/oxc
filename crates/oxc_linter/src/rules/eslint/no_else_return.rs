use crate::{context::LintContext, rule::Rule, AstNode};
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
    /// If an `if` block contains a `return` statement, the `else` block becomes
    /// unnecessary. Its contents can be placed outside of the block.
    ///
    /// ```javascript
    /// function foo() {
    ///   if (x) {
    ///     return y;
    ///   } else {
    ///     return z;
    ///   }
    /// }
    /// ```
    ///
    /// This rule is aimed at highlighting an unnecessary block of code
    /// following an `if` containing a return statement. As such, it will warn
    /// when it encounters an `else` following a chain of `if`s, all of them
    /// containing a `return` statement.
    ///
    /// Options
    /// This rule has an object option:
    ///
    /// - `allowElseIf`: `true` _(default)_ allows `else if` blocks after a return
    /// - `allowElseIf`: `false` disallows `else if` blocks after a return
    ///
    /// ### Examples
    ///
    /// #### `allowElseIf: true`
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo1() {
    ///     if (x) {
    ///         return y;
    ///     } else {
    ///         return z;
    ///     }
    /// }
    ///
    /// function foo2() {
    ///     if (x) {
    ///         return y;
    ///     } else if (z) {
    ///         return w;
    ///     } else {
    ///         return t;
    ///     }
    /// }
    ///
    /// function foo3() {
    ///     if (x) {
    ///         return y;
    ///     } else {
    ///         var t = "foo";
    ///     }
    ///
    ///     return t;
    /// }
    ///
    /// function foo4() {
    ///     if (error) {
    ///         return 'It failed';
    ///     } else {
    ///         if (loading) {
    ///             return "It's still loading";
    ///         }
    ///     }
    /// }
    ///
    /// // Two warnings for nested occurrences
    /// function foo5() {
    ///     if (x) {
    ///         if (y) {
    ///             return y;
    ///         } else {
    ///             return x;
    ///         }
    ///     } else {
    ///         return z;
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo1() {
    ///     if (x) {
    ///         return y;
    ///     }
    ///
    ///     return z;
    /// }
    ///
    /// function foo2() {
    ///     if (x) {
    ///         return y;
    ///     } else if (z) {
    ///         var t = "foo";
    ///     } else {
    ///         return w;
    ///     }
    /// }
    ///
    /// function foo3() {
    ///     if (x) {
    ///         if (z) {
    ///             return y;
    ///         }
    ///     } else {
    ///         return z;
    ///     }
    /// }
    ///
    /// function foo4() {
    ///     if (error) {
    ///         return 'It failed';
    ///     } else if (loading) {
    ///         return "It's still loading";
    ///     }
    /// }
    /// ```
    ///
    /// #### `allowElseIf: false`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     if (error) {
    ///         return 'It failed';
    ///     } else if (loading) {
    ///         return "It's still loading";
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     if (error) {
    ///         return 'It failed';
    ///     }
    ///
    ///     if (loading) {
    ///         return "It's still loading";
    ///     }
    /// }
    /// ```
    NoElseReturn,
    pedantic,
    conditional_fix
);

fn no_else_return_diagnostic(else_keyword: Span, last_return: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary 'else' after 'return'.")
        .with_labels([
            last_return.label("This consequent block always returns,"),
            else_keyword.label("Making this `else` block unnecessary."),
        ])
        .with_help("Remove the `else` block, moving its contents outside of the `if` statement.")
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

fn no_else_return_diagnostic_fix(
    ctx: &LintContext,
    last_return_span: Span,
    else_stmt_prev: &Statement,
    else_stmt: &Statement,
    if_block_node: &AstNode,
) {
    let prev_span = else_stmt_prev.span();
    let else_content_span = else_stmt.span();
    let else_keyword_span = Span::new(prev_span.end, else_content_span.start);
    let diagnostic = no_else_return_diagnostic(else_keyword_span, last_return_span);
    let parent_scope_id = if_block_node.scope_id();

    if !is_safe_from_name_collisions(ctx, else_stmt, parent_scope_id) {
        ctx.diagnostic(diagnostic);
        return;
    }
    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let target_span = Span::new(prev_span.end, else_content_span.end);

        // Capture the contents of the `else` statement, removing curly braces
        // for block statements
        let mut replacement_span = if let Statement::BlockStatement(block) = else_stmt {
            let first_stmt_start = block.body.first().map(|stmt| stmt.span().start);
            let last_stmt_end = block.body.last().map(|stmt| stmt.span().end);
            let (Some(start), Some(end)) = (first_stmt_start, last_stmt_end) else {
                return fixer.noop();
            };
            Span::new(start, end)
        } else {
            else_content_span
        };

        // expand the span start leftwards to include any leading whitespace
        replacement_span =
            replacement_span.expand_left(left_offset_for_whitespace(ctx, replacement_span.start));

        // Check if if statement's consequent block could introduce an ASI
        // hazard when `else` is removed.
        let needs_newline = match else_stmt_prev {
            Statement::ExpressionStatement(s) => !ctx.source_range(s.span).ends_with(';'),
            Statement::ReturnStatement(s) => !ctx.source_range(s.span).ends_with(';'),
            _ => false,
        };
        if needs_newline {
            let replacement = ctx.source_range(replacement_span);
            fixer.replace(target_span, "\n".to_string() + replacement)
        } else {
            fixer.replace_with(&target_span, &replacement_span)
        }
    });
}

#[allow(clippy::cast_possible_truncation)]
fn left_offset_for_whitespace(ctx: &LintContext, position: u32) -> u32 {
    if position == 0 {
        return position;
    }

    let chars = ctx.source_text()[..(position as usize)].chars().rev();
    let offset = chars.take_while(|c| c.is_whitespace()).count();
    debug_assert!(offset < u32::MAX as usize);
    offset as u32
}

fn naive_has_return(node: &Statement) -> Option<Span> {
    match node {
        Statement::BlockStatement(block) => {
            let last_child = block.body.last()?;
            if let Statement::ReturnStatement(r) = last_child {
                Some(r.span)
            } else {
                None
            }
        }
        Statement::ReturnStatement(r) => Some(r.span),
        _ => None,
    }
}

fn check_for_return_or_if(node: &Statement) -> Option<Span> {
    match node {
        Statement::ReturnStatement(r) => Some(r.span),
        Statement::IfStatement(if_stmt) => {
            let alternate = if_stmt.alternate.as_ref()?;
            if let (Some(_), Some(ret_span)) =
                (naive_has_return(alternate), naive_has_return(&if_stmt.consequent))
            {
                Some(ret_span)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn always_returns(stmt: &Statement) -> Option<Span> {
    match stmt {
        Statement::BlockStatement(block) => block.body.iter().find_map(check_for_return_or_if),
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

    if let Some(last_return_span) = always_returns(&if_stmt.consequent) {
        no_else_return_diagnostic_fix(ctx, last_return_span, &if_stmt.consequent, alternate, node);
    }
}

fn check_if_without_else(ctx: &LintContext, node: &AstNode) {
    let AstKind::IfStatement(if_stmt) = node.kind() else {
        return;
    };
    let mut current_node = if_stmt;
    let mut last_alternate;
    let mut last_alternate_prev;
    let mut last_return_span;

    loop {
        let Some(alternate) = &current_node.alternate else {
            return;
        };
        let Some(ret_span) = always_returns(&current_node.consequent) else {
            return;
        };
        last_alternate_prev = &current_node.consequent;
        last_alternate = alternate;
        last_return_span = ret_span;
        match alternate {
            Statement::IfStatement(if_stmt) => {
                current_node = if_stmt;
            }
            _ => break,
        }
    }

    no_else_return_diagnostic_fix(ctx, last_return_span, last_alternate_prev, last_alternate, node);
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
        ("function foo1() { if (true) { return x; } else { return y; } }", "function foo1() { if (true) { return x; } return y; }", None),
        ("function foo1() { if(true){ return x; }else{ return y; } }", "function foo1() { if(true){ return x; } return y; }", None),
("function foo2() { if (true) { var x = bar; return x; } else { var y = baz; return y; } }", "function foo2() { if (true) { var x = bar; return x; } var y = baz; return y; }", None),
("function foo3() { if (true) return x; else return y; }", "function foo3() { if (true) return x; return y; }", None),
("function foo4() { if (true) { if (false) return x; else return y; } else { return z; } }", "function foo4() { if (true) { if (false) return x; return y; } return z; }", None),
("function foo5() { if (true) { if (false) { if (true) return x; else { w = y; } } else { w = x; } } else { return z; } }", "function foo5() { if (true) { if (false) { if (true) return x; w = y; } else { w = x; } } else { return z; } }", None),
("function foo6() { if (true) { if (false) { if (true) return x; else return y; } } else { return z; } }", "function foo6() { if (true) { if (false) { if (true) return x; return y; } } else { return z; } }", None),
("function foo7() { if (true) { if (false) { if (true) return x; else return y; } return w; } else { return z; } }", "function foo7() { if (true) { if (false) { if (true) return x; return y; } return w; } return z; }", None),
("function foo8() { if (true) { if (false) { if (true) return x; else return y; } else { w = x; } } else { return z; } }", "function foo8() { if (true) { if (false) { if (true) return x; return y; } w = x; } else { return z; } }", None),
("function foo9() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", "function foo9() {if (x) { return true; } else if (y) { return true; } notAReturn(); }", None),
("function foo9a() {if (x) { return true; } else if (y) { return true; } else { notAReturn(); } }", "function foo9a() {if (x) { return true; } if (y) { return true; } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo9b() {if (x) { return true; } if (y) { return true; } else { notAReturn(); } }", "function foo9b() {if (x) { return true; } if (y) { return true; } notAReturn(); }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo10() { if (foo) return bar; else (foo).bar(); }", "function foo10() { if (foo) return bar; (foo).bar(); }", None),
("function foo13() { if (foo) return bar; 
			else { [1, 2, 3].map(foo) } }", "function foo13() { if (foo) return bar; [1, 2, 3].map(foo) }", None),
("function foo14() { if (foo) return bar 
			else { baz(); } 
			[1, 2, 3].map(foo) }", "function foo14() { if (foo) return bar\n baz(); 
			[1, 2, 3].map(foo) }", None),
("function foo17() { if (foo) return bar 
			else { baz() } 
			qaz() }", "function foo17() { if (foo) return bar\n baz() 
			qaz() }", None),
("function foo19() { if (true) { return x; } else if (false) { return y; } }", "function foo19() { if (true) { return x; } if (false) { return y; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo20() {if (x) { return true; } else if (y) { notAReturn() } else { notAReturn(); } }", "function foo20() {if (x) { return true; } if (y) { notAReturn() } else { notAReturn(); } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo21() { var x = true; if (x) { return x; } else if (x === false) { return false; } }", "function foo21() { var x = true; if (x) { return x; } if (x === false) { return false; } }", Some(serde_json::json!([{ "allowElseIf": false }]))),
("function foo() { var a; if (bar) { return true; } else { var a; } }", "function foo() { var a; if (bar) { return true; } var a; }", None),
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", "function foo() { if (bar) { var a; if (baz) { return true; } var a; } }", None),
("function foo() { var a; if (bar) { return true; } else { var a; } }", "function foo() { var a; if (bar) { return true; } var a; }", None),
("function foo() { if (bar) { var a; if (baz) { return true; } else { var a; } } }", "function foo() { if (bar) { var a; if (baz) { return true; } var a; } }", None),
("function foo() {let a; if (bar) { if (baz) { return true; } else { let a; } } }", "function foo() {let a; if (bar) { if (baz) { return true; } let a; } }", None),
("function foo() { try {} catch (a) { if (bar) { if (baz) { return true; } else { let a; } } } }", "function foo() { try {} catch (a) { if (bar) { if (baz) { return true; } let a; } } }", None),
("function foo() { if (bar) { return true; } else { let arguments; } }", "function foo() { if (bar) { return true; } let arguments; }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let arguments; } } }", "function foo() { if (bar) { if (baz) { return true; } let arguments; } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } a; }", "function foo() { if (bar) { if (baz) { return true; } let a; } a; }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { var a; } }", "function foo() { if (bar) { if (baz) { return true; } let a; } if (quux) { var a; } }", None),
("function foo() { if (quux) { var a; } if (bar) { if (baz) { return true; } else { let a; } } }", "function foo() { if (quux) { var a; } if (bar) { if (baz) { return true; } let a; } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } if (quux) { function a(){}  } }", "function foo() { if (bar) { if (baz) { return true; } let a; } if (quux) { function a(){}  } }", None),
("function foo() { if (bar) { if (baz) { return true; } else { let a; } } function a(){} }", "function foo() { if (bar) { if (baz) { return true; } let a; } function a(){} }", None),
("if (foo) { return true; } else { let a; }", "if (foo) { return true; } let a;", None)
    ];
    Tester::new(NoElseReturn::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
