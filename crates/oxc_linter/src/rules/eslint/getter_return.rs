use oxc_ast::{
    ast::{
        ChainElement, Expression, MemberExpression, MethodDefinitionKind, ObjectProperty,
        PropertyKind,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    pg::neighbors_filtered_by_edge_weight, AssignmentValue, BasicBlockElement, EdgeType, Register,
};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(getter-return): Expected to always return a value in getter.")]
#[diagnostic(severity(warning), help("Return a value from all code paths in getter."))]
struct GetterReturnDiagnostic(#[label] pub Span);

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
    /// Requires all getters to have a return statement
    ///
    /// ### Why is this bad?
    /// Getters should always return a value. If they don't, it's probably a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// class Person{
    ///     get name(){
    ///         // no return
    ///     }
    /// }
    /// ```
    GetterReturn,
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
        match callee.without_parenthesized() {
            Expression::MemberExpression(me) => Self::handle_member_expression(me),
            Expression::ChainExpression(ce) => match &ce.expression {
                ChainElement::MemberExpression(me) => Self::handle_member_expression(me),
                ChainElement::CallExpression(_) => {
                    false // todo: make a test for this
                }
            },
            _ => false,
        }
    }

    fn handle_paren_expr<'a>(expr: &'a Expression<'a>) -> bool {
        match expr.without_parenthesized() {
            Expression::CallExpression(ce) => Self::handle_actual_expression(&ce.callee),
            _ => false,
        }
    }

    fn is_wanted_node(node: &AstNode, ctx: &LintContext<'_>) -> bool {
        if let Some(parent) = ctx.nodes().parent_node(node.id()) {
            match parent.kind() {
                AstKind::MethodDefinition(mdef) => {
                    if matches!(mdef.kind, MethodDefinitionKind::Get) {
                        return true;
                    }
                }
                AstKind::ObjectProperty(ObjectProperty { kind, .. }) => {
                    if matches!(kind, PropertyKind::Get) {
                        return true;
                    }

                    if let Some(parent_2) = ctx.nodes().parent_node(parent.id()) {
                        if let Some(parent_3) = ctx.nodes().parent_node(parent_2.id()) {
                            if let Some(parent_4) = ctx.nodes().parent_node(parent_3.id()) {
                                // handle (X())
                                match parent_4.kind() {
                                    AstKind::ParenthesizedExpression(p) => {
                                        if Self::handle_paren_expr(&p.expression) {
                                            return true;
                                        }
                                    }
                                    AstKind::CallExpression(ce) => {
                                        if Self::handle_actual_expression(&ce.callee) {
                                            return true;
                                        }
                                    }
                                    _ => {}
                                }

                                if let Some(parent_5) = ctx.nodes().parent_node(parent_4.id()) {
                                    if let Some(parent_6) = ctx.nodes().parent_node(parent_5.id()) {
                                        match parent_6.kind() {
                                            AstKind::ParenthesizedExpression(p) => {
                                                if Self::handle_paren_expr(&p.expression) {
                                                    return true;
                                                }
                                            }
                                            AstKind::CallExpression(ce) => {
                                                if Self::handle_actual_expression(&ce.callee) {
                                                    return true;
                                                }
                                            }
                                            _ => {
                                                return false;
                                            }
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn run_diagnostic<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>, span: Span) {
        if !Self::is_wanted_node(node, ctx) {
            return;
        }

        let cfg = ctx.semantic().cfg();

        let output = neighbors_filtered_by_edge_weight(
            &cfg.graph,
            node.cfg_ix(),
            &|edge| match edge {
                EdgeType::Normal => None,
                // We don't need to handle backedges because we would have already visited
                // them on the forward pass
                | EdgeType::Backedge
                // We don't need to visit NewFunction edges because it's not going to be evaluated
                // immediately, and we are only doing a pass over things that will be immediately evaluated
                | EdgeType::NewFunction
                // By returning Some(X),
                // we signal that we don't walk to this path any farther.
                //
                // We stop this path on a ::Yes because if it was a ::No,
                // we would have already returned early before exploring more edges
                => Some(DefinitelyReturnsOrThrows::Yes),
            },
            // We ignore the state going into this rule because we only care whether
            // or not this path definitely returns or throws.
            //
            // Whether or not the path definitely returns is only has two states, Yes (the default)
            // or No (when we see this, we immediately stop walking). Other rules that require knowing
            // previous such as [`no_this_before_super`] we would want to observe this value.
            &mut |basic_block_id, _state_going_into_this_rule| {
                // Scan through the values in this basic block.
                for entry in cfg.basic_block_by_index(*basic_block_id) {
                    match entry {
                        // If the element is an assignment.
                        //
                        // Everything you can write in javascript that would have
                        // the function continue are expressed as assignments in the cfg.
                        BasicBlockElement::Assignment(to_reg, val) => {
                            // If the assignment is to the return register.
                            //
                            // The return register is a special register that return statements
                            // assign the returned value to.
                            if matches!(to_reg, Register::Return) {
                                // `allow_implicit` allows returning without a value to not
                                // fail the rule. We check for this by checking if the value
                                // being returned in the cfg this is expressed as
                                // `AssignmentValue::ImplicitUndefined`.
                                //
                                // There is an assumption being made here that returning an
                                // `undefined` will put the `undefined` directly into the
                                // return and will not put the `undefined` into an immediate
                                // register and return the register. However, the tests for
                                // this rule enforce that this invariant is not broken.
                                if !self.allow_implicit
                                    && matches!(val, AssignmentValue::ImplicitUndefined)
                                {
                                    // Return false as the second argument to signify we should
                                    // not continue walking this branch, as we know a return
                                    // is the end of this path.
                                    return (DefinitelyReturnsOrThrows::No, false);
                                }
                                // Otherwise, we definitely returned since we assigned
                                // to the return register.
                                //
                                // Return false as the second argument to signify we should
                                // not continue walking this branch, as we know a return
                                // is the end of this path.
                                return (DefinitelyReturnsOrThrows::Yes, false);
                            }
                        }
                        BasicBlockElement::Throw(_) => {
                            // Throws are classified as returning.
                            //
                            // todo: test with catching...
                            return (DefinitelyReturnsOrThrows::Yes, false);
                        }
                        BasicBlockElement::Unreachable => {
                            // Unreachable signifies the last element of this basic block and
                            // this path that will be observed by javascript, therefore if we
                            // haven't returned yet we won't after this.
                            //
                            // Return false as the second argument to signify we should
                            // not continue walking this branch, as we know a return
                            // is the end of this path.
                            return (DefinitelyReturnsOrThrows::No, false);
                        }
                    }
                }
                // Return true as the second argument to signify we should
                // continue walking this branch, as we haven't seen anything
                // that will signify to us that this path of the program will
                // definitely return or throw.
                (DefinitelyReturnsOrThrows::No, true)
            },
        );

        // Deciding whether we definitely return or throw in all
        // codepaths is as simple as seeing if each individual codepath
        // definitely returns or throws.
        let definitely_returns_in_all_codepaths =
            output.into_iter().all(|y| matches!(y, DefinitelyReturnsOrThrows::Yes));

        // If not, flag it as a diagnostic.
        if !definitely_returns_in_all_codepaths {
            ctx.diagnostic(GetterReturnDiagnostic(span));
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
enum DefinitelyReturnsOrThrows {
    #[default]
    No,
    Yes,
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        ("var foo = { get bar(){return true;} };", None),
        (
            "var foo = { get bar() {return;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "var foo = { get bar(){return true;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "var foo = { get bar(){if(bar) {return;} return true;} };",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){return true;} }", None),
        ("class foo { get bar(){if(baz){return true;} else {return false;} } }", None),
        ("class foo { get(){return true;} }", None),
        (
            "class foo { get bar(){return true;} }",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        ("class foo { get bar(){return;} }", Some(serde_json::json!([{ "allowImplicit": true }]))),
        ("Object.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});",
            None,
        ),
        ("Object.defineProperties(foo, { bar: { get: function () {return true;}} });", None),
        (
            "Object.defineProperties(foo, { bar: { get: function () { ~function (){ return true; }(); return true;}} });",
            None,
        ),
        ("Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});", None),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function () { ~function (){ return true; }();return true;}});",
            None,
        ),
        ("Object.create(foo, { bar: { get() {return true;} } });", None),
        ("Object.create(foo, { bar: { get: function () {return true;} } });", None),
        ("Object.create(foo, { bar: { get: () => {return true;} } });", None),
        (
            "Object.defineProperty(foo, \"bar\", { get: function () {return true;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperty(foo, \"bar\", { get: function (){return;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {return true;}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { get: function () {return;}} });",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
        (
            "Reflect.defineProperty(foo, \"bar\", { get: function () {return true;}});",
            Some(serde_json::json!([{ "allowImplicit": true }])),
        ),
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
        ("export abstract class Foo { protected abstract get foobar(): number; }", None),
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
    ];

    Tester::new(GetterReturn::NAME, pass, fail).test_and_snapshot();
}
