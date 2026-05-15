use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn callback_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected return with your callback function.")
        .with_help("Return the callback call or add an explicit return immediately after it.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
/// The rule takes a single option - an array of possible callback names - which may include object methods. The default callback names are `callback`, `cb`, `next`.
pub struct CallbackReturn(Vec<CompactStr>);

impl Default for CallbackReturn {
    fn default() -> Self {
        Self(vec!["callback".into(), "cb".into(), "next".into()])
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `return` statements after callbacks.
    ///
    /// ### Why is this bad?
    ///
    /// This rule is aimed at ensuring that callbacks used outside of the main function block are always part-of or immediately preceding a `return` statement.
    /// This rule decides what is a callback based on the name of the function being called.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function done(err) {
    ///     if (err) {
    ///         callback(err);
    ///     }
    ///     callback();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function done(err) {
    ///     if (err) {
    ///         return callback(err);
    ///     }
    ///     callback();
    /// }
    /// ```
    ///
    /// ### Known Limitations
    ///
    /// Because it is difficult to understand the meaning of a program through static analysis, this rule has limitations:
    ///
    /// - *false negatives* when this rule reports correct code, but the program calls the callback more than one time (which is incorrect behavior)
    /// - *false positives* when this rule reports incorrect code, but the program calls the callback only one time (which is correct behavior)
    ///
    /// #### Passing the callback by reference
    ///
    /// The static analysis of this rule does not detect that the program calls the callback if it is an argument of a function (for example,  `setTimeout`).
    ///
    /// Example of a *false negative* when this rule reports correct code:
    ///
    /// ```js
    /// function foo(err, callback) {
    ///     if (err) {
    ///         setTimeout(callback, 0); // this is bad, but WILL NOT warn
    ///     }
    ///     callback();
    /// }
    /// ```
    ///
    /// #### Triggering the callback within a nested function
    ///
    /// The static analysis of this rule does not detect that the program calls the callback from within a nested function or an immediately-invoked function expression (IIFE).
    ///
    /// Example of a *false negative* when this rule reports correct code:
    ///
    /// ```js
    /// function foo(err, callback) {
    ///     if (err) {
    ///         process.nextTick(function() {
    ///             return callback(); // this is bad, but WILL NOT warn
    ///         });
    ///     }
    ///     callback();
    /// }
    /// ```
    ///
    /// #### If/else statements
    ///
    /// The static analysis of this rule does not detect that the program calls the callback only one time in each branch of an `if` statement.
    ///
    /// Example of a *false positive* when this rule reports incorrect code:
    ///
    /// ```js
    /// function foo(err, callback) {
    ///     if (err) {
    ///         callback(err); // this is fine, but WILL warn
    ///     } else {
    ///         callback();    // this is fine, but WILL warn
    ///     }
    /// }
    /// ```
    CallbackReturn,
    node,
    style,
    version = "next",
    config = CallbackReturn,
);

impl Rule for CallbackReturn {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !self.is_callback(call_expr, ctx.source_text()) {
            return;
        }

        let Some(closest_block) = find_closest_block_parent(node, ctx) else {
            return;
        };

        if matches!(
            closest_block.kind(),
            AstKind::ReturnStatement(_) | AstKind::ArrowFunctionExpression(_)
        ) {
            return;
        }

        let block_body = match closest_block.kind() {
            AstKind::BlockStatement(block_stmt) => block_stmt.body.as_slice(),
            AstKind::FunctionBody(function_body) => function_body.statements.as_slice(),
            _ => &[],
        };

        if let Some(last_item) = block_body.last() {
            if is_callback_expression(call_expr, last_item)
                && is_function_like(&ctx.nodes().parent_node(closest_block.id()).kind())
            {
                return;
            }

            if matches!(last_item, Statement::ReturnStatement(_))
                && block_body
                    .get(block_body.len().saturating_sub(2))
                    .is_some_and(|previous_item| is_callback_expression(call_expr, previous_item))
            {
                return;
            }
        }

        if find_closest_function_parent(node, ctx).is_some() {
            ctx.diagnostic(callback_return_diagnostic(call_expr.span));
        }
    }
}

impl CallbackReturn {
    fn is_callback(&self, call_expr: &CallExpression, source_text: &str) -> bool {
        contains_only_identifiers(&call_expr.callee)
            && self.0.iter().any(|callback| {
                // compare by source text instead of `callee_name()` to handle cases like `obj.method(err)` with config `["obj.method"]`
                callback.as_str() == call_expr.callee.span().source_text(source_text)
            })
    }
}

fn contains_only_identifiers(expr: &Expression) -> bool {
    if let Expression::Identifier(_) = expr {
        return true;
    }

    if let Some(member_expr) = &expr.as_member_expression() {
        if matches!(member_expr.object(), Expression::Identifier(_)) {
            return true;
        }

        if let Some(obj_member_expr) = member_expr.object().as_member_expression() {
            return contains_only_identifiers(obj_member_expr.object());
        }
    }

    false
}

fn find_closest_block_parent<'a, 'b>(
    ast_node: &AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes().ancestors(ast_node.id()).find(|parent| {
        matches!(
            parent.kind(),
            AstKind::BlockStatement(_)
                | AstKind::FunctionBody(_)
                | AstKind::ReturnStatement(_)
                | AstKind::ArrowFunctionExpression(_)
        )
    })
}

fn find_closest_function_parent<'a, 'b>(
    ast_node: &AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    ctx.nodes().ancestors(ast_node.id()).find(|parent| is_function_like(&parent.kind()))
}

fn is_function_like(kind: &AstKind) -> bool {
    matches!(kind, AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
}

fn is_callback_expression(call_expr: &CallExpression, parent_node: &Statement) -> bool {
    let Statement::ExpressionStatement(expr_stmt) = parent_node else {
        return false;
    };

    if expr_stmt.expression.span() == call_expr.span {
        return true;
    }

    match expr_stmt.expression.without_parentheses() {
        Expression::BinaryExpression(binary_expr) => {
            binary_expr.right.without_parentheses().span() == call_expr.span
        }
        Expression::LogicalExpression(logical_expr) => {
            logical_expr.right.without_parentheses().span() == call_expr.span
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function a(err) { if (err) return callback (err); }", None),
        ("function a(err) { if (err) return callback (err); callback(); }", None),
        ("function a(err) { if (err) { return callback (err); } callback(); }", None),
        (
            "function a(err) { if (err) { return /* confusing comment */ callback (err); } callback(); }",
            None,
        ),
        ("function x(err) { if (err) { callback(); return; } }", None),
        (
            "function x(err) { if (err) { 
             log();
             callback(); return; } }",
            None,
        ),
        ("function x(err) { if (err) { callback(); return; } return callback(); }", None),
        ("function x(err) { if (err) { return callback(); } else { return callback(); } }", None),
        (
            "function x(err) { if (err) { return callback(); } else if (x) { return callback(); } }",
            None,
        ),
        ("function x(err) { if (err) return callback(); else return callback(); }", None),
        ("function x(cb) { cb && cb(); }", None),
        ("function x(next) { typeof next !== 'undefined' && next(); }", None),
        ("function x(next) { if (typeof next === 'function')  { return next() } }", None),
        ("function x() { switch(x) { case 'a': return next(); } }", None),
        ("function x() { for(x = 0; x < 10; x++) { return next(); } }", None),
        ("function x() { while(x) { return next(); } }", None),
        ("function a(err) { if (err) { obj.method (err); } }", None),
        ("callback()", None),
        ("callback(); callback();", None),
        ("while(x) { move(); }", None),
        ("for (var i = 0; i < 10; i++) { move(); }", None),
        ("for (var i = 0; i < 10; i++) move();", None),
        ("if (x) callback();", None),
        ("if (x) { callback(); }", None),
        ("var x = err => { if (err) { callback(); return; } }", None),
        ("var x = err => callback(err)", None),
        ("var x = err => { setTimeout( () => { callback(); }); }", None),
        ("class x { horse() { callback(); } } ", None),
        ("class x { horse() { if (err) { return callback(); } callback(); } } ", None),
        ("function a(err) { if (err) { callback(err) } }", Some(serde_json::json!([["cb"]]))),
        (
            "function a(err) { if (err) { callback(err) } next(); }",
            Some(serde_json::json!([["cb", "next"]])),
        ),
        (
            "function a(err) { if (err) { return next(err) } else { callback(); } }",
            Some(serde_json::json!([["cb", "next"]])),
        ),
        (
            "function a(err) { if (err) { return obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { return obj.prop.method(err); } }",
            Some(serde_json::json!([["obj.prop.method"]])),
        ),
        (
            "function a(err) { if (err) { return obj.prop.method(err); } otherObj.prop.method() }",
            Some(serde_json::json!([["obj.prop.method", "otherObj.prop.method"]])),
        ),
        (
            "function a(err) { if (err) { callback(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { otherObj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { //comment
            return obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { /*comment*/return obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { return obj.method(err); //comment
             } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { return obj.method(err); /*comment*/ } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { obj().method(err); } }",
            Some(serde_json::json!([["obj().method"]])),
        ),
        (
            "function a(err) { if (err) { obj.prop().method(err); } }",
            Some(serde_json::json!([["obj.prop().method"]])),
        ),
        (
            "function a(err) { if (err) { obj().prop.method(err); } }",
            Some(serde_json::json!([["obj().prop.method"]])),
        ),
        (
            "function a(err) { if (err) { obj().method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { obj().method(err); } obj.method(); }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        ("function x(err) { if (err) { setTimeout(callback, 0); } callback(); }", None),
        (
            "function x(err) { if (err) { process.nextTick(function(err) { callback(); }); } callback(); }",
            None,
        ),
    ];

    let fail = vec![
        ("function a(err) { if (err) { callback (err); } }", None),
        ("function a(callback) { if (typeof callback !== 'undefined') { callback(); } }", None),
        ("function a(callback) { if (typeof callback !== 'undefined') callback();  }", None),
        ("function a(callback) { if (err) { callback(); horse && horse(); } }", None),
        ("var x = (err) => { if (err) { callback (err); } }", None),
        ("var x = { x(err) { if (err) { callback (err); } } }", None),
        (
            "function x(err) { if (err) {
             log();
             callback(err); } }",
            None,
        ),
        ("var x = { x(err) { if (err) { callback && callback (err); } } }", None),
        ("function a(err) { callback (err); callback(); }", None),
        ("function a(err) { callback (err); horse(); }", None),
        ("function a(err) { if (err) { callback (err); horse(); return; } }", None),
        ("var a = (err) => { callback (err); callback(); }", None),
        (
            "function a(err) { if (err) { callback (err); } else if (x) { callback(err); return; } }",
            None,
        ),
        (
            "function x(err) { if (err) { return callback(); }
            else if (abc) {
            callback(); }
            else {
            return callback(); } }",
            None,
        ),
        ("class x { horse() { if (err) { callback(); } callback(); } } ", None),
        ("function x(err) { if (err) { callback() } else { callback() } }", None),
        ("function x(err) { if (err) return callback(); else callback(); }", None),
        ("() => { if (x) { callback(); } }", None),
        ("function b() { switch(x) { case 'horse': callback(); } }", None),
        (
            "function a() { switch(x) { case 'horse': move(); } }",
            Some(serde_json::json!([["move"]])),
        ),
        ("var x = function() { while(x) { move(); } }", Some(serde_json::json!([["move"]]))),
        (
            "function x() { for (var i = 0; i < 10; i++) { move(); } }",
            Some(serde_json::json!([["move"]])),
        ),
        (
            "var x = function() { for (var i = 0; i < 10; i++) move(); }",
            Some(serde_json::json!([["move"]])),
        ),
        (
            "function a(err) { if (err) { obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { obj.prop.method(err); } }",
            Some(serde_json::json!([["obj.prop.method"]])),
        ),
        (
            "function a(err) { if (err) { obj.prop.method(err); } otherObj.prop.method() }",
            Some(serde_json::json!([["obj.prop.method", "otherObj.prop.method"]])),
        ),
        (
            "function a(err) { if (err) { /*comment*/obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { //comment
            obj.method(err); } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { obj.method(err); /*comment*/ } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
        (
            "function a(err) { if (err) { obj.method(err); //comment
             } }",
            Some(serde_json::json!([["obj.method"]])),
        ),
    ];

    Tester::new(CallbackReturn::NAME, CallbackReturn::PLUGIN, pass, fail).test_and_snapshot();
}
