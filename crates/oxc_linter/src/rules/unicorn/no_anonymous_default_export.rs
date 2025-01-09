use std::fmt;

use oxc_ast::{
    ast::{AssignmentExpression, AssignmentTarget, ExportDefaultDeclarationKind, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_anonymous_default_export_diagnostic(span: Span, kind: ErrorNodeKind) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("This {kind} default export is missing a name"))
        // TODO: suggest a name. https://github.com/sindresorhus/eslint-plugin-unicorn/blob/d3e4b805da31c6ed7275e2e2e770b6b0fbcf11c2/rules/no-anonymous-default-export.js#L41
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAnonymousDefaultExport;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow anonymous functions and classes as the default export
    ///
    /// ### Why is this bad?
    /// Naming default exports improves codebase searchability by ensuring
    /// consistent identifier use for a module's default export, both where it's
    /// declared and where it's imported.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export default class {}
    /// export default function () {}
    /// export default () => {};
    /// module.exports = class {};
    /// module.exports = function () {};
    /// module.exports = () => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// export default class Foo {}
    /// export default function foo () {}
    ///
    /// const foo = () => {};
    /// export default foo;
    ///
    /// module.exports = class Foo {};
    /// module.exports = function foo () {};
    ///
    /// const foo = () => {};
    /// module.exports = foo;
    /// ```
    NoAnonymousDefaultExport,
    unicorn,
    restriction,
);

impl Rule for NoAnonymousDefaultExport {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let problem_node = match node.kind() {
            // ESM: export default
            AstKind::ExportDefaultDeclaration(export_decl) => match &export_decl.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => class_decl
                    .id
                    .as_ref()
                    .map_or(Some((export_decl.span, ErrorNodeKind::Class)), |_| None),
                ExportDefaultDeclarationKind::FunctionDeclaration(function_decl) => function_decl
                    .id
                    .as_ref()
                    .map_or(Some((export_decl.span, ErrorNodeKind::Function)), |_| None),
                ExportDefaultDeclarationKind::ArrowFunctionExpression(_) => {
                    Some((export_decl.span, ErrorNodeKind::Function))
                }
                ExportDefaultDeclarationKind::ParenthesizedExpression(expr) => {
                    let expr = expr.expression.get_inner_expression();
                    match expr {
                        Expression::ClassExpression(class_expr) => class_expr
                            .id
                            .as_ref()
                            .map_or(Some((class_expr.span, ErrorNodeKind::Class)), |_| None),
                        Expression::FunctionExpression(func_expr) => func_expr
                            .id
                            .as_ref()
                            .map_or(Some((func_expr.span, ErrorNodeKind::Function)), |_| None),
                        _ => None,
                    }
                }
                _ => None,
            },
            // CommonJS: module.exports
            AstKind::AssignmentExpression(expr) if is_common_js_export(expr) => match &expr.right {
                Expression::ClassExpression(class_expr) => {
                    class_expr.id.as_ref().map_or(Some((expr.span, ErrorNodeKind::Class)), |_| None)
                }
                Expression::FunctionExpression(function_expr) => function_expr
                    .id
                    .as_ref()
                    .map_or(Some((expr.span, ErrorNodeKind::Function)), |_| None),
                Expression::ArrowFunctionExpression(_) => {
                    Some((expr.span, ErrorNodeKind::Function))
                }
                _ => None,
            },
            _ => None,
        };

        if let Some((span, error_kind)) = problem_node {
            ctx.diagnostic(no_anonymous_default_export_diagnostic(span, error_kind));
        };
    }
}

fn is_common_js_export(expr: &AssignmentExpression) -> bool {
    if let AssignmentTarget::StaticMemberExpression(member_expr) = &expr.left {
        if let Expression::Identifier(object_ident) = &member_expr.object {
            if object_ident.name != "module" {
                return false;
            }
        }

        if member_expr.property.name != "exports" {
            return false;
        }
    }

    true
}

#[derive(Clone, Copy)]
enum ErrorNodeKind {
    Function,
    Class,
}

impl fmt::Display for ErrorNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => "function".fmt(f),
            Self::Class => "class".fmt(f),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"export default class Foo {}",
        r"export default function foo () {}",
        r"const foo = () => {}; export default foo;",
        r"module.exports = class Foo {};",
        r"module.exports = function foo () {};",
        r"const foo = () => {}; module.exports = foo;",
        // TODO: need handle this situation?
        // r"module['exports'] = function foo () {};",
    ];

    let fail = vec![
        r"export default class {}",
        r"export default function () {}",
        r"export default () => {};",
        r"module.exports = class {}",
        r"module.exports = function () {}",
        r"module.exports = () => {}",
        "export default (async function * () {})",
        "export default (class extends class {} {})",
    ];

    Tester::new(NoAnonymousDefaultExport::NAME, NoAnonymousDefaultExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
