use oxc_allocator::{Allocator, Box, CloneIn, GetAddress, HashMap, Vec};
use oxc_ast::{
    ast::{
        Argument, ArrowFunctionExpression, BindingPatternKind, CallExpression, Expression,
        FormalParameters, MemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_promise,
    AstNode,
};

fn no_nesting_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNesting;

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
    NoNesting,
    promise,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

fn is_within_promise_handler<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
        return false;
    }

    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };
    if !matches!(ctx.nodes().kind(parent.id()), AstKind::Argument(_)) {
        return false;
    };

    let Some(AstKind::CallExpression(call_expr)) = ctx.nodes().parent_kind(parent.id()) else {
        return false;
    };

    matches!(call_expr.callee_name(), Some("then" | "catch"))
}

fn is_inside_promise(node: &AstNode, ctx: &LintContext) -> bool {
    if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
        || !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::Argument(_)))
    {
        return false;
    }

    ctx.nodes()
        .ancestors(node.id())
        .nth(2)
        .is_some_and(|node| node.kind().as_call_expression().is_some_and(has_promise_callback))
}

// .skip(1) useful too
fn closest_promise_callback_def_vars<'a, 'b>(
    node: &'a AstNode<'b>,
    ctx: &'a LintContext<'b>,
) -> Option<&'a CallExpression<'b>> {
    println!("111oo");

    //if !matches!(node.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
    //    || !matches!(ctx.nodes().parent_kind(node.id()), Some(AstKind::Argument(_)))
    //{
    //    return None;
    //}

    let a = ctx
        .nodes()
        .ancestors(node.id())
        .filter_map(|node| node.kind().as_call_expression())
        .filter(|a| has_promise_callback(a))
        //.map(|)
        //        .map(|s| s.arguments.iter().map(|arg|))
        .nth(1);
    //  .nth(0);

    a
    //  .is_some_and(|node| node.kind().as_call_expression().is_some_and(has_promise_callback))
}

/*

fn get_arg_names<'b>(call: &CallExpression<'b>) -> Vec<CompactStr> {
    let mut a = vec![];
    call.arguments.iter().for_each(|new_expr| {
        //            let mut v: Vec<&CompactStr> = Vec::new_in(allocator);

        //  for argument in &new_expr.arguments {
        let Some(arg_expr) = new_expr.as_expression() else {
            return;
        };
        match arg_expr {
            Expression::ArrowFunctionExpression(arrow_expr) => {
                for param in &arrow_expr.params.items {
                    if let BindingPatternKind::BindingIdentifier(param_ident) = &param.pattern.kind
                    {
                        //  let n = param_ident.name;
                        a.push(param_ident.name.to_compact_str());
                        //     println!("arg {n}");
                        //   arg_names.push(&n.to_compact_str())
                        //   v.push(param_ident.name.to_compact_str());
                    };
                }
                //   self.check_parameter_names(&arrow_expr.params, ctx);
            }
            Expression::FunctionExpression(func_expr) => {
                // self.check_parameter_names(&func_expr.params, ctx);
            }
            _ => return,
        }
        //     }
    });
}
*/
fn has_promise_callback(call_expr: &CallExpression) -> bool {
    matches!(
        call_expr.callee.as_member_expression().and_then(MemberExpression::static_property_name),
        Some("then" | "catch")
    )
}

fn is_promise_then_or_catch(call_expr: &CallExpression) -> Option<String> {
    let member_expr = call_expr.callee.get_member_expr()?;
    let prop_name = member_expr.static_property_name()?;

    // hello.then(), hello.catch()
    if matches!(prop_name, "then" | "catch") {
        return Some(prop_name.into());
    }

    None
}

/// Get closest callback function scope outside of current callback.
/// ```
/// doThing()
///  .then(a => getB(a) <---- get this scopes args
///    .then(b => getC(a, b)) <--- when here
///  )
/// ```
/// We don't want a violation of this rule in such cases
/// because we cannot unnest the above as `a` would be undefined.
/// Here is the unnested version where would be `a` `undefined`
/// in the second `then` callback:
/// ```
/// doThing()
///  .then(a => getB(a))
///  .then(b => getC(a, b))
/// ```
///
fn get_closest_promise_callback_def_vars<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Vec<'a, Argument<'a>>> {
    let closest_prom_cb_args = ctx.semantic().nodes().ancestors(node.id()).find_map(|node| {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return None;
        };

        if let Some(prop_name) = is_promise_then_or_catch(call_expr) {
            if prop_name == "then" {
                return Some(&call_expr.arguments);
            } else {
                return None;
            }
        } else {
            return None;
        };
    });

    closest_prom_cb_args
}

impl Rule for NoNesting {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(prop_name) = is_promise_then_or_catch(call_expr) else {
            return;
        };

        let allocator = Allocator::default();
        /*
        if let Some(args) = get_closest_promise_callback_def_vars(node, ctx) {
            //println!("args  {call_expr:?}");
            match args.first() {
                Some(Argument::Identifier(identifier_reference)) => {
                    let name = identifier_reference.name;
                    println!("args id name  {name:?}");
                }
                Some(Argument::ArrowFunctionExpression(arrow)) => {
                    let p = &arrow.params.items;
                    println!("looo {p:?}");
                }
                a => {
                    println!("dooo {a:?}");
                }
            }
        } else {
            println!("dooo");
        }
        */

        let mut ancestors = ctx.nodes().ancestors(node.id());
        if ancestors.any(|node| is_inside_promise(node, ctx)) {
            // get the args passed into the nested promise call.
            let mut nested_call_args = vec![];
            call_expr.arguments.iter().for_each(|arg| {
                let Some(arg_expr) = arg.as_expression() else {
                    return;
                };
                match arg_expr {
                    Expression::ArrowFunctionExpression(arrow_expr) => {
                        for param in &arrow_expr.params.items {
                            if let BindingPatternKind::BindingIdentifier(param_ident) =
                                &param.pattern.kind
                            {
                                //  let n = param_ident.name;
                                nested_call_args.push(param_ident.name.to_compact_str());
                                //     println!("arg {n}");
                                //   arg_names.push(&n.to_compact_str())
                                //   v.push(param_ident.name.to_compact_str());
                            };
                        }
                        //   self.check_parameter_names(&arrow_expr.params, ctx);
                    }
                    _ => {}
                }
            });
            println!("nested call args {nested_call_args:?}");

            // Extract out this logic into two parts
            // 1. Gets names of variables defined in closest parent promise callback function scope.
            // 2. Checks if the argument callback of the nesteted promise call uses any of these variables from 1.

            if let Some(closest) = closest_promise_callback_def_vars(node, ctx) {
                // Compare the arg identifier names of the nested promise
                //
                // .then(a => getB(a)  <--- we need to get the args defined in this cb
                //   .then(b => getC(a, b)) <--- to see if they are used here in this cb scope
                // because if any are then we cannot unnest and so don't flag as rule violation.
                //      let mut closest_promise_cb_def_vars = vec![];
                // let mut closest_promise_cb_def_vars_symbols = vec![];
                //               let mut closest_cb_scope_bindings: Vec<(&str, SymbolId)>  = vec![];
                let mut closest_cb_scope_bindings: &HashMap<'_, &str, SymbolId> =
                    &HashMap::new_in(&allocator);

                closest.arguments.iter().for_each(|new_expr| {
                    //            let mut v: Vec<&CompactStr> = Vec::new_in(allocator);

                    //  for argument in &new_expr.arguments {
                    let Some(arg_expr) = new_expr.as_expression() else {
                        return;
                    };
                    match arg_expr {
                        Expression::ArrowFunctionExpression(arrow_expr) => {
                            let func_scope = arrow_expr.scope_id();
                            println!("scope id {func_scope:?}");

                            let bound_vars_for_scope = ctx.scopes().get_bindings(func_scope);
                            closest_cb_scope_bindings = bound_vars_for_scope;
                            // .closest_promise_cb_def_vars_symbols
                            // .push(param_ident.symbol_id());

                            //       for param in &arrow_expr.params.items {
                            //           if let BindingPatternKind::BindingIdentifier(param_ident) =
                            //               &param.pattern.kind
                            //           {
                            //               //  let n = param_ident.name;
                            //               closest_promise_cb_def_vars
                            //                   .push(param_ident.name.to_compact_str());
                            //               //closest_promise_cb_def_vars_symbols
                            //               //    .push(param_ident.symbol_id());
                            //
                            //               //param_ident.name.to_compact_str());
                            //               //     println!("arg {n}");
                            //               //   arg_names.push(&n.to_compact_str())
                            //               //   v.push(param_ident.name.to_compact_str());
                            //           };
                            //       }
                            //   self.check_parameter_names(&arrow_expr.params, ctx);
                        }
                        Expression::FunctionExpression(func_expr) => {
                            let func_scope = func_expr.scope_id();
                            println!("scope id {func_scope:?}");

                            // self.check_parameter_names(&func_expr.params, ctx);
                        }
                        _ => return,
                    }
                    //     }
                });

                //                println!("argys {closest_promise_cb_def_vars:?}");

                // Now check for references in cb_span to variables defined in the closest parent cb scope.
                if let Some(cb_span) = call_expr.arguments.get(0).map(|a| a.span()) {
                    //  .then((a,b,c) => getB(a)
                    //      // ^^^^^ closest_parent_cb_args_span
                    //    .then(d => getC(a, b))
                    //   // cb_span: ^^^^^^^^^^^ <- get this expression so we can check for usages of a,b,c there

                    // test
                    //  ctx.diagnostic(no_nesting_diagnostic(cb_span));

                    // now check in the cb_span for usage of variables defined in closest_parent_cb_args_span
                    for (binding_name, binding_symbol_id) in closest_cb_scope_bindings {
                        // Loop through a,b,c in:
                        //  .then((a,b,c) => getB(a)
                        //    .then(d => getC(a, b))
                        println!("checking binding name {binding_name:?} and symbol_id {binding_symbol_id:?}");
                        for usage in ctx.semantic().symbol_references(*binding_symbol_id) {
                            let usage_span: Span = ctx.reference_span(usage);
                            println!("ref span where used {usage_span:?}");

                            // test
                            //   ctx.diagnostic(no_nesting_diagnostic(usage_span));

                            if cb_span.contains_inclusive(usage_span) {
                                // Cannot unnest this nested promise as the nested cb refers to a variable
                                // defined in the parent promise callback scope. Unnesting would result in
                                // reference to an undefined variable.
                                return;
                            };
                        }
                    }
                }

                ctx.diagnostic(no_nesting_diagnostic(call_expr.callee.span()));
            };
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Promise.resolve(4).then(function(x) { return x })",
        "Promise.reject(4).then(function(x) { return x })",
        "Promise.resolve(4).then(function() {})",
        "Promise.reject(4).then(function() {})",
        "doThing().then(function() { return 4 })",
        "doThing().then(function() { throw 4 })",
        "doThing().then(null, function() { return 4 })",
        "doThing().then(null, function() { throw 4 })",
        "doThing().catch(null, function() { return 4 })",
        "doThing().catch(null, function() { throw 4 })",
        "doThing().then(() => 4)",
        "doThing().then(() => { throw 4 })",
        "doThing().then(()=>{}, () => 4)",
        "doThing().then(()=>{}, () => { throw 4 })",
        "doThing().catch(() => 4)",
        "doThing().catch(() => { throw 4 })",
        "var x = function() { return Promise.resolve(4) }",
        "function y() { return Promise.resolve(4) }",
        "function then() { return Promise.reject() }",
        "doThing(function(x) { return Promise.reject(x) })",
        "doThing().then(function() { return Promise.all([a,b,c]) })",
        "doThing().then(function() { return Promise.resolve(4) })",
        "doThing().then(() => Promise.resolve(4))",
        "doThing().then(() => Promise.all([a]))",
        "doThing()
			      .then(a => getB(a)
			        .then(b => getC(a, b))
			      )",
        "doThing()
			      .then(a => getB(a)
			        .then(function(b) { getC(a, b) })
			      )",
        "doThing()
            .then(a => {
              const c = a * 2;
              return getB(c).then(b => getC(c, b))
            })",
    ];

    let fail = vec![
        "doThing().then(function() { a.then() })",
        "doThing().then(function() { b.catch() })",
        "doThing().then(function() { return a.then() })",
        "doThing().then(function() { return b.catch() })",
        "doThing().then(() => { a.then() })",
        "doThing().then(() => { b.catch() })",
        "doThing().then(() => a.then())",
        "doThing().then(() => b.catch())",
        "
			      doThing()
			        .then(a => getB(a)
			          .then(b => getC(b))
			        )",
        "
			      doThing()
			        .then(a => getB(a)
			          .then(b => getC(a, b)
			            .then(c => getD(a, c))
			          )
			        )",
    ];

    Tester::new(NoNesting::NAME, NoNesting::PLUGIN, pass, fail).test_and_snapshot();
}
