use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_anonymous_default_export_diagnostic(span: Span, msg: &'static str) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(msg).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAnonymousDefaultExport {
    allow_array: bool,
    allow_arrow_function: bool,
    allow_anonymous_class: bool,
    allow_anonymous_function: bool,
    allow_call_expression: bool,
    allow_new: bool,
    allow_literal: bool,
    allow_object: bool,
}

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
    NoAnonymousDefaultExport,
    import,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoAnonymousDefaultExport {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_array: obj
                .and_then(|v| v.get("allowArray"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_arrow_function: obj
                .and_then(|v| v.get("allowArrowFunction"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_anonymous_class: obj
                .and_then(|v| v.get("allowAnonymousClass"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_anonymous_function: obj
                .and_then(|v| v.get("allowAnonymousFunction"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_call_expression: obj
                .and_then(|v| v.get("allowCallExpression"))
                .and_then(Value::as_bool)
                .unwrap_or(true),
            allow_new: obj
                .and_then(|v| v.get("allowNew"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_literal: obj
                .and_then(|v| v.get("allowLiteral"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_object: obj
                .and_then(|v| v.get("allowObject"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_decl) = node.kind() else {
            return;
        };
        match &export_decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func_decl)
                if !self.allow_anonymous_function =>
            {
                if func_decl.id.is_none() {
                    ctx.diagnostic(no_anonymous_default_export_diagnostic(
                        export_decl.span,
                        "Unexpected default export of anonymous function",
                    ));
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class_decl)
                if !self.allow_anonymous_class =>
            {
                if class_decl.id.is_none() {
                    ctx.diagnostic(no_anonymous_default_export_diagnostic(
                        export_decl.span,
                        "Unexpected default export of anonymous class",
                    ));
                }
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(_)
                if !self.allow_arrow_function =>
            {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Assign arrow function to a variable before exporting as module default",
                ));
            }
            ExportDefaultDeclarationKind::ObjectExpression(_) if !self.allow_object => {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Assign object to a variable before exporting as module default",
                ));
            }
            ExportDefaultDeclarationKind::CallExpression(_) if !self.allow_call_expression => {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Assign call result to a variable before exporting as module default",
                ));
            }
            ExportDefaultDeclarationKind::NewExpression(_) if !self.allow_new => {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Assign instance to a variable before exporting as module default",
                ));
            }
            ExportDefaultDeclarationKind::ArrayExpression(_) if !self.allow_array => {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Assign array to a variable before exporting as module default",
                ));
            }
            _ => {
                if let Some(expr) = export_decl.declaration.as_expression() {
                    if !self.allow_literal
                        && (expr.is_literal() || matches!(expr, Expression::TemplateLiteral(_)))
                    {
                        ctx.diagnostic(no_anonymous_default_export_diagnostic(
                            export_decl.span,
                            "Assign literal to a variable before exporting as module default",
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("export default function foo() {}", None),
        ("export default class MyClass {}", None),
        ("const foo = 123; export default foo", None),
        ("export default foo(bar)", None),
        ("export default []", Some(json!([{ "allowArray": true }]))),
        ("export default 123", Some(json!([{ "allowLiteral": true }]))),
        ("export default () => {}", Some(json!([{ "allowArrowFunction": true }]))),
        ("export default class {}", Some(json!([{ "allowAnonymousClass": true }]))),
        ("export default function() {}", Some(json!([{ "allowAnonymousFunction": true }]))),
        ("export default 'foo'", Some(json!([{ "allowLiteral": true }]))),
        (r"export default `123`", Some(json!([{ "allowLiteral": true }]))),
        (r"export default /^123/", Some(json!([{ "allowLiteral": true }]))),
        ("export default {}", Some(json!([{ "allowObject": true }]))),
        ("export default new Foo()", Some(json!([{ "allowNew": true }]))),
        ("export default foo(bar)", Some(json!([{ "allowCallExpression": true }]))),
        (
            r"
                const foo = 3;
                export { foo as default }
            ",
            None,
        ),
        (
            r"
                const foo = 3;
                export { foo as 'default' }
            ",
            None,
        ),
        ("const foo = 4; export { foo }", None),
        ("export * from './foo'", None),
    ];

    let fail = vec![
        ("export default []", None),
        ("export default () => {}", None),
        ("export default class {}", None),
        ("export default function () {}", None),
        ("export default foo(bar)", Some(json!([{ "allowCallExpression": false }]))),
        ("export default 123", None),
        ("export default {}", None),
        ("export default new Foo()", None),
        (r"export default `foo`", None),
        (r"export default /^123/", None),
    ];

    Tester::new(NoAnonymousDefaultExport::NAME, NoAnonymousDefaultExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
