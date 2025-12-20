use oxc_ast::{
    AstKind,
    ast::{Expression, PropertyKind, WithClause},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_empty_object_expression};

fn require_module_attributes_diagnostic(span: Span, import_type: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{import_type} with empty attribute list is not allowed."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireModuleAttributes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces non-empty attribute list in import/export statements and import() expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Import attributes are meant to provide metadata about how a module should be loaded
    /// (e.g., `with { type: "json" }`). An empty attribute object provides no information
    /// and should be removed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import foo from 'foo' with {};
    ///
    /// export {foo} from 'foo' with {};
    ///
    /// const foo = await import('foo', {});
    ///
    /// const foo = await import('foo', {with: {}});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import foo from 'foo';
    ///
    /// export {foo} from 'foo';
    ///
    /// const foo = await import('foo');
    ///
    /// const foo = await import('foo');
    /// ```
    RequireModuleAttributes,
    unicorn,
    style,
    pending,
);

impl Rule for RequireModuleAttributes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportExpression(import_expr) => {
                let Some(options) = &import_expr.options else {
                    return;
                };

                let Expression::ObjectExpression(obj_expr) = options.get_inner_expression() else {
                    return;
                };

                if obj_expr.properties.is_empty() {
                    ctx.diagnostic(require_module_attributes_diagnostic(
                        obj_expr.span,
                        "import expression",
                    ));
                    return;
                }

                let empty_with_prop = obj_expr.properties.iter().find_map(|prop| {
                    let obj_prop = prop.as_property()?;
                    if !obj_prop.method
                        && !obj_prop.shorthand
                        && !obj_prop.computed
                        && obj_prop.kind == PropertyKind::Init
                        && obj_prop.key.is_specific_static_name("with")
                        && is_empty_object_expression(&obj_prop.value)
                    {
                        Some(obj_prop)
                    } else {
                        None
                    }
                });

                if let Some(empty_with_prop) = empty_with_prop {
                    let span = empty_with_prop.value.span();
                    ctx.diagnostic(require_module_attributes_diagnostic(span, "import expression"));
                }
            }
            AstKind::ImportDeclaration(decl) => {
                check_with_clause(ctx, decl.with_clause.as_deref(), "import statement");
            }
            AstKind::ExportNamedDeclaration(decl) => {
                check_with_clause(ctx, decl.with_clause.as_deref(), "export statement");
            }
            AstKind::ExportAllDeclaration(decl) => {
                check_with_clause(ctx, decl.with_clause.as_deref(), "export statement");
            }
            _ => (),
        }
    }
}

fn check_with_clause(ctx: &LintContext, with_clause: Option<&WithClause>, import_type: &str) {
    if let Some(with_clause) = with_clause
        && with_clause.with_entries.is_empty()
    {
        ctx.diagnostic(require_module_attributes_diagnostic(with_clause.span, import_type));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import foo from "foo""#,
        r#"export {foo} from "foo""#,
        r#"export * from "foo""#,
        r#"import foo from "foo" with {type: "json"}"#,
        r#"export {foo} from "foo" with {type: "json"}"#,
        r#"export * from "foo" with {type: "json"}"#,
        "export {}",
        r#"import("foo")"#,
        r#"import("foo", {unknown: "unknown"})"#,
        r#"import("foo", {with: {type: "json"}})"#,
        r#"not_import("foo", {})"#,
        r#"not_import("foo", {with:{}})"#,
    ];

    let fail = vec![
        r#"import "foo" with {}"#,
        r#"import foo from "foo" with {}"#,
        r#"export {foo} from "foo" with {}"#,
        r#"export * from "foo" with {}"#,
        r#"export * from "foo"with{}"#,
        r#"export * from "foo"/* comment 1 */with/* comment 2 */{/* comment 3 */}/* comment 4 */"#,
        r#"import("foo", {})"#,
        r#"import("foo", (( {} )))"#,
        r#"import("foo", {},)"#,
        r#"import("foo", {with:{},},)"#,
        r#"import("foo", {with:{}, unknown:"unknown"},)"#,
        r#"import("foo", {"with":{}, unknown:"unknown"},)"#,
        r#"import("foo", {unknown:"unknown", with:{}, },)"#,
        r#"import("foo", {unknown:"unknown", with:{} },)"#,
        r#"import("foo", {unknown:"unknown", with:{}, unknown2:"unknown2", },)"#,
        r#"import("foo"/* comment 1 */, /* comment 2 */{/* comment 3 */}/* comment 4 */,/* comment 5 */)"#,
        r#"import("foo", {/* comment 1 */"with"/* comment 2 */:/* comment 3 */{/* comment 4 */}, }/* comment 5 */,)"#,
    ];

    Tester::new(RequireModuleAttributes::NAME, RequireModuleAttributes::PLUGIN, pass, fail)
        .test_and_snapshot();
}
