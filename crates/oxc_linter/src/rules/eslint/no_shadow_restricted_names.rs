use oxc_ast::{
    ast::{AssignmentTarget, BindingPattern, Expression, SimpleAssignmentTarget},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, globals::PRE_DEFINE_VAR, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-shadow-restricted-names): Shadowing of global properties such as 'undefined' is not allowed.")]
#[diagnostic(severity(warning), help("Shadowing of global properties '{0}'."))]
struct NoShadowRestrictedNamesDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoShadowRestrictedNames;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow redefine the global variables like 'undefined', 'NaN', 'Infinity', 'eval', 'arguments'.
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// function NaN(){}
    ///
    /// !function(Infinity){};
    ///
    /// var undefined = 5;
    ///
    /// try {} catch(eval){}
    /// ```
    NoShadowRestrictedNames,
    correctness
);

fn binding_pattern_is_global_obj(pat: &BindingPattern) -> Option<(Atom, Span)> {
    match &pat.kind {
        oxc_ast::ast::BindingPatternKind::BindingIdentifier(boxed_bind_identifier) => {
            if PRE_DEFINE_VAR.contains_key(boxed_bind_identifier.name.as_str()) {
                return Some((boxed_bind_identifier.name.clone(), boxed_bind_identifier.span));
            }
            None
        }
        oxc_ast::ast::BindingPatternKind::ObjectPattern(boxed_obj_pat) => {
            let properties = &boxed_obj_pat.properties;
            for property in properties {
                if let Some(value) = binding_pattern_is_global_obj(&property.value) {
                    return Some(value);
                }
            }
            boxed_obj_pat
                .rest
                .as_ref()
                .and_then(|boxed_rest| binding_pattern_is_global_obj(&boxed_rest.argument))
        }
        oxc_ast::ast::BindingPatternKind::ArrayPattern(boxed_arr_pat) => {
            for element in &boxed_arr_pat.elements {
                if let Some(value) = element.as_ref().and_then(binding_pattern_is_global_obj) {
                    return Some(value);
                }
            }
            boxed_arr_pat
                .rest
                .as_ref()
                .and_then(|boxed_rest| binding_pattern_is_global_obj(&boxed_rest.argument))
        }
        oxc_ast::ast::BindingPatternKind::AssignmentPattern(boxed_assign_pat) => {
            binding_pattern_is_global_obj(&boxed_assign_pat.left)
        }
    }
}

#[inline]
fn check_and_diagnostic(atom: Atom, span: Span, ctx: &LintContext) {
    if PRE_DEFINE_VAR.contains_key(atom.as_str()) {
        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(atom, span));
    }
}

impl Rule for NoShadowRestrictedNames {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut nearest_span: Option<Span> = None;
        for node in ctx.nodes().iter() {
            let kind = node.kind();
            match kind {
                AstKind::VariableDeclarator(decl) => {
                    if let Some((atom, span)) = binding_pattern_is_global_obj(&decl.id) {
                        if atom.as_str() != "undefined" || decl.init.is_some() {
                            ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(atom, span));
                        } else {
                            nearest_span = Some(span);
                        }
                    }
                }
                AstKind::ExpressionStatement(expr_stat) => {
                    if let Expression::AssignmentExpression(assign_expr) = &expr_stat.expression {
                        let left = &assign_expr.left;
                        match left {
                            AssignmentTarget::SimpleAssignmentTarget(
                                SimpleAssignmentTarget::AssignmentTargetIdentifier(ati),
                            ) if ati.name == "undefined" => {
                                if let Some(span) = nearest_span {
                                    ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                                        ati.name.clone(),
                                        span,
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                AstKind::Function(function) => {
                    if let Some(bind_ident) = function.id.as_ref() {
                        check_and_diagnostic(bind_ident.name.clone(), bind_ident.span, ctx);
                    }
                }
                AstKind::FormalParameter(param) => {
                    if let Some(value) = binding_pattern_is_global_obj(&param.pattern) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(value.0, value.1));
                    }
                }
                AstKind::Class(class_decl) => {
                    if let Some(bind_ident) = class_decl.id.as_ref() {
                        check_and_diagnostic(bind_ident.name.clone(), bind_ident.span, ctx);
                    }
                }
                AstKind::CatchClause(catch_clause) => {
                    if let Some(param) = catch_clause.param.as_ref() {
                        if let Some(value) = binding_pattern_is_global_obj(param) {
                            ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(value.0, value.1));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("function foo(bar){ var baz; }", None),
        ("!function foo(bar){ var baz; }", None),
        ("!function(bar){ var baz; }", None),
        ("try {} catch(e) {}", None),
        ("try {} catch(e: undefined) {}", None),
        (
            "export default function() {}",
            Some(json!({
                "parserOptions": {
                    "ecmaVersion": 6,
                    "sourceType": "module"
                }
            })),
        ),
        (
            "try {} catch {}",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2019 }
            })),
        ),
        ("var undefined;", None),
        ("var undefined;var undefined", None),
        (
            "let undefined",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2015 }
            })),
        ),
        ("var normal, undefined;", None),
        ("var undefined; doSomething(undefined);", None),
        ("class foo { undefined() { } }", None),
        (
            "class foo { #undefined() { } }",
            Some(json!({
                "parserOptions": { "ecmaVersion": 2019 }
            })),
        ),
        ("var normal, undefined; var undefined;", None),
    ];

    let fail = vec![
        ("function NaN(NaN) { var NaN; !function NaN(NaN) { try {} catch(NaN) {} }; }", None),
        ("function undefined(undefined) { !function undefined(undefined) { try {} catch(undefined) {} }; }", None),
        ("function Infinity(Infinity) { var Infinity; !function Infinity(Infinity) { try {} catch(Infinity) {} }; }", None),
        ("function arguments(arguments) { var arguments; !function arguments(arguments) { try {} catch(arguments) {} }; }", None),
        ("function eval(eval) { var eval; !function eval(eval) { try {} catch(eval) {} }; }", None),
        ("var eval = (eval) => { var eval; !function eval(eval) { try {} catch(eval) {} }; }", Some(json!({
            "parserOptions": {
                "ecmaVersion": 6
            }
        }))),
        ("var {undefined} = obj; var {a: undefined} = obj; var {a: {b: {undefined}}} = obj; var {a, ...undefined} = obj;", Some(json!({
            "parserOptions": {
                "ecmaVersion": 9
            }
        }))),
        ("var normal, undefined; undefined = 5;", None),
        ("try {} catch(undefined: undefined) {}", None),
        ("var [undefined] = [1]", Some(json!({
            "parserOptions": {
                "ecmaVersion": 6
            }
        }))),
        ("class undefined { }", None),
        ("class foo { undefined(undefined) { } }", None),
        ("class foo { #undefined(undefined) { } }", None),
    ];

    Tester::new(NoShadowRestrictedNames::NAME, pass, fail).test_and_snapshot();
}
