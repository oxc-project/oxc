use oxc_ast::{
    ast::{
        ArrayAssignmentTarget, AssignmentTarget, AssignmentTargetPattern, BindingPattern,
        Expression, PropertyKey, SimpleAssignmentTarget, VariableDeclaration,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, globals::PRE_DEFINE_VAR, rule::Rule, AstNode};

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

fn binding_pattern_is_global_obj(
    pat: &BindingPattern,
    ignore_undefined: bool,
) -> Option<(Atom, Span)> {
    match &pat.kind {
        oxc_ast::ast::BindingPatternKind::BindingIdentifier(boxed_bind_identifier) => {
            if ignore_undefined && boxed_bind_identifier.name.as_str() == "undefined" {
            } else if PRE_DEFINE_VAR.contains_key(boxed_bind_identifier.name.as_str()) {
                return Some((boxed_bind_identifier.name.clone(), boxed_bind_identifier.span));
            }
            None
        }
        oxc_ast::ast::BindingPatternKind::ObjectPattern(boxed_obj_pat) => {
            let properties = &boxed_obj_pat.properties;
            for property in properties {
                if let Some(value) = binding_pattern_is_global_obj(&property.value, false) {
                    return Some(value);
                }
            }
            boxed_obj_pat
                .rest
                .as_ref()
                .and_then(|boxed_rest| binding_pattern_is_global_obj(&boxed_rest.argument, false))
        }
        oxc_ast::ast::BindingPatternKind::ArrayPattern(boxed_arr_pat) => {
            for element in boxed_arr_pat.elements.iter() {
                if let Some(value) =
                    element.as_ref().and_then(|e| binding_pattern_is_global_obj(e, false))
                {
                    return Some(value);
                }
            }
            boxed_arr_pat
                .rest
                .as_ref()
                .and_then(|boxed_rest| binding_pattern_is_global_obj(&boxed_rest.argument, false))
        }
        oxc_ast::ast::BindingPatternKind::AssignmentPattern(boxed_assign_pat) => {
            binding_pattern_is_global_obj(&boxed_assign_pat.left, false)
        }
    }
}

fn get_nearest_undefined_declare_span(ctx: &LintContext) -> Option<Span> {
    let nodes = ctx.nodes();
    let mut span: Option<Span> = None;
    for node in nodes.iter() {
        let kind = node.kind();
        match kind {
            AstKind::VariableDeclaration(VariableDeclaration { declarations, .. }) => {
                for var_declarator in declarations {
                    let id = &var_declarator.id;
                    if let Some((name, s)) = binding_pattern_is_global_obj(id, false) {
                        if name == "undefined" {
                            span = Some(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    span
}

impl Rule for NoShadowRestrictedNames {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();
        match kind {
            AstKind::VariableDeclaration(VariableDeclaration { declarations, .. }) => {
                for var_declarator in declarations {
                    let id = &var_declarator.id;
                    if let Some(value) =
                        binding_pattern_is_global_obj(id, var_declarator.init.is_none())
                    {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(value.0, value.1));
                    }
                }
            }
            AstKind::ExpressionStatement(expr_stat) => match &expr_stat.expression {
                Expression::AssignmentExpression(assign_expr) => {
                    let left = &assign_expr.left;
                    match left {
                        AssignmentTarget::SimpleAssignmentTarget(
                            SimpleAssignmentTarget::AssignmentTargetIdentifier(ati),
                        ) if ati.name == "undefined" => {
                            let span = if let Some(span) = get_nearest_undefined_declare_span(ctx) {
                                span
                            } else {
                                ati.span
                            };
                            ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                                ati.name.clone(),
                                span,
                            ));
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            AstKind::Function(function) => {
                if let Some(bind_ident) = function.id.as_ref() {
                    if PRE_DEFINE_VAR.contains_key(bind_ident.name.as_str()) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                            bind_ident.name.clone(),
                            bind_ident.span,
                        ));
                    }
                }
                for param in function.params.items.iter() {
                    if let Some(value) = binding_pattern_is_global_obj(&param.pattern, false) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(value.0, value.1));
                    }
                }
            }
            AstKind::Class(class_decl) => {
                if let Some(bind_ident) = class_decl.id.as_ref() {
                    if PRE_DEFINE_VAR.contains_key(bind_ident.name.as_str()) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                            bind_ident.name.clone(),
                            bind_ident.span,
                        ));
                    }
                }
            }
            AstKind::CatchClause(catch_clause) => {
                if let Some(param) = catch_clause.param.as_ref() {
                    if let Some(value) = binding_pattern_is_global_obj(param, false) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(value.0, value.1));
                    }
                }
            }
            AstKind::MethodDefinition(method_definition) => match &method_definition.key {
                PropertyKey::Identifier(ident) => {
                    if PRE_DEFINE_VAR.contains_key(ident.name.as_str()) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                            ident.name.clone(),
                            ident.span,
                        ));
                    }
                }
                PropertyKey::PrivateIdentifier(ident) => {
                    if PRE_DEFINE_VAR.contains_key(ident.name.as_str()) {
                        ctx.diagnostic(NoShadowRestrictedNamesDiagnostic(
                            ident.name.clone(),
                            ident.span,
                        ));
                    }
                }
                PropertyKey::Expression(_) => {}
            },
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo(bar){ var baz; }", None),
        ("!function foo(bar){ var baz; }", None),
        ("!function(bar){ var baz; }", None),
        ("try {} catch(e) {}", None),
        ("try {} catch(e: undefined) {}", None),
        ("export default function() {}", None),
        ("try {} catch {}", None),
        ("var undefined;", None),
        ("var undefined;", None),
        ("var normal, undefined;", None),
        ("var undefined; doSomething(undefined);", None),
        ("var normal, undefined; var undefined;", None),
    ];

    let fail = vec![
        ("function NaN(NaN) { var NaN; !function NaN(NaN) { try {} catch(NaN) {} }; }", None),
        ("function undefined(undefined) { !function undefined(undefined) { try {} catch(undefined) {} }; }", None),
        ("function Infinity(Infinity) { var Infinity; !function Infinity(Infinity) { try {} catch(Infinity) {} }; }", None),
        ("function arguments(arguments) { var arguments; !function arguments(arguments) { try {} catch(arguments) {} }; }", None),
        ("function eval(eval) { var eval; !function eval(eval) { try {} catch(eval) {} }; }", None),
        ("var eval = (eval) => { var eval; !function eval(eval) { try {} catch(eval) {} }; }", None),
        ("var {undefined} = obj; var {a: undefined} = obj; var {a: {b: {undefined}}} = obj; var {a, ...undefined} = obj;", None),
        ("var normal, undefined; undefined = 5;", None),
        ("try {} catch(undefined: undefined) {}", None),
        ("var [undefined] = [1]", None),
        ("class undefined { }", None),
        ("class foo { undefined() { } }", None),
        ("class foo { #undefined() { } }", None),
        ("class foo { #undefined(undefined) { } }", None),
    ];

    Tester::new(NoShadowRestrictedNames::NAME, pass, fail).test_and_snapshot();
}
