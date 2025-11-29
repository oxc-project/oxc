use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_anonymous_default_export_diagnostic(span: Span, msg: &'static str) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(msg).with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoAnonymousDefaultExport {
    /// Allow anonymous array as default export.
    allow_array: bool,
    /// Allow anonymous arrow function as default export.
    allow_arrow_function: bool,
    /// Allow anonymous class as default export.
    allow_anonymous_class: bool,
    /// Allow anonymous function as default export.
    allow_anonymous_function: bool,
    /// Allow anonymous call expression as default export.
    allow_call_expression: bool,
    /// Allow anonymous new expression as default export.
    allow_new: bool,
    /// Allow anonymous literal as default export.
    allow_literal: bool,
    /// Allow anonymous object as default export.
    allow_object: bool,
}

impl Default for NoAnonymousDefaultExport {
    fn default() -> Self {
        Self {
            allow_array: false,
            allow_arrow_function: false,
            allow_anonymous_class: false,
            allow_anonymous_function: false,
            allow_call_expression: true,
            allow_new: false,
            allow_literal: false,
            allow_object: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports if a module's default export is unnamed.
    /// This includes several types of unnamed data types;
    /// literals, object expressions, arrays, anonymous functions, arrow functions,
    /// and anonymous class declarations.
    ///
    /// ### Why is this bad?
    ///
    /// Ensuring that default exports are named helps improve the grepability of
    /// the codebase by encouraging the re-use of the same identifier for
    /// the module's default export at its declaration site and at its import sites.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// export default [];
    /// export default () => {};
    /// export default class {};
    /// export default function() {};
    /// export default foo(bar);
    /// export default 123;
    /// export default {};
    /// export default new Foo();
    /// export default `foo`;
    /// export default /^123/;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = 123;
    /// export default foo;
    /// export default function foo() {};
    /// export default class MyClass {};
    /// export default function foo() {};
    /// export default foo(bar);
    /// /* eslint import/no-anonymous-default-export: ['error', {"allowLiteral": true}] */
    /// export default 123;
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowArray": true}] */
    /// export default []
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowArrowFunction": true}] */
    /// export default () => {};
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowAnonymousClass": true}] */
    /// export default class {};
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowAnonymousFunction": true}] */
    /// export default function() {};
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowObject": true}] */
    /// export default {};
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowNew": true}] */
    /// export default new Foo();
    /// /* eslint import/no-anonymous-default-export: ['error, {"allowCallExpression": true}] */
    /// export default foo(bar);
    /// ```
    ///
    /// By default, all types of anonymous default exports are forbidden,
    /// but any types can be selectively allowed by toggling them on in the options.
    NoAnonymousDefaultExport,
    import,
    style,
    config = NoAnonymousDefaultExport,
);

impl Rule for NoAnonymousDefaultExport {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoAnonymousDefaultExport>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_decl) = node.kind() else {
            return;
        };
        match &export_decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func_decl)
                if !self.allow_anonymous_function && func_decl.id.is_none() =>
            {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Unexpected default export of anonymous function",
                ));
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class_decl)
                if !self.allow_anonymous_class && class_decl.id.is_none() =>
            {
                ctx.diagnostic(no_anonymous_default_export_diagnostic(
                    export_decl.span,
                    "Unexpected default export of anonymous class",
                ));
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
                if let Some(expr) = export_decl.declaration.as_expression()
                    && !self.allow_literal
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
